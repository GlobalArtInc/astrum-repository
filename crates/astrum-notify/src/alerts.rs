use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use uuid::Uuid;

use crate::Notifier;
use crate::channel::Channel;
use crate::messages::Markup;

pub const ALERT: &str = "blobstore.alert";
pub const RECOVERED: &str = "blobstore.recovered";
pub const REPLICATION_FAILING: &str = "replication.failing";
pub const REPLICATION_DEAD_ITEMS: &str = "replication.dead_items";
pub const REPLICATION_RECOVERED: &str = "replication.recovered";
const ERROR_EXCERPT: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    LowSpace {
        available: String,
        minimum: String,
    },
    Quota {
        message: String,
    },
    Unavailable,
    RepositoryWarn {
        percent: u32,
        used: String,
        limit: String,
    },
    RepositoryLimit {
        used: String,
        limit: String,
    },
    Forecast {
        days: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub store: String,
    pub problem: Problem,
}

struct Texts {
    alert: &'static str,
    recovered: &'static str,
    low: &'static str,
    roomy: &'static str,
    quota: &'static str,
    within_quota: &'static str,
    unavailable: &'static str,
    reachable: &'static str,
    repository_warn: &'static str,
    repository_limit: &'static str,
    within_repository_quota: &'static str,
    forecast: &'static str,
    no_forecast: &'static str,
    open: &'static str,
}

const EN: Texts = Texts {
    alert: "Blob stores need attention",
    recovered: "Blob stores are fine again",
    low: "{available} free, less than {minimum}",
    roomy: "{available} free again",
    quota: "passed its soft quota: {message}",
    within_quota: "within its soft quota again",
    unavailable: "cannot be reached",
    reachable: "can be reached again",
    repository_warn: "repository is at {percent}% of its {limit} quota ({used} used)",
    repository_limit: "repository reached its {limit} quota ({used} used)",
    within_repository_quota: "repository is below its quota warning again",
    forecast: "will be full in {days} days at the current growth",
    no_forecast: "is no longer forecast to fill up soon",
    open: "Open the blob stores",
};

const RU: Texts = Texts {
    alert: "Blob store требуют внимания",
    recovered: "С blob store снова всё в порядке",
    low: "свободно {available}, меньше {minimum}",
    roomy: "снова свободно {available}",
    quota: "превышена мягкая квота: {message}",
    within_quota: "снова в пределах мягкой квоты",
    unavailable: "недоступен",
    reachable: "снова доступен",
    repository_warn: "репозиторий заполнен на {percent}% квоты {limit} (занято {used})",
    repository_limit: "репозиторий достиг квоты {limit} (занято {used})",
    within_repository_quota: "репозиторий снова ниже порога предупреждения о квоте",
    forecast: "заполнится через {days} дн. при нынешнем росте",
    no_forecast: "больше не заполнится в ближайшее время",
    open: "Открыть blob store",
};

fn texts(channel: &Channel) -> &'static Texts {
    if channel.language == "en" { &EN } else { &RU }
}

pub fn is_alert(event: &str) -> bool {
    matches!(
        event,
        ALERT | RECOVERED | REPLICATION_FAILING | REPLICATION_DEAD_ITEMS | REPLICATION_RECOVERED
    )
}

pub fn title(channel: &Channel, event: &str) -> &'static str {
    let texts = texts(channel);
    match event {
        RECOVERED => texts.recovered,
        REPLICATION_FAILING | REPLICATION_DEAD_ITEMS | REPLICATION_RECOVERED => {
            replication_title(channel, event)
        }
        _ => texts.alert,
    }
}

fn describe(texts: &Texts, problem: &Problem, recovered: bool) -> String {
    match (problem, recovered) {
        (Problem::LowSpace { available, minimum }, false) => texts
            .low
            .replace("{available}", available)
            .replace("{minimum}", minimum),
        (Problem::LowSpace { available, .. }, true) => {
            texts.roomy.replace("{available}", available)
        }
        (Problem::Quota { message }, false) => texts.quota.replace("{message}", message),
        (Problem::Quota { .. }, true) => texts.within_quota.to_owned(),
        (Problem::Unavailable, false) => texts.unavailable.to_owned(),
        (Problem::Unavailable, true) => texts.reachable.to_owned(),
        (
            Problem::RepositoryWarn {
                percent,
                used,
                limit,
            },
            false,
        ) => texts
            .repository_warn
            .replace("{percent}", &percent.to_string())
            .replace("{used}", used)
            .replace("{limit}", limit),
        (Problem::RepositoryLimit { used, limit }, false) => texts
            .repository_limit
            .replace("{used}", used)
            .replace("{limit}", limit),
        (Problem::RepositoryWarn { .. } | Problem::RepositoryLimit { .. }, true) => {
            texts.within_repository_quota.to_owned()
        }
        (Problem::Forecast { days }, false) => texts.forecast.replace("{days}", &days.to_string()),
        (Problem::Forecast { .. }, true) => texts.no_forecast.to_owned(),
    }
}

