use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use serde::de::DeserializeOwned;

use crate::error::ApiError;

pub struct Json<T>(pub T);

impl<S, T> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(Json(value)),
            Err(JsonRejection::MissingJsonContentType(_)) => {
                Err(ApiError::UnsupportedMediaType("JSON (application/json)"))
            }
            Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                Err(ApiError::PayloadTooLarge(rejection.body_text()))
            }
            Err(rejection) => Err(ApiError::BadRequest(rejection.body_text())),
        }
    }
}
