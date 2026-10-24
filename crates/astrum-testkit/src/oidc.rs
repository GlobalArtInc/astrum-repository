use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use hyper_util::service::TowerToHyperService;
use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, KeyPair};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;

use crate::tls::TestCertificates;

pub const KEY_ID: &str = "mock-key";

struct Grant {
    client_id: String,
    redirect_uri: String,
    nonce: String,
    challenge: Option<String>,
    claims: Map<String, Value>,
}

struct Inner {
    url: String,
    client_id: String,
    client_secret: Option<String>,
    key: EcdsaKeyPair,
    random: SystemRandom,
    claims: Mutex<Map<String, Value>>,
    user_info: Mutex<Map<String, Value>>,
    grants: Mutex<HashMap<String, Grant>>,
    tokens: Mutex<HashMap<String, Map<String, Value>>>,
    deny: Mutex<Option<String>>,
    token_requests: AtomicUsize,
    last_authorize: Mutex<HashMap<String, String>>,
}

pub struct MockProvider {
    pub url: String,
    inner: Arc<Inner>,
    task: JoinHandle<()>,
}

impl Drop for MockProvider {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl MockProvider {
    pub async fn start(client_id: &str, client_secret: Option<&str>) -> Self {
        Self::launch(client_id, client_secret, None).await
    }

    pub async fn start_tls(
        client_id: &str,
        client_secret: Option<&str>,
        certificates: &TestCertificates,
    ) -> Self {
        Self::launch(
            client_id,
            client_secret,
            Some(TlsAcceptor::from(certificates.server_config())),
        )
        .await
    }

    async fn launch(
        client_id: &str,
        client_secret: Option<&str>,
        tls: Option<TlsAcceptor>,
    ) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("the mock provider can listen");
        let url = format!(
            "{}://127.0.0.1:{}",
            if tls.is_some() { "https" } else { "http" },
            listener.local_addr().expect("a bound port").port()
        );
        let random = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &random)
            .expect("a test key can be generated");
        let key =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &random)
                .expect("the generated key loads");
        let inner = Arc::new(Inner {
            url: url.clone(),
            client_id: client_id.to_owned(),
            client_secret: client_secret.map(str::to_owned),
            key,
            random,
            claims: Mutex::new(Map::new()),
            user_info: Mutex::new(Map::new()),
            grants: Mutex::new(HashMap::new()),
            tokens: Mutex::new(HashMap::new()),
            deny: Mutex::new(None),
            token_requests: AtomicUsize::new(0),
            last_authorize: Mutex::new(HashMap::new()),
        });
        let app = Router::new()
            .route("/.well-known/openid-configuration", get(discovery))
            .route("/jwks", get(jwks))
            .route("/authorize", get(authorize))
            .route("/token", post(token))
            .route("/userinfo", get(user_info))
            .with_state(inner.clone());
        let task = match tls {
            None => tokio::spawn(async move {
                let _ = axum::serve(listener, app).await;
            }),
            Some(acceptor) => tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    let acceptor = acceptor.clone();
                    let app = app.clone();
                    tokio::spawn(async move {
                        let Ok(stream) = acceptor.accept(stream).await else {
                            return;
                        };
                        let _ = Builder::new(TokioExecutor::new())
                            .serve_connection(TokioIo::new(stream), TowerToHyperService::new(app))
                            .await;
                    });
                }
            }),
        };
        Self { url, inner, task }
    }

    pub fn sign_in_as(&self, claims: Value) {
        *self
            .inner
            .claims
            .lock()
            .expect("the mock lock is never poisoned") =
            claims.as_object().cloned().unwrap_or_default();
    }

    pub fn user_info(&self, claims: Value) {
        *self
            .inner
            .user_info
            .lock()
            .expect("the mock lock is never poisoned") =
            claims.as_object().cloned().unwrap_or_default();
    }

    pub fn deny(&self, error: Option<&str>) {
        *self
            .inner
            .deny
            .lock()
            .expect("the mock lock is never poisoned") = error.map(str::to_owned);
    }

    pub fn token_requests(&self) -> usize {
        self.inner.token_requests.load(Ordering::SeqCst)
    }

    pub fn last_authorize(&self) -> HashMap<String, String> {
        self.inner
            .last_authorize
            .lock()
            .expect("the mock lock is never poisoned")
            .clone()
    }

    pub fn jwks(&self) -> Value {
        jwks_value(&self.inner)
    }

    pub fn id_token(&self, claims: Value) -> String {
        sign(&self.inner, claims.as_object().cloned().unwrap_or_default())
    }
}

fn jwks_value(inner: &Inner) -> Value {
    let public = inner.key.public_key().as_ref();
    json!({"keys": [{
        "kty": "EC", "crv": "P-256", "use": "sig", "alg": "ES256", "kid": KEY_ID,
        "x": URL_SAFE_NO_PAD.encode(&public[1..33]),
        "y": URL_SAFE_NO_PAD.encode(&public[33..65]),
    }]})
}

fn sign(inner: &Inner, claims: Map<String, Value>) -> String {
    let header = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&json!({"alg": "ES256", "typ": "JWT", "kid": KEY_ID}))
            .expect("JSON serialises"),
    );
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("JSON serialises"));
    let input = format!("{header}.{payload}");
    let signature = inner
        .key
        .sign(&inner.random, input.as_bytes())
        .expect("the test key signs");
    format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.as_ref()))
}

