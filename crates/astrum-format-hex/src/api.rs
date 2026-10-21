use astrum_content::{Asset, CHALLENGE, ContentError, ContentRequest, Ctx};
use astrum_repo::RepositoryType;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value, json};

use crate::proto::RetirementReason;
use crate::registry::{Record, Retirement};
use crate::tarball::{self, MAX_TARBALL};
use crate::{Hex, ID, etf, hosted};

const MAX_BODY: usize = 64 * 1024;
const MAX_MESSAGE: usize = 140;
const LOCATION: &str = "location";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Api {
    Publish,
    Releases(String),
    Package(String),
    Release(String, String),
    Retire(String, String),
    Docs(String, String),
    Owners(String),
    Owner(String, String),
    Me,
    Auth,
    Index,
    Other,
}

fn named(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn route(parts: &[&str]) -> Api {
    let parts = match parts {
        ["repos", repository, rest @ ..] if !repository.is_empty() => rest,
        other => other,
    };
    match parts {
        [] | [""] => Api::Index,
        ["publish"] => Api::Publish,
        ["users", "me"] => Api::Me,
        ["auth"] => Api::Auth,
        ["packages", name] if named(name) => Api::Package((*name).to_owned()),
        ["packages", name, "releases"] if named(name) => Api::Releases((*name).to_owned()),
        ["packages", name, "releases", version] if named(name) => {
            Api::Release((*name).to_owned(), (*version).to_owned())
        }
        ["packages", name, "releases", version, "retire"] if named(name) => {
            Api::Retire((*name).to_owned(), (*version).to_owned())
        }
        ["packages", name, "releases", version, "docs"] if named(name) => {
            Api::Docs((*name).to_owned(), (*version).to_owned())
        }
        ["packages", name, "owners"] if named(name) => Api::Owners((*name).to_owned()),
        ["packages", name, "owners", user] if named(name) && !user.is_empty() => {
            Api::Owner((*name).to_owned(), (*user).to_owned())
        }
        _ => Api::Other,
    }
}

fn wants_erlang(request: &ContentRequest) -> bool {
    request
        .header("accept")
        .is_some_and(|accept| accept.contains(etf::CONTENT_TYPE))
}

fn reply(erlang: bool, status: StatusCode, value: &Value) -> Response {
    let (content_type, body) = match erlang {
        true => (etf::CONTENT_TYPE, etf::encode(value)),
        false => ("application/json", value.to_string().into_bytes()),
    };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .expect("an API answer is valid")
}

fn empty(status: StatusCode) -> Response {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .expect("an API answer is valid")
}

fn api_url(ctx: &Ctx) -> String {
    format!("{}/api", ctx.repository_url())
}

fn time(at: chrono::DateTime<chrono::FixedOffset>) -> String {
    at.with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn sign_in_message(ctx: &Ctx) -> String {
    format!(
        "Create an access token in Astrum Repository at {}/user/access-tokens and give it to the client as its API key, for example HEX_API_KEY=astrum_...",
        ctx.base_url
    )
}

fn failure(ctx: &Ctx, erlang: bool, error: ContentError) -> Response {
    let status = match &error {
        ContentError::BadRequest(_) | ContentError::Conflict(_) => StatusCode::UNPROCESSABLE_ENTITY,
        other => other.status(),
    };
    let message = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(error = format!("{inner:#}"), "a Hex API request failed");
            "the server failed; see its log".to_owned()
        }
        ContentError::Unauthorized => sign_in_message(ctx),
        ContentError::Forbidden => format!(
            "your user lacks the privileges this needs on the repository {}; ask an administrator of Astrum Repository for them",
            ctx.name()
        ),
        other => other.to_string(),
    };
    let mut response = reply(
        erlang,
        status,
        &json!({"status": status.as_u16(), "message": message}),
    );
    match error {
        ContentError::Unauthorized => {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(CHALLENGE),
            );
        }
        ContentError::MethodNotAllowed { allow, .. } => {
            response
                .headers_mut()
                .insert(header::ALLOW, HeaderValue::from_static(allow));
        }
        _ => {}
    }
    response
}

