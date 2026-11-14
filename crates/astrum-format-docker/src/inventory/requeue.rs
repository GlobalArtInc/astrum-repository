use astrum_content::{ContentError, Ctx};
use sea_orm::{ConnectionTrait, DbBackend, Statement};

use crate::children;
use crate::manifest::{Kind, Manifest};
use crate::reference::Digest;

const REASON: &str = "sbom";
const PARENTS: u64 = 16;
const TAGS: i64 = 1000;

fn legacy_subject(tag: &str) -> Option<Digest> {
    let stem = tag
        .strip_suffix(".sbom")
        .or_else(|| tag.strip_suffix(".att"))?;
    Digest::parse(&stem.replacen('-', ":", 1))
}

pub(crate) async fn after_store(
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
                [ctx.name().into(), image.into(), digests.into(), TAGS.into()],
            ))
            .await?;
        for row in rows {
            let component_id: i64 = row.try_get("", "component_id")?;
            astrum_scan::queue::enqueue_component(ctx.db(), ctx.name(), component_id, REASON)
                .await?;
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
