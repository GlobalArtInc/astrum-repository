use chrono::{DateTime, Utc};

use crate::channel::Channel;
use crate::messages::Markup;

pub const EXPIRING: &str = "customer.token.expiring";
pub const EXPIRED: &str = "customer.token.expired";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expiring {
    pub customer: String,
    pub customer_name: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub days_left: i64,
}

struct Texts {
    expiring: &'static str,
    expired: &'static str,
    open: &'static str,
}

const EN: Texts = Texts {
    expiring: "Customer tokens are about to expire",
    expired: "Customer tokens have expired",
    open: "Open the customers",
};

fn days_left(days: i64) -> String {
    match days {
        1 => "1 day left".to_owned(),
        _ => format!("{days} days left"),
    }
}

fn line(markup: Markup, token: &Expiring, expired: bool) -> String {
    let date = token.expires_at.format("%Y-%m-%d");
    let when = match expired {
        true => format!("expired {date}"),
        false => format!("expires {date}, {}", days_left(token.days_left)),
    };
    format!(
        "• {} ({}): {}, {when}",
        markup.escape(&token.customer_name),
        markup.code(&token.customer),
        markup.escape(&token.token)
    )
}

pub fn is_notice(event: &str) -> bool {
    event == EXPIRING || event == EXPIRED
}

pub fn title(event: &str) -> &'static str {
    match event {
        EXPIRED => EN.expired,
        _ => EN.expiring,
    }
}

pub fn render(
    channel: &Channel,
    event: &str,
    tokens: &[Expiring],
    base_url: Option<&str>,
) -> String {
    let markup = Markup::of(channel);
    let expired = event == EXPIRED;
    let mut text = markup.bold(title(event));
    for token in tokens {
        text.push('\n');
        text.push_str(&line(markup, token, expired));
    }
    if let Some(base) = base_url
        .map(|base| base.trim_end_matches('/'))
        .filter(|base| !base.is_empty())
    {
        text.push('\n');
        text.push_str(&markup.link(&format!("{base}/admin/customers"), EN.open));
    }
    text
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::channel::ChannelType;
    use crate::channel::tests::channel;
    use crate::messages::{Rendered, subject};

    #[test]
    fn emails_are_titled_by_the_notice() {
        let email = channel(ChannelType::Email, "en");
        for (event, title) in [
            (EXPIRING, "Customer tokens are about to expire"),
            (EXPIRED, "Customer tokens have expired"),
        ] {
            let rendered = Rendered {
                event: event.into(),
                text: String::new(),
                notice: None,
            };
            assert_eq!(subject(&email, &rendered), title);
        }
    }

    #[test]
    fn every_channel_type_names_the_customer_and_the_token() {
        let tokens = vec![Expiring {
            customer: "acme".into(),
            customer_name: "Acme".into(),
            token: "release".into(),
            expires_at: Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap(),
            days_left: 1,
        }];
        for kind in [
            ChannelType::Telegram,
            ChannelType::Slack,
            ChannelType::Webhook,
            ChannelType::Email,
        ] {
            let text = render(&channel(kind, "en"), EXPIRING, &tokens, None);
            assert!(text.contains("acme"), "{text}");
            assert!(text.contains("release"), "{text}");
        }
        assert_eq!(
            render(
                &channel(ChannelType::Telegram, "en"),
                EXPIRING,
                &tokens,
                Some("https://repo.example.com/")
            ),
            "<b>Customer tokens are about to expire</b>\n\
             • Acme (<code>acme</code>): release, expires 2026-12-01, 1 day left\n\
             <a href=\"https://repo.example.com/admin/customers\">Open the customers</a>"
        );
    }
}
