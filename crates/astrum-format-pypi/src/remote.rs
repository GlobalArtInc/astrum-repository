use std::sync::Mutex;

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use astrum_db::entity::component;
use axum::http::header;
use futures_util::TryStreamExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};

use crate::archive;
use crate::hosted::METADATA_SUFFIX;
use crate::index::{File, Project};
use crate::metadata::CoreMetadata;
use crate::{html, json, name};

const ACCEPT: &str = "application/vnd.pypi.simple.v1+json, application/vnd.pypi.simple.v1+html;q=0.2, text/html;q=0.1";
const MAX_PAGE: usize = 128 * 1024 * 1024;
const MAX_ROOT: usize = 512 * 1024 * 1024;

pub fn index_url(ctx: &Ctx, suffix: &str) -> Result<url::Url, ContentError> {
    let remote = ctx
        .repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .unwrap_or_default();
    let ends_in_simple = url::Url::parse(remote)
        .ok()
        .is_some_and(|url| url.path().trim_end_matches('/').ends_with("/simple"));
    let path = match ends_in_simple {
        true => format!("/{suffix}"),
        false => format!("/simple/{suffix}"),
    };
    proxy::remote_url(&ctx.repository, &path, None)
}

pub async fn read(ctx: &Ctx, asset: &Asset, limit: usize) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as usize > limit {
        return Err(ContentError::BadGateway(
            "the remote index is too large".to_owned(),
        ));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::not_found(asset.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn is_json(asset: &Asset, body: &[u8]) -> bool {
    asset.content_type.contains("json")
        || body
            .iter()
            .find(|byte| !byte.is_ascii_whitespace())
            .is_some_and(|byte| *byte == b'{')
}

async fn index(ctx: &Ctx, suffix: &str) -> Result<Option<(Asset, url::Url)>, ContentError> {
    let path = format!("/simple/{suffix}");
    let url = index_url(ctx, suffix)?;
    let origin = Origin::new(url.clone()).header(header::ACCEPT, ACCEPT);
    let found = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Metadata,
        || async { Ok(Some(origin)) },
        || PutAsset::cached(&path).kind("index"),
    )
    .await?;
    Ok(found.map(|asset| (asset, url)))
}

pub async fn page(ctx: &Ctx, project: &str) -> Result<Option<Project>, ContentError> {
    let Some((asset, url)) = index(ctx, &format!("{project}/")).await? else {
        return Ok(None);
    };
    let body = read(ctx, &asset, MAX_PAGE).await?;
    let parsed = match is_json(&asset, &body) {
        true => json::parse_project(project, &body, &url)
            .map_err(|error| ContentError::BadGateway(error.to_string()))?,
        false => html::parse_project(project, &String::from_utf8_lossy(&body), &url),
    };
    Ok(Some(Project {
        name: project.to_owned(),
        ..parsed
    }))
}

pub async fn names(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    let Some((asset, _)) = index(ctx, "").await? else {
        return Ok(Vec::new());
    };
    let body = read(ctx, &asset, MAX_ROOT).await?;
    match is_json(&asset, &body) {
        true => {
            json::parse_root(&body).map_err(|error| ContentError::BadGateway(error.to_string()))
        }
        false => Ok(html::parse_root(&String::from_utf8_lossy(&body))),
    }
}

pub struct Wanted {
    pub project: String,
    pub version: String,
    pub filename: String,
    pub metadata: bool,
}

pub fn wanted(path: &str) -> Option<Wanted> {
    let rest = path.strip_prefix("/packages/")?;
    let parts: Vec<&str> = rest.split('/').collect();
    let [project, version, filename] = parts.as_slice() else {
        return None;
    };
    let (filename, metadata) = match filename.strip_suffix(METADATA_SUFFIX) {
        Some(file) if !file.is_empty() => (file, true),
        _ => (*filename, false),
    };
    Some(Wanted {
        project: (*project).to_owned(),
        version: (*version).to_owned(),
        filename: filename.to_owned(),
        metadata,
    })
}

fn origin_of(file: &File, metadata: bool) -> Result<Option<Origin>, ContentError> {
    let mut url = url::Url::parse(&file.url)
        .map_err(|error| ContentError::BadGateway(format!("the file URL is not valid: {error}")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ContentError::BadGateway(format!(
            "the file URL uses {}, which the proxy does not fetch",
            url.scheme()
        )));
    }
    if metadata {
        if file.core_metadata.is_none() {
            return Ok(None);
        }
        let path = format!("{}{METADATA_SUFFIX}", url.path());
        url.set_path(&path);
        url.set_query(None);
    }
    Ok(Some(Origin::new(url)))
}

pub async fn file(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let Some(wanted) = wanted(path) else {
        return store::find(ctx.db(), ctx.name(), path).await;
    };
    let project = name::normalize(&wanted.project);
    let suffix = if wanted.metadata { METADATA_SUFFIX } else { "" };
    if !path.starts_with(&format!("/packages/{project}/")) {
        return Ok(None);
    }
    let existing = store::find(ctx.db(), ctx.name(), path).await?;
    let known = known_attributes(ctx, &project, &wanted.version).await?;
    let listed: Mutex<Option<File>> = Mutex::new(None);
    let fetched = proxy::fetch_from(
        ctx,
        path,
        Freshness::Content,
        || async {
            let Some(page) = page(ctx, &project).await? else {
                return Ok(None);
            };
            let Some(file) = page
                .find(&wanted.filename)
                .filter(|file| format!("{}{suffix}", file.local_path(&project)) == path)
            else {
                return Ok(None);
            };
            let origin = origin_of(file, wanted.metadata)?;
            *listed
                .lock()
                .expect("the listed file lock is never poisoned") = Some(file.clone());
            Ok(origin)
        },
        || {
            let file = listed
                .lock()
                .expect("the listed file lock is never poisoned")
                .take()
                .unwrap_or_default();
            describe(path, &project, &wanted, &file, known)
        },
    )
    .await?;
    if let Some(asset) = &fetched
        && !wanted.metadata
        && existing.as_ref().map(|old| old.blob_id) != Some(asset.blob_id)
    {
        annotate(ctx, asset, &wanted.filename).await;
    }
    Ok(fetched)
}

async fn known_attributes(
    ctx: &Ctx,
    project: &str,
    version: &str,
) -> Result<Option<Value>, ContentError> {
    Ok(component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Namespace.eq(""))
        .filter(component::Column::Name.eq(project))
        .filter(component::Column::Version.eq(version))
        .one(ctx.db())
        .await?
        .map(|found| found.attributes))
}

