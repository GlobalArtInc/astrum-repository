use std::collections::BTreeMap;

use astrum_blob::ContentStream;
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, View};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

use crate::gem::{self, GemError};
use crate::index::{self, InfoEntry, VersionsLine};
use crate::{GEM, MAX_FORM, Route, adopt, form_fields, read_spec, spec_of, text};

pub struct Entries {
    pub by_name: BTreeMap<String, Vec<InfoEntry>>,
    pub updated: Option<DateTime<Utc>>,
}

fn like(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

async fn gems(ctx: &Ctx, name: Option<&str>) -> Result<Vec<Asset>, ContentError> {
    let mut query = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(GEM));
    if let Some(name) = name {
        query = query.filter(Expr::col(asset::Column::Path).like(format!(
            "{}{}-%",
            gem::GEMS_PREFIX,
            like(name)
        )));
    }
    Ok(query
        .order_by_asc(asset::Column::Path)
        .all(ctx.db())
        .await?)
}

pub async fn entries(ctx: &Ctx, name: Option<&str>) -> Result<Entries, ContentError> {
    let mut by_name: BTreeMap<String, Vec<InfoEntry>> = BTreeMap::new();
    let mut updated = None;
    for found in gems(ctx, name).await? {
        if !ctx.allowed(View::Read, &found.path) {
            continue;
        }
        let Some(spec) = spec_of(&found) else {
            continue;
        };
        if name.is_some_and(|name| spec.name != name) {
            continue;
        }
        updated = updated.max(Some(found.blob_updated_at.with_timezone(&Utc)));
        by_name
            .entry(spec.name.clone())
            .or_default()
            .push(InfoEntry::of(&spec, &found.sha256));
    }
    Ok(Entries { by_name, updated })
}

fn versions(entries: &Entries) -> Vec<u8> {
    let lines: Vec<VersionsLine> = entries
        .by_name
        .iter()
        .map(|(name, list)| VersionsLine {
            name: name.clone(),
            versions: index::versions_of(list),
            checksum: index::md5_hex(&index::info(list)),
        })
        .collect();
    index::versions(
        entries.updated.unwrap_or(DateTime::<Utc>::UNIX_EPOCH),
        &lines,
    )
}

pub async fn index(ctx: &Ctx, route: &Route) -> Result<Option<Vec<u8>>, ContentError> {
    Ok(match route {
        Route::Versions => Some(versions(&entries(ctx, None).await?)),
        Route::Names => Some(index::names(entries(ctx, None).await?.by_name.keys())),
        Route::Info(name) => entries(ctx, Some(name))
            .await?
            .by_name
            .get(name)
            .map(|list| index::info(list)),
        Route::Specs(kind) => {
            let found = entries(ctx, None).await?;
            let versions: BTreeMap<String, Vec<String>> = found
                .by_name
                .iter()
                .map(|(name, list)| (name.clone(), index::versions_of(list)))
                .collect();
            Some(index::legacy_specs(*kind, &versions))
        }
        _ => None,
    })
}

fn redeploy_refused(ctx: &Ctx) -> Response {
    text(
        StatusCode::CONFLICT,
        format!(
            "Repushing of gem versions is not allowed: repository {} does not allow redeployment",
            ctx.name()
        ),
    )
}

pub async fn push(ctx: &Ctx, body: ContentStream) -> Result<Response, ContentError> {
    if !ctx.grant(View::Add).any() && !ctx.grant(View::Edit).any() {
        ctx.require(View::Add, &Route::Push.path())?;
    }
    let written = store::write_blob(ctx, body).await?;
    let discard = || store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    if written.size == 0 {
        discard();
        return Ok(text(
            StatusCode::BAD_REQUEST,
            "Missing gem file in the request body",
        ));
    }
    let spec = match read_spec(ctx, &written.store, written.id).await {
        Ok(spec) => spec,
        Err(error) => {
            discard();
            let message = match error {
                GemError::NotGem(message) => format!("Not a gem file: {message}"),
                GemError::Invalid(message) => message,
            };
            return Ok(text(StatusCode::UNPROCESSABLE_ENTITY, message));
        }
    };
    let path = spec.gem_path();
    let policy = ctx
        .repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    let view = match exists && policy == WritePolicy::Allow {
        true => View::Edit,
        false => View::Add,
    };
    if let Err(error) = ctx.require(view, &path) {
        discard();
        return Err(error);
    }
    match policy {
        WritePolicy::Deny => {
            discard();
            return Ok(text(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Repository {} is read-only", ctx.name()),
            ));
        }
        WritePolicy::AllowOnce if exists => {
            discard();
            return Ok(redeploy_refused(ctx));
        }
        _ => {}
    }
    match adopt(ctx, &spec, PutAsset::hosted(ctx, &path), &written).await {
        Ok(_) => {}
        Err(ContentError::Conflict(_)) => return Ok(redeploy_refused(ctx)),
        Err(error) => return Err(error),
    }
    tracing::info!(
        repository = ctx.name(),
        gem = spec.full_name(),
        "a gem was pushed"
    );
    Ok(text(
        StatusCode::OK,
        format!(
            "Successfully registered gem: {} ({})",
            spec.name,
            spec.version_and_platform()
        ),
    ))
}

pub async fn yank(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    if !ctx.grant(View::Delete).any() {
        ctx.require(View::Delete, &Route::Yank.path())?;
    }
    let body = request.bytes(MAX_FORM).await?;
    let form = form_fields(request.query.as_deref(), &body);
    let name = form.get("gem_name").map(String::as_str).unwrap_or_default();
    let version = form
        .get("version")
        .map(|value| value.trim())
        .unwrap_or_default();
    let platform = form
        .get("platform")
        .map(String::as_str)
        .filter(|value| !value.is_empty());
    if !gem::valid_name(name) {
        return Ok(text(
            StatusCode::BAD_REQUEST,
            "A valid gem_name is required",
        ));
    }
    if !gem::valid_version(version) {
        return Ok(text(StatusCode::BAD_REQUEST, "A valid version is required"));
    }
    if let Some(platform) = platform
        && !gem::valid_platform(platform)
    {
        return Ok(text(
            StatusCode::BAD_REQUEST,
            format!("Invalid platform: {platform}"),
        ));
    }
    let platform = gem::normalize_platform(platform);
    let path = gem::gem_path(&gem::full_name(name, version, &platform));
    let version_and_platform = gem::version_and_platform(version, &platform);
    let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? else {
        return Ok(text(
            StatusCode::NOT_FOUND,
            format!("The version {version_and_platform} of {name} does not exist."),
        ));
    };
    ctx.require(View::Delete, &path)?;
    store::delete_asset(&ctx.content, found).await?;
    tracing::info!(repository = ctx.name(), path, "a gem was yanked");
    Ok(text(
        StatusCode::OK,
        format!("Successfully deleted gem: {name} ({version_and_platform})"),
    ))
}
