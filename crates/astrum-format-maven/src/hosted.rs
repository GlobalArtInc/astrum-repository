use std::collections::BTreeMap;

use astrum_content::store::{self, PutAsset};
use astrum_content::{ContentError, ContentRequest, Ctx, View};
use astrum_db::entity::{asset, component};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, DbBackend, EntityTrait, QueryFilter, Statement};
use serde_json::Value;

use crate::metadata::{self, Metadata, Snapshot, SnapshotVersion, Versioning};
use crate::path::{self, Hash, Kind, METADATA, MavenPath};
use crate::pom::{self, Archetype};
use crate::respond::{self, Document, Found};
use crate::version::{self, SNAPSHOT};
use crate::{DOCUMENT_LIMIT, describe};

const SNAPSHOT_FILES: u64 = 20_000;
const CHECKSUM_LIMIT: usize = 4096;
const DESCRIPTION_LIMIT: usize = 500;
const ARCHETYPE: &str = "maven-archetype";

fn maven_attribute<'a>(attributes: &'a Value, name: &str) -> Option<&'a str> {
    attributes.get(crate::ID)?.get(name)?.as_str()
}

fn utc(time: chrono::DateTime<chrono::FixedOffset>) -> DateTime<Utc> {
    time.with_timezone(&Utc)
}

pub async fn metadata(ctx: &Ctx, path: &str) -> Result<Option<Document>, ContentError> {
    let Some(directory) = path.strip_suffix(&format!("/{METADATA}")) else {
        return Ok(None);
    };
    let segments: Vec<&str> = directory
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let mut documents = Vec::new();
    let mut last_modified: Option<DateTime<Utc>> = None;
    if segments.len() >= 3
        && segments.last().is_some_and(|last| last.ends_with(SNAPSHOT))
        && let Some((document, time)) = snapshot_level(ctx, directory, &segments).await?
    {
        documents.push(document);
        last_modified = last_modified.max(Some(time));
    }
    if segments.len() >= 2
        && let Some((document, time)) = artifact_level(ctx, &segments).await?
    {
        documents.push(document);
        last_modified = last_modified.max(Some(time));
    }
    if let Some(stored) = store::find(ctx.db(), ctx.name(), path).await? {
        let bytes = respond::read(ctx, &stored, DOCUMENT_LIMIT).await?;
        match metadata::parse(&bytes) {
            Ok(parsed) => {
                documents.push(Metadata {
                    plugins: parsed.plugins,
                    ..Default::default()
                });
                last_modified = last_modified.max(Some(utc(stored.blob_updated_at)));
            }
            Err(error) => {
                tracing::warn!(repository = ctx.name(), path, %error, "stored metadata cannot be read")
            }
        }
    }
    Ok(metadata::merge(documents)
        .filter(|merged| merged.versioning.is_some() || !merged.plugins.is_empty())
        .map(|merged| Document::xml(metadata::write(&merged), last_modified)))
}

async fn artifact_level(
    ctx: &Ctx,
    segments: &[&str],
) -> Result<Option<(Metadata, DateTime<Utc>)>, ContentError> {
    let (artifact_id, groups) = segments.split_last().expect("at least two segments");
    let group_id = groups.join(".");
    let components = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Namespace.eq(&group_id))
        .filter(component::Column::Name.eq(*artifact_id))
        .all(ctx.db())
        .await?;
    let Some(updated) = components
        .iter()
        .map(|component| utc(component.updated_at))
        .max()
    else {
        return Ok(None);
    };
    let versions = components
        .iter()
        .map(|component| {
            maven_attribute(&component.attributes, "baseVersion")
                .unwrap_or(&component.version)
                .to_owned()
        })
        .collect();
    Ok(Some((
        Metadata {
            group_id: Some(group_id),
            artifact_id: Some((*artifact_id).to_owned()),
            versioning: Some(metadata::versions(
                versions,
                Some(metadata::timestamp(updated)),
            )),
            ..Default::default()
        },
        updated,
    )))
}

struct Build {
    value: String,
    updated: DateTime<Utc>,
}

