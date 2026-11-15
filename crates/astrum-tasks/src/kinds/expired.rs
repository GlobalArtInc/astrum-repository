use astrum_content::Content;
use astrum_content::holds::{self, Expired};
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::count;
use crate::kind::{Expiry, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "security.purge-expired";
pub const SESSION_TIMEOUT: &str = "security.sessionTimeoutMinutes";
const HOLD_BATCH: u64 = 500;

pub struct PurgeExpired;

#[async_trait]
impl TaskKind for PurgeExpired {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Admin - Delete expired sessions and uploads"
    }

    fn singleton(&self) -> bool {
        true
    }

    fn describe(&self, _properties: &Properties) -> String {
        "deletes expired sign-in sessions, client tokens and unfinished uploads, and drops blob holds past their time, deleting their blobs if no asset names them".to_owned()
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let timeout: u32 = astrum_db::setting::get(run.db(), SESSION_TIMEOUT)
            .await
            .map_err(TaskError::failed)?
            .unwrap_or(0);
        let sessions = match timeout {
            0 => 0,
            minutes => {
                run.db()
                    .execute(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        "DELETE FROM user_session WHERE last_seen_at < now() - make_interval(mins => $1)",
                        [(minutes as i32).into()],
                    ))
                    .await?
                    .rows_affected()
            }
        };
        run.count("sessions", sessions as i64);
        run.info(match timeout {
            0 => "sessions never expire on this server (the session timeout is 0)".to_owned(),
            minutes => format!(
                "deleted {} idle for more than {}",
                count(sessions as i64, "session", "sessions"),
                count(minutes as i64, "minute", "minutes")
            ),
        });
        let tokens = run
            .db()
            .execute(Statement::from_string(
                DbBackend::Postgres,
                "DELETE FROM format_token WHERE expires_at IS NOT NULL AND expires_at < now()",
            ))
            .await?
            .rows_affected();
        run.count("tokens", tokens as i64);
        run.info(format!(
            "deleted {}",
            count(
                tokens as i64,
                "expired client token",
                "expired client tokens"
            )
        ));
        let mut uploads = 0;
        let mut released = 0;
        for expiry in run.expiries() {
            run.check()?;
            let removed = expiry.expire(run.content()).await?;
            match expiry.counts_as_upload() {
                true => uploads += removed,
                false => released += removed,
            }
            run.info(format!("{}: deleted {removed}", expiry.name()));
        }
        run.count("uploads", uploads as i64);
        let mut message = format!(
            "deleted {}, {} and {}",
            count(sessions as i64, "session", "sessions"),
            count(tokens as i64, "client token", "client tokens"),
            count(uploads as i64, "unfinished upload", "unfinished uploads")
        );
        if released > 0 {
            run.count("holds", released as i64);
            message.push_str(&format!(
                "; released {}",
                count(released as i64, "expired blob hold", "expired blob holds")
            ));
        }
        Ok(message)
    }
}

pub struct BlobHoldExpiry;

#[async_trait]
impl Expiry for BlobHoldExpiry {
    fn name(&self) -> &'static str {
        "Blobs held for transfers past their time"
    }

    fn counts_as_upload(&self) -> bool {
        false
    }

    async fn expire(&self, content: &Content) -> Result<u64, TaskError> {
        Ok(drain_lapsed_holds(content).await?.0)
    }
}

pub async fn drain_lapsed_holds(content: &Content) -> Result<(u64, u64), TaskError> {
    let mut released = 0;
    let mut freed = 0;
    loop {
        let batch = holds::take_expired(content.db(), HOLD_BATCH).await?;
        let taken = batch.len() as u64;
        for expired in batch.into_iter().filter(|expired| !expired.named) {
            match delete_held(content, &expired).await {
                Ok(bytes) => freed += bytes,
                Err(error) => {
                    tracing::warn!(%error, store = expired.store, blob = %expired.id, "a blob whose hold ran out stays in its store until compaction collects it");
                }
            }
        }
        released += taken;
        if taken < HOLD_BATCH {
            return Ok((released, freed));
        }
    }
}

async fn delete_held(content: &Content, expired: &Expired) -> Result<u64, astrum_blob::Error> {
    let store = content.blob_stores().get(&expired.store).await?;
    let size = store.size(expired.id).await?;
    Ok(match store.delete(expired.id).await? {
        true => size.unwrap_or(0),
        false => 0,
    })
}
