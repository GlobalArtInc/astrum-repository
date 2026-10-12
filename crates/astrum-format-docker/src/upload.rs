use std::io;

use astrum_blob::{BlobId, ContentStream, Digests, Written};
use astrum_content::store;
use astrum_content::{ContentError, Ctx};
use astrum_db::entity::docker_upload;
use axum::http::StatusCode;
use chrono::Utc;
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::{ActiveValue, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::RegistryError;

pub type Session = docker_upload::Model;

pub const EXPIRY_HOURS: i64 = 24;
const DRAIN_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    pub blob: Uuid,
    pub size: u64,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub md5: String,
}

impl Chunk {
    fn of(written: &Written) -> Self {
        Self {
            blob: written.id.uuid(),
            size: written.size,
            sha1: written.digests.sha1.clone(),
            sha256: written.digests.sha256.clone(),
            sha512: written.digests.sha512.clone(),
            md5: written.digests.md5.clone(),
        }
    }

    fn written(&self, store: &str) -> Written {
        Written {
            id: BlobId::from(self.blob),
            store: store.to_owned(),
            size: self.size,
            digests: Digests {
                sha1: self.sha1.clone(),
                sha256: self.sha256.clone(),
                sha512: self.sha512.clone(),
                md5: self.md5.clone(),
            },
        }
    }
}

pub fn chunks(session: &Session) -> Vec<Chunk> {
    serde_json::from_value(session.chunks.clone()).unwrap_or_default()
}

pub fn range(size: i64) -> String {
    format!("0-{}", (size - 1).max(0))
}

pub fn parse_range(value: &str) -> Option<(u64, u64)> {
    let value = value.trim();
    let value = value.strip_prefix("bytes ").unwrap_or(value).trim();
    let value = value.split('/').next()?.trim();
    let value = value.strip_prefix("bytes=").unwrap_or(value);
    let (start, end) = value.split_once('-')?;
    let start: u64 = start.trim().parse().ok()?;
    let end: u64 = end.trim().parse().ok()?;
    (end >= start).then_some((start, end))
}

pub async fn drain(mut body: ContentStream) {
    let mut read = 0usize;
    while let Ok(Some(chunk)) = body.try_next().await {
        read += chunk.len();
        if read > DRAIN_LIMIT {
            break;
        }
    }
}

pub fn out_of_order(session: &Session) -> RegistryError {
    RegistryError::new(
        StatusCode::RANGE_NOT_SATISFIABLE,
        "BLOB_UPLOAD_INVALID",
        format!("the upload is at offset {}", session.size),
    )
    .header(axum::http::header::RANGE, range(session.size))
}

pub async fn create(ctx: &Ctx, image: &str) -> Result<Session, ContentError> {
    let store = ctx.repository.blob_store().ok_or_else(|| {
        ContentError::internal(io::Error::other(format!(
            "the repository {} has no blob store",
            ctx.name()
        )))
    })?;
    let now = Utc::now().fixed_offset();
    let row = docker_upload::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        repository: ActiveValue::Set(ctx.name().to_owned()),
        image: ActiveValue::Set(image.to_owned()),
        blob_store: ActiveValue::Set(store.to_owned()),
        size: ActiveValue::Set(0),
        chunks: ActiveValue::Set(serde_json::json!([])),
        created_by: ActiveValue::Set(ctx.uploader()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    };
    Ok(docker_upload::Entity::insert(row)
        .exec_with_returning(ctx.db())
        .await?)
}

pub async fn load(ctx: &Ctx, id: &str, image: &str) -> Result<Session, RegistryError> {
    let Ok(id) = Uuid::parse_str(id) else {
        return Err(RegistryError::upload_unknown());
    };
    let session = docker_upload::Entity::find_by_id(id)
        .one(ctx.db())
        .await?
        .filter(|session| {
            session.repository == ctx.name()
                && session.image == image
                && session.created_by == ctx.uploader()
        })
        .ok_or_else(RegistryError::upload_unknown)?;
    Ok(session)
}

