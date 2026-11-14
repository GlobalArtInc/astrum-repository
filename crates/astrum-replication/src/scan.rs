use std::collections::{HashMap, HashSet};

use astrum_content::transfer::{Transfer, fingerprint, origin_of};
use astrum_content::{Component, Ctx};
use regex::Regex;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::Client;
use crate::filter::{compile, subject};
use crate::listing::{self, Key};
use crate::protocol::{InventoryKey, component_key, parse_component_key};
use crate::queue::{self, ReplicationRow};
use crate::reconcile::Ended;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub enqueue: u64,
    pub page: u32,
    pub batch: usize,
    pub remote_pages: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            enqueue: 100_000,
            page: 500,
            batch: 1000,
            remote_pages: 20_000,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub scanned: u64,
    pub filtered: u64,
    pub from_remote: u64,
    pub from_here: u64,
    pub empty: u64,
    pub remote_scanned: u64,
    pub enqueued: u64,
    pub unreadable: u64,
    pub first_unreadable: Option<String>,
    pub capped: bool,
}

pub(crate) struct Filters {
    name: Option<Regex>,
    version: Option<Regex>,
}

impl Filters {
    pub(crate) fn new(replication: &ReplicationRow) -> Result<Self, String> {
        let build = |pattern: &Option<String>| {
            pattern
                .as_deref()
                .map(compile)
                .transpose()
                .map_err(|reason| format!("a filter of the replication {reason}"))
        };
        Ok(Self {
            name: build(&replication.name_filter)?,
            version: build(&replication.version_filter)?,
        })
    }

    pub(crate) fn passes(&self, key: &Key) -> bool {
        self.name
            .as_ref()
            .is_none_or(|name| name.is_match(&subject(&key.0, &key.1)))
            && self
                .version
                .as_ref()
                .is_none_or(|version| version.is_match(&key.2))
    }
}

pub(crate) struct Scan<'a> {
    pub db: &'a DatabaseConnection,
    pub client: &'a Client,
    pub replication: &'a ReplicationRow,
    pub filters: Filters,
    pub remote: Uuid,
    pub own: Uuid,
    pub ctx: &'a Ctx,
    pub transfer: &'a dyn Transfer,
    pub limits: &'a Limits,
    pub stop: &'a (dyn Fn() -> bool + Send + Sync),
    pub tally: Tally,
}

