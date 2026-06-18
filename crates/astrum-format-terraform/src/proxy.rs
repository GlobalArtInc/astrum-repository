use std::sync::Mutex;

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use axum::http::{StatusCode, header};
use serde_json::{Value, json};
use url::Url;

use crate::hosted::{self, module_attributes, module_component, provider_component};
use crate::paths::{self, Endpoint, FileKind, Module, Provider};
use crate::sums::Sums;
use crate::{Document, Download, METADATA, MODULE_ARCHIVE, Terraform, archive, pgp, versions};

const MAX_DOCUMENT: u64 = 16 * 1024 * 1024;
const ACCEPT_JSON: &str = "application/json";

fn remote(ctx: &Ctx) -> Result<Url, ContentError> {
    let base = ctx
        .repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .ok_or_else(|| ContentError::internal(anyhow::anyhow!("the proxy has no remote URL")))?;
    let mut url = Url::parse(base).map_err(|error| {
        ContentError::BadGateway(format!("the remote URL is not valid: {error}"))
    })?;
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }
    Ok(url)
}

async fn document(ctx: &Ctx, path: &str, url: Url) -> Result<Option<(Asset, Value)>, ContentError> {
    let origin = Origin::new(url).header(header::ACCEPT, ACCEPT_JSON);
    let found = proxy::fetch_from(
        ctx,
        path,
        Freshness::Metadata,
        || async { Ok(Some(origin)) },
        || {
            PutAsset::cached(path)
                .kind(METADATA)
                .content_type("application/json")
        },
    )
    .await?;
    let Some(asset) = found else {
        return Ok(None);
    };
    let bytes = hosted::read(ctx, &asset, MAX_DOCUMENT).await?;
    let value = serde_json::from_slice(&bytes).map_err(|error| {
        ContentError::BadGateway(format!("the remote answered no JSON at {path}: {error}"))
    })?;
    Ok(Some((asset, value)))
}

struct Services {
    modules: Url,
    providers: Url,
}

async fn services(ctx: &Ctx) -> Result<Services, ContentError> {
    let base = remote(ctx)?;
    let discovery = base
        .join(".well-known/terraform.json")
        .map_err(|error| ContentError::BadGateway(error.to_string()))?;
    let found = match document(ctx, paths::DISCOVERY, discovery.clone()).await {
        Ok(found) => found.map(|(_, value)| value),
        Err(error) => {
            tracing::debug!(repository = ctx.name(), %error, "the remote has no service discovery");
            None
        }
    };
    let service = |name: &str, fallback: &str| -> Result<Url, ContentError> {
        let listed = found
            .as_ref()
            .and_then(|value| value[name].as_str())
            .and_then(|value| discovery.join(value).ok());
        let mut url = match listed {
            Some(url) => url,
            None => base
                .join(fallback)
                .map_err(|error| ContentError::BadGateway(error.to_string()))?,
        };
        if !url.path().ends_with('/') {
            let path = format!("{}/", url.path());
            url.set_path(&path);
        }
        Ok(url)
    };
    Ok(Services {
        modules: service("modules.v1", "v1/modules/")?,
        providers: service("providers.v1", "v1/providers/")?,
    })
}

fn join(base: &Url, relative: &str) -> Result<Url, ContentError> {
    base.join(relative)
        .map_err(|error| ContentError::BadGateway(error.to_string()))
}

pub async fn module_versions(
    ctx: &Ctx,
    module: &Module,
) -> Result<Option<Vec<String>>, ContentError> {
    let services = services(ctx).await?;
    let url = join(
        &services.modules,
        &format!(
            "{}/{}/{}/versions",
            module.namespace, module.name, module.system
        ),
    )?;
    let Some((_, value)) = document(ctx, &module.versions_path(), url).await? else {
        return Ok(None);
    };
    let mut listed: Vec<String> = value["modules"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|entry| entry["versions"].as_array().into_iter().flatten())
        .filter_map(|entry| entry["version"].as_str())
        .filter(|version| paths::valid_version(version))
        .map(str::to_owned)
        .collect();
    listed.sort();
    listed.dedup();
    versions::sort(&mut listed);
    Ok(Some(listed))
}

#[derive(Debug, PartialEq, Eq)]
struct Source {
    url: Option<Url>,
    extension: Option<&'static str>,
    subdir: String,
}

