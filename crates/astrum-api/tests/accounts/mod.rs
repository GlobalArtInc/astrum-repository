#![allow(dead_code)]

use astrum_api::router;
use astrum_db::entity::{audit_event, repository};
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::support::{Auth, Fixture, Reply};

pub const USERS: &str = "/service/rest/v1/security/users";
pub const ROLES: &str = "/service/rest/v1/security/roles";
pub const PRIVILEGES: &str = "/service/rest/v1/security/privileges";
pub const SELECTORS: &str = "/service/rest/v1/security/content-selectors";
pub const SECURITY: &str = "/service/rest/v1/security";

pub async fn create_user(fixture: &Fixture, user_id: &str, password: &str, roles: &[&str]) {
    let reply = fixture
        .call(
            Method::POST,
            USERS,
            Auth::Admin,
            Some(json!({
                "userId": user_id,
                "firstName": "Test",
                "lastName": "User",
                "emailAddress": format!("{user_id}@example.org"),
                "password": password,
                "status": "active",
                "roles": roles,
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

pub async fn create_role(fixture: &Fixture, id: &str, privileges: &[&str], roles: &[&str]) {
    let reply = fixture
        .call(
            Method::POST,
            ROLES,
            Auth::Admin,
            Some(json!({"id": id, "name": id, "privileges": privileges, "roles": roles})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

pub async fn create_repository(fixture: &Fixture, name: &str, format: &str) {
    let now = chrono::Utc::now().fixed_offset();
    repository::Entity::insert(repository::ActiveModel {
        name: ActiveValue::Set(name.to_owned()),
        format: ActiveValue::Set(format.to_owned()),
        kind: ActiveValue::Set("hosted".to_owned()),
        online: ActiveValue::Set(true),
        blob_store: ActiveValue::Set(None),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().reload().await.unwrap();
}

pub async fn send_raw(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    content_type: Option<&str>,
    body: &str,
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    let credentials = match auth {
        Auth::None => None,
        Auth::Admin => Some(("admin", fixture.admin_password.as_str())),
        Auth::Basic(user, password) => Some((user, password)),
    };
    if let Some((user, password)) = credentials {
        let encoded = STANDARD.encode(format!("{user}:{password}"));
        request = request.header(header::AUTHORIZATION, format!("Basic {encoded}"));
    }
    if let Some(content_type) = content_type {
        request = request.header(header::CONTENT_TYPE, content_type);
    }
    let response = router(fixture.state.clone())
        .oneshot(request.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Reply {
        status,
        headers,
        body,
    }
}

pub async fn audit(fixture: &Fixture, domain: &str) -> Vec<(String, String, String, Value)> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq(domain))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
        .into_iter()
        .map(|event| (event.kind, event.context, event.initiator, event.attributes))
        .collect()
}

pub fn ids(reply: &Reply, field: &str) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item[field].as_str().unwrap().to_owned())
        .collect()
}

pub fn error_ids(reply: &Reply) -> Vec<String> {
    ids(reply, "id")
}
