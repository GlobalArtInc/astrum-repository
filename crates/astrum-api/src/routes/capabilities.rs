use astrum_db::audit::Event;
use astrum_db::setting;
use astrum_db::system::{
    BASE_URL_KEY, BRANDING_KEY, BaseUrl, Branding, DEFAULT_TITLE, PROMETHEUS_KEY, Prometheus,
    SPACE_ALERT_KEY, SpaceAlert, UI_SETTINGS_KEY, UiSettings,
};
use astrum_security::sessions::TIMEOUT_KEY;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::error::ApiError;
use crate::{AppState, Caller, REST};

const READ: &str = "astrum:capabilities:read";
const CREATE: &str = "astrum:capabilities:create";
const UPDATE: &str = "astrum:capabilities:update";
const DELETE: &str = "astrum:capabilities:delete";
const DOMAIN: &str = "capability";

pub const BASE_URL: &str = "baseurl";
pub const UI_SETTINGS: &str = "rapture.settings";
pub const BRANDING: &str = "rapture.branding";
pub const PROMETHEUS: &str = "metrics.prometheus";
pub const SPACE_ALERT: &str = "blobstores.space";
const TYPES: [&str; 5] = [BASE_URL, UI_SETTINGS, BRANDING, PROMETHEUS, SPACE_ALERT];

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{REST}/v1/capabilities"), get(list).post(create))
        .route(&format!("{REST}/v1/capabilities/types"), get(types))
        .route(
            &format!("{REST}/v1/capabilities/{{id}}"),
            get(read).put(update).delete(remove),
        )
}

fn type_name(kind: &str) -> &'static str {
    match kind {
        BASE_URL => "Base URL",
        UI_SETTINGS => "UI: Settings",
        PROMETHEUS => "Metrics: Prometheus",
        SPACE_ALERT => "Blob Stores: Space Alert",
        _ => "UI: Branding",
    }
}

fn category(kind: &str) -> &'static str {
    match kind {
        BASE_URL => "Core",
        PROMETHEUS | SPACE_ALERT => "Monitoring",
        _ => "UI",
    }
}

fn field(
    id: &str,
    kind: &str,
    label: &str,
    help: &str,
    required: bool,
    initial: Option<&str>,
    min: Option<&str>,
) -> Value {
    json!({
        "id": id, "type": kind, "label": label, "helpText": help, "required": required,
        "disabled": false, "readOnly": false, "regexValidation": null, "initialValue": initial,
        "attributes": {}, "minValue": min, "maxValue": null, "storeApi": null, "storeFilters": null,
        "idMapping": null, "nameMapping": null, "allowAutocomplete": false,
    })
}

