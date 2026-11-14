use astrum_policy::subject::IMAGE_PREFIX;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement};

fn page(row: Option<QueryResult>) -> Result<(i64, i64, Option<String>, Option<String>), DbErr> {
    let row =
        row.ok_or_else(|| DbErr::RecordNotFound("a sweep page returned no row".to_owned()))?;
    Ok((
        row.try_get("", "n")?,
        row.try_get("", "changed")?,
        row.try_get("", "last_repository")?,
        row.try_get("", "last_subject")?,
    ))
}

pub(super) async fn mark_stale(db: &DatabaseConnection, chunk: i64) -> Result<u64, DbErr> {
    let sql = format!(
        "WITH page AS (\
           SELECT repository, subject FROM (\
             SELECT cs.repository, '{IMAGE_PREFIX}' || cs.digest AS subject \
             FROM content_policy p \
             JOIN repository r ON r.attributes #>> '{{policy,name}}' = p.name \
             JOIN component_scan cs ON cs.repository = r.name AND cs.digest IS NOT NULL \
             LEFT JOIN policy_verdict v \
               ON v.repository = cs.repository AND v.subject = '{IMAGE_PREFIX}' || cs.digest \
             WHERE v.subject IS NULL AND (p.vulnerabilities ->> 'mode') IS DISTINCT FROM 'off' \
             UNION \
             SELECT v.repository, v.subject FROM policy_verdict v \
             JOIN repository r ON r.name = v.repository \
             JOIN content_policy p ON p.name = r.attributes #>> '{{policy,name}}' \
             WHERE v.subject LIKE '{IMAGE_PREFIX}%' \
               AND (v.policy_revision <> p.revision OR v.policy <> p.name)) stale \
           WHERE (repository, subject) > ($1, $2) \
           ORDER BY repository, subject LIMIT $3), \
         live AS (\
           SELECT name FROM repository WHERE name IN (SELECT repository FROM page) \
           ORDER BY name FOR KEY SHARE), \
         queued AS (\
           INSERT INTO policy_dirty (repository, subject) \
           SELECT p.repository, p.subject FROM page p JOIN live l ON l.name = p.repository \
           ORDER BY p.repository, p.subject ON CONFLICT (repository, subject) DO NOTHING \
           RETURNING 1) \
         SELECT count(*) AS n, (SELECT count(*) FROM queued) AS changed, \
           (array_agg(repository ORDER BY repository DESC, subject DESC))[1] AS last_repository, \
           (array_agg(subject ORDER BY repository DESC, subject DESC))[1] AS last_subject \
         FROM page"
    );
    sweep_pages(db, chunk, &sql).await
}

pub(super) async fn drop_orphans(db: &DatabaseConnection, chunk: i64) -> Result<u64, DbErr> {
    let sql = format!(
        "WITH page AS (\
           SELECT repository, subject FROM policy_verdict \
           WHERE subject LIKE '{IMAGE_PREFIX}%' AND (repository, subject) > ($1, $2) \
           ORDER BY repository, subject LIMIT $3), \
         gone AS (\
           SELECT p.repository, p.subject FROM page p \
           WHERE NOT EXISTS (SELECT 1 FROM asset a \
             WHERE a.repository = p.repository AND a.format = 'docker' \
               AND a.kind IN ('manifest', 'tag') \
               AND a.attributes #>> '{{docker,digest}}' = substr(p.subject, {skip}))), \
         deleted AS (\
           DELETE FROM policy_verdict v USING gone g \
           WHERE v.repository = g.repository AND v.subject = g.subject RETURNING 1) \
         SELECT count(*) AS n, (SELECT count(*) FROM deleted) AS changed, \
           (array_agg(repository ORDER BY repository DESC, subject DESC))[1] AS last_repository, \
           (array_agg(subject ORDER BY repository DESC, subject DESC))[1] AS last_subject \
         FROM page",
        skip = IMAGE_PREFIX.len() + 1,
    );
    let removed = sweep_pages(db, chunk, &sql).await?;
    if removed > 0 {
        tracing::debug!(
            removed,
            "verdicts of images that no manifest or tag holds any more were removed"
        );
    }
    Ok(removed)
}

async fn sweep_pages(db: &DatabaseConnection, chunk: i64, sql: &str) -> Result<u64, DbErr> {
    let mut changed = 0u64;
    let mut after = (String::new(), String::new());
    loop {
        let (found, done, last_repository, last_subject) = page(
            db.query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql,
                [after.0.clone().into(), after.1.clone().into(), chunk.into()],
            ))
            .await?,
        )?;
        changed += done as u64;
        match (found < chunk, last_repository, last_subject) {
            (false, Some(repository), Some(subject)) => after = (repository, subject),
            _ => return Ok(changed),
        }
    }
}