async fn discovery(State(inner): State<Arc<Inner>>) -> axum::Json<Value> {
    let url = &inner.url;
    axum::Json(json!({
        "issuer": url,
        "authorization_endpoint": format!("{url}/authorize"),
        "token_endpoint": format!("{url}/token"),
        "userinfo_endpoint": format!("{url}/userinfo"),
        "jwks_uri": format!("{url}/jwks"),
        "end_session_endpoint": format!("{url}/logout"),
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
        "response_types_supported": ["code"],
        "id_token_signing_alg_values_supported": ["ES256"],
    }))
}

async fn jwks(State(inner): State<Arc<Inner>>) -> axum::Json<Value> {
    axum::Json(jwks_value(&inner))
}

fn random_code() -> String {
    let mut bytes = [0u8; 16];
    ring::rand::SecureRandom::fill(&SystemRandom::new(), &mut bytes).expect("random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

fn found(location: String) -> Response {
    (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
}

async fn authorize(
    State(inner): State<Arc<Inner>>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    *inner
        .last_authorize
        .lock()
        .expect("the mock lock is never poisoned") = query.clone();
    let Some(redirect_uri) = query.get("redirect_uri").cloned() else {
        return (StatusCode::BAD_REQUEST, "no redirect_uri").into_response();
    };
    let state = query.get("state").cloned().unwrap_or_default();
    let separator = if redirect_uri.contains('?') { '&' } else { '?' };
    let state_part = url_encode(&state);
    if let Some(error) = inner
        .deny
        .lock()
        .expect("the mock lock is never poisoned")
        .clone()
    {
        return found(format!(
            "{redirect_uri}{separator}error={}&error_description=refused&state={state_part}",
            url_encode(&error)
        ));
    }
    let code = random_code();
    let claims = inner
        .claims
        .lock()
        .expect("the mock lock is never poisoned")
        .clone();
    inner
        .grants
        .lock()
        .expect("the mock lock is never poisoned")
        .insert(
            code.clone(),
            Grant {
                client_id: query.get("client_id").cloned().unwrap_or_default(),
                redirect_uri: redirect_uri.clone(),
                nonce: query.get("nonce").cloned().unwrap_or_default(),
                challenge: query.get("code_challenge").cloned(),
                claims,
            },
        );
    found(format!(
        "{redirect_uri}{separator}code={}&state={state_part}",
        url_encode(&code)
    ))
}

fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn token_error(error: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        axum::Json(json!({"error": error, "error_description": format!("mock refused: {error}")})),
    )
        .into_response()
}

async fn token(State(inner): State<Arc<Inner>>, headers: HeaderMap, body: String) -> Response {
    inner.token_requests.fetch_add(1, Ordering::SeqCst);
    let form: HashMap<String, String> = url::form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();
    if form.get("grant_type").map(String::as_str) != Some("authorization_code") {
        return token_error("unsupported_grant_type");
    }
    let basic = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "))
        .and_then(|encoded| STANDARD.decode(encoded).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|pair| {
            pair.split_once(':').map(|(id, secret)| {
                (
                    url::form_urlencoded::parse(format!("a={id}").as_bytes())
                        .next()
                        .map(|(_, value)| value.into_owned())
                        .unwrap_or_default(),
                    url::form_urlencoded::parse(format!("a={secret}").as_bytes())
                        .next()
                        .map(|(_, value)| value.into_owned())
                        .unwrap_or_default(),
                )
            })
        });
    let (client_id, secret) = match basic {
        Some((id, secret)) => (id, Some(secret)),
        None => (
            form.get("client_id").cloned().unwrap_or_default(),
            form.get("client_secret").cloned(),
        ),
    };
    if client_id != inner.client_id || secret != inner.client_secret {
        return token_error("invalid_client");
    }
    let Some(grant) = form.get("code").and_then(|code| {
        inner
            .grants
            .lock()
            .expect("the mock lock is never poisoned")
            .remove(code)
    }) else {
        return token_error("invalid_grant");
    };
    if grant.client_id != client_id || Some(&grant.redirect_uri) != form.get("redirect_uri") {
        return token_error("invalid_grant");
    }
    if let Some(challenge) = &grant.challenge {
        let verifier = form.get("code_verifier").cloned().unwrap_or_default();
        if URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) != *challenge {
            return token_error("invalid_grant");
        }
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    let mut claims = Map::new();
    claims.insert("iss".to_owned(), json!(inner.url));
    claims.insert("aud".to_owned(), json!(inner.client_id));
    claims.insert("iat".to_owned(), json!(now));
    claims.insert("exp".to_owned(), json!(now + 300));
    claims.insert("nonce".to_owned(), json!(grant.nonce));
    for (name, value) in grant.claims {
        claims.insert(name, value);
    }
    let access_token = format!("at-{}", random_code());
    inner
        .tokens
        .lock()
        .expect("the mock lock is never poisoned")
        .insert(access_token.clone(), claims.clone());
    axum::Json(json!({
        "access_token": access_token,
        "token_type": "Bearer",
        "expires_in": 300,
        "id_token": sign(&inner, claims),
    }))
    .into_response()
}

async fn user_info(State(inner): State<Arc<Inner>>, headers: HeaderMap) -> Response {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    let Some(claims) = inner
        .tokens
        .lock()
        .expect("the mock lock is never poisoned")
        .get(token)
        .cloned()
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut info = Map::new();
    if let Some(subject) = claims.get("sub") {
        info.insert("sub".to_owned(), subject.clone());
    }
    for (name, value) in inner
        .user_info
        .lock()
        .expect("the mock lock is never poisoned")
        .iter()
    {
        info.insert(name.clone(), value.clone());
    }
    axum::Json(Value::Object(info)).into_response()
}
