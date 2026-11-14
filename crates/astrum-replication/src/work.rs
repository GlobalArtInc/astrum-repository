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
use crate::dispatch::{Dispatcher, Target};
use crate::error::{Error, clean};
use crate::filter::{compile, component_name};
use crate::protocol::{MAX_MISSING, Tombstone};
use crate::queue::{self, Item, ReplicationRow};

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

#[derive(Debug)]
pub(crate) enum Failure {
    Remote(Error),
    Local(String),
    Rejected(String),
}

impl Failure {
    pub(crate) fn permanent(&self) -> bool {
        matches!(
            self,
            Failure::Remote(Error::Permanent { .. }) | Failure::Rejected(_)
        )
    }

    pub(crate) fn retry_after(&self) -> Option<Duration> {
        match self {
            Failure::Remote(error) => error.retry_after(),
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> String {
        match self {
            Failure::Remote(error) => clean(&error.to_string()),
            Failure::Local(text) | Failure::Rejected(text) => clean(text),
        }
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Failure::Remote(error)
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

fn matches_filter(pattern: Option<&str>, text: &str) -> Result<bool, Failure> {
    let Some(pattern) = pattern else {
        return Ok(true);
    };
    let regex = compile(pattern)
        .map_err(|reason| Failure::Rejected(format!("a filter of the replication {reason}")))?;
    Ok(regex.is_match(text))
}

fn passes(replication: &ReplicationRow, item: &Item) -> Result<bool, Failure> {
    let name = component_name(&item.namespace, &item.name);
    Ok(matches_filter(replication.name_filter.as_deref(), &name)?
        && matches_filter(replication.version_filter.as_deref(), &item.version)?)
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
        let db = self.content().db();
        let Some(replication) = queue::replication(db, item.replication_id).await? else {
            return Ok(Done::Dropped(DISABLED));
        };
        if replication.remote_id != target.remote.id {
            return Ok(Done::Released);
        }
        if !replication.enabled {
            return Ok(Done::Dropped(DISABLED));
        }
        if !passes(&replication, item)? {
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
                self.send(client, &replication, item, &found, remote, own)
                    .await
            }
            None => self.retract(client, &replication, item).await,
        }
    }

    async fn remote_instance(&self, target: &Target, client: &Client) -> Result<Uuid, Failure> {
        if let Some(known) = target.remote.remote_instance_id {
            return Ok(known);
        }
        let learned = target
            .learned
            .get_or_try_init(|| async {
                let capabilities = client.check().await?;
                let instance = Uuid::parse_str(&capabilities.instance_id).map_err(|_| {
                    Failure::Local("the remote reported an instance id that is no UUID".to_owned())
                })?;
                queue::learned(
                    self.content().db(),
                    &target.remote,
                    instance,
                    &capabilities.version,
                )
                .await?;
                Ok::<Uuid, Failure>(instance)
            })
            .await?;
        Ok(*learned)
    }

    async fn context(&self, repository: &str) -> Result<Ctx, Failure> {
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
        client: &Client,
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
        let first = self.round(client, replication, item, found, &bundle).await;
        let outcome = match first {
            Err(Failure::Remote(Error::BlobsMissing(_))) => {
                self.round(client, replication, item, found, &bundle)
                    .await?
            }
            other => other?,
        };
        let Some(outcome) = outcome else {
            return Ok(Done::Lost);
        };
        if !matches!(outcome.status, Status::Copied | Status::Unchanged) {
            return Err(Failure::Rejected(format!(
                "the remote did not take the component: {:?}",
                outcome.status
            )));
        }
        queue::record_sent(self.content().db(), item, &fingerprint(&bundle), &origin).await?;
        Ok(Done::Sent)
    }

    async fn round(
        &self,
        client: &Client,
        replication: &ReplicationRow,
        item: &Item,
        found: &Component,
        bundle: &Bundle,
    ) -> Result<Option<Outcome>, Failure> {
        if !self
            .upload(client, replication, item, found, bundle)
            .await?
        {
            return Ok(None);
        }
        Ok(Some(
            client
                .commit(&replication.remote_repository, bundle)
                .await?,
        ))
    }

    async fn upload(
        &self,
        client: &Client,
        replication: &ReplicationRow,
        item: &Item,
        found: &Component,
        bundle: &Bundle,
    ) -> Result<bool, Failure> {
        let db = self.content().db();
        let digests = digests_of(bundle);
        let mut lacking: HashSet<String> = HashSet::new();
        for chunk in digests.chunks(MAX_MISSING) {
            lacking.extend(
                client
                    .missing(&replication.remote_repository, chunk)
                    .await?,
            );
        }
        if lacking.is_empty() {
            return Ok(true);
        }
        let rows = store::assets_of(db, found.id).await?;
        let by_digest: HashMap<&str, &Asset> =
            rows.iter().map(|row| (row.sha256.as_str(), row)).collect();
        for digest in digests.iter().filter(|digest| lacking.contains(*digest)) {
            let row = by_digest.get(digest.as_str()).ok_or_else(|| {
                Failure::Local("the component changed while it was being sent".to_owned())
            })?;
            if !queue::renew(db, item.id, self.settings().lease).await? {
                return Ok(false);
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
                .await?;
        }
        Ok(true)
    }

    async fn retract(
        &self,
        client: &Client,
        replication: &ReplicationRow,
        item: &Item,
    ) -> Result<Done, Failure> {
        let db = self.content().db();
        if !replication.propagate_deletes {
            queue::forget_sent(db, item).await?;
            return Ok(Done::Dropped(KEPT));
        }
        let Some(sent) = queue::sent(db, item).await? else {
            return Ok(Done::Dropped(NEVER_SENT));
        };
        let tombstone = Tombstone {
            namespace: item.namespace.clone(),
            name: item.name.clone(),
            version: item.version.clone(),
            fingerprint: sent.fingerprint,
            origin: sent.origin,
        };
        match client
            .tombstone(&replication.remote_repository, &tombstone)
            .await
        {
            Ok(()) => {}
            Err(Error::Permanent {
                status: 404, code, ..
            }) if code == ALREADY_GONE => {}
            Err(error) => return Err(error.into()),
        }
        queue::forget_sent(db, item).await?;
        Ok(Done::Deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replication(name: Option<&str>, version: Option<&str>) -> ReplicationRow {
        ReplicationRow {
            id: Uuid::nil(),
            source_repository: "source".to_owned(),
            remote_id: Uuid::nil(),
            remote_repository: "mirror".to_owned(),
            enabled: true,
            name_filter: name.map(str::to_owned),
            version_filter: version.map(str::to_owned),
            propagate_deletes: false,
        }
    }

    fn item(namespace: &str, name: &str, version: &str) -> Item {
        Item {
            id: 1,
            replication_id: Uuid::nil(),
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            generation: 1,
            attempts: 0,
        }
    }

    #[test]
    fn the_filters_match_the_whole_text_of_the_name_and_of_the_version() {
        let wanted = replication(Some("com\\.acme/lib-.*"), Some("1\\..*"));
        assert!(passes(&wanted, &item("com.acme", "lib-core", "1.4")).unwrap());
        assert!(!passes(&wanted, &item("com.acme", "lib-core", "2.0")).unwrap());
        assert!(!passes(&wanted, &item("org.acme", "lib-core", "1.4")).unwrap());
        assert!(!passes(&wanted, &item("com.acme", "x-lib-core", "1.4")).unwrap());
        let bare = replication(Some("app"), None);
        assert!(passes(&bare, &item("", "app", "7")).unwrap());
        assert!(!passes(&bare, &item("", "app2", "7")).unwrap());
        assert!(passes(&replication(None, None), &item("", "any", "")).unwrap());
    }

    #[test]
    fn a_filter_that_cannot_be_compiled_is_a_final_failure() {
        let broken = replication(Some("(unclosed"), None);
        let failure = passes(&broken, &item("", "app", "1")).unwrap_err();
        assert!(failure.permanent());
    }

    #[test]
    fn only_a_refusal_of_the_remote_or_a_local_rejection_is_final() {
        let permanent = Failure::Remote(Error::Permanent {
            status: 409,
            code: "x".to_owned(),
            message: String::new(),
        });
        let throttled = Failure::Remote(Error::Retryable {
            reason: "busy".to_owned(),
            retry_after: Some(Duration::from_secs(9)),
        });
        assert!(permanent.permanent());
        assert!(Failure::Rejected("no".to_owned()).permanent());
        for temporary in [
            Failure::Remote(Error::BlobsMissing(vec!["a".repeat(64)])),
            Failure::Remote(Error::Refused("blocked".to_owned())),
            Failure::Local("disk".to_owned()),
            throttled,
        ] {
            assert!(!temporary.permanent());
        }
        let throttled = Failure::Remote(Error::Retryable {
            reason: "busy".to_owned(),
            retry_after: Some(Duration::from_secs(9)),
        });
        assert_eq!(throttled.retry_after(), Some(Duration::from_secs(9)));
        assert_eq!(Failure::Local("x".to_owned()).retry_after(), None);
        let long = Failure::Local(format!("a\nb{}", "c".repeat(900)));
        assert_eq!(long.text().chars().count(), 500);
        assert!(!long.text().contains('\n'));
    }
}
