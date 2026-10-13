use astrum_content::{ContentError, ContentRequest, Ctx};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use serde_json::{Value, json};

pub const X_TERRAFORM_GET: &str = "X-Terraform-Get";

pub fn json(
    request: &ContentRequest,
    status: StatusCode,
    value: &Value,
) -> Result<Response, ContentError> {
    let body = serde_json::to_vec(value).map_err(ContentError::internal)?;
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

pub fn error(
    request: &ContentRequest,
    status: StatusCode,
    message: &str,
) -> Result<Response, ContentError> {
    json(request, status, &json!({ "errors": [message] }))
}

pub fn location(location: &str) -> Result<Response, ContentError> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(X_TERRAFORM_GET, location)
        .body(Body::empty())
        .map_err(ContentError::internal)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn directory(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Terraform registry. Terraform finds its modules and providers through <a href=\".well-known/terraform.json\">service discovery</a>; providers are also served through the network mirror protocol.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}