fn type_json(kind: &str) -> Value {
    let (about, fields) = match kind {
        BASE_URL => (
            "Sets the address users reach this server at, for links the server writes into notifications, emails and API answers.\n\n<p>Set it when a reverse proxy in front of Astrum Repository presents another address than the one requests arrive at.</p>",
            vec![field(
                "url",
                "url",
                "URL",
                "The address of the server as users see it, e.g. https://repo.example.com",
                true,
                None,
                None,
            )],
        ),
        UI_SETTINGS => (
            "Adjusts the user interface.",
            vec![
                field(
                    "title",
                    "string",
                    "Title",
                    "The title of the browser tab",
                    true,
                    Some(DEFAULT_TITLE),
                    None,
                ),
                field(
                    "debugAllowed",
                    "checkbox",
                    "Debug allowed",
                    "Let developers debug the user interface",
                    false,
                    Some("true"),
                    None,
                ),
                field(
                    "statusIntervalAuthenticated",
                    "number",
                    "Authenticated user status interval",
                    "Seconds between status requests of signed-in users",
                    true,
                    Some("5"),
                    None,
                ),
                field(
                    "statusIntervalAnonymous",
                    "number",
                    "Anonymous user status interval",
                    "Seconds between status requests of anonymous users",
                    true,
                    Some("60"),
                    None,
                ),
                field(
                    "sessionTimeout",
                    "number",
                    "Session timeout",
                    "Minutes of inactivity after which a session ends; 0 keeps sessions open",
                    true,
                    Some("0"),
                    None,
                ),
                field(
                    "requestTimeout",
                    "number",
                    "Standard request timeout",
                    "Seconds to wait for ordinary requests",
                    true,
                    Some("60"),
                    Some("30"),
                ),
                field(
                    "longRequestTimeout",
                    "number",
                    "Extended request timeout",
                    "Seconds to wait for requests that take long",
                    true,
                    Some("180"),
                    Some("180"),
                ),
            ],
        ),
        PROMETHEUS => (
            "Serves metrics in the Prometheus text format at <code>/service/metrics/prometheus</code>: requests by area and repository, response times, blob stores, health checks, tasks and the database pool.\n\n<p>The scraper signs in like any client, with a user or an access token that holds the privilege <code>as-metrics-all</code> (read access to metrics). Grant it to the anonymous role only when the address cannot be reached from outside.</p>",
            vec![field(
                "repositories",
                "checkbox",
                "Report every repository",
                "Requests and the online state of each repository; turn it off when there are very many repositories",
                false,
                Some("true"),
                None,
            )],
        ),
        SPACE_ALERT => (
            "Watches the free space of the blob stores and sends a message to notification channels when a store runs low, passes its soft quota or cannot be reached, and again when the problem is over.\n\n<p>File, remote and group stores tell their free space; S3 stores only take part through their quota and their reachability.</p>",
            vec![
                field(
                    "minimumFree",
                    "number",
                    "Minimum free space (GB)",
                    "Alert when a blob store has fewer gigabytes free than this",
                    true,
                    Some("10"),
                    Some("1"),
                ),
                field(
                    "channels",
                    "string",
                    "Notification channels",
                    "Names of the channels under Settings, System, Notifications that get the alerts, separated by commas",
                    true,
                    None,
                    None,
                ),
                field(
                    "interval",
                    "number",
                    "Check interval (minutes)",
                    "Minutes between two checks",
                    true,
                    Some("5"),
                    Some("1"),
                ),
                field(
                    "repeat",
                    "number",
                    "Repeat after (hours)",
                    "Hours after which a problem that lasts is reported again; 0 reports it once",
                    true,
                    Some("24"),
                    Some("0"),
                ),
                field(
                    "quota",
                    "checkbox",
                    "Alert when a blob store passes its soft quota",
                    "Uses the soft quota set on the blob store",
                    false,
                    Some("true"),
                    None,
                ),
                field(
                    "unavailable",
                    "checkbox",
                    "Alert when a blob store cannot be reached",
                    "A remote store whose agent does not answer, or a store whose directory is gone",
                    false,
                    Some("true"),
                    None,
                ),
                field(
                    "recovery",
                    "checkbox",
                    "Send a message when a problem is over",
                    "Reports the stores that are fine again",
                    false,
                    Some("true"),
                    None,
                ),
            ],
        ),
        _ => (
            "Adds your own header and footer to the user interface.",
            vec![
                field(
                    "headerEnabled",
                    "checkbox",
                    "Enable header",
                    "Show the header HTML snippet",
                    false,
                    Some("true"),
                    None,
                ),
                field(
                    "headerHtml",
                    "text-area",
                    "Header HTML snippet",
                    "HTML shown above every page; $baseUrl stands for the base URL of the server, e.g. to load an image",
                    false,
                    None,
                    None,
                ),
                field(
                    "footerEnabled",
                    "checkbox",
                    "Enable footer",
                    "Show the footer HTML snippet",
                    false,
                    Some("true"),
                    None,
                ),
                field(
                    "footerHtml",
                    "text-area",
                    "Footer HTML snippet",
                    "HTML shown below every page; $baseUrl stands for the base URL of the server, e.g. to load an image",
                    false,
                    None,
                    None,
                ),
            ],
        ),
    };
    json!({"id": kind, "name": type_name(kind), "about": about, "formFields": fields})
}

