mod support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const CAPABILITIES: &str = "/service/rest/v1/capabilities";

async fn post(fixture: &Fixture, body: Value) -> support::Reply {
    fixture
        .call(Method::POST, CAPABILITIES, Auth::Admin, Some(body))
        .await
}

#[tokio::test]
async fn settings_appear_as_capabilities() {
    let fixture = Fixture::new().await;
    assert_eq!(
        fixture
            .call(Method::GET, CAPABILITIES, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let types = fixture
        .call(
            Method::GET,
            &format!("{CAPABILITIES}/types"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    let ids: Vec<_> = types
        .as_array()
        .unwrap()
        .iter()
        .map(|kind| kind["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "baseurl",
            "rapture.settings",
            "rapture.branding",
            "metrics.prometheus",
            "blobstores.space"
        ]
    );
    assert_eq!(types[0]["formFields"][0]["id"], "url");

    let listed = fixture
        .call(Method::GET, CAPABILITIES, Auth::Admin, None)
        .await
        .json();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["type"], "rapture.settings");
    assert_eq!(listed[0]["typeName"], "UI: Settings");
    assert_eq!(listed[0]["properties"]["title"], "Astrum Repository");
    assert_eq!(listed[0]["tags"]["Category"], "UI");

    let bad = post(
        &fixture,
        json!({"type": "baseurl", "enabled": true, "properties": {"url": "notaurl"}}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.json()[0]["id"], "properties.url");
    let created = post(
        &fixture,
        json!({"type": "baseurl", "enabled": true, "notes": "proxy", "properties": {"url": "https://repo.example.com/"}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let created = created.json();
    assert_eq!(created["id"], "baseurl");
    assert_eq!(created["state"], "active");
    assert_eq!(created["properties"]["url"], "https://repo.example.com");
    let again = post(
        &fixture,
        json!({"type": "baseurl", "properties": {"url": "https://other.example.com"}}),
    )
    .await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        again.json()[0]["message"],
        "Only one capability of type 'Base URL' can be created"
    );

    let state = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/state",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(state["baseUrl"], "https://repo.example.com");

    let disabled = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/baseurl"),
            Auth::Admin,
            Some(json!({"type": "baseurl", "enabled": false, "notes": "", "properties": {"url": "https://repo.example.com"}})),
        )
        .await;
    assert_eq!(disabled.status, StatusCode::OK);
    assert_eq!(disabled.json()["state"], "disabled");
    let state = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/state",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(state["baseUrl"], "http://localhost");

    let removed = fixture
        .call(
            Method::DELETE,
            &format!("{CAPABILITIES}/baseurl"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        fixture
            .call(
                Method::GET,
                &format!("{CAPABILITIES}/baseurl"),
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=capability",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(audit.as_array().unwrap().len(), 3, "{audit}");
}

#[tokio::test]
async fn ui_settings_and_branding_reach_the_user_interface() {
    let fixture = Fixture::new().await;
    let short = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/rapture.settings"),
            Auth::Admin,
            Some(json!({"type": "rapture.settings", "enabled": true, "properties": {"title": "Repo", "requestTimeout": "10"}})),
        )
        .await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST);
    assert_eq!(short.json()[0]["id"], "properties.requestTimeout");
    let saved = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/rapture.settings"),
            Auth::Admin,
            Some(
                json!({"type": "rapture.settings", "enabled": true, "properties": {
                    "title": "Team Repository", "sessionTimeout": "45", "debugAllowed": "false",
                    "statusIntervalAuthenticated": "5", "statusIntervalAnonymous": "60",
                    "requestTimeout": "60", "longRequestTimeout": "180"
                }}),
            ),
        )
        .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.text());
    assert_eq!(saved.json()["properties"]["sessionTimeout"], "45");

    let created = fixture
        .call(
            Method::POST,
            CAPABILITIES,
            Auth::Admin,
            Some(
                json!({"type": "rapture.branding", "enabled": true, "properties": {
                    "headerEnabled": "true", "headerHtml": "<img src=\"$baseUrl/logo.png\"> Team",
                    "footerEnabled": "false", "footerHtml": "<b>hidden</b>"
                }}),
            ),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let state = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/state",
            Auth::None,
            None,
        )
        .await
        .json();
    assert_eq!(state["uiSettings"]["title"], "Team Repository");
    assert_eq!(state["sessionTimeoutMinutes"], 45);
    assert_eq!(
        state["branding"]["headerHtml"],
        "<img src=\"http://localhost/logo.png\"> Team"
    );
    assert_eq!(state["branding"]["footerHtml"], Value::Null);
}

#[tokio::test]
async fn the_nodes_page_lists_this_replica() {
    let fixture = Fixture::new().await;
    let reply = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/nodes",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let nodes = reply.json();
    assert_eq!(nodes["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(nodes["nodes"][0]["current"], true);
    assert_eq!(nodes["nodes"][0]["id"], fixture.state.node_id().to_string());
    sea_orm::sqlx::query(
        "INSERT INTO node (id, name, version, started_at, last_seen) VALUES \
         (gen_random_uuid(), 'other', '3.0.0', now(), now()), \
         (gen_random_uuid(), 'gone', '3.0.0', now(), now() - interval '10 minutes')",
    )
    .execute(fixture.state.db().conn().get_postgres_connection_pool())
    .await
    .unwrap();
    let nodes = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/nodes",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let names: Vec<_> = nodes["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["name"].as_str().unwrap().to_owned(),
                node["current"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&("other".to_owned(), false)));
}