fn hosted_only(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.kind() {
        RepositoryType::Hosted => Ok(()),
        kind => Err(ContentError::BadRequest(format!(
            "{} is a {kind} repository; the Hex API publishes to and describes hosted repositories",
            ctx.name()
        ))),
    }
}

fn not_allowed(method: &Method, allow: &'static str) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow,
    }
}

fn replace_wanted(request: &ContentRequest) -> bool {
    url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
        .any(|(key, value)| key == "replace" && value == "true")
}

fn configs(ctx: &Ctx, name: &str, version: &str) -> Value {
    let requirement = match semver::Version::parse(version) {
        Ok(parsed) if parsed.pre.is_empty() => format!("~> {}.{}", parsed.major, parsed.minor),
        _ => version.to_owned(),
    };
    json!({
        "mix.exs": format!("{{:{name}, \"{requirement}\", repo: \"{}\"}}", ctx.name()),
        "rebar.config": format!("{{{name}, \"{version}\"}}"),
        "erlang.mk": format!("dep_{name} = hex {version}"),
    })
}

fn docs_url(ctx: &Ctx, name: &str, version: &str, has_docs: bool) -> Value {
    match has_docs {
        true => json!(format!(
            "{}{}",
            ctx.repository_url(),
            crate::docs_path(name, version)
        )),
        false => Value::Null,
    }
}

fn release_document(ctx: &Ctx, found: &Asset, record: &Record, has_docs: bool) -> Value {
    let hex = found.attributes.get(ID).cloned().unwrap_or(Value::Null);
    let mut requirements = Map::new();
    for requirement in &record.requirements {
        let mut entry = json!({
            "app": requirement.app.clone().unwrap_or_else(|| requirement.name.clone()),
            "optional": requirement.optional,
            "requirement": requirement.requirement,
        });
        if let Some(repository) = &requirement.repository {
            entry["repository"] = json!(repository);
        }
        requirements.insert(requirement.name.clone(), entry);
    }
    let api = api_url(ctx);
    json!({
        "version": record.version,
        "checksum": found.sha256,
        "url": format!("{api}/packages/{}/releases/{}", record.name, record.version),
        "package_url": format!("{api}/packages/{}", record.name),
        "html_url": format!("{}/browse/{}", ctx.base_url, ctx.name()),
        "docs_html_url": docs_url(ctx, &record.name, &record.version, has_docs),
        "has_docs": has_docs,
        "configs": configs(ctx, &record.name, &record.version),
        "meta": {
            "app": hex.get("app").cloned().unwrap_or_else(|| json!(record.name)),
            "build_tools": hex.get("buildTools").cloned().unwrap_or_else(|| json!([])),
            "elixir": hex.get("elixir").cloned().unwrap_or(Value::Null),
        },
        "requirements": requirements,
        "retirement": record.retired.as_ref().map(Retirement::attributes).unwrap_or(Value::Null),
        "downloads": found.download_count,
        "inserted_at": time(found.created_at),
        "updated_at": time(found.blob_updated_at),
    })
}

fn package_document(ctx: &Ctx, name: &str, releases: &[(Asset, Record, bool)]) -> Value {
    let api = api_url(ctx);
    let latest = releases
        .last()
        .map(|(found, _, _)| found.attributes.get(ID).cloned().unwrap_or(Value::Null))
        .unwrap_or(Value::Null);
    let mut retirements = Map::new();
    for (_, record, _) in releases {
        if let Some(retired) = &record.retired {
            retirements.insert(record.version.clone(), retired.attributes());
        }
    }
    let stable = releases
        .iter()
        .rev()
        .map(|(_, record, _)| record)
        .find(|record| {
            semver::Version::parse(&record.version).is_ok_and(|version| version.pre.is_empty())
        })
        .map(|record| record.version.clone());
    let latest_version = releases.last().map(|(_, record, _)| record.version.clone());
    let configured = stable
        .as_deref()
        .or(latest_version.as_deref())
        .map(|version| configs(ctx, name, version))
        .unwrap_or(Value::Null);
    json!({
        "name": name,
        "repository": ctx.name(),
        "url": format!("{api}/packages/{name}"),
        "html_url": format!("{}/browse/{}", ctx.base_url, ctx.name()),
        "docs_html_url": Value::Null,
        "meta": {
            "description": latest.get("description").cloned().unwrap_or(Value::Null),
            "licenses": latest.get("licenses").cloned().unwrap_or_else(|| json!([])),
            "links": latest.get("links").cloned().unwrap_or_else(|| json!({})),
            "maintainers": [],
        },
        "releases": releases.iter().rev().map(|(found, record, has_docs)| json!({
            "version": record.version,
            "url": format!("{api}/packages/{name}/releases/{}", record.version),
            "has_docs": has_docs,
            "inserted_at": time(found.created_at),
        })).collect::<Vec<_>>(),
        "retirements": retirements,
        "owners": [],
        "configs": configured,
        "latest_version": latest_version,
        "latest_stable_version": stable,
        "downloads": {"all": releases.iter().map(|(found, _, _)| found.download_count).sum::<i64>()},
        "inserted_at": releases.first().map(|(found, _, _)| json!(time(found.created_at))).unwrap_or(Value::Null),
        "updated_at": releases.iter().map(|(found, _, _)| found.blob_updated_at).max().map(|at| json!(time(at))).unwrap_or(Value::Null),
    })
}

