use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rustls::pki_types::CertificateDer;
use serde_json::{Map, Value};

use super::OidcError;
use super::config::OidcSettings;
use super::jwt::{self, Key};
use crate::ssrf::{Guard, GuardedResolver, SsrfProtection, literal_allowed};
use crate::tls;

const METADATA_TTL: Duration = Duration::from_secs(3600);
const JWKS_REFRESH: Duration = Duration::from_secs(60);
const CLOCK_SKEW: i64 = 60;
const TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE: usize = 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    pub issuer: Option<String>,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub end_session_endpoint: Option<String>,
    pub token_auth_methods: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tokens {
    pub id_token: String,
    pub access_token: Option<String>,
}

struct CachedMetadata {
    fingerprint: String,
    at: Instant,
    metadata: Metadata,
}

struct CachedKeys {
    fingerprint: String,
    at: Instant,
    keys: Vec<Key>,
}

#[derive(Default)]
pub struct OidcProvider {
    metadata: Mutex<Option<CachedMetadata>>,
    keys: Mutex<Option<CachedKeys>>,
}

impl std::fmt::Debug for OidcProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OidcProvider")
    }
}

pub struct Http {
    client: reqwest::Client,
    guard: Arc<Guard>,
}

impl Http {
    pub fn new(
        ssrf: &SsrfProtection,
        trust_store: Option<&[CertificateDer<'static>]>,
    ) -> Result<Self, OidcError> {
        let guard = Arc::new(Guard::new(ssrf));
        let redirect_guard = guard.clone();
        let redirects = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                attempt.error("too many redirects")
            } else if !literal_allowed(&redirect_guard, attempt.url()) {
                attempt.error("the redirect goes to an address the SSRF protection blocks")
            } else {
                attempt.follow()
            }
        });
        let mut builder = reqwest::Client::builder()
            .user_agent(format!("Astrum/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(TIMEOUT)
            .timeout(TIMEOUT)
            .redirect(redirects)
            .no_proxy()
            .dns_resolver(Arc::new(GuardedResolver::new(guard.clone())));
        if let Some(certificates) = trust_store {
            match tls::http_client_config(certificates) {
                Ok(config) => builder = builder.use_preconfigured_tls(config),
                Err(error) => {
                    tracing::warn!(%error, "the truststore cannot be used; the system roots apply")
                }
            }
        }
        let client = builder
            .build()
            .map_err(|error| OidcError::failed(format!("cannot build an HTTP client: {error}")))?;
        Ok(Self { client, guard })
    }

    async fn json(&self, request: reqwest::RequestBuilder, url: &str) -> Result<Value, OidcError> {
        let parsed = url::Url::parse(url)
            .map_err(|_| OidcError::failed(format!("{url} is not a valid URL")))?;
        if !literal_allowed(&self.guard, &parsed) {
            return Err(OidcError::failed(format!(
                "{url} points to an address the SSRF protection blocks"
            )));
        }
        let mut response = request
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                OidcError::failed(format!("Request to {url} failed: {}", describe(&error)))
            })?;
        let status = response.status();
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| OidcError::failed(format!("Request to {url} failed: {error}")))?
        {
            body.extend_from_slice(&chunk);
            if body.len() > MAX_RESPONSE {
                return Err(OidcError::failed(format!(
                    "{url} returned an oversized response"
                )));
            }
        }
        if !status.is_success() {
            return Err(OidcError::failed(format!(
                "{url} returned HTTP {}{}",
                status.as_u16(),
                error_details(&body)
            )));
        }
        serde_json::from_slice(&body)
            .map_err(|_| OidcError::failed(format!("{url} did not return JSON")))
    }

    pub async fn get(&self, url: &str, bearer: Option<&str>) -> Result<Value, OidcError> {
        let mut request = self.client.get(url);
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        self.json(request, url).await
    }
}

fn describe(error: &reqwest::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(inner) = source {
        text.push_str(": ");
        text.push_str(&inner.to_string());
        source = inner.source();
    }
    text
}