pub fn render(
    channel: &Channel,
    event: &str,
    findings: &[Finding],
    base_url: Option<&str>,
) -> String {
    let markup = Markup::of(channel);
    let texts = texts(channel);
    let recovered = event == RECOVERED;
    let mut text = format!(
        "{} {}",
        if recovered { "✅" } else { "⚠️" },
        markup.bold(title(channel, event))
    );
    for finding in findings {
        text.push_str(&format!(
            "\n• {}: {}",
            markup.code(&finding.store),
            markup.escape(&describe(texts, &finding.problem, recovered))
        ));
    }
    if let Some(base) = base_url
        .map(|base| base.trim_end_matches('/'))
        .filter(|base| !base.is_empty())
    {
        text.push('\n');
        text.push_str(&markup.link(&format!("{base}/admin/repository/blobstores"), texts.open));
    }
    text
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplicationReport {
    pub replication: String,
    pub remote: String,
    pub pending: i64,
    pub leased: i64,
    pub dead: i64,
    pub suspended: bool,
    pub last_error: Option<String>,
}

struct ReplicationTexts {
    failing: &'static str,
    dead_items: &'static str,
    recovered: &'static str,
    remote: &'static str,
    backlog: &'static str,
    suspended: &'static str,
    error: &'static str,
}

const REPLICATION_EN: ReplicationTexts = ReplicationTexts {
    failing: "Replication is failing",
    dead_items: "Replication has dead items",
    recovered: "Replication is healthy again",
    remote: "remote",
    backlog: "{pending} waiting, {leased} in progress, {dead} dead",
    suspended: "the remote is paused after failures",
    error: "last error",
};

const REPLICATION_RU: ReplicationTexts = ReplicationTexts {
    failing: "Репликация не работает",
    dead_items: "В репликации есть мёртвые элементы",
    recovered: "Репликация снова работает",
    remote: "удалённый сервер",
    backlog: "ждут {pending}, в работе {leased}, мёртвых {dead}",
    suspended: "удалённый сервер приостановлен после сбоев",
    error: "последняя ошибка",
};

fn replication_texts(channel: &Channel) -> &'static ReplicationTexts {
    if channel.language == "en" {
        &REPLICATION_EN
    } else {
        &REPLICATION_RU
    }
}

fn replication_title(channel: &Channel, event: &str) -> &'static str {
    let texts = replication_texts(channel);
    match event {
        REPLICATION_RECOVERED => texts.recovered,
        REPLICATION_DEAD_ITEMS => texts.dead_items,
        _ => texts.failing,
    }
}

