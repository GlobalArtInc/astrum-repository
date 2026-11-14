use std::sync::atomic::Ordering;

use astrum_blob::Written;
use astrum_content::replicated::{
    self, Committed, Reason, Refusal as Refused, Removal, check_bundle_shape, instance,
};
use astrum_content::staging::{self, Counted, MAX_BLOB_BYTES, Staged};
use astrum_content::store;
use astrum_content::transfer::{Bundle, REPLICATION_VIA, Status, fingerprint};
use astrum_content::{ContentError, Ctx};
use astrum_db::audit::{self, Event};
use futures_util::StreamExt;
use serde_json::json;
use uuid::Uuid;

use crate::Client;
use crate::dispatch::Dispatcher;
use crate::error::Error;
use crate::protocol::component_key;
use crate::queue::{self, Item, ReplicationRow};
use crate::target::Target;
use crate::work::{Done, Failure, Pause, Phase, SYSTEM, digests_of};

const FETCH: Phase = Phase::Negotiate;
const DOWNLOAD: Phase = Phase::Upload;
const AUDIT_DOMAIN: &str = "repository.content";
const GONE: &str = "component_not_found";
const FROM_HERE: &str = "the component came from this server";
const EMPTY: &str = "the component has no assets";
const KEPT: &str = "deletes are not propagated by this replication";
const NEVER_PULLED: &str = "the component was never pulled";
const MISSING_TWICE: &str = "the staged blobs of the component went missing twice";

fn content_failure(error: ContentError) -> Failure {
    match error {
        ContentError::Conflict(text)
        | ContentError::BadRequest(text)
        | ContentError::NotFound(text) => Failure::Rejected(text),
        other => Failure::Local(other.to_string()),
    }
}

fn refusal_failure(refusal: Refused) -> Failure {
    match refusal {
        Refused::Content(error) => content_failure(error),
        Refused::Status {
            reason: Reason::TooMany,
            message,
            ..
        } => Failure::Local(message),
        Refused::Status { message, .. } => Failure::Rejected(message),
        Refused::Missing(missing) => Failure::Local(format!(
            "{} staged blob(s) of the component are gone",
            missing.len()
        )),
    }
}

impl Dispatcher {
    async fn audit_pull(&self, event: Event) {
        if let Err(error) = audit::record(&self.db(), event.by(SYSTEM)).await {
            tracing::error!(%error, "an audit event was not recorded");
        }
    }

    pub(crate) async fn pull(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
    ) -> Result<Done, Failure> {
        let client = target.client()?;
        let remote = self.remote_instance(target, client).await?;
        let ctx = self.context(&replication.source_repository).await?;
        replicated::require_hosted(&ctx).map_err(refusal_failure)?;
        replicated::format_of(&ctx).map_err(refusal_failure)?;
        if replicated::read_only(&ctx) {
            return Err(Failure::Rejected(format!(
                "the repository {} is read-only",
                ctx.name()
            )));
        }
        if !ctx.repository.online {
            return Err(Failure::Local(format!(
                "the repository {} is offline",
                ctx.name()
            )));
        }
        let fetched = client
            .bundle(
                &replication.remote_repository,
                &item.namespace,
                &item.name,
                &item.version,
            )
            .await;
        match fetched {
            Ok(bundle) => {
                self.take(target, replication, item, &ctx, bundle, remote)
                    .await
            }
            Err(Error::Permanent {
                status: 404, code, ..
            }) if code == GONE => self.withdraw(target, replication, item, &ctx).await,
            Err(error) => Err(Failure::blame(FETCH, error)),
        }
    }

    async fn take(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        ctx: &Ctx,
        mut bundle: Bundle,
        remote: Uuid,
    ) -> Result<Done, Failure> {
        if bundle.assets.is_empty() {
            return Ok(Done::Dropped(EMPTY));
        }
        if bundle.format != ctx.format_id() {
            return Err(Failure::Rejected(format!(
                "the remote bundle has the format {}, the repository {} holds {}",
                bundle.format,
                ctx.name(),
                ctx.format_id()
            )));
        }
        if (&bundle.namespace, &bundle.name, &bundle.version)
            != (&item.namespace, &item.name, &item.version)
        {
            return Err(Failure::Rejected(
                "the remote answered with another component than the one asked for".to_owned(),
            ));
        }
        check_bundle_shape(&bundle).map_err(refusal_failure)?;
        let origin = match bundle.origin.as_deref() {
            Some(origin) => instance(origin).map_err(refusal_failure)?,
            None => remote.to_string(),
        };
        if origin.eq_ignore_ascii_case(&self.instance().await?.to_string()) {
            return Ok(Done::Dropped(FROM_HERE));
        }
        bundle.origin = Some(origin.clone());
        let conflict = replicated::conflict_for(ctx, true);
        let mut committed = None;
        for _ in 0..2 {
            if let Some(pause) = self.fetch(target, replication, item, ctx, &bundle).await? {
                return Ok(match pause {
                    Pause::Lost => Done::Lost,
                    Pause::Stale => Done::Released,
                });
            }
            match replicated::commit(ctx, bundle.clone(), Some(remote.to_string()), conflict).await
            {
                Ok(done) => {
                    committed = Some(done);
                    break;
                }
                Err(Refused::Missing(_)) => continue,
                Err(refusal) => return Err(refusal_failure(refusal)),
            }
        }
        let Committed { outcome, .. } =
            committed.ok_or_else(|| Failure::Local(MISSING_TWICE.to_owned()))?;
        if !matches!(outcome.status, Status::Copied | Status::Unchanged) {
            return Err(Failure::Rejected(format!(
                "the repository did not take the component: {:?}",
                outcome.status
            )));
        }
        if outcome.status == Status::Copied {
            let event = Event::new(AUDIT_DOMAIN, "replicated", ctx.name()).with(json!({
                "repository": ctx.name(),
                "component": component_key(&bundle.namespace, &bundle.name, &bundle.version),
                "assets": outcome.assets.len(),
                "bytes": outcome.bytes_copied,
                "origin": origin,
                "via": REPLICATION_VIA,
            }));
            self.audit_pull(event).await;
        }
        if replication.propagate_deletes {
            queue::record_sent(&self.db(), item, &fingerprint(&bundle), &origin).await?;
        }
        Ok(Done::Sent)
    }

