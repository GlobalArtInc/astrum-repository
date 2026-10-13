use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub const HTTP_KEY: &str = "http.client";
pub const PLACEHOLDER: &str = "#~ASTRUM~PLACEHOLDER~PASSWORD~#";
pub const MAX_TIMEOUT: i32 = 3600;
pub const MAX_RETRIES: i32 = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpSettings {
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub timeout: Option<i32>,
    #[serde(default)]
    pub retries: Option<i32>,
    #[serde(default, deserialize_with = "null_default")]
    pub http_proxy: ProxySettings,
    #[serde(default, deserialize_with = "null_default")]
    pub https_proxy: ProxySettings,
    #[serde(default, deserialize_with = "null_default")]
    pub non_proxy_hosts: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxySettings {
    #[serde(default, deserialize_with = "null_default")]
    pub enabled: bool,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default, deserialize_with = "port_text")]
    pub port: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    pub auth_info: AuthInfo,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    #[serde(default, deserialize_with = "null_default")]
    pub enabled: bool,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub ntlm_host: Option<String>,
    #[serde(default)]
    pub ntlm_domain: Option<String>,
}

fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn port_text<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => Some(text),
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(other) => Some(other.to_string()),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub id: String,
    pub message: String,
}

fn blank_to_none(value: &mut Option<String>) {
    if let Some(text) = value {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            *value = None;
        } else if trimmed.len() != text.len() {
            *value = Some(trimmed.to_owned());
        }
    }
}

pub fn kept_secret(given: Option<String>) -> Option<String> {
    given.filter(|value| !value.is_empty() && value != PLACEHOLDER)
}

pub fn mask(stored: &Option<String>) -> Option<String> {
    stored.as_ref().map(|_| PLACEHOLDER.to_owned())
}

fn range(problems: &mut Vec<Problem>, id: &str, value: Option<i32>, low: i32, high: i32) {
    let Some(value) = value else {
        return;
    };
    let message = if value < low {
        format!("must be greater than or equal to {low}")
    } else if value > high {
        format!("must be less than or equal to {high}")
    } else {
        return;
    };
    problems.push(Problem {
        id: format!("PARAMETER {id}"),
        message,
    });
}

impl HttpSettings {
    pub fn prepare(
        mut self,
        stored: &HttpSettings,
        seal: &dyn Fn(&str) -> String,
    ) -> Result<Self, Vec<Problem>> {
        let mut problems = Vec::new();
        blank_to_none(&mut self.user_agent);
        range(&mut problems, "timeout", self.timeout, 1, MAX_TIMEOUT);
        range(&mut problems, "retries", self.retries, 0, MAX_RETRIES);
        self.http_proxy
            .prepare("httpProxy", &stored.http_proxy, seal, &mut problems);
        self.https_proxy
            .prepare("httpsProxy", &stored.https_proxy, seal, &mut problems);
        let mut hosts = Vec::new();
        for host in self.non_proxy_hosts.drain(..) {
            let host = host.trim().to_owned();
            if host.is_empty() || hosts.contains(&host) {
                continue;
            }
            if host
                .chars()
                .any(|c| c.is_whitespace() || c == '/' || c == ',')
            {
                problems.push(Problem {
                    id: "nonProxyHosts".to_owned(),
                    message: format!("\"{host}\" is not a host name or pattern"),
                });
            }
            hosts.push(host);
        }
        self.non_proxy_hosts = hosts;
        match problems.is_empty() {
            true => Ok(self),
            false => Err(problems),
        }
    }

    pub fn masked(&self) -> Self {
        let mut masked = self.clone();
        for proxy in [&mut masked.http_proxy, &mut masked.https_proxy] {
            proxy.auth_info.password = mask(&proxy.auth_info.password);
        }
        masked
    }

    pub fn document(&self) -> Value {
        serde_json::to_value(self.masked()).unwrap_or(Value::Null)
    }
}

impl ProxySettings {
    fn prepare(
        &mut self,
        prefix: &str,
        stored: &ProxySettings,
        seal: &dyn Fn(&str) -> String,
        problems: &mut Vec<Problem>,
    ) {
        if !self.enabled {
            *self = ProxySettings::default();
            return;
        }
        blank_to_none(&mut self.host);
        blank_to_none(&mut self.port);
        let mut add = |field: &str, message: &str| {
            problems.push(Problem {
                id: format!("{prefix}.{field}"),
                message: message.to_owned(),
            })
        };
        match &self.host {
            None => add("host", "Host is required"),
            Some(host) if host.contains(['/', ' ', ':', '@']) => {
                add("host", "Host must be a host name or an address")
            }
            Some(_) => {}
        }
        match self.port.as_deref().map(str::parse::<u32>) {
            None => add("port", "Port is required"),
            Some(Ok(port)) if (1..=65535).contains(&port) => {
                self.port = Some(port.to_string());
            }
            Some(_) => add("port", "Port must be a number from 1 to 65535"),
        }
        let auth = &mut self.auth_info;
        if !auth.enabled {
            *auth = AuthInfo::default();
            return;
        }
        blank_to_none(&mut auth.username);
        blank_to_none(&mut auth.ntlm_host);
        blank_to_none(&mut auth.ntlm_domain);
        if auth.username.is_none() {
            add("authInfo.username", "Username is required");
        }
        auth.password = match kept_secret(auth.password.take()) {
            Some(plain) => Some(seal(&plain)),
            None => stored
                .auth_info
                .password
                .clone()
                .filter(|_| stored.auth_info.enabled),
        };
        if auth.password.is_none() {
            add("authInfo.password", "Password is required");
        }
    }