struct Stored {
    enabled: bool,
    notes: String,
    properties: Map<String, Value>,
    description: Option<String>,
    status: Option<String>,
}

fn text(value: impl ToString) -> Value {
    Value::String(value.to_string())
}

async fn session_timeout(state: &AppState) -> Result<u32, ApiError> {
    Ok(setting::get::<u32>(state.db().conn(), TIMEOUT_KEY)
        .await?
        .unwrap_or(0))
}

async fn load(state: &AppState, kind: &str) -> Result<Option<Stored>, ApiError> {
    let db = state.db().conn();
    Ok(match kind {
        BASE_URL => setting::get::<BaseUrl>(db, BASE_URL_KEY)
            .await?
            .map(|base| Stored {
                enabled: base.enabled,
                notes: base.notes,
                description: Some(base.url.clone()).filter(|url| !url.is_empty()),
                status: None,
                properties: Map::from_iter([("url".to_owned(), text(base.url))]),
            }),
        UI_SETTINGS => {
            let ui = setting::get::<UiSettings>(db, UI_SETTINGS_KEY)
                .await?
                .unwrap_or_default();
            Some(Stored {
                enabled: ui.enabled,
                notes: ui.notes,
                description: None,
                status: None,
                properties: Map::from_iter([
                    ("title".to_owned(), text(ui.title)),
                    ("debugAllowed".to_owned(), text(ui.debug_allowed)),
                    (
                        "statusIntervalAuthenticated".to_owned(),
                        text(ui.status_interval_authenticated),
                    ),
                    (
                        "statusIntervalAnonymous".to_owned(),
                        text(ui.status_interval_anonymous),
                    ),
                    (
                        "sessionTimeout".to_owned(),
                        text(session_timeout(state).await?),
                    ),
                    ("requestTimeout".to_owned(), text(ui.request_timeout)),
                    (
                        "longRequestTimeout".to_owned(),
                        text(ui.long_request_timeout),
                    ),
                ]),
            })
        }
        BRANDING => setting::get::<Branding>(db, BRANDING_KEY)
            .await?
            .map(|brand| Stored {
                enabled: brand.enabled,
                notes: brand.notes,
                description: None,
                status: None,
                properties: Map::from_iter([
                    ("headerEnabled".to_owned(), text(brand.header_enabled)),
                    ("headerHtml".to_owned(), text(brand.header_html)),
                    ("footerEnabled".to_owned(), text(brand.footer_enabled)),
                    ("footerHtml".to_owned(), text(brand.footer_html)),
                ]),
            }),
        PROMETHEUS => setting::get::<Prometheus>(db, PROMETHEUS_KEY)
            .await?
            .map(|metrics| Stored {
                enabled: metrics.enabled,
                notes: metrics.notes,
                description: Some(crate::metrics::PATH.to_owned()),
                status: Some(match metrics.enabled {
                    true => format!(
                        "Metrics are served at {} to callers with the privilege as-metrics-all.",
                        crate::metrics::PATH
                    ),
                    false => "The metrics endpoint answers 404 while the capability is disabled."
                        .to_owned(),
                }),
                properties: Map::from_iter([(
                    "repositories".to_owned(),
                    text(metrics.repositories),
                )]),
            }),
        SPACE_ALERT => match setting::get::<SpaceAlert>(db, SPACE_ALERT_KEY).await? {
            Some(alert) => Some(Stored {
                enabled: alert.enabled,
                notes: alert.notes,
                description: Some(format!(
                    "Below {} GB free; {}",
                    alert.minimum_free_gigabytes,
                    alert.channels.join(", ")
                )),
                status: Some(match alert.enabled {
                    true => crate::space::status(&crate::space::watch(state).await?),
                    false => "The blob stores are not watched while the capability is disabled."
                        .to_owned(),
                }),
                properties: Map::from_iter([
                    ("minimumFree".to_owned(), text(alert.minimum_free_gigabytes)),
                    ("channels".to_owned(), text(alert.channels.join(", "))),
                    ("interval".to_owned(), text(alert.interval_minutes)),
                    ("repeat".to_owned(), text(alert.repeat_hours)),
                    ("quota".to_owned(), text(alert.quota)),
                    ("unavailable".to_owned(), text(alert.unavailable)),
                    ("recovery".to_owned(), text(alert.recovery)),
                ]),
            }),
            None => None,
        },
        _ => None,
    })
}

