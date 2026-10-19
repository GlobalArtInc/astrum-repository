use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::FutureExt;
use futures_util::future::{BoxFuture, join_all};

use crate::{
    BlobId, BlobStore, Content, ContentStream, Error, FillPolicy, StoreKind, StoredBlob, Totals,
    Written,
};

const SPACE_TTL: Duration = Duration::from_secs(5);
const SPACE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct GroupBlobStore {
    name: String,
    members: Vec<Arc<dyn BlobStore>>,
    policy: FillPolicy,
    next: AtomicUsize,
    roomiest: Mutex<Option<(Instant, usize)>>,
}

impl GroupBlobStore {
    pub fn new(
        name: impl Into<String>,
        members: Vec<Arc<dyn BlobStore>>,
        policy: FillPolicy,
    ) -> Result<Self, Error> {
        let name = name.into();
        if members.is_empty() {
            return Err(Error::Config {
                store: name,
                message: "a group needs at least one member".to_owned(),
            });
        }
        Ok(Self {
            name,
            members,
            policy,
            next: AtomicUsize::new(0),
            roomiest: Mutex::new(None),
        })
    }

    pub fn members(&self) -> &[Arc<dyn BlobStore>] {
        &self.members
    }

    async fn writer(&self) -> Result<&Arc<dyn BlobStore>, Error> {
        let index = match self.policy {
            FillPolicy::WriteToFirst => 0,
            FillPolicy::RoundRobin => {
                self.next.fetch_add(1, Ordering::Relaxed) % self.members.len()
            }
            FillPolicy::MostFreeSpace => self.roomiest().await?,
        };
        Ok(&self.members[index])
    }

    fn cached_roomiest(&self) -> std::sync::MutexGuard<'_, Option<(Instant, usize)>> {
        self.roomiest
            .lock()
            .expect("the group space cache lock is never poisoned")
    }

    async fn roomiest(&self) -> Result<usize, Error> {
        if let Some((measured, index)) = *self.cached_roomiest()
            && measured.elapsed() < SPACE_TTL
        {
            return Ok(index);
        }
        let spaces = join_all(
            self.members
                .iter()
                .map(|member| tokio::time::timeout(SPACE_TIMEOUT, member.available_space())),
        )
        .await;
        let mut best: Option<(usize, u64)> = None;
        let mut failure = None;
        for (index, space) in spaces.into_iter().enumerate() {
            let member = self.members[index].name();
            let space = match space {
                Ok(Ok(Some(space))) => space,
                Ok(Ok(None)) => u64::MAX,
                Ok(Err(error)) => {
                    tracing::warn!(group = %self.name, member, %error, "a member of a group is left out of writes");
                    failure.get_or_insert(error);
                    continue;
                }
                Err(_) => {
                    tracing::warn!(group = %self.name, member, "a member of a group did not report its free space in time");
                    continue;
                }
            };
            if best.is_none_or(|(_, most)| space > most) {
                best = Some((index, space));
            }
        }
        let Some((index, _)) = best else {
            return Err(failure.unwrap_or_else(|| Error::Config {
                store: self.name.clone(),
                message: "none of its members reported its free space".to_owned(),
            }));
        };
        *self.cached_roomiest() = Some((Instant::now(), index));
        Ok(index)
    }

    async fn first_found<'a, T: Send>(
        &'a self,
        probe: impl Fn(&'a Arc<dyn BlobStore>) -> BoxFuture<'a, Result<Option<T>, Error>>,
    ) -> Result<Option<T>, Error> {
        let mut failure = None;
        for member in &self.members {
            match probe(member).await {
                Ok(Some(found)) => return Ok(Some(found)),
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(group = %self.name, member = member.name(), %error, "a member of a group cannot be searched");
                    failure.get_or_insert(error);
                }
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl BlobStore for GroupBlobStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StoreKind {
        StoreKind::Group
    }

    async fn put(&self, content: ContentStream) -> Result<Written, Error> {
        self.writer().await?.put(content).await
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        self.first_found(|member| member.read(id).boxed()).await
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        self.first_found(|member| member.read_range(id, offset, length).boxed())
            .await
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        self.first_found(|member| member.size(id).boxed()).await
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        let mut deleted = false;
        let mut failure = None;
        for member in &self.members {
            match member.delete(id).await {
                Ok(gone) => deleted |= gone,
                Err(error) => {
                    tracing::warn!(group = %self.name, member = member.name(), %error, "a member of a group cannot delete");
                    failure.get_or_insert(error);
                }
            }
        }
        match (deleted, failure) {
            (false, Some(error)) => Err(error),
            _ => Ok(deleted),
        }
    }

    async fn list(&self, after: Option<BlobId>, limit: usize) -> Result<Vec<StoredBlob>, Error> {
        let mut found = Vec::new();
        for member in &self.members {
            found.extend(member.list(after, limit).await?);
        }
        found.sort_by_key(|blob| blob.id);
        found.dedup_by_key(|blob| blob.id);
        found.truncate(limit);
        Ok(found)
    }

    async fn totals(&self) -> Result<Totals, Error> {
        let mut totals = Totals::default();
        for member in &self.members {
            let member = member.totals().await?;
            totals.blob_count += member.blob_count;
            totals.total_bytes += member.total_bytes;
        }
        Ok(totals)
    }

    async fn available_space(&self) -> Result<Option<u64>, Error> {
        let mut available = 0u64;
        for member in &self.members {
            match member.available_space().await? {
                Some(space) => available = available.saturating_add(space),
                None => return Ok(None),
            }
        }
        Ok(Some(available))
    }

    async fn check(&self) -> Result<(), Error> {
        for member in &self.members {
            member.check().await?;
        }
        Ok(())
    }
}
