use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use regex::Regex;
use sea_orm::{ConnectionTrait, DbErr};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::Client;
use crate::error::Error;
use crate::filter::{compile, subject};
use crate::queue::{self, Item, RemoteRow, ReplicationRow};
use crate::work::Failure;

pub(crate) struct Compiled {
    name_pattern: Option<String>,
    version_pattern: Option<String>,
    name: Option<Regex>,
    version: Option<Regex>,
}

fn build(pattern: Option<&str>) -> Result<Option<Regex>, Failure> {
    pattern
        .map(|pattern| {
            compile(pattern).map_err(|reason| {
                Failure::Rejected(format!("a filter of the replication {reason}"))
            })
        })
        .transpose()
}

impl Compiled {
    pub(crate) fn of(replication: &ReplicationRow) -> Result<Self, Failure> {
        Ok(Self {
            name_pattern: replication.name_filter.clone(),
            version_pattern: replication.version_filter.clone(),
            name: build(replication.name_filter.as_deref())?,
            version: build(replication.version_filter.as_deref())?,
        })
    }

    fn current(&self, replication: &ReplicationRow) -> bool {
        self.name_pattern == replication.name_filter
            && self.version_pattern == replication.version_filter
    }

    pub(crate) fn passes(&self, item: &Item) -> bool {
        let named = self
            .name
            .as_ref()
            .is_none_or(|regex| regex.is_match(&subject(&item.namespace, &item.name)));
        named
            && self
                .version
                .as_ref()
                .is_none_or(|regex| regex.is_match(&item.version))
    }
}

pub(crate) struct Target {
    pub remote: RemoteRow,
    pub client: Result<Client, String>,
    pub learned: OnceCell<Uuid>,
    halted: AtomicBool,
    checked: Mutex<Option<Instant>>,
    filters: Mutex<HashMap<Uuid, Arc<Compiled>>>,
    holding: Mutex<HashMap<tokio::task::Id, Vec<i64>>>,
}

pub(crate) struct Held<'a> {
    target: &'a Target,
    task: Option<tokio::task::Id>,
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        if let (Some(task), false) = (self.task, std::thread::panicking()) {
            self.target
                .holding
                .lock()
                .expect("the holding lock is never poisoned")
                .remove(&task);
        }
    }
}

