#![allow(dead_code)]

use astrum_db::entity::{customer, repository};
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::{NewToken, Scope, Security, bootstrap};
use astrum_testkit::TestDatabase;
use sea_orm::{ActiveValue, EntityTrait};

pub struct World {
    pub _test: TestDatabase,
    pub db: astrum_db::Database,
    pub security: Security,
}

pub async fn world() -> World {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    bootstrap(db.conn(), data.path()).await.unwrap();
    let now = chrono::Utc::now().fixed_offset();
    customer::Entity::insert(customer::ActiveModel {
        id: ActiveValue::Set("acme".to_owned()),
        name: ActiveValue::Set("Acme".to_owned()),
        reference: ActiveValue::Set(String::new()),
        note: ActiveValue::Set(String::new()),
        disabled: ActiveValue::Set(false),
        created_by: ActiveValue::Set("admin".to_owned()),
        created_at: ActiveValue::Set(now),
    })
    .exec(db.conn())
    .await
    .unwrap();
    for name in ["raw-a", "raw-b"] {
        repository::Entity::insert(repository::ActiveModel {
            name: ActiveValue::Set(name.to_owned()),
            format: ActiveValue::Set("raw".to_owned()),
            kind: ActiveValue::Set("hosted".to_owned()),
            online: ActiveValue::Set(true),
            blob_store: ActiveValue::Set(None),
            attributes: ActiveValue::Set(serde_json::json!({})),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        })
        .exec(db.conn())
        .await
        .unwrap();
    }
    let security = Security::load(db.conn().clone()).await.unwrap();
    World {
        _test: test,
        db,
        security,
    }
}

pub fn customer_token(repositories: &[&str]) -> NewToken {
    NewToken {
        name: "release".to_owned(),
        scope: Scope::Read,
        days: Some(30),
        repositories: Some(repositories.iter().map(|name| (*name).to_owned()).collect()),
        customer: Some("acme".to_owned()),
        created_by: Some("admin".to_owned()),
    }
}

pub async fn issue(world: &World, repositories: &[&str]) -> String {
    world
        .security
        .create_token(ADMIN_USER, customer_token(repositories))
        .await
        .unwrap()
        .1
}