fn error_details(body: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return String::new();
    };
    match value.get("error").and_then(Value::as_str) {
        Some(error) => match value.get("error_description").and_then(Value::as_str) {
            Some(description) => format!(" ({error}: {description})"),
            None => format!(" ({error})"),
        },
        None => String::new(),
    }
}

fn text(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

pub fn strip_trailing_slash(value: &str) -> String {
    value.trim().trim_end_matches('/').to_owned()
}

fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

impl OidcProvider {
    pub fn reset(&self) {
        *self
            .metadata
            .lock()
            .expect("the OIDC lock is never poisoned") = None;
        *self.keys.lock().expect("the OIDC lock is never poisoned") = None;
    }

    pub async fn resolve(settings: &OidcSettings, http: &Http) -> Result<Metadata, OidcError> {
        let discovery = match &settings.issuer_url {
            Some(issuer) => Some(
                http.get(
                    &format!(
                        "{}/.well-known/openid-configuration",
                        strip_trailing_slash(issuer)
                    ),
                    None,
                )
                .await?,
            ),
            None => None,
        };
        let from_discovery = |field: &str| discovery.as_ref().and_then(|value| text(value, field));
        let issuer = from_discovery("issuer")
            .or_else(|| settings.issuer_url.as_deref().map(strip_trailing_slash));
        let methods = discovery
            .as_ref()
            .and_then(|value| value.get("token_endpoint_auth_methods_supported"))
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        let metadata = Metadata {
            issuer,
            authorization_endpoint: settings
                .idp_authorization_url
                .clone()
                .or_else(|| from_discovery("authorization_endpoint"))
                .ok_or_else(|| {
                    OidcError::failed("The provider does not define an authorization endpoint")
                })?,
            token_endpoint: settings
                .idp_token_url
                .clone()
                .or_else(|| from_discovery("token_endpoint"))
                .ok_or_else(|| {
                    OidcError::failed("The provider does not define a token endpoint")
                })?,
            userinfo_endpoint: from_discovery("userinfo_endpoint"),
            jwks_uri: settings
                .idp_jwks_url
                .clone()
                .or_else(|| from_discovery("jwks_uri")),
            end_session_endpoint: settings
                .idp_logout_url
                .clone()
                .or_else(|| from_discovery("end_session_endpoint")),
            token_auth_methods: methods,
        };
        if metadata.jwks_uri.is_none() && settings.idp_jwks.is_none() {
            return Err(OidcError::failed(
                "The provider does not define a JWKS endpoint and no JSON Web Keys are configured",
            ));
        }
        Ok(metadata)
    }

    pub async fn metadata(
        &self,
        settings: &OidcSettings,
        http: &Http,
    ) -> Result<Metadata, OidcError> {
        let fingerprint = settings.fingerprint();
        if let Some(cached) = self
            .metadata
            .lock()
            .expect("the OIDC lock is never poisoned")
            .as_ref()
            .filter(|cached| {
                cached.fingerprint == fingerprint && cached.at.elapsed() < METADATA_TTL
            })
        {
            return Ok(cached.metadata.clone());
        }
        let metadata = Self::resolve(settings, http).await?;
        *self
            .metadata
            .lock()
            .expect("the OIDC lock is never poisoned") = Some(CachedMetadata {
            fingerprint,
            at: Instant::now(),
            metadata: metadata.clone(),
        });
        Ok(metadata)
    }

    pub async fn exchange_code(
        &self,
        settings: &OidcSettings,
        client_secret: Option<&str>,
        http: &Http,
        code: &str,
        redirect_uri: &str,
        verifier: &str,
    ) -> Result<Tokens, OidcError> {
        let metadata = self.metadata(settings, http).await?;
        let mut form: Vec<(String, String)> = vec![
            ("grant_type".to_owned(), "authorization_code".to_owned()),
            ("code".to_owned(), code.to_owned()),
            ("redirect_uri".to_owned(), redirect_uri.to_owned()),
            ("code_verifier".to_owned(), verifier.to_owned()),
            ("client_id".to_owned(), settings.client_id.clone()),
        ];
        let post = !metadata.token_auth_methods.is_empty()
            && !metadata
                .token_auth_methods
                .iter()
                .any(|method| method == "client_secret_basic")
            && metadata
                .token_auth_methods
                .iter()
                .any(|method| method == "client_secret_post");
        let mut request = http.client.post(&metadata.token_endpoint);
        if let Some(secret) = client_secret.filter(|secret| !secret.is_empty()) {
            if post {
                form.push(("client_secret".to_owned(), secret.to_owned()));
            } else {
                let credentials = format!("{}:{}", encode(&settings.client_id), encode(secret));
                request = request.header(
                    reqwest::header::AUTHORIZATION,
                    format!("Basic {}", STANDARD.encode(credentials)),
                );
            }
        }
        for (name, value) in &settings.token_request_custom_params {
            form.push((name.clone(), value.clone()));
        }
        let body = form
            .iter()
            .map(|(name, value)| format!("{}={}", encode(name), encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        let request = request
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body);
        let response = http.json(request, &metadata.token_endpoint).await?;
        let id_token = text(&response, "id_token")
            .ok_or_else(|| OidcError::failed("The token response does not contain an ID token"))?;
        Ok(Tokens {
            id_token,
            access_token: text(&response, "access_token"),
        })
    }

    async fn keys(
        &self,
        settings: &OidcSettings,
        metadata: &Metadata,
        http: &Http,
        key_id: Option<&str>,
    ) -> Result<Vec<Key>, OidcError> {
        let fingerprint = settings.fingerprint();
        if let Some(cached) = self
            .keys
            .lock()
            .expect("the OIDC lock is never poisoned")
            .as_ref()
            .filter(|cached| cached.fingerprint == fingerprint)
            .filter(|cached| {
                jwt::has_key(&cached.keys, key_id) || cached.at.elapsed() < JWKS_REFRESH
            })
        {
            return Ok(cached.keys.clone());
        }
        let jwks = match &settings.idp_jwks {
            Some(inline) => serde_json::from_str(inline).map_err(|_| {
                OidcError::failed("The configured JSON Web Keys are not valid JSON")
            })?,
            None => {
                let uri = metadata.jwks_uri.as_deref().ok_or_else(|| {
                    OidcError::failed("The provider does not define a JWKS endpoint")
                })?;
                http.get(uri, None).await?
            }
        };
        let keys = jwt::parse_jwks(&jwks).map_err(|error| OidcError::failed(error.0))?;
        *self.keys.lock().expect("the OIDC lock is never poisoned") = Some(CachedKeys {
            fingerprint,
            at: Instant::now(),
            keys: keys.clone(),
        });
        Ok(keys)
    }

    pub async fn verify_id_token(
        &self,
        settings: &OidcSettings,
        http: &Http,
        token: &str,
        nonce: &str,
    ) -> Result<Map<String, Value>, OidcError> {
        let metadata = self.metadata(settings, http).await?;
        let decoded = jwt::decode(token).map_err(|error| OidcError::failed(error.0))?;
        if let Some(wanted) = &settings.idp_jws_algorithm
            && *wanted != decoded.algorithm
        {
            return Err(OidcError::failed(format!(
                "The ID token is signed with {} instead of {wanted}",
                decoded.algorithm
            )));
        }
        let keys = self
            .keys(settings, &metadata, http, decoded.key_id.as_deref())
            .await?;
        decoded
            .verify(&keys)
            .map_err(|error| OidcError::failed(error.0))?;
        check_claims(
            &decoded.claims,
            &settings.client_id,
            metadata.issuer.as_deref(),
            nonce,
            chrono::Utc::now().timestamp(),
        )?;
        Ok(decoded.claims)
    }

    pub async fn user_info(
        &self,
        settings: &OidcSettings,
        http: &Http,
        access_token: &str,
    ) -> Result<Map<String, Value>, OidcError> {
        let metadata = self.metadata(settings, http).await?;
        let Some(endpoint) = metadata.userinfo_endpoint else {
            return Ok(Map::new());
        };
        match http.get(&endpoint, Some(access_token)).await? {
            Value::Object(claims) => Ok(claims),
            _ => Ok(Map::new()),
        }
    }
}

pub fn check_claims(
    claims: &Map<String, Value>,
    client_id: &str,
    issuer: Option<&str>,
    nonce: &str,
    now: i64,
) -> Result<(), OidcError> {
    let invalid = |reason: &str| OidcError::failed(format!("The ID token is not valid: {reason}"));
    let audiences: Vec<&str> = match claims.get("aud") {
        Some(Value::String(audience)) => vec![audience.as_str()],
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    if !audiences.contains(&client_id) {
        return Err(invalid("it was issued for another audience"));
    }
    if let Some(issuer) = issuer
        && claims.get("iss").and_then(Value::as_str) != Some(issuer)
    {
        return Err(invalid("it was issued by another issuer"));
    }
    match claims.get("exp").and_then(Value::as_i64) {
        Some(expires) if expires + CLOCK_SKEW >= now => {}
        Some(_) => return Err(invalid("it has expired")),
        None => return Err(invalid("it has no expiry")),
    }
    if claims
        .get("nbf")
        .and_then(Value::as_i64)
        .is_some_and(|not_before| not_before - CLOCK_SKEW > now)
    {
        return Err(invalid("it is not valid yet"));
    }
    if claims
        .get("iat")
        .and_then(Value::as_i64)
        .is_some_and(|issued| issued - CLOCK_SKEW > now)
    {
        return Err(invalid("it was issued in the future"));
    }
    if claims.get("nonce").and_then(Value::as_str) != Some(nonce) {
        return Err(invalid("the nonce does not match the sign-in request"));
    }
    if audiences.len() > 1
        && claims
            .get("azp")
            .and_then(Value::as_str)
            .is_some_and(|party| party != client_id)
    {
        return Err(OidcError::failed(
            "The ID token was issued to another client",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn claims(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn id_token_claims_are_checked() {
        let good = json!({"aud": "astrum", "iss": "https://sso", "exp": 1000, "nonce": "n"});
        check_claims(
            &claims(good.clone()),
            "astrum",
            Some("https://sso"),
            "n",
            1000,
        )
        .unwrap();
        check_claims(&claims(good.clone()), "astrum", None, "n", 1059).unwrap();
        for (claims_value, issuer, now) in [
            (
                json!({"aud": "other", "exp": 1000, "nonce": "n"}),
                None,
                900,
            ),
            (
                json!({"aud": ["x", "astrum"], "azp": "x", "exp": 1000, "nonce": "n"}),
                None,
                900,
            ),
            (good.clone(), Some("https://other"), 900),
            (good.clone(), None, 1061),
            (json!({"aud": "astrum", "nonce": "n"}), None, 900),
            (
                json!({"aud": "astrum", "exp": 2000, "nbf": 1100, "nonce": "n"}),
                None,
                900,
            ),
            (
                json!({"aud": "astrum", "exp": 2000, "nonce": "m"}),
                None,
                900,
            ),
        ] {
            assert!(
                check_claims(&claims(claims_value.clone()), "astrum", issuer, "n", now).is_err(),
                "{claims_value}"
            );
        }
        check_claims(
            &claims(json!({"aud": ["astrum", "x"], "azp": "astrum", "exp": 1000, "nonce": "n"})),
            "astrum",
            None,
            "n",
            900,
        )
        .unwrap();
    }

    #[test]
    fn error_bodies_are_summarised() {
        assert_eq!(
            error_details(br#"{"error":"invalid_grant","error_description":"bad code"}"#),
            " (invalid_grant: bad code)"
        );
        assert_eq!(error_details(b"<html>"), "");
        assert_eq!(
            strip_trailing_slash(" https://sso/realms/x// "),
            "https://sso/realms/x"
        );
    }
}
