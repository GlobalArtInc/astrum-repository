mod support;

use std::sync::Arc;

use astrum_db::entity::{oauth_client, oauth_grant};
use astrum_security::ldap::TlsClient;
use astrum_testkit::ldap::{BASE, LdapTestServer, SYSTEM_DN, SYSTEM_PASSWORD};
use axum::http::{Method, StatusCode};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const TOKENS: &str = "/service/rest/internal/access-tokens";
const NUGET_KEY: &str = "/service/rest/internal/ui/user/nuget-api-key";
const GRANTS: &str = "/service/rest/internal/ui/oauth/grants";
const CARGO_ME: &str = "/repository/crates/me";
const GEMS_KEY: &str = "/repository/gems/api/v1/api_key";
const REALMS: &str = "/service/rest/v1/security/realms/active";
const LOCAL_PASSWORD: &str = "local-bob-password";
const DIRECTORY_PASSWORD: &str = "bob-secret";
const ALICE_PASSWORD: &str = "alice-secret";

struct World {
    fixture: Fixture,
    _directory: LdapTestServer,
}

fn server(port: u16) -> Value {
    json!({
        "name": "corp", "protocol": "ldap", "host": "127.0.0.1", "port": port,
        "searchBase": BASE, "authScheme": "SIMPLE", "authUsername": SYSTEM_DN,
        "authPassword": SYSTEM_PASSWORD, "connectionTimeoutSeconds": 5,
        "userBaseDn": "ou=people", "userSubtree": true, "userObjectClass": "inetOrgPerson",
        "userIdAttribute": "uid", "userRealNameAttribute": "cn",
        "userEmailAddressAttribute": "mail", "ldapGroupsAsRoles": true,
        "groupType": "dynamic", "userMemberOfAttribute": "memberOf"
    })
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name, "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "cargo": {"requireAuthentication": false}
    })
}

async fn expect(fixture: &Fixture, method: Method, path: &str, body: Value, status: StatusCode) {
    let reply = fixture.call(method, path, Auth::Admin, Some(body)).await;
    assert_eq!(reply.status, status, "{}", reply.text());
}

async fn setup_with(realms: Value) -> World {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_cargo::Cargo::default()));
    content.register(Arc::new(astrum_format_rubygems::Rubygems));
    let directory = LdapTestServer::start().await;
    fixture
        .state
        .security()
        .ldap()
        .set_tls(TlsClient::with_roots(vec![directory.ca()]));
    let ldap = "/service/rest/v1/security/ldap";
    expect(
        &fixture,
        Method::POST,
        ldap,
        server(directory.port),
        StatusCode::CREATED,
    )
    .await;
    let role = json!({"id": "developers", "name": "Developers", "source": "LDAP",
        "privileges": ["as-apikey-all", "as-repository-view-*-*-read"]});
    let roles = "/service/rest/v1/security/roles";
    expect(&fixture, Method::POST, roles, role, StatusCode::OK).await;
    expect(
        &fixture,
        Method::PUT,
        REALMS,
        realms,
        StatusCode::NO_CONTENT,
    )
    .await;
    for (kind, name) in [("cargo", "crates"), ("rubygems", "gems")] {
        let path = format!("/service/rest/v1/repositories/{kind}/hosted");
        expect(
            &fixture,
            Method::POST,
            &path,
            hosted(name),
            StatusCode::CREATED,
        )
        .await;
    }
    fixture.add_user("bob", LOCAL_PASSWORD, &["as-admin"]).await;
    World {
        fixture,
        _directory: directory,
    }
}

fn local_first() -> Value {
    json!([
        "AstrumAuthenticatingRealm",
        "AccessTokenRealm",
        "LdapRealm",
        "CargoToken",
        "NuGetApiKey",
        "RubyGemsApiKey"
    ])
}

async fn setup() -> World {
    setup_with(local_first()).await
}

impl World {
    async fn answer(
        &self,
        wanted: StatusCode,
        method: Method,
        path: &str,
        who: Auth<'_>,
        body: Option<Value>,
    ) -> Reply {
        let reply = self.fixture.call(method.clone(), path, who, body).await;
        assert_eq!(reply.status, wanted, "{method} {path}: {}", reply.text());
        reply
    }

