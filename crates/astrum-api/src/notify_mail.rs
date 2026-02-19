use astrum_mail::Mailer;
use astrum_notify::{Mail, MailError, MailSender};
use async_trait::async_trait;

const NOT_SET_UP: &str = "Email is not set up on this server: configure the email server first";

pub(crate) struct NotificationMail(pub(crate) Mailer);

#[async_trait]
impl MailSender for NotificationMail {
    async fn send(&self, mail: Mail) -> Result<(), MailError> {
        let message = astrum_mail::Message {
            to: mail.to,
            subject: mail.subject,
            text: mail.text,
        };
        self.0.send(&message).await.map_err(|error| {
            let retryable = !matches!(
                error,
                astrum_mail::MailError::Disabled | astrum_mail::MailError::Invalid(_)
            );
            let message = match error {
                astrum_mail::MailError::Disabled => NOT_SET_UP.to_owned(),
                other => other.to_string(),
            };
            MailError { message, retryable }
        })
    }
}
