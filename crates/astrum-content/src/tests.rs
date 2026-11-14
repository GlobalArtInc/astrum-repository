use std::sync::Arc;

use astrum_blob::BlobStores;
use astrum_db::entity::component;
use astrum_repo::{Repository, Request, Spec};
use astrum_security::{SecretBox, SecurityModel};
use astrum_testkit::TestDatabase;
use async_trait::async_trait;
use axum::response::Response;
use serde_json::json;

use crate::store::Asset;
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
        _reference: Option<&str>,
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
        .check_digest(&ctx, "releases", "sha256:abc", None)
        .await
        .unwrap();
    content.register_gate(Arc::new(Refuse));
    let refused = content
        .gate()
        .check_digest(&ctx, "releases", "sha256:abc", None)
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
