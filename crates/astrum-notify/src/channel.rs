use astrum_security::SecretBox;
use regex_lite_free::{chat_id_ok, header_value_ok, proxy_host_ok};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity;
use crate::notice::Action;

pub const MAX_NAME_LENGTH: usize = 100;
const MAX_PROXY_HOST_LENGTH: usize = 255;
const MAX_PROXY_CREDENTIAL_BYTES: usize = 255;
const MAX_SIGNING_SECRET_LENGTH: usize = 255;
const MAX_AUTHORIZATION_LENGTH: usize = 4096;
const MAX_RECIPIENTS: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    Telegram,
    Slack,
    Webhook,
    Email,
}

impl ChannelType {
    pub fn id(self) -> &'static str {
        match self {
            ChannelType::Telegram => "telegram",
            ChannelType::Slack => "slack",
            ChannelType::Webhook => "webhook",
            ChannelType::Email => "email",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        [Self::Telegram, Self::Slack, Self::Webhook, Self::Email]
            .into_iter()
            .find(|kind| kind.id() == value)
    }

    fn default_name(self) -> &'static str {
        match self {
            ChannelType::Telegram => "Telegram",
            ChannelType::Slack => "Slack",
            ChannelType::Webhook => "Webhook",
            ChannelType::Email => "Email",
        }
    }

    pub fn has_url(self) -> bool {
        matches!(self, ChannelType::Slack | ChannelType::Webhook)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub kind: ChannelType,
    pub enabled: bool,
    pub chat_id: Option<String>,
    pub message_thread_id: Option<i64>,
    pub recipients: Vec<String>,
    pub target_host: Option<String>,
    pub repositories: Vec<String>,
    pub events: Vec<Action>,
    pub language: String,
    pub proxy: Proxy,
    pub sealed: Sealed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sealed {
    pub secret: Option<String>,
    pub proxy_password: Option<String>,
    pub signing_secret: Option<String>,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Secrets {
    pub secret: Option<String>,
    pub proxy_password: Option<String>,
    pub signing_secret: Option<String>,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proxy {
    Global,
    Direct,
    Custom {
        kind: ProxyKind,
        host: String,
        port: u16,
        username: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProxyKind {
    Http,
    Socks5,
}

impl ProxyKind {
    pub fn id(self) -> &'static str {
        match self {
            ProxyKind::Http => "http",
            ProxyKind::Socks5 => "socks5",
        }
    }
}

impl Proxy {
    pub fn mode(&self) -> &'static str {
        match self {
            Proxy::Global => "global",
            Proxy::Direct => "none",
            Proxy::Custom { .. } => "custom",
        }
    }
}

impl Channel {
    pub fn wants(&self, action: Action) -> bool {
        self.events.contains(&action)
    }

    pub fn target(&self) -> Option<String> {
        match self.kind {
            ChannelType::Telegram => self.chat_id.clone(),
            ChannelType::Email => Some(self.recipients.join(", ")).filter(|text| !text.is_empty()),
            _ => self.target_host.clone(),
        }
    }

    pub fn open(
        &self,
        secrets: &SecretBox,
    ) -> Result<Secrets, astrum_security::secrets::SecretError> {
        let open = |value: &Option<String>| -> Result<Option<String>, astrum_security::secrets::SecretError> {
            value.as_deref().map(|sealed| secrets.open(sealed)).transpose()
        };
        Ok(Secrets {
            secret: open(&self.sealed.secret)?,
            proxy_password: open(&self.sealed.proxy_password)?,
            signing_secret: open(&self.sealed.signing_secret)?,
            authorization: open(&self.sealed.authorization)?,
        })
    }

    pub fn from_row(row: entity::Model) -> Self {
        let proxy = match row.proxy_mode.as_str() {
            "none" => Proxy::Direct,
            "custom" => match (row.proxy_host, row.proxy_port) {
                (Some(host), Some(port)) => Proxy::Custom {
                    kind: if row.proxy_type.as_deref() == Some("socks5") {
                        ProxyKind::Socks5
                    } else {
                        ProxyKind::Http
                    },
                    host,
                    port: u16::try_from(port).unwrap_or(0),
                    username: row.proxy_username,
                },
                _ => Proxy::Global,
            },
            _ => Proxy::Global,
        };
        Self {
            id: row.id,
            name: row.name,
            kind: ChannelType::parse(&row.kind).unwrap_or(ChannelType::Webhook),
            enabled: row.enabled,
            chat_id: row.chat_id,
            message_thread_id: row.message_thread_id,
            recipients: row.recipients,
            target_host: row.target_host,
            repositories: row.repositories,
            events: row
                .events
                .iter()
                .filter_map(|event| Action::parse(event))
                .collect(),
            language: row.language,
            proxy,
            sealed: Sealed {
                secret: row.secret,
                proxy_password: row.proxy_password,
                signing_secret: row.signing_secret,
                authorization: row.authorization_header,
            },
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Draft {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub enabled: Option<bool>,
    pub chat_id: Option<String>,
    pub message_thread_id: Option<i64>,
    pub recipients: Option<Vec<String>>,
    pub secret: Option<String>,
    pub repositories: Option<Vec<String>>,
    pub events: Option<Vec<String>>,
    pub language: Option<String>,
    pub proxy_mode: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i64>,
    pub proxy_username: Option<String>,
    pub proxy_password: Option<String>,
    pub signing_secret: Option<String>,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {message}")]
pub struct Invalid {
    pub field: &'static str,
    pub message: String,
}

fn invalid(field: &'static str, message: impl Into<String>) -> Invalid {
    Invalid {
        field,
        message: message.into(),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Keep,
    Set(String),
    Clear,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Changes {
    pub secret: Change,
    pub proxy_password: Change,
    pub signing_secret: Change,
    pub authorization: Change,
}

fn trim(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub fn webhook_url(value: &str) -> Option<url::Url> {
    let url = url::Url::parse(value.trim()).ok()?;
    let plain = matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some_and(|host| !host.is_empty())
        && url.username().is_empty()
        && url.password().is_none();
    plain.then_some(url)
}

pub fn bot_token_ok(value: &str) -> bool {
    match value.split_once(':') {
        Some((bot, key)) => {
            !bot.is_empty()
                && bot.bytes().all(|b| b.is_ascii_digit())
                && !key.is_empty()
                && key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        }
        None => false,
    }
}

fn email_ok(value: &str) -> bool {
    let Some((local, domain)) = value.rsplit_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, ',' | '<' | '>'))
}

pub struct Validated {
    pub channel: Channel,
    pub changes: Changes,
}

pub fn validate(
    draft: &Draft,
    existing: Option<&Channel>,
    for_test: bool,
) -> Result<Validated, Invalid> {
    let kind = trim(draft.kind.as_deref())
        .and_then(|kind| ChannelType::parse(&kind))
        .ok_or_else(|| invalid("type", "Type must be telegram, slack, webhook or email"))?;
    let name = match trim(draft.name.as_deref()) {
        Some(name) => name,
        None if for_test => kind.default_name().to_owned(),
        None => return Err(invalid("name", "Name is required")),
    };
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(invalid(
            "name",
            format!("Name must not be longer than {MAX_NAME_LENGTH} characters"),
        ));
    }
    let mut chat_id = None;
    let mut message_thread_id = None;
    if kind == ChannelType::Telegram {
        let id = trim(draft.chat_id.as_deref())
            .ok_or_else(|| invalid("chatId", "Chat ID is required"))?;
        if !chat_id_ok(&id) {
            return Err(invalid(
                "chatId",
                "Chat ID must be a number such as -1001234567890 or a public channel name such as @releases",
            ));
        }
        chat_id = Some(id);
        if let Some(thread) = draft.message_thread_id {
            if thread <= 0 {
                return Err(invalid(
                    "messageThreadId",
                    "Topic ID must be a positive number",
                ));
            }
            message_thread_id = Some(thread);
        }
    }
    let mut recipients = Vec::new();
    if kind == ChannelType::Email {
        for recipient in draft.recipients.iter().flatten() {
            let recipient = recipient.trim();
            if recipient.is_empty() {
                continue;
            }
            if !email_ok(recipient) {
                return Err(invalid(
                    "recipients",
                    format!("{recipient} is not an email address"),
                ));
            }
            if !recipients
                .iter()
                .any(|known: &String| known.eq_ignore_ascii_case(recipient))
            {
                recipients.push(recipient.to_owned());
            }
        }
        if recipients.is_empty() {
            return Err(invalid("recipients", "Enter at least one email address"));
        }
        if recipients.len() > MAX_RECIPIENTS {
            return Err(invalid(
                "recipients",
                format!("A channel can have at most {MAX_RECIPIENTS} recipients"),
            ));
        }
    }
    let language = trim(draft.language.as_deref()).unwrap_or_else(|| "ru".to_owned());
    if language != "ru" && language != "en" {
        return Err(invalid("language", "Language must be ru or en"));
    }
    let proxy = proxy(draft)?;
    let events = if for_test {
        Vec::new()
    } else {
        events(draft.events.as_deref())?
    };
    let repositories = if for_test {
        Vec::new()
    } else {
        let mut names: Vec<String> = Vec::new();
        for name in draft.repositories.iter().flatten() {
            let name = name.trim();
            if !name.is_empty() && !names.iter().any(|known| known == name) {
                names.push(name.to_owned());
            }
        }
        names
    };
    let mut channel = Channel {
        id: existing
            .map(|existing| existing.id)
            .unwrap_or_else(Uuid::new_v4),
        name,
        kind,
        enabled: draft.enabled.unwrap_or(true),
        chat_id,
        message_thread_id,
        recipients,
        target_host: None,
        repositories,
        events,
        language,
        proxy,
        sealed: Sealed::default(),
    };
    let (secret, target_host) = secret(draft, &channel, existing)?;
    channel.target_host = target_host;
    let changes = Changes {
        secret,
        proxy_password: proxy_password(draft, &channel, existing)?,
        signing_secret: signing_secret(draft, &channel, existing)?,
        authorization: authorization(draft, &channel, existing)?,
    };
    Ok(Validated { channel, changes })
}

fn proxy(draft: &Draft) -> Result<Proxy, Invalid> {
    let mode = trim(draft.proxy_mode.as_deref()).unwrap_or_else(|| "global".to_owned());
    match mode.as_str() {
        "global" => return Ok(Proxy::Global),
        "none" => return Ok(Proxy::Direct),
        "custom" => {}
        _ => {
            return Err(invalid(
                "proxyMode",
                "Connection must be global, none or custom",
            ));
        }
    }
    let kind = match trim(draft.proxy_type.as_deref()).as_deref() {
        Some("http") => ProxyKind::Http,
        Some("socks5") => ProxyKind::Socks5,
        _ => return Err(invalid("proxyType", "Proxy type must be http or socks5")),
    };
    let mut host = trim(draft.proxy_host.as_deref())
        .ok_or_else(|| invalid("proxyHost", "Proxy host is required"))?;
    if host.starts_with('[') && host.ends_with(']') {
        host = host[1..host.len() - 1].to_owned();
    }
    if host.len() > MAX_PROXY_HOST_LENGTH {
        return Err(invalid(
            "proxyHost",
            format!("Proxy host must not be longer than {MAX_PROXY_HOST_LENGTH} characters"),
        ));
    }
    if !proxy_host_ok(&host) {
        return Err(invalid(
            "proxyHost",
            "Proxy host must be a host name or an IP address, without a scheme or port",
        ));
    }
    let port = draft
        .proxy_port
        .ok_or_else(|| invalid("proxyPort", "Proxy port is required"))?;
    let port = u16::try_from(port)
        .ok()
        .filter(|port| *port > 0)
        .ok_or_else(|| invalid("proxyPort", "Proxy port must be between 1 and 65535"))?;
    let username = trim(draft.proxy_username.as_deref());
    if username
        .as_ref()
        .is_some_and(|name| name.len() > MAX_PROXY_CREDENTIAL_BYTES)
    {
        return Err(invalid(
            "proxyUsername",
            format!(
                "Proxy username must not be longer than {MAX_PROXY_CREDENTIAL_BYTES} characters"
            ),
        ));
    }
    Ok(Proxy::Custom {
        kind,
        host,
        port,
        username,
    })
}

fn events(events: Option<&[String]>) -> Result<Vec<Action>, Invalid> {
    let mut chosen = Vec::new();
    for event in events.into_iter().flatten() {
        let action = Action::parse(event.trim())
            .ok_or_else(|| invalid("events", format!("Unknown event: {event}")))?;
        chosen.push(action);
    }
    if chosen.is_empty() {
        return Err(invalid("events", "Choose at least one event"));
    }
    Ok(Action::ALL
        .into_iter()
        .filter(|action| chosen.contains(action))
        .collect())
}

fn secret_required(kind: ChannelType) -> &'static str {
    match kind {
        ChannelType::Telegram => "Bot token is required",
        _ => "Webhook URL is required",
    }
}

fn secret(
    draft: &Draft,
    channel: &Channel,
    existing: Option<&Channel>,
) -> Result<(Change, Option<String>), Invalid> {
    if channel.kind == ChannelType::Email {
        return Ok((
            if existing.is_some_and(|existing| existing.sealed.secret.is_some()) {
                Change::Clear
            } else {
                Change::Keep
            },
            None,
        ));
    }
    let Some(value) = trim(draft.secret.as_deref()) else {
        if let Some(existing) = existing
            && existing.sealed.secret.is_some()
            && existing.kind == channel.kind
        {
            return Ok((Change::Keep, existing.target_host.clone()));
        }
        return Err(invalid("secret", secret_required(channel.kind)));
    };
    if channel.kind == ChannelType::Telegram {
        if !bot_token_ok(&value) {
            return Err(invalid(
                "secret",
                "The bot token must look like 123456789:AAHdqTcvCH1vGWJxfSeofSAs0K5PALDsaw",
            ));
        }
        return Ok((Change::Set(value), None));
    }
    let url = webhook_url(&value)
        .ok_or_else(|| invalid("secret", "The webhook URL must be an http or https URL"))?;
    let host = url
        .host_str()
        .map(|host| host.trim_matches(['[', ']']).to_owned());
    Ok((Change::Set(value), host))
}

fn same_proxy(existing: &Channel, channel: &Channel) -> bool {
    match (&existing.proxy, &channel.proxy) {
        (
            Proxy::Custom {
                kind: a_kind,
                host: a_host,
                port: a_port,
                ..
            },
            Proxy::Custom {
                kind: b_kind,
                host: b_host,
                port: b_port,
                ..
            },
        ) => a_kind == b_kind && a_host.eq_ignore_ascii_case(b_host) && a_port == b_port,
        _ => false,
    }
}

fn proxy_password(
    draft: &Draft,
    channel: &Channel,
    existing: Option<&Channel>,
) -> Result<Change, Invalid> {
    let value = draft.proxy_password.as_deref();
    let stored = existing.is_some_and(|existing| existing.sealed.proxy_password.is_some());
    let username = match &channel.proxy {
        Proxy::Custom { username, .. } => username.clone(),
        _ => None,
    };
    let custom = matches!(channel.proxy, Proxy::Custom { .. });
    if !custom || username.is_none() {
        if custom && value.is_some_and(|value| !value.is_empty()) {
            return Err(invalid(
                "proxyUsername",
                "Enter the username that goes with the proxy password",
            ));
        }
        return Ok(if stored { Change::Clear } else { Change::Keep });
    }
    match value {
        None => {
            if stored && !existing.is_some_and(|existing| same_proxy(existing, channel)) {
                return Err(invalid(
                    "proxyPassword",
                    "Enter the proxy password again: the proxy has changed",
                ));
            }
            Ok(Change::Keep)
        }
        Some("") => Ok(if stored { Change::Clear } else { Change::Keep }),
        Some(value) if value.len() > MAX_PROXY_CREDENTIAL_BYTES => Err(invalid(
            "proxyPassword",
            format!(
                "Proxy password must not be longer than {MAX_PROXY_CREDENTIAL_BYTES} characters"
            ),
        )),
        Some(value) => Ok(Change::Set(value.to_owned())),
    }
}

fn signing_secret(
    draft: &Draft,
    channel: &Channel,
    existing: Option<&Channel>,
) -> Result<Change, Invalid> {
    let stored = existing.is_some_and(|existing| existing.sealed.signing_secret.is_some());
    let value = draft.signing_secret.as_deref();
    if channel.kind != ChannelType::Webhook || value == Some("") {
        return Ok(if stored { Change::Clear } else { Change::Keep });
    }
    match value {
        None => Ok(Change::Keep),
        Some(value) if value.chars().count() > MAX_SIGNING_SECRET_LENGTH => Err(invalid(
            "signingSecret",
            format!(
                "Signing secret must not be longer than {MAX_SIGNING_SECRET_LENGTH} characters"
            ),
        )),
        Some(value) => Ok(Change::Set(value.to_owned())),
    }
}

fn authorization(
    draft: &Draft,
    channel: &Channel,
    existing: Option<&Channel>,
) -> Result<Change, Invalid> {
    let stored = existing.is_some_and(|existing| existing.sealed.authorization.is_some());
    let value = draft.authorization.as_deref();
    if channel.kind != ChannelType::Webhook || value.is_some_and(|value| value.trim().is_empty()) {
        return Ok(if stored { Change::Clear } else { Change::Keep });
    }
    let Some(value) = value else {
        if stored {
            let same_host = existing
                .and_then(|existing| existing.target_host.as_deref())
                .zip(channel.target_host.as_deref())
                .is_some_and(|(before, now)| before.eq_ignore_ascii_case(now));
            if !same_host {
                return Err(invalid(
                    "authorization",
                    "Enter the Authorization header again: the webhook host has changed",
                ));
            }
        }
        return Ok(Change::Keep);
    };
    let header = value.trim();
    if header.len() > MAX_AUTHORIZATION_LENGTH {
        return Err(invalid(
            "authorization",
            format!(
                "The Authorization header must not be longer than {MAX_AUTHORIZATION_LENGTH} characters"
            ),
        ));
    }
    if !header_value_ok(header) {
        return Err(invalid(
            "authorization",
            "The Authorization header must be one line of printable ASCII characters",
        ));
    }
    Ok(Change::Set(header.to_owned()))
}

mod regex_lite_free {
    pub fn chat_id_ok(value: &str) -> bool {
        if let Some(name) = value.strip_prefix('@') {
            let mut chars = name.chars();
            return (4..=32).contains(&name.len())
                && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        }
        let digits = value.strip_prefix('-').unwrap_or(value);
        (1..=20).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit())
    }

    pub fn header_value_ok(value: &str) -> bool {
        !value.is_empty()
            && value
                .bytes()
                .all(|b| (0x20..=0x7e).contains(&b) || b == b'\t')
    }

    pub fn proxy_host_ok(value: &str) -> bool {
        if value.contains(':') {
            return value
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || b == b':' || b == b'.');
        }
        let bytes = value.as_bytes();
        let edge = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        !bytes.is_empty()
            && edge(bytes[0])
            && edge(bytes[bytes.len() - 1])
            && bytes
                .iter()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn channel(kind: ChannelType, language: &str) -> Channel {
        Channel {
            id: Uuid::nil(),
            name: "Releases".into(),
            kind,
            enabled: true,
            chat_id: Some("-100123".into()),
            message_thread_id: None,
            recipients: Vec::new(),
            target_host: Some("hooks.example.com".into()),
            repositories: Vec::new(),
            events: Action::ALL.to_vec(),
            language: language.into(),
            proxy: Proxy::Global,
            sealed: Sealed::default(),
        }
    }

    fn draft(kind: &str) -> Draft {
        Draft {
            name: Some(" Releases ".into()),
            kind: Some(kind.into()),
            events: Some(vec!["deleted".into(), "published".into()]),
            ..Draft::default()
        }
    }

    #[test]
    fn telegram_channels_need_a_token_and_a_chat() {
        let mut telegram = draft("telegram");
        assert_eq!(
            validate(&telegram, None, false).err().unwrap().field,
            "chatId"
        );
        telegram.chat_id = Some("@ab".into());
        assert_eq!(
            validate(&telegram, None, false).err().unwrap().field,
            "chatId"
        );
        telegram.chat_id = Some("-1001234567890".into());
        assert_eq!(
            validate(&telegram, None, false).err().unwrap().field,
            "secret"
        );
        telegram.secret = Some("not a token".into());
        assert_eq!(
            validate(&telegram, None, false).err().unwrap().field,
            "secret"
        );
        telegram.secret = Some("123456:AAH-x_y".into());
        let valid = validate(&telegram, None, false).unwrap();
        assert_eq!(valid.channel.name, "Releases");
        assert_eq!(valid.channel.events, [Action::Published, Action::Deleted]);
        assert_eq!(valid.changes.secret, Change::Set("123456:AAH-x_y".into()));
        assert_eq!(valid.channel.target(), Some("-1001234567890".into()));
    }

    #[test]
    fn webhooks_keep_their_secrets_only_for_the_same_host_and_proxy() {
        let mut hook = draft("webhook");
        hook.secret = Some("https://user:pw@hooks.example.com/x".into());
        assert_eq!(validate(&hook, None, false).err().unwrap().field, "secret");
        hook.secret = Some("https://hooks.example.com/x".into());
        hook.authorization = Some("Bearer one\ntwo".into());
        assert_eq!(
            validate(&hook, None, false).err().unwrap().field,
            "authorization"
        );
        hook.authorization = Some("Bearer token".into());
        let first = validate(&hook, None, false).unwrap();
        assert_eq!(
            first.channel.target_host.as_deref(),
            Some("hooks.example.com")
        );
        let mut saved = first.channel.clone();
        saved.sealed = Sealed {
            secret: Some("sealed".into()),
            authorization: Some("sealed".into()),
            ..Sealed::default()
        };
        let mut again = draft("webhook");
        again.secret = Some("https://other.example.com/y".into());
        assert_eq!(
            validate(&again, Some(&saved), false).err().unwrap().field,
            "authorization"
        );
        again.secret = None;
        let kept = validate(&again, Some(&saved), false).unwrap();
        assert_eq!(kept.changes.secret, Change::Keep);
        assert_eq!(kept.changes.authorization, Change::Keep);
        again.authorization = Some(String::new());
        assert_eq!(
            validate(&again, Some(&saved), false)
                .unwrap()
                .changes
                .authorization,
            Change::Clear
        );
    }

    #[test]
    fn custom_proxies_are_checked() {
        let mut hook = draft("slack");
        hook.secret = Some("https://hooks.slack.com/services/x".into());
        hook.proxy_mode = Some("custom".into());
        assert_eq!(
            validate(&hook, None, false).err().unwrap().field,
            "proxyType"
        );
        hook.proxy_type = Some("socks5".into());
        hook.proxy_host = Some("http://proxy".into());
        assert_eq!(
            validate(&hook, None, false).err().unwrap().field,
            "proxyHost"
        );
        hook.proxy_host = Some("[::1]".into());
        hook.proxy_port = Some(70000);
        assert_eq!(
            validate(&hook, None, false).err().unwrap().field,
            "proxyPort"
        );
        hook.proxy_port = Some(1080);
        hook.proxy_password = Some("pw".into());
        assert_eq!(
            validate(&hook, None, false).err().unwrap().field,
            "proxyUsername"
        );
        hook.proxy_username = Some("me".into());
        let valid = validate(&hook, None, false).unwrap();
        assert_eq!(valid.changes.proxy_password, Change::Set("pw".into()));
        assert!(
            matches!(valid.channel.proxy, Proxy::Custom { ref host, port: 1080, .. } if host == "::1")
        );
    }

    #[test]
    fn email_channels_need_recipients() {
        let mut mail = draft("email");
        assert_eq!(
            validate(&mail, None, false).err().unwrap().field,
            "recipients"
        );
        mail.recipients = Some(vec!["team@example.com".into(), "bad address".into()]);
        assert_eq!(
            validate(&mail, None, false).err().unwrap().field,
            "recipients"
        );
        mail.recipients = Some(vec![
            "team@example.com".into(),
            "TEAM@example.com".into(),
            " ".into(),
        ]);
        let valid = validate(&mail, None, false).unwrap();
        assert_eq!(valid.channel.recipients, ["team@example.com"]);
        assert_eq!(valid.changes.secret, Change::Keep);
    }
}