fn describe(
    path: &str,
    project: &str,
    wanted: &Wanted,
    file: &File,
    known: Option<Value>,
) -> PutAsset {
    let mut pypi = serde_json::Map::new();
    pypi.insert("name".to_owned(), json!(project));
    pypi.insert("version".to_owned(), json!(wanted.version));
    if let Some(requires) = &file.requires_python {
        pypi.insert("requires_python".to_owned(), json!(requires));
    }
    let filename = match wanted.metadata {
        true => format!("{}{METADATA_SUFFIX}", wanted.filename),
        false => wanted.filename.clone(),
    };
    PutAsset::cached(path)
        .component(ComponentKey {
            namespace: String::new(),
            name: project.to_owned(),
            version: wanted.version.clone(),
            kind: String::new(),
            attributes: known
                .unwrap_or_else(|| json!({ "pypi": {"name": project, "version": wanted.version} })),
        })
        .kind(if wanted.metadata {
            "metadata"
        } else {
            "package"
        })
        .content_type(name::content_type(&filename))
        .attributes(json!({ "pypi": Value::Object(pypi) }))
}

async fn annotate(ctx: &Ctx, asset: &Asset, filename: &str) {
    let found = async {
        let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
        let source = archive::Blob {
            store,
            id: asset.blob_id.into(),
            size: asset.size.max(0) as u64,
        };
        archive::metadata(&source, filename).await
    }
    .await;
    let metadata = match found {
        Ok(Some(text)) => CoreMetadata::parse(&text),
        Ok(None) => return,
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path = asset.path, %error, "the package metadata cannot be read");
            return;
        }
    };
    let mut attributes = asset.attributes.clone();
    if let (Some(target), Value::Object(found)) = (
        attributes.get_mut("pypi").and_then(Value::as_object_mut),
        metadata.asset_attributes()["pypi"].take(),
    ) {
        target.extend(found);
    }
    let updated = async {
        store::set_attributes(ctx.db(), asset.id, attributes).await?;
        if let Some(component) = asset.component_id {
            store::set_component_attributes(ctx.db(), component, metadata.component_attributes())
                .await?;
        }
        Ok::<_, ContentError>(())
    }
    .await;
    if let Err(error) = updated {
        tracing::warn!(repository = ctx.name(), path = asset.path, %error, "the package metadata was not saved");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_paths_split_into_parts() {
        let found =
            wanted("/packages/six/1.17.0/six-1.17.0-py2.py3-none-any.whl.metadata").unwrap();
        assert_eq!(found.project, "six");
        assert_eq!(found.version, "1.17.0");
        assert_eq!(found.filename, "six-1.17.0-py2.py3-none-any.whl");
        assert!(found.metadata);
        assert!(wanted("/packages/six/six-1.0.tar.gz").is_none());
        assert!(wanted("/simple/six/").is_none());
        assert!(!wanted("/packages/six/1.0/.metadata").unwrap().metadata);
    }

    #[test]
    fn file_origins_must_be_http() {
        let file = File {
            url: "https://files.example/a/six-1.0-py3-none-any.whl".to_owned(),
            core_metadata: Some(Default::default()),
            ..Default::default()
        };
        assert_eq!(
            origin_of(&file, true).unwrap().unwrap().url.as_str(),
            "https://files.example/a/six-1.0-py3-none-any.whl.metadata"
        );
        let plain = File {
            core_metadata: None,
            ..file.clone()
        };
        assert!(origin_of(&plain, true).unwrap().is_none());
        let ftp = File {
            url: "ftp://files.example/a.whl".to_owned(),
            ..file
        };
        assert!(origin_of(&ftp, false).is_err());
    }
}
