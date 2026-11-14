use astrum_content::share;
use astrum_db::entity::{repository, share_link};
use astrum_testkit::TestDatabase;
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn list_puts_active_links_first_and_fills_the_rest_with_the_newest_expired_ones() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let created = Utc::now().fixed_offset();
    repository::Entity::insert(repository::ActiveModel {
        name: ActiveValue::Set("files".to_owned()),
        format: ActiveValue::Set("raw".to_owned()),
        kind: ActiveValue::Set("hosted".to_owned()),
        online: ActiveValue::Set(true),
        blob_store: ActiveValue::Set(None),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(created),
        updated_at: ActiveValue::Set(created),
    })
    .exec(db.conn())
    .await
    .unwrap();
    let now = Utc::now();
    let mut expired = Vec::new();
    let models: Vec<share_link::ActiveModel> = (0..1005)
        .map(|index| {
            let id = Uuid::new_v4();
            let active = index >= 1000;
            if !active {
                expired.push(id);
            }
            share_link::ActiveModel {
                id: ActiveValue::Set(id),
                secret_hash: ActiveValue::Set(format!("bulk-{id}")),
                repository: ActiveValue::Set("files".to_owned()),
                path: ActiveValue::Set("/notes/one.txt".to_owned()),
                prefix: ActiveValue::Set(false),
                note: ActiveValue::Set(String::new()),
                created_by: ActiveValue::Set("admin".to_owned()),
                created_source: ActiveValue::Set("default".to_owned()),
                created_at: ActiveValue::Set(
                    if active {
                        now - Duration::days(1) - Duration::seconds(index)
                    } else {
                        now - Duration::seconds(index)
                    }
                    .fixed_offset(),
                ),
                expires_at: ActiveValue::Set(
                    if active {
                        now + Duration::hours(1)
                    } else {
                        now - Duration::hours(1)
                    }
                    .fixed_offset(),
                ),
                max_downloads: ActiveValue::Set(None),
                downloads: ActiveValue::Set(0),
                bytes_served: ActiveValue::Set(0),
                last_used_at: ActiveValue::Set(None),
                last_used_ip: ActiveValue::Set(None),
            }
        })
        .collect();
    share_link::Entity::insert_many(models)
        .exec(db.conn())
        .await
        .unwrap();
    let listed = share::list(db.conn(), Some(("default", "admin")), None)
        .await
        .unwrap();
    assert_eq!(listed.len(), 1000);
    let current = Utc::now().fixed_offset();
    assert!(listed[..5].iter().all(|link| link.expires_at > current));
    assert!(listed[5..].iter().all(|link| link.expires_at <= current));
    assert_eq!(listed[5].id, expired[0]);
}
