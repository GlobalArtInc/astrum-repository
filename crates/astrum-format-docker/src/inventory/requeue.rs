use astrum_content::{ContentError, Ctx};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::children;
use crate::manifest::{Kind, Manifest};
use crate::reference::Digest;

const REASON: &str = "sbom";
const PARENTS: u64 = 16;
const BATCH: usize = 500;
const LIMIT: usize = 5000;

fn legacy_subject(tag: &str) -> Option<Digest> {
    let stem = tag
        .strip_suffix(".sbom")
        .or_else(|| tag.strip_suffix(".att"))?;
    Digest::parse(&stem.replacen('-', ":", 1))
}

pub(crate) async fn after_store(ctx: &Ctx, image: &str, parsed: &Manifest, tags: &[String]) {
    if let Err(error) = queue(ctx, image, parsed, tags).await {
        tracing::warn!(
            %error,
            repository = ctx.name(),
            image,
            "the images of a stored SBOM or attestation were not queued for a new scan"
        );
    }
}

async fn queue(
    ctx: &Ctx,
    image: &str,
    parsed: &Manifest,
    tags: &[String],
) -> Result<(), ContentError> {
    if ctx.repository.policy_name().is_none() {
        return Ok(());
    }
    let mut subjects: Vec<Digest> = Vec::new();
    if matches!(parsed.kind, Kind::Sbom | Kind::Attestation)
        && let Some(subject) = &parsed.subject
    {
        subjects.push(subject.clone());
    }
    subjects.extend(tags.iter().filter_map(|tag| legacy_subject(tag)));
    for subject in subjects {
        let mut digests = vec![subject.to_string()];
        for parent in children::parents(ctx.db(), ctx.name(), &subject, PARENTS).await? {
            digests.push(parent.to_string());
        }
        let rows = ctx
            .db()
            .query_all(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT DISTINCT component_id FROM asset \
                 WHERE repository = $1 AND format = 'docker' AND kind = 'tag' \
                   AND component_id IS NOT NULL \
                   AND attributes #>> '{docker,name}' = $2 \
                   AND attributes #>> '{docker,digest}' = ANY($3) \
                 ORDER BY component_id LIMIT $4",
                [
                    ctx.name().into(),
                    image.into(),
                    digests.into(),
                    (LIMIT as i64 + 1).into(),
                ],
            ))
            .await?;
        let mut ids = Vec::with_capacity(rows.len());
        for row in rows {
            ids.push(row.try_get::<i64>("", "component_id")?);
        }
        if ids.len() > LIMIT {
            ids.truncate(LIMIT);
            tracing::warn!(
                repository = ctx.name(),
                image,
                limit = LIMIT,
                "more tags hold the image of a stored SBOM than are queued for a new scan; the rest wait for the next sweep"
            );
        }
        for batch in ids.chunks(BATCH) {
            astrum_scan::queue::enqueue_components(ctx.db(), ctx.name(), batch, REASON).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_legacy_sbom_or_attestation_tag_names_its_subject_and_other_tags_name_none() {
        let hex = "ab".repeat(32);
        let wanted = Digest::parse(&format!("sha256:{hex}"));
        assert!(wanted.is_some());
        assert_eq!(legacy_subject(&format!("sha256-{hex}.sbom")), wanted);
        assert_eq!(legacy_subject(&format!("sha256-{hex}.att")), wanted);
        assert_eq!(legacy_subject(&format!("sha256-{hex}.sig")), None);
        assert_eq!(legacy_subject("sha256-short.sbom"), None);
        assert_eq!(legacy_subject("1.0"), None);
    }
}