    async fn local_token(&self) -> Value {
        let body = json!({"name": "mine", "scope": "WRITE", "expiresInDays": null});
        let reply = self
            .answer(
                StatusCode::CREATED,
                Method::POST,
                TOKENS,
                local(),
                Some(body),
            )
            .await;
        reply.json()
    }

    async fn local_nuget_key(&self) -> String {
        let body = json!({"password": LOCAL_PASSWORD});
        let reply = self
            .answer(StatusCode::OK, Method::POST, NUGET_KEY, local(), Some(body))
            .await;
        reply.json()["apiKey"].as_str().unwrap().to_owned()
    }

    async fn local_grant(&self) -> uuid::Uuid {
        let db = self.fixture.state.db().conn();
        let now = chrono::Utc::now().fixed_offset();
        oauth_client::Entity::insert(oauth_client::ActiveModel {
            id: ActiveValue::Set("agent".to_owned()),
            name: ActiveValue::Set("Agent".to_owned()),
            redirect_uris: ActiveValue::Set(json!(["http://127.0.0.1/cb"])),
            created_at: ActiveValue::Set(now),
        })
        .exec(db)
        .await
        .unwrap();
        let id = uuid::Uuid::new_v4();
        oauth_grant::Entity::insert(oauth_grant::ActiveModel {
            id: ActiveValue::Set(id),
            client_id: ActiveValue::Set("agent".to_owned()),
            user_id: ActiveValue::Set("bob".to_owned()),
            realm: ActiveValue::Set("AstrumAuthenticatingRealm".to_owned()),
            scope: ActiveValue::Set("read".to_owned()),
            resource: ActiveValue::Set(None),
            access_hash: ActiveValue::Set("a".to_owned()),
            access_expires_at: ActiveValue::Set(now),
            refresh_hash: ActiveValue::Set("r".to_owned()),
            refresh_expires_at: ActiveValue::Set(now + chrono::Duration::days(1)),
            created_at: ActiveValue::Set(now),
            last_used_at: ActiveValue::Set(None),
        })
        .exec(db)
        .await
        .unwrap();
        id
    }
}

fn namesake() -> Auth<'static> {
    Auth::Basic("bob", DIRECTORY_PASSWORD)
}

fn local() -> Auth<'static> {
    Auth::Basic("bob", LOCAL_PASSWORD)
}

fn alice() -> Auth<'static> {
    Auth::Basic("alice", ALICE_PASSWORD)
}

const REFUSED: StatusCode = StatusCode::UNAUTHORIZED;

