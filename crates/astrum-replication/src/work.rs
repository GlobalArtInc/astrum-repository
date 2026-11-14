use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use astrum_content::store::{self, Asset};
use astrum_content::transfer::{Bundle, Outcome, Status, fingerprint, origin_of};
use astrum_content::{Component, ContentError, Ctx};
use astrum_db::entity::component;
use astrum_security::SecurityModel;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::Client;
use crate::dispatch::Dispatcher;
use crate::error::{Error, clean};
use crate::protocol::{MAX_MISSING, Tombstone};
use crate::queue::{self, Item, ReplicationRow};
use crate::target::Target;

const DISABLED: &str = "the replication is disabled or gone";
const FILTERED: &str = "the component is outside the filters of the replication";
const FROM_REMOTE: &str = "the component came from the remote";
const EMPTY: &str = "the component has no assets";
const KEPT: &str = "deletes are not propagated by this replication";
const NEVER_SENT: &str = "the component was never sent";
const ALREADY_GONE: &str = "component_not_found";

#[derive(Debug)]
pub(crate) enum Done {
    Sent,
    Deleted,
    Dropped(&'static str),
    Released,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Handshake,
    Negotiate,
    Upload,
    Commit,
    Retract,
}

#[derive(Debug)]
pub(crate) enum Failure {
    Down(Error),
    Stuck(Error),
    Remote(Error),
    Local(String),
    Rejected(String),
}

fn unreachable_for(phase: Phase, error: &Error) -> bool {
    if phase == Phase::Handshake {
        return true;
    }
    match error {
        Error::Retryable { status: None, .. } => true,
        Error::Retryable {
            status: Some(408 | 429),
            ..
        } => true,
        Error::Retryable { .. } => phase == Phase::Negotiate,
        Error::Permanent { status: 401, .. } => true,
        Error::Refused(_) => true,
        _ => false,
    }
}

impl Failure {
    pub(crate) fn blame(phase: Phase, error: Error) -> Self {
        if !unreachable_for(phase, &error) {
            return Failure::Remote(error);
        }
        let silent = matches!(error, Error::Retryable { status: None, .. });
        match silent && matches!(phase, Phase::Upload | Phase::Commit | Phase::Retract) {
            true => Failure::Stuck(error),
            false => Failure::Down(error),
        }
    }

    pub(crate) fn permanent(&self) -> bool {
        matches!(
            self,
            Failure::Remote(Error::Permanent { .. }) | Failure::Rejected(_)
        )
    }