pub async fn append(
    ctx: &Ctx,
    session: &Session,
    claimed: Option<(u64, u64)>,
    body: ContentStream,
) -> Result<Session, RegistryError> {
    if let Some((start, _)) = claimed
        && start != session.size as u64
    {
        drain(body).await;
        return Err(out_of_order(session));
    }
    let written = store::write_blob(ctx, body).await?;
    if written.size == 0 {
        crate::storage::discard(ctx, &written);
        return Ok(session.clone());
    }
    if let Some((start, end)) = claimed
        && end - start + 1 != written.size
    {
        crate::storage::discard(ctx, &written);
        return Err(RegistryError::new(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "BLOB_UPLOAD_INVALID",
            format!(
                "the chunk holds {} bytes but its range names {}",
                written.size,
                end - start + 1
            ),
        )
        .header(axum::http::header::RANGE, range(session.size)));
    }
    let chunk = serde_json::to_value(vec![Chunk::of(&written)]).map_err(ContentError::internal)?;
    let updated = Session::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE docker_upload SET size = size + $2, chunks = chunks || $3::jsonb, \
         updated_at = now() WHERE id = $1 AND size = $4 RETURNING *",
        vec![
            session.id.into(),
            (written.size as i64).into(),
            chunk.into(),
            session.size.into(),
        ],
    ))
    .one(ctx.db())
    .await?;
    match updated {
        Some(updated) => Ok(updated),
        None => {
            crate::storage::discard(ctx, &written);
            match docker_upload::Entity::find_by_id(session.id)
                .one(ctx.db())
                .await?
            {
                Some(current) => Err(out_of_order(&current)),
                None => Err(RegistryError::upload_unknown()),
            }
        }
    }
}

pub async fn claim(ctx: &Ctx, session: &Session) -> Result<Session, RegistryError> {
    Session::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM docker_upload WHERE id = $1 RETURNING *",
        vec![session.id.into()],
    ))
    .one(ctx.db())
    .await?
    .ok_or_else(RegistryError::upload_unknown)
}

pub async fn assemble(ctx: &Ctx, session: &Session) -> Result<Written, ContentError> {
    let chunks = chunks(session);
    if let [only] = chunks.as_slice() {
        return Ok(only.written(&session.blob_store));
    }
    let store = ctx.content.blob_stores().get(&session.blob_store).await?;
    let ids: Vec<Uuid> = chunks.iter().map(|chunk| chunk.blob).collect();
    let reader = store.clone();
    let body: ContentStream = futures_util::stream::iter(ids)
        .then(move |id| {
            let store = reader.clone();
            async move {
                match store.read(id.into()).await {
                    Ok(Some(content)) => Ok(content.stream),
                    Ok(None) => Err(io::Error::other("a chunk of the upload is missing")),
                    Err(error) => Err(io::Error::other(error)),
                }
            }
        })
        .try_flatten()
        .boxed();
    let written = store.put(body).await;
    discard_chunks(ctx, session, &chunks);
    Ok(written?)
}

pub fn discard_chunks(ctx: &Ctx, session: &Session, chunks: &[Chunk]) {
    for chunk in chunks {
        store::discard(ctx.content.clone(), &session.blob_store, chunk.blob);
    }
}

pub async fn sweep(content: &astrum_content::Content) -> Result<u64, ContentError> {
    let expired = Session::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM docker_upload WHERE updated_at < now() - make_interval(hours => $1) \
         RETURNING *",
        vec![(EXPIRY_HOURS as i32).into()],
    ))
    .all(content.db())
    .await?;
    let mut blobs = Vec::new();
    for session in &expired {
        for chunk in chunks(session) {
            blobs.push((session.blob_store.clone(), chunk.blob));
        }
    }
    content.purge_blobs(blobs).await;
    content
        .db()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM format_token WHERE realm = $1 AND expires_at < now()",
            vec![crate::auth::REALM.into()],
        ))
        .await?;
    Ok(expired.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_are_inclusive_and_ordered() {
        assert_eq!(parse_range("0-99"), Some((0, 99)));
        assert_eq!(parse_range("bytes=100-199"), Some((100, 199)));
        assert_eq!(parse_range("bytes 5-9/10"), Some((5, 9)));
        assert_eq!(parse_range("9-5"), None);
        assert_eq!(parse_range("x-5"), None);
        assert_eq!(parse_range(""), None);
        assert_eq!(range(0), "0-0");
        assert_eq!(range(1), "0-0");
        assert_eq!(range(100), "0-99");
    }

    #[test]
    fn chunks_round_trip_through_json() {
        let chunk = Chunk {
            blob: Uuid::nil(),
            size: 3,
            sha1: "a".to_owned(),
            sha256: "b".to_owned(),
            sha512: "c".to_owned(),
            md5: "d".to_owned(),
        };
        let written = chunk.written("default");
        assert_eq!(written.size, 3);
        assert_eq!(written.store, "default");
        assert_eq!(Chunk::of(&written), chunk);
    }
}