    async fn fetch(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        ctx: &Ctx,
        bundle: &Bundle,
    ) -> Result<Option<Pause>, Failure> {
        let db = &self.db();
        let client = target.client()?;
        let lacking = staging::absent(db, ctx.name(), digests_of(bundle)).await?;
        for digest in lacking {
            if !queue::renew(db, item.id, self.settings().lease).await? {
                return Ok(Some(Pause::Lost));
            }
            if !target.fresh(db, self.settings().fresh).await? {
                return Ok(Some(Pause::Stale));
            }
            let size = bundle
                .assets
                .iter()
                .find(|asset| asset.sha256 == digest)
                .map(|asset| asset.size)
                .ok_or_else(|| {
                    Failure::Local("a digest to fetch is not in the bundle".to_owned())
                })?;
            self.download(client, replication, ctx, &digest, size)
                .await?;
        }
        Ok(None)
    }

    async fn download(
        &self,
        client: &Client,
        replication: &ReplicationRow,
        ctx: &Ctx,
        sha256: &str,
        size: u64,
    ) -> Result<(), Failure> {
        if size > MAX_BLOB_BYTES {
            return Err(Failure::Rejected(format!(
                "the bundle names a blob of {size} bytes, a replicated blob may hold at most {MAX_BLOB_BYTES}"
            )));
        }
        staging::staged_room(&self.db(), ctx.name())
            .await
            .map_err(refusal_failure)?;
        let download = client
            .blob(&replication.remote_repository, sha256)
            .await
            .map_err(|error| Failure::blame(DOWNLOAD, error))?;
        if download.length.is_some_and(|length| length != size) {
            return Err(Failure::Rejected(format!(
                "the remote announced another size for the blob {sha256} than the bundle names"
            )));
        }
        let (stream, failures) = download.into_parts();
        let (counted, exceeded) = Counted::new(stream, size);
        let written = match store::write_blob(ctx, counted.boxed()).await {
            Ok(written) => written,
            Err(error) => {
                if let Some(error) = failures.take() {
                    return Err(Failure::blame(DOWNLOAD, error));
                }
                if exceeded.load(Ordering::Relaxed) {
                    return Err(Failure::Rejected(format!(
                        "the remote sent more than the {size} bytes the bundle names for the blob {sha256}"
                    )));
                }
                return Err(content_failure(error));
            }
        };
        self.stage_blob(replication, ctx, sha256, size, written)
            .await
    }

    async fn stage_blob(
        &self,
        replication: &ReplicationRow,
        ctx: &Ctx,
        sha256: &str,
        size: u64,
        written: Written,
    ) -> Result<(), Failure> {
        if written.digests.sha256 != sha256 || written.size != size {
            staging::discard(self.content(), &written).await;
            return Err(Failure::Rejected(format!(
                "the remote sent a blob of {} bytes with the SHA-256 {}, not the {size} bytes and {sha256} the bundle names",
                written.size, written.digests.sha256
            )));
        }
        let staged =
            match staging::stage(self.content(), ctx.name(), &written, &ctx.uploader()).await {
                Ok(staged) => staged,
                Err(refusal) => {
                    staging::discard(self.content(), &written).await;
                    return Err(refusal_failure(refusal));
                }
            };
        if staged == Staged::Present {
            staging::discard(self.content(), &written).await;
        }
        self.counters().uploaded(replication.id, size);
        Ok(())
    }

    async fn withdraw(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        ctx: &Ctx,
    ) -> Result<Done, Failure> {
        let db = &self.db();
        if !replication.propagate_deletes {
            queue::forget_sent(db, item).await?;
            return Ok(Done::Dropped(KEPT));
        }
        let Some(sent) = queue::sent(db, item).await? else {
            return Ok(Done::Dropped(NEVER_PULLED));
        };
        if !target.fresh(db, self.settings().fresh).await? {
            return Ok(Done::Released);
        }
        let removal = Removal {
            namespace: &item.namespace,
            name: &item.name,
            version: &item.version,
            fingerprint: &sent.fingerprint,
            origin: &sent.origin,
        };
        match replicated::delete(ctx, &removal).await {
            Ok(origin) => {
                let event = Event::new(AUDIT_DOMAIN, "deleted", ctx.name()).with(json!({
                    "repository": ctx.name(),
                    "component": component_key(&item.namespace, &item.name, &item.version),
                    "origin": origin,
                    "via": REPLICATION_VIA,
                }));
                self.audit_pull(event).await;
            }
            Err(Refused::Status { code, .. }) if code == GONE => {}
            Err(refusal) => return Err(refusal_failure(refusal)),
        }
        queue::forget_sent(db, item).await?;
        Ok(Done::Deleted)
    }
}
