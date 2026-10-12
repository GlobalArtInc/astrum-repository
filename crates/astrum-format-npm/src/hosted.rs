use std::collections::BTreeSet;

use astrum_content::store::{self, PutAsset};
use astrum_content::{ContentError, ContentRequest, Ctx, View};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use axum::response::Response;
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value};

use crate::attachment::{self, Tarball};
use crate::documents::{self, Stored, not_found};
use crate::packument;
use crate::path::Name;
use crate::{Npm, reply};

pub const MAX_PUBLISH: usize = 256 * 1024 * 1024;
const MAX_SMALL: usize = 64 * 1024;

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub fn writable(ctx: &Ctx) -> Result<(), ContentError> {
    let kind = match ctx.kind() {
        RepositoryType::Hosted => {
            if policy(ctx) == WritePolicy::Deny {
                return Err(ContentError::BadRequest(format!(
                    "the repository {} is read-only",
                    ctx.name()
                )));
            }
            return Ok(());
        }
        RepositoryType::Proxy => "proxy",
        RepositoryType::Group => "group",
    };
    Err(ContentError::BadRequest(format!(
        "{} is a {kind} repository; publish to a hosted repository",
        ctx.name()
    )))
}

fn parse(body: &[u8]) -> Result<Value, ContentError> {
    serde_json::from_slice(body).map_err(|error| {
        ContentError::BadRequest(format!("the request body is not valid JSON: {error}"))
    })
}

pub async fn put(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    request: ContentRequest,
) -> Result<Response, ContentError> {
    writable(ctx)?;
    let packument_path = name.packument_path();
    if ![View::Add, View::Edit, View::Delete]
        .into_iter()
        .any(|view| ctx.allowed(view, &packument_path))
    {
        ctx.require(View::Add, &packument_path)?;
    }
    let incoming = parse(&request.bytes(MAX_PUBLISH).await?)?;
    if !incoming.is_object() {
        return Err(ContentError::BadRequest(
            "the request body is not a package document".to_owned(),
        ));
    }
    if let Some(given) = incoming.get("name").and_then(Value::as_str)
        && given != name.to_string()
    {
        return Err(ContentError::BadRequest(format!(
            "the document is for {given}, not {name}"
        )));
    }
    let _held = npm.lock(ctx, name).await;
    let stored = documents::hosted(ctx, name).await?;
    let publishing = incoming
        .get("_attachments")
        .and_then(Value::as_object)
        .is_some_and(|attachments| !attachments.is_empty());
    match publishing {
        true => publish(ctx, name, stored, &incoming).await,
        false => update(ctx, name, stored, &incoming).await,
    }
}

pub async fn publish_document(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    incoming: &Value,
) -> Result<(), ContentError> {
    writable(ctx)?;
    let _held = npm.lock(ctx, name).await;
    let stored = documents::hosted(ctx, name).await?;
    publish(ctx, name, stored, incoming).await.map(|_| ())
}

async fn publish(
    ctx: &Ctx,
    name: &Name,
    stored: Option<Stored>,
    incoming: &Value,
) -> Result<Response, ContentError> {
    let tarballs = attachment::tarballs(name, incoming).map_err(ContentError::BadRequest)?;
    let known = stored
        .as_ref()
        .and_then(|stored| packument::versions(&stored.doc))
        .cloned()
        .unwrap_or_default();
    ctx.require(View::Add, &name.packument_path())?;
    let mut fresh = Vec::new();
    for tarball in &tarballs {
        let path = name.tarball_path(&tarball.file);
        let exists = known.contains_key(&tarball.version)
            || store::find(ctx.db(), ctx.name(), &path).await?.is_some();
        ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
        if exists && policy(ctx) == WritePolicy::AllowOnce {
            return Err(ContentError::Conflict(format!(
                "{name}@{} is already published and {} does not allow redeploying",
                tarball.version,
                ctx.name()
            )));
        }
        if !exists {
            fresh.push(path);
        }
    }
    let mut added = Vec::new();
    for Tarball {
        version,
        mut manifest,
        file,
        data,
    } in tarballs
    {
        let path = name.tarball_path(&file);
        let put = documents::tarball_put(
            name,
            PutAsset::hosted(ctx, path.clone()),
            Some(version.clone()),
        );
        if let Err(error) = store::put(ctx, put, documents::stream(data)).await {
            forget(ctx, &fresh).await;
            return Err(error);
        }
        manifest["dist"]["tarball"] = Value::String(format!("{}{path}", ctx.repository_url()));
        added.push((version, manifest));
    }
    let doc = packument::publish(
        stored.map(|stored| stored.doc),
        name,
        incoming,
        &added,
        &now(),
    );
    if let Err(error) = documents::save(ctx, name, &doc).await {
        forget(ctx, &fresh).await;
        return Err(error);
    }
    Ok(reply::success())
}

