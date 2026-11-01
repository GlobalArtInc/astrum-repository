use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as Wait;

use astrum_content::format::{UploadSpec, UploadedFile};
use astrum_content::{Ctx, Format, UploadForm, View};
use astrum_db::audit::Event;
use astrum_db::entity::mcp_upload_ticket;
use astrum_mcp::{ToolError, required_text};
use astrum_repo::RepositoryType;
use astrum_security::{Scope, Subject, password, tokens};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{Duration, SecondsFormat, Utc};
use futures_util::StreamExt;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::{Map, Value, json};
use tokio::io::AsyncWriteExt;

use super::loopback::{Fetched, refusal};
use crate::routes::components::{Failure, repository_failure, require_upload_spec, submit};
use crate::routes::content::context_at;
use crate::routes::internal::client_ip;
use crate::routes::repositories::base_url;
use crate::{ApiError, AppState, Caller, limits};

pub const PATH: &str = "/mcp/upload/{ticket}";
pub const MAX_UPLOAD_BYTES: u64 = 512 * 1024 * 1024;
pub const TICKET_TTL: Duration = Duration::minutes(5);
pub const MAX_OPEN_TICKETS_PER_USER: u64 = 20;
pub const MAX_INLINE_BYTES: usize = 64 * 1024;

const TICKET_PREFIX: &str = "astrum_up_";
const SECRET_LENGTH: usize = 40;
const MAX_FILE_NAME_BYTES: usize = 255;
const MAX_FIELDS: usize = 32;
const MAX_FIELD_CHARS: usize = 2048;
const MAX_REFUSAL_BYTES: usize = 64 * 1024;
const SHOWN_KEY_CHARS: usize = 64;
const UNKNOWN_TICKET: &str = "The upload ticket is unknown, expired or already used.";
const WRITE: &str = "WRITE";
const READ: &str = "READ";

struct Plan {
    ctx: Ctx,
    format: Arc<dyn Format>,
    repository: String,
    file_name: String,
    file_field: String,
    fields: BTreeMap<String, String>,
    not_set: Vec<&'static str>,
}

struct Saved {
    repository: String,
    format: &'static str,
    file_name: String,
    size: u64,
}

impl Saved {
    fn json(&self) -> Value {
        json!({
            "repository": self.repository,
            "format": self.format,
            "fileName": self.file_name,
            "size": self.size,
        })
    }
}

struct Staged {
    path: Option<PathBuf>,
    size: u64,
}

impl Staged {
    fn release(mut self) -> (PathBuf, u64) {
        let path = self.path.take().expect("a staged upload is released once");
        (path, self.size)
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        if let Some(path) = self.path.take()
            && let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, path = %path.display(), "a staged upload was not removed");
        }
    }
}

enum StageError {
    TooLarge,
    Empty,
    Interrupted(String),
    Io(std::io::Error),
}

struct Slot {
    name: &'static str,
    canonical: String,
    aliases: Vec<String>,
    file: bool,
    text_asset: bool,
    optional: bool,
}

fn new_secret() -> String {
    format!("{TICKET_PREFIX}{}", password::random(SECRET_LENGTH))
}

fn looks_like_ticket(value: &str) -> bool {
    value.len() == TICKET_PREFIX.len() + SECRET_LENGTH
        && value.starts_with(TICKET_PREFIX)
        && value[TICKET_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
}

fn scope_text(scope: Scope) -> &'static str {
    match scope {
        Scope::Write => WRITE,
        Scope::Read => READ,
    }
}

fn scope_of(text: &str) -> Scope {
    match text {
        WRITE => Scope::Write,
        _ => Scope::Read,
    }
}

fn valid_file_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("fileName must not be empty".to_owned());
    }
    if name.len() > MAX_FILE_NAME_BYTES {
        return Err(format!(
            "fileName is longer than {MAX_FILE_NAME_BYTES} bytes"
        ));
    }
    if name == "." || name == ".." {
        return Err("fileName must be a file name, not . or ..".to_owned());
    }
    if name
        .chars()
        .any(|c| c == '/' || c == '\\' || c.is_control())
    {
        return Err(
            "fileName is a bare file name: no / or \\ and no control characters".to_owned(),
        );
    }
    Ok(())
}

