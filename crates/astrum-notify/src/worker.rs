use std::collections::HashMap;
use std::time::Duration;

use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Postgres};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, FromQueryResult, Statement, TransactionTrait};
use uuid::Uuid;

use crate::channel::Channel;
use crate::messages::{self, Links};
use crate::notice::{self, Action, Notice};
use crate::payload::{Envelope, Message};
use crate::transport::Transport;
use crate::{Notifier, store};

pub const LOCK_KEY: i64 = i64::from_be_bytes(*b"astrntfy");
pub const BUFFER_CAPACITY: usize = 500;
const DELIVERY_BATCH: u64 = 100;

struct Lock {
    connection: PoolConnection<Postgres>,
}

impl Lock {
    async fn try_take(notifier: &Notifier) -> Result<Option<Self>, DbErr> {
        let mut connection = notifier
            .db()
            .get_postgres_connection_pool()
            .acquire()
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        connection.close_on_drop();
        let (taken,): (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock($1)")
            .bind(LOCK_KEY)
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| DbErr::Custom(error.to_string()))?;
        Ok(taken.then_some(Self { connection }))
    }

    async fn release(mut self) {
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(LOCK_KEY)
            .execute(&mut *self.connection)
            .await;
    }
}

#[derive(FromQueryResult)]
struct Ready {
    channel_id: Uuid,
    last: i64,
}

#[derive(FromQueryResult)]
struct ChangeRow {
    repository: String,
    repository_type: String,
    format: String,
    namespace: String,
    name: String,
    version: String,
    action: String,
    component_id: Option<i64>,
    initiator: Option<String>,
}

#[derive(FromQueryResult)]
struct DeliveryRow {
    id: i64,
    channel_id: Uuid,
    delivery_id: Uuid,
    event: String,
    text: String,
    notice: Option<serde_json::Value>,
    written_at: chrono::DateTime<chrono::FixedOffset>,
    attempts: i32,
}

fn seconds(duration: Duration) -> f64 {
    duration.as_secs_f64()
}

impl Notifier {
    pub async fn tick(&self) -> Result<(), DbErr> {
        let Some(lock) = Lock::try_take(self).await? else {
            return Ok(());
        };
        let result = async {
            self.flush(false).await?;
            self.deliver().await
        }
        .await;
        lock.release().await;
        result
    }

    pub async fn flush_now(&self) -> Result<(), DbErr> {
        let Some(lock) = Lock::try_take(self).await? else {
            return Ok(());
        };
        let result = async {
            self.flush(true).await?;
            self.deliver().await
        }
        .await;
        lock.release().await;
        result
    }