fn parse_source(location: &str) -> Source {
    let (forced, rest) = match location.split_once("::") {
        Some((forced, rest)) if !forced.contains('/') && !forced.contains(':') => {
            (Some(forced), rest)
        }
        _ => (None, location),
    };
    let (scheme, after) = rest.split_once("://").unwrap_or(("", rest));
    let (mut address, mut subdir) = match after.find("//") {
        Some(at) => (
            format!("{scheme}://{}", &after[..at]),
            after[at + 2..].to_owned(),
        ),
        None => (rest.to_owned(), String::new()),
    };
    if let Some((dir, query)) = subdir.clone().split_once('?') {
        subdir = dir.to_owned();
        address = format!("{address}?{query}");
    }
    let url = Url::parse(&address).ok().filter(|url| {
        matches!(url.scheme(), "http" | "https") && forced.is_none_or(|forced| forced == "http")
    });
    let extension = url.as_ref().and_then(|url| {
        url.query_pairs()
            .find(|(name, _)| name == "archive")
            .and_then(|(_, value)| paths::archive_parameter(&value))
            .or_else(|| paths::archive_extension(url.path()))
    });
    Source {
        url,
        extension,
        subdir,
    }
}

pub async fn module_download(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
) -> Result<Download, ContentError> {
    let tokens = terraform.tokens(ctx).await?;
    if let Some(cached) = hosted::module_archive(ctx, module).await? {
        return Ok(Download::Location(tokens.sign(
            ctx.name(),
            &hosted::module_location(module, &cached),
            &cached.path,
        )));
    }
    let services = services(ctx).await?;
    let url = join(
        &services.modules,
        &format!(
            "{}/{}/{}/{}/download",
            module.namespace, module.name, module.system, module.version
        ),
    )?;
    let remote = proxy::request(
        ctx,
        url.clone(),
        &[(header::ACCEPT, ACCEPT_JSON.to_owned())],
    )
    .await?;
    let status = remote.status;
    let header_location = remote.header("x-terraform-get").map(str::to_owned);
    let location = match (status, header_location) {
        (_, Some(location)) if status.is_success() => location,
        (StatusCode::OK, None) => {
            let body = remote.bytes(MAX_DOCUMENT as usize).await?;
            let value: Value = serde_json::from_slice(&body).unwrap_or_default();
            match value["location"].as_str() {
                Some(location) => location.to_owned(),
                None => {
                    return Ok(Download::Missing(
                        StatusCode::BAD_GATEWAY,
                        "The remote registry named no module location".to_owned(),
                    ));
                }
            }
        }
        (StatusCode::NOT_FOUND, _) => {
            return Ok(Download::Missing(
                StatusCode::NOT_FOUND,
                format!(
                    "Module {} version {} not found",
                    module.address(),
                    module.version
                ),
            ));
        }
        (status, _) => {
            return Ok(Download::Missing(
                StatusCode::BAD_GATEWAY,
                format!("The remote registry answered {status}"),
            ));
        }
    };
    let absolute = match Url::parse(&location) {
        Ok(_) => location.clone(),
        Err(_) if !location.contains("::") => url
            .join(&location)
            .map(|joined| joined.to_string())
            .unwrap_or(location.clone()),
        Err(_) => location.clone(),
    };
    let source = parse_source(&absolute);
    let (Some(source_url), Some(extension)) = (source.url.clone(), source.extension) else {
        return Ok(Download::Location(absolute));
    };
    match cache_module(ctx, module, source_url, extension, &source.subdir).await {
        Ok(Some(asset)) => Ok(Download::Location(tokens.sign(
            ctx.name(),
            &hosted::module_location(module, &asset),
            &asset.path,
        ))),
        Ok(None) => Ok(Download::Location(absolute)),
        Err(error) => {
            tracing::info!(repository = ctx.name(), %error, "a module package was not cached");
            Ok(Download::Location(absolute))
        }
    }
}

