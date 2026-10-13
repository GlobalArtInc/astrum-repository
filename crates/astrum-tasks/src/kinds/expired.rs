use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::count;
use crate::kind::{Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "security.purge-expired";
pub const SESSION_TIMEOUT: &str = "security.sessionTimeoutMinutes";

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
        "deletes expired sign-in sessions, client tokens and unfinished uploads".to_owned()
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
        for expiry in run.expiries() {
            run.check()?;
            let removed = expiry.expire(run.content()).await?;
            uploads += removed;
            run.info(format!("{}: deleted {removed}", expiry.name()));
        }
        run.count("uploads", uploads as i64);
        Ok(format!(
            "deleted {}, {} and {}",
            count(sessions as i64, "session", "sessions"),
            count(tokens as i64, "client token", "client tokens"),
            count(uploads as i64, "unfinished upload", "unfinished uploads")
        ))
    }
}