    pub fn url(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let host = self.host.as_deref()?;
        let port = self.port.as_deref()?;
        let host = match host.parse::<std::net::Ipv6Addr>() {
            Ok(_) => format!("[{host}]"),
            Err(_) => host.to_owned(),
        };
        Some(format!("http://{host}:{port}"))
    }
}

pub fn bypasses(patterns: &[String], host: &str) -> bool {
    let host = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    patterns.iter().any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        if pattern == "*" {
            return true;
        }
        glob(&pattern, &host)
    })
}

fn glob(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !text.starts_with(first) || text.len() < first.len() + last.len() || !text.ends_with(last) {
        return false;
    }
    let mut rest = &text[first.len()..text.len() - last.len()];
    for middle in &parts[1..parts.len() - 1] {
        match rest.find(middle) {
            Some(at) => rest = &rest[at + middle.len()..],
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn seal(plain: &str) -> String {
        format!("sealed:{plain}")
    }

    fn parse(value: Value) -> HttpSettings {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn the_default_document_matches_the_reference() {
        assert_eq!(
            HttpSettings::default().document(),
            json!({
                "userAgent": null, "timeout": null, "retries": null,
                "httpProxy": {"enabled": false, "host": null, "port": null, "authInfo": {
                    "enabled": false, "username": null, "password": null, "ntlmHost": null, "ntlmDomain": null}},
                "httpsProxy": {"enabled": false, "host": null, "port": null, "authInfo": {
                    "enabled": false, "username": null, "password": null, "ntlmHost": null, "ntlmDomain": null}},
                "nonProxyHosts": []
            })
        );
        assert_eq!(parse(json!({"httpProxy": null})), HttpSettings::default());
    }

    #[test]
    fn bounds_and_proxy_fields_are_checked() {
        let problems = parse(json!({"timeout": 0, "retries": 11, "httpProxy": {"enabled": true, "host": "", "port": "x"},
            "httpsProxy": {"enabled": true, "host": "proxy", "port": 3128, "authInfo": {"enabled": true}}}))
        .prepare(&HttpSettings::default(), &seal)
        .unwrap_err();
        let ids: Vec<&str> = problems.iter().map(|problem| problem.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "PARAMETER timeout",
                "PARAMETER retries",
                "httpProxy.host",
                "httpProxy.port",
                "httpsProxy.authInfo.username",
                "httpsProxy.authInfo.password"
            ]
        );
    }

    #[test]
    fn passwords_are_sealed_masked_and_kept() {
        let first = parse(json!({"httpProxy": {"enabled": true, "host": "proxy.example", "port": 3128,
            "authInfo": {"enabled": true, "username": "u", "password": "p"}}, "nonProxyHosts": [" localhost ", "", "*.example.com"]}))
        .prepare(&HttpSettings::default(), &seal)
        .unwrap();
        assert_eq!(first.http_proxy.port.as_deref(), Some("3128"));
        assert_eq!(
            first.http_proxy.auth_info.password.as_deref(),
            Some("sealed:p")
        );
        assert_eq!(first.non_proxy_hosts, ["localhost", "*.example.com"]);
        let document = first.document();
        assert_eq!(
            document["httpProxy"]["authInfo"]["password"],
            json!(PLACEHOLDER)
        );
        for again in [json!(PLACEHOLDER), json!(""), Value::Null] {
            let mut sent = document.clone();
            sent["httpProxy"]["authInfo"]["password"] = again;
            let kept = parse(sent).prepare(&first, &seal).unwrap();
            assert_eq!(
                kept.http_proxy.auth_info.password.as_deref(),
                Some("sealed:p")
            );
        }
        let disabled = parse(json!({"httpProxy": {"enabled": false, "host": "proxy.example"}}))
            .prepare(&first, &seal)
            .unwrap();
        assert_eq!(disabled.http_proxy, ProxySettings::default());
        assert_eq!(
            first.http_proxy.url().as_deref(),
            Some("http://proxy.example:3128")
        );
    }

    #[test]
    fn non_proxy_hosts_match_like_globs() {
        let patterns = vec![
            "*.example.com".to_owned(),
            "localhost".to_owned(),
            "10.*".to_owned(),
        ];
        assert!(bypasses(&patterns, "repo.example.com"));
        assert!(bypasses(&patterns, "LOCALHOST"));
        assert!(bypasses(&patterns, "10.1.2.3"));
        assert!(!bypasses(&patterns, "example.com"));
        assert!(!bypasses(&patterns, "repo.example.org"));
        assert!(bypasses(&["*".to_owned()], "anything"));
    }
}
