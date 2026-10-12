mod support;

use std::sync::Arc;

use astrum_blob::config::{self, GroupSettings, Settings, StoreConfig};
use astrum_blob::{
    BlobStores, DEFAULT_STORE, Error, FillPolicy, S3Security, StoreKind, Totals, bootstrap,
};
use astrum_security::SecretBox;
use support::{Db, S3Server, chunks, file_config, pattern, read_all};

fn secrets() -> SecretBox {
    SecretBox::from_key([3; 32])
}

#[tokio::test]
async fn a_fresh_installation_gets_the_default_store_once() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    assert!(bootstrap(db.db.conn(), data.path()).await.unwrap());
    assert!(data.path().join("blobs/default/content").is_dir());

    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets());
    let config = stores.config(DEFAULT_STORE).await.unwrap().unwrap();
    assert_eq!(config.file().unwrap().path, "default");
    assert_eq!(config.soft_quota, None);
    let store = stores.get(DEFAULT_STORE).await.unwrap();
    assert_eq!(store.kind(), StoreKind::File);
    let written = store.put(chunks(b"first blob", 4)).await.unwrap();
    assert!(
        data.path()
            .join("blobs/default/content")
            .join(written.id.location())
            .is_file()
    );

    assert!(!bootstrap(db.db.conn(), data.path()).await.unwrap());
    assert!(config::delete(db.db.conn(), DEFAULT_STORE).await.unwrap());
    assert!(
        !bootstrap(db.db.conn(), data.path()).await.unwrap(),
        "a store the administrator deleted does not come back"
    );
    assert!(stores.configs().await.unwrap().is_empty());
}

#[tokio::test]
async fn an_installation_with_stores_gets_no_default() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    db.register_file("imported", "imported").await;
    assert!(!bootstrap(db.db.conn(), data.path()).await.unwrap());
    assert!(
        config::find(db.db.conn(), DEFAULT_STORE)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn stores_open_once_until_forgotten() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    db.register_file("files", "files").await;
    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets());

    let first = stores.get("files").await.unwrap();
    let again = stores.get("files").await.unwrap();
    assert!(Arc::ptr_eq(&first, &again));
    stores.forget();
    assert!(!Arc::ptr_eq(&first, &stores.get("files").await.unwrap()));
    assert!(data.path().join("blobs/files/content").is_dir());

    assert!(matches!(
        stores.get("FILES").await,
        Err(Error::UnknownStore(name)) if name == "FILES"
    ));
}

#[tokio::test]
async fn names_are_unique_ignoring_case() {
    let db = Db::new().await;
    db.register_file("Files", "a").await;
    assert_eq!(
        config::name_taken(db.db.conn(), "fILES").await.unwrap(),
        Some("Files".to_owned())
    );
    assert_eq!(
        config::name_taken(db.db.conn(), "other").await.unwrap(),
        None
    );
    assert!(matches!(
        config::insert(db.db.conn(), &file_config("FILES", "b")).await,
        Err(Error::Duplicate(_))
    ));
}

#[tokio::test]
async fn file_paths_may_not_overlap() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    db.register_file("default", "default").await;
    db.register_file("outside", "/srv/blobs/outside").await;
    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets());

    assert!(stores.overlapping("other", None).await.unwrap().is_empty());
    assert!(
        stores
            .overlapping("defaults", None)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        stores.overlapping("default", None).await.unwrap(),
        ["default"]
    );
    assert_eq!(
        stores.overlapping("./default/", None).await.unwrap(),
        ["default"]
    );
    assert_eq!(
        stores.overlapping("default/inner", None).await.unwrap(),
        ["default"]
    );
    assert_eq!(stores.overlapping("", None).await.unwrap(), ["default"]);
    assert_eq!(stores.overlapping("/srv", None).await.unwrap(), ["outside"]);
    assert_eq!(
        stores
            .overlapping(&data.path().join("blobs/default").to_string_lossy(), None)
            .await
            .unwrap(),
        ["default"]
    );
    assert!(
        stores
            .overlapping("default", Some("default"))
            .await
            .unwrap()
            .is_empty()
    );
}

async fn group(db: &Db, policy: FillPolicy) -> (tempfile::TempDir, BlobStores) {
    let data = tempfile::tempdir().unwrap();
    db.register_file("one", "one").await;
    db.register_file("two", "two").await;
    db.register(&StoreConfig {
        name: "both".to_owned(),
        settings: Settings::Group(GroupSettings {
            members: vec!["one".to_owned(), "two".to_owned()],
            fill_policy: policy,
        }),
        soft_quota: None,
    })
    .await;
    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets());
    (data, stores)
}

