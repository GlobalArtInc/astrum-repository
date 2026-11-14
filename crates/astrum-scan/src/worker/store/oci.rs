use astrum_policy::subject;
use sea_orm::{ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, Statement};

pub(super) async fn previous_digest(
    txn: &DatabaseTransaction,
    component_id: i64,
) -> Result<Option<String>, DbErr> {
    let row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT digest FROM component_scan WHERE component_id = $1 FOR UPDATE",
            [component_id.into()],
        ))
        .await?;
    match row {
        Some(row) => row.try_get("", "digest"),
        None => Ok(None),
    }
}

pub(super) async fn mark_subjects(
    txn: &DatabaseTransaction,
    component_id: i64,
    before: Option<&str>,
    now: Option<(&str, i64)>,
) -> Result<(), DbErr> {
    let mut digests: Vec<String> = before
        .into_iter()
        .chain(now.map(|(digest, _)| digest))
        .map(str::to_owned)
        .collect();
    digests.sort();
    digests.dedup();
    let sharing = now.map(|(_, inventory_id)| inventory_id);
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        format!(
            "INSERT INTO policy_dirty (repository, subject) \
             SELECT repository, subject FROM (\
               SELECT repository, '{component}' || id AS subject, id AS n \
               FROM component WHERE id = $1 \
               UNION ALL \
               SELECT repository, '{component}' || component_id, component_id \
               FROM component_scan WHERE inventory_id = $2::bigint AND component_id <> $1 \
               UNION ALL \
               SELECT c.repository, '{image}' || d, 0::bigint \
               FROM component c CROSS JOIN unnest($3::text[]) AS d WHERE c.id = $1) marks \
             ORDER BY (subject LIKE '{image}%'), n, subject, repository \
             ON CONFLICT (repository, subject) DO NOTHING",
            component = subject::COMPONENT_PREFIX,
            image = subject::IMAGE_PREFIX,
        ),
        [component_id.into(), sharing.into(), digests.into()],
    ))
    .await?;
    Ok(())
}
