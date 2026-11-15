mod support;
#[path = "support/namesake.rs"]
mod world;

use astrum_db::MIGRATIONS;
use astrum_db::entity::{access_token, security_oidc_user};
use astrum_security::oidc::{self, OidcUser};
use astrum_security::{
    AuthError, NewToken, OidcSignIn, Scope, Security, Subject, TokenError, realm, tokens,
};
use astrum_testkit::TestDatabase;
use axum::http::{Method, StatusCode};
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue, ConnectionTrait, DbBackend, EntityTrait, Statement};
use serde_json::{Value, json};
use uuid::Uuid;
use world::{World, setup_with};

const REALMS: &str = "/service/rest/v1/security/realms/active";

fn realms(oidc_on: bool) -> Value {
    let mut list = vec!["AstrumAuthenticatingRealm", "AccessTokenRealm", "LdapRealm"];
    if oidc_on {
        list.push("OAuth2Realm");
    }
    json!(list)
}

async fn external_world() -> World {
    let world = setup_with(realms(true)).await;
    world.fixture.state.security().set_external_tokens(true);
    world
}

async fn set_realms(world: &World, active: Value) {
    let reply = world
        .fixture
        .call(Method::PUT, REALMS, support::Auth::Admin, Some(active))
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn seed_oidc(world: &World, user_id: &str, subject: &str, days_ago: i64) {
    let security = world.fixture.state.security();
    let at = (Utc::now() - Duration::days(days_ago)).fixed_offset();
    security_oidc_user::Entity::insert(security_oidc_user::ActiveModel {
        user_id: ActiveValue::Set(user_id.to_owned()),
        subject: ActiveValue::Set(subject.to_owned()),
        first_name: ActiveValue::Set(String::new()),
        last_name: ActiveValue::Set(String::new()),
        email: ActiveValue::Set(String::new()),
        groups: ActiveValue::Set(json!(["developers"])),
        last_login_at: ActiveValue::Set(at),
        created_at: ActiveValue::Set(at),
    })
    .exec(security.db())
    .await
    .unwrap();
    security.changed().await.unwrap();
}

async fn age_oidc(world: &World, days_ago: i64) {
    let security = world.fixture.state.security();
    security
        .db()
        .execute_unprepared(&format!(
            "UPDATE security_oidc_user SET last_login_at = now() - interval '{days_ago} days'"
        ))
        .await
        .unwrap();
    security.changed().await.unwrap();
}

async fn sign_in(world: &World, user_id: &str, subject: &str) {
    let user = OidcUser {
        user_id: user_id.to_owned(),
        subject: subject.to_owned(),
        first_name: String::new(),
        last_name: String::new(),
        email: String::new(),
        groups: vec!["developers".to_owned()],
        last_login_at: Utc::now().fixed_offset(),
    };
    let accepted = world
        .fixture
        .state
        .security()
        .oidc_sign_in(&user)
        .await
        .unwrap();
    assert!(matches!(accepted, OidcSignIn::Accepted(_)));
}

async fn mint(security: &Security, source: &str, user_id: &str, bound: Option<&str>) -> String {
    let new = NewToken {
        source: source.to_owned(),
        external_subject: bound.map(str::to_owned),
        ..NewToken::personal("ci", Scope::Write, None)
    };
    security.create_token(user_id, new).await.unwrap().1
}

async fn insert_row(
    security: &Security,
    source: &str,
    user_id: &str,
    bound: Option<&str>,
    customer: Option<&str>,
) -> String {
    let token = format!(
        "astrum_{}{}",
        Uuid::new_v4().simple(),
        &Uuid::new_v4().simple().to_string()[..8]
    );
    let now = Utc::now().fixed_offset();
    access_token::Entity::insert(access_token::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(user_id.to_owned()),
        source: ActiveValue::Set(source.to_owned()),
        name: ActiveValue::Set("row".to_owned()),
        token_hash: ActiveValue::Set(tokens::hash(&token)),
        token_hint: ActiveValue::Set(token[..11].to_owned()),
        scope: ActiveValue::Set("READ".to_owned()),
        created_at: ActiveValue::Set(now),
        expires_at: ActiveValue::Set(Some(now + Duration::days(30))),
        last_used_at: ActiveValue::Set(None),
        customer: ActiveValue::Set(customer.map(str::to_owned)),
        repositories: ActiveValue::Set(customer.map(|_| vec!["crates".to_owned()])),
        created_by: ActiveValue::Set(None),
        last_used_ip: ActiveValue::Set(None),
        expiry_notice: ActiveValue::Set(None),
        external_subject: ActiveValue::Set(bound.map(str::to_owned)),
    })
    .exec(security.db())
    .await
    .unwrap();
    token
}