fn size_limit(arguments: &Map<String, Value>) -> Result<u64, ToolError> {
    match arguments.get("sizeBytes") {
        None | Some(Value::Null) => Ok(MAX_UPLOAD_BYTES),
        Some(value) => match value.as_u64() {
            Some(size) if (1..=MAX_UPLOAD_BYTES).contains(&size) => Ok(size),
            _ => Err(ToolError::invalid(format!(
                "sizeBytes must be a whole number from 1 to {MAX_UPLOAD_BYTES}"
            ))),
        },
    }
}

fn file_name_of(arguments: &Map<String, Value>) -> Result<String, ToolError> {
    let name = arguments
        .get("fileName")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::invalid("the argument fileName is required"))?;
    valid_file_name(name).map_err(ToolError::invalid)?;
    Ok(name.to_owned())
}

fn supplied_fields(arguments: &Map<String, Value>) -> Result<BTreeMap<String, String>, ToolError> {
    match arguments.get("fields") {
        None | Some(Value::Null) => Ok(BTreeMap::new()),
        Some(Value::Object(object)) => object
            .iter()
            .map(|(key, value)| match value {
                Value::String(text) => Ok((key.clone(), text.clone())),
                _ => Err(ToolError::invalid(format!(
                    "the field {} must be a string",
                    shown(key)
                ))),
            })
            .collect(),
        Some(_) => Err(ToolError::invalid(
            "fields must be an object whose values are strings",
        )),
    }
}

fn shown(key: &str) -> String {
    let mut text: String = key.chars().take(SHOWN_KEY_CHARS).collect();
    if text.len() < key.len() {
        text.push('…');
    }
    text
}

fn slots(spec: &UploadSpec) -> Vec<Slot> {
    let format = spec.format;
    let mut slots = Vec::new();
    let mut add =
        |name: &'static str, canonical: String, file: bool, text_asset: bool, optional: bool| {
            let mut aliases = vec![
                name.to_owned(),
                format!("{format}.{name}"),
                canonical.clone(),
            ];
            aliases.dedup();
            slots.push(Slot {
                name,
                canonical,
                aliases,
                file,
                text_asset,
                optional,
            });
        };
    for field in &spec.component_fields {
        add(
            field.name,
            format!("{format}.{}", field.name),
            false,
            false,
            field.optional,
        );
    }
    for field in &spec.asset_fields {
        let file = field.kind == "FILE";
        let canonical = match spec.multiple_upload {
            true if file => format!("{format}.{}1", field.name),
            true => format!("{format}.asset1.{}", field.name),
            false => format!("{format}.{}", field.name),
        };
        add(field.name, canonical, file, !file, field.optional);
    }
    slots
}

fn normalize(
    spec: &UploadSpec,
    file_name: &str,
    supplied: &BTreeMap<String, String>,
) -> Result<(BTreeMap<String, String>, String), String> {
    if supplied.len() > MAX_FIELDS {
        return Err(format!("at most {MAX_FIELDS} fields are accepted"));
    }
    let slots = slots(spec);
    let file_field = slots
        .iter()
        .find(|slot| slot.file)
        .map(|slot| slot.canonical.clone())
        .ok_or_else(|| format!("the {} upload form has no file field", spec.format))?;
    let mut fields = BTreeMap::new();
    for (key, value) in supplied {
        if value.chars().count() > MAX_FIELD_CHARS {
            return Err(format!(
                "the field {} is longer than {MAX_FIELD_CHARS} characters",
                shown(key)
            ));
        }
        let Some(slot) = slots
            .iter()
            .find(|slot| slot.aliases.iter().any(|alias| alias == key))
        else {
            let names: Vec<&str> = slots
                .iter()
                .filter(|slot| !slot.file)
                .map(|slot| slot.name)
                .collect();
            return Err(match names.is_empty() {
                true => format!("{} takes no fields, got {}", spec.format, shown(key)),
                false => format!(
                    "unknown field {}; the {} upload form has: {}",
                    shown(key),
                    spec.format,
                    names.join(", ")
                ),
            });
        };
        if slot.file {
            return Err(format!(
                "the field {} is the file itself: it travels in the body of the PUT, not in fields",
                shown(key)
            ));
        }
        if fields
            .insert(slot.canonical.clone(), value.clone())
            .is_some()
        {
            return Err(format!("the field {} is given twice", slot.name));
        }
    }
    if let Some(slot) = slots
        .iter()
        .find(|slot| slot.text_asset && slot.name == "filename")
    {
        fields
            .entry(slot.canonical.clone())
            .or_insert_with(|| file_name.to_owned());
    }
    Ok((fields, file_field))
}

