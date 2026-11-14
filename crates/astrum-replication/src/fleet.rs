use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use sea_orm::DbErr;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::dispatch::Dispatcher;
use crate::queue;
use crate::target::Target;

struct Task {
    stop: CancellationToken,
    handle: JoinHandle<()>,
}

#[derive(Default)]
struct Fleet {
    tasks: HashMap<Uuid, Task>,
}

impl Fleet {
    fn running(&self, remote: Uuid) -> bool {
        self.tasks
            .get(&remote)
            .is_some_and(|task| !task.handle.is_finished())
    }

    fn reap(&mut self) {
        self.tasks.retain(|_, task| !task.handle.is_finished());
    }

    fn start(&mut self, remote: Uuid, stop: CancellationToken, handle: JoinHandle<()>) {
        self.tasks.insert(remote, Task { stop, handle });
    }

    async fn stop(&mut self, grace: Duration) {
        let mut handles = Vec::new();
        for (_, task) in self.tasks.drain() {
            task.stop.cancel();
            handles.push(task.handle);
        }
        let finished = async {
            for handle in &mut handles {
                let _ = handle.await;
            }
        };
        if tokio::time::timeout(grace, finished).await.is_err() {
            tracing::warn!("replication workers did not finish in time and were aborted");
        }
        for handle in &handles {
            handle.abort();
        }
    }
}

impl Drop for Fleet {
    fn drop(&mut self) {
        for task in self.tasks.values() {
            task.stop.cancel();
            task.handle.abort();
        }
    }
}

impl Dispatcher {
    pub(crate) async fn lead(&self, stop: CancellationToken) {
        let mut fleet = Fleet::default();
        let mut timer = tokio::time::interval(self.settings().discover);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                () = stop.cancelled() => break,
                _ = timer.tick() => {}
            }
            if let Err(error) = self.discover(&mut fleet, &stop).await {
                tracing::warn!(%error, "replication remotes could not be looked up");
            }
        }
        fleet.stop(self.settings().grace).await;
    }

    async fn discover(&self, fleet: &mut Fleet, stop: &CancellationToken) -> Result<(), DbErr> {
        let db = &self.db();
        queue::requeue_expired(db).await?;
        fleet.reap();
        for remote in queue::remotes_due(db, self.settings().max_remotes).await? {
            if fleet.running(remote.id) {
                continue;
            }
            let child = stop.child_token();
            let handle = tokio::spawn(self.clone().keep(remote.id, child.clone()));
            fleet.start(remote.id, child, handle);
        }
        Ok(())
    }

    async fn keep(self, remote: Uuid, stop: CancellationToken) {
        let db = &self.db();
        while !stop.is_cancelled() {
            let row = match queue::remote(db, remote).await {
                Ok(Some(row)) => row,
                Ok(None) => return,
                Err(error) => {
                    tracing::warn!(%error, "a replication remote could not be read");
                    self.nap(&stop, self.settings().idle).await;
                    continue;
                }
            };
            if let Some(seconds) = row.suspended_for {
                let wait = Duration::from_secs_f64(seconds.max(0.0)).min(self.settings().discover);
                self.nap(&stop, wait).await;
                continue;
            }
            let workers = Self::workers_of(&row);
            let target = Arc::new(self.target(row).await);
            let mut pool = JoinSet::new();
            for _ in 0..workers {
                pool.spawn(self.clone().loiter(target.clone(), stop.clone()));
            }
            while let Some(joined) = pool.join_next_with_id().await {
                if let Err(error) = joined {
                    self.crashed(&target, error).await;
                }
            }
            self.nap(&stop, self.settings().tick).await;
        }
    }

    async fn loiter(self, target: Arc<Target>, stop: CancellationToken) {
        let settings = self.settings();
        let mut rest = settings.tick;
        while !stop.is_cancelled() && !target.halted() {
            let pass = self.drain(&target, &stop).await;
            if pass.leased > 0 {
                rest = settings.tick;
                continue;
            }
            self.nap(&stop, rest).await;
            rest = grow(rest, settings.idle_max);
        }
    }
}

fn grow(rest: Duration, ceiling: Duration) -> Duration {
    (rest * 2).min(ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_idle_worker_waits_longer_each_time_up_to_the_ceiling() {
        let ceiling = Duration::from_secs(30);
        let mut rest = Duration::from_secs(5);
        let mut seen = Vec::new();
        for _ in 0..5 {
            seen.push(rest.as_secs());
            rest = grow(rest, ceiling);
        }
        assert_eq!(seen, [5, 10, 20, 30, 30]);
    }
}
