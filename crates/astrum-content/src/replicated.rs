use std::sync::Arc;

use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryFilter,
    Statement, TransactionTrait, Value,
};
use uuid::Uuid;

use crate::staging::{self, StagedSource};
use crate::store::{self, PutAsset};
use crate::transfer::{
    Bundle, Conflict, MAX_ASSETS, Outcome, REPLICATED, Terms, Transfer, fingerprint, origin_of,
};
use crate::{ContentError, Ctx, Format};

const SET_CONTEXT: &str =
    "SELECT set_config('astrum.action', $1, true), set_config('astrum.initiator', $2, true)";

const LOCK_COMPONENT: &str = "SELECT id, attributes FROM component \
     WHERE repository = $1 AND namespace = $2 AND name = $3 AND version = $4 FOR UPDATE";

const COMPONENT_BLOBS: &str = "SELECT blob_store, blob_id FROM asset WHERE component_id = $1";

const ASSET_ROWS: &str = "SELECT path, sha256, size FROM asset WHERE component_id = $1";

const DELETE_COMPONENT: &str = "DELETE FROM component WHERE id = $1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    BadRequest,
    Conflict,
    NotFound,
    TooLarge,
    TooMany,
    Unprocessable,
}

#[derive(Debug)]
pub enum Refusal {
    Content(ContentError),
    Status {
        reason: Reason,
        code: &'static str,
        message: String,
    },
    Missing(Vec<String>),
}

impl From<ContentError> for Refusal {
    fn from(error: ContentError) -> Self {
        Refusal::Content(error)
    }
}

impl From<DbErr> for Refusal {
    fn from(error: DbErr) -> Self {
        Refusal::Content(error.into())
    }
}

pub fn refuse(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::Status {
        reason: Reason::BadRequest,
        code,
        message: message.into(),
    }
}

fn conflict(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::Status {
        reason: Reason::Conflict,
        code,
        message: message.into(),
    }
}

pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn instance(text: &str) -> Result<String, Refusal> {
    Uuid::parse_str(text)
        .map(|id| id.to_string())
        .map_err(|_| refuse("bad_instance", "an instance id is not a UUID"))
}

pub fn require_hosted(ctx: &Ctx) -> Result<(), Refusal> {
    match ctx.kind() {
        RepositoryType::Hosted => Ok(()),
        _ => Err(refuse(
            "not_hosted",
            format!(
                "{} is not a hosted repository and takes no part in the replication of components",
                ctx.name()
            ),
        )),
    }
}

pub fn replicable(format: &dyn Format) -> Result<(), String> {
    let transfer = format
        .transfer()
        .ok_or_else(|| format!("the format {} takes no part in replication", format.id()))?;
    transfer.replicable().map_err(str::to_owned)
}

pub fn format_of(ctx: &Ctx) -> Result<Arc<dyn Format>, Refusal> {
    let format = ctx.content.format(ctx.format_id()).ok_or_else(|| {
        refuse(
            "unsupported_format",
            format!(
                "the format {} takes no part in replication",
                ctx.format_id()
            ),
        )
    })?;
    replicable(format.as_ref()).map_err(|reason| refuse("unsupported_format", reason))?;
    Ok(format)
}

pub fn transfer_of(format: &Arc<dyn Format>) -> Result<&dyn Transfer, ContentError> {
    format.transfer().ok_or_else(|| {
        ContentError::internal(anyhow::anyhow!(
            "the format {} lost its transfer",
            format.id()
        ))
    })
}

pub fn read_only(ctx: &Ctx) -> bool {
    PutAsset::hosted(ctx, "").write_policy == Some(WritePolicy::Deny)
}

pub fn conflict_for(ctx: &Ctx, may_replace: bool) -> Conflict {
    let policy = PutAsset::hosted(ctx, "").write_policy;
    match (policy, may_replace) {
        (Some(WritePolicy::Allow), true) => Conflict::Replace,
        _ => Conflict::Fail,
    }
}

