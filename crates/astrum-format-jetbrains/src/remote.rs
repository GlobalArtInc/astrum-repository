use astrum_blob::ContentStream;
use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use chrono::{DateTime, FixedOffset, Utc};
use futures_util::TryStreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::form_urlencoded;

use crate::builds::Build;
use crate::hosted::{component_key, read_plugin};
use crate::plugin::{artifact_path, valid_id, valid_version};
use crate::query::Query;
use crate::{ARTIFACT, ID, ZIP_TYPE};

const ALIAS: &str = "alias";
const LIST: &str = "list";
const MAX_BUILD: usize = 64;
const MAX_LIST: usize = 256 * 1024 * 1024;

#[derive(Default)]
pub struct Remote;

enum Target {
    Missing,
    Path(String),
}

fn target_of(alias: &Asset) -> Option<Target> {
    let node = alias.attributes.get(ID)?;
    if node.get("missing").and_then(Value::as_bool) == Some(true) {
        return Some(Target::Missing);
    }
    node.get("target")
        .and_then(Value::as_str)
        .map(|path| Target::Path(path.to_owned()))
}

fn parse_date(value: Option<&str>) -> Option<DateTime<FixedOffset>> {
    let parsed = httpdate::parse_http_date(value?).ok()?;
    Some(DateTime::<Utc>::from(parsed).fixed_offset())
}

fn safe(text: &str) -> String {
    text.chars()
        .map(
            |c| match c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                true => c,
                false => '_',
            },
        )
        .collect()
}

fn checked_build(query: &Query) -> Result<&str, ContentError> {
    let build = query.build.as_deref().unwrap_or_default();
    match build.is_empty() || (build.len() <= MAX_BUILD && Build::parse(build).is_some()) {
        true => Ok(build),
        false => Err(ContentError::BadRequest(format!(
            "{build:?} is not a build number"
        ))),
    }
}

fn encode<K: AsRef<str>, V: AsRef<str>>(pairs: &[(K, V)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs {
        serializer.append_pair(name.as_ref(), value.as_ref());
    }
    serializer.finish()
}

fn digest(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))[..12].to_owned()
}

pub async fn read_list(ctx: &Ctx, asset: &Asset) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let content = store
        .read(asset.blob_id.into())
        .await?
        .ok_or_else(|| ContentError::not_found(asset.path.clone()))?;
    let mut stream = content.stream;
    let mut collected = Vec::new();
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if collected.len() + chunk.len() > MAX_LIST {
            return Err(ContentError::BadGateway(
                "the cached plugin list is too large".to_owned(),
            ));
        }
        collected.extend_from_slice(&chunk);
    }
    Ok(collected)
}

impl Remote {
    pub async fn listing(&self, ctx: &Ctx, query: &Query) -> Result<Option<Asset>, ContentError> {
        checked_build(query)?;
        let mut pairs: Vec<(&str, String)> = Vec::new();
        if let Some(build) = &query.build {
            pairs.push(("build", build.clone()));
        }
        if !query.channel.is_empty() {
            pairs.push(("channel", query.channel.clone()));
        }
        let mut ids = query.ids.clone();
        ids.sort();
        ids.dedup();
        for id in ids {
            pairs.push(("pluginId", id));
        }
        let canonical = encode(&pairs);
        let label = match &query.build {
            Some(build) => safe(build.get(..build.len().min(MAX_BUILD)).unwrap_or_default()),
            None => "all".to_owned(),
        };
        let path = format!("/plugins/list/{label}-{}.xml", digest(&canonical));
        let url = proxy::remote_url(&ctx.repository, "plugins/list", Some(&canonical))?;
        let described = path.clone();
        proxy::fetch_from(
            ctx,
            &path,
            Freshness::Metadata,
            || async { Ok(Some(Origin::new(url))) },
            || {
                PutAsset::cached(described)
                    .kind(LIST)
                    .content_type("text/xml")
            },
        )
        .await
    }

    pub async fn managed(
        &self,
        ctx: &Ctx,
        id: &str,
        query: &Query,
    ) -> Result<Option<Asset>, ContentError> {
        if !valid_id(id) {
            return Err(ContentError::BadRequest(format!(
                "{id:?} is not a plugin id"
            )));
        }
        let build = checked_build(query)?;
        let mut pairs = vec![("action", "download"), ("id", id)];
        if !build.is_empty() {
            pairs.push(("build", build));
        }
        if let Some(os) = &query.os {
            pairs.push(("os", os));
        }
        if let Some(arch) = &query.arch {
            pairs.push(("arch", arch));
        }
        let alias = format!(
            "/pluginManager/{id}/{}.json",
            safe(&format!(
                "{}-{}-{}-{}",
                if build.is_empty() { "any" } else { build },
                if query.channel.is_empty() {
                    "stable"
                } else {
                    &query.channel
                },
                query.os.as_deref().unwrap_or("any"),
                query.arch.as_deref().unwrap_or("any")
            ))
        );
        let url = proxy::remote_url(&ctx.repository, "pluginManager", Some(&encode(&pairs)))?;
        self.resolve(ctx, &alias, url).await
    }