fn fields_not_set(spec: &UploadSpec, fields: &BTreeMap<String, String>) -> Vec<&'static str> {
    slots(spec)
        .into_iter()
        .filter(|slot| !slot.file && !slot.optional && !fields.contains_key(&slot.canonical))
        .map(|slot| slot.name)
        .collect()
}

fn next_step(not_set: &[&str]) -> String {
    let mut text = "Run the curl command on the machine that holds the file, with <path-to-file> replaced by its path; do not read the file into this conversation. The URL works once. Success is HTTP 201; any other status comes with a JSON field message that gives the reason.".to_owned();
    if !not_set.is_empty() {
        text.push_str(&format!(
            " The upload form of this format also asks for: {}. If the upload is refused for lack of them, request a new URL with them set.",
            not_set.join(", ")
        ));
    }
    text.push_str(" Afterwards check the result with list_versions or search_components.");
    text
}

fn broken(error: impl std::fmt::Display) -> ToolError {
    tracing::error!(%error, "an upload ticket could not be handled");
    ToolError::failed("the server failed to handle the upload ticket; see its log")
}

async fn refused(failure: Failure) -> ToolError {
    let (parts, body) = failure.into_response().into_parts();
    let bytes = axum::body::to_bytes(body, MAX_REFUSAL_BYTES)
        .await
        .unwrap_or_default();
    refusal(&Fetched {
        status: parts.status,
        headers: parts.headers,
        body: bytes.to_vec(),
        truncated: false,
    })
}

async fn through<T>(result: Result<T, Failure>) -> Result<T, ToolError> {
    match result {
        Ok(value) => Ok(value),
        Err(failure) => Err(refused(failure).await),
    }
}

async fn plan(
    state: &AppState,
    subject: &Subject,
    base: &str,
    arguments: &Map<String, Value>,
) -> Result<Plan, ToolError> {
    let repository = required_text(arguments, "repository")?;
    let file_name = file_name_of(arguments)?;
    let supplied = supplied_fields(arguments)?;
    let (ctx, format) = through(
        context_at(
            state,
            Caller(Some(subject.clone())),
            &repository,
            base.to_owned(),
            crate::auth::client_address().unwrap_or_default(),
        )
        .await
        .map_err(|error| repository_failure(error, &repository)),
    )
    .await?;
    if !ctx.grant(View::Add).any() {
        return Err(ToolError::failed(format!(
            "you may not add content to {repository}"
        )));
    }
    if ctx.kind() != RepositoryType::Hosted {
        return Err(ToolError::failed(format!(
            "{repository} is a {} repository: only hosted repositories take uploads",
            ctx.kind().as_str()
        )));
    }
    let spec = through(require_upload_spec(&*format)).await?;
    let (fields, file_field) =
        normalize(&spec, &file_name, &supplied).map_err(ToolError::invalid)?;
    let not_set = fields_not_set(&spec, &fields);
    Ok(Plan {
        ctx,
        format,
        repository,
        file_name,
        file_field,
        fields,
        not_set,
    })
}

async fn staging_path(state: &AppState) -> std::io::Result<PathBuf> {
    let directory = state.data_dir().join("tmp");
    tokio::fs::create_dir_all(&directory).await?;
    Ok(directory.join(format!("upload-{}", uuid::Uuid::new_v4())))
}

async fn stage_bytes(state: &AppState, bytes: &[u8]) -> Result<Staged, ToolError> {
    let path = staging_path(state).await.map_err(broken)?;
    let staged = Staged {
        path: Some(path.clone()),
        size: bytes.len() as u64,
    };
    tokio::fs::write(&path, bytes).await.map_err(broken)?;
    Ok(staged)
}

