mod support;

use std::sync::Arc;

use astrum_db::entity::access_token;
use astrum_security::{NewToken, Scope};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::EntityTrait;
use serde_json::{Value, json};
use support::{Auth, Fixture};

struct Shop {
    fixture: Fixture,
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    })
}

impl Shop {
    async fn new(peer: Option<&str>) -> Self {
        let mut fixture = Fixture::new().await;
        fixture.peer = peer.map(|peer| peer.parse().unwrap());
        astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
            .await
            .unwrap();
        fixture
            .state
            .content()
            .register(Arc::new(astrum_format_raw::Raw));
        for name in ["onprem-files", "private-files"] {
            let reply = fixture
                .call(
                    Method::POST,
                    "/service/rest/v1/repositories/raw/hosted",
                    Auth::Admin,
                    Some(hosted(name)),
                )
                .await;
            assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
            let stored = fixture
                .send_body(
                    Method::PUT,
                    &format!("/repository/{name}/a.txt"),
                    Auth::Admin,
                    Body::from(format!("{name} content")),
                    &[],
                )
                .await;
            assert_eq!(stored.status, StatusCode::CREATED);
        }
        Shop { fixture }
    }

    async fn customer_token(&self, repositories: &[&str]) -> (uuid::Uuid, String) {
        self.fixture.customer_token("acme", repositories).await
    }

    async fn get(&self, path: &str, user: &str, token: &str) -> (StatusCode, String) {
        let reply = self
            .fixture
            .call(Method::GET, path, Auth::Basic(user, token), None)
            .await;
        (reply.status, reply.text())
    }
}

#[tokio::test]
async fn a_customer_token_reads_the_listed_repositories_only() {
    let shop = Shop::new(None).await;
    let (_, token) = shop.customer_token(&["onprem-files"]).await;

    let listed = shop
        .get("/repository/onprem-files/a.txt", "acme", &token)
        .await;
    assert_eq!(listed, (StatusCode::OK, "onprem-files content".to_owned()));
    let by_owner = shop
        .get("/repository/onprem-files/a.txt", "admin", &token)
        .await;
    assert_eq!(by_owner.0, StatusCode::OK);
    let as_bearer = shop
        .fixture
        .send(
            Method::GET,
            "/repository/onprem-files/a.txt",
            Auth::None,
            None,
            &[("authorization", &format!("Bearer {token}"))],
        )
        .await;
    assert_eq!(as_bearer.status, StatusCode::OK);

    let other = shop
        .get("/repository/private-files/a.txt", "acme", &token)
        .await;
    assert_eq!(other.0, StatusCode::FORBIDDEN, "{}", other.1);

    let repositories = shop
        .fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::Basic("acme", &token),
            None,
        )
        .await;
    assert_eq!(repositories.status, StatusCode::OK);
    let names: Vec<String> = repositories
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|repository| repository["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, vec!["onprem-files"]);

    let upload = shop
        .fixture
        .send_body(
            Method::PUT,
            "/repository/onprem-files/b.txt",
            Auth::Basic("acme", &token),
            Body::from("x"),
            &[],
        )
        .await;
    assert_eq!(upload.status, StatusCode::FORBIDDEN);

    let personal = shop
        .fixture
        .state
        .security()
        .create_token("admin", NewToken::personal("ci", Scope::Read, Some(5)))
        .await
        .unwrap()
        .1;
    let ordinary = shop
        .get("/repository/private-files/a.txt", "admin", &personal)
        .await;
    assert_eq!(
        ordinary.0,
        StatusCode::OK,
        "tokens without a list behave as before"
    );
}

