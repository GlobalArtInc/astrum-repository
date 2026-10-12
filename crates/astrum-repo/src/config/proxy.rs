use serde::{Deserialize, Serialize};

use crate::config::problems::{Problems, blank_to_none, web_url};
use crate::config::{Check, Context};

pub const MAX_RETRIES: i32 = 10;
pub const MAX_TIMEOUT: i32 = 3600;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub remote_url: Option<String>,
    pub content_max_age: Option<i32>,
    pub metadata_max_age: Option<i32>,
    pub preserve_encoded_characters: Option<bool>,
}

impl Check for Proxy {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        match self.remote_url.as_deref().map(str::trim) {
            None | Some("") => problems.add("remoteUrl", "must not be empty"),
            Some(url) if !web_url(url) => {
                problems.add("remoteUrl", "must be an http or https URL with a host")
            }
            Some(url) => self.remote_url = Some(url.to_owned()),
        }
        problems.require("contentMaxAge", &self.content_max_age);
        problems.require("metadataMaxAge", &self.metadata_max_age);
        self.preserve_encoded_characters.get_or_insert(false);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NegativeCache {
    pub enabled: Option<bool>,
    pub time_to_live: Option<i32>,
}

impl Check for NegativeCache {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        problems.require("enabled", &self.enabled);
        problems.require("timeToLive", &self.time_to_live);
        problems.range("timeToLive", self.time_to_live, 0, i32::MAX);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpClient {
    pub blocked: Option<bool>,
    pub auto_block: Option<bool>,
    pub connection: Option<Connection>,
    pub authentication: Option<Authentication>,
}

impl Check for HttpClient {
    fn check(&mut self, context: &Context<'_>, stored: Option<&Self>, problems: &mut Problems) {
        self.blocked.get_or_insert(false);
        self.auto_block.get_or_insert(true);
        self.connection
            .get_or_insert_with(Connection::default)
            .check(context, None, problems);
        if let Some(authentication) = &mut self.authentication {
            let stored = stored.and_then(|client| client.authentication.as_ref());
            authentication.check(context, stored, problems);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub retries: Option<i32>,
    pub user_agent_suffix: Option<String>,
    pub timeout: Option<i32>,
    pub enable_circular_redirects: Option<bool>,
    pub enable_cookies: Option<bool>,
    pub use_trust_store: Option<bool>,
}

impl Check for Connection {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        problems.range("retries", self.retries, 0, MAX_RETRIES);
        problems.range("timeout", self.timeout, 1, MAX_TIMEOUT);
        blank_to_none(&mut self.user_agent_suffix);
        self.enable_circular_redirects.get_or_insert(false);
        self.enable_cookies.get_or_insert(false);
        self.use_trust_store.get_or_insert(false);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthenticationType {
    Username,
    Ntlm,
    BearerToken,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Authentication {
    #[serde(rename = "type")]
    pub kind: Option<AuthenticationType>,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub ntlm_host: Option<String>,
    pub ntlm_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preemptive: Option<bool>,
}

impl Authentication {
    pub fn redact(&mut self) {
        self.password = None;
        self.bearer_token = None;
    }
}

impl Check for Authentication {
    fn check(&mut self, context: &Context<'_>, stored: Option<&Self>, problems: &mut Problems) {
        let Some(kind) = self.kind else {
            problems.add("type", "must not be empty");
            return;
        };
        self.password = context.secret(
            self.password.take(),
            stored.and_then(|old| old.password.as_ref()),
        );
        self.bearer_token = context.secret(
            self.bearer_token.take(),
            stored.and_then(|old| old.bearer_token.as_ref()),
        );
        blank_to_none(&mut self.ntlm_host);
        blank_to_none(&mut self.ntlm_domain);
        match kind {
            AuthenticationType::Username | AuthenticationType::Ntlm => {
                if self
                    .username
                    .as_deref()
                    .is_none_or(|name| name.trim().is_empty())
                {
                    problems.add("username", "must not be blank");
                }
                problems.require("password", &self.password);
                self.bearer_token = None;
                if kind == AuthenticationType::Username {
                    self.ntlm_host = None;
                    self.ntlm_domain = None;
                }
            }
            AuthenticationType::BearerToken => {
                problems.require("bearerToken", &self.bearer_token);
                self.username = None;
                self.password = None;
                self.ntlm_host = None;
                self.ntlm_domain = None;
            }
        }
        if !context.spec.has_preemptive_authentication() {
            self.preemptive = None;
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replication {
    pub preemptive_pull_enabled: Option<bool>,
    pub asset_path_regex: Option<String>,
}

impl Check for Replication {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.preemptive_pull_enabled.get_or_insert(false);
        blank_to_none(&mut self.asset_path_regex);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FirewallMode {
    Disabled,
    Audit,
    Quarantine,
    Pccs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Firewall {
    pub mode: Option<FirewallMode>,
}

impl Check for Firewall {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {}
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::Spec;

    fn seal(plain: &str) -> String {
        format!("sealed:{plain}")
    }

    fn check<T: Check + serde::de::DeserializeOwned>(
        format: &str,
        value: serde_json::Value,
        stored: Option<&T>,
    ) -> (T, Vec<String>) {
        let mut section: T = serde_json::from_value(value).unwrap();
        let context = Context::new(Spec::from_api(format, "proxy").unwrap(), &seal);
        let mut problems = Problems::default();
        section.check(&context, stored, &mut problems);
        let messages = problems
            .finish()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|problem| format!("{}: {}", problem.id, problem.message))
            .collect();
        (section, messages)
    }

    #[test]
    fn a_proxy_needs_a_web_remote_and_ages() {
        let (_, messages) = check::<Proxy>("raw", json!({}), None);
        assert_eq!(
            messages,
            [
                "PARAMETER remoteUrl: must not be empty",
                "PARAMETER contentMaxAge: must not be null",
                "PARAMETER metadataMaxAge: must not be null"
            ]
        );
        let (_, messages) = check::<Proxy>(
            "raw",
            json!({"remoteUrl": "ftp://example.org", "contentMaxAge": -1, "metadataMaxAge": 1440}),
            None,
        );
        assert_eq!(
            messages,
            ["PARAMETER remoteUrl: must be an http or https URL with a host"]
        );
        let (proxy, messages) = check::<Proxy>(
            "raw",
            json!({"remoteUrl": " https://repo1.maven.org/maven2/ ", "contentMaxAge": -1, "metadataMaxAge": 1440}),
            None,
        );
        assert!(messages.is_empty());
        assert_eq!(
            proxy.remote_url.as_deref(),
            Some("https://repo1.maven.org/maven2/")
        );
        assert_eq!(proxy.preserve_encoded_characters, Some(false));
    }

    #[test]
    fn the_negative_cache_lives_zero_minutes_or_more() {
        let (_, messages) =
            check::<NegativeCache>("raw", json!({"enabled": true, "timeToLive": -3}), None);
        assert_eq!(
            messages,
            ["PARAMETER timeToLive: must be greater than or equal to 0"]
        );
        let (_, messages) = check::<NegativeCache>("raw", json!({}), None);
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn the_http_client_fills_its_defaults_and_bounds_the_connection() {
        let (client, messages) = check::<HttpClient>("raw", json!({}), None);
        assert!(messages.is_empty());
        assert_eq!(client.blocked, Some(false));
        assert_eq!(client.auto_block, Some(true));
        assert_eq!(
            serde_json::to_value(client.connection.unwrap()).unwrap(),
            json!({
                "retries": null, "userAgentSuffix": null, "timeout": null,
                "enableCircularRedirects": false, "enableCookies": false, "useTrustStore": false
            })
        );
        let (_, messages) = check::<HttpClient>(
            "raw",
            json!({"connection": {"retries": 11, "timeout": 0}}),
            None,
        );
        assert_eq!(
            messages,
            [
                "PARAMETER retries: must be less than or equal to 10",
                "PARAMETER timeout: must be greater than or equal to 1"
            ]
        );
    }

    #[test]
    fn authentication_needs_the_secrets_of_its_type() {
        let (_, messages) =
            check::<Authentication>("raw", json!({"type": "username", "username": "u"}), None);
        assert_eq!(messages, ["PARAMETER password: must not be null"]);
        let (_, messages) = check::<Authentication>("raw", json!({"type": "bearerToken"}), None);
        assert_eq!(messages, ["PARAMETER bearerToken: must not be null"]);
        let (_, messages) =
            check::<Authentication>("raw", json!({"username": "u", "password": "p"}), None);
        assert_eq!(messages, ["PARAMETER type: must not be empty"]);
        let (_, messages) =
            check::<Authentication>("raw", json!({"type": "ntlm", "password": "p"}), None);
        assert_eq!(messages, ["PARAMETER username: must not be blank"]);
        assert!(
            serde_json::from_value::<Authentication>(json!({"type": "USERNAME"})).is_err(),
            "types are spelled exactly"
        );
    }

    #[test]
    fn secrets_are_sealed_and_kept_when_left_out() {
        let (first, messages) = check::<Authentication>(
            "raw",
            json!({"type": "username", "username": "u", "password": "p", "ntlmHost": "h", "bearerToken": "t"}),
            None,
        );
        assert!(messages.is_empty());
        assert_eq!(first.password.as_deref(), Some("sealed:p"));
        assert_eq!(first.bearer_token, None);
        assert_eq!(first.ntlm_host, None);
        for left_out in [
            json!({"type": "username", "username": "u"}),
            json!({"type": "username", "username": "u", "password": ""}),
            json!({"type": "username", "username": "u", "password": null}),
        ] {
            let (kept, messages) = check::<Authentication>("raw", left_out, Some(&first));
            assert!(messages.is_empty());
            assert_eq!(kept.password.as_deref(), Some("sealed:p"));
        }
        let (changed, _) = check::<Authentication>(
            "raw",
            json!({"type": "username", "username": "u", "password": "q"}),
            Some(&first),
        );
        assert_eq!(changed.password.as_deref(), Some("sealed:q"));
        let (bearer, messages) = check::<Authentication>(
            "raw",
            json!({"type": "bearerToken", "bearerToken": "t", "username": "u"}),
            Some(&first),
        );
        assert!(messages.is_empty());
        assert_eq!(bearer.bearer_token.as_deref(), Some("sealed:t"));
        assert_eq!((bearer.username, bearer.password), (None, None));
    }

    #[test]
    fn only_maven_proxies_authenticate_preemptively() {
        let given =
            json!({"type": "username", "username": "u", "password": "p", "preemptive": true});
        let (maven, _) = check::<Authentication>("maven", given.clone(), None);
        assert_eq!(maven.preemptive, Some(true));
        let (raw, _) = check::<Authentication>("raw", given, None);
        assert_eq!(raw.preemptive, None);
    }

    #[test]
    fn redaction_drops_every_secret() {
        let (mut authentication, _) = check::<Authentication>(
            "raw",
            json!({"type": "ntlm", "username": "u", "password": "p", "ntlmHost": "h", "ntlmDomain": "d"}),
            None,
        );
        authentication.redact();
        assert_eq!(
            serde_json::to_value(&authentication).unwrap(),
            json!({"type": "ntlm", "username": "u", "ntlmHost": "h", "ntlmDomain": "d"})
        );
    }
}
