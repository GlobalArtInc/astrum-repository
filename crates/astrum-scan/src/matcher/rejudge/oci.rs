use astrum_policy::subject::IMAGE_PREFIX;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};

const KEY_PATTERN: &str = "^oci:(.+):([a-z0-9]+:[a-f0-9]+)$";

pub(super) async fn fan_out(
    db: &DatabaseConnection,
    chunk: i64,
    package_id: i64,
) -> Result<u64, DbErr> {
    let sql = format!(
        "WITH page AS (\
           SELECT i.id, parts[1] AS repository, '{IMAGE_PREFIX}' || parts[2] AS subject \
           FROM sca_inventory_package ip \
           JOIN sca_inventory i ON i.id = ip.inventory_id, \
           LATERAL regexp_match(i.key, '{KEY_PATTERN}') AS parts \
           WHERE ip.package_id = $1 AND i.id > $2 AND parts IS NOT NULL \
           ORDER BY i.id LIMIT $3), \
         queued AS (\
           INSERT INTO policy_dirty (repository, subject) \
           SELECT p.repository, p.subject FROM page p \
           JOIN repository r ON r.name = p.repository \
           ORDER BY p.subject, p.repository \
           ON CONFLICT (repository, subject) DO NOTHING) \
         SELECT count(*) AS n, COALESCE(max(id), $2) AS last FROM page"
    );
    let mut after = 0i64;
    let mut subjects = 0u64;
    loop {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql.clone(),
                [package_id.into(), after.into(), chunk.into()],
            ))
            .await?
            .ok_or_else(|| {
                DbErr::RecordNotFound("the image fan-out page returned no row".to_owned())
            })?;
        let found: i64 = row.try_get("", "n")?;
        after = row.try_get("", "last")?;
        subjects += found as u64;
        if found < chunk {
            return Ok(subjects);
        }
    }
}
