use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use futures_util::TryStreamExt;
use serde_json::Value;

use crate::addon::{Addon, addon_path, message_key, valid_id, valid_key, valid_version};
use crate::hosted::{self, Refusal, component_key, read_addon};
use crate::updates::{Update, link};
use crate::{ADDON, METADATA, XPI_TYPE};

const MAX_DOCUMENT: u64 = 16 * 1024 * 1024;
const LOCALE: &str = "en-US";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Release {
    pub guid: String,
    pub name: String,
    pub version: String,
    pub url: String,
    pub hash: Option<String>,
    pub size: Option<u64>,
    pub min: Option<String>,
    pub max: Option<String>,
}

fn text(node: Option<&Value>) -> Option<String> {
    node.and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn localized(node: Option<&Value>) -> String {
    match node {
        Some(Value::String(name)) => name.trim().to_owned(),
        Some(Value::Object(names)) => names
            .get(LOCALE)
            .or_else(|| names.values().find(|value| value.is_string()))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned(),
        _ => String::new(),
    }
}

pub fn release_of(guid: &str, name: &str, node: &Value) -> Option<Release> {
    let version = text(node.get("version"))?;
    let file = node.get("file")?;
    let url = text(file.get("url"))?;
    let firefox = node.get("compatibility").and_then(|c| c.get("firefox"));
    let bound = |name: &str| text(firefox.and_then(|f| f.get(name)));
    (valid_id(guid) && valid_version(&version)).then(|| Release {
        guid: guid.to_owned(),
        name: name.to_owned(),
        version,
        url,
        hash: text(file.get("hash")),
        size: file.get("size").and_then(Value::as_u64),
        min: bound("min"),
        max: bound("max").filter(|max| max != "*"),
    })
}

pub fn current_of(document: &Value) -> Option<Release> {
    let guid = text(document.get("guid"))?;
    let name = localized(document.get("name"));
    release_of(&guid, &name, document.get("current_version")?)
}

fn checked(key: &str) -> Result<(), ContentError> {
    match valid_key(key) {
        true => Ok(()),
        false => Err(ContentError::BadRequest(format!(
            "{key:?} is neither an add-on id nor a slug"
        ))),
    }
}

async fn read_document(ctx: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > MAX_DOCUMENT {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
    }
    let blob_store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = blob_store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn expected_hash(hash: &str) -> Option<&str> {
    let (algorithm, value) = hash.split_once(':')?;
    algorithm.eq_ignore_ascii_case("sha256").then_some(value)
}

#[derive(Default)]
pub struct Remote;

impl Remote {
    async fn document(
        &self,
        ctx: &Ctx,
        cache: String,
        api: String,
    ) -> Result<Option<Value>, ContentError> {
        let described = cache.clone();
        let found = proxy::fetch_from(
            ctx,
            &cache,
            Freshness::Metadata,
            || async {
                let url = proxy::remote_url(&ctx.repository, &api, None)?;
                Ok(Some(
                    Origin::new(url).header(header::ACCEPT, "application/json"),
                ))
            },
            || {
                PutAsset::cached(described)
                    .kind(METADATA)
                    .content_type("application/json")
            },
        )
        .await?;
        let Some(found) = found else {
            return Ok(None);
        };
        let body = read_document(ctx, &found).await?;
        match serde_json::from_slice(&body) {
            Ok(value) => Ok(Some(value)),
            Err(error) => {
                tracing::info!(repository = ctx.name(), path = cache, %error, "the remote answered a document that is not JSON");
                store::delete(ctx, &cache).await?;
                Err(ContentError::BadGateway(format!(
                    "the remote answer for {api} is not JSON"
                )))
            }
        }
    }

    pub async fn current(&self, ctx: &Ctx, key: &str) -> Result<Option<Release>, ContentError> {
        checked(key)?;
        let Some(document) = self
            .document(
                ctx,
                format!("/-/addon/{key}.json"),
                format!("api/v5/addons/addon/{key}/"),
            )
            .await?
        else {
            return Ok(None);
        };
        match document.get("guid").and_then(Value::as_str) {
            Some(_) => Ok(current_of(&document)),
            None => Err(ContentError::BadGateway(format!(
                "the remote did not describe the add-on {key}"
            ))),
        }
    }

    async fn release(
        &self,
        ctx: &Ctx,
        guid: &str,
        version: &str,
    ) -> Result<Option<Release>, ContentError> {
        let Some(document) = self
            .document(
                ctx,
                format!("/-/addon/{guid}/{version}.json"),
                format!("api/v5/addons/addon/{guid}/versions/{version}/"),
            )
            .await?
        else {
            return Ok(None);
        };
        Ok(release_of(guid, "", &document))
    }

    pub async fn download(&self, ctx: &Ctx, release: &Release) -> Result<Asset, ContentError> {
        let path = addon_path(&release.guid, &release.version);
        if let Some(cached) = store::find(ctx.db(), ctx.name(), &path).await? {
            return Ok(cached);
        }
        let url = url::Url::parse(&release.url)
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https"))
            .ok_or_else(|| {
                ContentError::BadGateway(format!(
                    "the remote gave {:?} as the file of {} {}",
                    release.url, release.guid, release.version
                ))
            })?;
        let remote = proxy::request(ctx, url, &[]).await?;
        match remote.status {
            StatusCode::OK => {}
            StatusCode::NOT_FOUND | StatusCode::GONE => {
                return Err(ContentError::not_found(format!(
                    "the remote has no file for {} {}",
                    release.guid, release.version
                )));
            }
            status => {
                return Err(ContentError::BadGateway(format!(
                    "the remote answered {status} for the file of {} {}",
                    release.guid, release.version
                )));
            }
        }
        let written = store::write_blob(ctx, remote.body).await?;
        let discard = || store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        if let Some(expected) = release.hash.as_deref().and_then(expected_hash)
            && !expected.eq_ignore_ascii_case(&written.digests.sha256)
        {
            discard();
            tracing::warn!(
                repository = ctx.name(),
                guid = release.guid,
                version = release.version,
                expected,
                found = written.digests.sha256,
                "a fetched add-on does not match the hash of the remote"
            );
            return Err(ContentError::BadGateway(format!(
                "the file of {} {} from the remote does not match its SHA-256 hash",
                release.guid, release.version
            )));
        }
        let found = match read_addon(ctx, &written).await {
            Ok(found) => found,
            Err(refusal) => {
                discard();
                return Err(match refusal {
                    Refusal::Content(message) => ContentError::BadGateway(format!(
                        "the remote did not send a Firefox add-on: {message}"
                    )),
                    Refusal::Request(error) => error,
                });
            }
        };
        if !found.id.is_empty() && found.id != release.guid {
            discard();
            return Err(ContentError::BadGateway(format!(
                "the remote sent the add-on {} for {}",
                found.id, release.guid
            )));
        }
        let name = match found.name.is_empty() || message_key(&found.name).is_some() {
            true if !release.name.is_empty() => release.name.clone(),
            _ => found.name.clone(),
        };
        let addon = Addon {
            id: release.guid.clone(),
            name,
            version: release.version.clone(),
            ..found
        };
        let put = PutAsset::cached(path)
            .kind(ADDON)
            .content_type(XPI_TYPE)
            .component(component_key(&addon));
        store::adopt(ctx, put, &written).await
    }

    async fn cached_newest(&self, ctx: &Ctx, key: &str) -> Result<Option<Asset>, ContentError> {
        if !valid_id(key) {
            return Ok(None);
        }
        Ok(hosted::newest(hosted::stored(ctx, Some(key)).await?).map(|found| found.asset))
    }

    pub async fn latest(&self, ctx: &Ctx, key: &str) -> Result<Option<Asset>, ContentError> {
        let found = match self.current(ctx, key).await {
            Ok(Some(release)) => self.download(ctx, &release).await.map(Some),
            Ok(None) => return Ok(None),
            Err(error) => Err(error),
        };
        match found {
            Ok(found) => Ok(found),
            Err(error @ ContentError::BadRequest(_)) => Err(error),
            Err(error) => match self.cached_newest(ctx, key).await? {
                Some(cached) => {
                    tracing::info!(repository = ctx.name(), key, %error, "serving the newest cached version");
                    Ok(Some(cached))
                }
                None => Err(error),
            },
        }
    }

    pub async fn version(
        &self,
        ctx: &Ctx,
        key: &str,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        checked(key)?;
        if !valid_version(version) {
            return Err(ContentError::BadRequest(format!(
                "{version:?} is not an add-on version"
            )));
        }
        if valid_id(key)
            && let Some(cached) =
                store::find(ctx.db(), ctx.name(), &addon_path(key, version)).await?
        {
            return Ok(Some(cached));
        }
        let Some(current) = self.current(ctx, key).await? else {
            return Ok(None);
        };
        if current.version == version {
            return self.download(ctx, &current).await.map(Some);
        }
        if let Some(cached) =
            store::find(ctx.db(), ctx.name(), &addon_path(&current.guid, version)).await?
        {
            return Ok(Some(cached));
        }
        match self.release(ctx, &current.guid, version).await? {
            Some(release) => self.download(ctx, &release).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn updates(
        &self,
        ctx: &Ctx,
        key: &str,
        base: &str,
    ) -> Result<Option<(String, Vec<Update>)>, ContentError> {
        let release = match self.current(ctx, key).await {
            Ok(release) => release,
            Err(error @ ContentError::BadRequest(_)) => return Err(error),
            Err(error) => {
                let cached = match valid_id(key) {
                    true => hosted::stored(ctx, Some(key)).await?,
                    false => Vec::new(),
                };
                if cached.is_empty() {
                    return Err(error);
                }
                tracing::info!(repository = ctx.name(), key, %error, "answering with the cached versions");
                let updates = cached.iter().map(|found| found.update(base)).collect();
                return Ok(Some((key.to_owned(), updates)));
            }
        };
        let Some(release) = release else {
            return Ok(None);
        };
        let cached = store::find(
            ctx.db(),
            ctx.name(),
            &addon_path(&release.guid, &release.version),
        )
        .await?;
        let hash = match (&release.hash, cached) {
            (Some(hash), _) if expected_hash(hash).is_some() => Some(hash.to_ascii_lowercase()),
            (_, Some(cached)) => crate::updates::sha256_hash(&cached.sha256),
            _ => None,
        };
        let update = Update {
            link: link(base, &release.guid, &release.version),
            version: release.version,
            hash,
            min: release.min,
            max: release.max,
        };
        Ok(Some((release.guid, vec![update])))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_current_version_of_an_addon_document_is_read() {
        let document = json!({
            "id": 607454,
            "guid": "uBlock0@raymondhill.net",
            "slug": "ublock-origin",
            "name": {"de": "uBlock Origin DE", "en-US": "uBlock Origin"},
            "current_version": {
                "version": "1.61.2",
                "compatibility": {"firefox": {"min": "115.0", "max": "*"}},
                "file": {"url": "https://addons.example/file/1/ublock.xpi", "hash": "sha256:ab", "size": 3}
            }
        });
        let release = current_of(&document).unwrap();
        assert_eq!(release.guid, "uBlock0@raymondhill.net");
        assert_eq!(release.name, "uBlock Origin");
        assert_eq!(release.version, "1.61.2");
        assert_eq!(release.url, "https://addons.example/file/1/ublock.xpi");
        assert_eq!(release.hash.as_deref(), Some("sha256:ab"));
        assert_eq!(release.size, Some(3));
        assert_eq!(release.min.as_deref(), Some("115.0"));
        assert_eq!(release.max, None);
        assert_eq!(
            current_of(&json!({"guid": "a@b", "current_version": null})),
            None
        );
        assert_eq!(
            current_of(&json!({"guid": "../x", "current_version": document["current_version"]})),
            None
        );
        assert_eq!(localized(Some(&json!({"fr": "Nom"}))), "Nom");
        assert_eq!(localized(Some(&json!("Plain"))), "Plain");
    }

    #[test]
    fn hashes_name_their_algorithm() {
        assert_eq!(expected_hash("sha256:abc"), Some("abc"));
        assert_eq!(expected_hash("SHA256:abc"), Some("abc"));
        assert_eq!(expected_hash("sha1:abc"), None);
        assert_eq!(expected_hash("abc"), None);
    }
}
