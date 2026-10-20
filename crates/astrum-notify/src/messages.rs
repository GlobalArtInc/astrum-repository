use std::collections::BTreeSet;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::channel::{Channel, ChannelType};
use crate::notice::{Action, Notice};

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
    repository: &'static str,
    by: &'static str,
    system: &'static str,
    open: &'static str,
    summary: &'static str,
    more: &'static str,
    test_title: &'static str,
    test_body: &'static str,
}

const RU: Texts = Texts {
    published: "Опубликован компонент",
    updated: "Обновлён компонент",
    deleted: "Удалён компонент",
    repository: "Репозиторий",
    by: "Автор",
    system: "система",
    open: "Открыть в Astrum Repository",
    summary: "Изменения компонентов: {}",
    more: "…и ещё {}",
    test_title: "Тестовое сообщение от Astrum Repository",
    test_body: "Уведомления канала «{}» настроены правильно.",
};

const EN: Texts = Texts {
    published: "Component published",
    updated: "Component updated",
    deleted: "Component deleted",
    repository: "Repository",
    by: "By",
    system: "system",
    open: "Open in Astrum Repository",
    summary: "Component changes: {}",
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
        if notice.format != "docker" || notice.action == Action::Deleted {
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
    }
}

fn title(texts: &Texts, action: Action) -> &'static str {
    match action {
        Action::Published => texts.published,
        Action::Updated => texts.updated,
        Action::Deleted => texts.deleted,
    }
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
    if let Some(by) = initiator(texts, notice.initiator.as_deref()) {
        text.push_str(&format!("\n{}: {}", texts.by, markup.escape(&by)));
    }
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
    let mut text = format!(
        "📋 {}",
        markup.bold(
            &texts
                .summary
                .replace("{}", &(notices.len() + overflow).to_string())
        )
    );
    let mut shown = 0;
    for notice in notices {
        let line = format!(
            "\n{} {} · {}",
            emoji(notice),
            markup.strong(&coordinates(notice)),
            markup.code(&notice.repository)
        );
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
            event: format!("component.{}", notice.action.id()),
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
}