async fn cache_module(
    ctx: &Ctx,
    module: &Module,
    url: Url,
    extension: &'static str,
    subdir: &str,
) -> Result<Option<Asset>, ContentError> {
    let path = module.archive_path(extension);
    let found = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async { Ok(Some(Origin::new(url))) },
        || {
            PutAsset::cached(&path)
                .kind(MODULE_ARCHIVE)
                .component(module_component(module))
                .content_type(match extension {
                    "zip" => "application/zip",
                    _ => "application/gzip",
                })
                .attributes(module_attributes(module, subdir))
        },
    )
    .await?;
    let Some(asset) = found else {
        return Ok(None);
    };
    if subdir.is_empty() || subdir.contains(['*', '?', '[']) {
        let data = hosted::read(ctx, &asset, 512 * 1024 * 1024).await?;
        let wanted = subdir.to_owned();
        let resolved = tokio::task::spawn_blocking(move || {
            let layout = archive::layout(&data, extension)?;
            match wanted.is_empty() {
                true => layout.single_top_directory(),
                false => resolve_glob(&layout, &wanted),
            }
        })
        .await
        .map_err(ContentError::internal)?
        .unwrap_or_else(|| subdir.to_owned());
        if resolved != subdir {
            store::set_attributes(ctx.db(), asset.id, module_attributes(module, &resolved)).await?;
            return store::find(ctx.db(), ctx.name(), &path).await;
        }
    }
    Ok(Some(asset))
}

fn resolve_glob(layout: &archive::Layout, pattern: &str) -> Option<String> {
    let pattern = pattern.trim_matches('/');
    let mut expression = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => expression.push_str("[^/]*"),
            '?' => expression.push_str("[^/]"),
            '[' | ']' => expression.push(c),
            other => expression.push_str(&regex_escape(other)),
        }
    }
    expression.push('$');
    let matcher = regex::Regex::new(&expression).ok()?;
    let found: Vec<&String> = layout
        .directories
        .iter()
        .filter(|directory| matcher.is_match(directory))
        .collect();
    match found.as_slice() {
        [only] => Some((*only).clone()),
        _ => None,
    }
}

fn regex_escape(c: char) -> String {
    regex::escape(&c.to_string())
}

pub async fn provider_versions(ctx: &Ctx, provider: &Provider) -> Result<Document, ContentError> {
    let services = services(ctx).await?;
    let url = join(
        &services.providers,
        &format!("{}/{}/versions", provider.namespace, provider.kind),
    )?;
    Ok(match document(ctx, &provider.versions_path(), url).await? {
        Some((_, value)) => Document::Found(value),
        None => Document::missing(format!("Provider {} not found", provider.address())),
    })
}

async fn upstream_download(
    ctx: &Ctx,
    provider: &Provider,
    os: &str,
    arch: &str,
) -> Result<Option<(Url, Value)>, ContentError> {
    let services = services(ctx).await?;
    let url = join(
        &services.providers,
        &format!(
            "{}/{}/{}/download/{os}/{arch}",
            provider.namespace, provider.kind, provider.version
        ),
    )?;
    Ok(
        document(ctx, &provider.download_path(os, arch), url.clone())
            .await?
            .map(|(_, value)| (url, value)),
    )
}

fn base_name(url: &str) -> &str {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    path.rsplit('/').next().unwrap_or_default()
}

pub async fn provider_download(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
    os: &str,
    arch: &str,
) -> Result<Document, ContentError> {
    let Some((endpoint, mut value)) = upstream_download(ctx, provider, os, arch).await? else {
        return Ok(Document::missing(format!(
            "Provider {} version {} has no package for {os}_{arch}",
            provider.address(),
            provider.version
        )));
    };
    let tokens = terraform.tokens(ctx).await?;
    let from = provider.download_path(os, arch);
    for (field, expected) in [
        ("download_url", provider.archive_file(os, arch)),
        ("shasums_url", provider.shasums_file()),
        ("shasums_signature_url", provider.signature_file()),
    ] {
        let Some(url) = value[field].as_str().map(str::to_owned) else {
            continue;
        };
        let name = base_name(&url);
        let local = match provider.kind_of(name) {
            Some(kind) if provider.stored_name(&kind, name) == expected => {
                tokens.link(ctx.name(), &from, &provider.file_path(&expected))
            }
            _ => endpoint
                .join(&url)
                .map(|joined| joined.to_string())
                .unwrap_or(url),
        };
        value[field] = Value::String(local);
    }
    Ok(Document::Found(value))
}

