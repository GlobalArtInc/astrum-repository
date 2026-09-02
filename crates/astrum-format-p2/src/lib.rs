mod bundle;
pub mod paths;
pub mod testing;
pub mod xml;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::zip;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, View, serve,
};
use astrum_db::entity::{asset, component};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use futures_util::{FutureExt, TryStreamExt};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::paths::{Artifact, Encoding, Metadata, REMOTE, Route};
use crate::xml::{Descriptor, Descriptors};

pub const ID: &str = "p2";
pub const METADATA: &str = "metadata";
pub const ARTIFACT: &str = "artifact";

const MAX_METADATA: u64 = 512 * 1024 * 1024;
const PARSED_KEPT: usize = 8;
const ATTEMPTS: usize = 3;

const SEARCH_KEYS: &[SearchKey] = &[SearchKey {
    param: "p2.pluginName",
    target: SearchTarget::Component,
    pointer: &["p2", "pluginName"],
}];

type ParsedKey = (String, i64, Uuid);

#[derive(Default)]
pub struct P2 {
    parsed: Mutex<HashMap<ParsedKey, Arc<Descriptors>>>,
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a p2 proxy repository. Add <code>{url}/</code> as a software site in Eclipse (Help, Install New Software) or as a repository with the <code>p2</code> layout in a Tycho build.</p>\n    <p>The cached files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

async fn read(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn remote_base(ctx: &Ctx) -> Result<url::Url, ContentError> {
    proxy::remote_url(&ctx.repository, "/", None)
}

fn remote_of(ctx: &Ctx, path: &str) -> Result<url::Url, ContentError> {
    match paths::remote_target(path) {
        Some(url) => Ok(url),
        None => proxy::remote_url(&ctx.repository, path, None),
    }
}

/// Whether a location outside the remote of the proxy is a child that a composite repository
/// cached by this proxy names. Only those are fetched, so the proxy cannot be steered to other
/// addresses.
async fn named_by_a_composite(ctx: &Ctx, url: &url::Url) -> Result<bool, ContentError> {
    let rows = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(
            Condition::any()
                .add(asset::Column::Path.like("%/compositeContent.%"))
                .add(asset::Column::Path.like("%/compositeArtifacts.%")),
        )
        .all(ctx.db())
        .await?;
    let wanted = url.as_str();
    Ok(rows.iter().any(|row| {
        row.attributes
            .get(ID)
            .and_then(|p2| p2.get("remoteChildren"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|child| paths::under(wanted, child))
    }))
}

async fn origin(ctx: &Ctx, path: &str) -> Result<Option<Origin>, ContentError> {
    match paths::remote_target(path) {
        Some(url) => Ok(named_by_a_composite(ctx, &url)
            .await?
            .then(|| Origin::new(url))),
        None => proxy::remote_url(&ctx.repository, path, None).map(|url| Some(Origin::new(url))),
    }
}

fn p2_attributes(found: &Asset) -> Option<&Map<String, Value>> {
    found.attributes.get(ID).and_then(Value::as_object)
}

fn checked(found: &Asset) -> bool {
    p2_attributes(found)
        .and_then(|p2| p2.get("checked"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn needs_name(artifact: &Artifact, found: &Asset) -> bool {
    artifact.is_jar()
        && found.component_id.is_some()
        && p2_attributes(found).is_some_and(|p2| !p2.contains_key("pluginName"))
}

async fn set_attributes_of_blob(
    ctx: &Ctx,
    found: &Asset,
    attributes: Value,
) -> Result<bool, ContentError> {
    let result = asset::Entity::update_many()
        .col_expr(asset::Column::Attributes, Expr::value(attributes))
        .filter(asset::Column::Id.eq(found.id))
        .filter(asset::Column::BlobId.eq(found.blob_id))
        .exec(ctx.db())
        .await?;
    Ok(result.rows_affected == 1)
}

fn decode(encoding: Encoding, bytes: Vec<u8>, entry: &str) -> Result<(String, String), String> {
    let (data, entry) = match encoding {
        Encoding::Xml => (bytes, entry.to_owned()),
        Encoding::Xz => (
            bundle::xz_decompress(&bytes, MAX_METADATA)?,
            entry.to_owned(),
        ),
        Encoding::Jar => {
            let source = zip::Memory(bytes);
            let entries = zip::entries(&source)
                .now_or_never()
                .expect("an archive in memory is read at once")
                .map_err(|error| error.to_string())?;
            let chosen = entries
                .iter()
                .find(|found| found.name == entry)
                .or_else(|| entries.iter().find(|found| !found.is_directory()))
                .ok_or_else(|| "the archive holds no repository file".to_owned())?;
            let data = zip::read(&source, chosen, MAX_METADATA)
                .now_or_never()
                .expect("an archive in memory is read at once")
                .map_err(|error| error.to_string())?;
            (data, chosen.name.clone())
        }
    };
    let text =
        String::from_utf8(data).map_err(|_| "the repository file is not UTF-8 text".to_owned())?;
    Ok((text, entry))
}

fn encode(encoding: Encoding, text: String, entry: &str) -> Vec<u8> {
    match encoding {
        Encoding::Xml => text.into_bytes(),
        Encoding::Xz => bundle::xz_compress(text.as_bytes()),
        Encoding::Jar => zip::write(&[(entry, text.as_bytes())]),
    }
}

struct Transformed {
    bytes: Option<Vec<u8>>,
    remote_children: Vec<String>,
}

fn transform(
    metadata: &Metadata,
    bytes: Vec<u8>,
    remote: &url::Url,
    base: &url::Url,
) -> Result<Transformed, String> {
    let (text, entry) = decode(metadata.encoding, bytes, &metadata.entry_name())?;
    let composite = metadata.repository.is_composite();
    let directory: Vec<&str> = paths::segments(&metadata.location);
    let mut remote_children = Vec::new();
    let rewritten = xml::rewrite(&text, |location| {
        if !composite {
            return None;
        }
        let resolved = remote.join(location.trim()).ok()?;
        if !matches!(resolved.scheme(), "http" | "https") {
            return None;
        }
        let plain = paths::without_target(&resolved);
        let mapped = paths::location_of(&plain, base);
        if mapped.first().map(String::as_str) == Some(REMOTE) {
            remote_children.push(plain.to_string());
        }
        let mut written = paths::relative(&directory, &mapped);
        if location.trim().ends_with('/') && written != "." {
            written.push('/');
        }
        Some(written)
    })?;
    Ok(Transformed {
        bytes: rewritten.map(|text| encode(metadata.encoding, text, &entry)),
        remote_children,
    })
}

fn component_attributes(classifier: &str, id: &str, version: &str, name: Option<&Value>) -> Value {
    let mut p2 = Map::new();
    p2.insert("classifier".to_owned(), json!(classifier));
    p2.insert("id".to_owned(), json!(id));
    p2.insert("version".to_owned(), json!(version));
    if let Some(name) = name {
        p2.insert("pluginName".to_owned(), name.clone());
    }
    json!({ ID: p2 })
}

impl P2 {
    async fn metadata(
        &self,
        ctx: &Ctx,
        path: &str,
        metadata: Option<&Metadata>,
    ) -> Result<Option<Asset>, ContentError> {
        let content_type = metadata
            .map(|metadata| metadata.encoding.content_type())
            .unwrap_or("text/plain");
        for _ in 0..ATTEMPTS {
            let found = proxy::fetch_from(
                ctx,
                path,
                Freshness::Metadata,
                || origin(ctx, path),
                || {
                    PutAsset::cached(path)
                        .kind(METADATA)
                        .content_type(content_type)
                },
            )
            .await?;
            let Some(found) = found else {
                return Ok(None);
            };
            let Some(metadata) = metadata.filter(|metadata| metadata.repository.is_rewritten())
            else {
                return Ok(Some(found));
            };
            if checked(&found) {
                return Ok(Some(found));
            }
            if self.rewrite(ctx, path, metadata, &found).await? {
                return store::find(ctx.db(), ctx.name(), path).await;
            }
        }
        Err(ContentError::BadGateway(format!(
            "{path} changed on the remote while the proxy read it"
        )))
    }

    async fn rewrite(
        &self,
        ctx: &Ctx,
        path: &str,
        metadata: &Metadata,
        found: &Asset,
    ) -> Result<bool, ContentError> {
        let bytes = read(ctx, found, MAX_METADATA).await?;
        let remote = remote_of(ctx, path)?;
        let base = remote_base(ctx)?;
        let job = metadata.clone();
        let transformed =
            tokio::task::spawn_blocking(move || transform(&job, bytes, &remote, &base))
                .await
                .map_err(ContentError::internal)?;
        let mut p2 = Map::new();
        p2.insert("checked".to_owned(), json!(true));
        let transformed = match transformed {
            Ok(transformed) => transformed,
            Err(problem) => {
                tracing::warn!(repository = ctx.name(), path, %problem, "a p2 repository file is served as the remote sent it");
                p2.insert("problem".to_owned(), json!(problem));
                return set_attributes_of_blob(ctx, found, json!({ ID: p2 })).await;
            }
        };
        if metadata.repository.is_composite() {
            p2.insert(
                "remoteChildren".to_owned(),
                json!(transformed.remote_children),
            );
        }
        let attributes = json!({ ID: p2 });
        match transformed.bytes {
            None => set_attributes_of_blob(ctx, found, attributes).await,
            Some(bytes) => {
                let mut put = PutAsset::cached(path)
                    .kind(METADATA)
                    .content_type(metadata.encoding.content_type())
                    .attributes(attributes);
                put.remote_etag = found.remote_etag.clone();
                put.remote_last_modified = found.remote_last_modified;
                store::put(ctx, put, astrum_blob::content_of(bytes)).await?;
                Ok(true)
            }
        }
    }

    async fn parsed(&self, ctx: &Ctx, found: &Asset) -> Result<Arc<Descriptors>, ContentError> {
        let key = (found.repository.clone(), found.id, found.blob_id);
        if let Some(parsed) = self
            .parsed
            .lock()
            .expect("the descriptor cache lock is never poisoned")
            .get(&key)
        {
            return Ok(parsed.clone());
        }
        let Route::Metadata(metadata) = paths::route(&found.path) else {
            return Ok(Arc::default());
        };
        let bytes = read(ctx, found, MAX_METADATA).await?;
        let parsed = tokio::task::spawn_blocking(move || {
            decode(metadata.encoding, bytes, &metadata.entry_name())
                .and_then(|(text, _)| Descriptors::parse(&text))
        })
        .await
        .map_err(ContentError::internal)?
        .map_err(ContentError::BadGateway)?;
        let parsed = Arc::new(parsed);
        let mut cache = self
            .parsed
            .lock()
            .expect("the descriptor cache lock is never poisoned");
        if cache.len() >= PARSED_KEPT {
            cache.clear();
        }
        cache.insert(key, parsed.clone());
        Ok(parsed)
    }

    async fn descriptor(&self, ctx: &Ctx, artifact: &Artifact) -> Option<Descriptor> {
        for candidate in artifact.artifacts_paths() {
            let found = match store::find(ctx.db(), ctx.name(), &candidate).await {
                Ok(Some(found)) => found,
                Ok(None) => continue,
                Err(error) => {
                    tracing::info!(repository = ctx.name(), %error, "the artifact descriptors cannot be looked up");
                    return None;
                }
            };
            return match self.parsed(ctx, &found).await {
                Ok(parsed) => parsed
                    .find(artifact.folder.classifier(), artifact.stem())
                    .cloned(),
                Err(error) => {
                    tracing::info!(repository = ctx.name(), path = candidate, %error, "the artifact descriptors cannot be read");
                    None
                }
            };
        }
        None
    }

    async fn artifact(
        &self,
        ctx: &Ctx,
        path: &str,
        artifact: &Artifact,
    ) -> Result<Option<Asset>, ContentError> {
        if let Some(existing) = store::find(ctx.db(), ctx.name(), path).await?
            && proxy::is_fresh(ctx, &existing, Freshness::Content)
            && !needs_name(artifact, &existing)
        {
            return Ok(Some(existing));
        }
        let descriptor = self.descriptor(ctx, artifact).await;
        let coordinates = descriptor
            .as_ref()
            .map(|found| (found.id.clone(), found.version.clone()))
            .or_else(|| paths::split_name(artifact.stem()));
        let classifier = artifact.folder.classifier();
        let known_name = match &coordinates {
            Some((id, version)) => component::Entity::find()
                .filter(component::Column::Repository.eq(ctx.name()))
                .filter(component::Column::Namespace.eq(artifact.folder.name()))
                .filter(component::Column::Name.eq(id.as_str()))
                .filter(component::Column::Version.eq(version.as_str()))
                .one(ctx.db())
                .await?
                .and_then(|row| row.attributes.get(ID)?.get("pluginName").cloned())
                .filter(Value::is_string),
            None => None,
        };
        let found = proxy::fetch_from(
            ctx,
            path,
            Freshness::Content,
            || origin(ctx, path),
            || {
                let mut put = PutAsset::cached(path)
                    .kind(ARTIFACT)
                    .content_type(artifact.content_type());
                let Some((id, version)) = coordinates.clone() else {
                    return put.attributes(json!({ ID: { "classifier": classifier } }));
                };
                let mut attributes =
                    component_attributes(classifier, &id, &version, known_name.as_ref());
                if let Some(descriptor) = &descriptor
                    && !descriptor.properties.is_empty()
                {
                    attributes[ID]["properties"] = json!(descriptor.properties);
                }
                put = put.attributes(attributes);
                put.component(ComponentKey {
                    namespace: artifact.folder.name().to_owned(),
                    name: id.clone(),
                    version: version.clone(),
                    kind: String::new(),
                    attributes: component_attributes(
                        classifier,
                        &id,
                        &version,
                        known_name.as_ref(),
                    ),
                })
            },
        )
        .await?;
        let Some(found) = found else {
            return Ok(None);
        };
        if needs_name(artifact, &found)
            && let Some(component_id) = found.component_id
        {
            self.name(ctx, &found, component_id, artifact).await?;
            return store::find(ctx.db(), ctx.name(), path).await;
        }
        Ok(Some(found))
    }

    async fn name(
        &self,
        ctx: &Ctx,
        found: &Asset,
        component_id: i64,
        artifact: &Artifact,
    ) -> Result<(), ContentError> {
        let store = ctx.content.blob_stores().get(&found.blob_store).await?;
        let source = zip::Blob {
            store,
            id: found.blob_id.into(),
            size: found.size.max(0) as u64,
        };
        let name = bundle::name(&source, artifact.folder == paths::Folder::Features).await;
        let mut attributes = found.attributes.clone();
        if let Some(p2) = attributes.get_mut(ID).and_then(Value::as_object_mut) {
            p2.insert("pluginName".to_owned(), json!(name));
        }
        if !set_attributes_of_blob(ctx, found, attributes).await? {
            return Ok(());
        }
        if let Some(name) = name
            && let Some(row) = store::component_by_id(ctx.db(), component_id).await?
        {
            let mut attributes = row.attributes.clone();
            match attributes.get_mut(ID).and_then(Value::as_object_mut) {
                Some(p2) => {
                    p2.insert("pluginName".to_owned(), json!(name));
                }
                None => attributes = json!({ ID: { "pluginName": name } }),
            }
            store::set_component_attributes(ctx.db(), component_id, attributes).await?;
        }
        Ok(())
    }

    async fn lookup(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return store::find(ctx.db(), ctx.name(), path).await;
        }
        match paths::route(path) {
            Route::Index => self.metadata(ctx, path, None).await,
            Route::Metadata(metadata) => self.metadata(ctx, path, Some(&metadata)).await,
            Route::Artifact(artifact) => self.artifact(ctx, path, &artifact).await,
            Route::Directory | Route::Unknown => Ok(None),
        }
    }

    async fn read_request(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if paths::route(path) == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let found = self
            .lookup(ctx, path)
            .await?
            .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        let options = Serve {
            count_download: found.kind == ARTIFACT,
            ..Serve::default()
        };
        serve(ctx, &found, &request.method, &request.headers, options).await
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        store::delete_asset(&ctx.content, found).await?;
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }
}

#[async_trait]
impl Format for P2 {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match &request.method {
            &Method::GET | &Method::HEAD => self.read_request(&ctx, &request).await,
            &Method::DELETE => self.delete(&ctx, &request.path).await,
            method => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: "GET,HEAD,DELETE",
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        self.lookup(ctx, path).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        &[&["p2", "pluginName"]]
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE regexp_replace(a.path, '[^/]*$', '') || c.name || '/' || c.version || '/' || regexp_replace(a.path, '^.*/', '') END",
            ),
            generated: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Repository;

    fn metadata(path: &str) -> Metadata {
        match paths::route(path) {
            Route::Metadata(found) => found,
            other => panic!("{path} is {other:?}"),
        }
    }

    const COMPOSITE: &str = "<?xml version='1.0' encoding='UTF-8'?>
<?compositeMetadataRepository version='1.0.0'?>
<repository name='Release' type='org.eclipse.equinox.internal.p2.metadata.repository.CompositeMetadataRepository' version='1.0.0'>
  <properties size='1'>
    <property name='p2.timestamp' value='1'/>
  </properties>
  <children size='4'>
    <child location='../../technology/packages/2024-03/'/>
    <child location='202403131000'/>
    <child location='https://download.example.org/releases/2024-03/extra'/>
    <child location='https://other.example.net/updates/x'/>
  </children>
</repository>
";

    #[test]
    fn composite_children_are_kept_inside_the_proxy() {
        let base = url::Url::parse("https://download.example.org/releases/2024-03/").unwrap();
        let remote = base.join("compositeContent.jar").unwrap();
        let jar = zip::write(&[("compositeContent.xml", COMPOSITE.as_bytes())]);
        let found = metadata("/compositeContent.jar");
        assert_eq!(found.repository, Repository::CompositeContent);
        let transformed = transform(&found, jar, &remote, &base).unwrap();
        assert_eq!(
            transformed.remote_children,
            [
                "https://download.example.org/technology/packages/2024-03",
                "https://other.example.net/updates/x"
            ]
        );
        let (text, entry) = decode(
            Encoding::Jar,
            transformed.bytes.unwrap(),
            "compositeContent.xml",
        )
        .unwrap();
        assert_eq!(entry, "compositeContent.xml");
        assert!(text.contains(
            "<child location='.remote/https/download.example.org/technology/packages/2024-03/'/>"
        ));
        assert!(text.contains("<child location='202403131000'/>"));
        assert!(text.contains("<child location='extra'/>"));
        assert!(text.contains("<child location='.remote/https/other.example.net/updates/x'/>"));
    }

    #[test]
    fn nested_composites_point_back_up() {
        let base = url::Url::parse("https://download.example.org/releases/").unwrap();
        let remote =
            url::Url::parse("https://other.example.net/updates/x/compositeArtifacts.xml").unwrap();
        let found = metadata("/.remote/https/other.example.net/updates/x/compositeArtifacts.xml");
        let xml = COMPOSITE.replace("compositeMetadataRepository", "compositeArtifactRepository");
        let transformed = transform(&found, xml.into_bytes(), &remote, &base).unwrap();
        let text = String::from_utf8(transformed.bytes.unwrap()).unwrap();
        assert!(text.contains("<child location='202403131000'/>"));
        assert!(text.contains("<child location='../../technology/packages/2024-03/'/>"));
        assert!(text.contains("<child location='../../../../../2024-03/extra'/>"));
        assert!(text.contains("<child location='.'/>"));
        assert_eq!(transformed.remote_children.len(), 3);
    }

    #[test]
    fn simple_repositories_lose_their_mirrors_in_every_encoding() {
        let xml = "<?xml version='1.0' encoding='UTF-8'?>
<repository name='x' type='org.eclipse.equinox.p2.artifact.repository.simpleRepository' version='1'>
  <properties size='2'>
    <property name='p2.mirrorsURL' value='https://mirrors.example.org/x'/>
    <property name='p2.timestamp' value='1'/>
  </properties>
</repository>
";
        let base = url::Url::parse("https://p2.example.org/").unwrap();
        for (path, bytes) in [
            ("/artifacts.xml", xml.as_bytes().to_vec()),
            ("/artifacts.xml.xz", bundle::xz_compress(xml.as_bytes())),
            (
                "/artifacts.jar",
                zip::write(&[("artifacts.xml", xml.as_bytes())]),
            ),
        ] {
            let found = metadata(path);
            let remote = base.join(path.trim_start_matches('/')).unwrap();
            let transformed = transform(&found, bytes, &remote, &base).unwrap();
            let (text, _) = decode(
                found.encoding,
                transformed.bytes.expect(path),
                &found.entry_name(),
            )
            .unwrap();
            assert!(!text.contains("mirrors"), "{path}");
            assert!(text.contains("<properties size='1'>"), "{path}");
            assert!(transformed.remote_children.is_empty());
        }
        let clean = xml.replace(
            "    <property name='p2.mirrorsURL' value='https://mirrors.example.org/x'/>\n",
            "",
        );
        let transformed = transform(
            &metadata("/artifacts.xml"),
            clean.into_bytes(),
            &base.join("artifacts.xml").unwrap(),
            &base,
        )
        .unwrap();
        assert!(transformed.bytes.is_none());
    }

    #[test]
    fn broken_archives_are_reported() {
        let base = url::Url::parse("https://p2.example.org/").unwrap();
        let remote = base.join("artifacts.jar").unwrap();
        assert!(transform(&metadata("/artifacts.jar"), b"no".to_vec(), &remote, &base).is_err());
        assert!(
            transform(
                &metadata("/artifacts.xml"),
                vec![0xff, 0xfe],
                &remote,
                &base
            )
            .is_err()
        );
    }
}
