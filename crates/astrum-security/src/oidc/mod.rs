pub mod claims;
pub mod config;
pub mod jwt;
pub mod provider;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub use config::{Draft, OidcSettings, SETTINGS_KEY, SecretChange, View};
pub use provider::{Http, Metadata, OidcProvider};

pub const SOURCE: &str = "OAuth2";
pub const LOGIN_PATH: &str = "/oidc/login";
pub const CALLBACK_PATH: &str = "/oidc/callback";
pub const LOGIN_TTL_SECONDS: i64 = 600;
const DEFAULT_SCOPE: &str = "openid profile email";
const MAX_RETURN: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Expired,
    Provider,
    Denied,
    Failed,
}

impl Failure {
    pub fn code(self) -> &'static str {
        match self {
            Failure::Expired => "expired",
            Failure::Provider => "provider",
            Failure::Denied => "denied",
            Failure::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct OidcError {
    pub kind: Failure,
    pub message: String,
}

impl OidcError {
    pub fn new(kind: Failure, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(Failure::Failed, message)
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Self::new(Failure::Denied, message)
    }

    pub fn expired(message: impl Into<String>) -> Self {
        Self::new(Failure::Expired, message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OidcUser {
    pub user_id: String,
    pub subject: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub groups: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingLogin {
    pub state: String,
    pub nonce: String,
    pub verifier: String,
    pub redirect_uri: String,
    pub return_to: Option<String>,
    pub expires: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Callback {
    pub state: Option<String>,
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn code_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub fn return_path(encoded: Option<&str>) -> Option<String> {
    let raw = encoded.map(str::trim).filter(|value| !value.is_empty())?;
    let decoded = STANDARD
        .decode(raw)
        .or_else(|_| URL_SAFE_NO_PAD.decode(raw.trim_end_matches('=')))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .filter(|text| text.starts_with('/') || text.starts_with('#'));
    let target = decoded.unwrap_or_else(|| raw.to_owned());
    let target = match target.strip_prefix('#') {
        Some(hash) => format!("/#{}", hash.trim_start_matches('/')),
        None => target,
    };
    let safe = target.starts_with('/')
        && !target.starts_with("//")
        && !target.starts_with("/\\")
        && target.len() <= MAX_RETURN
        && !target.chars().any(char::is_control)
        && !target.starts_with("/oidc/")
        && target != "/login"
        && !target.starts_with("/login?");
    safe.then_some(target)
}

pub fn begin(
    settings: &OidcSettings,
    metadata: &Metadata,
    redirect_uri: &str,
    return_to: Option<String>,
    now: i64,
) -> Result<(PendingLogin, String), OidcError> {
    let pending = PendingLogin {
        state: random_token(),
        nonce: random_token(),
        verifier: format!("{}{}", random_token(), random_token()),
        redirect_uri: redirect_uri.to_owned(),
        return_to,
        expires: now + LOGIN_TTL_SECONDS,
    };
    let mut parameters: Vec<(String, String)> = vec![
        ("response_type".to_owned(), "code".to_owned()),
        ("client_id".to_owned(), settings.client_id.clone()),
        ("redirect_uri".to_owned(), redirect_uri.to_owned()),
        ("scope".to_owned(), DEFAULT_SCOPE.to_owned()),
        ("state".to_owned(), pending.state.clone()),
        ("nonce".to_owned(), pending.nonce.clone()),
        (
            "code_challenge".to_owned(),
            code_challenge(&pending.verifier),
        ),
        ("code_challenge_method".to_owned(), "S256".to_owned()),
    ];
    for (name, value) in &settings.authorization_custom_params {
        match parameters.iter_mut().find(|(known, _)| known == name) {
            Some(existing) => existing.1 = value.clone(),
            None => parameters.push((name.clone(), value.clone())),
        }
    }
    let mut url = url::Url::parse(&metadata.authorization_endpoint).map_err(|_| {
        OidcError::failed("The authorization endpoint of the provider is not a valid URL")
    })?;
    {
        let mut query = url.query_pairs_mut();
        for (name, value) in &parameters {
            query.append_pair(name, value);
        }
    }
    Ok((pending, url.to_string()))
}

pub fn check_callback(
    pending: Option<&PendingLogin>,
    callback: &Callback,
    now: i64,
) -> Result<(PendingLogin, String), OidcError> {
    let Some(pending) = pending else {
        return Err(OidcError::expired(
            "The sign-in request expired or was started in another browser, please try again",
        ));
    };
    let state = callback.state.as_deref().unwrap_or_default();
    let same = !state.is_empty()
        && bool::from(subtle::ConstantTimeEq::ct_eq(
            state.as_bytes(),
            pending.state.as_bytes(),
        ));
    if !same {
        return Err(OidcError::expired(
            "The sign-in request expired or was started in another browser, please try again",
        ));
    }
    if pending.expires < now {
        return Err(OidcError::expired(
            "The sign-in request expired, please try again",
        ));
    }
    if let Some(error) = callback.error.as_deref().filter(|error| !error.is_empty()) {
        let description = callback
            .error_description
            .as_deref()
            .map(|description| format!(": {description}"))
            .unwrap_or_default();
        return Err(OidcError::new(
            Failure::Provider,
            format!("The identity provider returned {error}{description}"),
        ));
    }
    let code = callback
        .code
        .clone()
        .filter(|code| !code.is_empty())
        .ok_or_else(|| {
            OidcError::new(
                Failure::Provider,
                "The identity provider did not return an authorization code",
            )
        })?;
    Ok((pending.clone(), code))
}

pub fn user_from_claims(
    settings: &OidcSettings,
    claims: &Map<String, Value>,
) -> Result<OidcUser, OidcError> {
    claims::check_exact_match(claims, &settings.exact_match_claims)?;
    let user_id = claims::user_id(claims, settings.username_claim.as_deref())?;
    let groups = match settings.groups_claim.as_deref() {
        Some(claim) => claims::strings(claims, claim),
        None => Vec::new(),
    };
    let first_name = claims::claim_or_default(
        claims,
        settings.first_name_claim.as_deref(),
        claims::DEFAULT_FIRST_NAME,
    )
    .or_else(|| claims::string(claims, "name"));
    let last_name = claims::claim_or_default(
        claims,
        settings.last_name_claim.as_deref(),
        claims::DEFAULT_LAST_NAME,
    );
    let email = claims::claim_or_default(
        claims,
        settings.email_claim.as_deref(),
        claims::DEFAULT_EMAIL,
    );
    let mut unique: Vec<String> = Vec::new();
    for group in groups {
        if !unique.contains(&group) {
            unique.push(group);
        }
    }
    Ok(OidcUser {
        user_id,
        subject: claims::string(claims, "sub").unwrap_or_default(),
        first_name: first_name.unwrap_or_default(),
        last_name: last_name.unwrap_or_default(),
        email: email.unwrap_or_default(),
        groups: unique,
    })
}

pub fn merge_user_info(claims: &mut Map<String, Value>, user_info: Map<String, Value>) {
    if user_info.get("sub").is_some() && user_info.get("sub") == claims.get("sub") {
        for (name, value) in user_info {
            claims.entry(name).or_insert(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn settings() -> OidcSettings {
        OidcSettings {
            client_id: "astrum".to_owned(),
            groups_claim: Some("groups".to_owned()),
            ..OidcSettings::default()
        }
    }

    #[test]
    fn the_authorization_request_carries_pkce_and_custom_parameters() {
        let mut settings = settings();
        settings
            .authorization_custom_params
            .insert("scope".to_owned(), "openid groups".to_owned());
        settings
            .authorization_custom_params
            .insert("prompt".to_owned(), "login".to_owned());
        let metadata = Metadata {
            authorization_endpoint: "https://sso/auth?tenant=1".to_owned(),
            ..Metadata::default()
        };
        let (pending, location) = begin(
            &settings,
            &metadata,
            "https://astrum/oidc/callback",
            Some("/browse".to_owned()),
            100,
        )
        .unwrap();
        let url = url::Url::parse(&location).unwrap();
        let query: std::collections::BTreeMap<String, String> =
            url.query_pairs().into_owned().collect();
        assert_eq!(query["tenant"], "1");
        assert_eq!(query["response_type"], "code");
        assert_eq!(query["scope"], "openid groups");
        assert_eq!(query["prompt"], "login");
        assert_eq!(query["state"], pending.state);
        assert_eq!(query["nonce"], pending.nonce);
        assert_eq!(query["code_challenge"], code_challenge(&pending.verifier));
        assert_eq!(query["redirect_uri"], "https://astrum/oidc/callback");
        assert_eq!(pending.expires, 100 + LOGIN_TTL_SECONDS);
        assert!(pending.verifier.len() >= 43);
    }

    #[test]
    fn callbacks_must_match_their_request() {
        let pending = PendingLogin {
            state: "s".to_owned(),
            nonce: "n".to_owned(),
            verifier: "v".to_owned(),
            redirect_uri: "r".to_owned(),
            return_to: None,
            expires: 100,
        };
        let ok = Callback {
            state: Some("s".to_owned()),
            code: Some("c".to_owned()),
            ..Callback::default()
        };
        assert_eq!(check_callback(Some(&pending), &ok, 50).unwrap().1, "c");
        assert_eq!(
            check_callback(None, &ok, 50).unwrap_err().kind,
            Failure::Expired
        );
        assert_eq!(
            check_callback(Some(&pending), &ok, 101).unwrap_err().kind,
            Failure::Expired
        );
        let forged = Callback {
            state: Some("x".to_owned()),
            ..ok.clone()
        };
        assert_eq!(
            check_callback(Some(&pending), &forged, 50)
                .unwrap_err()
                .kind,
            Failure::Expired
        );
        let refused = Callback {
            error: Some("access_denied".to_owned()),
            error_description: Some("no".to_owned()),
            ..ok
        };
        let error = check_callback(Some(&pending), &refused, 50).unwrap_err();
        assert_eq!(error.kind, Failure::Provider);
        assert_eq!(
            error.message,
            "The identity provider returned access_denied: no"
        );
    }

    #[test]
    fn return_paths_stay_on_this_server() {
        let encode = |text: &str| STANDARD.encode(text);
        assert_eq!(
            return_path(Some(&encode("/browse/maven"))).as_deref(),
            Some("/browse/maven")
        );
        assert_eq!(
            return_path(Some(&encode("#browse/welcome"))).as_deref(),
            Some("/#browse/welcome")
        );
        assert_eq!(
            return_path(Some("/admin/security/ldap")).as_deref(),
            Some("/admin/security/ldap")
        );
        for unsafe_target in [
            "//evil.example",
            "https://evil.example",
            "/\\evil",
            "/oidc/login",
            "/login",
        ] {
            assert_eq!(
                return_path(Some(&encode(unsafe_target))),
                None,
                "{unsafe_target}"
            );
            assert_eq!(return_path(Some(unsafe_target)), None, "{unsafe_target}");
        }
        assert_eq!(return_path(None), None);
        assert_eq!(
            return_path(Some(&encode(&format!("/{}", "a".repeat(3000))))),
            None
        );
    }

    #[test]
    fn users_come_from_claims_and_user_info() {
        let mut claims = json!({
            "sub": "u-1", "preferred_username": "rita", "name": "Rita",
            "groups": ["devs", "devs", "ops"]
        })
        .as_object()
        .unwrap()
        .clone();
        merge_user_info(
            &mut claims,
            json!({"sub": "u-1", "email": "rita@example.org", "name": "Ignored"})
                .as_object()
                .unwrap()
                .clone(),
        );
        merge_user_info(
            &mut claims,
            json!({"sub": "other", "family_name": "Wrong"})
                .as_object()
                .unwrap()
                .clone(),
        );
        let user = user_from_claims(&settings(), &claims).unwrap();
        assert_eq!(user.user_id, "rita");
        assert_eq!(user.subject, "u-1");
        assert_eq!(user.first_name, "Rita");
        assert_eq!(user.last_name, "");
        assert_eq!(user.email, "rita@example.org");
        assert_eq!(user.groups, ["devs", "ops"]);
        let mut strict = settings();
        strict
            .exact_match_claims
            .insert("hd".to_owned(), "example.com".to_owned());
        assert_eq!(
            user_from_claims(&strict, &claims).unwrap_err().kind,
            Failure::Denied
        );
    }
}