async fn stage_stream(state: &AppState, body: Body, limit: u64) -> Result<Staged, StageError> {
    let path = staging_path(state).await.map_err(StageError::Io)?;
    let mut staged = Staged {
        path: Some(path.clone()),
        size: 0,
    };
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(StageError::Io)?;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| StageError::Interrupted(error.to_string()))?;
        staged.size += chunk.len() as u64;
        if staged.size > limit {
            return Err(StageError::TooLarge);
        }
        file.write_all(&chunk).await.map_err(StageError::Io)?;
    }
    file.flush().await.map_err(StageError::Io)?;
    if staged.size == 0 {
        return Err(StageError::Empty);
    }
    Ok(staged)
}

async fn save(
    state: &AppState,
    subject: &Subject,
    plan: Plan,
    staged: Staged,
) -> Result<Saved, Failure> {
    let format = plan.format.id();
    let (path, size) = staged.release();
    let form = UploadForm {
        fields: plan.fields,
        files: BTreeMap::from([(
            plan.file_field,
            UploadedFile {
                filename: Some(plan.file_name.clone()),
                path,
                size,
            },
        )]),
    };
    submit(plan.ctx, &*plan.format, form).await?;
    state
        .audit(
            &Caller(Some(subject.clone())),
            Event::new("repository.upload", "uploaded", &plan.repository).with(json!({
                "format": format,
                "fileName": plan.file_name,
                "size": size,
            })),
        )
        .await;
    Ok(Saved {
        repository: plan.repository,
        format,
        file_name: plan.file_name,
        size,
    })
}

pub(super) async fn request_upload(
    state: &AppState,
    subject: &Subject,
    base: &str,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let max_bytes = size_limit(arguments)?;
    let plan = plan(state, subject, base, arguments).await?;
    let now = Utc::now().fixed_offset();
    let db = state.db().conn();
    mcp_upload_ticket::Entity::delete_many()
        .filter(mcp_upload_ticket::Column::ExpiresAt.lte(now))
        .exec(db)
        .await
        .map_err(broken)?;
    let open = mcp_upload_ticket::Entity::find()
        .filter(mcp_upload_ticket::Column::UserId.eq(subject.user_id.as_str()))
        .filter(mcp_upload_ticket::Column::ExpiresAt.gt(now))
        .count(db)
        .await
        .map_err(broken)?;
    if open >= MAX_OPEN_TICKETS_PER_USER {
        return Err(ToolError::failed(format!(
            "you already hold {MAX_OPEN_TICKETS_PER_USER} open upload tickets: use them or wait until they expire"
        )));
    }
    let secret = new_secret();
    let expires_at = now + TICKET_TTL;
    mcp_upload_ticket::Entity::insert(mcp_upload_ticket::ActiveModel {
        ticket_hash: ActiveValue::Set(tokens::hash(&secret)),
        user_id: ActiveValue::Set(subject.user_id.clone()),
        realm: ActiveValue::Set(subject.realm.clone()),
        scope: ActiveValue::Set(scope_text(subject.scope).to_owned()),
        repository: ActiveValue::Set(plan.repository.clone()),
        fields: ActiveValue::Set(json!(plan.fields)),
        file_field: ActiveValue::Set(plan.file_field.clone()),
        file_name: ActiveValue::Set(plan.file_name.clone()),
        max_bytes: ActiveValue::Set(max_bytes as i64),
        client: ActiveValue::Set(super::client().unwrap_or_else(|| "access token".to_owned())),
        created_at: ActiveValue::Set(now),
        expires_at: ActiveValue::Set(expires_at),
    })
    .exec(db)
    .await
    .map_err(broken)?;
    let expires = expires_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    state
        .audit(
            &Caller(Some(subject.clone())),
            Event::new("repository.upload", "ticket-issued", &plan.repository).with(json!({
                "fileName": plan.file_name,
                "maxBytes": max_bytes,
                "expiresAt": expires,
            })),
        )
        .await;
    let upload_url = format!("{base}/mcp/upload/{secret}");
    Ok(json!({
        "repository": plan.repository,
        "format": plan.format.id(),
        "method": "PUT",
        "uploadUrl": upload_url,
        "curl": format!("curl -sS -w '\\nHTTP %{{http_code}}\\n' -T <path-to-file> '{upload_url}'"),
        "expiresAt": expires,
        "expiresInSeconds": TICKET_TTL.num_seconds(),
        "maxBytes": max_bytes,
        "singleUse": true,
        "fields": plan.fields,
        "fieldsNotSet": plan.not_set,
        "next": next_step(&plan.not_set),
    }))
}