fn hide_userinfo(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("://") {
        let (head, tail) = rest.split_at(at + 3);
        out.push_str(head);
        let end = tail
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '/' | '?' | '#' | '"' | '\'' | '<' | '>')
            })
            .unwrap_or(tail.len());
        let authority = &tail[..end];
        match authority.rfind('@') {
            Some(at) => out.push_str(&authority[at + 1..]),
            None => out.push_str(authority),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

const TOKEN_PARAMETER: &str = "token=";

fn hide_query_tokens(word: &str) -> String {
    let lowered = word.to_ascii_lowercase();
    let mut out = String::with_capacity(word.len());
    let mut from = 0;
    while let Some(found) = lowered[from..].find(TOKEN_PARAMETER) {
        let start = from + found;
        let value = start + TOKEN_PARAMETER.len();
        let bounded = word[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric());
        out.push_str(&word[from..value]);
        from = value;
        if bounded {
            let end = word[value..]
                .find(['&', '#', '"', '\'', '<', '>'])
                .map_or(word.len(), |at| value + at);
            out.push_str("[hidden]");
            from = end;
        }
    }
    out.push_str(&word[from..]);
    out
}

fn takes_secret(word: &str, previous: &str) -> bool {
    let word = word.to_ascii_lowercase();
    let after_header = previous
        .trim_end_matches(':')
        .eq_ignore_ascii_case("authorization");
    word == "bearer" || (after_header && matches!(word.as_str(), "basic" | "token"))
}

pub fn excerpt(text: &str) -> String {
    let mut hidden: Vec<String> = Vec::new();
    let mut after_scheme = false;
    let mut previous = "";
    for word in text.split_whitespace() {
        hidden.push(match after_scheme {
            true => "[hidden]".to_owned(),
            false => hide_query_tokens(&hide_userinfo(word)),
        });
        after_scheme = takes_secret(word, previous);
        previous = word;
    }
    hidden.join(" ").chars().take(ERROR_EXCERPT).collect()
}

pub fn render_replication(channel: &Channel, event: &str, report: &ReplicationReport) -> String {
    let markup = Markup::of(channel);
    let texts = replication_texts(channel);
    let recovered = event == REPLICATION_RECOVERED;
    let mut text = format!(
        "{} {}\n• {}\n• {}: {}",
        if recovered { "✅" } else { "⚠️" },
        markup.bold(replication_title(channel, event)),
        markup.code(&report.replication),
        markup.escape(texts.remote),
        markup.code(&report.remote),
    );
    if recovered {
        return text;
    }
    let backlog = texts
        .backlog
        .replace("{pending}", &report.pending.to_string())
        .replace("{leased}", &report.leased.to_string())
        .replace("{dead}", &report.dead.to_string());
    text.push_str(&format!("\n• {}", markup.escape(&backlog)));
    if report.suspended {
        text.push_str(&format!("\n• {}", markup.escape(texts.suspended)));
    }
    if let Some(error) = report.last_error.as_deref().map(excerpt)
        && !error.is_empty()
    {
        text.push_str(&format!(
            "\n• {}: {}",
            markup.escape(texts.error),
            markup.escape(&error)
        ));
    }
    text
}

pub async fn queue(
    db: &impl ConnectionTrait,
    channel: &Channel,
    event: &str,
    text: String,
) -> Result<(), DbErr> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO notification_delivery (channel_id, delivery_id, event, text, written_at) \
         VALUES ($1, $2, $3, $4, now())",
        [
            channel.id.into(),
            Uuid::new_v4().into(),
            event.into(),
            text.into(),
        ],
    ))
    .await?;
    Ok(())
}