impl Target {
    pub(crate) fn new(
        remote: RemoteRow,
        client: Result<Client, String>,
        learned: OnceCell<Uuid>,
    ) -> Self {
        Self {
            remote,
            client,
            learned,
            halted: AtomicBool::new(false),
            checked: Mutex::new(None),
            filters: Mutex::new(HashMap::new()),
            holding: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn client(&self) -> Result<&Client, Failure> {
        self.client
            .as_ref()
            .map_err(|reason| Failure::Down(Error::Refused(reason.clone())))
    }

    pub(crate) fn hold(&self, ids: Vec<i64>) -> Held<'_> {
        let task = tokio::task::try_id();
        if let Some(task) = task {
            self.holding
                .lock()
                .expect("the holding lock is never poisoned")
                .insert(task, ids);
        }
        Held { target: self, task }
    }

    pub(crate) fn reclaim(&self, task: tokio::task::Id) -> Vec<i64> {
        self.holding
            .lock()
            .expect("the holding lock is never poisoned")
            .remove(&task)
            .unwrap_or_default()
    }

    pub(crate) fn halt(&self) {
        self.halted.store(true, Ordering::SeqCst);
    }

    pub(crate) fn halted(&self) -> bool {
        self.halted.load(Ordering::SeqCst)
    }

    pub(crate) async fn fresh(
        &self,
        db: &impl ConnectionTrait,
        within: Duration,
    ) -> Result<bool, DbErr> {
        if self.halted() {
            return Ok(false);
        }
        let recent = self
            .checked
            .lock()
            .expect("the freshness lock is never poisoned")
            .is_some_and(|at| at.elapsed() < within);
        if recent {
            return Ok(true);
        }
        let current = queue::remote(db, self.remote.id).await?;
        let same = current
            .is_some_and(|now| now.same_settings(&self.remote) && now.suspended_for.is_none());
        match same {
            true => {
                *self
                    .checked
                    .lock()
                    .expect("the freshness lock is never poisoned") = Some(Instant::now());
            }
            false => self.halt(),
        }
        Ok(same)
    }

    pub(crate) fn passes(
        &self,
        replication: &ReplicationRow,
        item: &Item,
    ) -> Result<bool, Failure> {
        let known = self
            .filters
            .lock()
            .expect("the filter lock is never poisoned")
            .get(&replication.id)
            .filter(|compiled| compiled.current(replication))
            .cloned();
        let compiled = match known {
            Some(compiled) => compiled,
            None => {
                let compiled = Arc::new(Compiled::of(replication)?);
                self.filters
                    .lock()
                    .expect("the filter lock is never poisoned")
                    .insert(replication.id, compiled.clone());
                compiled
            }
        };
        Ok(compiled.passes(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replication(name: Option<&str>, version: Option<&str>) -> ReplicationRow {
        ReplicationRow {
            id: Uuid::nil(),
            direction: "push".to_owned(),
            source_repository: "source".to_owned(),
            remote_id: Uuid::nil(),
            remote_repository: "mirror".to_owned(),
            enabled: true,
            name_filter: name.map(str::to_owned),
            version_filter: version.map(str::to_owned),
            propagate_deletes: false,
        }
    }

    fn item(namespace: &str, name: &str, version: &str) -> Item {
        Item {
            id: 1,
            replication_id: Uuid::nil(),
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
            generation: 1,
            attempts: 0,
        }
    }

    fn target() -> Target {
        let remote = RemoteRow {
            id: Uuid::nil(),
            url: "https://remote.example".to_owned(),
            token: "sealed".to_owned(),
            use_trust_store: false,
            allow_private_network: false,
            max_concurrency: 2,
            max_bytes_per_second: None,
            remote_instance_id: None,
            updated_at: "2026-01-01T00:00:00+00:00".parse().unwrap(),
            failures: 0,
            suspended_for: None,
        };
        Target::new(remote, Err("no client".to_owned()), OnceCell::new())
    }

    #[test]
    fn the_filters_match_the_whole_subject_and_the_whole_version() {
        let target = target();
        let wanted = replication(Some("com\\.acme/lib-.*"), Some("1\\..*"));
        assert!(
            target
                .passes(&wanted, &item("com.acme", "lib-core", "1.4"))
                .unwrap()
        );
        assert!(
            !target
                .passes(&wanted, &item("com.acme", "lib-core", "2.0"))
                .unwrap()
        );
        assert!(
            !target
                .passes(&wanted, &item("org.acme", "lib-core", "1.4"))
                .unwrap()
        );
        assert!(
            !target
                .passes(&wanted, &item("com.acme", "x-lib-core", "1.4"))
                .unwrap()
        );
        let raw = replication(Some("/docs/.*"), None);
        assert!(
            target
                .passes(&raw, &item("/docs", "/docs/a.txt", ""))
                .unwrap()
        );
        assert!(
            !target
                .passes(&raw, &item("/other", "/other/a.txt", ""))
                .unwrap()
        );
        assert!(
            target
                .passes(&replication(None, None), &item("", "any", ""))
                .unwrap()
        );
    }

    #[test]
    fn a_filter_is_compiled_once_until_the_replication_changes_it() {
        let target = target();
        let first = replication(Some("app"), None);
        assert!(target.passes(&first, &item("", "app", "1")).unwrap());
        let kept = target
            .filters
            .lock()
            .unwrap()
            .get(&Uuid::nil())
            .cloned()
            .unwrap();
        assert!(target.passes(&first, &item("", "app2", "1")).is_ok());
        let again = target
            .filters
            .lock()
            .unwrap()
            .get(&Uuid::nil())
            .cloned()
            .unwrap();
        assert!(Arc::ptr_eq(&kept, &again));
        let changed = replication(Some("app2"), None);
        assert!(target.passes(&changed, &item("", "app2", "1")).unwrap());
        let replaced = target
            .filters
            .lock()
            .unwrap()
            .get(&Uuid::nil())
            .cloned()
            .unwrap();
        assert!(!Arc::ptr_eq(&kept, &replaced));
    }

    #[test]
    fn a_filter_that_cannot_be_compiled_is_a_final_failure() {
        let broken = replication(Some("(unclosed"), None);
        let failure = target().passes(&broken, &item("", "app", "1")).unwrap_err();
        assert!(failure.permanent());
    }

    #[tokio::test]
    async fn the_items_of_a_crashed_worker_are_found_and_those_of_a_finished_one_are_not() {
        let target = Arc::new(target());
        let mut pool = tokio::task::JoinSet::new();
        let crashing = target.clone();
        pool.spawn(async move {
            let _held = crashing.hold(vec![1, 2, 3]);
            panic!("a worker breaks");
        });
        let finishing = target.clone();
        pool.spawn(async move {
            let _held = finishing.hold(vec![4, 5]);
        });
        let mut reclaimed = Vec::new();
        while let Some(joined) = pool.join_next_with_id().await {
            match joined {
                Ok((id, ())) => assert!(target.reclaim(id).is_empty()),
                Err(error) => reclaimed.extend(target.reclaim(error.id())),
            }
        }
        assert_eq!(reclaimed, [1, 2, 3]);
    }
}
