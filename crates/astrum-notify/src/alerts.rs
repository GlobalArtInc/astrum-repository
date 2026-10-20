use sea_orm::{ConnectionTrait, DbBackend, DbErr, Statement};
use uuid::Uuid;

use crate::Notifier;
use crate::channel::Channel;
use crate::messages::Markup;

pub const ALERT: &str = "blobstore.alert";
pub const RECOVERED: &str = "blobstore.recovered";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    LowSpace { available: String, minimum: String },
    Quota { message: String },
    Unavailable,
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
