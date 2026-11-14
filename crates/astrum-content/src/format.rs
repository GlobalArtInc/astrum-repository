use std::collections::BTreeMap;
use std::path::PathBuf;

use async_trait::async_trait;
use axum::response::Response;
use serde::Serialize;

use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;

use crate::inventory::Inventory;
use crate::proxy::{self, Freshness};
use crate::store::{self, Asset, Component, PutAsset};
use crate::transfer::Transfer;
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

/// How the browse tree of the UI places the assets of a format.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrowseLayout {
    /// SQL expression giving the tree path of an asset, written over `a` (a row of `asset`) and
    /// `c` (its component, left joined, so its columns are null without one); NULL leaves the asset
    /// out of the tree. `None` shows every asset under its stored path.
    pub path: Option<&'static str>,
    /// Files the format answers without storing them as assets; the tree offers them at the root
    /// unless an asset of that path exists.
    pub generated: &'static [&'static str],
}

#[async_trait]
pub trait Format: Send + Sync + 'static {
    fn id(&self) -> &'static str;

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError>;

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError>;

    async fn gate_asset(&self, _ctx: &Ctx, _asset: &Asset) -> Result<(), ContentError> {
        Ok(())
    }

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

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        &[]
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout::default()
    }

    /// The transfer of components this format takes part in (promotion and replication), or `None`.
    ///
    /// A format with no state returns a `static` `GenericTransfer`. A format whose `after` hook needs
    /// its own state (a scheduler, signing keys, a lock map) builds its `GenericTransfer` once, in
    /// `new` or in a `std::sync::OnceLock` field, with `GenericTransfer::after_with` and a closure
    /// that captures `Arc` clones of that state, and returns a reference to the field:
    ///
    /// ```ignore
    /// struct Indexed {
    ///     pending: Arc<Mutex<BTreeSet<String>>>,
    ///     transfer: GenericTransfer,
    /// }
    ///
    /// impl Indexed {
    ///     fn new() -> Self {
    ///         let pending = Arc::new(Mutex::new(BTreeSet::new()));
    ///         let seen = pending.clone();
    ///         let transfer = base().after_with(move |ctx, promoted| {
    ///             let seen = seen.clone();
    ///             Box::pin(async move { schedule(&seen, ctx, promoted).await })
    ///         });
    ///         Self { pending, transfer }
    ///     }
    /// }
    ///
    /// fn transfer(&self) -> Option<&dyn Transfer> {
    ///     Some(&self.transfer)
    /// }
    /// ```
    ///
    /// The hook gets the repository and a `Promoted` listing what the transfer wrote; a failing hook
    /// does not fail the transfer, its error comes back in `Outcome::warnings`.
    fn transfer(&self) -> Option<&dyn Transfer> {
        None
    }

    fn can_protect(&self, _asset: &Asset) -> bool {
        false
    }

    fn protected(&self, _repository: &Repository, _asset: &Asset) -> Option<String> {
        None
    }

    async fn inventory(
        &self,
        _ctx: &Ctx,
        _component: &Component,
    ) -> Result<Option<Inventory>, ContentError> {
        Ok(None)
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
