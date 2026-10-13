use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mail {
    pub to: Vec<String>,
    pub subject: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct MailError {
    pub message: String,
    pub retryable: bool,
}

#[async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, mail: Mail) -> Result<(), MailError>;
}