async fn resolve(world: &World, token: &str) -> Result<Subject, AuthError> {
    world
        .fixture
        .state
        .security()
        .authenticate_token(None, token, None)
        .await
}

fn refused(result: Result<Subject, AuthError>) -> bool {
    matches!(result, Err(AuthError::BadCredentials))
}

#[tokio::test]
async fn an_oidc_token_acts_with_the_roles_of_its_groups_in_the_token_realm() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    let token = mint(
        world.fixture.state.security(),
        oidc::SOURCE,
        "rita",
        Some("sub-rita"),
    )
    .await;
    let subject = resolve(&world, &token).await.unwrap();
    assert_eq!(subject.user_id, "rita");
    assert_eq!(subject.source, oidc::SOURCE);
    assert_eq!(subject.realm, realm::ACCESS_TOKEN);
    assert_eq!(
        subject.roles.iter().map(String::as_str).collect::<Vec<_>>(),
        ["developers"]
    );
    assert!(subject.is_permitted("astrum:repository-view:cargo:crates:read"));
    assert!(subject.token.is_some());
}

#[tokio::test]
async fn an_oidc_token_dies_when_a_local_user_takes_its_id_in_any_case() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    seed_oidc(&world, "sam", "sub-sam", 1).await;
    let security = world.fixture.state.security();
    let rita = mint(security, oidc::SOURCE, "rita", Some("sub-rita")).await;
    let sam = mint(security, oidc::SOURCE, "sam", Some("sub-sam")).await;
    assert!(resolve(&world, &rita).await.is_ok());
    assert!(resolve(&world, &sam).await.is_ok());
    world
        .fixture
        .add_user("rita", "local-rita-password", &["as-admin"])
        .await;
    world
        .fixture
        .add_user("SAM", "local-sam-password", &["as-admin"])
        .await;
    assert!(refused(resolve(&world, &rita).await));
    assert!(refused(resolve(&world, &sam).await));
}

#[tokio::test]
async fn a_default_token_of_a_user_who_is_not_local_never_resolves_an_external_user() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    let security = world.fixture.state.security();
    for (user, bound) in [("rita", Some("sub-rita")), ("alice", None)] {
        let token = mint(security, "default", user, bound).await;
        assert!(refused(resolve(&world, &token).await), "{user}");
    }
}

#[tokio::test]
async fn an_oidc_token_is_bound_to_the_subject_it_was_created_for() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-old", 1).await;
    let security = world.fixture.state.security();
    let bound = mint(security, oidc::SOURCE, "rita", Some("sub-old")).await;
    let unbound = insert_row(security, oidc::SOURCE, "rita", None, None).await;
    assert!(resolve(&world, &bound).await.is_ok());
    assert!(refused(resolve(&world, &unbound).await));
    sign_in(&world, "rita", "sub-new").await;
    assert!(refused(resolve(&world, &bound).await));
}

#[tokio::test]
async fn an_oidc_token_needs_a_sign_in_within_ninety_days() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 89).await;
    let token = mint(
        world.fixture.state.security(),
        oidc::SOURCE,
        "rita",
        Some("sub-rita"),
    )
    .await;
    assert!(resolve(&world, &token).await.is_ok());
    age_oidc(&world, 91).await;
    assert!(refused(resolve(&world, &token).await));
    sign_in(&world, "rita", "sub-rita").await;
    assert!(resolve(&world, &token).await.is_ok());
}

#[tokio::test]
async fn an_oidc_token_is_refused_while_the_oidc_realm_is_off() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    let token = mint(
        world.fixture.state.security(),
        oidc::SOURCE,
        "rita",
        Some("sub-rita"),
    )
    .await;
    assert!(resolve(&world, &token).await.is_ok());
    set_realms(&world, realms(false)).await;
    assert!(refused(resolve(&world, &token).await));
    set_realms(&world, realms(true)).await;
    assert!(resolve(&world, &token).await.is_ok());
}

#[tokio::test]
async fn an_ldap_token_resolves_through_the_directory() {
    let world = external_world().await;
    let security = world.fixture.state.security();
    let alice = mint(security, "LDAP", "alice", None).await;
    let ghost = mint(security, "LDAP", "ghost", None).await;
    let subject = resolve(&world, &alice).await.unwrap();
    assert_eq!(subject.user_id, "alice");
    assert_eq!(subject.source, "LDAP");
    assert_eq!(subject.realm, realm::ACCESS_TOKEN);
    assert!(subject.roles.contains("developers"));
    assert!(subject.roles.contains("admins"));
    assert!(refused(resolve(&world, &ghost).await));
}

