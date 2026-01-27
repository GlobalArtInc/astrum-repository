pub mod settings;
pub mod smtp;

use astrum_db::setting;
use astrum_outbound::Outbound;

pub use settings::{EMAIL_KEY, EmailRequest, EmailSettings};

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("sending email is disabled")]
    Disabled,
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Connection(String),
    #[error("{0}")]
    Protocol(String),
    #[error("the TLS connection failed: {0}")]
    Tls(String),
    #[error("the email settings cannot be read: {0}")]
    Settings(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub to: Vec<String>,
    pub subject: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Mailer {
    outbound: Outbound,
}

impl Mailer {
    pub fn new(outbound: Outbound) -> Self {
        Self { outbound }
    }

    pub async fn settings(&self) -> Result<EmailSettings, MailError> {
        setting::get(self.outbound.db(), EMAIL_KEY)
            .await
            .map(Option::unwrap_or_default)
            .map_err(|error| MailError::Settings(error.to_string()))
    }

    pub async fn save(&self, settings: &EmailSettings) -> Result<(), MailError> {
        setting::put(self.outbound.db(), EMAIL_KEY, settings)
            .await
            .map_err(|error| MailError::Settings(error.to_string()))
    }

    pub async fn reset(&self) -> Result<(), MailError> {
        self.save(&EmailSettings::default()).await
    }

    pub async fn send(&self, message: &Message) -> Result<(), MailError> {
        let settings = self.settings().await?;
        if !settings.enabled {
            return Err(MailError::Disabled);
        }
        self.send_with(&settings, message).await
    }

    pub async fn send_with(
        &self,
        settings: &EmailSettings,
        message: &Message,
    ) -> Result<(), MailError> {
        if message.to.is_empty() {
            return Err(MailError::Invalid("a message needs a recipient".to_owned()));
        }
        if let Some(bad) = message
            .to
            .iter()
            .find(|address| !settings::valid_address(address))
        {
            return Err(MailError::Invalid(format!(
                "\"{bad}\" is not a well-formed email address"
            )));
        }
        let password = match &settings.password {
            Some(sealed) => Some(
                self.outbound
                    .secrets()
                    .open(sealed)
                    .map_err(|error| MailError::Settings(error.to_string()))?,
            ),
            None => None,
        };
        let trust = match settings.astrum_trust_store_enabled {
            true => self
                .outbound
                .trust_roots()
                .await
                .map_err(|error| MailError::Settings(error.to_string()))?,
            false => Vec::new(),
        };
        let outcome = smtp::deliver(settings, password.as_deref(), &trust, message).await;
        if let Err(error) = &outcome {
            tracing::warn!(%error, host = settings.host, "an email was not sent");
        }
        outcome
    }
}
