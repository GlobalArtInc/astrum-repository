use astrum_db::entity::{repository, security_user};
use astrum_db::revision;
use astrum_security::bootstrap::ADMIN_PASSWORD_FILE;
use astrum_security::catalogue::{ADMIN_USER, ANONYMOUS_USER};
use astrum_security::model::{ANONYMOUS_KEY, AnonymousSettings};
use astrum_security::{AuthError, Security, bootstrap, password};
use astrum_testkit::TestDatabase;
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::json;

async fn ready() -> (TestDatabase, astrum_db::Database, tempfile::TempDir) {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    (test, db, data)
}

#[tokio::test]
async fn a_fresh_installation_gets_an_admin_with_a_password_file() {
    let (_test, db, data) = ready().await;
    let first = bootstrap(db.conn(), data.path()).await.unwrap();
    let file = first.admin_password_file.unwrap();
    assert_eq!(file, data.path().join(ADMIN_PASSWORD_FILE));
    let password = std::fs::read_to_string(&file).unwrap();
    assert_eq!(password.len(), 24);

    let second = bootstrap(db.conn(), data.path()).await.unwrap();
    assert!(
        second.admin_password_file.is_none(),
        "users exist, nothing is created again"
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), password);

    let security = Security::load(db.conn().clone()).await.unwrap();
    let admin = security.authenticate(ADMIN_USER, &password).await.unwrap();
    assert!(admin.is_permitted("astrum:users:delete"));
    assert!(admin.is_permitted("astrum:repository-view:maven2:anything:add"));
    assert!(!admin.anonymous);
    assert!(matches!(
        security.authenticate(ADMIN_USER, "wrong").await,
        Err(AuthError::BadCredentials)
    ));
    assert!(matches!(
        security.authenticate(ANONYMOUS_USER, "").await,
        Err(AuthError::BadCredentials)
    ));
    assert!(matches!(
        security.authenticate("nobody", &password).await,
        Err(AuthError::BadCredentials)
    ));
}

