#![allow(dead_code)]

use std::sync::Arc;

use astrum_db::entity::{oauth_client, oauth_grant};
use astrum_security::ldap::TlsClient;
use astrum_testkit::ldap::{BASE, LdapTestServer, SYSTEM_DN, SYSTEM_PASSWORD};
use axum::http::{Method, StatusCode};
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};

use crate::support::{Auth, Fixture, Reply};

pub const TOKENS: &str = "/service/rest/internal/access-tokens";
pub const NUGET_KEY: &str = "/service/rest/internal/ui/user/nuget-api-key";
pub const GRANTS: &str = "/service/rest/internal/ui/oauth/grants";
pub const CARGO_ME: &str = "/repository/crates/me";
pub const GEMS_KEY: &str = "/repository/gems/api/v1/api_key";
const REALMS: &str = "/service/rest/v1/security/realms/active";
pub const LOCAL_PASSWORD: &str = "local-bob-password";
pub const DIRECTORY_PASSWORD: &str = "bob-secret";
pub const ALICE_PASSWORD: &str = "alice-secret";

pub struct Session {
    cookies: String,
    csrf: String,
}

pub struct World {
    pub fixture: Fixture,
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

pub async fn setup_bare(realms: Value) -> World {
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
    let admins = json!({"id": "admins", "name": "Admins", "source": "LDAP",
        "privileges": ["as-accesstokens-all"]});
    expect(&fixture, Method::POST, roles, admins, StatusCode::OK).await;
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
    World {
        fixture,
        _directory: directory,
    }
}

pub async fn setup_with(realms: Value) -> World {
    let world = setup_bare(realms).await;
    world
        .fixture
        .add_user("bob", LOCAL_PASSWORD, &["as-admin"])
        .await;
    world
}

pub fn local_first() -> Value {
    json!([
        "AstrumAuthenticatingRealm",
        "AccessTokenRealm",
        "LdapRealm",
        "CargoToken",
        "NuGetApiKey",
        "RubyGemsApiKey"
    ])
}

pub async fn setup() -> World {
    setup_with(local_first()).await
}

impl World {
    pub async fn answer(
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

    pub async fn local_token(&self) -> Value {
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

    pub async fn local_nuget_key(&self) -> String {
        let body = json!({"password": LOCAL_PASSWORD});
        let reply = self
            .answer(StatusCode::OK, Method::POST, NUGET_KEY, local(), Some(body))
            .await;
        reply.json()["apiKey"].as_str().unwrap().to_owned()
    }

    pub async fn grant(&self, user_id: &str, realm: &str) -> uuid::Uuid {
        let db = self.fixture.state.db().conn();
        let now = chrono::Utc::now().fixed_offset();
        let id = uuid::Uuid::new_v4();
        let client = format!("agent-{id}");
        oauth_client::Entity::insert(oauth_client::ActiveModel {
            id: ActiveValue::Set(client.clone()),
            name: ActiveValue::Set(format!("Agent of {realm}")),
            redirect_uris: ActiveValue::Set(json!(["http://127.0.0.1/cb"])),
            created_at: ActiveValue::Set(now),
        })
        .exec(db)
        .await
        .unwrap();
        oauth_grant::Entity::insert(oauth_grant::ActiveModel {
            id: ActiveValue::Set(id),
            client_id: ActiveValue::Set(client),
            user_id: ActiveValue::Set(user_id.to_owned()),
            realm: ActiveValue::Set(realm.to_owned()),
            scope: ActiveValue::Set("read".to_owned()),
            resource: ActiveValue::Set(None),
            access_hash: ActiveValue::Set(format!("a-{id}")),
            access_expires_at: ActiveValue::Set(now),
            refresh_hash: ActiveValue::Set(format!("r-{id}")),
            refresh_expires_at: ActiveValue::Set(now + chrono::Duration::days(1)),
            created_at: ActiveValue::Set(now),
            last_used_at: ActiveValue::Set(None),
        })
        .exec(db)
        .await
        .unwrap();
        id
    }

    pub async fn local_grant(&self) -> uuid::Uuid {
        self.grant("bob", "AstrumAuthenticatingRealm").await
    }

    pub async fn session(&self, user: &str, password: &str) -> Session {
        let body = json!({"username": user, "password": password});
        let path = "/service/rest/internal/session";
        let reply = self
            .answer(StatusCode::OK, Method::POST, path, Auth::None, Some(body))
            .await;
        let cookie = |wanted: &str| {
            reply
                .headers
                .get_all(axum::http::header::SET_COOKIE)
                .iter()
                .filter_map(|value| value.to_str().unwrap().split(';').next())
                .filter_map(|pair| pair.split_once('='))
                .find(|(name, _)| *name == wanted)
                .map(|(_, value)| value.to_owned())
                .unwrap()
        };
        let csrf = cookie("astrum_csrf");
        Session {
            cookies: format!(
                "astrum_session={}; astrum_csrf={csrf}",
                cookie("astrum_session")
            ),
            csrf,
        }
    }

    pub async fn within(
        &self,
        wanted: StatusCode,
        method: Method,
        path: &str,
        session: &Session,
    ) -> Reply {
        let headers = [
            ("cookie", session.cookies.as_str()),
            ("x-astrum-csrf", session.csrf.as_str()),
        ];
        let reply = self
            .fixture
            .send(method.clone(), path, Auth::None, None, &headers)
            .await;
        assert_eq!(reply.status, wanted, "{method} {path}: {}", reply.text());
        reply
    }
}

pub fn namesake() -> Auth<'static> {
    Auth::Basic("bob", DIRECTORY_PASSWORD)
}

pub fn local() -> Auth<'static> {
    Auth::Basic("bob", LOCAL_PASSWORD)
}

pub fn alice() -> Auth<'static> {
    Auth::Basic("alice", ALICE_PASSWORD)
}

pub const REFUSED: StatusCode = StatusCode::UNAUTHORIZED;