#[tokio::test]
async fn a_customer_token_reaches_no_management_surface() {
    let shop = Shop::new(None).await;
    let (_, token) = shop.customer_token(&["onprem-files"]).await;
    let auth = || Auth::Basic("acme", &token);
    for path in [
        "/service/rest/v1/security/users",
        "/service/rest/v1/security/roles",
        "/service/rest/v1/email",
        "/service/rest/v1/tasks",
        "/service/rest/v1/formats/upload-specs",
        "/service/rest/internal/access-tokens",
        "/service/rest/internal/ui/user/nuget-api-key",
    ] {
        let reply = shop.fixture.call(Method::GET, path, auth(), None).await;
        assert_eq!(
            reply.status,
            StatusCode::FORBIDDEN,
            "{path}: {}",
            reply.text()
        );
    }
    for (path, body) in [
        (
            "/service/rest/v1/security/users",
            json!({"userId": "mallory", "firstName": "M", "lastName": "M", "emailAddress": "m@example.org", "password": "correct horse battery", "status": "active", "roles": ["nx-admin"]}),
        ),
        (
            "/service/rest/internal/access-tokens",
            json!({"name": "mine", "scope": "WRITE", "expiresInDays": 30}),
        ),
        ("/mcp", json!({"jsonrpc": "2.0", "id": 1, "method": "ping"})),
    ] {
        let reply = shop
            .fixture
            .call(Method::POST, path, auth(), Some(body))
            .await;
        assert_eq!(
            reply.status,
            StatusCode::FORBIDDEN,
            "{path}: {}",
            reply.text()
        );
    }
    let users = shop
        .fixture
        .call(
            Method::GET,
            "/service/rest/v1/security/users",
            Auth::Admin,
            None,
        )
        .await;
    assert!(!users.text().contains("mallory"));
}

#[tokio::test]
async fn revoking_a_customer_token_takes_effect_on_the_next_request() {
    let shop = Shop::new(None).await;
    let (id, token) = shop.customer_token(&["onprem-files"]).await;
    let path = "/repository/onprem-files/a.txt";
    assert_eq!(shop.get(path, "acme", &token).await.0, StatusCode::OK);
    assert!(
        shop.fixture
            .state
            .security()
            .revoke_customer_token("acme", id)
            .await
            .unwrap()
    );
    assert_eq!(
        shop.get(path, "acme", &token).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn the_client_address_of_a_customer_token_is_stored() {
    let shop = Shop::new(Some("203.0.113.9")).await;
    let (_, token) = shop.customer_token(&["onprem-files"]).await;
    shop.get("/repository/onprem-files/a.txt", "acme", &token)
        .await;
    let row = access_token::Entity::find()
        .one(shop.fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.last_used_ip.as_deref(), Some("203.0.113.9"));
    assert!(row.last_used_at.is_some());
}

#[tokio::test]
async fn the_personal_page_neither_shows_nor_deletes_customer_tokens() {
    let shop = Shop::new(None).await;
    let (id, token) = shop.customer_token(&["onprem-files"]).await;
    let tokens = "/service/rest/internal/access-tokens";
    let listed = shop
        .fixture
        .call(
            Method::GET,
            &format!("{tokens}?userId=*"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    assert!(listed.json().as_array().unwrap().is_empty());
    let removed = shop
        .fixture
        .call(Method::DELETE, &format!("{tokens}/{id}"), Auth::Admin, None)
        .await;
    assert_eq!(removed.status, StatusCode::NOT_FOUND, "{}", removed.text());
    assert_eq!(
        shop.get("/repository/onprem-files/a.txt", "acme", &token)
            .await
            .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_customer_token_sees_its_customer_and_not_its_owner() {
    let shop = Shop::new(None).await;
    shop.fixture
        .add_user("olivia", "correct horse battery", &["nx-admin"])
        .await;
    let (_, token) = shop
        .fixture
        .customer_token_of("olivia", "acme", &["onprem-files"])
        .await;
    for path in [
        "/service/rest/internal/session",
        "/service/rest/internal/ui/state",
    ] {
        let reply = shop
            .fixture
            .call(Method::GET, path, Auth::Basic("acme", &token), None)
            .await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        let text = reply.text();
        for private in ["olivia", "example.org", "Tester"] {
            assert!(!text.contains(private), "{path}: {text}");
        }
        assert_eq!(reply.json()["user"]["userId"], "acme", "{path}");
    }
    let owner = shop
        .fixture
        .call(
            Method::GET,
            "/service/rest/internal/session",
            Auth::Basic("olivia", "correct horse battery"),
            None,
        )
        .await;
    assert_eq!(owner.json()["user"]["emailAddress"], "olivia@example.org");
}