#[tokio::test]
async fn anonymous_access_is_off_until_switched_on() {
    let (_test, db, data) = ready().await;
    bootstrap(db.conn(), data.path()).await.unwrap();
    let security = Security::load(db.conn().clone()).await.unwrap();
    assert!(security.anonymous().is_none());

    astrum_db::setting::put(
        db.conn(),
        ANONYMOUS_KEY,
        &AnonymousSettings {
            enabled: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        security.anonymous().is_none(),
        "the model changes when a change is announced"
    );
    security.changed().await.unwrap();
    let anonymous = security.anonymous().unwrap();
    assert!(anonymous.anonymous);
    assert!(anonymous.is_permitted("astrum:repository-view:npm:npm-proxy:read"));
    assert!(anonymous.is_permitted("astrum:search:read"));
    assert!(!anonymous.is_permitted("astrum:repository-view:npm:npm-proxy:add"));
    assert!(!anonymous.is_permitted("astrum:users:read"));
}

#[tokio::test]
async fn passwords_from_the_old_server_are_upgraded_on_sign_in() {
    let (_test, db, data) = ready().await;
    bootstrap(db.conn(), data.path()).await.unwrap();
    let legacy = "$shiro1$SHA-512$1024$c2FsdHNhbHRzYWx0c2FsdA==$d2hl1gll86cw2smEFHHtXiCtq5zRJ7Q2aZGMiF/B4XRiEoAQtM4mj7bhkGA5Z8+TozMb0JLk+wlZCeIDpuhy8Q==";
    security_user::Entity::update(security_user::ActiveModel {
        id: ActiveValue::Unchanged(ADMIN_USER.to_owned()),
        password_hash: ActiveValue::Set(Some(legacy.to_owned())),
        ..Default::default()
    })
    .exec(db.conn())
    .await
    .unwrap();
    let security = Security::load(db.conn().clone()).await.unwrap();
    assert_eq!(
        password::verify(legacy, "admin123"),
        password::Verified::MatchButOutdated
    );
    security.authenticate(ADMIN_USER, "admin123").await.unwrap();
    let stored = security_user::Entity::find_by_id(ADMIN_USER.to_owned())
        .one(db.conn())
        .await
        .unwrap()
        .unwrap()
        .password_hash
        .unwrap();
    assert!(stored.starts_with("$argon2id$"), "{stored}");
    security.authenticate(ADMIN_USER, "admin123").await.unwrap();
}

#[tokio::test]
async fn every_repository_brings_its_own_privileges() {
    let (_test, db, data) = ready().await;
    bootstrap(db.conn(), data.path()).await.unwrap();
    astrum_db::entity::blob_store::Entity::insert(astrum_db::entity::blob_store::ActiveModel {
        name: ActiveValue::Set("default".to_owned()),
        kind: ActiveValue::Set("file".to_owned()),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    })
    .exec(db.conn())
    .await
    .unwrap();
    repository::Entity::insert(repository::ActiveModel {
        name: ActiveValue::Set("maven-releases".to_owned()),
        format: ActiveValue::Set("maven2".to_owned()),
        kind: ActiveValue::Set("hosted".to_owned()),
        online: ActiveValue::Set(true),
        blob_store: ActiveValue::Set(Some("default".to_owned())),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    })
    .exec(db.conn())
    .await
    .unwrap();
    let security = Security::load(db.conn().clone()).await.unwrap();
    let model = security.model();
    assert!(
        model
            .privileges
            .contains_key("as-repository-view-maven2-maven-releases-read")
    );
    assert!(
        model
            .privileges
            .contains_key("as-repository-admin-maven2-maven-releases-edit")
    );
    assert!(
        model
            .privileges
            .contains_key("as-repository-view-yum-*-browse")
    );
    assert_eq!(model.roles.len(), 2);
}

async fn put_user(db: &astrum_db::Database, user_id: &str, password: &str) {
    let now = chrono::Utc::now().fixed_offset();
    security_user::Entity::insert(security_user::ActiveModel {
        id: ActiveValue::Set(user_id.to_owned()),
        first_name: ActiveValue::Set(user_id.to_owned()),
        last_name: ActiveValue::Set("Tester".to_owned()),
        email: ActiveValue::Set(format!("{user_id}@example.org")),
        password_hash: ActiveValue::Set(Some(password::hash(password))),
        status: ActiveValue::Set("active".to_owned()),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .on_conflict(
        sea_orm::sea_query::OnConflict::column(security_user::Column::Id)
            .update_column(security_user::Column::PasswordHash)
            .to_owned(),
    )
    .exec(db.conn())
    .await
    .unwrap();
}

#[tokio::test]
async fn a_change_announced_by_one_server_reaches_another_once() {
    let (_test, db, data) = ready().await;
    bootstrap(db.conn(), data.path()).await.unwrap();
    let here = Security::load(db.conn().clone()).await.unwrap();
    let there = Security::load(db.conn().clone()).await.unwrap();
    assert!(!there.sync().await.unwrap(), "nothing changed yet");

    put_user(&db, "carol", "first-secret").await;
    here.changed().await.unwrap();
    assert!(here.model().users.contains_key("carol"));
    assert!(!there.model().users.contains_key("carol"));
    let announced = revision::current(db.conn(), revision::SECURITY)
        .await
        .unwrap();

    assert!(there.sync().await.unwrap(), "the other server reloads");
    assert!(there.model().users.contains_key("carol"));
    assert!(!there.sync().await.unwrap(), "and only once");
    assert!(
        !here.sync().await.unwrap(),
        "the server that announced it knows"
    );
    assert_eq!(
        revision::current(db.conn(), revision::SECURITY)
            .await
            .unwrap(),
        announced,
        "catching up announces nothing"
    );
}

#[tokio::test]
async fn a_server_that_missed_a_change_checks_before_it_refuses() {
    let (_test, db, data) = ready().await;
    bootstrap(db.conn(), data.path()).await.unwrap();
    let here = Security::load(db.conn().clone()).await.unwrap();
    let there = Security::load(db.conn().clone()).await.unwrap();

    put_user(&db, "dave", "first-secret").await;
    here.changed().await.unwrap();
    let dave = there
        .authenticate("dave", "first-secret")
        .await
        .expect("a user created through another server can sign in at once");
    assert_eq!(dave.user_id, "dave");

    put_user(&db, "dave", "second-secret").await;
    here.changed().await.unwrap();
    let subject = here.authenticate("dave", "second-secret").await.unwrap();
    let session = here
        .open_session(&subject, "127.0.0.1", "test")
        .await
        .unwrap();
    let kept = there
        .session(&session.token)
        .await
        .unwrap()
        .expect("the session opened after the password change stays valid");
    assert_eq!(kept.subject.user_id, "dave");
    assert!(here.session(&session.token).await.unwrap().is_some());
    assert!(matches!(
        there.authenticate("dave", "first-secret").await,
        Err(AuthError::BadCredentials)
    ));
}
