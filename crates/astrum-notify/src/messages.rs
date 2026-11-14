use std::collections::BTreeSet;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::channel::{Channel, ChannelType};
use crate::notice::{Action, Detail, Notice};

pub const SUMMARY_THRESHOLD: usize = 5;
const SUMMARY_MAX_LINES: usize = 20;
const SUMMARY_MAX_LENGTH: usize = 3500;
const SYSTEM_USER: &str = "*SYSTEM";

pub const TEST: &str = "test";
pub const SUMMARY: &str = "component.summary";

const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

struct Texts {
    published: &'static str,
    updated: &'static str,
    deleted: &'static str,
    vulnerability_found: &'static str,
    policy_blocked: &'static str,
    released_title: &'static str,
    released: &'static str,
    in_word: &'static str,
    repository: &'static str,
    advisory: &'static str,
    policy: &'static str,
    severities: [&'static str; 5],
    by: &'static str,
    system: &'static str,
    open: &'static str,
    summary: &'static str,
    security_summary: &'static str,
    more: &'static str,
    test_title: &'static str,
    test_body: &'static str,
}

const RU: Texts = Texts {
    published: "Опубликован компонент",
    updated: "Обновлён компонент",
    deleted: "Удалён компонент",
    vulnerability_found: "Найдена уязвимость",
    policy_blocked: "Скачивание заблокировано политикой",
    released_title: "Версия доступна",
    released: "{} можно скачать сейчас",
    in_word: "в",
    repository: "Репозиторий",
    advisory: "Уязвимость",
    policy: "Политика",
    severities: ["критическая", "высокая", "средняя", "низкая", "неизвестна"],
    by: "Автор",
    system: "система",
    open: "Открыть в Astrum Repository",
    summary: "Изменения компонентов: {}",
    security_summary: "Уведомления безопасности: {}",
    more: "…и ещё {}",
    test_title: "Тестовое сообщение от Astrum Repository",
    test_body: "Уведомления канала «{}» настроены правильно.",
};

const EN: Texts = Texts {
    published: "Component published",
    updated: "Component updated",
    deleted: "Component deleted",
    vulnerability_found: "Vulnerability found",
    policy_blocked: "Download blocked by policy",
    released_title: "Version can be downloaded",
    released: "{} can be downloaded now",
    in_word: "in",
    repository: "Repository",
    advisory: "Advisory",
    policy: "Policy",
    severities: ["critical", "high", "medium", "low", "unknown"],
    by: "By",
    system: "system",
    open: "Open in Astrum Repository",
    summary: "Component changes: {}",
    security_summary: "Security notices: {}",
    more: "…and {} more",
    test_title: "Test message from Astrum Repository",
    test_body: "Notifications of the channel “{}” are set up correctly.",
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Markup {
    Html,
    Mrkdwn,
    Plain,
}

impl Markup {
    pub(crate) fn of(channel: &Channel) -> Self {
        match channel.kind {
            ChannelType::Telegram => Markup::Html,
            ChannelType::Email => Markup::Plain,
            ChannelType::Slack | ChannelType::Webhook => Markup::Mrkdwn,
        }
    }

    pub(crate) fn escape(self, text: &str) -> String {
        match self {
            Markup::Html => text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;"),
            Markup::Mrkdwn => text
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;"),
            Markup::Plain => text.to_owned(),
        }
    }

    pub(crate) fn bold(self, text: &str) -> String {
        match self {
            Markup::Html => format!("<b>{}</b>", self.escape(text)),
            Markup::Mrkdwn => format!("*{}*", self.escape(text)),
            Markup::Plain => text.to_owned(),
        }
    }

    pub(crate) fn strong(self, text: &str) -> String {
        match self {
            Markup::Html => self.bold(text),
            Markup::Mrkdwn => self.code(text),
            Markup::Plain => text.to_owned(),
        }
    }

    pub(crate) fn code(self, text: &str) -> String {
        match self {
            Markup::Html => format!("<code>{}</code>", self.escape(text)),
            Markup::Mrkdwn => format!("`{}`", self.escape(&text.replace('`', "'"))),
            Markup::Plain => text.to_owned(),
        }
    }

    pub(crate) fn link(self, url: &str, text: &str) -> String {
        match self {
            Markup::Html => format!("<a href=\"{}\">{}</a>", self.escape(url), self.escape(text)),
            Markup::Mrkdwn => format!(
                "<{}|{}>",
                self.escape(url).replace('|', "%7C"),
                self.escape(text)
            ),
            Markup::Plain => format!("{text}: {url}"),
        }
    }
}

fn texts(channel: &Channel) -> &'static Texts {
    if channel.language == "en" { &EN } else { &RU }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    pub event: String,
    pub text: String,
    pub notice: Option<Notice>,
}

#[derive(Clone, Debug, Default)]
pub struct Links {
    pub base_url: Option<String>,
}

impl Links {
    fn base(&self) -> Option<&str> {
        self.base_url
            .as_deref()
            .map(|base| base.trim_end_matches('/'))
            .filter(|base| !base.is_empty())
    }

