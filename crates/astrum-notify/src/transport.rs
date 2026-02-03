use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use astrum_security::ssrf::{Guard, GuardedResolver};
use serde_json::{Value, json};

use crate::channel::{Channel, ChannelType, Proxy, ProxyKind, Secrets, bot_token_ok, webhook_url};
use crate::mail::{Mail, MailSender};
use crate::messages;
use crate::payload::{self, DELIVERY_HEADER, EVENT_HEADER, Message, SIGNATURE_HEADER};

const MAX_ERROR_LENGTH: usize = 300;
const MIN_HIDDEN_LENGTH: usize = 4;
const MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct DeliveryError {
    pub message: String,
    pub status: u16,
    pub retryable: bool,
    pub retry_after: Duration,
}

impl DeliveryError {
    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: 0,
            retryable: false,
            retry_after: Duration::ZERO,
        }
    }

    fn retryable(message: impl Into<String>) -> Self {
        Self {
            retryable: true,
            ..Self::permanent(message)
        }
    }

    fn hide(mut self, secrets: &[Option<&str>]) -> Self {
        self.message = sanitize(&self.message, secrets);
        self
    }
}

fn sanitize(message: &str, secrets: &[Option<&str>]) -> String {
    let mut text = message.to_owned();
    for secret in secrets.iter().flatten() {
        if secret.len() >= MIN_HIDDEN_LENGTH {
            text = text.replace(secret, "***");
        }
    }
    if text.chars().count() > MAX_ERROR_LENGTH {
        let cut: String = text.chars().take(MAX_ERROR_LENGTH).collect();
        format!("{cut}…")
    } else {
        text
    }
}

pub struct Transport<'a> {
    pub telegram_api: &'a str,
    pub guard: Arc<Guard>,
    pub mail: Option<Arc<dyn MailSender>>,
}