impl Build {
    fn rank(&self) -> (Option<(String, u32)>, DateTime<Utc>) {
        (
            version::timestamped(&self.value)
                .map(|found| (found.timestamp.to_owned(), found.build)),
            self.updated,
        )
    }
}

async fn snapshot_level(
    ctx: &Ctx,
    directory: &str,
    segments: &[&str],
) -> Result<Option<(Metadata, DateTime<Utc>)>, ContentError> {
    let count = segments.len();
    let base_version = segments[count - 1];
    let artifact_id = segments[count - 2];
    let group_id = segments[..count - 2].join(".");
    let prefix = format!("{directory}/");
    let assets = store::under(ctx.db(), ctx.name(), &prefix, SNAPSHOT_FILES).await?;
    let mut newest: BTreeMap<(Option<String>, String), Build> = BTreeMap::new();
    for found in &assets {
        if found.path[prefix.len()..].contains('/')
            || !matches!(found.kind.as_str(), "ARTIFACT" | "ARTIFACT_SUBORDINATE")
        {
            continue;
        }
        let (Some(value), Some(extension)) = (
            maven_attribute(&found.attributes, "version"),
            maven_attribute(&found.attributes, "extension"),
        ) else {
            continue;
        };
        let classifier = maven_attribute(&found.attributes, "classifier").map(str::to_owned);
        let build = Build {
            value: value.to_owned(),
            updated: utc(found.blob_updated_at),
        };
        let key = (classifier, extension.to_owned());
        if newest
            .get(&key)
            .is_none_or(|known| build.rank() > known.rank())
        {
            newest.insert(key, build);
        }
    }
    let Some(last_updated) = newest.values().map(|build| build.updated).max() else {
        return Ok(None);
    };
    let snapshot = newest
        .values()
        .filter_map(|build| version::timestamped(&build.value))
        .max()
        .map(|found| Snapshot {
            timestamp: Some(found.timestamp.to_owned()),
            build_number: Some(found.build),
            local_copy: false,
        })
        .unwrap_or(Snapshot {
            timestamp: None,
            build_number: Some(1),
            local_copy: false,
        });
    let mut entries: Vec<SnapshotVersion> = newest
        .into_iter()
        .map(|((classifier, extension), build)| SnapshotVersion {
            classifier,
            extension,
            value: build.value,
            updated: Some(metadata::timestamp(build.updated)),
        })
        .collect();
    entries.sort_by(|left, right| {
        (left.classifier.is_some(), &left.classifier, &left.extension).cmp(&(
            right.classifier.is_some(),
            &right.classifier,
            &right.extension,
        ))
    });
    Ok(Some((
        Metadata {
            group_id: Some(group_id),
            artifact_id: Some(artifact_id.to_owned()),
            version: Some(base_version.to_owned()),
            versioning: Some(Versioning {
                snapshot: Some(snapshot),
                last_updated: Some(metadata::timestamp(last_updated)),
                snapshot_versions: entries,
                ..Default::default()
            }),
            plugins: Vec::new(),
        },
        last_updated,
    )))
}

pub async fn catalog(ctx: &Ctx) -> Result<Document, ContentError> {
    let found = asset::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT * FROM asset WHERE repository = $1 AND kind = 'ARTIFACT' AND attributes #>> '{maven2,packaging}' = $2 ORDER BY path",
            [ctx.name().into(), ARCHETYPE.into()],
        ))
        .all(ctx.db())
        .await?;
    let archetypes: Vec<Archetype> = found
        .iter()
        .filter_map(|asset| {
            Some(Archetype {
                group_id: maven_attribute(&asset.attributes, "groupId")?.to_owned(),
                artifact_id: maven_attribute(&asset.attributes, "artifactId")?.to_owned(),
                version: maven_attribute(&asset.attributes, "version")?.to_owned(),
                description: maven_attribute(&asset.attributes, "description").map(str::to_owned),
            })
        })
        .collect();
    let last_modified = found.iter().map(|asset| utc(asset.blob_updated_at)).max();
    Ok(Document::xml(
        pom::write_catalog(&pom::merge_catalogs([archetypes])),
        last_modified,
    ))
}

