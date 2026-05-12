use std::sync::Arc;

use astrum_content::{ContentError, ContentRequest, Ctx, View, group, proxy};
use astrum_repo::{Repository, RepositoryType};
use axum::http::{HeaderValue, Method, header};
use axum::response::{IntoResponse, Response};

const MAX_REQUEST: usize = 16 * 1024 * 1024;
const MAX_ANSWER: usize = 32 * 1024 * 1024;
const FORWARDED: [header::HeaderName; 3] = [
    header::CONTENT_TYPE,
    header::CONTENT_ENCODING,
    header::ACCEPT,
];

pub async fn audit(ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
    if request.method != Method::POST {
        return Err(ContentError::MethodNotAllowed {
            method: request.method.to_string(),
            allow: "POST",
        });
    }
    ctx.require(View::Read, "/")?;
    let Some(auditor) = auditor(ctx).await? else {
        return Err(ContentError::BadRequest(
            "security audits are answered only through npm proxy repositories".to_owned(),
        ));
    };
    let body = request.bytes(MAX_REQUEST).await?;
    let auditor = ctx.member(auditor);
    let url = proxy::remote_url(&auditor.repository, &request.path, request.query.as_deref())?;
    let headers: Vec<(header::HeaderName, String)> = FORWARDED
        .iter()
        .filter_map(|name| {
            request
                .header(name.as_str())
                .map(|value| (name.clone(), value.to_owned()))
        })
        .collect();
    let answer = proxy::send_body(&auditor, Method::POST, url, &headers, Some(body)).await?;
    let status = answer.status;
    let kind = answer
        .header(header::CONTENT_TYPE.as_str())
        .and_then(|value| HeaderValue::from_str(value).ok());
    let encoding = answer
        .header(header::CONTENT_ENCODING.as_str())
        .and_then(|value| HeaderValue::from_str(value).ok());
    let bytes = answer.bytes(MAX_ANSWER).await?;
    let mut response = (status, bytes).into_response();
    let headers = response.headers_mut();
    if let Some(kind) = kind {
        headers.insert(header::CONTENT_TYPE, kind);
    }
    if let Some(encoding) = encoding {
        headers.insert(header::CONTENT_ENCODING, encoding);
    }
    if status.is_server_error() {
        tracing::info!(repository = ctx.name(), %status, "the remote failed a security audit");
    }
    Ok(response)
}

async fn auditor(ctx: &Ctx) -> Result<Option<Arc<Repository>>, ContentError> {
    match ctx.kind() {
        RepositoryType::Proxy => Ok(Some(ctx.repository.clone())),
        RepositoryType::Hosted => Ok(None),
        RepositoryType::Group => Ok(group::members(ctx).await?.into_iter().find(|member| {
            member.kind == RepositoryType::Proxy
                && !proxy::remote_blocked(&ctx.member(member.clone()))
        })),
    }
}
