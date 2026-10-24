mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use astrum_blob::config::{self, Settings};
use astrum_blob::{
    BlobId, BlobStore, BlobStores, Content, ContentStream, Error, FileBlobStore, FillPolicy,
    GroupBlobStore, Metrics, StoreKind, StoredBlob, Totals, Written,
};
use astrum_security::SecretBox;
use async_trait::async_trait;
use support::{Db, chunks, file_config, pattern, read_all};

struct Member {
    file: FileBlobStore,
    space: Option<u64>,
    down: AtomicBool,
}

impl Member {
    async fn new(dir: &tempfile::TempDir, name: &str, space: Option<u64>) -> Arc<Self> {
        let file = FileBlobStore::open(name, dir.path().join(name), Metrics::detached(name))
            .await
            .unwrap();
        Arc::new(Self {
            file,
            space,
            down: AtomicBool::new(false),
        })
    }

    fn fail(&self) -> Result<(), Error> {
        match self.down.load(Ordering::Relaxed) {
            true => Err(Error::Remote {
                store: self.file.name().to_owned(),
                operation: "answer",
                message: "it is down".to_owned(),
            }),
            false => Ok(()),
        }
    }
}

#[async_trait]
impl BlobStore for Member {
    fn name(&self) -> &str {
        self.file.name()
    }

    fn kind(&self) -> StoreKind {
        StoreKind::Remote
    }

    async fn put(&self, content: ContentStream) -> Result<Written, Error> {
        self.fail()?;
        self.file.put(content).await
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        self.fail()?;
        self.file.read(id).await
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        self.fail()?;
        self.file.read_range(id, offset, length).await
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        self.fail()?;
        self.file.size(id).await
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        self.fail()?;
        self.file.delete(id).await
    }

    async fn list(&self, after: Option<BlobId>, limit: usize) -> Result<Vec<StoredBlob>, Error> {
        self.fail()?;
        self.file.list(after, limit).await
    }

    async fn totals(&self) -> Result<Totals, Error> {
        self.fail()?;
        self.file.totals().await
    }

    async fn available_space(&self) -> Result<Option<u64>, Error> {
        self.fail()?;
        Ok(self.space)
    }

    async fn check(&self) -> Result<(), Error> {
        self.fail()
    }
}

fn group(members: &[&Arc<Member>], policy: FillPolicy) -> GroupBlobStore {
    GroupBlobStore::new(
        "group",
        members
            .iter()
            .map(|member| (*member).clone() as Arc<dyn BlobStore>)
            .collect(),
        policy,
    )
    .unwrap()
}

#[tokio::test]
async fn most_free_space_writes_to_the_roomiest_member() {
    let dir = tempfile::tempdir().unwrap();
    let small = Member::new(&dir, "small", Some(10)).await;
    let large = Member::new(&dir, "large", Some(1_000)).await;
    let group = group(&[&small, &large], FillPolicy::MostFreeSpace);
    for _ in 0..3 {
        assert_eq!(group.put(chunks(b"blob", 2)).await.unwrap().store, "large");
    }
}

#[tokio::test]
async fn a_member_without_a_known_limit_counts_as_roomiest() {
    let dir = tempfile::tempdir().unwrap();
    let disk = Member::new(&dir, "disk", Some(u64::MAX / 2)).await;
    let bucket = Member::new(&dir, "bucket", None).await;
    let group = group(&[&disk, &bucket], FillPolicy::MostFreeSpace);
    assert_eq!(group.put(chunks(b"blob", 2)).await.unwrap().store, "bucket");
}

#[tokio::test]
async fn a_member_that_is_down_is_left_out_of_writes() {
    let dir = tempfile::tempdir().unwrap();
    let roomy = Member::new(&dir, "roomy", Some(1_000)).await;
    let tight = Member::new(&dir, "tight", Some(10)).await;
    roomy.down.store(true, Ordering::Relaxed);
    let group = group(&[&roomy, &tight], FillPolicy::MostFreeSpace);
    assert_eq!(group.put(chunks(b"blob", 2)).await.unwrap().store, "tight");

    tight.down.store(true, Ordering::Relaxed);
    let group = self::group(&[&roomy, &tight], FillPolicy::MostFreeSpace);
    let error = group.put(chunks(b"blob", 2)).await.unwrap_err();
    assert!(matches!(error, Error::Remote { .. }), "{error}");
}

