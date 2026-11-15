use std::io;

use astrum_content::ContentError;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use super::EXTRACTOR;
use super::facts::LayerFacts;

struct Capped {
    bytes: Vec<u8>,
    limit: usize,
}

impl io::Write for Capped {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.bytes.len() + buf.len() > self.limit {
            return Err(io::Error::other(
                "the facts are larger than the cache accepts",
            ));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub async fn load(
    db: &impl ConnectionTrait,
    digest: &str,
) -> Result<Option<LayerFacts>, ContentError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT facts::text AS facts FROM oci_layer_facts WHERE digest = $1 AND extractor = $2",
            [digest.into(), EXTRACTOR.into()],
        ))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let text: String = row.try_get("", "facts")?;
    match serde_json::from_str(&text) {
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
    limit: usize,
) -> Result<bool, ContentError> {
    let mut capped = Capped {
        bytes: Vec::new(),
        limit,
    };
    if let Err(error) = serde_json::to_writer(&mut capped, facts) {
        tracing::warn!(%error, digest, limit, "layer facts are too large to cache");
        return Ok(false);
    }
    let text = String::from_utf8(capped.bytes).map_err(ContentError::internal)?;
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO oci_layer_facts (digest, extractor, facts) \
         VALUES ($1, $2, CAST($3 AS text)::jsonb) \
         ON CONFLICT (digest) DO UPDATE SET extractor = EXCLUDED.extractor, \
           facts = EXCLUDED.facts, extracted_at = now() \
         WHERE oci_layer_facts.extractor <= EXCLUDED.extractor",
        [digest.into(), EXTRACTOR.into(), text.into()],
    ))
    .await?;
    Ok(true)
}