async fn forget(ctx: &Ctx, paths: &[String]) {
    for path in paths {
        if let Err(error) = store::delete(ctx, path).await {
            tracing::warn!(repository = ctx.name(), path, %error, "a tarball of a failed publish stays");
        }
    }
}

async fn update(
    ctx: &Ctx,
    name: &Name,
    stored: Option<Stored>,
    incoming: &Value,
) -> Result<Response, ContentError> {
    let stored = stored.ok_or_else(|| not_found(name))?;
    let updated =
        packument::update(stored.doc, incoming, &now()).map_err(ContentError::BadRequest)?;
    ctx.require(View::Edit, &name.packument_path())?;
    let removed: Vec<String> = updated
        .removed
        .iter()
        .map(|(version, manifest)| {
            let file =
                packument::tarball_file(manifest).unwrap_or_else(|| name.tarball_file(version));
            name.tarball_path(&file)
        })
        .collect();
    for path in &removed {
        ctx.require(View::Delete, path)?;
    }
    if packument::versions(&updated.doc).is_none_or(|versions| versions.is_empty()) {
        return remove_package(ctx, name).await;
    }
    documents::save(ctx, name, &updated.doc).await?;
    for path in &removed {
        store::delete(ctx, path).await?;
    }
    Ok(reply::success())
}

pub async fn unpublish(npm: &Npm, ctx: &Ctx, name: &Name) -> Result<Response, ContentError> {
    writable(ctx)?;
    ctx.require(View::Delete, &name.packument_path())?;
    let _held = npm.lock(ctx, name).await;
    remove_package(ctx, name).await
}

async fn remove_package(ctx: &Ctx, name: &Name) -> Result<Response, ContentError> {
    ctx.require(View::Delete, &name.packument_path())?;
    let tarballs = store::under(ctx.db(), ctx.name(), &name.tarball_prefix(), 1_000_000).await?;
    for tarball in &tarballs {
        ctx.require(View::Delete, &tarball.path)?;
    }
    let packument = store::find(ctx.db(), ctx.name(), &name.packument_path()).await?;
    if packument.is_none() && tarballs.is_empty() {
        return Err(not_found(name));
    }
    for tarball in tarballs {
        store::delete_asset(&ctx.content, tarball).await?;
    }
    if let Some(packument) = packument {
        store::delete_asset(&ctx.content, packument).await?;
    }
    Ok(reply::success())
}

pub async fn delete_tarball(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    file: &str,
) -> Result<Response, ContentError> {
    writable(ctx)?;
    let path = name.tarball_path(file);
    ctx.require(View::Delete, &path)?;
    let _held = npm.lock(ctx, name).await;
    let stored = documents::hosted(ctx, name).await?;
    let deleted = store::delete(ctx, &path).await?;
    let Some(stored) = stored else {
        return match deleted {
            true => Ok(reply::success()),
            false => Err(ContentError::not_found(format!("{path} not found"))),
        };
    };
    let gone: BTreeSet<String> = packument::versions(&stored.doc)
        .into_iter()
        .flatten()
        .filter(|(version, manifest)| {
            packument::tarball_file(manifest).unwrap_or_else(|| name.tarball_file(version)) == file
        })
        .map(|(version, _)| version.clone())
        .collect();
    if gone.is_empty() {
        return match deleted {
            true => Ok(reply::success()),
            false => Err(ContentError::not_found(format!("{path} not found"))),
        };
    }
    let rev = packument::next_rev(Some(&stored.doc));
    let mut doc = packument::remove_versions(stored.doc, &gone);
    if packument::versions(&doc).is_none_or(|versions| versions.is_empty()) {
        return remove_package(ctx, name).await;
    }
    doc["_rev"] = Value::String(rev);
    doc["time"]["modified"] = Value::String(now());
    documents::save(ctx, name, &doc).await?;
    Ok(reply::success())
}