    pub async fn version(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        if !valid_id(id) || !valid_version(version) {
            return Err(ContentError::BadRequest(format!(
                "{id:?} {version:?} is not a plugin version"
            )));
        }
        for extension in ["zip", "jar"] {
            if let Some(cached) =
                store::find(ctx.db(), ctx.name(), &artifact_path(id, version, extension)).await?
            {
                return Ok(Some(cached));
            }
        }
        let pairs = [("pluginId", id), ("version", version)];
        let alias = format!("/plugin/download/{id}/{version}.json");
        let url = proxy::remote_url(&ctx.repository, "plugin/download", Some(&encode(&pairs)))?;
        self.resolve(ctx, &alias, url).await
    }

    async fn resolve(
        &self,
        ctx: &Ctx,
        alias_path: &str,
        url: url::Url,
    ) -> Result<Option<Asset>, ContentError> {
        let alias = store::find(ctx.db(), ctx.name(), alias_path).await?;
        let known = match alias.as_ref().and_then(target_of) {
            Some(Target::Path(path)) => store::find(ctx.db(), ctx.name(), &path).await?,
            _ => None,
        };
        if let Some(alias) = &alias
            && proxy::is_fresh(ctx, alias, Freshness::Metadata)
        {
            match target_of(alias) {
                Some(Target::Missing) => return Ok(None),
                Some(Target::Path(_)) if known.is_some() => return Ok(known),
                _ => {}
            }
        }
        let mut conditional: Vec<(header::HeaderName, String)> = Vec::new();
        if let (Some(alias), Some(_)) = (&alias, &known)
            && let Some(etag) = &alias.remote_etag
        {
            conditional.push((header::IF_NONE_MATCH, etag.clone()));
        }
        let stale = |error: ContentError| match known.clone() {
            Some(cached) => {
                tracing::info!(repository = ctx.name(), alias = alias_path, %error, "serving a stale copy");
                Ok(Some(cached))
            }
            None => Err(error),
        };
        let remote = match proxy::request(ctx, url, &conditional).await {
            Ok(remote) => remote,
            Err(error) => return stale(error),
        };
        match remote.status {
            StatusCode::OK => {
                let etag = remote.header("etag").map(str::to_owned);
                let modified = parse_date(remote.header("last-modified"));
                if let (Some(alias), Some(cached)) = (&alias, &known)
                    && etag.is_some()
                    && alias.remote_etag == etag
                {
                    store::touch(ctx, alias.id, None, None).await?;
                    return Ok(Some(cached.clone()));
                }
                let cached = self.adopt(ctx, remote.body, etag.clone(), modified).await?;
                self.remember(ctx, alias_path, json!({ "target": cached.path }), etag)
                    .await?;
                Ok(Some(cached))
            }
            StatusCode::NOT_MODIFIED => match (&alias, known) {
                (Some(alias), Some(cached)) => {
                    store::touch(ctx, alias.id, None, None).await?;
                    Ok(Some(cached))
                }
                _ => Err(ContentError::BadGateway(
                    "the remote answered 304 to a plain request".to_owned(),
                )),
            },
            StatusCode::NOT_FOUND | StatusCode::GONE => {
                self.remember(ctx, alias_path, json!({ "missing": true }), None)
                    .await?;
                Ok(None)
            }
            status => stale(ContentError::BadGateway(format!(
                "the remote answered {status}"
            ))),
        }
    }

    async fn adopt(
        &self,
        ctx: &Ctx,
        body: ContentStream,
        etag: Option<String>,
        modified: Option<DateTime<FixedOffset>>,
    ) -> Result<Asset, ContentError> {
        let written = store::write_blob(ctx, body).await?;
        let found = match read_plugin(ctx, &written).await {
            Ok(found) => found,
            Err(refusal) => {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                return Err(ContentError::BadGateway(format!(
                    "the remote did not send a plugin archive: {}",
                    refusal.into_error()
                )));
            }
        };
        if let Err(message) = found.plugin.validate() {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(ContentError::BadGateway(format!(
                "the remote sent a plugin that cannot be cached: {message}"
            )));
        }
        let path = found.plugin.path(found.packaging);
        if let Some(existing) = store::find(ctx.db(), ctx.name(), &path).await? {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            store::touch(ctx, existing.id, etag, modified).await?;
            return Ok(existing);
        }
        let mut put = PutAsset::cached(path)
            .kind(ARTIFACT)
            .content_type(ZIP_TYPE)
            .component(component_key(&found, ""));
        put.remote_etag = etag;
        put.remote_last_modified = modified;
        store::adopt(ctx, put, &written).await
    }

    async fn remember(
        &self,
        ctx: &Ctx,
        alias_path: &str,
        node: Value,
        etag: Option<String>,
    ) -> Result<(), ContentError> {
        let mut put = PutAsset::cached(alias_path)
            .kind(ALIAS)
            .content_type("application/json")
            .attributes(json!({ ID: node }));
        put.remote_etag = etag;
        store::put(ctx, put, astrum_blob::content_of(Vec::new())).await?;
        Ok(())
    }
}