    pub fn component(&self, notice: &Notice) -> Option<String> {
        if notice.action == Action::Deleted {
            return None;
        }
        let base = self.base()?;
        let id = format!("{}:{}:{}", notice.format, notice.namespace, notice.name);
        let mut link = format!(
            "{base}/search/component/{}",
            utf8_percent_encode(&id, COMPONENT)
        );
        if !notice.version.is_empty() {
            link.push_str("?version=");
            link.push_str(&utf8_percent_encode(&notice.version, COMPONENT).to_string());
        }
        Some(link)
    }

    pub fn pull_command(&self, notice: &Notice) -> Option<String> {
        if notice.format != "docker"
            || notice.action == Action::Deleted
            || notice.action.is_security()
        {
            return None;
        }
        let base = self.base()?;
        let host = url::Url::parse(base).ok()?;
        let authority = match host.port() {
            Some(port) => format!("{}:{port}", host.host_str()?),
            None => host.host_str()?.to_owned(),
        };
        Some(format!(
            "docker pull {authority}/{}/{}",
            notice.repository,
            coordinates(notice)
        ))
    }
}

pub fn coordinates(notice: &Notice) -> String {
    let (namespace, name, version) = (&notice.namespace, &notice.name, &notice.version);
    match notice.format.as_str() {
        "docker" if version.is_empty() => name.clone(),
        "docker" => format!("{name}:{version}"),
        "npm" => {
            let scope = if namespace.is_empty() {
                String::new()
            } else if namespace.starts_with('@') {
                format!("{namespace}/")
            } else {
                format!("@{namespace}/")
            };
            if version.is_empty() {
                format!("{scope}{name}")
            } else {
                format!("{scope}{name}@{version}")
            }
        }
        "raw" => name.clone(),
        _ => {
            let mut text = String::new();
            if !namespace.is_empty() {
                text.push_str(namespace);
                text.push(':');
            }
            text.push_str(name);
            if !version.is_empty() {
                text.push(':');
                text.push_str(version);
            }
            text
        }
    }
}

fn emoji(notice: &Notice) -> &'static str {
    match notice.action {
        Action::Published if notice.format == "docker" => "🐳",
        Action::Published => "📦",
        Action::Updated => "🔄",
        Action::Deleted => "🗑️",
        Action::VulnerabilityFound => "🛡️",
        Action::PolicyBlocked => "⛔",
        Action::CooldownReleased => "🔓",
    }
}

fn title(texts: &Texts, action: Action) -> &'static str {
    match action {
        Action::Published => texts.published,
        Action::Updated => texts.updated,
        Action::Deleted => texts.deleted,
        Action::VulnerabilityFound => texts.vulnerability_found,
        Action::PolicyBlocked => texts.policy_blocked,
        Action::CooldownReleased => texts.released_title,
    }
}

fn severity(texts: &Texts, name: &str) -> &'static str {
    match name {
        "critical" => texts.severities[0],
        "high" => texts.severities[1],
        "medium" => texts.severities[2],
        "low" => texts.severities[3],
        _ => texts.severities[4],
    }
}