fn require_write(ctx: &Ctx, path: &str, exists: bool) -> Result<(), ContentError> {
    ctx.require(if exists { View::Edit } else { View::Add }, path)
}

pub async fn put_checksum(
    ctx: &Ctx,
    parsed: &MavenPath,
    hash: Hash,
    request: ContentRequest,
) -> Result<Response, ContentError> {
    require_write(ctx, &parsed.path, false)?;
    let body = request.bytes(CHECKSUM_LIMIT).await?;
    let text = String::from_utf8_lossy(&body);
    let given = text
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if given.len() != hash.hex_length() || !given.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ContentError::BadRequest(format!(
            "Not a Maven2 digest: {}",
            parsed.path
        )));
    }
    if let Some(stored) = store::find(ctx.db(), ctx.name(), &parsed.base).await?
        && Found::Stored(Box::new(stored)).digest(hash) != given
    {
        tracing::warn!(
            repository = ctx.name(),
            path = parsed.path,
            "an uploaded checksum does not match the stored content; the computed one is served"
        );
    }
    Ok(StatusCode::CREATED.into_response())
}

pub async fn put_metadata(
    ctx: &Ctx,
    parsed: &MavenPath,
    request: ContentRequest,
) -> Result<Response, ContentError> {
    let stored = store::find(ctx.db(), ctx.name(), &parsed.path).await?;
    require_write(ctx, &parsed.path, stored.is_some())?;
    let body = request.bytes(DOCUMENT_LIMIT).await?;
    let uploaded = metadata::parse(&body).map_err(|_| {
        ContentError::BadRequest(format!(
            "Metadata at path {} is not a valid maven-metadata.xml",
            parsed.path.trim_start_matches('/')
        ))
    })?;
    if uploaded.plugins.is_empty() {
        return Ok(StatusCode::CREATED.into_response());
    }
    let known = match &stored {
        Some(stored) => metadata::parse(&respond::read(ctx, stored, DOCUMENT_LIMIT).await?)
            .map(|parsed| parsed.plugins)
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let merged = metadata::merge([
        Metadata {
            plugins: uploaded.plugins,
            ..Default::default()
        },
        Metadata {
            plugins: known,
            ..Default::default()
        },
    ])
    .unwrap_or_default();
    let mut put = describe(parsed, PutAsset::hosted(ctx, parsed.path.clone()));
    put.write_policy = None;
    put.check_content_type = false;
    store::put(
        ctx,
        put,
        astrum_blob::content_of(metadata::write(&merged).into_bytes()),
    )
    .await?;
    Ok(StatusCode::CREATED.into_response())
}

pub async fn put_asset(
    ctx: &Ctx,
    parsed: &MavenPath,
    request: ContentRequest,
) -> Result<Response, ContentError> {
    let exists = store::find(ctx.db(), ctx.name(), &parsed.path)
        .await?
        .is_some();
    require_write(ctx, &parsed.path, exists)?;
    let put = describe(parsed, PutAsset::hosted(ctx, parsed.path.clone()));
    match &parsed.kind {
        Kind::Artifact(coordinates) if is_pom(coordinates) => {
            let body = request.bytes(DOCUMENT_LIMIT).await?;
            let put = with_pom(put, &body);
            store::put(ctx, put, astrum_blob::content_of(body)).await?;
        }
        _ => {
            store::put(ctx, put, request.into_stream()).await?;
        }
    }
    Ok(StatusCode::CREATED.into_response())
}

pub fn is_pom(coordinates: &path::Coordinates) -> bool {
    coordinates.extension == "pom" && coordinates.classifier.is_none()
}

pub fn with_pom(mut put: PutAsset, body: &[u8]) -> PutAsset {
    let Ok(parsed) = pom::parse(body) else {
        return put;
    };
    if let Some(maven) = put
        .attributes
        .get_mut(crate::ID)
        .and_then(Value::as_object_mut)
    {
        if parsed.packaging == ARCHETYPE
            && let Some(description) = parsed.description
        {
            let short: String = description.chars().take(DESCRIPTION_LIMIT).collect();
            maven.insert("description".to_owned(), Value::String(short));
        }
        maven.insert("packaging".to_owned(), Value::String(parsed.packaging));
    }
    put
}
