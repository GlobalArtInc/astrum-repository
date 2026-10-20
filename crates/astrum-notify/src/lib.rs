pub mod alerts;
pub mod channel;
mod entity;
pub mod mail;
pub mod messages;
pub mod notice;
pub mod payload;
pub mod store;
pub mod transport;
mod worker;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use astrum_db::setting;
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, SSRF_KEY, SsrfProtection};
use sea_orm::DatabaseConnection;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

pub use channel::{Channel, ChannelType, Draft, Invalid, Secrets};
pub use mail::{Mail, MailError, MailSender};
pub use transport::DeliveryError;

pub const DEFAULT_TELEGRAM_API: &str = "https://api.telegram.org";
pub const TELEGRAM_API_ENV: &str = "ASTRUM_NOTIFICATIONS_TELEGRAM_API_URL";
pub const POLL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing {
    pub quiet: Duration,
    pub max_delay: Duration,
    pub backoff: Duration,
    pub attempts: u32,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            quiet: Duration::from_secs(10),
            max_delay: Duration::from_secs(60),
            backoff: Duration::from_secs(2),
            attempts: 4,
        }
    }
}

#[derive(Clone)]
pub struct Notifier {
    inner: Arc<Inner>,
}

struct Inner {
    db: DatabaseConnection,
    secrets: SecretBox,
    telegram_api: RwLock<String>,
    mail: RwLock<Option<Arc<dyn MailSender>>>,
    timing: RwLock<Timing>,
    wake: Notify,
    stopping: AtomicBool,
}

impl std::fmt::Debug for Notifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Notifier")
    }
}

impl Notifier {
    pub fn new(db: DatabaseConnection, secrets: SecretBox) -> Self {
        let telegram_api = std::env::var(TELEGRAM_API_ENV)
            .ok()
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_TELEGRAM_API.to_owned());
        Self {
            inner: Arc::new(Inner {
                db,
                secrets,
                telegram_api: RwLock::new(telegram_api),
                mail: RwLock::new(None),
                timing: RwLock::new(Timing::default()),
                wake: Notify::new(),
                stopping: AtomicBool::new(false),
            }),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn secrets(&self) -> &SecretBox {
        &self.inner.secrets
    }

    pub fn set_mail_sender(&self, sender: Arc<dyn MailSender>) {
        *self.inner.mail.write().expect("never poisoned") = Some(sender);
    }

    pub fn set_telegram_api(&self, url: impl Into<String>) {
        *self.inner.telegram_api.write().expect("never poisoned") = url.into();
    }

    pub fn set_timing(&self, timing: Timing) {
        *self.inner.timing.write().expect("never poisoned") = timing;
    }

    pub fn timing(&self) -> Timing {
        *self.inner.timing.read().expect("never poisoned")
    }

    fn mail(&self) -> Option<Arc<dyn MailSender>> {
        self.inner.mail.read().expect("never poisoned").clone()
    }

    fn telegram_api(&self) -> String {
        self.inner
            .telegram_api
            .read()
            .expect("never poisoned")
            .clone()
    }

    pub async fn base_url(&self) -> Option<String> {
        match setting::get::<BaseUrl>(self.db(), BASE_URL_KEY).await {
            Ok(Some(base)) if base.enabled && !base.url.trim().is_empty() => {
                Some(base.url.trim_end_matches('/').to_owned())
            }
            Ok(_) => None,
            Err(error) => {
                tracing::warn!(%error, "the base URL could not be read");
                None
            }
        }
    }

    async fn guard(&self) -> Arc<Guard> {
        let settings = setting::get::<SsrfProtection>(self.db(), SSRF_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        Arc::new(Guard::new(&settings))
    }

    pub async fn send_test(
        &self,
        channel: &Channel,
        secrets: &Secrets,
        base_url: Option<&str>,
    ) -> Result<(), DeliveryError> {
        let message = payload::Message {
            event: messages::TEST.to_owned(),
            delivery_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            text: messages::test(channel, base_url),
            envelope: None,
        };
        let api = self.telegram_api();
        let transport = transport::Transport {
            telegram_api: &api,
            guard: self.guard().await,
            mail: self.mail(),
        };
        transport.send(channel, secrets, &message).await
    }

    pub fn wake(&self) {
        self.inner.wake.notify_one();
    }

    pub fn spawn(&self) -> JoinHandle<()> {
        let notifier = self.clone();
        tokio::spawn(async move {
            loop {
                if notifier.inner.stopping.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(error) = notifier.tick().await {
                    tracing::warn!(%error, "notifications could not be processed");
                }
                tokio::select! {
                    () = tokio::time::sleep(POLL) => {}
                    () = notifier.inner.wake.notified() => {}
                }
            }
        })
    }

    pub fn stop(&self) {
        self.inner.stopping.store(true, Ordering::Relaxed);
        self.inner.wake.notify_one();
    }
}
