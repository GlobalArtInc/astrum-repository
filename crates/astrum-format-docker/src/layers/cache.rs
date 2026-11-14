use astrum_content::ContentError;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::EXTRACTOR;
use super::facts::LayerFacts;

pub async fn load(
    db: &impl ConnectionTrait,
    digest: &str,
) -> Result<Option<LayerFacts>, ContentError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT facts FROM oci_layer_facts WHERE digest = $1 AND extractor = $2",
            [digest.into(), EXTRACTOR.into()],
        ))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let document: serde_json::Value = row.try_get("", "facts")?;
    match serde_json::from_value(document) {
        Ok(facts) => Ok(Some(facts)),
        Err(error) => {
            tracing::warn!(%error, digest, "cached layer facts are unreadable and are extracted again");
            Ok(None)
        }
    }
}

pub async fn store(
    db: &impl ConnectionTrait,
    digest: &str,
    facts: &LayerFacts,
) -> Result<(), ContentError> {
    let document = serde_json::to_value(facts).map_err(ContentError::internal)?;
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO oci_layer_facts (digest, extractor, facts) VALUES ($1, $2, $3) \
         ON CONFLICT (digest) DO UPDATE SET extractor = EXCLUDED.extractor, \
           facts = EXCLUDED.facts, extracted_at = now() \
         WHERE oci_layer_facts.extractor <= EXCLUDED.extractor",
        [digest.into(), EXTRACTOR.into(), document.into()],
    ))
    .await?;
    Ok(())
}
