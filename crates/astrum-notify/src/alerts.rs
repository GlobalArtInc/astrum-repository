use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use uuid::Uuid;

use crate::Notifier;
use crate::channel::Channel;
use crate::messages::Markup;

pub const ALERT: &str = "blobstore.alert";
pub const RECOVERED: &str = "blobstore.recovered";

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
    event == ALERT || event == RECOVERED
}

pub fn title(channel: &Channel, event: &str) -> &'static str {
    let texts = texts(channel);
    match event {
        RECOVERED => texts.recovered,
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

impl Notifier {
    pub async fn enqueue(
        &self,
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
}