fn detail_lines(markup: Markup, texts: &Texts, notice: &Notice) -> Vec<String> {
    let empty = Detail::default();
    let detail = notice.detail.as_ref().unwrap_or(&empty);
    let mut lines = Vec::new();
    if notice.action == Action::VulnerabilityFound
        && let Some(advisory) = detail.advisory.as_deref()
    {
        let mut line = format!("{}: {}", texts.advisory, markup.code(advisory));
        if let Some(name) = detail.severity.as_deref() {
            line.push_str(" · ");
            line.push_str(&markup.escape(severity(texts, name)));
        }
        lines.push(line);
    }
    if let Some(policy) = detail.policy.as_deref() {
        lines.push(format!("{}: {}", texts.policy, markup.code(policy)));
    }
    lines
}

fn released(markup: Markup, texts: &Texts, notice: &Notice) -> String {
    let subject = format!(
        "{} ({}) {} {}",
        markup.strong(&coordinates(notice)),
        markup.escape(&notice.format),
        texts.in_word,
        markup.code(&notice.repository)
    );
    texts.released.replacen("{}", &subject, 1)
}

fn initiator(texts: &Texts, initiator: Option<&str>) -> Option<String> {
    let initiator = initiator.filter(|value| !value.trim().is_empty())?;
    if initiator == SYSTEM_USER {
        return Some(texts.system.to_owned());
    }
    if initiator.starts_with('*') {
        return None;
    }
    Some(initiator.to_owned())
}

pub fn single(channel: &Channel, links: &Links, notice: &Notice) -> String {
    let markup = Markup::of(channel);
    let texts = texts(channel);
    let mut text = if notice.action == Action::CooldownReleased {
        format!("{} {}", emoji(notice), released(markup, texts, notice))
    } else {
        let mut text = format!(
            "{} {}\n{}\n{}: {}",
            emoji(notice),
            markup.bold(title(texts, notice.action)),
            markup.strong(&coordinates(notice)),
            texts.repository,
            markup.code(&notice.repository)
        );
        if !notice.format.is_empty() {
            text.push_str(" · ");
            text.push_str(&markup.escape(&notice.format));
        }
        for line in detail_lines(markup, texts, notice) {
            text.push('\n');
            text.push_str(&line);
        }
        if let Some(by) = initiator(texts, notice.initiator.as_deref()) {
            text.push_str(&format!("\n{}: {}", texts.by, markup.escape(&by)));
        }
        text
    };
    if let Some(pull) = links.pull_command(notice) {
        text.push('\n');
        text.push_str(&markup.code(&pull));
    }
    if let Some(link) = links.component(notice) {
        text.push('\n');
        text.push_str(&markup.link(&link, texts.open));
    }
    text
}

fn summary(channel: &Channel, notices: &[Notice], overflow: usize) -> String {
    let markup = Markup::of(channel);
    let texts = texts(channel);
    let heading = if notices.iter().all(|notice| notice.action.is_security()) {
        texts.security_summary
    } else {
        texts.summary
    };
    let mut text = format!(
        "📋 {}",
        markup.bold(&heading.replace("{}", &(notices.len() + overflow).to_string()))
    );
    let mut shown = 0;
    for notice in notices {
        let mut line = format!(
            "\n{} {} · {}",
            emoji(notice),
            markup.strong(&coordinates(notice)),
            markup.code(&notice.repository)
        );
        if let Some(detail) = notice.detail.as_ref() {
            let extra = match notice.action {
                Action::VulnerabilityFound => detail.advisory.as_deref(),
                _ => detail.policy.as_deref(),
            };
            if let Some(extra) = extra {
                line.push_str(" · ");
                line.push_str(&markup.code(extra));
            }
        }
        if shown == SUMMARY_MAX_LINES
            || text.chars().count() + line.chars().count() > SUMMARY_MAX_LENGTH
        {
            break;
        }
        text.push_str(&line);
        shown += 1;
    }
    let rest = notices.len() - shown + overflow;
    if rest > 0 {
        text.push('\n');
        text.push_str(&markup.escape(&texts.more.replace("{}", &rest.to_string())));
    }
    let initiators: BTreeSet<Option<String>> = notices
        .iter()
        .map(|notice| initiator(texts, notice.initiator.as_deref()))
        .collect();
    if initiators.len() == 1
        && let Some(Some(by)) = initiators.into_iter().next()
    {
        text.push_str(&format!("\n{}: {}", texts.by, markup.escape(&by)));
    }
    text
}

