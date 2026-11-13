use astrum_content::inventory::LicenseFact;
use astrum_content::store::{self, Component};
use astrum_content::{ContentError, Ctx};
use futures_util::TryStreamExt;

use crate::DOCUMENT_LIMIT;
use crate::path::Coordinates;
use crate::pom::{self, Parent, Pom};
use crate::version;

const SOURCE: &str = "maven.pom";
const MAX_DEPTH: usize = 5;

pub(crate) async fn fact(
    ctx: &Ctx,
    component: &Component,
) -> Result<Option<LicenseFact>, ContentError> {
    let mut coordinates = Coordinates {
        group_id: component.namespace.clone(),
        artifact_id: component.name.clone(),
        version: component.version.clone(),
        base_version: version::base_version(&component.version),
        classifier: None,
        extension: "pom".to_owned(),
    };
    let mut visited: Vec<String> = Vec::new();
    for depth in 0..=MAX_DEPTH {
        let path = coordinates.path();
        if visited.contains(&path) {
            return Ok(None);
        }
        let Some(pom) = read_pom(ctx, &path).await? else {
            return Ok(None);
        };
        visited.push(path);
        if let Some(declared) = pom.license_text() {
            return Ok(Some(LicenseFact {
                declared,
                source: SOURCE.to_owned(),
            }));
        }
        let Some(parent) = pom.parent else {
            return Ok(None);
        };
        if depth == MAX_DEPTH || !valid(&parent) {
            return Ok(None);
        }
        coordinates = Coordinates {
            group_id: parent.group_id,
            artifact_id: parent.artifact_id,
            base_version: version::base_version(&parent.version),
            version: parent.version,
            classifier: None,
            extension: "pom".to_owned(),
        };
    }
    Ok(None)
}

fn valid(parent: &Parent) -> bool {
    [&parent.group_id, &parent.artifact_id, &parent.version]
        .into_iter()
        .all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && !segment.contains('/')
                && !segment.chars().any(char::is_control)
        })
}

async fn read_pom(ctx: &Ctx, path: &str) -> Result<Option<Pom>, ContentError> {
    let Some(asset) = store::find(ctx.db(), ctx.name(), path).await? else {
        return Ok(None);
    };
    let Ok(size) = usize::try_from(asset.size) else {
        return Ok(None);
    };
    if size > DOCUMENT_LIMIT {
        return Ok(None);
    }
    let blobs = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = blobs.read(asset.blob_id.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob of the repository is missing",
        )));
    };
    let mut stream = content.stream;
    let mut bytes = Vec::with_capacity(size);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if bytes.len() + chunk.len() > DOCUMENT_LIMIT {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(pom::parse(&bytes).ok())
}
