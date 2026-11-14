mod accounts;
mod shop;
mod support;

use accounts::{USERS, audit, create_role, create_user, error_ids};
use axum::http::{Method, StatusCode};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use shop::{CUSTOMERS, customer, shop, usage_row};
use support::{Auth, Fixture};

#[derive(Clone, Copy, PartialEq)]
enum Right {
    Read,
    Create,
    Update,
    Delete,
}

const CALLERS: [(&str, Right); 4] = [
    ("reader", Right::Read),
    ("creator", Right::Create),
    ("updater", Right::Update),
    ("deleter", Right::Delete),
];

async fn people(fixture: &Fixture) {
    for (user, privileges) in [
        ("reader", vec!["as-customers-read"]),
        (
            "creator",
            vec!["as-customers-create", "as-repository-view-raw-files-a-read"],
        ),
        ("updater", vec!["as-customers-update"]),
        ("deleter", vec!["as-customers-delete"]),
    ] {
        let role = format!("{user}-role");
        create_role(fixture, &role, &privileges, &[]).await;
        create_user(fixture, user, "correct horse", &[role.as_str()]).await;
    }
    create_user(fixture, "nobody", "correct horse", &[]).await;
}

fn requests() -> Vec<(Method, String, Option<Value>, Right, StatusCode)> {
    let one = format!("{CUSTOMERS}/acme");
    let token = format!("{one}/tokens/00000000-0000-0000-0000-000000000000");
    let body = json!({"name": "Acme", "reference": "", "note": "", "disabled": false});
    vec![
        (
            Method::GET,
            CUSTOMERS.to_owned(),
            None,
            Right::Read,
            StatusCode::OK,
        ),
        (Method::GET, one.clone(), None, Right::Read, StatusCode::OK),
        (
            Method::GET,
            format!("{one}/tokens"),
            None,
            Right::Read,
            StatusCode::OK,
        ),
        (
            Method::GET,
            format!("{one}/usage"),
            None,
            Right::Read,
            StatusCode::OK,
        ),
        (
            Method::GET,
            format!("{CUSTOMERS}/usage.csv"),
            None,
            Right::Read,
            StatusCode::OK,
        ),
        (
            Method::POST,
            CUSTOMERS.to_owned(),
            Some(json!({"id": "other", "name": "Other"})),
            Right::Create,
            StatusCode::CREATED,
        ),
        (
            Method::POST,
            format!("{one}/tokens"),
            Some(shop::issue_body("admin", &["files-a"])),
            Right::Create,
            StatusCode::CREATED,
        ),
        (
            Method::PUT,
            one.clone(),
            Some(body),
            Right::Update,
            StatusCode::OK,
        ),
        (
            Method::DELETE,
            token,
            None,
            Right::Delete,
            StatusCode::NOT_FOUND,
        ),
        (
            Method::POST,
            format!("{one}/revoke"),
            Some(json!({})),
            Right::Delete,
            StatusCode::OK,
        ),
        (
            Method::DELETE,
            one,
            None,
            Right::Delete,
            StatusCode::NO_CONTENT,
        ),
    ]
}

#[tokio::test]
async fn every_route_demands_its_privilege() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    people(&fixture).await;
    for (method, path, body, right, success) in requests() {
        let anonymous = fixture
            .call(method.clone(), &path, Auth::None, body.clone())
            .await;
        assert_eq!(
            anonymous.status,
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
        let nobody = Auth::Basic("nobody", "correct horse");
        let refused = fixture
            .call(method.clone(), &path, nobody, body.clone())
            .await;
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{method} {path}");
        for (user, held) in CALLERS {
            let caller = Auth::Basic(user, "correct horse");
            let reply = fixture
                .call(method.clone(), &path, caller, body.clone())
                .await;
            let expected = match right == Right::Read || right == held {
                true => success,
                false => StatusCode::FORBIDDEN,
            };
            assert_eq!(reply.status, expected, "{user}: {method} {path}");
        }
    }
}

