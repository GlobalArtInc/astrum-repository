use std::collections::VecDeque;
use std::time::Duration;

use astrum_db::setting;
use astrum_security::ssrf::{Guard, SSRF_KEY, SsrfProtection};
use sea_orm::DbErr;
use tokio_util::sync::CancellationToken;

use crate::dispatch::{Dispatcher, Pass};
use crate::error::{Error, backoff, clean};
use crate::protocol::component_key;
use crate::queue::{self, Finished, Item, RemoteRow};
use crate::target::Target;
use crate::work::{Done, Failure};
use crate::{Client, Remote};

const TOKEN_PAUSE: Duration = Duration::from_secs(15 * 60);

impl Dispatcher {
    pub(crate) async fn target(&self, remote: RemoteRow) -> Target {
        let client = self
            .client_for(&remote)
            .await
            .map_err(|failure| failure.text());
        Target::new(remote, client)
    }

    async fn client_for(&self, remote: &RemoteRow) -> Result<Client, Failure> {
        let content = self.content();
        let token = content.secrets().open(&remote.token).map_err(|_| {
            Failure::Local(
                "the stored token cannot be opened with the key of this server".to_owned(),
            )
        })?;
        let snapshot = content
            .outbound()
            .snapshot()
            .await
            .map_err(|error| Failure::Local(error.to_string()))?;
        let ssrf: SsrfProtection = setting::get(content.db(), SSRF_KEY)
            .await
            .map_err(|error| Failure::Local(error.to_string()))?
            .unwrap_or_default();
        let guard = Guard::new(&snapshot.guard_settings(&ssrf));
        let settings = Remote {
            url: remote.url.clone(),
            use_trust_store: remote.use_trust_store,
            allow_private_network: remote.allow_private_network,
            max_concurrency: Self::workers_of(remote),
            max_bytes_per_second: remote
                .max_bytes_per_second
                .and_then(|rate| u64::try_from(rate).ok()),
        };
        let own = self.instance().await?;
        let client = Client::new(&settings, &token, &snapshot, content.secrets(), &guard)
            .and_then(|client| client.with_instance(&own.to_string()))
            .map_err(Failure::Remote)?;
        Ok(client)
    }

    pub(crate) async fn drain(&self, target: &Target, stop: &CancellationToken) -> Pass {
        let mut pass = Pass::default();
        let permit = tokio::select! {
            () = stop.cancelled() => return pass,
            permit = self.inner.gate.acquire() => permit,
        };
        let Ok(_permit) = permit else {
            return pass;
        };
        if let Err(error) = self.batch(target, stop, &mut pass).await {
            tracing::warn!(%error, "a replication worker stopped on a database error");
        }
        pass
    }

    async fn batch(
        &self,
        target: &Target,
        stop: &CancellationToken,
        pass: &mut Pass,
    ) -> Result<(), DbErr> {
        let db = self.content().db();
        let settings = self.settings();
        if stop.is_cancelled() || !target.fresh(db, settings.fresh).await? {
            return Ok(());
        }
        let workers = i64::try_from(Self::workers_of(&target.remote)).unwrap_or(1);
        let batch = i64::try_from(settings.batch).unwrap_or(1);
        let leased =
            queue::lease_share(db, target.remote.id, settings.lease, batch, workers).await?;
        pass.leased += u32::try_from(leased.len()).unwrap_or(u32::MAX);
        let mut rest: VecDeque<Item> = leased.into();
        while let Some(item) = rest.pop_front() {
            let waiting: Vec<i64> = rest.iter().map(|waiting| waiting.id).collect();
            if stop.is_cancelled() || !target.fresh(db, settings.fresh).await? {
                return self.give_back(&item, &waiting, pass).await;
            }
            queue::renew_many(db, &waiting, settings.lease).await?;
            let verdict = tokio::select! {
                biased;
                () = stop.cancelled() => None,
                verdict = self.transfer(target, &item) => Some(verdict),
            };
            let Some(verdict) = verdict else {
                return self.give_back(&item, &waiting, pass).await;
            };
            match verdict {
                Ok(Done::Lost) => pass.abandoned += 1,
                Ok(Done::Released) => {
                    queue::release(db, item.id).await?;
                    pass.released += 1;
                    if target.halted() {
                        queue::release_many(db, &waiting).await?;
                        return Ok(());
                    }
                }
                Ok(done) => self.complete(target, &item, done, pass).await?,
                Err(Failure::Down(error)) => {
                    self.suspend(target, &item, &error, pass).await?;
                    queue::release_many(db, &waiting).await?;
                    return Ok(());
                }
                Err(failure) => self.abandon(&item, failure, pass).await?,
            }
        }
        Ok(())
    }

    async fn give_back(&self, item: &Item, waiting: &[i64], pass: &mut Pass) -> Result<(), DbErr> {
        let mut ids = vec![item.id];
        ids.extend_from_slice(waiting);
        queue::release_many(self.content().db(), &ids).await?;
        pass.released += u32::try_from(ids.len()).unwrap_or(u32::MAX);
        Ok(())
    }

    async fn complete(
        &self,
        target: &Target,
        item: &Item,
        done: Done,
        pass: &mut Pass,
    ) -> Result<(), DbErr> {
        let db = self.content().db();
        match done {
            Done::Sent | Done::Deleted => {
                pass.sent += 1;
                queue::succeeded(db, item.replication_id).await?;
                queue::remote_ok(db, target.remote.id).await?;
            }
            Done::Dropped(reason) => {
                pass.dropped += 1;
                tracing::debug!(replication = %item.replication_id, reason, "a replication item was dropped");
            }
            Done::Lost | Done::Released => {}
        }
        if queue::finish(db, item).await? == Finished::Again {
            pass.again += 1;
        }
        Ok(())
    }

    async fn suspend(
        &self,
        target: &Target,
        item: &Item,
        error: &Error,
        pass: &mut Pass,
    ) -> Result<(), DbErr> {
        let db = self.content().db();
        let refused = error.status().filter(|status| matches!(status, 401 | 403));
        let failures = u32::try_from(target.remote.failures).unwrap_or(0);
        let (delay, text) = match refused {
            Some(status) => (
                TOKEN_PAUSE,
                format!("the remote refused the token (status {status}); enter a valid token"),
            ),
            None => (
                backoff(failures).max(error.retry_after().unwrap_or_default()),
                clean(&error.to_string()),
            ),
        };
        queue::defer(db, item.id, delay).await?;
        queue::suspend(db, target.remote.id, delay, &text).await?;
        queue::failed(db, item.replication_id, &text).await?;
        target.halt();
        pass.suspended += 1;
        Ok(())
    }

    async fn abandon(&self, item: &Item, failure: Failure, pass: &mut Pass) -> Result<(), DbErr> {
        let db = self.content().db();
        let attempts = u32::try_from(item.attempts).unwrap_or(0);
        let delay = failure
            .retry_after()
            .map_or_else(|| backoff(attempts), |after| backoff(attempts).max(after));
        let text = failure.text();
        let failed = queue::fail(db, item, failure.permanent(), delay, &text).await?;
        queue::failed(db, item.replication_id, &text).await?;
        pass.failed += 1;
        if failed.is_some_and(|failed| failed.state == "dead") {
            pass.dead += 1;
            tracing::warn!(
                replication = %item.replication_id,
                component = %component_key(&item.namespace, &item.name, &item.version),
                error = %text,
                "a replication item is dead"
            );
        }
        Ok(())
    }
}