impl Transport<'_> {
    pub async fn send(
        &self,
        channel: &Channel,
        secrets: &Secrets,
        message: &Message,
    ) -> Result<(), DeliveryError> {
        match channel.kind {
            ChannelType::Telegram => self.telegram(channel, secrets, &message.text).await,
            ChannelType::Email => self.email(channel, message).await,
            ChannelType::Slack => {
                let body = serde_json::to_vec(&json!({"text": message.text}))
                    .expect("a JSON value always serializes");
                self.webhook(channel, secrets, body, Vec::new()).await
            }
            ChannelType::Webhook => {
                let body = payload::body(message);
                let mut headers = vec![
                    (EVENT_HEADER, message.event.clone()),
                    (DELIVERY_HEADER, message.delivery_id.clone()),
                ];
                if let Some(secret) = &secrets.signing_secret {
                    headers.push((SIGNATURE_HEADER, payload::signature(secret, &body)));
                }
                if let Some(authorization) = &secrets.authorization {
                    headers.push(("Authorization", authorization.clone()));
                }
                self.webhook(channel, secrets, body, headers).await
            }
        }
    }

    async fn email(&self, channel: &Channel, message: &Message) -> Result<(), DeliveryError> {
        let Some(sender) = &self.mail else {
            return Err(DeliveryError::permanent(
                "Email is not set up on this server: configure the email server first",
            ));
        };
        let rendered = messages::Rendered {
            event: message.event.clone(),
            text: message.text.clone(),
            notice: message
                .envelope
                .as_ref()
                .map(|envelope| envelope.notice.clone()),
        };
        let mail = Mail {
            to: channel.recipients.clone(),
            subject: messages::subject(channel, &rendered),
            text: message.text.clone(),
        };
        sender.send(mail).await.map_err(|error| DeliveryError {
            message: sanitize(
                &format!("The email server refused the message: {}", error.message),
                &[],
            ),
            status: 0,
            retryable: error.retryable,
            retry_after: Duration::ZERO,
        })
    }

    fn client(
        &self,
        channel: &Channel,
        secrets: &Secrets,
        guarded: bool,
    ) -> Result<reqwest::Client, DeliveryError> {
        let mut builder = reqwest::Client::builder()
            .user_agent(format!("Astrum/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none());
        builder = match &channel.proxy {
            Proxy::Global | Proxy::Direct => {
                let builder = builder.no_proxy();
                if guarded {
                    builder.dns_resolver(Arc::new(GuardedResolver::new(self.guard.clone())))
                } else {
                    builder
                }
            }
            Proxy::Custom {
                kind,
                host,
                port,
                username,
            } => {
                let host = if host.contains(':') {
                    format!("[{host}]")
                } else {
                    host.clone()
                };
                let scheme = match kind {
                    ProxyKind::Http => "http",
                    ProxyKind::Socks5 => "socks5h",
                };
                let mut proxy =
                    reqwest::Proxy::all(format!("{scheme}://{host}:{port}")).map_err(|error| {
                        DeliveryError::permanent(format!("The proxy address is not valid: {error}"))
                    })?;
                if let Some(username) = username {
                    proxy =
                        proxy.basic_auth(username, secrets.proxy_password.as_deref().unwrap_or(""));
                }
                builder.proxy(proxy)
            }
        };
        builder.build().map_err(|error| {
            DeliveryError::permanent(format!("Could not prepare the request: {error}"))
        })
    }

    async fn telegram(
        &self,
        channel: &Channel,
        secrets: &Secrets,
        text: &str,
    ) -> Result<(), DeliveryError> {
        let token = secrets.secret.as_deref().unwrap_or("");
        if !bot_token_ok(token) {
            return Err(DeliveryError::permanent("The bot token is malformed"));
        }
        let hidden = [Some(token), secrets.proxy_password.as_deref()];
        let mut body = json!({
            "chat_id": channel.chat_id,
            "text": text,
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
        });
        if let Some(thread) = channel.message_thread_id {
            body["message_thread_id"] = json!(thread);
        }
        let client = self.client(channel, secrets, false)?;
        let url = format!(
            "{}/bot{token}/sendMessage",
            self.telegram_api.trim_end_matches('/')
        );
        let response = client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&body).expect("a JSON value always serializes"))
            .send()
            .await
            .map_err(|error| unreachable("Telegram", channel, &error).hide(&hidden))?;
        let status = response.status().as_u16();
        proxy_refusal(channel, status).map_err(|error| error.hide(&hidden))?;
        let header_delay = retry_after_header(response.headers());
        let content = response.text().await.unwrap_or_default();
        let json: Option<Value> = serde_json::from_str(&content).ok().filter(Value::is_object);
        let ok = json
            .as_ref()
            .and_then(|json| json.get("ok"))
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if (200..300).contains(&status) && ok {
            return Ok(());
        }
        let description = json
            .as_ref()
            .and_then(|json| json.get("description"))
            .and_then(Value::as_str);
        let delay = json
            .as_ref()
            .and_then(|json| json.pointer("/parameters/retry_after"))
            .and_then(Value::as_u64)
            .filter(|seconds| *seconds > 0)
            .map(Duration::from_secs)
            .unwrap_or(header_delay);
        Err(DeliveryError {
            message: match description {
                Some(description) => format!("Telegram answered {status}: {description}"),
                None => format!("Telegram answered {status}"),
            },
            status,
            retryable: retryable(status),
            retry_after: delay.min(MAX_RETRY_AFTER),
        }
        .hide(&hidden))
    }

    async fn webhook(
        &self,
        channel: &Channel,
        secrets: &Secrets,
        body: Vec<u8>,
        headers: Vec<(&'static str, String)>,
    ) -> Result<(), DeliveryError> {
        let address = secrets.secret.as_deref().unwrap_or("");
        let hidden = [
            Some(address),
            secrets.authorization.as_deref(),
            secrets.signing_secret.as_deref(),
            secrets.proxy_password.as_deref(),
        ];
        let Some(url) = webhook_url(address) else {
            return Err(DeliveryError::permanent(
                "The webhook URL is not a valid http or https URL",
            ));
        };
        check_host(&self.guard, &url)
            .await
            .map_err(|error| error.hide(&hidden))?;
        let client = self.client(channel, secrets, true)?;
        let mut request = client
            .post(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/json; charset=UTF-8",
            )
            .body(body);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = request
            .send()
            .await
            .map_err(|error| unreachable("the webhook", channel, &error).hide(&hidden))?;
        let status = response.status().as_u16();
        proxy_refusal(channel, status).map_err(|error| error.hide(&hidden))?;
        if (200..300).contains(&status) {
            return Ok(());
        }
        let delay = retry_after_header(response.headers());
        let reason = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_owned();
        let content = response.text().await.unwrap_or_default();
        let reason = if !content.trim().is_empty() && !content.contains("<html") {
            content.trim().to_owned()
        } else {
            reason
        };
        Err(DeliveryError {
            message: if reason.is_empty() {
                format!("The webhook answered {status}")
            } else {
                format!("The webhook answered {status}: {reason}")
            },
            status,
            retryable: retryable(status),
            retry_after: delay.min(MAX_RETRY_AFTER),
        }
        .hide(&hidden))
    }
}