fn capability_json(kind: &str, stored: &Stored) -> Value {
    json!({
        "id": kind,
        "enabled": stored.enabled,
        "notes": stored.notes,
        "properties": stored.properties,
        "active": stored.enabled,
        "error": false,
        "description": stored.description,
        "state": if stored.enabled { "active" } else { "disabled" },
        "stateDescription": if stored.enabled { "Active" } else { "Disabled" },
        "status": stored.status,
        "typeName": type_name(kind),
        "tags": {"Category": category(kind)},
        "disableWarningMessage": null,
        "deleteWarningMessage": null,
        "type": kind,
        "isSystem": false,
    })
}

async fn list(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let mut all = Vec::new();
    for kind in TYPES {
        if let Some(stored) = load(&state, kind).await? {
            all.push(capability_json(kind, &stored));
        }
    }
    Ok(Json(Value::Array(all)))
}

async fn types(caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    Ok(Json(Value::Array(
        TYPES.iter().map(|kind| type_json(kind)).collect(),
    )))
}

fn known(id: &str) -> Result<&'static str, ApiError> {
    TYPES
        .into_iter()
        .find(|kind| *kind == id)
        .ok_or_else(|| ApiError::not_found("Capability not found"))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    caller.require(READ)?;
    let kind = known(&id)?;
    let stored = load(&state, kind)
        .await?
        .ok_or_else(|| ApiError::not_found("Capability not found"))?;
    Ok(Json(capability_json(kind, &stored)))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CapabilityXo {
    #[serde(rename = "type")]
    kind: Option<String>,
    enabled: Option<bool>,
    notes: Option<String>,
    properties: Map<String, Value>,
}