#[tokio::test]
async fn a_customer_is_created_changed_and_deleted() {
    let fixture = shop().await;
    fixture.add_user("olivia", "correct horse", &[]).await;
    let created = fixture
        .call(
            Method::POST,
            CUSTOMERS,
            Auth::Admin,
            Some(json!({"id": "acme", "name": " Acme Ltd ", "reference": "LIC-1", "note": "n"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["name"], "Acme Ltd");
    assert_eq!(created.json()["tokens"], 0);
    assert_eq!(created.json()["createdBy"], "admin");

    let duplicate = fixture
        .call(
            Method::POST,
            CUSTOMERS,
            Auth::Admin,
            Some(json!({"id": "acme", "name": "Again"})),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);

    for (id, name, field) in [
        ("olivia", "Olivia", "id"),
        ("admin", "Admin", "id"),
        ("Acme", "Upper", "id"),
        ("", "Empty", "id"),
        ("x", "", "name"),
    ] {
        let reply = fixture
            .call(
                Method::POST,
                CUSTOMERS,
                Auth::Admin,
                Some(json!({"id": id, "name": name})),
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{id}");
        assert_eq!(error_ids(&reply), [field], "{id}");
    }
    let long = fixture
        .call(
            Method::POST,
            CUSTOMERS,
            Auth::Admin,
            Some(json!({"id": "x".repeat(65), "name": "n".repeat(201), "note": "n".repeat(2001)})),
        )
        .await;
    assert_eq!(error_ids(&long), ["id", "name", "note"]);

    let path = format!("{CUSTOMERS}/acme");
    let missing = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "Acme", "reference": "", "note": ""})),
        )
        .await;
    assert_eq!(error_ids(&missing), ["disabled"]);
    let changed = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "Acme GmbH", "reference": "LIC-2", "note": "", "disabled": true})),
        )
        .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.text());
    assert_eq!(changed.json()["name"], "Acme GmbH");
    assert_eq!(changed.json()["disabled"], true);
    assert_eq!(changed.json()["id"], "acme");

    let removed = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    for method in [Method::GET, Method::DELETE] {
        let gone = fixture.call(method, &path, Auth::Admin, None).await;
        assert_eq!(gone.status, StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn a_user_cannot_take_the_id_of_a_customer() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let body = |user_id: &str| {
        json!({
            "userId": user_id,
            "firstName": "Test",
            "lastName": "User",
            "emailAddress": "test@example.org",
            "password": "correct horse",
            "status": "active",
            "roles": [],
        })
    };
    let refused = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(body("acme")))
        .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert_eq!(error_ids(&refused), ["userId"]);
    assert_eq!(
        refused.json()[0]["message"],
        "a customer with this id exists"
    );
    let other = fixture
        .call(Method::POST, USERS, Auth::Admin, Some(body("acme-two")))
        .await;
    assert_eq!(other.status, StatusCode::OK, "{}", other.text());
}