async fn any_download(
    ctx: &Ctx,
    provider: &Provider,
) -> Result<Option<(Url, Value)>, ContentError> {
    let cached = hosted::under(
        ctx,
        &format!("{}/download/", provider.directory()),
        &[METADATA],
    )
    .await?;
    let services = services(ctx).await?;
    for asset in cached {
        let rest = asset.path.rsplit('/').take(2).collect::<Vec<_>>();
        if let [arch, os] = rest.as_slice()
            && let Ok(bytes) = hosted::read(ctx, &asset, MAX_DOCUMENT).await
            && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        {
            let url = join(
                &services.providers,
                &format!(
                    "{}/{}/{}/download/{os}/{arch}",
                    provider.namespace, provider.kind, provider.version
                ),
            )?;
            return Ok(Some((url, value)));
        }
    }
    let Document::Found(listed) = provider_versions(ctx, &provider.at("")).await? else {
        return Ok(None);
    };
    let platform = listed["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|entry| entry["version"].as_str() == Some(provider.version.as_str()))
        .and_then(|entry| entry["platforms"].as_array()?.first().cloned());
    let Some(platform) = platform else {
        return Ok(None);
    };
    let (Some(os), Some(arch)) = (platform["os"].as_str(), platform["arch"].as_str()) else {
        return Ok(None);
    };
    upstream_download(ctx, provider, os, arch).await
}

pub async fn file(
    terraform: &Terraform,
    ctx: &Ctx,
    path: &str,
) -> Result<Option<Asset>, ContentError> {
    let _ = terraform;
    let Endpoint::ProviderFile(provider, name, kind) = Endpoint::parse(path) else {
        return store::find(ctx.db(), ctx.name(), path).await;
    };
    let shasum: Mutex<Option<String>> = Mutex::new(None);
    let kind_for_origin = kind.clone();
    let found = proxy::fetch_from(
        ctx,
        path,
        Freshness::Content,
        || async {
            let download = match &kind_for_origin {
                FileKind::Archive { os, arch } => {
                    upstream_download(ctx, &provider, os, arch).await?
                }
                FileKind::Shasums | FileKind::Signature => any_download(ctx, &provider).await?,
                _ => None,
            };
            let Some((endpoint, value)) = download else {
                return Ok(None);
            };
            let field = match kind_for_origin {
                FileKind::Archive { .. } => "download_url",
                FileKind::Shasums => "shasums_url",
                _ => "shasums_signature_url",
            };
            let Some(url) = value[field].as_str() else {
                return Ok(None);
            };
            if let FileKind::Archive { .. } = kind_for_origin {
                *shasum.lock().expect("the shasum lock is never poisoned") =
                    value["shasum"].as_str().map(str::to_owned);
            }
            Ok(Some(Origin::new(join(&endpoint, url)?)))
        },
        || {
            let extra = match &kind {
                FileKind::Archive { os, arch } => json!({"os": os, "arch": arch}),
                _ => json!({}),
            };
            let mut attributes = json!({crate::ID: {
                "kind": crate::KIND_PROVIDER,
                "namespace": provider.namespace,
                "name": provider.kind,
                "provider": provider.kind,
                "version": provider.version,
            }});
            if let (Some(values), Value::Object(extra)) =
                (attributes[crate::ID].as_object_mut(), extra)
            {
                values.extend(extra);
            }
            PutAsset::cached(path)
                .kind(kind.asset_kind())
                .content_type(kind.content_type())
                .component(provider_component(&provider))
                .attributes(attributes)
        },
    )
    .await?;
    let expected = shasum
        .lock()
        .expect("the shasum lock is never poisoned")
        .take();
    if let (Some(asset), Some(expected)) = (&found, expected)
        && !asset.sha256.eq_ignore_ascii_case(&expected)
    {
        store::delete(ctx, path).await?;
        return Err(ContentError::BadGateway(format!(
            "{name} does not match the SHA-256 the remote registry published"
        )));
    }
    Ok(found)
}

fn upstream_host(ctx: &Ctx) -> Option<String> {
    ctx.repository
        .attributes
        .terraform
        .as_ref()
        .and_then(|terraform| terraform.upstream_host.clone())
        .or_else(|| {
            let url = remote(ctx).ok()?;
            let host = url.host_str()?.to_ascii_lowercase();
            Some(match url.port() {
                Some(port) => format!("{host}:{port}"),
                None => host,
            })
        })
        .map(|host| host.to_ascii_lowercase())
}

fn not_mirrored(ctx: &Ctx, host: &str) -> Document {
    Document::missing(format!(
        "This repository mirrors providers of {}, not of {host}",
        upstream_host(ctx).unwrap_or_default()
    ))
}

pub async fn mirror_index(
    ctx: &Ctx,
    host: &str,
    provider: &Provider,
) -> Result<Document, ContentError> {
    if upstream_host(ctx).as_deref() != Some(host) {
        return Ok(not_mirrored(ctx, host));
    }
    let Document::Found(listed) = provider_versions(ctx, provider).await? else {
        return Ok(Document::missing(format!(
            "Provider {} not found",
            provider.address()
        )));
    };
    let mut found: Vec<String> = listed["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["version"].as_str())
        .filter(|version| paths::valid_version(version))
        .map(str::to_owned)
        .collect();
    versions::sort(&mut found);
    Ok(Document::Found(crate::mirror_index_document(&found)))
}

pub async fn mirror_version(
    terraform: &Terraform,
    ctx: &Ctx,
    host: &str,
    provider: &Provider,
    document: &str,
) -> Result<Document, ContentError> {
    if upstream_host(ctx).as_deref() != Some(host) {
        return Ok(not_mirrored(ctx, host));
    }
    let Some((_, download)) = any_download(ctx, provider).await? else {
        return Ok(Document::missing(format!(
            "Provider {} version {} not found",
            provider.address(),
            provider.version
        )));
    };
    let bad_gateway = |message: String| Ok(Document::Missing(StatusCode::BAD_GATEWAY, message));
    let sums = file(
        terraform,
        ctx,
        &provider.file_path(&provider.shasums_file()),
    )
    .await?;
    let signature = file(
        terraform,
        ctx,
        &provider.file_path(&provider.signature_file()),
    )
    .await?;
    let (Some(sums), Some(signature)) = (sums, signature) else {
        return bad_gateway(format!(
            "The checksums of {} {} are not available upstream",
            provider.address(),
            provider.version
        ));
    };
    let sums = hosted::read(ctx, &sums, MAX_DOCUMENT).await?;
    let signature = hosted::read(ctx, &signature, MAX_DOCUMENT).await?;
    let signed = download["signing_keys"]["gpg_public_keys"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|key| key["ascii_armor"].as_str())
        .any(|armored| pgp::verify(&sums, &signature, armored));
    if !signed {
        return bad_gateway(format!(
            "The checksums of {} {} are not signed by a key the upstream registry lists",
            provider.address(),
            provider.version
        ));
    }
    let parsed = Sums::parse(&sums).map_err(ContentError::BadGateway)?;
    let tokens = terraform.tokens(ctx).await?;
    let mut archives = serde_json::Map::new();
    for (name, digest) in &parsed.digests {
        if let Some((os, arch)) = provider.platform_of(name) {
            archives.insert(
                format!("{os}_{arch}"),
                json!({
                    "url": tokens.link(ctx.name(), document, &provider.file_path(name)),
                    "hashes": [format!("zh:{digest}")],
                }),
            );
        }
    }
    Ok(Document::Found(json!({ "archives": archives })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_sources_are_split_into_archive_and_subdirectory() {
        let source = parse_source("https://example.com/vpc.tar.gz//modules/x?archive=tgz");
        assert_eq!(
            source.url.unwrap().as_str(),
            "https://example.com/vpc.tar.gz?archive=tgz"
        );
        assert_eq!(source.extension, Some("tar.gz"));
        assert_eq!(source.subdir, "modules/x");
        let git = parse_source("git::https://github.com/acme/vpc?ref=v1.0.0");
        assert_eq!(git.url, None);
        let plain = parse_source("https://example.com/download/vpc.zip");
        assert_eq!(plain.extension, Some("zip"));
        assert_eq!(plain.subdir, "");
        assert_eq!(parse_source("s3::https://bucket/x.zip").url, None);
    }

    #[test]
    fn upstream_file_names_are_taken_from_urls() {
        assert_eq!(base_name("https://x/y/z.zip?a=1"), "z.zip");
        assert_eq!(base_name("../../f_SHA256SUMS"), "f_SHA256SUMS");
    }
}
