use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, Asset, ComponentKey, PutAsset};
use astrum_content::{ContentError, Ctx};
use astrum_db::entity::asset;
use futures_util::TryStreamExt;
use sea_orm::{
    ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, Statement, Value as SqlValue,
};
use serde_json::{Map, Value, json};

use crate::manifest::Manifest;
use crate::reference::{Algorithm, Digest};

pub const BLOB: &str = "blob";
pub const MANIFEST: &str = "manifest";
pub const TAG: &str = "tag";
pub const OCTET_STREAM: &str = "application/octet-stream";
const ANNOTATIONS_KEPT: usize = 16 * 1024;

pub fn blob_path(digest: &Digest) -> String {
    format!("/v2/-/blobs/{digest}")
}

pub fn manifest_path(image: &str, reference: &str) -> String {
    format!("/v2/{image}/manifests/{reference}")
}

async fn assets(
    db: &impl ConnectionTrait,
    filter: &str,
    values: Vec<SqlValue>,
) -> Result<Vec<Asset>, ContentError> {
    let sql = format!(
        "SELECT * FROM asset WHERE format = 'docker' AND repository = $1 AND {filter} ORDER BY path"
    );
    Ok(asset::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            &sql,
            values,
        ))
        .all(db)
        .await?)
}

pub async fn find(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    store::find(ctx.db(), ctx.name(), path).await
}

pub async fn tags(ctx: &Ctx, image: &str) -> Result<Vec<Asset>, ContentError> {
    assets(
        ctx.db(),
        "kind = 'tag' AND attributes #>> '{docker,name}' = $2",
        vec![ctx.name().into(), image.into()],
    )
    .await
}

pub fn tag_name(asset: &Asset, image: &str) -> String {
    asset
        .attributes
        .pointer("/docker/tag")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            asset
                .path
                .strip_prefix(&format!("/v2/{image}/manifests/"))
                .unwrap_or_default()
                .to_owned()
        })
}

pub fn digest_of(asset: &Asset) -> Option<Digest> {
    asset
        .attributes
        .pointer("/docker/digest")
        .and_then(Value::as_str)
        .and_then(Digest::parse)
}

pub fn media_type_of(asset: &Asset) -> String {
    asset
        .attributes
        .pointer("/docker/mediaType")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| asset.content_type.clone())
}

pub async fn tags_pointing_at(
    ctx: &Ctx,
    image: &str,
    digest: &Digest,
) -> Result<Vec<Asset>, ContentError> {
    assets(
        ctx.db(),
        "kind = 'tag' AND attributes #>> '{docker,name}' = $2 \
         AND attributes #>> '{docker,digest}' = $3",
        vec![ctx.name().into(), image.into(), digest.to_string().into()],
    )
    .await
}

pub async fn referrers(
    ctx: &Ctx,
    image: &str,
    subject: &Digest,
) -> Result<Vec<Asset>, ContentError> {
    assets(
        ctx.db(),
        "kind = 'manifest' AND attributes #>> '{docker,name}' = $2 \
         AND attributes #>> '{docker,subject}' = $3",
        vec![ctx.name().into(), image.into(), subject.to_string().into()],
    )
    .await
}

pub async fn has_image(ctx: &Ctx, image: &str) -> Result<bool, ContentError> {
    #[derive(FromQueryResult)]
    struct Found {
        found: bool,
    }
    let found = Found::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT EXISTS (SELECT 1 FROM asset WHERE format = 'docker' AND repository = $1 \
         AND attributes #>> '{docker,name}' = $2) AS found",
        vec![ctx.name().into(), image.into()],
    ))
    .one(ctx.db())
    .await?;
    Ok(found.is_some_and(|found| found.found))
}

pub async fn images(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    #[derive(FromQueryResult)]
    struct Name {
        name: String,
    }
    let names = Name::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT DISTINCT attributes #>> '{docker,name}' AS name FROM asset \
         WHERE format = 'docker' AND repository = $1 AND kind IN ('tag', 'manifest') \
         AND attributes #>> '{docker,name}' IS NOT NULL ORDER BY 1",
        vec![ctx.name().into()],
    ))
    .all(ctx.db())
    .await?;
    Ok(names.into_iter().map(|name| name.name).collect())
}

