#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_blob::{BlobStores, FileSettings, Settings, StoreConfig, content_of};
use astrum_content::import::{Described, Imported, Plan};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::transfer::{
    Bundle, Conflict, DescribeFn, GenericTransfer, LocalSource, Outcome, PROMOTED, Terms, Transfer,
};
use astrum_content::{Content, ContentError, Ctx};
use astrum_repo::config::{Attributes, Group, Quota, Repository, Storage, WritePolicy};
use astrum_repo::{RepositoryType, format};
use astrum_security::{SecretBox, SecurityModel};
use astrum_testkit::TestDatabase;
use futures_util::future::BoxFuture;
use sea_orm::{ConnectionTrait, DbBackend, Statement};

pub struct World {
    pub _test: TestDatabase,
    pub db: astrum_db::Database,
    pub dir: tempfile::TempDir,
    pub content: Content,
}

pub async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let dir = tempfile::tempdir().unwrap();
    astrum_blob::bootstrap(db.conn(), dir.path()).await.unwrap();
    std::fs::create_dir_all(dir.path().join("blobs").join("other")).unwrap();
    astrum_blob::config::insert(
        db.conn(),
        &StoreConfig {
            name: "other".to_owned(),
            settings: Settings::File(FileSettings {
                path: "other".to_owned(),
            }),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    let secrets = SecretBox::from_key([7; 32]);
    let stores = Arc::new(BlobStores::new(
        db.conn().clone(),
        dir.path(),
        secrets.clone(),
    ));
    let content = Content::new(db.conn().clone(), stores, secrets);
    World {
        _test: test,
        db,
        dir,
        content,
    }
}

pub async fn repository(world: &World, name: &str, store: &str, policy: WritePolicy) -> Ctx {
    repository_of(world, name, store, policy, RepositoryType::Hosted).await
}

pub async fn repository_of(
    world: &World,
    name: &str,
    store: &str,
    policy: WritePolicy,
    kind: RepositoryType,
) -> Ctx {
    repository_with(world, name, store, policy, kind, None).await
}

pub async fn capped_repository(world: &World, name: &str, limit: i64) -> Ctx {
    let quota = Quota {
        limit_bytes: Some(limit),
        warn_percent: None,
        enforce: true,
    };
    repository_with(
        world,
        name,
        "default",
        WritePolicy::Allow,
        RepositoryType::Hosted,
        Some(quota),
    )
    .await
}

async fn repository_with(
    world: &World,
    name: &str,
    store: &str,
    policy: WritePolicy,
    kind: RepositoryType,
    quota: Option<Quota>,
) -> Ctx {
    let created = astrum_repo::store::Writer::begin(world.db.conn())
        .await
        .unwrap()
        .create(Repository {
            name: name.to_owned(),
            format: format("raw").unwrap(),
            kind,
            online: true,
            attributes: Attributes {
                group: (kind == RepositoryType::Group).then(|| Group {
                    member_names: Some(Vec::new()),
                    writable_member: None,
                }),
                storage: Some(Storage {
                    blob_store_name: Some(store.to_owned()),
                    strict_content_type_validation: Some(false),
                    write_policy: Some(policy),
                    latest_policy: None,
                    quota,
                }),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    world.content.context(
        Arc::new(created),
        Arc::new(SecurityModel::default()),
        None,
        "",
        "",
    )
}

pub fn key() -> ComponentKey {
    ComponentKey {
        namespace: "g".to_owned(),
        name: "lib".to_owned(),
        version: "1".to_owned(),
        ..Default::default()
    }
}

pub async fn seed(ctx: &Ctx, path: &str, body: &'static [u8]) {
    store::put(
        ctx,
        PutAsset::hosted(ctx, path).component(key()),
        content_of(body),
    )
    .await
    .unwrap();
}

pub async fn bundle_of(ctx: &Ctx) -> Bundle {
    let any = store::under(ctx.db(), ctx.name(), "/", 1).await.unwrap();
    let component = store::component_by_id(ctx.db(), any[0].component_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    transfer().bundle(ctx, &component).await.unwrap()
}

pub fn plan(_: RepositoryType, path: &str, _: &str) -> Plan {
    match path {
        derived if derived.ends_with(".sha1") => Plan::Derived,
        refused if refused.ends_with(".bad") => Plan::Skip("not a raw file".to_owned()),
        _ => Plan::Import,
    }
}

pub fn describe(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(async move {
        let component = file.component.as_ref().map(|found| found.key());
        let mut put = file.put(file.path.clone());
        put.component = component;
        Ok(Some(Described::source(put)))
    })
}

pub fn describe_rewritten(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(async move {
        let component = file.component.as_ref().map(|found| found.key());
        let mut put = file.put(file.path.clone());
        put.component = component.clone();
        let mut note = file.put(format!("{}.note", file.path));
        note.component = component;
        Ok(Some(
            Described::bytes(put, b"rewritten".to_vec()).with(note, b"note".to_vec()),
        ))
    })
}

pub fn transfer_of(describe: DescribeFn) -> GenericTransfer {
    GenericTransfer {
        plan,
        describe,
        after: None,
        admit: None,
        replaceable: None,
    }
}

pub fn transfer() -> GenericTransfer {
    transfer_of(describe)
}

pub async fn run(
    source: &Ctx,
    target: &Ctx,
    bundle: &Bundle,
    conflict: Conflict,
    dry_run: bool,
) -> Result<Outcome, ContentError> {
    let mut terms = Terms::new(conflict, PROMOTED);
    terms.dry_run = dry_run;
    transfer()
        .receive(target, bundle, &LocalSource::new(source.clone()), terms)
        .await
}

pub async fn number(world: &World, sql: &str) -> i64 {
    world
        .db
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

pub async fn target_rows(world: &World, repository: &str) -> (i64, i64) {
    let components = number(
        world,
        &format!("SELECT count(*) AS n FROM component WHERE repository = '{repository}'"),
    )
    .await;
    let assets = number(
        world,
        &format!("SELECT count(*) AS n FROM asset WHERE repository = '{repository}'"),
    )
    .await;
    (components, assets)
}

pub async fn hold_rows(world: &World) -> i64 {
    number(world, "SELECT count(*) AS n FROM blob_hold").await
}

pub async fn unnamed_blobs(world: &World) -> i64 {
    let listed = world
        .content
        .blob_stores()
        .get("default")
        .await
        .unwrap()
        .list(None, 1000)
        .await
        .unwrap();
    let named = number(world, "SELECT count(DISTINCT blob_id) AS n FROM asset").await;
    listed.len() as i64 - named
}

pub async fn wait_for_lock(world: &World) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while number(
        world,
        "SELECT count(*) AS n FROM pg_stat_activity WHERE wait_event_type = 'Lock' AND datname = current_database()",
    )
    .await
        == 0
    {
        assert!(Instant::now() < deadline, "no statement ever waited for a lock");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub async fn blobs_in(world: &World, store: &str) -> usize {
    world
        .content
        .blob_stores()
        .get(store)
        .await
        .unwrap()
        .list(None, 1000)
        .await
        .unwrap()
        .len()
}

pub fn blob_file(world: &World, store: &str, id: uuid::Uuid) -> PathBuf {
    world
        .dir
        .path()
        .join("blobs")
        .join(store)
        .join("content")
        .join(astrum_blob::BlobId::from(id).location())
}

pub async fn row(ctx: &Ctx, path: &str) -> store::Asset {
    store::find(ctx.db(), ctx.name(), path)
        .await
        .unwrap()
        .unwrap()
}