pub(super) async fn upload_component(
    state: &AppState,
    subject: &Subject,
    base: &str,
    arguments: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let encoded = arguments
        .get("contentBase64")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::invalid("the argument contentBase64 is required"))?;
    if encoded.len() > MAX_INLINE_BYTES.div_ceil(3) * 4 {
        return Err(too_big_inline());
    }
    let bytes = STANDARD.decode(encoded).map_err(|error| {
        ToolError::invalid(format!(
            "contentBase64 is not standard base64 without line breaks: {error}"
        ))
    })?;
    if bytes.is_empty() {
        return Err(ToolError::invalid("contentBase64 holds no bytes"));
    }
    if bytes.len() > MAX_INLINE_BYTES {
        return Err(too_big_inline());
    }
    let plan = plan(state, subject, base, arguments).await?;
    let staged = stage_bytes(state, &bytes).await?;
    let saved = through(save(state, subject, plan, staged).await).await?;
    Ok(saved.json())
}

fn too_big_inline() -> ToolError {
    ToolError::failed(format!(
        "upload_component takes files up to {MAX_INLINE_BYTES} bytes: use request_upload for this one"
    ))
}

fn problem(status: StatusCode, message: &str) -> Response {
    (status, axum::Json(json!({ "id": "*", "message": message }))).into_response()
}