async fn check_host(guard: &Guard, url: &url::Url) -> Result<(), DeliveryError> {
    let blocked = |reason: &str| {
        DeliveryError::permanent(format!(
            "The webhook host is blocked by the SSRF protection: {reason}"
        ))
    };
    match url.host() {
        Some(url::Host::Ipv4(ip)) if !guard.ip_allowed(IpAddr::V4(ip), false) => {
            Err(blocked(&format!("{ip} is an internal address")))
        }
        Some(url::Host::Ipv6(ip)) if !guard.ip_allowed(IpAddr::V6(ip), false) => {
            Err(blocked(&format!("{ip} is an internal address")))
        }
        Some(url::Host::Domain(host)) => {
            let port = url.port_or_known_default().unwrap_or(443);
            let found: Vec<_> = tokio::net::lookup_host((host, port))
                .await
                .map_err(|error| {
                    DeliveryError::retryable(format!("Could not resolve {host}: {error}"))
                })?
                .collect();
            let host_allowed = guard.host_allowed(host);
            if found
                .iter()
                .any(|address| guard.ip_allowed(address.ip(), host_allowed))
            {
                Ok(())
            } else {
                Err(blocked(&format!(
                    "{host} resolves only to internal addresses"
                )))
            }
        }
        _ => Ok(()),
    }
}

fn proxy_refusal(channel: &Channel, status: u16) -> Result<(), DeliveryError> {
    let Proxy::Custom {
        kind: ProxyKind::Http,
        username,
        ..
    } = &channel.proxy
    else {
        return Ok(());
    };
    if status == 407 {
        return Err(DeliveryError::permanent(if username.is_some() {
            "The proxy rejected the username or password (407 Proxy Authentication Required)"
        } else {
            "The proxy requires a username and password (407 Proxy Authentication Required)"
        }));
    }
    Ok(())
}

fn unreachable(service: &str, channel: &Channel, error: &reqwest::Error) -> DeliveryError {
    let mut cause = String::new();
    let mut source: Option<&dyn std::error::Error> = Some(error);
    while let Some(current) = source {
        cause = current.to_string();
        source = current.source();
    }
    let through = match &channel.proxy {
        Proxy::Custom { host, port, .. } => format!(" through the proxy {host}:{port}"),
        _ => String::new(),
    };
    if error.is_timeout() {
        return DeliveryError::retryable(format!("{service} did not answer in time{through}"));
    }
    DeliveryError::retryable(format!("Could not reach {service}{through}: {cause}"))
}

fn retryable(status: u16) -> bool {
    status == 429 || status >= 500
}

fn retry_after_header(headers: &reqwest::header::HeaderMap) -> Duration {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_never_leave_in_messages() {
        let text = sanitize(
            "failed https://hooks.example.com/T0/B0/xyz with Bearer abcdef",
            &[
                Some("https://hooks.example.com/T0/B0/xyz"),
                Some("abc"),
                Some("Bearer abcdef"),
            ],
        );
        assert_eq!(text, "failed *** with ***");
        let long = sanitize(&"x".repeat(400), &[]);
        assert_eq!(long.chars().count(), MAX_ERROR_LENGTH + 1);
    }
}
