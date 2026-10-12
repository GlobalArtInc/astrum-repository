use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::claims;
use super::jwt::SUPPORTED_ALGORITHMS;

pub const SETTINGS_KEY: &str = "security.oidc";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OidcSettings {
    #[serde(default)]
    pub issuer_url: Option<String>,
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub idp_authorization_url: Option<String>,
    #[serde(default)]
    pub idp_token_url: Option<String>,
    #[serde(default)]
    pub idp_jwks_url: Option<String>,
    #[serde(default)]
    pub idp_logout_url: Option<String>,
    #[serde(default)]
    pub idp_jws_algorithm: Option<String>,
    #[serde(default)]
    pub idp_jwks: Option<String>,
    #[serde(default)]
    pub username_claim: Option<String>,
    #[serde(default)]
    pub first_name_claim: Option<String>,
    #[serde(default)]
    pub last_name_claim: Option<String>,
    #[serde(default)]
    pub email_claim: Option<String>,
    #[serde(default)]
    pub groups_claim: Option<String>,
    #[serde(default)]
    pub exact_match_claims: BTreeMap<String, String>,
    #[serde(default)]
    pub authorization_custom_params: BTreeMap<String, String>,
    #[serde(default)]
    pub token_request_custom_params: BTreeMap<String, String>,
    #[serde(default)]
    pub use_trust_store: bool,
}