#[tokio::test]
async fn disabling_a_customer_refuses_its_token_and_enabling_restores_it() {
    let fixture = shop().await;
    let (_, token) = fixture.customer_token("acme", &["files-a"]).await;
    let read = || {
        fixture.call(
            Method::GET,
            "/repository/files-a/a.txt",
            Auth::Basic("acme", &token),
            None,
        )
    };
    assert_eq!(read().await.status, StatusCode::OK);
    let path = format!("{CUSTOMERS}/acme");
    let put = |disabled: bool| {
        fixture.call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "acme", "reference": "", "note": "", "disabled": disabled})),
        )
    };
    assert_eq!(put(true).await.status, StatusCode::OK);
    assert_eq!(read().await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(put(false).await.status, StatusCode::OK);
    assert_eq!(read().await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_list_carries_aggregates_for_every_customer() {
    let fixture = shop().await;
    let (first, _) = fixture.customer_token("acme", &["files-a"]).await;
    let (second, _) = fixture.customer_token("acme", &["files-b"]).await;
    let (third, _) = fixture.customer_token("globex", &["files-a"]).await;
    customer(&fixture, "empty").await;
    usage_row(&fixture, first, "acme", "files-a", 0, 3, 300).await;
    usage_row(&fixture, second, "acme", "files-b", 29, 2, 20).await;
    usage_row(&fixture, second, "acme", "files-b", 30, 100, 1000).await;
    usage_row(&fixture, third, "globex", "files-a", 5, 7, 70).await;
    let listed = fixture
        .call(Method::GET, CUSTOMERS, Auth::Admin, None)
        .await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let items = listed.json();
    let ids: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["acme", "empty", "globex"]);
    let acme = &items[0];
    assert_eq!(acme["tokens"], 2);
    assert_eq!(acme["downloads30d"], 5);
    assert_eq!(acme["bytes30d"], 320);
    assert!(acme["lastAccessAt"].is_null());
    let expiry = chrono::DateTime::parse_from_rfc3339(acme["nearestExpiry"].as_str().unwrap())
        .unwrap()
        .with_timezone(&Utc);
    assert!(
        (expiry - (Utc::now() + Duration::days(30)))
            .num_seconds()
            .abs()
            < 120
    );
    assert_eq!(items[1]["tokens"], 0);
    assert_eq!(items[1]["downloads30d"], 0);
    assert!(items[1]["nearestExpiry"].is_null());
    assert_eq!(items[2]["downloads30d"], 7);
}

#[tokio::test]
async fn the_last_access_of_a_customer_is_the_last_use_of_its_tokens() {
    let fixture = shop().await;
    let (_, token) = fixture.customer_token("acme", &["files-a"]).await;
    let used = fixture
        .call(
            Method::GET,
            "/repository/files-a/a.txt",
            Auth::Basic("acme", &token),
            None,
        )
        .await;
    assert_eq!(used.status, StatusCode::OK);
    let one = fixture
        .call(Method::GET, &format!("{CUSTOMERS}/acme"), Auth::Admin, None)
        .await;
    assert!(one.json()["lastAccessAt"].is_string(), "{}", one.text());
}

#[tokio::test]
async fn changes_are_audited_without_secrets() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let issued = shop::issue(
        &fixture,
        Auth::Admin,
        "acme",
        shop::issue_body("admin", &["files-a"]),
    )
    .await;
    assert_eq!(issued.status, StatusCode::CREATED, "{}", issued.text());
    let path = format!("{CUSTOMERS}/acme");
    fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"name": "Acme 2", "reference": "", "note": "", "disabled": false})),
        )
        .await;
    let revoked = fixture
        .call(
            Method::POST,
            &format!("{path}/revoke"),
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(revoked.json(), json!({"revoked": 1}));
    fixture.call(Method::DELETE, &path, Auth::Admin, None).await;

    let events = audit(&fixture, "security.customer").await;
    let kinds: Vec<&str> = events.iter().map(|event| event.0.as_str()).collect();
    assert_eq!(kinds, ["created", "updated", "revoked", "deleted"]);
    assert!(
        events
            .iter()
            .all(|event| event.1 == "acme" && event.2 == "admin")
    );
    assert_eq!(events[1].3["before"]["name"], "acme Ltd");
    assert_eq!(events[1].3["after"]["name"], "Acme 2");
    assert_eq!(events[2].3["count"], 1);
    let tokens = audit(&fixture, "security.accesstoken").await;
    assert_eq!(tokens[0].0, "created");
    assert_eq!(tokens[0].3["customer"], "acme");
    assert_eq!(tokens[0].3["repositories"], json!(["files-a"]));
    let secret = issued.json()["token"].as_str().unwrap().to_owned();
    for event in events.iter().chain(&tokens) {
        assert!(!event.3.to_string().contains(&secret));
    }
}
