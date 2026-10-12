mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::Auth;
use ui_support::*;

const TYPES: &str = "/service/rest/internal/ui/privileges/types";

fn field<'a>(types: &'a Value, type_id: &str, field_id: &str) -> &'a Value {
    types
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == type_id)
        .unwrap_or_else(|| panic!("no type {type_id}"))["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == field_id)
        .unwrap_or_else(|| panic!("no field {field_id} in {type_id}"))
}

#[tokio::test]
async fn privilege_types_describe_their_forms() {
    let fixture = ready().await;
    let reply = get(&fixture, TYPES, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let types = reply.json();
    assert_eq!(
        names(&types, "id"),
        [
            "application",
            "wildcard",
            "repository-view",
            "repository-admin",
            "repository-content-selector"
        ]
    );
    let application = &types[0]["fields"];
    assert_eq!(
        names(application, "id"),
        ["name", "description", "domain", "actions"]
    );
    assert_eq!(
        field(&types, "application", "description")["required"],
        false
    );
    let actions = field(&types, "application", "actions");
    assert_eq!(actions["kind"], "actions");
    assert_eq!(actions["required"], true);
    let values = actions["values"].as_array().unwrap();
    assert!(values.contains(&json!("READ")));
    assert!(values.contains(&json!("START")));
    assert!(!values.contains(&json!("BROWSE")));
    assert_eq!(field(&types, "wildcard", "pattern")["kind"], "permission");
    let format = field(&types, "repository-view", "format");
    assert_eq!(format["kind"], "format");
    assert_eq!(format["any"], true);
    assert_eq!(format["values"][0], "*");
    assert!(
        format["values"]
            .as_array()
            .unwrap()
            .contains(&json!("maven2"))
    );
    assert_eq!(
        field(&types, "repository-admin", "repository")["kind"],
        "repository"
    );
    assert_eq!(
        field(&types, "repository-content-selector", "contentSelector")["kind"],
        "contentSelector"
    );
    let view_actions = field(&types, "repository-view", "actions")["values"].clone();
    assert_eq!(
        view_actions,
        json!(["BROWSE", "READ", "EDIT", "ADD", "DELETE", "ALL"])
    );
}

#[tokio::test]
async fn the_privileges_api_requires_what_the_types_describe() {
    let fixture = ready().await;
    let types = get(&fixture, TYPES, Auth::Admin).await.json();
    for item in types.as_array().unwrap() {
        let type_id = item["id"].as_str().unwrap();
        let reply = fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/security/privileges/{type_id}"),
                Auth::Admin,
                Some(json!({})),
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{type_id}");
        let refused = names(&reply.json(), "id");
        let required: Vec<String> = item["fields"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|field| field["required"] == true)
            .map(|field| field["id"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(refused, required, "{type_id}");
    }
}

#[tokio::test]
async fn privilege_types_need_privilege_read() {
    let fixture = ready().await;
    user_with(&fixture, "reader", &["as-privileges-read"]).await;
    assert_eq!(
        get(&fixture, TYPES, user("reader")).await.status,
        StatusCode::OK
    );
    user_with(&fixture, "other", &["as-users-read"]).await;
    assert_eq!(
        get(&fixture, TYPES, user("other")).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&fixture, TYPES, Auth::None).await.status,
        StatusCode::UNAUTHORIZED
    );
}
