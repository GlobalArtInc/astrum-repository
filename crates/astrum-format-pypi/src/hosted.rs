use std::collections::HashMap;

use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::{asset, component};
use astrum_repo::config::WritePolicy;
use bytes::Bytes;
use chrono::Utc;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::Value;

use crate::ID;
use crate::archive;
use crate::index::{File, Hashes, Project};
use crate::metadata::CoreMetadata;
use crate::name;

pub const METADATA_SUFFIX: &str = ".metadata";

pub async fn names(ctx: &Ctx) -> Result<Vec<String>, ContentError> {
    Ok(component::Entity::find()
        .select_only()
        .column(component::Column::Name)
        .distinct()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .order_by_asc(component::Column::Name)
        .into_tuple::<String>()
        .all(ctx.db())
        .await?)
}

fn yanked(attributes: &Value) -> Option<String> {
    match attributes.pointer("/pypi/yanked") {
        Some(Value::Bool(true)) => Some(String::new()),
        Some(Value::String(reason)) => Some(reason.clone()),
        _ => None,
    }
}

pub async fn project(ctx: &Ctx, project: &str) -> Result<Option<Project>, ContentError> {
    let components = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(component::Column::Name.eq(project))
        .all(ctx.db())
        .await?;
    if components.is_empty() {
        return Ok(None);
    }
    let versions: HashMap<i64, String> = components
        .iter()
        .map(|component| (component.id, component.version.clone()))
        .collect();
    let assets = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::ComponentId.is_in(versions.keys().copied()))
        .order_by_asc(asset::Column::Path)
        .all(ctx.db())
        .await?;
    let metadata: HashMap<&str, &str> = assets
        .iter()
        .filter_map(|asset| {
            asset
                .path
                .strip_suffix(METADATA_SUFFIX)
                .map(|file| (file, asset.sha256.as_str()))
        })
        .collect();
    let files: Vec<File> = assets
        .iter()
        .filter(|asset| !asset.path.ends_with(METADATA_SUFFIX))
        .map(|asset| File {
            filename: asset.path.rsplit('/').next().unwrap_or_default().to_owned(),
            url: asset.path.clone(),
            version: asset.component_id.and_then(|id| versions.get(&id).cloned()),
            hashes: [("sha256".to_owned(), asset.sha256.clone())].into(),
            requires_python: asset
                .attributes
                .pointer("/pypi/requires_python")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_owned),
            yanked: yanked(&asset.attributes),
            core_metadata: metadata
                .get(asset.path.as_str())
                .map(|sha| Hashes::from([("sha256".to_owned(), (*sha).to_owned())])),
            size: Some(asset.size.max(0) as u64),
            upload_time: Some(
                asset
                    .created_at
                    .with_timezone(&Utc)
                    .format("%Y-%m-%dT%H:%M:%S%.6fZ")
                    .to_string(),
            ),
        })
        .collect();
    if files.is_empty() {
        return Ok(None);
    }
    Ok(Some(Project {
        name: project.to_owned(),
        files,
        versions: Vec::new(),
    }))
}

pub struct Package {
    pub metadata: CoreMetadata,
    pub filename: String,
}

impl Package {
    pub fn path(&self) -> Result<(String, ComponentKey), ContentError> {
        let given = self
            .metadata
            .name()
            .ok_or_else(|| ContentError::BadRequest("the package has no name".to_owned()))?;
        let version = self
            .metadata
            .version()
            .ok_or_else(|| ContentError::BadRequest("the package has no version".to_owned()))?;
        if !name::is_valid_name(given) {
            return Err(ContentError::BadRequest(format!(
                "{given} is not a valid project name"
            )));
        }
        if !name::is_valid_version(version) {
            return Err(ContentError::BadRequest(format!(
                "{version} is not a version this repository can store"
            )));
        }
        if !name::is_safe_segment(&self.filename) || self.filename.ends_with(METADATA_SUFFIX) {
            return Err(ContentError::BadRequest(format!(
                "{} is not a valid file name",
                self.filename
            )));
        }
        let project = name::normalize(given);
        let path = format!("/packages/{project}/{version}/{}", self.filename);
        let key = ComponentKey {
            namespace: String::new(),
            name: project,
            version: version.to_owned(),
            kind: String::new(),
            attributes: self.metadata.component_attributes(),
        };
        Ok((path, key))
    }
}