#[tokio::test]
async fn customer_tokens_keep_working_and_never_take_another_source() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    let security = world.fixture.state.security();
    let (_, plain) = world.fixture.customer_token("acme", &["crates"]).await;
    let subject = resolve(&world, &plain).await.unwrap();
    assert_eq!(subject.realm, realm::ACCESS_TOKEN);
    assert_eq!(subject.token.unwrap().customer.as_deref(), Some("acme"));
    let token = insert_row(
        security,
        oidc::SOURCE,
        "rita",
        Some("sub-rita"),
        Some("acme"),
    )
    .await;
    assert!(refused(resolve(&world, &token).await));
}

#[tokio::test]
async fn external_tokens_are_refused_while_the_server_flag_is_off() {
    let world = setup_with(realms(true)).await;
    let security = world.fixture.state.security();
    assert!(!security.external_tokens_enabled());
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    security.set_external_tokens(true);
    let oidc_token = mint(security, oidc::SOURCE, "rita", Some("sub-rita")).await;
    let ldap_token = mint(security, "LDAP", "alice", None).await;
    let local_token = mint(security, "default", "bob", None).await;
    for token in [&oidc_token, &ldap_token, &local_token] {
        assert!(resolve(&world, token).await.is_ok());
    }
    security.set_external_tokens(false);
    assert!(refused(resolve(&world, &oidc_token).await));
    assert!(refused(resolve(&world, &ldap_token).await));
    let local = resolve(&world, &local_token).await.unwrap();
    assert_eq!(local.user_id, "bob");
    assert_eq!(local.source, "default");
}

#[tokio::test]
async fn create_token_refuses_rows_that_could_never_authenticate() {
    let world = external_world().await;
    let security = world.fixture.state.security();
    let draft = |source: &str, bound: Option<&str>, customer: Option<&str>| NewToken {
        source: source.to_owned(),
        external_subject: bound.map(str::to_owned),
        customer: customer.map(str::to_owned),
        repositories: customer.map(|_| vec!["crates".to_owned()]),
        ..NewToken::personal("ci", Scope::Read, Some(30))
    };
    let created = |draft: NewToken| async { security.create_token("rita", draft).await };
    assert!(matches!(
        created(draft("SAML", None, None)).await,
        Err(TokenError::UnknownSource)
    ));
    assert!(matches!(
        created(draft(oidc::SOURCE, None, None)).await,
        Err(TokenError::MissingSubject)
    ));
    assert!(matches!(
        created(draft(oidc::SOURCE, Some(""), None)).await,
        Err(TokenError::MissingSubject)
    ));
    assert!(matches!(
        created(draft("LDAP", None, Some("acme"))).await,
        Err(TokenError::ExternalCustomer)
    ));
    assert!(
        created(draft(oidc::SOURCE, Some("sub-rita"), None))
            .await
            .is_ok()
    );
    assert!(created(draft("LDAP", None, None)).await.is_ok());
    security.set_external_tokens(false);
    assert!(matches!(
        created(draft("LDAP", None, None)).await,
        Err(TokenError::ExternalOff)
    ));
    assert!(matches!(
        created(draft(oidc::SOURCE, Some("sub-rita"), None)).await,
        Err(TokenError::ExternalOff)
    ));
    assert!(created(draft("default", None, None)).await.is_ok());
}

#[tokio::test]
async fn an_ldap_token_never_reaches_a_local_user_or_another_spelling_of_a_directory_user() {
    let world = external_world().await;
    let security = world.fixture.state.security();
    let alice = mint(security, "LDAP", "alice", None).await;
    let carol = mint(security, "LDAP", "carol", None).await;
    assert!(resolve(&world, &alice).await.is_ok());
    assert!(resolve(&world, &carol).await.is_ok());
    for user in ["bob", "BOB", "ALICE"] {
        let token = mint(security, "LDAP", user, None).await;
        assert!(refused(resolve(&world, &token).await), "{user}");
    }
    set_realms(
        &world,
        json!(["AstrumAuthenticatingRealm", "AccessTokenRealm"]),
    )
    .await;
    assert!(refused(resolve(&world, &alice).await));
    set_realms(&world, realms(true)).await;
    assert!(resolve(&world, &alice).await.is_ok());
    let dave = mint(security, "LDAP", "dave", None).await;
    world.directory.stop();
    assert!(refused(resolve(&world, &dave).await));
}