fn too_many(wait: Wait) -> Response {
    let mut response = problem(
        StatusCode::TOO_MANY_REQUESTS,
        "Too many upload attempts; slow down and try again later.",
    );
    if let Ok(value) = HeaderValue::from_str(&wait.as_secs().max(1).to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

fn too_large(limit: u64) -> Response {
    problem(
        StatusCode::PAYLOAD_TOO_LARGE,
        &format!("The file is larger than the {limit} bytes this ticket allows."),
    )
}

fn declared_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

async fn eat(
    state: &AppState,
    ticket: &str,
) -> Result<Option<mcp_upload_ticket::Model>, sea_orm::DbErr> {
    if !looks_like_ticket(ticket) {
        return Ok(None);
    }
    let eaten = mcp_upload_ticket::Entity::delete_many()
        .filter(mcp_upload_ticket::Column::TicketHash.eq(tokens::hash(ticket)))
        .filter(mcp_upload_ticket::Column::ExpiresAt.gt(Utc::now().fixed_offset()))
        .exec_with_returning(state.db().conn())
        .await?;
    Ok(eaten.into_iter().next())
}

pub(super) async fn receive(
    State(state): State<AppState>,
    Path(ticket): Path<String>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response {
    let key = format!(
        "upload:{}",
        crate::auth::client_address().unwrap_or_default()
    );
    if let Err(wait) = state.limits().take(&key, limits::UPLOAD_ATTEMPTS) {
        return too_many(wait);
    }
    let row = match eat(&state, &ticket).await {
        Ok(Some(row)) => row,
        Ok(None) => return problem(StatusCode::NOT_FOUND, UNKNOWN_TICKET),
        Err(error) => return ApiError::from(error).into_response(),
    };
    let client = row.client.clone();
    super::scoped(client, accept(state, row, headers, uri, body)).await
}

async fn accept(
    state: AppState,
    row: mcp_upload_ticket::Model,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response {
    let Some(subject) = state.security().subject_of(&row.user_id, &row.realm).await else {
        return problem(
            StatusCode::FORBIDDEN,
            "The user this ticket was issued to can no longer upload.",
        );
    };
    let subject = subject.with_scope(scope_of(&row.scope));
    let limit = u64::try_from(row.max_bytes).unwrap_or(0);
    if declared_length(&headers).is_some_and(|length| length > limit) {
        return too_large(limit);
    }
    let fields: BTreeMap<String, String> = match serde_json::from_value(row.fields) {
        Ok(fields) => fields,
        Err(error) => return ApiError::internal(error).into_response(),
    };
    let (ctx, format) = match context_at(
        &state,
        Caller(Some(subject.clone())),
        &row.repository,
        base_url(&headers, &uri),
        crate::auth::client_address().unwrap_or_else(|| client_ip(&headers)),
    )
    .await
    {
        Ok(found) => found,
        Err(error) => return repository_failure(error, &row.repository).into_response(),
    };
    if let Err(failure) = require_upload_spec(&*format) {
        return failure.into_response();
    }
    if !ctx.grant(View::Add).any() {
        return ApiError::forbidden().into_response();
    }
    let staged = match stage_stream(&state, body, limit).await {
        Ok(staged) => staged,
        Err(StageError::TooLarge) => return too_large(limit),
        Err(StageError::Empty) => {
            return problem(StatusCode::BAD_REQUEST, "The upload body is empty.");
        }
        Err(StageError::Interrupted(error)) => {
            return problem(
                StatusCode::BAD_REQUEST,
                &format!("The upload was interrupted: {error}"),
            );
        }
        Err(StageError::Io(error)) => return ApiError::internal(error).into_response(),
    };
    let plan = Plan {
        ctx,
        format,
        repository: row.repository,
        file_name: row.file_name,
        file_field: row.file_field,
        fields,
        not_set: Vec::new(),
    };
    match save(&state, &subject, plan, staged).await {
        Ok(saved) => (StatusCode::CREATED, axum::Json(saved.json())).into_response(),
        Err(failure) => failure.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supplied(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    }

    fn normalized(
        format: &dyn Format,
        file_name: &str,
        pairs: &[(&str, &str)],
    ) -> Result<(BTreeMap<String, String>, String), String> {
        normalize(&format.upload_spec().unwrap(), file_name, &supplied(pairs))
    }

    #[test]
    fn raw_fields_get_their_form_names_and_the_file_name() {
        let (fields, file) =
            normalized(&astrum_format_raw::Raw, "a.txt", &[("directory", "/docs")]).unwrap();
        assert_eq!(file, "raw.asset1");
        assert_eq!(
            fields,
            supplied(&[("raw.directory", "/docs"), ("raw.asset1.filename", "a.txt")])
        );
        let (named, _) = normalized(
            &astrum_format_raw::Raw,
            "a.txt",
            &[("raw.directory", "/d"), ("filename", "b.txt")],
        )
        .unwrap();
        assert_eq!(
            named,
            supplied(&[("raw.directory", "/d"), ("raw.asset1.filename", "b.txt")])
        );
    }

    #[test]
    fn npm_takes_no_fields_and_reads_raw_asset() {
        let (fields, file) = normalized(&astrum_format_npm::Npm::default(), "a.tgz", &[]).unwrap();
        assert_eq!(file, "npm.asset");
        assert!(fields.is_empty());
        let refused =
            normalized(&astrum_format_npm::Npm::default(), "a.tgz", &[("tag", "x")]).unwrap_err();
        assert!(refused.contains("takes no fields"), "{refused}");
    }

    #[test]
    fn required_fields_left_unset_are_listed() {
        let raw = astrum_format_raw::Raw.upload_spec().unwrap();
        let unset = |spec: &UploadSpec, pairs: &[(&str, &str)]| {
            let (fields, _) = normalize(spec, "a.txt", &supplied(pairs)).unwrap();
            fields_not_set(spec, &fields)
        };
        assert_eq!(unset(&raw, &[]), ["directory"]);
        assert!(unset(&raw, &[("directory", "/d")]).is_empty());
        let npm = astrum_format_npm::Npm::default().upload_spec().unwrap();
        assert!(unset(&npm, &[]).is_empty());
    }

    #[test]
    fn maven_assets_move_under_the_first_asset() {
        let (fields, file) = normalized(
            &astrum_format_maven::Maven,
            "lib-1.0.jar",
            &[
                ("groupId", "org.example"),
                ("artifactId", "lib"),
                ("maven2.version", "1.0"),
                ("extension", "jar"),
                ("maven2.asset1.classifier", "sources"),
            ],
        )
        .unwrap();
        assert_eq!(file, "maven2.asset1");
        assert_eq!(
            fields,
            supplied(&[
                ("maven2.groupId", "org.example"),
                ("maven2.artifactId", "lib"),
                ("maven2.version", "1.0"),
                ("maven2.asset1.extension", "jar"),
                ("maven2.asset1.classifier", "sources"),
            ])
        );
    }

    #[test]
    fn wrong_fields_are_refused_with_the_valid_names() {
        let unknown =
            normalized(&astrum_format_raw::Raw, "a.txt", &[("folder", "/x")]).unwrap_err();
        assert!(unknown.contains("directory, filename"), "{unknown}");
        let file =
            normalized(&astrum_format_raw::Raw, "a.txt", &[("raw.asset1", "x")]).unwrap_err();
        assert!(file.contains("the file itself"), "{file}");
        let twice = normalized(
            &astrum_format_raw::Raw,
            "a.txt",
            &[("directory", "/x"), ("raw.directory", "/y")],
        )
        .unwrap_err();
        assert!(twice.contains("given twice"), "{twice}");
    }

    #[test]
    fn fields_are_limited_in_number_and_size() {
        let keys: Vec<String> = (0..=MAX_FIELDS).map(|index| format!("k{index}")).collect();
        let many: BTreeMap<String, String> = keys
            .iter()
            .map(|key| (key.clone(), String::new()))
            .collect();
        let spec = astrum_format_raw::Raw.upload_spec().unwrap();
        assert!(
            normalize(&spec, "a", &many)
                .unwrap_err()
                .contains("at most")
        );
        let long = "x".repeat(MAX_FIELD_CHARS);
        assert!(normalize(&spec, "a", &supplied(&[("directory", &long)])).is_ok());
        let too_long = "x".repeat(MAX_FIELD_CHARS + 1);
        assert!(
            normalize(&spec, "a", &supplied(&[("directory", &too_long)]))
                .unwrap_err()
                .contains("longer than")
        );
    }

    #[test]
    fn file_names_are_bare_names() {
        for good in ["a.tgz", "@scope-pkg-1.0.tgz", ".hidden", &"n".repeat(255)] {
            assert!(valid_file_name(good).is_ok(), "{good}");
        }
        for bad in [
            "",
            "  ",
            ".",
            "..",
            "a/b",
            "a\\b",
            "a\nb",
            "a\u{0}b",
            &"n".repeat(256),
        ] {
            assert!(valid_file_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn sizes_are_bounded() {
        let size = |value: Value| size_limit(&Map::from_iter([("sizeBytes".to_owned(), value)]));
        assert_eq!(size_limit(&Map::new()).unwrap(), MAX_UPLOAD_BYTES);
        assert_eq!(size(json!(10)).unwrap(), 10);
        assert_eq!(size(json!(MAX_UPLOAD_BYTES)).unwrap(), MAX_UPLOAD_BYTES);
        for bad in [
            json!(0),
            json!(-1),
            json!(MAX_UPLOAD_BYTES + 1),
            json!(1.5),
            json!("9"),
        ] {
            assert!(size(bad.clone()).is_err(), "{bad}");
        }
    }

    #[test]
    fn tickets_are_random_shaped_and_stored_hashed() {
        let first = new_secret();
        let second = new_secret();
        assert_ne!(first, second);
        assert!(looks_like_ticket(&first), "{first}");
        assert!(!tokens::looks_like_token(&first));
        assert!(!looks_like_ticket(&first[..first.len() - 1]));
        assert!(!looks_like_ticket(&format!("{first}x")));
        assert!(!looks_like_ticket(
            &first.replace("astrum_up_", "astrum_at_")
        ));
        assert!(!looks_like_ticket(&format!(
            "{}!",
            &first[..first.len() - 1]
        )));
        let hashed = tokens::hash(&first);
        assert_eq!(hashed.len(), 64);
        assert_ne!(hashed, first);
        assert_eq!(hashed, tokens::hash(&first));
    }

    #[test]
    fn a_ticket_keeps_its_scope_and_unknown_scopes_only_read() {
        assert_eq!(scope_of(scope_text(Scope::Write)), Scope::Write);
        assert_eq!(scope_of(scope_text(Scope::Read)), Scope::Read);
        assert_eq!(scope_of("ADMIN"), Scope::Read);
    }
}