fn retirement_of(request: &ContentRequest, body: &[u8]) -> Result<Retirement, ContentError> {
    let json_body = request
        .header("content-type")
        .is_some_and(|value| value.contains("json"));
    let given = match json_body {
        true => serde_json::from_slice::<Value>(body)
            .map_err(|error| ContentError::BadRequest(format!("the body is not JSON: {error}")))?,
        false => etf::decode(body).map_err(|error| ContentError::BadRequest(error.to_string()))?,
    };
    let reason = given
        .get("reason")
        .and_then(Value::as_str)
        .and_then(RetirementReason::from_word)
        .ok_or_else(|| {
            ContentError::BadRequest(
                "give reason as one of other, invalid, security, deprecated or renamed".to_owned(),
            )
        })?;
    let message = given
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_owned);
    if message
        .as_ref()
        .is_some_and(|message| message.chars().count() > MAX_MESSAGE)
    {
        return Err(ContentError::BadRequest(format!(
            "the message may be at most {MAX_MESSAGE} characters long"
        )));
    }
    Ok(Retirement { reason, message })
}

type Answer = Result<(StatusCode, Option<Value>), ContentError>;

impl Hex {
    pub(crate) async fn api(&self, ctx: &Ctx, request: ContentRequest, api: Api) -> Response {
        let erlang = wants_erlang(&request);
        match self.answer(ctx, request, api).await {
            Ok((StatusCode::FOUND, Some(document))) => {
                let mut response = empty(StatusCode::FOUND);
                if let Some(location) = document[LOCATION]
                    .as_str()
                    .and_then(|location| HeaderValue::from_str(location).ok())
                {
                    response.headers_mut().insert(header::LOCATION, location);
                }
                response
            }
            Ok((status, Some(document))) => reply(erlang, status, &document),
            Ok((status, None)) => empty(status),
            Err(error) => failure(ctx, erlang, error),
        }
    }