fn property(properties: &Map<String, Value>, name: &str) -> Option<String> {
    match properties.get(name)? {
        Value::String(value) => Some(value.clone()),
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

fn flag(properties: &Map<String, Value>, name: &str, default: bool) -> bool {
    property(properties, name)
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn number(
    properties: &Map<String, Value>,
    name: &str,
    label: &str,
    default: u32,
    min: u32,
) -> Result<u32, ApiError> {
    let Some(value) = property(properties, name).filter(|value| !value.trim().is_empty()) else {
        return Ok(default);
    };
    let parsed = value.trim().parse::<u32>().map_err(|_| {
        ApiError::invalid(
            format!("properties.{name}"),
            format!("{label} must be a whole number"),
        )
    })?;
    if parsed < min {
        return Err(ApiError::invalid(
            format!("properties.{name}"),
            format!("{label} must be at least {min}"),
        ));
    }
    Ok(parsed)
}

fn valid_base_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

async fn save(state: &AppState, kind: &str, xo: &CapabilityXo) -> Result<(), ApiError> {
    let db = state.db().conn();
    let enabled = xo.enabled.unwrap_or(true);
    let notes = xo.notes.clone().unwrap_or_default();
    let properties = &xo.properties;
    match kind {
        BASE_URL => {
            let url = property(properties, "url")
                .unwrap_or_default()
                .trim()
                .to_owned();
            if url.is_empty() {
                return Err(ApiError::invalid("properties.url", "URL is required"));
            }
            if !valid_base_url(&url) {
                return Err(ApiError::invalid(
                    "properties.url",
                    "URL must be an http or https URL without credentials, query or fragment",
                ));
            }
            setting::put(
                db,
                BASE_URL_KEY,
                &BaseUrl {
                    url: url.trim_end_matches('/').to_owned(),
                    enabled,
                    notes,
                },
            )
            .await?;
            state.forget_base_url();
        }
        UI_SETTINGS => {
            let defaults = UiSettings::default();
            let title = property(properties, "title")
                .unwrap_or_default()
                .trim()
                .to_owned();
            if title.is_empty() {
                return Err(ApiError::invalid("properties.title", "Title is required"));
            }
            if title.chars().count() > 200 {
                return Err(ApiError::invalid(
                    "properties.title",
                    "Title must not be longer than 200 characters",
                ));
            }
            let settings = UiSettings {
                title,
                debug_allowed: flag(properties, "debugAllowed", defaults.debug_allowed),
                status_interval_authenticated: number(
                    properties,
                    "statusIntervalAuthenticated",
                    "Authenticated user status interval",
                    defaults.status_interval_authenticated,
                    1,
                )?,
                status_interval_anonymous: number(
                    properties,
                    "statusIntervalAnonymous",
                    "Anonymous user status interval",
                    defaults.status_interval_anonymous,
                    1,
                )?,
                request_timeout: number(
                    properties,
                    "requestTimeout",
                    "Standard request timeout",
                    defaults.request_timeout,
                    30,
                )?,
                long_request_timeout: number(
                    properties,
                    "longRequestTimeout",
                    "Extended request timeout",
                    defaults.long_request_timeout,
                    180,
                )?,
                enabled,
                notes,
            };
            let timeout = number(properties, "sessionTimeout", "Session timeout", 0, 0)?;
            setting::put(db, UI_SETTINGS_KEY, &settings).await?;
            setting::put(db, TIMEOUT_KEY, &timeout).await?;
            state.security().changed().await?;
        }
        BRANDING => {
            let defaults = Branding::default();
            let limit = |name: &str| -> Result<String, ApiError> {
                let html = property(properties, name).unwrap_or_default();
                if html.len() > 64 * 1024 {
                    return Err(ApiError::invalid(
                        format!("properties.{name}"),
                        "The HTML snippet must not be longer than 64 KiB",
                    ));
                }
                Ok(html)
            };
            let branding = Branding {
                header_enabled: flag(properties, "headerEnabled", defaults.header_enabled),
                header_html: limit("headerHtml")?,
                footer_enabled: flag(properties, "footerEnabled", defaults.footer_enabled),
                footer_html: limit("footerHtml")?,
                enabled,
                notes,
            };
            setting::put(db, BRANDING_KEY, &branding).await?;
        }
        PROMETHEUS => {
            let defaults = Prometheus::default();
            let metrics = Prometheus {
                repositories: flag(properties, "repositories", defaults.repositories),
                enabled,
                notes,
            };
            setting::put(db, PROMETHEUS_KEY, &metrics).await?;
        }
        SPACE_ALERT => {
            let defaults = SpaceAlert::default();
            let alert = SpaceAlert {
                minimum_free_gigabytes: number(
                    properties,
                    "minimumFree",
                    "Minimum free space",
                    defaults.minimum_free_gigabytes,
                    1,
                )?,
                interval_minutes: number(
                    properties,
                    "interval",
                    "Check interval",
                    defaults.interval_minutes,
                    1,
                )?,
                repeat_hours: number(
                    properties,
                    "repeat",
                    "Repeat after",
                    defaults.repeat_hours,
                    0,
                )?,
                quota: flag(properties, "quota", defaults.quota),
                unavailable: flag(properties, "unavailable", defaults.unavailable),
                recovery: flag(properties, "recovery", defaults.recovery),
                channels: channels(state, properties).await?,
                enabled,
                notes,
            };
            setting::put(db, SPACE_ALERT_KEY, &alert).await?;
            match alert.enabled {
                true => crate::space::recheck_soon(state).await?,
                false => crate::space::forget(state).await?,
            }
        }
        _ => return Err(ApiError::not_found("Capability not found")),
    }
    Ok(())
}

async fn channels(
    state: &AppState,
    properties: &Map<String, Value>,
) -> Result<Vec<String>, ApiError> {
    let mut wanted: Vec<String> = Vec::new();
    for name in property(properties, "channels")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        if !wanted.iter().any(|known| known.eq_ignore_ascii_case(name)) {
            wanted.push(name.to_owned());
        }
    }
    if wanted.is_empty() {
        return Err(ApiError::invalid(
            "properties.channels",
            "Name at least one notification channel",
        ));
    }
    let existing = astrum_notify::store::list(state.db().conn()).await?;
    let mut named = Vec::new();
    let mut unknown = Vec::new();
    for name in wanted {
        match existing
            .iter()
            .find(|channel| channel.name.eq_ignore_ascii_case(&name))
        {
            Some(channel) => named.push(channel.name.clone()),
            None => unknown.push(name),
        }
    }
    if !unknown.is_empty() {
        return Err(ApiError::invalid(
            "properties.channels",
            format!("No notification channel is named {}", unknown.join(", ")),
        ));
    }
    Ok(named)
}

fn audit(kind_of_change: &str, kind: &str, stored: Option<&Stored>) -> Event {
    let mut attributes = json!({"type": kind});
    if let Some(stored) = stored {
        attributes["enabled"] = json!(stored.enabled.to_string());
        attributes["notes"] = json!(stored.notes);
        for (name, value) in &stored.properties {
            attributes[name] = value.clone();
        }
    }
    Event::new(DOMAIN, kind_of_change, type_name(kind)).with(attributes)
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    crate::Json(xo): crate::Json<CapabilityXo>,
) -> Result<impl IntoResponse, ApiError> {
    caller.require(CREATE)?;
    let kind = xo
        .kind
        .as_deref()
        .and_then(|kind| TYPES.into_iter().find(|known| *known == kind))
        .ok_or_else(|| {
            ApiError::invalid("type", "This server does not know that capability type")
        })?;
    let exists = match kind {
        UI_SETTINGS => true,
        _ => load(&state, kind).await?.is_some(),
    };
    if exists {
        return Err(ApiError::invalid(
            "*",
            format!(
                "Only one capability of type '{}' can be created",
                type_name(kind)
            ),
        ));
    }
    save(&state, kind, &xo).await?;
    let stored = load(&state, kind).await?.expect("just saved");
    state
        .audit(&caller, audit("created", kind, Some(&stored)))
        .await;
    Ok((StatusCode::CREATED, Json(capability_json(kind, &stored))))
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
    crate::Json(xo): crate::Json<CapabilityXo>,
) -> Result<Json<Value>, ApiError> {
    caller.require(UPDATE)?;
    let kind = known(&id)?;
    if load(&state, kind).await?.is_none() {
        return Err(ApiError::not_found("Capability not found"));
    }
    save(&state, kind, &xo).await?;
    let stored = load(&state, kind).await?.expect("just saved");
    state
        .audit(&caller, audit("updated", kind, Some(&stored)))
        .await;
    Ok(Json(capability_json(kind, &stored)))
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    caller.require(DELETE)?;
    let kind = known(&id)?;
    let key = match kind {
        BASE_URL => BASE_URL_KEY,
        UI_SETTINGS => UI_SETTINGS_KEY,
        PROMETHEUS => PROMETHEUS_KEY,
        SPACE_ALERT => SPACE_ALERT_KEY,
        _ => BRANDING_KEY,
    };
    if !setting::delete(state.db().conn(), key).await? {
        return Err(ApiError::not_found("Capability not found"));
    }
    if kind == BASE_URL {
        state.forget_base_url();
    }
    if kind == SPACE_ALERT {
        crate::space::forget(&state).await?;
    }
    state.audit(&caller, audit("deleted", kind, None)).await;
    Ok(StatusCode::NO_CONTENT)
}
