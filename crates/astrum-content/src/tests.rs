use std::sync::Arc;

use astrum_blob::BlobStores;
use astrum_db::entity::component;
use astrum_repo::{Repository, Request, Spec};
use astrum_security::{SecretBox, SecurityModel};
use astrum_testkit::TestDatabase;
use async_trait::async_trait;
use axum::response::Response;
use bytes::Bytes;
use chrono::DateTime;
use futures_util::{StreamExt, stream};
use sea_orm::ConnectionTrait;
use serde_json::json;

use crate::store::{self, Asset, PutAsset};
use crate::{Content, ContentError, ContentRequest, Ctx, Format, Gate};

struct Refuse;

#[async_trait]
impl Gate for Refuse {
    async fn check(&self, _ctx: &Ctx, _asset: &Asset) -> Result<(), ContentError> {
        Err(ContentError::Blocked("refused".to_owned()))
    }

    async fn check_digest(
        &self,
        _ctx: &Ctx,
        _repository: &str,
        _digest: &str,
    ) -> Result<(), ContentError> {
        Err(ContentError::Blocked("refused".to_owned()))
    }
}

struct Plain;

#[async_trait]
impl Format for Plain {
    fn id(&self) -> &'static str {
        "plain"
    }

    async fn handle(&self, _ctx: Ctx, _request: ContentRequest) -> Result<Response, ContentError> {
        Err(ContentError::not_found("nothing here"))
    }

    async fn get(&self, _ctx: &Ctx, _path: &str) -> Result<Option<Asset>, ContentError> {
        Ok(None)
    }
}

async fn setup() -> (TestDatabase, Content, tempfile::TempDir, Ctx) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let dir = tempfile::tempdir().unwrap();
    let blobs = Arc::new(BlobStores::new(
        db.conn().clone(),
        dir.path(),
        SecretBox::from_key([7; 32]),
    ));
    let content = Content::new(db.conn().clone(), blobs, SecretBox::from_key([7; 32]));
    let request: Request = serde_json::from_value(json!({
        "name": "releases",
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"},
    }))
    .unwrap();
    let repository = Repository::prepare(
        Spec::from_api("maven", "hosted").unwrap(),
        request,
        None,
        &|plain: &str| plain.to_owned(),
    )
    .unwrap();
    let ctx = content.context(
        Arc::new(repository),
        Arc::new(SecurityModel::default()),
        None,
        "http://h",
        "127.0.0.1",
    );
    (test, content, dir, ctx)
}

#[tokio::test]
async fn the_default_gate_passes_and_a_registered_gate_replaces_it() {
    let (_test, content, _dir, ctx) = setup().await;
    content
        .gate()
        .check_digest(&ctx, "releases", "sha256:abc")
        .await
        .unwrap();
    content.register_gate(Arc::new(Refuse));
    let refused = content
        .gate()
        .check_digest(&ctx, "releases", "sha256:abc")
        .await;
    assert!(
        matches!(refused, Err(ContentError::Blocked(_))),
        "{refused:?}"
    );
}

#[tokio::test]
async fn a_format_has_no_inventory_unless_it_defines_one() {
    let (_test, _content, _dir, ctx) = setup().await;
    let now = chrono::Utc::now().fixed_offset();
    let component = component::Model {
        id: 1,
        repository: "releases".to_owned(),
        format: "plain".to_owned(),
        namespace: String::new(),
        name: "left-pad".to_owned(),
        version: "1.0.0".to_owned(),
        kind: "package".to_owned(),
        attributes: json!({}),
        created_at: now,
        updated_at: now,
    };
    let inventory = Plain.inventory(&ctx, &component).await.unwrap();
    assert!(inventory.is_none());
}

#[tokio::test]
async fn a_published_date_is_stored_replaced_and_kept_by_a_write_without_one() {
    let (_test, _content, dir, ctx) = setup().await;
    astrum_blob::bootstrap(ctx.db(), dir.path()).await.unwrap();
    ctx.db()
        .execute_unprepared(
            "INSERT INTO repository (name, format, type) VALUES ('releases', 'maven2', 'hosted')",
        )
        .await
        .unwrap();
    let path = "org/x/1.0/x-1.0.jar";
    let body = || stream::iter(vec![Ok::<Bytes, std::io::Error>(Bytes::from_static(b"x"))]).boxed();
    let first = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap();
    let second = DateTime::parse_from_rfc3339("2026-02-01T00:00:00Z").unwrap();

    store::put(&ctx, PutAsset::cached(path).published(first), body())
        .await
        .unwrap();
    let asset = store::find(ctx.db(), "releases", path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(asset.upstream_published_at, Some(first));

    store::put(&ctx, PutAsset::cached(path).published(second), body())
        .await
        .unwrap();
    let asset = store::find(ctx.db(), "releases", path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(asset.upstream_published_at, Some(second));

    store::put(&ctx, PutAsset::cached(path), body())
        .await
        .unwrap();
    let asset = store::find(ctx.db(), "releases", path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(asset.upstream_published_at, Some(second));
}
