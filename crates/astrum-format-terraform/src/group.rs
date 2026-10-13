use std::collections::{BTreeMap, BTreeSet};

use astrum_content::group;
use astrum_content::{ContentError, Ctx};
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::paths::{Module, Provider};
use crate::{Document, Download, Terraform, versions};

pub async fn module_versions(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
) -> Result<Option<Vec<String>>, ContentError> {
    let mut found = BTreeSet::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match Box::pin(terraform.module_versions(&member, module)).await {
            Ok(Some(listed)) => found.extend(listed),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
            }
        }
    }
    let mut found: Vec<String> = found.into_iter().collect();
    versions::sort(&mut found);
    Ok(Some(found))
}

pub async fn module_download(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
) -> Result<Download, ContentError> {
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match Box::pin(terraform.module_download(&member, module)).await {
            Ok(Download::Location(location)) => {
                let tokens = terraform.tokens(ctx).await?;
                return Ok(Download::Location(tokens.resign(
                    ctx.name(),
                    &location,
                    &module.download_path(),
                )));
            }
            Ok(Download::Missing(..)) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
            }
        }
    }
    Ok(Download::Missing(
        StatusCode::NOT_FOUND,
        format!(
            "Module {} version {} not found",
            module.address(),
            module.version
        ),
    ))
}

pub async fn provider_versions(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
) -> Result<Document, ContentError> {
    let mut merged: BTreeMap<String, Value> = BTreeMap::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match Box::pin(terraform.provider_versions(&member, provider)).await {
            Ok(Document::Found(document)) => {
                for entry in document["versions"].as_array().into_iter().flatten() {
                    if let Some(version) = entry["version"].as_str() {
                        merged
                            .entry(version.to_owned())
                            .or_insert_with(|| entry.clone());
                    }
                }
            }
            Ok(Document::Missing(..)) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
            }
        }
    }
    if merged.is_empty() {
        return Ok(Document::missing(format!(
            "Provider {} not found",
            provider.address()
        )));
    }
    let mut ordered: Vec<String> = merged.keys().cloned().collect();
    versions::sort(&mut ordered);
    let entries: Vec<Value> = ordered
        .iter()
        .filter_map(|version| merged.remove(version))
        .collect();
    Ok(Document::Found(json!({ "versions": entries })))
}

fn resign(terraform_tokens: &crate::tokens::Tokens, ctx: &Ctx, document: &mut Value, path: &str) {
    for field in [
        "download_url",
        "shasums_url",
        "shasums_signature_url",
        "url",
    ] {
        if let Some(url) = document[field].as_str() {
            let resigned = terraform_tokens.resign(ctx.name(), url, path);
            document[field] = Value::String(resigned);
        }
    }
}

pub async fn provider_download(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
    os: &str,
    arch: &str,
) -> Result<Document, ContentError> {
    let path = provider.download_path(os, arch);
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        match Box::pin(terraform.provider_download(&member, provider, os, arch)).await {
            Ok(Document::Found(mut document)) => {
                let tokens = terraform.tokens(ctx).await?;
                resign(&tokens, ctx, &mut document, &path);
                return Ok(Document::Found(document));
            }
            Ok(Document::Missing(..)) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
            }
        }
    }
    Ok(Document::missing(format!(
        "Provider {} version {} has no package for {os}_{arch}",
        provider.address(),
        provider.version
    )))
}

pub async fn mirror_index(
    terraform: &Terraform,
    ctx: &Ctx,
    host: &str,
    provider: &Provider,
) -> Result<Document, ContentError> {
    let mut found = BTreeSet::new();
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        if let Ok(Document::Found(document)) =
            Box::pin(terraform.mirror_index(&member, host, provider)).await
            && let Some(listed) = document["versions"].as_object()
        {
            found.extend(listed.keys().cloned());
        }
    }
    if found.is_empty() {
        return Ok(Document::missing(format!(
            "Provider {} not found",
            provider.address()
        )));
    }
    let mut found: Vec<String> = found.into_iter().collect();
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
    for member in group::members(ctx).await? {
        let member = ctx.member(member);
        if let Ok(Document::Found(mut found)) =
            Box::pin(terraform.mirror_version(&member, host, provider, document)).await
        {
            let tokens = terraform.tokens(ctx).await?;
            if let Some(archives) = found["archives"].as_object_mut() {
                for entry in archives.values_mut() {
                    resign(&tokens, ctx, entry, document);
                }
            }
            return Ok(Document::Found(found));
        }
    }
    Ok(Document::missing(format!(
        "Provider {} version {} not found",
        provider.address(),
        provider.version
    )))
}
