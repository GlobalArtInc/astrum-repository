#![allow(dead_code)]

use astrum_api::{AppState, router};
use astrum_security::SecretBox;
use astrum_security::catalogue::ADMIN_USER;
use astrum_testkit::TestDatabase;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::TempDir;
use tower::ServiceExt;

pub struct Fixture {
    pub test: TestDatabase,
    pub data: TempDir,
    pub state: AppState,
    pub admin_password: String,
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body)
            .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&self.body)))
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

pub enum Auth<'a> {
    None,
    Admin,
    Basic(&'a str, &'a str),
}

impl Fixture {
    pub async fn add_user(&self, user_id: &str, password: &str, roles: &[&str]) {
        use astrum_db::entity::{security_user, security_user_role};
        use sea_orm::{ActiveValue, EntityTrait};
        let now = chrono::Utc::now().fixed_offset();
        security_user::Entity::insert(security_user::ActiveModel {
            id: ActiveValue::Set(user_id.to_owned()),
            first_name: ActiveValue::Set(user_id.to_owned()),
            last_name: ActiveValue::Set("Tester".to_owned()),
            email: ActiveValue::Set(format!("{user_id}@example.org")),
            password_hash: ActiveValue::Set(Some(astrum_security::password::hash(password))),
            status: ActiveValue::Set("active".to_owned()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.state.db().conn())
        .await
        .unwrap();
        security_user_role::Entity::insert(security_user_role::ActiveModel {
            user_id: ActiveValue::Set(user_id.to_owned()),
            source: ActiveValue::Set("default".to_owned()),
            roles: ActiveValue::Set(serde_json::json!(roles)),
            updated_at: ActiveValue::Set(now),
        })
        .exec(self.state.db().conn())
        .await
        .unwrap();
        self.state.security().reload().await.unwrap();
    }

    pub async fn new() -> Self {
        let test = TestDatabase::create().await;
        let data = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(data.path().join("tmp")).unwrap();
        let db = test.migrated().await;
        let created = astrum_security::bootstrap(db.conn(), data.path())
            .await
            .unwrap();
        let admin_password = std::fs::read_to_string(created.admin_password_file.unwrap()).unwrap();
        let state = AppState::load(db, data.path().to_owned(), SecretBox::from_key([7; 32]))
            .await
            .unwrap();
        Fixture {
            test,
            data,
            state,
            admin_password,
        }
    }

    pub async fn call(
        &self,
        method: Method,
        path: &str,
        auth: Auth<'_>,
        body: Option<Value>,
    ) -> Reply {
        self.send(method, path, auth, body, &[]).await
    }

    pub async fn send(
        &self,
        method: Method,
        path: &str,
        auth: Auth<'_>,
        body: Option<Value>,
        headers: &[(&str, &str)],
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        let credentials = match auth {
            Auth::None => None,
            Auth::Admin => Some((ADMIN_USER, self.admin_password.as_str())),
            Auth::Basic(user, password) => Some((user, password)),
        };
        if let Some((user, password)) = credentials {
            let encoded = STANDARD.encode(format!("{user}:{password}"));
            request = request.header(header::AUTHORIZATION, format!("Basic {encoded}"));
        }
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let body = match body {
            Some(value) => {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(&value).unwrap())
            }
            None => Body::empty(),
        };
        let response = router(self.state.clone())
            .oneshot(request.body(body).unwrap())
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
}