impl Notifier {
    pub async fn enqueue(
        &self,
        db: &impl ConnectionTrait,
        channel: &Channel,
        event: &str,
        text: String,
    ) -> Result<(), DbErr> {
        queue(db, channel, event, text).await?;
        self.wake();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::ChannelType;
    use crate::channel::tests::channel;
    use crate::messages::{Rendered, subject};

    fn findings() -> Vec<Finding> {
        vec![
            Finding {
                store: "default".into(),
                problem: Problem::LowSpace {
                    available: "8.21 GB".into(),
                    minimum: "10.00 GB".into(),
                },
            },
            Finding {
                store: "far<away>".into(),
                problem: Problem::Unavailable,
            },
        ]
    }

    fn repository_findings() -> Vec<Finding> {
        vec![
            Finding {
                store: "docker-prod".into(),
                problem: Problem::RepositoryWarn {
                    percent: 92,
                    used: "9.20 GB".into(),
                    limit: "10.00 GB".into(),
                },
            },
            Finding {
                store: "raw".into(),
                problem: Problem::RepositoryLimit {
                    used: "5.00 GB".into(),
                    limit: "5.00 GB".into(),
                },
            },
            Finding {
                store: "default".into(),
                problem: Problem::Forecast { days: 9 },
            },
        ]
    }

    #[test]
    fn alerts_follow_the_language_and_markup_of_the_channel() {
        let telegram = channel(ChannelType::Telegram, "en");
        assert_eq!(
            render(
                &telegram,
                ALERT,
                &findings(),
                Some("https://repo.example.com/")
            ),
            "⚠️ <b>Blob stores need attention</b>\n\
             • <code>default</code>: 8.21 GB free, less than 10.00 GB\n\
             • <code>far&lt;away&gt;</code>: cannot be reached\n\
             <a href=\"https://repo.example.com/admin/repository/blobstores\">Open the blob stores</a>"
        );
        let slack = channel(ChannelType::Slack, "ru");
        assert_eq!(
            render(&slack, RECOVERED, &findings(), None),
            "✅ *С blob store снова всё в порядке*\n\
             • `default`: снова свободно 8.21 GB\n\
             • `far&lt;away&gt;`: снова доступен"
        );
    }

    #[test]
    fn repository_and_forecast_problems_name_the_numbers() {
        let telegram = channel(ChannelType::Telegram, "en");
        assert_eq!(
            render(&telegram, ALERT, &repository_findings(), None),
            "⚠️ <b>Blob stores need attention</b>\n\
             • <code>docker-prod</code>: repository is at 92% of its 10.00 GB quota (9.20 GB used)\n\
             • <code>raw</code>: repository reached its 5.00 GB quota (5.00 GB used)\n\
             • <code>default</code>: will be full in 9 days at the current growth"
        );
        let slack = channel(ChannelType::Slack, "en");
        assert_eq!(
            render(&slack, RECOVERED, &repository_findings(), None),
            "✅ *Blob stores are fine again*\n\
             • `docker-prod`: repository is below its quota warning again\n\
             • `raw`: repository is below its quota warning again\n\
             • `default`: is no longer forecast to fill up soon"
        );
    }

    #[test]
    fn emails_are_titled_by_the_alert() {
        let email = channel(ChannelType::Email, "ru");
        let rendered = Rendered {
            event: ALERT.into(),
            text: render(&email, ALERT, &findings(), None),
            notice: None,
        };
        assert_eq!(subject(&email, &rendered), "Blob store требуют внимания");
        assert!(
            rendered
                .text
                .starts_with("⚠️ Blob store требуют внимания\n• default: ")
        );
    }

    fn report() -> ReplicationReport {
        ReplicationReport {
            replication: "eu<mirror>".into(),
            remote: "dr-site".into(),
            pending: 5,
            leased: 1,
            dead: 2,
            suspended: true,
            last_error: Some(
                "temporary failure: https://user:secret@remote.example/path\nAuthorization: Bearer abc123".into(),
            ),
        }
    }

    #[test]
    fn replication_alerts_name_the_state_counts_and_a_clean_error() {
        let telegram = channel(ChannelType::Telegram, "en");
        assert_eq!(
            render_replication(&telegram, REPLICATION_FAILING, &report()),
            "⚠️ <b>Replication is failing</b>\n\
             • <code>eu&lt;mirror&gt;</code>\n\
             • remote: <code>dr-site</code>\n\
             • 5 waiting, 1 in progress, 2 dead\n\
             • the remote is paused after failures\n\
             • last error: temporary failure: https://remote.example/path Authorization: Bearer [hidden]"
        );
        let slack = channel(ChannelType::Slack, "ru");
        assert_eq!(
            render_replication(&slack, REPLICATION_RECOVERED, &report()),
            "✅ *Репликация снова работает*\n• `eu&lt;mirror&gt;`\n• удалённый сервер: `dr-site`"
        );
        assert!(is_alert(REPLICATION_DEAD_ITEMS));
        assert_eq!(
            title(&telegram, REPLICATION_DEAD_ITEMS),
            "Replication has dead items"
        );
    }

    #[test]
    fn an_error_excerpt_is_cut_after_the_secrets_are_hidden() {
        let long = format!("https://a:b@host/{}", "x".repeat(400));
        let cut = excerpt(&long);
        assert_eq!(cut.chars().count(), 200);
        assert!(cut.starts_with("https://host/"));
        assert_eq!(excerpt("a\n\tb  c"), "a b c");
    }

    #[test]
    fn token_parameters_and_authorization_values_are_hidden() {
        assert_eq!(
            excerpt("GET https://h/x?a=1&Access_Token=abc123&b=2 failed, token=zzz."),
            "GET https://h/x?a=1&Access_Token=[hidden]&b=2 failed, token=[hidden]"
        );
        assert_eq!(
            excerpt("sent Authorization: BASIC dXNlcjpwdw== and authorization: token ghp_x end"),
            "sent Authorization: BASIC [hidden] and authorization: token [hidden] end"
        );
        assert_eq!(
            excerpt("Authorization: bearer abc; mytoken=keep xtoken=a the token refused"),
            "Authorization: bearer [hidden] mytoken=keep xtoken=a the token refused"
        );
    }
}
