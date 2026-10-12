use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::{
    BlobId, BlobStore, Content, ContentStream, Error, FillPolicy, StoreKind, Totals, Written,
};

pub struct GroupBlobStore {
    name: String,
    members: Vec<Arc<dyn BlobStore>>,
    policy: FillPolicy,
    next: AtomicUsize,
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
        })
    }

    pub fn members(&self) -> &[Arc<dyn BlobStore>] {
        &self.members
    }

    fn writer(&self) -> &Arc<dyn BlobStore> {
        let index = match self.policy {
            FillPolicy::WriteToFirst => 0,
            FillPolicy::RoundRobin => {
                self.next.fetch_add(1, Ordering::Relaxed) % self.members.len()
            }
        };
        &self.members[index]
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
        self.writer().put(content).await
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        for member in &self.members {
            if let Some(content) = member.read(id).await? {
                return Ok(Some(content));
            }
        }
        Ok(None)
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        for member in &self.members {
            if let Some(content) = member.read_range(id, offset, length).await? {
                return Ok(Some(content));
            }
        }
        Ok(None)
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        for member in &self.members {
            if let Some(size) = member.size(id).await? {
                return Ok(Some(size));
            }
        }
        Ok(None)
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        let mut deleted = false;
        for member in &self.members {
            deleted |= member.delete(id).await?;
        }
        Ok(deleted)
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
