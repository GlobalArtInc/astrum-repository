use astrum_content::Content;
use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::kind::{Expiry, TaskError};

pub const ACTION: &str = "cooldown.released";

const RELEASE: &str = "WITH due AS (\
    DELETE FROM cooldown_wait WHERE available_at <= now() \
    RETURNING repository, component_id, blocked), \
announced AS (\
    SELECT notification_capture(due.repository, c.format, c.namespace, c.name, c.version, $1, c.id, NULL) AS captured \
    FROM due JOIN component c ON c.id = due.component_id WHERE due.blocked) \
SELECT (SELECT count(*) FROM due)::bigint AS deleted, (SELECT count(*) FROM announced)::bigint AS announced";

pub struct CooldownExpiry;

#[async_trait]
impl Expiry for CooldownExpiry {
    fn name(&self) -> &'static str {
        "Cooldown waits whose version became available"
    }

    async fn expire(&self, content: &Content) -> Result<u64, TaskError> {
        let row = content
            .db()
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                RELEASE,
                [ACTION.into()],
            ))
            .await?;
        let Some(row) = row else { return Ok(0) };
        Ok(row.try_get::<i64>("", "deleted")?.max(0) as u64)
    }
}