#[tokio::test]
async fn reads_and_deletes_look_past_a_member_that_is_down() {
    let dir = tempfile::tempdir().unwrap();
    let first = Member::new(&dir, "first", Some(10)).await;
    let second = Member::new(&dir, "second", Some(10)).await;
    let written = second.put(chunks(&pattern(500), 50)).await.unwrap();
    first.down.store(true, Ordering::Relaxed);
    let group = group(&[&first, &second], FillPolicy::WriteToFirst);

    assert_eq!(read_all(&group, written.id).await, pattern(500));
    let slice = group.read_range(written.id, 5, 5).await.unwrap().unwrap();
    assert_eq!(support::collect(slice).await, &pattern(500)[5..10]);
    assert_eq!(group.size(written.id).await.unwrap(), Some(500));

    let unknown = BlobId::new();
    assert!(matches!(
        group.read(unknown).await,
        Err(Error::Remote { .. })
    ));
    assert!(group.size(unknown).await.is_err());
    assert!(group.check().await.is_err());

    assert!(group.delete(written.id).await.unwrap());
    assert!(group.delete(written.id).await.is_err());
    first.down.store(false, Ordering::Relaxed);
    assert!(!group.delete(written.id).await.unwrap());
    assert!(group.read(written.id).await.unwrap().is_none());
}

#[tokio::test]
async fn a_store_turns_into_a_group_that_keeps_its_blobs() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(data.path().join("blobs/store")).unwrap();
    let mut store_config = file_config("store", "store");
    store_config.soft_quota = Some(astrum_blob::SoftQuota {
        kind: astrum_blob::QuotaKind::SpaceUsedQuota,
        limit: 1_000_000,
    });
    db.register(&store_config).await;
    let stores = BlobStores::new(
        db.db.conn().clone(),
        data.path(),
        SecretBox::from_key([5; 32]),
    );
    let written = stores
        .get("store")
        .await
        .unwrap()
        .put(chunks(&pattern(300), 30))
        .await
        .unwrap();
    assert_eq!(written.store, "store");

    let group = config::promote(db.db.conn(), "store", "store-disk")
        .await
        .unwrap();
    stores.changed().await.unwrap();
    let settings = group.group().unwrap();
    assert_eq!(settings.members, vec!["store-disk".to_owned()]);
    assert_eq!(settings.fill_policy, FillPolicy::WriteToFirst);
    assert_eq!(group.soft_quota, store_config.soft_quota);

    let moved = stores.config("store-disk").await.unwrap().unwrap();
    assert_eq!(moved.file().unwrap().path, "store");
    assert_eq!(moved.soft_quota, None);
    assert_eq!(
        stores.metrics("store-disk").totals().await.unwrap(),
        Totals {
            blob_count: 1,
            total_bytes: 300
        }
    );
    assert_eq!(stores.config("store").await.unwrap().unwrap(), group);

    let promoted = stores.get("store").await.unwrap();
    assert_eq!(promoted.kind(), StoreKind::Group);
    assert_eq!(read_all(promoted.as_ref(), written.id).await, pattern(300));
    let member = stores.get("store-disk").await.unwrap();
    assert_eq!(read_all(member.as_ref(), written.id).await, pattern(300));
    assert_eq!(
        promoted.totals().await.unwrap(),
        Totals {
            blob_count: 1,
            total_bytes: 300
        }
    );
    let next = promoted.put(chunks(b"next", 2)).await.unwrap();
    assert_eq!(next.store, "store-disk");

    assert!(matches!(
        config::promote(db.db.conn(), "store", "again").await,
        Err(Error::Config { .. })
    ));
    db.register(&file_config("other", "other")).await;
    assert!(matches!(
        config::promote(db.db.conn(), "other", "store-disk").await,
        Err(Error::Duplicate(name)) if name == "store-disk"
    ));
    assert_eq!(
        stores.config("other").await.unwrap().unwrap().settings,
        Settings::File(astrum_blob::FileSettings {
            path: "other".to_owned()
        })
    );
    assert!(matches!(
        config::promote(db.db.conn(), "missing", "x").await,
        Err(Error::UnknownStore(_))
    ));
}
