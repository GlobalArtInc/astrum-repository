use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const EMAIL_KEY: &str = "email";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_PORT: u16 = 25;
pub const DEFAULT_FROM: &str = "astrum@example.org";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub from_address: String,
    pub subject_prefix: Option<String>,
    pub start_tls_enabled: bool,
    pub start_tls_required: bool,
    pub ssl_on_connect_enabled: bool,
    pub ssl_server_identity_check_enabled: bool,
    pub astrum_trust_store_enabled: bool,
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_PORT,
            username: String::new(),
            password: None,
            from_address: DEFAULT_FROM.to_owned(),
            subject_prefix: None,
            start_tls_enabled: false,
            start_tls_required: false,
            ssl_on_connect_enabled: false,
            ssl_server_identity_check_enabled: false,
            astrum_trust_store_enabled: false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailRequest {
    pub enabled: Option<bool>,
    pub host: Option<String>,
    pub port: Option<Value>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: Option<String>,
    pub subject_prefix: Option<String>,
    pub start_tls_enabled: Option<bool>,
    pub start_tls_required: Option<bool>,
    pub ssl_on_connect_enabled: Option<bool>,
    pub ssl_server_identity_check_enabled: Option<bool>,
    pub astrum_trust_store_enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub id: String,
    pub message: String,
}

pub fn valid_address(address: &str) -> bool {
    let Some((local, domain)) = address.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && !address
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '<' | '>' | ',' | ';'))
}

impl EmailRequest {
    pub fn prepare(
        self,
        stored: &EmailSettings,
        seal: &dyn Fn(&str) -> String,
    ) -> Result<EmailSettings, Vec<Problem>> {
        let mut problems = Vec::new();
        let mut add = |id: &str, message: &str| {
            problems.push(Problem {
                id: format!("PARAMETER {id}"),
                message: message.to_owned(),
            })
        };
        let host = self.host.unwrap_or_default().trim().to_owned();
        if host.is_empty() {
            add("host", "must not be blank");
        }
        let port = match &self.port {
            None | Some(Value::Null) => {
                add("port", "must not be null");
                0
            }
            Some(value) => match value
                .as_u64()
                .or_else(|| value.as_str().and_then(|text| text.trim().parse().ok()))
            {
                Some(port @ 1..=65535) => port as u16,
                _ => {
                    add("port", "must be a port number from 1 to 65535");
                    0
                }
            },
        };
        let from_address = self.from_address.unwrap_or_default().trim().to_owned();
        if from_address.is_empty() {
            add("fromAddress", "must not be blank");
        } else if !valid_address(&from_address) {
            add("fromAddress", "must be a well-formed email address");
        }
        let username = self.username.unwrap_or_default().trim().to_owned();
        let password = match astrum_outbound::settings::kept_secret(self.password) {
            Some(plain) => Some(seal(&plain)),
            None => stored.password.clone(),
        }
        .filter(|_| !username.is_empty());
        if !problems.is_empty() {
            return Err(problems);
        }
        let ssl_on_connect = self.ssl_on_connect_enabled.unwrap_or(false);
        let start_tls_required = self.start_tls_required.unwrap_or(false);
        Ok(EmailSettings {
            enabled: self.enabled.unwrap_or(false),
            host,
            port,
            username,
            password,
            from_address,
            subject_prefix: self
                .subject_prefix
                .map(|prefix| prefix.trim().to_owned())
                .filter(|prefix| !prefix.is_empty()),
            start_tls_enabled: self.start_tls_enabled.unwrap_or(false) || start_tls_required,
            start_tls_required,
            ssl_on_connect_enabled: ssl_on_connect,
            ssl_server_identity_check_enabled: self
                .ssl_server_identity_check_enabled
                .unwrap_or(false),
            astrum_trust_store_enabled: self.astrum_trust_store_enabled.unwrap_or(false),
        })
    }
}

impl EmailSettings {
    pub fn document(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "host": self.host,
            "port": self.port,
            "username": Some(&self.username).filter(|username| !username.is_empty()),
            "password": astrum_outbound::settings::mask(&self.password),
            "fromAddress": self.from_address,
            "subjectPrefix": self.subject_prefix,
            "startTlsEnabled": self.start_tls_enabled,
            "startTlsRequired": self.start_tls_required,
            "sslOnConnectEnabled": self.ssl_on_connect_enabled,
            "sslServerIdentityCheckEnabled": self.ssl_server_identity_check_enabled,
            "astrumTrustStoreEnabled": self.astrum_trust_store_enabled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seal(plain: &str) -> String {
        format!("sealed:{plain}")
    }

    fn request(value: Value) -> EmailRequest {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn the_default_document_matches_the_reference() {
        assert_eq!(
            EmailSettings::default().document(),
            json!({
                "enabled": false, "host": "localhost", "port": 25, "username": null, "password": null,
                "fromAddress": "astrum@example.org", "subjectPrefix": null, "startTlsEnabled": false,
                "startTlsRequired": false, "sslOnConnectEnabled": false,
                "sslServerIdentityCheckEnabled": false, "astrumTrustStoreEnabled": false
            })
        );
    }

    #[test]
    fn required_fields_are_checked() {
        let problems = request(json!({}))
            .prepare(&EmailSettings::default(), &seal)
            .unwrap_err();
        let ids: Vec<&str> = problems.iter().map(|problem| problem.id.as_str()).collect();
        assert_eq!(
            ids,
            ["PARAMETER host", "PARAMETER port", "PARAMETER fromAddress"]
        );
        let problems = request(json!({"host": "mail", "port": 70000, "fromAddress": "bad"}))
            .prepare(&EmailSettings::default(), &seal)
            .unwrap_err();
        assert_eq!(problems.len(), 2);
        assert_eq!(problems[1].message, "must be a well-formed email address");
    }

    #[test]
    fn passwords_are_sealed_kept_and_masked() {
        let first = request(
            json!({"enabled": true, "host": "mail.example", "port": "587",
            "username": "robot", "password": "s3cret", "fromAddress": "astrum@example.org",
            "startTlsRequired": true, "astrumTrustStoreEnabled": true}),
        )
        .prepare(&EmailSettings::default(), &seal)
        .unwrap();
        assert_eq!(first.port, 587);
        assert_eq!(first.password.as_deref(), Some("sealed:s3cret"));
        assert!(first.start_tls_enabled, "requiring STARTTLS enables it");
        assert!(first.astrum_trust_store_enabled);
        let document = first.document();
        assert_eq!(document["password"], astrum_outbound::PLACEHOLDER);
        let kept = request(document.clone()).prepare(&first, &seal).unwrap();
        assert_eq!(kept.password.as_deref(), Some("sealed:s3cret"));
        let mut anonymous = document;
        anonymous["username"] = json!("");
        let cleared = request(anonymous).prepare(&first, &seal).unwrap();
        assert_eq!(cleared.password, None);
    }

    #[test]
    fn addresses_are_plain_mailboxes() {
        for good in ["a@b", "first.last@example.org", "x+tag@mail.example"] {
            assert!(valid_address(good), "{good}");
        }
        for bad in [
            "",
            "a",
            "@b",
            "a@",
            "a b@c",
            "a@b@c",
            "<a@b>",
            "a@b.",
            "a@b\r\nRCPT",
        ] {
            assert!(!valid_address(bad), "{bad:?}");
        }
    }
}