pub fn check_bundle_shape(bundle: &Bundle) -> Result<(), Refusal> {
    if bundle.assets.is_empty() || bundle.assets.len() > MAX_ASSETS {
        return Err(refuse(
            "bad_bundle",
            format!("a bundle holds between 1 and {MAX_ASSETS} assets"),
        ));
    }
    match bundle
        .assets
        .iter()
        .position(|asset| !valid_sha256(&asset.sha256))
    {
        Some(position) => Err(refuse(
            "bad_digest",
            format!(
                "the SHA-256 of the asset at position {position} is not 64 lowercase hexadecimal characters"
            ),
        )),
        None => Ok(()),
    }
}

fn origin_for(bundle: Option<&str>, sender: Option<String>) -> Result<String, Refusal> {
    match (bundle, sender) {
        (Some(origin), _) => instance(origin),
        (None, Some(sender)) => Ok(sender),
        (None, None) => Err(refuse(
            "origin_missing",
            "the bundle names no origin and the request names no sender instance",
        )),
    }
}

pub struct Committed {
    pub outcome: Outcome,
    pub origin: String,
}

pub async fn commit(
    ctx: &Ctx,
    mut bundle: Bundle,
    sender: Option<String>,
    conflict: Conflict,
) -> Result<Committed, Refusal> {
    require_hosted(ctx)?;
    let format = format_of(ctx)?;
    check_bundle_shape(&bundle)?;
    let origin = origin_for(bundle.origin.as_deref(), sender)?;
    bundle.origin = Some(origin.clone());
    let digests: Vec<String> = bundle
        .assets
        .iter()
        .map(|asset| asset.sha256.clone())
        .collect();
    let lacking = staging::absent(ctx.db(), ctx.name(), digests.clone()).await?;
    if !lacking.is_empty() {
        return Err(Refusal::Missing(lacking));
    }
    let source = Arc::new(StagedSource::new(ctx.content.clone(), ctx.name()));
    let task = {
        let (ctx, source) = (ctx.clone(), source.clone());
        tokio::spawn(async move {
            let transfer = transfer_of(&format)?;
            let terms = Terms::new(conflict, REPLICATED);
            let outcome = transfer
                .receive(&ctx, &bundle, source.as_ref(), terms)
                .await?;
            staging::release(&ctx, &digests).await;
            Ok::<Outcome, ContentError>(outcome)
        })
    };
    let outcome = match task.await.map_err(ContentError::internal)? {
        Ok(outcome) => outcome,
        Err(error) => {
            let lacked = source.lacked();
            return Err(match lacked.is_empty() {
                true => error.into(),
                false => Refusal::Missing(lacked),
            });
        }
    };
    Ok(Committed { outcome, origin })
}

#[derive(FromQueryResult)]
struct Locked {
    id: i64,
    attributes: serde_json::Value,
}

#[derive(FromQueryResult)]
struct Signed {
    path: String,
    sha256: String,
    size: i64,
}

#[derive(FromQueryResult)]
struct Blob {
    blob_store: String,
    blob_id: Uuid,
}

type Signature = Vec<(String, String, i64)>;

fn signature(mut rows: Signature) -> Signature {
    for row in &mut rows {
        row.1.make_ascii_lowercase();
    }
    rows.sort();
    rows
}

fn signature_of(bundle: &Bundle) -> Signature {
    signature(
        bundle
            .assets
            .iter()
            .map(|asset| (asset.path.clone(), asset.sha256.clone(), asset.size as i64))
            .collect(),
    )
}

fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

pub fn component_gone() -> Refusal {
    Refusal::Status {
        reason: Reason::NotFound,
        code: "component_not_found",
        message: "the component is not in the repository".to_owned(),
    }
}

fn not_replicated() -> Refusal {
    conflict(
        "origin_mismatch",
        "the component was not replicated from the sender, so a tombstone does not apply to it",
    )
}

