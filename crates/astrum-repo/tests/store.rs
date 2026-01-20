use astrum_db::entity::{blob_store, repository};
use astrum_repo::{Repository, Request, Spec, WriteError, Writer, store};
use astrum_testkit::TestDatabase;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use serde_json::{Value, json};

async fn ready() -> (TestDatabase, astrum_db::Database) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let now = chrono::Utc::now().fixed_offset();
    blob_store::Entity::insert(blob_store::ActiveModel {
        name: ActiveValue::Set("default".to_owned()),
        kind: ActiveValue::Set("File".to_owned()),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(db.conn())
    .await
    .unwrap();
    (test, db)
}

fn prepared(format: &str, kind: &str, body: Value) -> Repository {
    let request: Request = serde_json::from_value(body).unwrap();
    Repository::prepare(
        Spec::from_api(format, kind).unwrap(),
        request,
        None,
        &|plain: &str| format!("sealed:{plain}"),
    )
    .unwrap()
}

fn hosted(name: &str) -> Repository {
    prepared(
        "maven",
        "hosted",
        json!({
            "name": name, "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"},
        }),
    )
}

fn group(name: &str, members: &[&str]) -> Repository {
    prepared(
        "maven",
        "group",
        json!({
            "name": name, "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": members},
        }),
    )
}

async fn create(db: &DatabaseConnection, repository: Repository) {
    Writer::begin(db)
        .await
        .unwrap()
        .create(repository)
        .await
        .unwrap();
}

#[tokio::test]
async fn rows_keep_the_format_id_and_the_blob_store_in_columns() {
    let (_test, db) = ready().await;
    create(db.conn(), hosted("Releases")).await;
    let row = repository::Entity::find_by_id("Releases".to_owned())
        .one(db.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (row.format.as_str(), row.kind.as_str(), row.online),
        ("maven2", "hosted", true)
    );
    assert_eq!(row.blob_store.as_deref(), Some("default"));
    assert_eq!(
        row.attributes["storage"],
        json!({"strictContentTypeValidation": true, "writePolicy": "ALLOW"})
    );
    let found = store::find(db.conn(), "releases").await.unwrap().unwrap();
    assert_eq!(found, hosted("Releases"));
    assert!(store::find(db.conn(), "snapshots").await.unwrap().is_none());
}

#[tokio::test]
async fn deleting_a_member_takes_it_out_of_every_group() {
    let (_test, db) = ready().await;
    create(db.conn(), hosted("releases")).await;
    create(db.conn(), hosted("snapshots")).await;
    create(db.conn(), group("public", &["RELEASES", "snapshots"])).await;
    create(db.conn(), group("everything", &["public", "releases"])).await;
    let deleted = Writer::begin(db.conn())
        .await
        .unwrap()
        .delete("Releases")
        .await
        .unwrap();
    assert_eq!(deleted.name, "releases");
    let names: Vec<(String, Vec<String>)> = store::list(db.conn())
        .await
        .unwrap()
        .into_iter()
        .map(|repository| {
            let members = repository
                .group()
                .map(|group| group.members().to_vec())
                .unwrap_or_default();
            (repository.name, members)
        })
        .collect();
    assert_eq!(
        names,
        [
            ("everything".to_owned(), vec!["public".to_owned()]),
            ("public".to_owned(), vec!["snapshots".to_owned()]),
            ("snapshots".to_owned(), vec![]),
        ]
    );
    let missing = Writer::begin(db.conn())
        .await
        .unwrap()
        .delete("releases")
        .await;
    assert!(matches!(missing, Err(WriteError::NotFound(_))));
}

#[tokio::test]
async fn a_row_this_version_cannot_read_is_reported() {
    let (_test, db) = ready().await;
    let now = chrono::Utc::now().fixed_offset();
    repository::Entity::insert(repository::ActiveModel {
        name: ActiveValue::Set("frontend".to_owned()),
        format: ActiveValue::Set("bower".to_owned()),
        kind: ActiveValue::Set("hosted".to_owned()),
        online: ActiveValue::Set(true),
        blob_store: ActiveValue::Set(None),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(db.conn())
    .await
    .unwrap();
    let error = store::list(db.conn()).await.unwrap_err();
    assert_eq!(
        error.to_string(),
        "the stored repository \"frontend\" cannot be read: unknown format \"bower\""
    );
}

#[tokio::test]
async fn updates_check_against_the_other_repositories() {
    let (_test, db) = ready().await;
    create(db.conn(), hosted("releases")).await;
    create(db.conn(), group("public", &["releases"])).await;
    create(db.conn(), group("outer", &["public"])).await;
    let cycle = Writer::begin(db.conn())
        .await
        .unwrap()
        .update("public", group("public", &["releases", "outer"]))
        .await;
    let Err(WriteError::Invalid(problems)) = cycle else {
        panic!("{cycle:?}");
    };
    assert_eq!(problems[0].id, "PARAMETER memberNames");
    let renamed = Writer::begin(db.conn())
        .await
        .unwrap()
        .update("public", group("PUBLIC", &["releases"]))
        .await;
    assert!(matches!(renamed, Err(WriteError::Refused(_))));
    let mut offline = group("public", &["releases"]);
    offline.online = false;
    Writer::begin(db.conn())
        .await
        .unwrap()
        .update("PUBLIC", offline)
        .await
        .unwrap();
    assert!(
        !store::find(db.conn(), "public")
            .await
            .unwrap()
            .unwrap()
            .online
    );
}