    async fn answer(&self, ctx: &Ctx, request: ContentRequest, api: Api) -> Answer {
        let method = request.method.clone();
        match (&method, api) {
            (&Method::POST, Api::Publish) => self.publish(ctx, request, None).await,
            (&Method::POST, Api::Releases(name)) => self.publish(ctx, request, Some(name)).await,
            (_, Api::Publish | Api::Releases(_)) => Err(not_allowed(&method, "POST")),
            (&Method::GET | &Method::HEAD, Api::Package(name)) => {
                let releases = self.described(ctx, &name).await?;
                Ok((
                    StatusCode::OK,
                    Some(package_document(ctx, &name, &releases)),
                ))
            }
            (_, Api::Package(_)) => Err(not_allowed(&method, "GET,HEAD")),
            (&Method::GET | &Method::HEAD, Api::Release(name, version)) => {
                hosted_only(ctx)?;
                let found = hosted::release(ctx, &name, &version).await?;
                let record = hosted::record(&found).ok_or_else(|| {
                    ContentError::not_found(format!("{name} {version} has no metadata"))
                })?;
                let has_docs = hosted::has_docs(ctx, &name, &version).await?;
                Ok((
                    StatusCode::OK,
                    Some(release_document(ctx, &found, &record, has_docs)),
                ))
            }
            (&Method::DELETE, Api::Release(name, version)) => {
                hosted_only(ctx)?;
                hosted::remove(ctx, &self.keys, &name, &version).await?;
                Ok((StatusCode::NO_CONTENT, None))
            }
            (_, Api::Release(_, _)) => Err(not_allowed(&method, "GET,HEAD,DELETE")),
            (&Method::POST, Api::Retire(name, version)) => {
                hosted_only(ctx)?;
                hosted::may_add(ctx)?;
                let body = request.bytes(MAX_BODY).await?;
                let retirement = retirement_of(&request, &body)?;
                hosted::retire(ctx, &self.keys, &name, &version, Some(retirement)).await?;
                Ok((StatusCode::NO_CONTENT, None))
            }
            (&Method::DELETE, Api::Retire(name, version)) => {
                hosted_only(ctx)?;
                hosted::retire(ctx, &self.keys, &name, &version, None).await?;
                Ok((StatusCode::NO_CONTENT, None))
            }
            (_, Api::Retire(_, _)) => Err(not_allowed(&method, "POST,DELETE")),
            (&Method::POST | &Method::PUT, Api::Docs(name, version)) => {
                hosted_only(ctx)?;
                hosted::may_add(ctx)?;
                let body = request.bytes(MAX_TARBALL as usize).await?;
                let replaced = hosted::put_docs(ctx, &name, &version, body).await?;
                let location = format!("{}/packages/{name}/releases/{version}/docs", api_url(ctx));
                Ok((
                    match replaced {
                        true => StatusCode::OK,
                        false => StatusCode::CREATED,
                    },
                    Some(json!({"url": location})),
                ))
            }
            (&Method::GET | &Method::HEAD, Api::Docs(name, version)) => {
                hosted_only(ctx)?;
                hosted::release(ctx, &name, &version).await?;
                match hosted::has_docs(ctx, &name, &version).await? {
                    true => Ok((
                        StatusCode::FOUND,
                        Some(json!({ LOCATION: docs_url(ctx, &name, &version, true) })),
                    )),
                    false => Err(ContentError::not_found(format!(
                        "{name} {version} has no documentation"
                    ))),
                }
            }
            (&Method::DELETE, Api::Docs(name, version)) => {
                hosted_only(ctx)?;
                hosted::remove_docs(ctx, &name, &version).await?;
                Ok((StatusCode::NO_CONTENT, None))
            }
            (_, Api::Docs(_, _)) => Err(not_allowed(&method, "GET,HEAD,POST,PUT,DELETE")),
            (&Method::GET | &Method::HEAD, Api::Owners(name)) => {
                self.described(ctx, &name).await?;
                Ok((StatusCode::OK, Some(json!([]))))
            }
            (_, Api::Owners(_)) => Err(not_allowed(&method, "GET,HEAD")),
            (&Method::PUT, Api::Owner(_, _)) => {
                hosted_only(ctx)?;
                hosted::may_add(ctx)?;
                Ok((StatusCode::NO_CONTENT, None))
            }
            (_, Api::Owner(_, _)) => Err(ContentError::BadRequest(
                "Astrum Repository keeps no package owners; who may publish is decided by the privileges on the repository".to_owned(),
            )),
            (&Method::GET | &Method::HEAD, Api::Me) => match &ctx.subject {
                Some(subject) if !subject.anonymous => Ok((
                    StatusCode::OK,
                    Some(json!({
                        "username": subject.user_id,
                        "url": format!("{}/users/me", api_url(ctx)),
                        "organizations": [],
                    })),
                )),
                _ => Err(ContentError::Unauthorized),
            },
            (_, Api::Me) => Err(not_allowed(&method, "GET,HEAD")),
            (&Method::GET | &Method::HEAD, Api::Auth) => match &ctx.subject {
                Some(subject) if !subject.anonymous => Ok((StatusCode::NO_CONTENT, None)),
                _ => Err(ContentError::Unauthorized),
            },
            (_, Api::Auth) => Err(not_allowed(&method, "GET,HEAD")),
            (&Method::GET | &Method::HEAD, Api::Index) => {
                let api = api_url(ctx);
                Ok((
                    StatusCode::OK,
                    Some(json!({
                        "packages_url": format!("{api}/packages"),
                        "package_url": format!("{api}/packages/{{name}}"),
                        "package_release_url": format!("{api}/packages/{{name}}/releases/{{version}}"),
                        "package_owners_url": format!("{api}/packages/{{name}}/owners"),
                        "repository_url": ctx.repository_url(),
                    })),
                ))
            }
            (_, Api::Index) => Err(not_allowed(&method, "GET,HEAD")),
            (_, Api::Other) => Err(ContentError::not_found(format!(
                "{} is not part of the Hex API this repository answers",
                request.path
            ))),
        }
    }