    async fn flush(&self, everything: bool) -> Result<(), DbErr> {
        let timing = self.timing();
        let ready = Ready::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT channel_id, max(id) AS last FROM notification_change GROUP BY channel_id \
             HAVING $1 OR max(at) <= now() - make_interval(secs => $2) \
             OR min(at) <= now() - make_interval(secs => $3)",
            [
                everything.into(),
                seconds(timing.quiet).into(),
                seconds(timing.max_delay).into(),
            ],
        ))
        .all(self.db())
        .await?;
        if ready.is_empty() {
            return Ok(());
        }
        let links = Links {
            base_url: self.base_url().await,
        };
        for window in ready {
            self.flush_channel(window.channel_id, window.last, &links)
                .await?;
        }
        Ok(())
    }

    async fn flush_channel(&self, channel_id: Uuid, last: i64, links: &Links) -> Result<(), DbErr> {
        let txn = self.db().begin().await?;
        let rows = ChangeRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "WITH taken AS (DELETE FROM notification_change WHERE channel_id = $1 AND id <= $2 \
             RETURNING *) SELECT * FROM taken ORDER BY id",
            [channel_id.into(), last.into()],
        ))
        .all(&txn)
        .await?;
        let channel = store::find(&txn, channel_id)
            .await?
            .filter(|channel| channel.enabled);
        if let Some(channel) = channel {
            let changes = rows.into_iter().filter_map(|row| {
                Some(Notice {
                    action: Action::parse(&row.action)?,
                    repository: row.repository,
                    repository_type: row.repository_type,
                    format: row.format,
                    namespace: row.namespace,
                    name: row.name,
                    version: row.version,
                    initiator: row.initiator,
                    component_id: row.component_id,
                })
            });
            let batch = notice::combine(changes, BUFFER_CAPACITY);
            let wanted: Vec<Notice> = batch
                .notices
                .into_iter()
                .filter(|notice| channel.wants(notice.action))
                .collect();
            if !wanted.is_empty() {
                if batch.overflow > 0 && channel.kind == crate::ChannelType::Webhook {
                    tracing::warn!(
                        channel = %channel.name,
                        dropped = batch.overflow,
                        "component changes beyond the window's capacity were not sent"
                    );
                }
                for rendered in messages::render(&channel, links, &wanted, batch.overflow) {
                    let envelope = rendered
                        .notice
                        .map(|notice| Envelope::new(notice, links))
                        .map(|envelope| serde_json::to_value(envelope).expect("serializable"));
                    txn.execute(Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        "INSERT INTO notification_delivery \
                         (channel_id, delivery_id, event, text, notice, written_at) \
                         VALUES ($1, $2, $3, $4, $5, now())",
                        [
                            channel_id.into(),
                            Uuid::new_v4().into(),
                            rendered.event.into(),
                            rendered.text.into(),
                            envelope.into(),
                        ],
                    ))
                    .await?;
                }
            }
        }
        txn.commit().await
    }

    async fn deliver(&self) -> Result<(), DbErr> {
        let due = DeliveryRow::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, channel_id, delivery_id, event, text, notice, written_at, attempts \
             FROM notification_delivery WHERE next_attempt_at <= now() ORDER BY id LIMIT $1",
            [DELIVERY_BATCH.into()],
        ))
        .all(self.db())
        .await?;
        if due.is_empty() {
            return Ok(());
        }
        let api = self.telegram_api();
        let transport = Transport {
            telegram_api: &api,
            guard: self.guard().await,
            mail: self.mail(),
        };
        let timing = self.timing();
        let mut channels: HashMap<Uuid, Option<Channel>> = HashMap::new();
        for row in due {
            if let std::collections::hash_map::Entry::Vacant(slot) = channels.entry(row.channel_id)
            {
                let channel = store::find(self.db(), row.channel_id)
                    .await?
                    .filter(|channel| channel.enabled);
                slot.insert(channel);
            }
            let Some(channel) = channels.get(&row.channel_id).cloned().flatten() else {
                self.forget(row.id).await?;
                continue;
            };
            let secrets = match channel.open(self.secrets()) {
                Ok(secrets) => secrets,
                Err(error) => {
                    tracing::warn!(channel = %channel.name, %error, "the secrets of a notification channel cannot be read");
                    self.forget(row.id).await?;
                    continue;
                }
            };
            let message = Message {
                event: row.event,
                delivery_id: row.delivery_id.to_string(),
                timestamp: row.written_at.to_utc(),
                text: row.text,
                envelope: row
                    .notice
                    .and_then(|value| serde_json::from_value::<Envelope>(value).ok()),
            };
            let attempt = u32::try_from(row.attempts).unwrap_or(0) + 1;
            match transport.send(&channel, &secrets, &message).await {
                Ok(()) => {
                    tracing::debug!(channel = %channel.name, "a notification was sent");
                    self.forget(row.id).await?;
                }
                Err(error) if error.retryable && attempt < timing.attempts => {
                    let delay = if error.retry_after > Duration::ZERO {
                        error.retry_after
                    } else {
                        timing.backoff * (1u32 << (attempt - 1).min(20))
                    };
                    tracing::info!(
                        channel = %channel.name,
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        error = %error.message,
                        "a notification will be sent again"
                    );
                    self.db()
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            "UPDATE notification_delivery SET attempts = $2, last_error = $3, \
                             next_attempt_at = now() + make_interval(secs => $4) WHERE id = $1",
                            [
                                row.id.into(),
                                (attempt as i32).into(),
                                error.message.into(),
                                seconds(delay).into(),
                            ],
                        ))
                        .await?;
                }
                Err(error) => {
                    tracing::warn!(
                        channel = %channel.name,
                        attempt,
                        error = %error.message,
                        "a notification could not be sent"
                    );
                    self.forget(row.id).await?;
                }
            }
        }
        Ok(())
    }

    async fn forget(&self, id: i64) -> Result<(), DbErr> {
        self.db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM notification_delivery WHERE id = $1",
                [id.into()],
            ))
            .await?;
        Ok(())
    }
}