#[tokio::test]
async fn a_namesake_cannot_create_access_tokens_of_the_local_user() {
    let world = setup().await;
    let body = json!({"name": "stolen", "scope": "WRITE", "expiresInDays": null});
    world
        .answer(REFUSED, Method::POST, TOKENS, namesake(), Some(body))
        .await;
    let listed = world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
    assert_eq!(listed.json().as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn a_namesake_cannot_list_the_access_tokens_of_the_local_user() {
    let world = setup().await;
    world.local_token().await;
    world
        .answer(REFUSED, Method::GET, TOKENS, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_delete_the_access_tokens_of_the_local_user() {
    let world = setup().await;
    let id = world.local_token().await["id"].as_str().unwrap().to_owned();
    let path = format!("{TOKENS}/{id}");
    world
        .answer(REFUSED, Method::DELETE, &path, namesake(), None)
        .await;
    let listed = world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_namesake_cannot_read_the_nuget_key_of_the_local_user() {
    let world = setup().await;
    world.local_nuget_key().await;
    let body = json!({"password": DIRECTORY_PASSWORD});
    world
        .answer(REFUSED, Method::POST, NUGET_KEY, namesake(), Some(body))
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_reset_the_nuget_key_of_the_local_user() {
    let world = setup().await;
    let key = world.local_nuget_key().await;
    let body = json!({"password": DIRECTORY_PASSWORD});
    let path = format!("{NUGET_KEY}/reset");
    world
        .answer(REFUSED, Method::POST, &path, namesake(), Some(body))
        .await;
    assert_eq!(world.local_nuget_key().await, key);
}

#[tokio::test]
async fn a_namesake_gets_no_cargo_token_of_the_local_user() {
    let world = setup().await;
    world
        .answer(REFUSED, Method::GET, CARGO_ME, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_gets_no_rubygems_key_of_the_local_user() {
    let world = setup().await;
    world
        .answer(StatusCode::OK, Method::GET, GEMS_KEY, local(), None)
        .await;
    world
        .answer(REFUSED, Method::GET, GEMS_KEY, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_revoke_the_rubygems_key_of_the_local_user() {
    let world = setup().await;
    world
        .answer(StatusCode::OK, Method::GET, GEMS_KEY, local(), None)
        .await;
    world
        .answer(REFUSED, Method::DELETE, GEMS_KEY, namesake(), None)
        .await;
    world
        .answer(StatusCode::OK, Method::DELETE, GEMS_KEY, local(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_list_the_oauth_grants_of_the_local_user() {
    let world = setup().await;
    world.local_grant().await;
    world
        .answer(REFUSED, Method::GET, GRANTS, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_namesake_cannot_revoke_the_oauth_grants_of_the_local_user() {
    let world = setup().await;
    let id = world.local_grant().await;
    let path = format!("{GRANTS}/{id}");
    world
        .answer(REFUSED, Method::DELETE, &path, namesake(), None)
        .await;
    world
        .answer(StatusCode::NO_CONTENT, Method::DELETE, &path, local(), None)
        .await;
}

#[tokio::test]
async fn the_rule_holds_when_the_directory_realm_comes_first() {
    let realms = json!(["LdapRealm", "AstrumAuthenticatingRealm", "AccessTokenRealm"]);
    let world = setup_with(realms).await;
    world
        .answer(REFUSED, Method::GET, TOKENS, namesake(), None)
        .await;
    world
        .answer(StatusCode::OK, Method::GET, TOKENS, local(), None)
        .await;
}

#[tokio::test]
async fn the_rule_ignores_the_case_of_ids() {
    let world = setup().await;
    let password = "carol-local-password";
    world
        .fixture
        .add_user("Carol", password, &["as-admin"])
        .await;
    for typed in ["carol", "CAROL"] {
        let who = Auth::Basic(typed, "carol-secret");
        world.answer(REFUSED, Method::GET, TOKENS, who, None).await;
    }
    let own = Auth::Basic("Carol", password);
    world
        .answer(StatusCode::OK, Method::GET, TOKENS, own, None)
        .await;
}

#[tokio::test]
async fn a_deleted_local_user_frees_the_id_for_the_directory() {
    let world = setup().await;
    let path = "/service/rest/v1/repositories";
    world
        .answer(REFUSED, Method::GET, path, namesake(), None)
        .await;
    let user = "/service/rest/v1/security/users/bob";
    world
        .answer(
            StatusCode::NO_CONTENT,
            Method::DELETE,
            user,
            Auth::Admin,
            None,
        )
        .await;
    world
        .answer(StatusCode::OK, Method::GET, path, namesake(), None)
        .await;
}

#[tokio::test]
async fn a_directory_user_without_a_local_namesake_still_reads_repositories() {
    let world = setup().await;
    let path = "/service/rest/v1/repositories";
    let reply = world
        .answer(StatusCode::OK, Method::GET, path, alice(), None)
        .await;
    assert!(reply.text().contains("crates"), "{}", reply.text());
}

#[tokio::test]
async fn directory_only_users_get_no_credentials_to_keep() {
    let world = setup().await;
    let denied = StatusCode::FORBIDDEN;
    let token = json!({"name": "dead", "scope": "WRITE", "expiresInDays": null});
    let reply = world
        .answer(denied, Method::POST, TOKENS, alice(), Some(token))
        .await;
    assert!(
        reply.text().contains("managed outside Astrum"),
        "{}",
        reply.text()
    );
    let key = json!({"password": ALICE_PASSWORD});
    world
        .answer(denied, Method::POST, NUGET_KEY, alice(), Some(key))
        .await;
    world
        .answer(denied, Method::GET, CARGO_ME, alice(), None)
        .await;
    world
        .answer(denied, Method::GET, GEMS_KEY, alice(), None)
        .await;
}
