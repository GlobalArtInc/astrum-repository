use astrum_db::audit::Event;
use astrum_mail::{EmailRequest, MailError, Message};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::error::{ApiError, FieldError};
use crate::routes::outbound::{SETTINGS_READ, SETTINGS_UPDATE};
use crate::{AppState, Caller};

pub const DOMAIN: &str = "email";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/service/rest/v1/email",
            get(read).put(update).delete(reset),
        )
        .route("/service/rest/v1/email/verify", post(verify))
}

fn failed(error: MailError) -> ApiError {
    ApiError::internal(error)
}

async fn read(State(state): State<AppState>, caller: Caller) -> Result<Json<Value>, ApiError> {
    caller.require(SETTINGS_READ)?;
    Ok(Json(
        state.mailer().settings().await.map_err(failed)?.document(),
    ))
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<EmailRequest>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let crate::Json(request) = body?;
    let mailer = state.mailer();
    let stored = mailer.settings().await.map_err(failed)?;
    let secrets = state.secrets().clone();
    let seal = move |plain: &str| secrets.seal(plain);
    let prepared = request.prepare(&stored, &seal).map_err(|problems| {
        ApiError::Invalid(
            problems
                .into_iter()
                .map(|problem| FieldError {
                    id: problem.id,
                    message: problem.message,
                })
                .collect(),
        )
    })?;
    mailer.save(&prepared).await.map_err(failed)?;
    let mut attributes = prepared.document();
    if let Value::Object(map) = &mut attributes {
        map.remove("password");
    }
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "updated", "email").with(attributes),
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reset(State(state): State<AppState>, caller: Caller) -> Result<StatusCode, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    state.mailer().reset().await.map_err(failed)?;
    state
        .audit(&caller, Event::new(DOMAIN, "deleted", "email"))
        .await;
    Ok(StatusCode::NO_CONTENT)
}

fn address(body: &Bytes) -> String {
    let text = String::from_utf8_lossy(body).trim().to_owned();
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::String(address)) => address.trim().to_owned(),
        Ok(Value::Object(map)) => map
            .get("address")
            .or_else(|| map.get("email"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned(),
        _ => text,
    }
}

async fn verify(
    State(state): State<AppState>,
    caller: Caller,
    body: Bytes,
) -> Result<Response, ApiError> {
    caller.require(SETTINGS_UPDATE)?;
    let to = address(&body);
    let mailer = state.mailer();
    let settings = mailer.settings().await.map_err(failed)?;
    let message = Message {
        to: vec![to.clone()],
        subject: "Email configuration test".to_owned(),
        text: format!(
            "This message was sent by Astrum Repository to check its email settings.\n\nMail server: {}:{}\n",
            settings.host, settings.port
        ),
    };
    let outcome = mailer.send_with(&settings, &message).await;
    state
        .audit(
            &caller,
            Event::new(DOMAIN, "verified", "email").with(json!({
                "to": to,
                "success": outcome.is_ok(),
            })),
        )
        .await;
    Ok(match outcome {
        Ok(()) => Json(json!({"success": true, "reason": null})).into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"success": false, "reason": error.to_string()})),
        )
            .into_response(),
    })
}