pub fn render(
    channel: &Channel,
    links: &Links,
    notices: &[Notice],
    overflow: usize,
) -> Vec<Rendered> {
    if channel.kind != ChannelType::Webhook && notices.len() + overflow > SUMMARY_THRESHOLD {
        return vec![Rendered {
            event: SUMMARY.to_owned(),
            text: summary(channel, notices, overflow),
            notice: None,
        }];
    }
    notices
        .iter()
        .map(|notice| Rendered {
            event: notice.action.event().to_owned(),
            text: single(channel, links, notice),
            notice: Some(notice.clone()),
        })
        .collect()
}

pub fn test(channel: &Channel, base_url: Option<&str>) -> String {
    let markup = Markup::of(channel);
    let texts = texts(channel);
    let mut text = format!(
        "✅ {}\n{}",
        markup.bold(texts.test_title),
        markup.escape(&texts.test_body.replace("{}", &channel.name))
    );
    if let Some(base) = base_url.filter(|base| !base.trim().is_empty()) {
        text.push('\n');
        text.push_str(&markup.link(base, "Astrum Repository"));
    }
    text
}

pub fn subject(channel: &Channel, rendered: &Rendered) -> String {
    let texts = texts(channel);
    match &rendered.notice {
        Some(notice) => format!("{}: {}", title(texts, notice.action), coordinates(notice)),
        None if rendered.event == TEST => texts.test_title.to_owned(),
        None if crate::alerts::is_alert(&rendered.event) => {
            crate::alerts::title(channel, &rendered.event).to_owned()
        }
        None if crate::customers::is_notice(&rendered.event) => {
            crate::customers::title(&rendered.event).to_owned()
        }
        None => texts.summary.replace("{}", "…"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::tests::channel;

    fn notice(format: &str, namespace: &str, name: &str, version: &str) -> Notice {
        Notice {
            repository: "releases".into(),
            repository_type: "hosted".into(),
            format: format.into(),
            namespace: namespace.into(),
            name: name.into(),
            version: version.into(),
            action: Action::Published,
            initiator: Some("jane".into()),
            component_id: Some(3),
            detail: None,
        }
    }

    #[test]
    fn coordinates_follow_the_format() {
        assert_eq!(
            coordinates(&notice("maven2", "org.example", "demo", "1.0")),
            "org.example:demo:1.0"
        );
        assert_eq!(coordinates(&notice("docker", "", "app", "1.2")), "app:1.2");
        assert_eq!(
            coordinates(&notice("npm", "scope", "pkg", "2.0.0")),
            "@scope/pkg@2.0.0"
        );
        assert_eq!(
            coordinates(&notice("raw", "/docs", "docs/a.txt", "")),
            "docs/a.txt"
        );
    }

    #[test]
    fn telegram_gets_html_and_slack_gets_mrkdwn() {
        let links = Links {
            base_url: Some("https://repo.example.com/".into()),
        };
        let mut docker = notice("docker", "", "app<x>", "1.0");
        docker.repository = "docker-hosted".into();
        let html = single(&channel(ChannelType::Telegram, "ru"), &links, &docker);
        assert!(html.starts_with("🐳 <b>Опубликован компонент</b>\n<b>app&lt;x&gt;:1.0</b>"));
        assert!(html.contains("Автор: jane"));
        assert!(
            html.contains(
                "<code>docker pull repo.example.com/docker-hosted/app&lt;x&gt;:1.0</code>"
            )
        );
        assert!(html.contains(
            "<a href=\"https://repo.example.com/search/component/docker%3A%3Aapp%3Cx%3E?version=1.0\">Открыть в Astrum Repository</a>"
        ));
        let slack = single(
            &channel(ChannelType::Slack, "en"),
            &Links::default(),
            &notice("maven2", "org.example", "demo", "1.0"),
        );
        assert_eq!(
            slack,
            "📦 *Component published*\n`org.example:demo:1.0`\nRepository: `releases` · maven2\nBy: jane"
        );
    }

    #[test]
    fn a_released_version_is_announced_in_one_sentence() {
        let mut released = notice("npm", "", "lodash", "4.17.22");
        released.action = Action::CooldownReleased;
        let slack = single(
            &channel(ChannelType::Slack, "en"),
            &Links::default(),
            &released,
        );
        assert_eq!(
            slack,
            "🔓 `lodash@4.17.22` (npm) in `releases` can be downloaded now"
        );
        let mut hostile = notice("maven2", "g", "x<y", "1.0");
        hostile.action = Action::CooldownReleased;
        let html = single(
            &channel(ChannelType::Telegram, "en"),
            &Links::default(),
            &hostile,
        );
        assert_eq!(
            html,
            "🔓 <b>g:x&lt;y:1.0</b> (maven2) in <code>releases</code> can be downloaded now"
        );
        let hooks = render(
            &channel(ChannelType::Webhook, "en"),
            &Links::default(),
            &[released],
            0,
        );
        assert_eq!(hooks[0].event, "cooldown.released");
    }

    #[test]
    fn many_changes_become_one_summary_except_for_json_webhooks() {
        let notices: Vec<_> = (0..7)
            .map(|i| notice("maven2", "g", &format!("a{i}"), "1"))
            .collect();
        let links = Links::default();
        let chat = render(&channel(ChannelType::Telegram, "en"), &links, &notices, 2);
        assert_eq!(chat.len(), 1);
        assert_eq!(chat[0].event, SUMMARY);
        assert!(chat[0].text.starts_with("📋 <b>Component changes: 9</b>"));
        assert!(chat[0].text.contains("…and 2 more") || chat[0].text.contains("By: jane"));
        let hooks = render(&channel(ChannelType::Webhook, "en"), &links, &notices, 2);
        assert_eq!(hooks.len(), 7);
        assert_eq!(hooks[0].event, "component.published");
    }

    fn security(action: Action, advisory: Option<&str>) -> Notice {
        let mut found = notice("maven2", "org.example", "demo", "1.0");
        found.action = action;
        found.initiator = None;
        found.detail = Some(Detail {
            policy: Some("strict".into()),
            advisory: advisory.map(str::to_owned),
            severity: advisory.map(|_| "critical".to_owned()),
        });
        found
    }

    #[test]
    fn security_notices_are_titled_in_every_markup_and_language_and_never_carry_reasons() {
        let vulnerable = security(Action::VulnerabilityFound, Some("GHSA-1"));
        let blocked = security(Action::PolicyBlocked, None);
        let cases = [
            (
                ChannelType::Telegram,
                "en",
                &vulnerable,
                "🛡️ <b>Vulnerability found</b>\n<b>org.example:demo:1.0</b>\nRepository: <code>releases</code> · maven2\nAdvisory: <code>GHSA-1</code> · critical\nPolicy: <code>strict</code>",
            ),
            (
                ChannelType::Telegram,
                "ru",
                &vulnerable,
                "🛡️ <b>Найдена уязвимость</b>\n<b>org.example:demo:1.0</b>\nРепозиторий: <code>releases</code> · maven2\nУязвимость: <code>GHSA-1</code> · критическая\nПолитика: <code>strict</code>",
            ),
            (
                ChannelType::Slack,
                "en",
                &vulnerable,
                "🛡️ *Vulnerability found*\n`org.example:demo:1.0`\nRepository: `releases` · maven2\nAdvisory: `GHSA-1` · critical\nPolicy: `strict`",
            ),
            (
                ChannelType::Slack,
                "ru",
                &vulnerable,
                "🛡️ *Найдена уязвимость*\n`org.example:demo:1.0`\nРепозиторий: `releases` · maven2\nУязвимость: `GHSA-1` · критическая\nПолитика: `strict`",
            ),
            (
                ChannelType::Email,
                "en",
                &vulnerable,
                "🛡️ Vulnerability found\norg.example:demo:1.0\nRepository: releases · maven2\nAdvisory: GHSA-1 · critical\nPolicy: strict",
            ),
            (
                ChannelType::Email,
                "ru",
                &vulnerable,
                "🛡️ Найдена уязвимость\norg.example:demo:1.0\nРепозиторий: releases · maven2\nУязвимость: GHSA-1 · критическая\nПолитика: strict",
            ),
            (
                ChannelType::Telegram,
                "en",
                &blocked,
                "⛔ <b>Download blocked by policy</b>\n<b>org.example:demo:1.0</b>\nRepository: <code>releases</code> · maven2\nPolicy: <code>strict</code>",
            ),
            (
                ChannelType::Telegram,
                "ru",
                &blocked,
                "⛔ <b>Скачивание заблокировано политикой</b>\n<b>org.example:demo:1.0</b>\nРепозиторий: <code>releases</code> · maven2\nПолитика: <code>strict</code>",
            ),
            (
                ChannelType::Slack,
                "en",
                &blocked,
                "⛔ *Download blocked by policy*\n`org.example:demo:1.0`\nRepository: `releases` · maven2\nPolicy: `strict`",
            ),
            (
                ChannelType::Slack,
                "ru",
                &blocked,
                "⛔ *Скачивание заблокировано политикой*\n`org.example:demo:1.0`\nРепозиторий: `releases` · maven2\nПолитика: `strict`",
            ),
            (
                ChannelType::Email,
                "en",
                &blocked,
                "⛔ Download blocked by policy\norg.example:demo:1.0\nRepository: releases · maven2\nPolicy: strict",
            ),
            (
                ChannelType::Email,
                "ru",
                &blocked,
                "⛔ Скачивание заблокировано политикой\norg.example:demo:1.0\nРепозиторий: releases · maven2\nПолитика: strict",
            ),
        ];
        for (kind, language, notice, expected) in cases {
            let channel = channel(kind, language);
            assert_eq!(single(&channel, &Links::default(), notice), expected);
        }
        let rendered = Rendered {
            event: "vulnerability.found".into(),
            text: String::new(),
            notice: Some(vulnerable.clone()),
        };
        assert_eq!(
            subject(&channel(ChannelType::Email, "en"), &rendered),
            "Vulnerability found: org.example:demo:1.0"
        );
        let hooks = render(
            &channel(ChannelType::Webhook, "en"),
            &Links::default(),
            &[vulnerable, blocked],
            0,
        );
        assert_eq!(hooks[0].event, "vulnerability.found");
        assert_eq!(hooks[1].event, "policy.blocked");
    }

    #[test]
    fn text_from_outside_is_escaped_and_a_docker_security_notice_has_no_pull_command() {
        let mut hostile = security(Action::VulnerabilityFound, Some("GHSA-<b>x</b>&"));
        hostile.format = "docker".into();
        let links = Links {
            base_url: Some("https://repo.example.com".into()),
        };
        let html = single(&channel(ChannelType::Telegram, "en"), &links, &hostile);
        assert!(
            html.contains("<code>GHSA-&lt;b&gt;x&lt;/b&gt;&amp;</code>"),
            "{html}"
        );
        assert!(!html.contains("docker pull"));
        assert!(links.pull_command(&hostile).is_none());
        assert!(links.component(&hostile).is_some());
    }

    #[test]
    fn a_window_of_security_notices_is_one_summary_with_its_own_title() {
        let notices: Vec<_> = (0..7)
            .map(|i| {
                let mut found = security(Action::VulnerabilityFound, Some("GHSA-1"));
                found.name = format!("a{i}");
                found
            })
            .collect();
        let chat = render(
            &channel(ChannelType::Telegram, "en"),
            &Links::default(),
            &notices,
            3,
        );
        assert_eq!(chat.len(), 1);
        assert_eq!(chat[0].event, SUMMARY);
        assert!(chat[0].text.starts_with("📋 <b>Security notices: 10</b>"));
        assert!(chat[0].text.contains("<code>GHSA-1</code>"));
        assert!(chat[0].text.contains("…and 3 more"));
    }
}