impl OidcSettings {
    pub fn fingerprint(&self) -> String {
        [
            self.issuer_url.as_deref().unwrap_or_default(),
            self.idp_authorization_url.as_deref().unwrap_or_default(),
            self.idp_token_url.as_deref().unwrap_or_default(),
            self.idp_jwks_url.as_deref().unwrap_or_default(),
            self.idp_logout_url.as_deref().unwrap_or_default(),
            self.idp_jwks.as_deref().unwrap_or_default(),
            if self.use_trust_store { "1" } else { "0" },
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub idp_authorization_url: Option<String>,
    pub idp_token_url: Option<String>,
    pub idp_jwks_url: Option<String>,
    pub idp_logout_url: Option<String>,
    pub idp_jws_algorithm: Option<String>,
    pub idp_jwks: Option<String>,
    pub username_claim: Option<String>,
    pub first_name_claim: Option<String>,
    pub last_name_claim: Option<String>,
    pub email_claim: Option<String>,
    pub groups_claim: Option<String>,
    pub exact_match_claims: Option<BTreeMap<String, Option<String>>>,
    pub authorization_custom_params: Option<BTreeMap<String, Option<String>>>,
    pub token_request_custom_params: Option<BTreeMap<String, Option<String>>>,
    pub use_trust_store: Option<bool>,
    pub redirect_uri: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub field: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub client_secret_set: bool,
    pub idp_authorization_url: Option<String>,
    pub idp_token_url: Option<String>,
    pub idp_jwks_url: Option<String>,
    pub idp_logout_url: Option<String>,
    pub idp_jws_algorithm: Option<String>,
    pub idp_jwks: Option<String>,
    pub username_claim: Option<String>,
    pub first_name_claim: Option<String>,
    pub last_name_claim: Option<String>,
    pub email_claim: Option<String>,
    pub groups_claim: Option<String>,
    pub exact_match_claims: BTreeMap<String, String>,
    pub authorization_custom_params: BTreeMap<String, String>,
    pub token_request_custom_params: BTreeMap<String, String>,
    pub use_trust_store: bool,
    pub redirect_uri: String,
    pub configured: bool,
}

impl View {
    pub fn of(settings: Option<&OidcSettings>, redirect_uri: String) -> Self {
        match settings {
            Some(settings) => Self {
                issuer_url: settings.issuer_url.clone(),
                client_id: Some(settings.client_id.clone()),
                client_secret: None,
                client_secret_set: settings.client_secret.is_some(),
                idp_authorization_url: settings.idp_authorization_url.clone(),
                idp_token_url: settings.idp_token_url.clone(),
                idp_jwks_url: settings.idp_jwks_url.clone(),
                idp_logout_url: settings.idp_logout_url.clone(),
                idp_jws_algorithm: settings.idp_jws_algorithm.clone(),
                idp_jwks: settings.idp_jwks.clone(),
                username_claim: settings.username_claim.clone(),
                first_name_claim: settings.first_name_claim.clone(),
                last_name_claim: settings.last_name_claim.clone(),
                email_claim: settings.email_claim.clone(),
                groups_claim: settings.groups_claim.clone(),
                exact_match_claims: settings.exact_match_claims.clone(),
                authorization_custom_params: settings.authorization_custom_params.clone(),
                token_request_custom_params: settings.token_request_custom_params.clone(),
                use_trust_store: settings.use_trust_store,
                redirect_uri,
                configured: true,
            },
            None => Self {
                issuer_url: None,
                client_id: None,
                client_secret: None,
                client_secret_set: false,
                idp_authorization_url: None,
                idp_token_url: None,
                idp_jwks_url: None,
                idp_logout_url: None,
                idp_jws_algorithm: None,
                idp_jwks: None,
                username_claim: Some(claims::DEFAULT_USERNAME.to_owned()),
                first_name_claim: Some(claims::DEFAULT_FIRST_NAME.to_owned()),
                last_name_claim: Some(claims::DEFAULT_LAST_NAME.to_owned()),
                email_claim: Some(claims::DEFAULT_EMAIL.to_owned()),
                groups_claim: None,
                exact_match_claims: BTreeMap::new(),
                authorization_custom_params: BTreeMap::new(),
                token_request_custom_params: BTreeMap::new(),
                use_trust_store: false,
                redirect_uri,
                configured: false,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecretChange {
    Keep,
    Remove,
    Set(String),
}

fn text(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn map(value: &Option<BTreeMap<String, Option<String>>>) -> BTreeMap<String, String> {
    value
        .iter()
        .flatten()
        .filter_map(|(key, value)| {
            let key = key.trim();
            match (key.is_empty(), value) {
                (false, Some(value)) => Some((key.to_owned(), value.clone())),
                _ => None,
            }
        })
        .collect()
}

pub fn check_url(value: &str) -> Result<(), &'static str> {
    match url::Url::parse(value) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => Ok(()),
        Ok(_) => Err("must be an http or https URL"),
        Err(_) => Err("is not a valid URL"),
    }
}

pub fn check(draft: &Draft) -> Result<(OidcSettings, SecretChange), Vec<Problem>> {
    let mut problems = Vec::new();
    let mut add = |field: &'static str, message: String| problems.push(Problem { field, message });
    let settings = OidcSettings {
        issuer_url: text(&draft.issuer_url),
        client_id: text(&draft.client_id).unwrap_or_default(),
        client_secret: None,
        idp_authorization_url: text(&draft.idp_authorization_url),
        idp_token_url: text(&draft.idp_token_url),
        idp_jwks_url: text(&draft.idp_jwks_url),
        idp_logout_url: text(&draft.idp_logout_url),
        idp_jws_algorithm: text(&draft.idp_jws_algorithm),
        idp_jwks: text(&draft.idp_jwks),
        username_claim: text(&draft.username_claim),
        first_name_claim: text(&draft.first_name_claim),
        last_name_claim: text(&draft.last_name_claim),
        email_claim: text(&draft.email_claim),
        groups_claim: text(&draft.groups_claim),
        exact_match_claims: map(&draft.exact_match_claims),
        authorization_custom_params: map(&draft.authorization_custom_params),
        token_request_custom_params: map(&draft.token_request_custom_params),
        use_trust_store: draft.use_trust_store.unwrap_or(false),
    };
    if settings.client_id.is_empty() {
        add("clientId", "Client ID is required".to_owned());
    }
    for (field, label, value) in [
        ("issuerUrl", "Issuer URL", &settings.issuer_url),
        (
            "idpAuthorizationUrl",
            "Authorization URL",
            &settings.idp_authorization_url,
        ),
        ("idpTokenUrl", "Token URL", &settings.idp_token_url),
        ("idpJwksUrl", "JSON Web Key URL", &settings.idp_jwks_url),
        ("idpLogoutUrl", "Logout URL", &settings.idp_logout_url),
    ] {
        if let Some(value) = value
            && let Err(problem) = check_url(value)
        {
            add(field, format!("{label} {problem}"));
        }
    }
    if settings.issuer_url.is_none()
        && (settings.idp_authorization_url.is_none()
            || settings.idp_token_url.is_none()
            || (settings.idp_jwks_url.is_none() && settings.idp_jwks.is_none()))
    {
        add(
            "issuerUrl",
            "Set the Issuer URL, or the Authorization URL, Token URL and JSON Web Keys".to_owned(),
        );
    }
    if let Some(jwks) = &settings.idp_jwks
        && serde_json::from_str::<serde_json::Value>(jwks).is_err()
    {
        add("idpJwks", "JSON Web Keys must be valid JSON".to_owned());
    }
    if let Some(algorithm) = &settings.idp_jws_algorithm
        && !SUPPORTED_ALGORITHMS.contains(&algorithm.as_str())
    {
        add(
            "idpJwsAlgorithm",
            format!(
                "JWT Signature Algorithm must be one of {}",
                SUPPORTED_ALGORITHMS.join(", ")
            ),
        );
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let secret = match draft.client_secret.as_deref() {
        None => SecretChange::Keep,
        Some("") => SecretChange::Remove,
        Some(value) => SecretChange::Set(value.to_owned()),
    };
    Ok((settings, secret))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_draft_needs_a_client_and_a_way_to_the_provider() {
        let problems = check(&Draft::default()).unwrap_err();
        let fields: Vec<&str> = problems.iter().map(|problem| problem.field).collect();
        assert_eq!(fields, ["clientId", "issuerUrl"]);
        let problems = check(&Draft {
            client_id: Some("astrum".to_owned()),
            issuer_url: Some("ftp://sso".to_owned()),
            idp_jwks: Some("{".to_owned()),
            idp_jws_algorithm: Some("HS256".to_owned()),
            ..Draft::default()
        })
        .unwrap_err();
        let fields: Vec<&str> = problems.iter().map(|problem| problem.field).collect();
        assert_eq!(fields, ["issuerUrl", "idpJwks", "idpJwsAlgorithm"]);
    }

    #[test]
    fn explicit_endpoints_replace_discovery() {
        let (settings, secret) = check(&Draft {
            client_id: Some(" astrum ".to_owned()),
            client_secret: Some(String::new()),
            idp_authorization_url: Some("https://sso/auth".to_owned()),
            idp_token_url: Some("https://sso/token".to_owned()),
            idp_jwks: Some(r#"{"keys":[]}"#.to_owned()),
            authorization_custom_params: Some(BTreeMap::from([
                ("prompt".to_owned(), Some("login".to_owned())),
                (" ".to_owned(), Some("dropped".to_owned())),
                ("scope".to_owned(), None),
            ])),
            ..Draft::default()
        })
        .unwrap();
        assert_eq!(settings.client_id, "astrum");
        assert_eq!(secret, SecretChange::Remove);
        assert_eq!(
            settings.authorization_custom_params,
            BTreeMap::from([("prompt".to_owned(), "login".to_owned())])
        );
    }

    #[test]
    fn secrets_are_kept_unless_changed() {
        let draft = Draft {
            client_id: Some("astrum".to_owned()),
            issuer_url: Some("https://sso.example.com".to_owned()),
            ..Draft::default()
        };
        assert_eq!(check(&draft).unwrap().1, SecretChange::Keep);
        let changed = Draft {
            client_secret: Some("s".to_owned()),
            ..draft
        };
        assert_eq!(
            check(&changed).unwrap().1,
            SecretChange::Set("s".to_owned())
        );
    }

    #[test]
    fn an_empty_configuration_shows_the_default_claims() {
        let view = View::of(None, "https://astrum/oidc/callback".to_owned());
        assert_eq!(view.username_claim.as_deref(), Some("preferred_username"));
        assert!(!view.configured);
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["redirectUri"], "https://astrum/oidc/callback");
        assert_eq!(json["clientSecret"], serde_json::Value::Null);
    }
}
