use std::io;

use astrum_blob::ContentStream;
use astrum_content::{ContentError, ContentRequest, Ctx};
use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::TryStreamExt;
use serde_json::json;

use crate::hosted;
use crate::provenance::MAX_PROVENANCE;

const CHART_FIELD: &str = "chart";
const PROVENANCE_FIELD: &str = "prov";

pub struct PushError(pub ContentError);

impl From<ContentError> for PushError {
    fn from(error: ContentError) -> Self {
        PushError(error)
    }
}

impl IntoResponse for PushError {
    fn into_response(self) -> Response {
        let status = self.0.status();
        let message = match &self.0 {
            ContentError::Internal(error) => {
                tracing::error!(error = format!("{error:#}"), "a chart push failed");
                "the server failed; see its log".to_owned()
            }
            other => other.to_string(),
        };
        let mut response = (status, Json(json!({ "error": message }))).into_response();
        if matches!(self.0, ContentError::Unauthorized) {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(astrum_content::CHALLENGE),
            );
        }
        response
    }
}

fn bad(error: multer::Error) -> ContentError {
    ContentError::BadRequest(format!("the upload form cannot be read: {error}"))
}

fn multipart(request: &ContentRequest) -> Option<multer::Multipart<'static>> {
    let boundary = request
        .header("content-type")
        .and_then(|value| multer::parse_boundary(value).ok())?;
    let constraints = multer::Constraints::new().size_limit(
        multer::SizeLimit::new()
            .per_field(MAX_PROVENANCE as u64)
            .for_field(CHART_FIELD, u64::MAX),
    );
    Some(multer::Multipart::with_constraints(
        request.take_body().into_data_stream(),
        boundary,
        constraints,
    ))
}

async fn small(field: multer::Field<'static>) -> Result<Vec<u8>, ContentError> {
    Ok(field.bytes().await.map_err(bad)?.to_vec())
}

pub async fn charts(ctx: &Ctx, request: ContentRequest) -> Result<Response, PushError> {
    hosted::may_add(ctx)?;
    let Some(mut form) = multipart(&request) else {
        hosted::upload_chart(ctx, request.into_stream()).await?;
        return Ok(saved());
    };
    let mut chart = None;
    let mut provenance = None;
    while let Some(field) = form.next_field().await.map_err(bad)? {
        match field.name() {
            Some(CHART_FIELD) if chart.is_none() => {
                let body: ContentStream = Box::pin(field.map_err(io::Error::other));
                chart = Some(hosted::upload_chart(ctx, body).await?);
            }
            Some(PROVENANCE_FIELD) if provenance.is_none() => {
                provenance = Some(small(field).await?);
            }
            Some(CHART_FIELD) | Some(PROVENANCE_FIELD) => {
                return Err(ContentError::BadRequest(
                    "the form holds more than one chart or provenance file".to_owned(),
                )
                .into());
            }
            _ => {
                let mut field = field;
                while field.chunk().await.map_err(bad)?.is_some() {}
            }
        }
    }
    let Some((_, stored)) = chart else {
        return Err(ContentError::BadRequest("the form has no chart field".to_owned()).into());
    };
    if let Some(text) = provenance {
        let signed = crate::provenance::chart(&String::from_utf8_lossy(&text))
            .map_err(|error| ContentError::BadRequest(error.to_string()))?;
        if signed.file_name() != stored.file_name() {
            return Err(ContentError::BadRequest(format!(
                "the provenance file signs {} {}, not the chart {} {}",
                signed.name, signed.version, stored.name, stored.version
            ))
            .into());
        }
        hosted::put_provenance(ctx, None, &text).await?;
    }
    Ok(saved())
}

pub async fn provenance(ctx: &Ctx, request: ContentRequest) -> Result<Response, PushError> {
    hosted::may_add(ctx)?;
    let text = match multipart(&request) {
        None => request.bytes(MAX_PROVENANCE).await?.to_vec(),
        Some(mut form) => {
            let mut found = None;
            while let Some(field) = form.next_field().await.map_err(bad)? {
                match field.name() {
                    Some(PROVENANCE_FIELD) if found.is_none() => found = Some(small(field).await?),
                    _ => {
                        let mut field = field;
                        while field.chunk().await.map_err(bad)?.is_some() {}
                    }
                }
            }
            found
                .ok_or_else(|| ContentError::BadRequest("the form has no prov field".to_owned()))?
        }
    };
    hosted::put_provenance(ctx, None, &text).await?;
    Ok(saved())
}

fn saved() -> Response {
    (StatusCode::CREATED, Json(json!({ "saved": true }))).into_response()
}