fn version_body(body: &[u8]) -> Result<String, ContentError> {
    let text = match serde_json::from_slice::<Value>(body) {
        Ok(Value::String(version)) => version,
        _ => String::from_utf8_lossy(body)
            .trim()
            .trim_matches('"')
            .to_owned(),
    };
    match text.is_empty() {
        true => Err(ContentError::BadRequest(
            "the request names no version".to_owned(),
        )),
        false => Ok(text),
    }
}

async fn tagged(ctx: &Ctx, name: &Name) -> Result<Stored, ContentError> {
    let stored = documents::hosted(ctx, name)
        .await?
        .ok_or_else(|| not_found(name))?;
    ctx.require(View::Edit, &name.packument_path())?;
    Ok(stored)
}

pub async fn set_tag(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    tag: &str,
    request: ContentRequest,
) -> Result<Response, ContentError> {
    writable(ctx)?;
    if !packument::valid_tag(tag) {
        return Err(ContentError::BadRequest(format!(
            "{tag} cannot be a tag: tags must not look like versions or ranges"
        )));
    }
    let version = version_body(&request.bytes(MAX_SMALL).await?)?;
    let _held = npm.lock(ctx, name).await;
    let stored = tagged(ctx, name).await?;
    if !packument::versions(&stored.doc).is_some_and(|versions| versions.contains_key(&version)) {
        return Err(ContentError::BadRequest(format!(
            "{name} has no version {version}"
        )));
    }
    let doc = packument::set_tag(stored.doc, tag, Some(&version), &now());
    documents::save(ctx, name, &doc).await?;
    Ok(reply::success())
}

pub async fn remove_tag(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    tag: &str,
) -> Result<Response, ContentError> {
    writable(ctx)?;
    if tag == "latest" {
        return Err(ContentError::BadRequest(
            "the latest tag cannot be removed".to_owned(),
        ));
    }
    let _held = npm.lock(ctx, name).await;
    let stored = tagged(ctx, name).await?;
    if packument::dist_tags(&stored.doc).get(tag).is_none() {
        return Err(ContentError::not_found(format!("{name} has no tag {tag}")));
    }
    let doc = packument::set_tag(stored.doc, tag, None, &now());
    documents::save(ctx, name, &doc).await?;
    Ok(reply::success())
}

pub async fn replace_tags(
    npm: &Npm,
    ctx: &Ctx,
    name: &Name,
    request: ContentRequest,
    merge: bool,
) -> Result<Response, ContentError> {
    writable(ctx)?;
    let tags = match parse(&request.bytes(MAX_SMALL).await?)? {
        Value::Object(tags) => tags,
        _ => {
            return Err(ContentError::BadRequest(
                "the request body is not an object of tags".to_owned(),
            ));
        }
    };
    let _held = npm.lock(ctx, name).await;
    let stored = tagged(ctx, name).await?;
    let versions = packument::versions(&stored.doc)
        .cloned()
        .unwrap_or_default();
    let mut checked = Map::new();
    for (tag, version) in tags {
        let known = version
            .as_str()
            .is_some_and(|version| versions.contains_key(version));
        if !packument::valid_tag(&tag) || !known {
            return Err(ContentError::BadRequest(format!(
                "the tag {tag} does not name a version of {name}"
            )));
        }
        checked.insert(tag, version);
    }
    let doc = packument::replace_tags(stored.doc, checked, merge, &now());
    documents::save(ctx, name, &doc).await?;
    Ok(reply::success())
}