    pub(crate) fn retry_after(&self) -> Option<Duration> {
        match self {
            Failure::Remote(error) | Failure::Down(error) | Failure::Stuck(error) => {
                error.retry_after()
            }
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> String {
        match self {
            Failure::Remote(error) | Failure::Down(error) | Failure::Stuck(error) => {
                clean(&error.to_string())
            }
            Failure::Local(text) | Failure::Rejected(text) => clean(text),
        }
    }
}

impl From<DbErr> for Failure {
    fn from(error: DbErr) -> Self {
        Failure::Local(error.to_string())
    }
}

impl From<ContentError> for Failure {
    fn from(error: ContentError) -> Self {
        Failure::Local(error.to_string())
    }
}

impl From<astrum_blob::Error> for Failure {
    fn from(error: astrum_blob::Error) -> Self {
        Failure::Local(error.to_string())
    }
}

enum Pause {
    Lost,
    Stale,
}

enum Round {
    Committed(Outcome),
    Paused(Pause),
}

fn digests_of(bundle: &Bundle) -> Vec<String> {
    let mut digests: Vec<String> = bundle
        .assets
        .iter()
        .map(|asset| asset.sha256.clone())
        .collect();
    digests.sort();
    digests.dedup();
    digests
}

impl Dispatcher {
    pub(crate) async fn transfer(&self, target: &Target, item: &Item) -> Result<Done, Failure> {
        let db = &self.db();
        let Some(replication) = queue::replication(db, item.replication_id).await? else {
            return Ok(Done::Dropped(DISABLED));
        };
        if replication.remote_id != target.remote.id {
            return Ok(Done::Released);
        }
        if !replication.enabled {
            return Ok(Done::Dropped(DISABLED));
        }
        if !target.passes(&replication, item)? {
            return Ok(Done::Dropped(FILTERED));
        }
        let client = target.client()?;
        let remote = self.remote_instance(target, client).await?;
        let found = component::Entity::find()
            .filter(component::Column::Repository.eq(replication.source_repository.as_str()))
            .filter(component::Column::Namespace.eq(item.namespace.as_str()))
            .filter(component::Column::Name.eq(item.name.as_str()))
            .filter(component::Column::Version.eq(item.version.as_str()))
            .one(db)
            .await?;
        match found {
            Some(found) => {
                let own = self.instance().await?;
                self.send(target, &replication, item, &found, remote, own)
                    .await
            }
            None => self.retract(target, &replication, item, remote).await,
        }
    }

    pub(crate) async fn remote_instance(
        &self,
        target: &Target,
        client: &Client,
    ) -> Result<Uuid, Failure> {
        let learned = target
            .learned
            .get_or_try_init(|| async {
                let capabilities = client
                    .check()
                    .await
                    .map_err(|error| Failure::blame(Phase::Handshake, error))?;
                let instance = Uuid::parse_str(&capabilities.instance_id).map_err(|_| {
                    Failure::Local("the remote reported an instance id that is no UUID".to_owned())
                })?;
                if target.remote.remote_instance_id != Some(instance) {
                    queue::learned(&self.db(), &target.remote, instance, &capabilities.version)
                        .await?;
                }
                Ok::<Uuid, Failure>(instance)
            })
            .await?;
        Ok(*learned)
    }

    pub(crate) async fn context(&self, repository: &str) -> Result<Ctx, Failure> {
        let found = self
            .content()
            .repository(repository)
            .await?
            .ok_or_else(|| {
                Failure::Rejected(format!("the source repository {repository} is gone"))
            })?;
        Ok(self.content().context(
            found,
            Arc::new(SecurityModel::default()),
            None,
            String::new(),
            String::new(),
        ))
    }

    async fn send(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        found: &Component,
        remote: Uuid,
        own: Uuid,
    ) -> Result<Done, Failure> {
        let origin = origin_of(&found.attributes)
            .map(str::to_owned)
            .unwrap_or_else(|| own.to_string());
        if origin.eq_ignore_ascii_case(&remote.to_string()) {
            return Ok(Done::Dropped(FROM_REMOTE));
        }
        let ctx = self.context(&replication.source_repository).await?;
        let format = self.content().format(ctx.format_id()).ok_or_else(|| {
            Failure::Rejected(format!("the format {} is not served", ctx.format_id()))
        })?;
        let transfer = format.transfer().ok_or_else(|| {
            Failure::Rejected(format!(
                "the format {} takes no part in replication",
                ctx.format_id()
            ))
        })?;
        let mut bundle = transfer.bundle(&ctx, found).await?;
        if bundle.assets.is_empty() {
            return Ok(Done::Dropped(EMPTY));
        }
        bundle.origin = Some(origin.clone());
        let first = self.round(target, replication, item, found, &bundle).await;
        let round = match first {
            Err(Failure::Remote(Error::BlobsMissing(_))) => {
                self.round(target, replication, item, found, &bundle)
                    .await?
            }
            other => other?,
        };
        let outcome = match round {
            Round::Committed(outcome) => outcome,
            Round::Paused(Pause::Lost) => return Ok(Done::Lost),
            Round::Paused(Pause::Stale) => return Ok(Done::Released),
        };
        if !matches!(outcome.status, Status::Copied | Status::Unchanged) {
            return Err(Failure::Rejected(format!(
                "the remote did not take the component: {:?}",
                outcome.status
            )));
        }
        if replication.propagate_deletes {
            queue::record_sent(&self.db(), item, &fingerprint(&bundle), &origin).await?;
        }
        Ok(Done::Sent)
    }

    async fn round(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        found: &Component,
        bundle: &Bundle,
    ) -> Result<Round, Failure> {
        if let Some(pause) = self
            .upload(target, replication, item, found, bundle)
            .await?
        {
            return Ok(Round::Paused(pause));
        }
        if !target.fresh(&self.db(), self.settings().fresh).await? {
            return Ok(Round::Paused(Pause::Stale));
        }
        let committed = target
            .client()?
            .commit(&replication.remote_repository, bundle)
            .await
            .map_err(|error| Failure::blame(Phase::Commit, error))?;
        Ok(Round::Committed(committed))
    }

    async fn upload(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        found: &Component,
        bundle: &Bundle,
    ) -> Result<Option<Pause>, Failure> {
        let db = &self.db();
        let client = target.client()?;
        let digests = digests_of(bundle);
        let mut lacking: HashSet<String> = HashSet::new();
        for chunk in digests.chunks(MAX_MISSING) {
            let asked = client
                .missing(&replication.remote_repository, chunk)
                .await
                .map_err(|error| Failure::blame(Phase::Negotiate, error))?;
            lacking.extend(asked);
        }
        if lacking.is_empty() {
            return Ok(None);
        }
        let rows = store::assets_of(db, found.id).await?;
        let by_digest: HashMap<&str, &Asset> =
            rows.iter().map(|row| (row.sha256.as_str(), row)).collect();
        for digest in digests.iter().filter(|digest| lacking.contains(*digest)) {
            let row = by_digest.get(digest.as_str()).ok_or_else(|| {
                Failure::Local("the component changed while it was being sent".to_owned())
            })?;
            if !queue::renew(db, item.id, self.settings().lease).await? {
                return Ok(Some(Pause::Lost));
            }
            if !target.fresh(db, self.settings().fresh).await? {
                return Ok(Some(Pause::Stale));
            }
            let size = u64::try_from(row.size)
                .map_err(|_| Failure::Local("an asset has a negative size".to_owned()))?;
            let blobs = self.content().blob_stores().get(&row.blob_store).await?;
            let content = blobs.read(row.blob_id.into()).await?.ok_or_else(|| {
                Failure::Local(format!(
                    "the blob of {} is missing from the blob store {}",
                    row.path, row.blob_store
                ))
            })?;
            client
                .put_blob(&replication.remote_repository, digest, size, content.stream)
                .await
                .map_err(|error| Failure::blame(Phase::Upload, error))?;
            self.counters().uploaded(replication.id, size);
        }
        Ok(None)
    }

    async fn retract(
        &self,
        target: &Target,
        replication: &ReplicationRow,
        item: &Item,
        remote: Uuid,
    ) -> Result<Done, Failure> {
        let db = &self.db();
        if !replication.propagate_deletes {
            queue::forget_sent(db, item).await?;
            return Ok(Done::Dropped(KEPT));
        }
        let Some(sent) = queue::sent(db, item).await? else {
            return Ok(Done::Dropped(NEVER_SENT));
        };
        if sent.origin.eq_ignore_ascii_case(&remote.to_string()) {
            queue::forget_sent(db, item).await?;
            return Ok(Done::Dropped(FROM_REMOTE));
        }
        if !target.fresh(db, self.settings().fresh).await? {
            return Ok(Done::Released);
        }
        let tombstone = Tombstone {
            namespace: item.namespace.clone(),
            name: item.name.clone(),
            version: item.version.clone(),
            fingerprint: sent.fingerprint,
            origin: sent.origin,
        };
        match target
            .client()?
            .tombstone(&replication.remote_repository, &tombstone)
            .await
        {
            Ok(()) => {}
            Err(Error::Permanent {
                status: 404, code, ..
            }) if code == ALREADY_GONE => {}
            Err(error) => return Err(Failure::blame(Phase::Retract, error)),
        }
        queue::forget_sent(db, item).await?;
        Ok(Done::Deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retryable(status: Option<u16>) -> Error {
        Error::Retryable {
            reason: "busy".to_owned(),
            retry_after: Some(Duration::from_secs(9)),
            status,
        }
    }

    fn permanent(status: u16) -> Error {
        Error::Permanent {
            status,
            code: "x".to_owned(),
            message: String::new(),
        }
    }

    #[test]
    fn the_handshake_and_the_address_are_the_remotes_and_most_else_is_the_items() {
        use Phase::*;
        let down = |phase, error| {
            matches!(
                Failure::blame(phase, error),
                Failure::Down(_) | Failure::Stuck(_)
            )
        };
        for error in [
            retryable(None),
            retryable(Some(503)),
            permanent(401),
            permanent(403),
            permanent(404),
            permanent(405),
            permanent(409),
            Error::Refused("redirected".to_owned()),
        ] {
            assert!(down(Handshake, error), "handshake");
        }
        for phase in [Negotiate, Upload, Commit, Retract] {
            assert!(down(phase, retryable(None)), "{phase:?}");
            assert!(down(phase, retryable(Some(429))), "{phase:?}");
            assert!(down(phase, retryable(Some(408))), "{phase:?}");
            assert!(down(phase, permanent(401)), "{phase:?}");
            assert!(
                down(phase, Error::Refused("blocked".to_owned())),
                "{phase:?}"
            );
            assert!(!down(phase, permanent(403)), "{phase:?}");
            assert!(!down(phase, permanent(404)), "{phase:?}");
            assert!(!down(phase, permanent(409)), "{phase:?}");
            assert!(!down(phase, permanent(422)), "{phase:?}");
            assert!(
                !down(phase, Error::BlobsMissing(vec!["a".repeat(64)])),
                "{phase:?}"
            );
        }
        assert!(down(Negotiate, retryable(Some(503))));
        for phase in [Upload, Commit, Retract] {
            assert!(!down(phase, retryable(Some(503))), "{phase:?}");
        }
    }

    #[test]
    fn only_silence_in_the_phases_that_carry_the_item_counts_against_the_item() {
        use Phase::*;
        let stuck = |phase, error| matches!(Failure::blame(phase, error), Failure::Stuck(_));
        for phase in [Upload, Commit, Retract] {
            assert!(stuck(phase, retryable(None)), "{phase:?}");
            assert!(!stuck(phase, retryable(Some(429))), "{phase:?}");
            assert!(!stuck(phase, permanent(401)), "{phase:?}");
            assert!(
                !stuck(phase, Error::Refused("blocked".to_owned())),
                "{phase:?}"
            );
        }
        for phase in [Handshake, Negotiate] {
            assert!(!stuck(phase, retryable(None)), "{phase:?}");
        }
    }

    #[test]
    fn only_a_refusal_of_the_remote_or_a_local_rejection_is_final() {
        assert!(Failure::Remote(permanent(409)).permanent());
        assert!(Failure::Rejected("no".to_owned()).permanent());
        for temporary in [
            Failure::Remote(Error::BlobsMissing(vec!["a".repeat(64)])),
            Failure::Remote(Error::Refused("blocked".to_owned())),
            Failure::Remote(retryable(Some(503))),
            Failure::Down(permanent(401)),
            Failure::Stuck(retryable(None)),
            Failure::Local("disk".to_owned()),
        ] {
            assert!(!temporary.permanent());
        }
        assert_eq!(
            Failure::Down(retryable(None)).retry_after(),
            Some(Duration::from_secs(9))
        );
        assert_eq!(Failure::Local("x".to_owned()).retry_after(), None);
        let long = Failure::Local(format!("a\nb{}", "c".repeat(900)));
        assert_eq!(long.text().chars().count(), 500);
        assert!(!long.text().contains('\n'));
    }
}