fn changed_since() -> Refusal {
    conflict(
        "fingerprint_mismatch",
        "the component changed after the sender last saw it",
    )
}

pub struct Removal<'a> {
    pub namespace: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub fingerprint: &'a str,
    pub origin: &'a str,
}

pub async fn delete(ctx: &Ctx, request: &Removal<'_>) -> Result<String, Refusal> {
    require_hosted(ctx)?;
    let format = format_of(ctx)?;
    let origin = instance(request.origin)?;
    if !valid_sha256(request.fingerprint) {
        return Err(refuse(
            "bad_digest",
            "the fingerprint is not 64 lowercase hexadecimal characters",
        ));
    }
    let found = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Namespace.eq(request.namespace))
        .filter(component::Column::Name.eq(request.name))
        .filter(component::Column::Version.eq(request.version))
        .one(ctx.db())
        .await?
        .ok_or_else(component_gone)?;
    let bundle = transfer_of(&format)?.bundle(ctx, &found).await?;
    let refusal = match origin_of(&found.attributes) == Some(origin.as_str()) {
        false => Some(not_replicated()),
        true => (fingerprint(&bundle) != request.fingerprint).then(changed_since),
    };
    if let Some(refusal) = refusal {
        let present = component::Entity::find_by_id(found.id)
            .one(ctx.db())
            .await?
            .is_some();
        return Err(match present {
            true => refusal,
            false => component_gone(),
        });
    }
    let seen = signature_of(&bundle);
    let txn = ctx.db().begin().await?;
    txn.query_one(statement(
        SET_CONTEXT,
        vec![REPLICATED.into(), ctx.uploader().into()],
    ))
    .await?;
    if !store::lock_repository(&txn, ctx.name()).await? {
        return Err(component_gone());
    }
    let locked = Locked::find_by_statement(statement(
        LOCK_COMPONENT,
        vec![
            ctx.name().into(),
            request.namespace.into(),
            request.name.into(),
            request.version.into(),
        ],
    ))
    .one(&txn)
    .await?
    .ok_or_else(component_gone)?;
    if locked.id != found.id || origin_of(&locked.attributes) != Some(origin.as_str()) {
        return Err(not_replicated());
    }
    store::lock_usage(&txn, ctx.name()).await?;
    let held = Signed::find_by_statement(statement(ASSET_ROWS, vec![locked.id.into()]))
        .all(&txn)
        .await?
        .into_iter()
        .map(|row| (row.path, row.sha256, row.size))
        .collect();
    if signature(held) != seen {
        return Err(changed_since());
    }
    let assets = store::assets_of(&txn, locked.id).await?;
    if let Some(reason) = store::protection(
        &txn,
        &ctx.content,
        ctx.name(),
        &assets.iter().collect::<Vec<_>>(),
    )
    .await?
    {
        return Err(conflict("protected", reason));
    }
    let blobs = Blob::find_by_statement(statement(COMPONENT_BLOBS, vec![locked.id.into()]))
        .all(&txn)
        .await?;
    txn.execute(statement(DELETE_COMPONENT, vec![locked.id.into()]))
        .await?;
    txn.commit().await?;
    for blob in blobs {
        store::discard(ctx.content.clone(), &blob.blob_store, blob.blob_id);
    }
    Ok(origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{HOLD_LOST, transient_conflict};

    #[test]
    fn only_the_conflicts_of_a_racing_transfer_are_transient() {
        let racing = [
            ContentError::Conflict(HOLD_LOST.to_owned()),
            ContentError::Conflict("files changed while the transfer ran; try again".to_owned()),
        ];
        for error in &racing {
            assert!(transient_conflict(error), "{error}");
        }
        let final_ones = [
            ContentError::Conflict("files: /a exist with different content".to_owned()),
            ContentError::BadRequest(HOLD_LOST.to_owned()),
            ContentError::NotFound("files changed while the transfer ran; try again".to_owned()),
        ];
        for error in &final_ones {
            assert!(!transient_conflict(error), "{error}");
        }
    }
}
