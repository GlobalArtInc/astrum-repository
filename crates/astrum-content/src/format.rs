use std::collections::BTreeMap;
use std::path::PathBuf;

use async_trait::async_trait;
use axum::response::Response;
use serde::Serialize;

use astrum_repo::RepositoryType;

use crate::proxy::{self, Freshness};
use crate::store::{self, Asset, PutAsset};
use crate::{ContentError, ContentRequest, Ctx, group};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchTarget {
    Component,
    Asset,
}

#[derive(Clone, Copy, Debug)]
pub struct SearchKey {
    pub param: &'static str,
    pub target: SearchTarget,
    pub pointer: &'static [&'static str],
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadField {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub description: Option<&'static str>,
    pub optional: bool,
    pub group: Option<&'static str>,
    pub group_help_text: Option<&'static str>,
    pub options: Option<Vec<&'static str>>,
}

impl UploadField {
    pub fn text(name: &'static str, optional: bool) -> Self {
        Self {
            name,
            kind: "STRING",
            description: None,
            optional,
            group: None,
            group_help_text: None,
            options: None,
        }
    }

    pub fn file(name: &'static str) -> Self {
        Self {
            kind: "FILE",
            ..Self::text(name, false)
        }
    }

    pub fn describe(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn in_group(mut self, group: &'static str) -> Self {
        self.group = Some(group);
        self
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadSpec {
    pub format: &'static str,
    pub multiple_upload: bool,
    pub component_fields: Vec<UploadField>,
    pub asset_fields: Vec<UploadField>,
}

#[derive(Debug)]
pub struct UploadedFile {
    pub filename: Option<String>,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Default)]
pub struct UploadForm {
    pub fields: BTreeMap<String, String>,
    pub files: BTreeMap<String, UploadedFile>,
}

impl UploadForm {
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    pub async fn stream(file: &UploadedFile) -> Result<astrum_blob::ContentStream, ContentError> {
        use futures_util::StreamExt;
        let opened = tokio::fs::File::open(&file.path)
            .await
            .map_err(ContentError::internal)?;
        let reader = tokio_util::io::ReaderStream::new(opened);
        Ok(reader.boxed())
    }
}

impl Drop for UploadForm {
    fn drop(&mut self) {
        for file in self.files.values() {
            let _ = std::fs::remove_file(&file.path);
        }
    }
}

#[async_trait]
pub trait Format: Send + Sync + 'static {
    fn id(&self) -> &'static str;

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError>;

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError>;

    fn upload_spec(&self) -> Option<UploadSpec> {
        None
    }

    async fn upload(&self, _ctx: Ctx, _form: UploadForm) -> Result<(), ContentError> {
        Err(ContentError::BadRequest(format!(
            "{} repositories do not take uploads through the components API",
            self.id()
        )))
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        &[]
    }
}

pub async fn fetch(
    format: &dyn Format,
    ctx: &Ctx,
    path: &str,
    freshness: Freshness,
    describe: impl Fn() -> PutAsset,
) -> Result<Option<Asset>, ContentError> {
    match ctx.kind() {
        RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
        RepositoryType::Proxy => proxy::fetch(ctx, path, None, freshness, describe).await,
        RepositoryType::Group => {
            Ok(
                group::first(ctx, |member| async move { format.get(&member, path).await })
                    .await?
                    .map(|(_, asset)| asset),
            )
        }
    }
}