impl Scan<'_> {
    pub(crate) fn check(&self) -> Result<(), Ended> {
        match (self.stop)() {
            true => Err(Ended::Stopped),
            false => Ok(()),
        }
    }

    pub(crate) async fn local(&mut self) -> Result<(), Ended> {
        let repository = self.replication.source_repository.clone();
        let mut after: Option<Key> = None;
        let mut pending: Vec<(Key, String)> = Vec::new();
        loop {
            self.check()?;
            let rows =
                listing::page(self.db, &repository, after.as_ref(), self.limits.page).await?;
            let Some(last) = rows.last() else {
                break;
            };
            after = Some(listing::key_of(last));
            let full = rows.len() == self.limits.page as usize;
            for found in &rows {
                self.consider(found, &mut pending).await;
            }
            while pending.len() >= self.limits.batch && !self.tally.capped {
                let rest = pending.split_off(self.limits.batch);
                let batch = std::mem::replace(&mut pending, rest);
                self.compare(&batch).await?;
            }
            if self.tally.capped || !full {
                break;
            }
        }
        if !self.tally.capped && !pending.is_empty() {
            self.compare(&pending).await?;
        }
        Ok(())
    }

    async fn consider(&mut self, found: &Component, pending: &mut Vec<(Key, String)>) {
        self.tally.scanned += 1;
        let key = listing::key_of(found);
        if !self.filters.passes(&key) {
            self.tally.filtered += 1;
            return;
        }
        let origin = origin_of(&found.attributes)
            .map(str::to_owned)
            .unwrap_or_else(|| self.own.to_string());
        if origin.eq_ignore_ascii_case(&self.remote.to_string()) {
            self.tally.from_remote += 1;
            return;
        }
        match self.transfer.bundle(self.ctx, found).await {
            Ok(bundle) if bundle.assets.is_empty() => self.tally.empty += 1,
            Ok(bundle) => pending.push((key, fingerprint(&bundle))),
            Err(error) => self.unreadable(&key, &error.to_string()),
        }
    }

    pub(crate) fn unreadable(&mut self, key: &Key, error: &str) {
        self.tally.unreadable += 1;
        if self.tally.first_unreadable.is_none() {
            self.tally.first_unreadable = Some(format!(
                "{}: {error}",
                component_key(&key.0, &key.1, &key.2)
            ));
        }
    }

    async fn compare(&mut self, batch: &[(Key, String)]) -> Result<(), Ended> {
        self.check()?;
        let items: Vec<InventoryKey> = batch
            .iter()
            .map(|(key, digest)| InventoryKey {
                key: component_key(&key.0, &key.1, &key.2),
                fingerprint: digest.clone(),
            })
            .collect();
        let reply = self
            .client
            .inventory_compare(&self.replication.remote_repository, &items)
            .await?;
        let asked: HashMap<&str, &Key> = items
            .iter()
            .zip(batch)
            .map(|(item, (key, _))| (item.key.as_str(), key))
            .collect();
        let mut seen = HashSet::new();
        let mut wanted = Vec::new();
        for differing in &reply.differing {
            let Some(key) = asked.get(differing.as_str()) else {
                return Err(Ended::Failed(
                    "the remote named a component that was not asked about".to_owned(),
                ));
            };
            if seen.insert(differing.as_str()) {
                wanted.push((*key).clone());
            }
        }
        self.enqueue(wanted).await
    }

    pub(crate) async fn enqueue(&mut self, keys: Vec<Key>) -> Result<(), Ended> {
        let mut rest = keys.as_slice();
        while !rest.is_empty() {
            let room = self.limits.enqueue.saturating_sub(self.tally.enqueued);
            let room = usize::try_from(room).unwrap_or(usize::MAX);
            if room == 0 {
                self.tally.capped = true;
                return Ok(());
            }
            let (now, later) = rest.split_at(room.min(rest.len()));
            self.tally.enqueued += queue::enqueue(self.db, self.replication.id, now).await?;
            rest = later;
        }
        Ok(())
    }

    pub(crate) async fn remote(&mut self) -> Result<(), Ended> {
        let own = self.own.to_string();
        let mut after: Option<String> = None;
        let mut position: Option<Key> = None;
        for _ in 0..self.limits.remote_pages {
            self.check()?;
            let page = self
                .client
                .inventory(
                    &self.replication.remote_repository,
                    after.as_deref(),
                    self.limits.page,
                )
                .await?;
            if page.items.len() > self.limits.page as usize {
                return Err(Ended::Failed(
                    "the remote sent more inventory entries than were asked for".to_owned(),
                ));
            }
            let mut candidates = Vec::new();
            for entry in &page.items {
                let key = parse_component_key(&entry.key).ok_or_else(|| {
                    Ended::Failed(
                        "the remote inventory holds a key that is no component key".to_owned(),
                    )
                })?;
                if position.as_ref().is_some_and(|last| *last >= key) {
                    return Err(Ended::Failed(
                        "the remote inventory is not in key order".to_owned(),
                    ));
                }
                position = Some(key.clone());
                self.tally.remote_scanned += 1;
                let ours = entry
                    .origin
                    .as_deref()
                    .is_some_and(|origin| origin.eq_ignore_ascii_case(&own));
                if ours && self.filters.passes(&key) {
                    candidates.push(key);
                }
            }
            let gone =
                listing::absent(self.db, &self.replication.source_repository, &candidates).await?;
            self.enqueue(gone).await?;
            if self.tally.capped {
                return Ok(());
            }
            let Some(next) = page.next else {
                return Ok(());
            };
            if parse_component_key(&next) != position {
                return Err(Ended::Failed(
                    "the remote inventory continues from a key that is not its last entry"
                        .to_owned(),
                ));
            }
            after = Some(next);
        }
        Err(Ended::Failed(format!(
            "the remote inventory has more than {} pages",
            self.limits.remote_pages
        )))
    }
}