    async fn publish(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        expected: Option<String>,
    ) -> Answer {
        hosted_only(ctx)?;
        hosted::may_add(ctx)?;
        if request
            .header("content-length")
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|length| length > MAX_TARBALL)
        {
            return Err(ContentError::BadRequest(format!(
                "the package tarball is larger than {MAX_TARBALL} bytes"
            )));
        }
        let replace = replace_wanted(&request);
        let body = request.bytes(MAX_TARBALL as usize).await?;
        let published =
            hosted::publish(ctx, &self.keys, body, replace, expected.as_deref(), None).await?;
        let found = hosted::release(ctx, &published.name, &published.version).await?;
        let record = hosted::record(&found).ok_or_else(|| {
            ContentError::internal(std::io::Error::other("a published release has no metadata"))
        })?;
        let has_docs = hosted::has_docs(ctx, &published.name, &published.version).await?;
        tracing::info!(
            repository = ctx.name(),
            file = tarball::file_name(&published.name, &published.version),
            "a Hex package was published"
        );
        let status = match published.replaced {
            true => StatusCode::OK,
            false => StatusCode::CREATED,
        };
        Ok((
            status,
            Some(release_document(ctx, &found, &record, has_docs)),
        ))
    }

    async fn described(
        &self,
        ctx: &Ctx,
        name: &str,
    ) -> Result<Vec<(Asset, Record, bool)>, ContentError> {
        hosted_only(ctx)?;
        let releases = hosted::releases(ctx, name).await?;
        if releases.is_empty() {
            return Err(ContentError::not_found(format!(
                "the package {name} is not in {}",
                ctx.name()
            )));
        }
        let mut described = Vec::with_capacity(releases.len());
        for (found, record) in releases {
            let has_docs = hosted::has_docs(ctx, name, &record.version).await?;
            described.push((found, record, has_docs));
        }
        Ok(described)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_paths_route_with_and_without_a_repository() {
        assert_eq!(route(&["publish"]), Api::Publish);
        assert_eq!(route(&["repos", "acme", "publish"]), Api::Publish);
        assert_eq!(route(&["users", "me"]), Api::Me);
        assert_eq!(
            route(&["packages", "hello"]),
            Api::Package("hello".to_owned())
        );
        assert_eq!(
            route(&["packages", "hello", "releases"]),
            Api::Releases("hello".to_owned())
        );
        assert_eq!(
            route(&["repos", "acme", "packages", "hello", "releases", "1.0.0"]),
            Api::Release("hello".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(
            route(&["packages", "hello", "releases", "1.0.0", "retire"]),
            Api::Retire("hello".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(
            route(&["packages", "hello", "releases", "1.0.0", "docs"]),
            Api::Docs("hello".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(
            route(&["packages", "hello", "owners"]),
            Api::Owners("hello".to_owned())
        );
        assert_eq!(
            route(&["packages", "hello", "owners", "jane"]),
            Api::Owner("hello".to_owned(), "jane".to_owned())
        );
        assert_eq!(route(&["auth"]), Api::Auth);
        assert_eq!(route(&[]), Api::Index);
        assert_eq!(route(&[""]), Api::Index);
        assert_eq!(route(&["packages", "a-b"]), Api::Other);
        assert_eq!(route(&["keys"]), Api::Other);
    }
}