pub async fn repositories_holding(ctx: &Ctx, digest: &Digest) -> Result<Vec<String>, ContentError> {
    #[derive(FromQueryResult)]
    struct Holder {
        repository: String,
    }
    let column = match digest.algorithm() {
        Algorithm::Sha256 => "sha256",
        Algorithm::Sha512 => "sha512",
    };
    let sql = format!(
        "SELECT DISTINCT repository FROM asset WHERE {column} = $1 AND format = 'docker' \
         AND kind = 'blob' AND path = $2 AND repository <> $3 ORDER BY repository"
    );
    let holders = Holder::find_by_statement(Statement::from_sql_and_values(
        DbBackend::Postgres,
        &sql,
        vec![
            digest.hex().into(),
            blob_path(digest).into(),
            ctx.name().into(),
        ],
    ))
    .all(ctx.db())
    .await?;
    Ok(holders
        .into_iter()
        .map(|holder| holder.repository)
        .collect())
}

pub fn blob_attributes(digest: &Digest) -> Value {
    json!({"docker": {"digest": digest.to_string()}})
}

pub fn blob_put(digest: &Digest) -> PutAsset {
    PutAsset::cached(blob_path(digest))
        .kind(BLOB)
        .content_type(OCTET_STREAM)
        .attributes(blob_attributes(digest))
}

pub fn manifest_attributes(
    image: &str,
    digest: &Digest,
    manifest: &Manifest,
    tag: Option<&str>,
) -> Value {
    let kind = match tag {
        Some(tag) => crate::manifest::kind_for_tag(manifest.kind, tag),
        None => manifest.kind,
    };
    let mut docker = Map::new();
    docker.insert("name".to_owned(), image.into());
    docker.insert("digest".to_owned(), digest.to_string().into());
    docker.insert("mediaType".to_owned(), manifest.media_type.clone().into());
    docker.insert("artifactKind".to_owned(), kind.as_str().into());
    if let Some(artifact_type) = &manifest.artifact_type {
        docker.insert("artifactType".to_owned(), artifact_type.clone().into());
    }
    if let Some(subject) = &manifest.subject {
        docker.insert("subject".to_owned(), subject.to_string().into());
        if let Some(annotations) = &manifest.annotations {
            let kept = serde_json::to_vec(annotations)
                .map(|bytes| bytes.len() <= ANNOTATIONS_KEPT)
                .unwrap_or(false);
            match kept {
                true => docker.insert("annotations".to_owned(), Value::Object(annotations.clone())),
                false => docker.insert("largeAnnotations".to_owned(), true.into()),
            };
        }
    }
    if let Some(tag) = tag {
        docker.insert("tag".to_owned(), tag.into());
    }
    json!({ "docker": docker })
}

pub fn manifest_put(
    image: &str,
    reference: &str,
    digest: &Digest,
    manifest: &Manifest,
    tag: Option<&str>,
) -> PutAsset {
    let attributes = manifest_attributes(image, digest, manifest, tag);
    let mut put = PutAsset::cached(manifest_path(image, reference))
        .kind(if tag.is_some() { TAG } else { MANIFEST })
        .content_type(manifest.media_type.clone())
        .attributes(attributes);
    put.verified = false;
    if let Some(tag) = tag {
        let kind = crate::manifest::kind_for_tag(manifest.kind, tag);
        put = put.component(ComponentKey {
            namespace: String::new(),
            name: image.to_owned(),
            version: tag.to_owned(),
            kind: kind.as_str().to_owned(),
            attributes: json!({"docker": {
                "digest": digest.to_string(),
                "mediaType": manifest.media_type,
                "artifactKind": kind.as_str(),
            }}),
        });
    }
    put
}

pub async fn read_all(ctx: &Ctx, asset: &Asset, limit: usize) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::not_found(format!(
            "the blob of {} is missing",
            asset.path
        )));
    };
    let mut stream = content.stream;
    let mut collected = Vec::with_capacity(asset.size.clamp(0, limit as i64) as usize);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if collected.len() + chunk.len() > limit {
            return Err(ContentError::internal(std::io::Error::other(format!(
                "{} is larger than {limit} bytes",
                asset.path
            ))));
        }
        collected.extend_from_slice(&chunk);
    }
    Ok(collected)
}

pub async fn stream_of(ctx: &Ctx, asset: &Asset) -> Result<ContentStream, ContentError> {
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    store
        .read(asset.blob_id.into())
        .await?
        .map(|content| content.stream)
        .ok_or_else(|| ContentError::not_found(format!("the blob of {} is missing", asset.path)))
}

pub async fn adopt_blob(
    ctx: &Ctx,
    digest: &Digest,
    written: Written,
) -> Result<Asset, ContentError> {
    if let Some(existing) = find(ctx, &blob_path(digest)).await? {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        return Ok(existing);
    }
    store::adopt(ctx, blob_put(digest), &written).await
}

pub fn discard(ctx: &Ctx, written: &Written) {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
}