#[tokio::test]
async fn a_round_robin_group_spreads_writes_over_its_members() {
    let db = Db::new().await;
    let (_data, stores) = group(&db, FillPolicy::RoundRobin).await;
    let both = stores.get("both").await.unwrap();
    assert_eq!(both.kind(), StoreKind::Group);

    let first = both.put(chunks(&pattern(100), 10)).await.unwrap();
    let second = both.put(chunks(&pattern(200), 10)).await.unwrap();
    let third = both.put(chunks(&pattern(300), 10)).await.unwrap();
    assert_eq!(
        [
            first.store.as_str(),
            second.store.as_str(),
            third.store.as_str()
        ],
        ["one", "two", "one"]
    );
    for written in [&first, &second, &third] {
        assert_eq!(
            read_all(both.as_ref(), written.id).await,
            pattern(written.size as usize)
        );
        let member = stores.get(&written.store).await.unwrap();
        assert!(member.exists(written.id).await.unwrap());
    }
    let slice = both.read_range(second.id, 10, 5).await.unwrap().unwrap();
    assert_eq!(support::collect(slice).await, &pattern(200)[10..15]);
    assert_eq!(
        both.totals().await.unwrap(),
        Totals {
            blob_count: 3,
            total_bytes: 600
        }
    );
    assert!(both.available_space().await.unwrap().is_some());
    both.check().await.unwrap();

    assert!(both.delete(second.id).await.unwrap());
    assert!(!both.exists(second.id).await.unwrap());
    assert_eq!(both.size(first.id).await.unwrap(), Some(100));
}

#[tokio::test]
async fn a_write_to_first_group_fills_its_first_member() {
    let db = Db::new().await;
    let (_data, stores) = group(&db, FillPolicy::WriteToFirst).await;
    let both = stores.get("both").await.unwrap();
    for _ in 0..3 {
        assert_eq!(both.put(chunks(b"x", 1)).await.unwrap().store, "one");
    }
}

#[tokio::test]
async fn members_of_a_group_cannot_be_deleted() {
    let db = Db::new().await;
    let (_data, _stores) = group(&db, FillPolicy::WriteToFirst).await;
    assert!(matches!(
        config::delete(db.db.conn(), "two").await,
        Err(Error::InUse(name)) if name == "two"
    ));
    assert!(config::delete(db.db.conn(), "both").await.unwrap());
    assert!(config::delete(db.db.conn(), "two").await.unwrap());
    assert!(!config::delete(db.db.conn(), "two").await.unwrap());
}

#[tokio::test]
async fn a_group_that_contains_itself_does_not_open() {
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    db.register(&StoreConfig {
        name: "loop".to_owned(),
        settings: Settings::Group(GroupSettings {
            members: vec!["loop".to_owned()],
            fill_policy: FillPolicy::WriteToFirst,
        }),
        soft_quota: None,
    })
    .await;
    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets());
    assert!(matches!(
        stores.get("loop").await,
        Err(Error::Config { .. })
    ));
}

#[tokio::test]
async fn s3_stores_open_with_sealed_keys() {
    let server = S3Server::start().await;
    let db = Db::new().await;
    let data = tempfile::tempdir().unwrap();
    let secrets = secrets();
    let mut settings = support::s3_settings(&server, "sealed", "");
    settings.security = S3Security {
        access_key_id: Some(support::ACCESS_KEY.to_owned()),
        secret_access_key: Some(secrets.seal(support::SECRET_KEY)),
        ..Default::default()
    };
    db.register(&StoreConfig {
        name: "s3".to_owned(),
        settings: Settings::S3(Box::new(settings.clone())),
        soft_quota: None,
    })
    .await;
    let stores = BlobStores::new(db.db.conn().clone(), data.path(), secrets);
    stores
        .open_s3("s3", &settings)
        .await
        .unwrap()
        .ensure_bucket()
        .await
        .unwrap();
    let store = stores.get("s3").await.unwrap();
    assert_eq!(store.kind(), StoreKind::S3);
    store.check().await.unwrap();
    let written = store.put(chunks(b"sealed", 2)).await.unwrap();
    assert_eq!(read_all(store.as_ref(), written.id).await, b"sealed");

    let other = BlobStores::new(
        db.db.conn().clone(),
        data.path(),
        SecretBox::from_key([4; 32]),
    );
    assert!(matches!(other.get("s3").await, Err(Error::Secret(_))));
}
