use std::collections::VecDeque;

use astrum_content::Component;
use astrum_content::transfer::{fingerprint, origin_of};
use sea_orm::{DatabaseConnection, DbErr};

use crate::listing::{self, Key};
use crate::protocol::{InventoryEntry, parse_component_key};
use crate::reconcile::Ended;
use crate::scan::Scan;

struct Local {
    after: Option<Key>,
    rows: VecDeque<Component>,
    more: bool,
}

impl Local {
    fn new() -> Self {
        Self {
            after: None,
            rows: VecDeque::new(),
            more: true,
        }
    }

    async fn head(
        &mut self,
        db: &DatabaseConnection,
        repository: &str,
        page: u32,
    ) -> Result<Option<&Component>, DbErr> {
        if self.rows.is_empty() && self.more {
            let rows = listing::page(db, repository, self.after.as_ref(), page).await?;
            self.more = rows.len() == page as usize;
            if let Some(last) = rows.last() {
                self.after = Some(listing::key_of(last));
            }
            self.rows = rows.into();
        }
        Ok(self.rows.front())
    }
}

fn named(origin: Option<&str>, instance: &str) -> bool {
    origin.is_some_and(|origin| origin.eq_ignore_ascii_case(instance))
}

impl Scan<'_> {
    pub(crate) async fn pull(&mut self) -> Result<(), Ended> {
        let repository = self.replication.source_repository.clone();
        let mut local = Local::new();
        let mut wanted: Vec<Key> = Vec::new();
        let mut after: Option<String> = None;
        let mut position: Option<Key> = None;
        let mut finished = false;
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
                self.unlisted(&mut local, &repository, Some(&key), &mut wanted)
                    .await?;
                let here = self.listed(&mut local, &repository, &key).await?;
                self.compare_entry(entry, key, here, &mut wanted).await;
            }
            self.flush(&mut wanted, false).await?;
            if self.tally.capped {
                return Ok(());
            }
            let Some(next) = page.next else {
                finished = true;
                break;
            };
            if parse_component_key(&next) != position {
                return Err(Ended::Failed(
                    "the remote inventory continues from a key that is not its last entry"
                        .to_owned(),
                ));
            }
            after = Some(next);
        }
        if !finished {
            return Err(Ended::Failed(format!(
                "the remote inventory has more than {} pages",
                self.limits.remote_pages
            )));
        }
        self.unlisted(&mut local, &repository, None, &mut wanted)
            .await?;
        self.flush(&mut wanted, true).await
    }

    async fn flush(&mut self, wanted: &mut Vec<Key>, all: bool) -> Result<(), Ended> {
        if wanted.is_empty() || (!all && wanted.len() < self.limits.batch) {
            return Ok(());
        }
        let keys = std::mem::take(wanted);
        self.enqueue(keys).await
    }

    async fn unlisted(
        &mut self,
        local: &mut Local,
        repository: &str,
        before: Option<&Key>,
        wanted: &mut Vec<Key>,
    ) -> Result<(), Ended> {
        loop {
            let Some(head) = local.head(self.db, repository, self.limits.page).await? else {
                return Ok(());
            };
            let key = listing::key_of(head);
            if before.is_some_and(|before| key >= *before) {
                return Ok(());
            }
            let Some(found) = local.rows.pop_front() else {
                return Ok(());
            };
            self.tally.scanned += 1;
            self.consider_unlisted(&found, key, wanted);
            self.flush(wanted, false).await?;
            if self.tally.capped {
                return Ok(());
            }
        }
    }

    fn consider_unlisted(&mut self, found: &Component, key: Key, wanted: &mut Vec<Key>) {
        let ours = named(origin_of(&found.attributes), &self.remote.to_string());
        if !self.replication.propagate_deletes || !ours {
            return;
        }
        match self.filters.passes(&key) {
            true => wanted.push(key),
            false => self.tally.filtered += 1,
        }
    }

    async fn listed(
        &mut self,
        local: &mut Local,
        repository: &str,
        key: &Key,
    ) -> Result<Option<Component>, Ended> {
        let Some(head) = local.head(self.db, repository, self.limits.page).await? else {
            return Ok(None);
        };
        if listing::key_of(head) != *key {
            return Ok(None);
        }
        self.tally.scanned += 1;
        Ok(local.rows.pop_front())
    }

    async fn compare_entry(
        &mut self,
        entry: &InventoryEntry,
        key: Key,
        here: Option<Component>,
        wanted: &mut Vec<Key>,
    ) {
        if !self.filters.passes(&key) {
            self.tally.filtered += 1;
            return;
        }
        if named(entry.origin.as_deref(), &self.own.to_string()) {
            self.tally.from_here += 1;
            return;
        }
        let Some(found) = here else {
            wanted.push(key);
            return;
        };
        match self.transfer.bundle(self.ctx, &found).await {
            Ok(bundle) if fingerprint(&bundle) == entry.fingerprint => {}
            Ok(_) => wanted.push(key),
            Err(error) => self.unreadable(&key, &error.to_string()),
        }
    }
}