#[tokio::test]
async fn a_new_local_user_clears_the_external_tokens_of_its_namesakes_in_any_case_for_good() {
    let world = external_world().await;
    seed_oidc(&world, "rita", "sub-rita", 1).await;
    let security = world.fixture.state.security();
    let oidc_token = mint(security, oidc::SOURCE, "rita", Some("sub-rita")).await;
    mint(security, "LDAP", "alice", None).await;
    assert!(resolve(&world, &oidc_token).await.is_ok());
    let body = json!({"userId": "RITA", "firstName": "Rita", "lastName": "Tester",
        "emailAddress": "rita@example.org", "password": "local-rita-password", "status": "active",
        "roles": []});
    let users = "/service/rest/v1/security/users";
    let created = world
        .fixture
        .call(Method::POST, users, support::Auth::Admin, Some(body))
        .await;
    assert_eq!(created.status, StatusCode::OK, "{}", created.text());
    assert_eq!(
        security
            .tokens_of(oidc::SOURCE, "rita")
            .await
            .unwrap()
            .len(),
        0
    );
    assert_eq!(security.tokens_of("LDAP", "alice").await.unwrap().len(), 1);
    let removed = world
        .fixture
        .call(
            Method::DELETE,
            &format!("{users}/RITA"),
            support::Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert!(refused(resolve(&world, &oidc_token).await));
    assert_eq!(
        security
            .tokens_of(oidc::SOURCE, "rita")
            .await
            .unwrap()
            .len(),
        0
    );

    mint(security, "LDAP", "ALICE", None).await;
    mint(security, "default", "Zed", None).await;
    assert_eq!(
        security.delete_tokens_of("default", "zed").await.unwrap(),
        0
    );
    assert_eq!(security.delete_tokens_of("LDAP", "Alice").await.unwrap(), 2);
    assert_eq!(security.tokens_of("default", "Zed").await.unwrap().len(), 1);

    mint(security, oidc::SOURCE, "rita", Some("sub-rita")).await;
    mint(security, oidc::SOURCE, "Rita", Some("sub-big-rita")).await;
    assert_eq!(
        security
            .delete_tokens_of(oidc::SOURCE, "Rita")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        security
            .tokens_of(oidc::SOURCE, "rita")
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        security
            .tokens_of(oidc::SOURCE, "Rita")
            .await
            .unwrap()
            .len(),
        0
    );
}

async fn names(conn: &sea_orm::DatabaseConnection) -> Vec<String> {
    conn.query_all(Statement::from_string(
        DbBackend::Postgres,
        "SELECT name FROM access_token ORDER BY name".to_owned(),
    ))
    .await
    .unwrap()
    .into_iter()
    .map(|row| row.try_get::<String>("", "name").unwrap())
    .collect()
}

#[tokio::test]
async fn the_migration_clears_orphan_default_rows_and_keeps_local_and_customer_rows() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let since = MIGRATIONS
        .all()
        .unwrap()
        .iter()
        .filter(|migration| migration.stamp >= 1_794_810_000_000)
        .count();
    assert_eq!(MIGRATIONS.revert(conn, since).await.unwrap().len(), since);
    conn.execute_unprepared(
        "INSERT INTO security_user (id, status) VALUES ('lena', 'active'); \
         INSERT INTO customer (id, name, created_by) VALUES ('acme', 'Acme', 'admin')",
    )
    .await
    .unwrap();
    let token = |name: &str, user: &str, customer: &str, repositories: &str, scope: &str| {
        format!(
            "INSERT INTO access_token (id, user_id, name, token_hash, token_hint, scope, customer, repositories, expires_at) \
             VALUES (gen_random_uuid(), '{user}', '{name}', md5('{name}'), 'astrum_xxxx', '{scope}', {customer}, {repositories}, now() + interval '1 day')"
        )
    };
    for sql in [
        token("orphan", "ghost", "NULL", "NULL", "WRITE"),
        token("local", "lena", "NULL", "NULL", "WRITE"),
        token("customer", "ghost", "'acme'", "ARRAY['a']", "READ"),
    ] {
        conn.execute_unprepared(&sql).await.unwrap();
    }
    assert_eq!(db.migrate().await.unwrap().len(), since);
    assert_eq!(names(conn).await, ["customer", "local"]);
    let unbound = conn
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT count(*) AS n FROM access_token WHERE external_subject IS NULL".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unbound.try_get::<i64>("", "n").unwrap(), 2);
}