pub fn file_name(given: Option<&str>) -> Result<String, ContentError> {
    let given = given
        .map(|name| name.rsplit(['/', '\\']).next().unwrap_or(name).trim())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| ContentError::BadRequest("the uploaded file has no name".to_owned()))?;
    Ok(given.to_owned())
}

pub async fn authorize(ctx: &Ctx, path: &str) -> Result<(), ContentError> {
    let redeploys = ctx
        .repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
        == WritePolicy::Allow;
    let exists = store::find(ctx.db(), ctx.name(), path).await?.is_some();
    ctx.require(
        if exists && redeploys {
            View::Edit
        } else {
            View::Add
        },
        path,
    )
}

pub async fn store_package(
    ctx: &Ctx,
    package: &Package,
    body: astrum_blob::ContentStream,
) -> Result<Asset, ContentError> {
    let (path, key) = package.path()?;
    authorize(ctx, &path).await?;
    let put = PutAsset::hosted(ctx, path.clone())
        .component(key.clone())
        .kind("package")
        .content_type(name::content_type(&package.filename))
        .attributes(package.metadata.asset_attributes());
    let stored = store::put(ctx, put, body).await?;
    if name::is_wheel(&package.filename) {
        companion_metadata(ctx, &stored, key).await;
    }
    Ok(stored)
}

async fn companion_metadata(ctx: &Ctx, wheel: &Asset, key: ComponentKey) {
    let found = async {
        let store = ctx.content.blob_stores().get(&wheel.blob_store).await?;
        let source = archive::Blob {
            store,
            id: wheel.blob_id.into(),
            size: wheel.size.max(0) as u64,
        };
        archive::metadata(&source, &wheel.path).await
    }
    .await;
    let text = match found {
        Ok(Some(text)) => text,
        Ok(None) => {
            tracing::info!(
                repository = ctx.name(),
                path = wheel.path,
                "the wheel has no METADATA file"
            );
            return;
        }
        Err(error) => {
            tracing::info!(repository = ctx.name(), path = wheel.path, %error, "the wheel's METADATA cannot be read");
            return;
        }
    };
    let mut put = PutAsset::hosted(ctx, format!("{}{METADATA_SUFFIX}", wheel.path))
        .component(key)
        .kind("metadata")
        .content_type("text/plain");
    put.write_policy = Some(WritePolicy::Allow);
    put.check_content_type = false;
    let body: astrum_blob::ContentStream = Box::pin(futures_util::stream::once(async move {
        Ok::<_, std::io::Error>(Bytes::from(text))
    }));
    if let Err(error) = store::put(ctx, put, body).await {
        tracing::warn!(repository = ctx.name(), path = wheel.path, %error, "the wheel's METADATA was not stored");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, version: &str, filename: &str) -> Package {
        Package {
            metadata: CoreMetadata::from_form([("name", name), ("version", version)]),
            filename: filename.to_owned(),
        }
    }

    #[test]
    fn packages_land_under_their_normalized_name() {
        let (path, key) = package("Foo_Bar", "2.0", "other-9.9.tar.gz")
            .path()
            .unwrap();
        assert_eq!(path, "/packages/foo-bar/2.0/other-9.9.tar.gz");
        assert_eq!(
            (key.name.as_str(), key.version.as_str()),
            ("foo-bar", "2.0")
        );
        assert_eq!(key.namespace, "");
    }

    #[test]
    fn bad_names_versions_and_files_are_refused() {
        for (name, version, filename) in [
            ("", "1.0", "a-1.0.tar.gz"),
            ("a/b", "1.0", "a-1.0.tar.gz"),
            ("a", "../1", "a-1.0.tar.gz"),
            ("a", "1.0", ".."),
            ("a", "1.0", "a-1.0-py3-none-any.whl.metadata"),
        ] {
            assert!(
                package(name, version, filename).path().is_err(),
                "{name} {version} {filename}"
            );
        }
        assert_eq!(
            file_name(Some("C:\\dist\\a-1.0.tar.gz")).unwrap(),
            "a-1.0.tar.gz"
        );
        assert_eq!(
            file_name(Some("dist/a-1.0.tar.gz")).unwrap(),
            "a-1.0.tar.gz"
        );
        assert!(file_name(Some("  ")).is_err());
        assert!(file_name(None).is_err());
    }
}
