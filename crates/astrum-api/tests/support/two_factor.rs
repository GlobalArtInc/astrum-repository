use astrum_api::{AppState, router};
use astrum_security::totp::{self, TotpSecret};
use astrum_security::two_factor::store::Confirmation;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use chrono::Utc;
use http_body_util::BodyExt;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{Auth, Fixture, Reply};

pub const SESSION: &str = "/service/rest/internal/session";
pub const SECOND: &str = "/service/rest/internal/session/second-factor";
pub const ACCOUNT: &str = "/service/rest/internal/ui/user";
pub const FACTORS: &str = "/service/rest/internal/ui/user/two-factor";
pub const TOKENS: &str = "/service/rest/internal/access-tokens";
pub const AUDIT: &str = "/service/rest/internal/audit";
pub const REALM: &str = "AstrumAuthenticatingRealm";
pub const JANE: &str = "jane-password-1";
pub const PAT: &str = "pat-password-1";

pub struct Factor {
    pub secret: TotpSecret,
    pub codes: Vec<String>,
}

impl Factor {
    pub fn code(&self) -> String {
        let step = totp::step_at(u64::try_from(Utc::now().timestamp()).unwrap());
        totp::code_at(&self.secret, step)
    }

    pub fn wrong(&self) -> String {
        let step = totp::step_at(u64::try_from(Utc::now().timestamp()).unwrap());
        let near: Vec<String> = (step - 2..=step + 2)
            .map(|step| totp::code_at(&self.secret, step))
            .collect();
        (0..1_000_000)
            .map(|n| format!("{n:06}"))
            .find(|candidate| !near.contains(candidate))
            .unwrap()
    }
}

pub async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(sql)
        .await
        .unwrap();
}

pub async fn number(fixture: &Fixture, query: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT ({query})::bigint"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get_by_index::<i64>(0)
        .unwrap()
}

pub async fn world() -> Fixture {
    let fixture = Fixture::new().await;
    fixture.add_user("jane", JANE, &[]).await;
    fixture.add_user("pat", PAT, &[]).await;
    fixture
}

pub async fn enroll(fixture: &Fixture, user: &str) -> Factor {
    let security = fixture.state.security();
    let pending = security
        .begin_totp(user, REALM, "Astrum (test)", user, "127.0.0.1")
        .await
        .unwrap();
    let secret = TotpSecret::from_base32(&pending.secret).unwrap();
    let now = Utc::now();
    let code = totp::code_at(
        &secret,
        totp::step_at(u64::try_from(now.timestamp()).unwrap()),
    );
    let outcome = security
        .confirm_totp(user, &pending.ceremony, &code, now)
        .await
        .unwrap();
    let Confirmation::Enrolled { recovery_codes, .. } = outcome else {
        panic!("{outcome:?}");
    };
    execute(
        fixture,
        &format!("UPDATE user_second_factor SET totp_last_step = 0 WHERE user_id = '{user}'"),
    )
    .await;
    Factor {
        secret,
        codes: recovery_codes.unwrap().as_slice().to_vec(),
    }
}

pub async fn password(
    fixture: &Fixture,
    user: &str,
    secret: &str,
    headers: &[(&str, &str)],
) -> Reply {
    fixture
        .send(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": user, "password": secret})),
            headers,
        )
        .await
}

pub fn set_cookies(reply: &Reply) -> Vec<String> {
    reply
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_owned())
        .collect()
}

pub fn cookie_value(reply: &Reply, name: &str) -> Option<String> {
    set_cookies(reply).iter().find_map(|cookie| {
        let (key, rest) = cookie.split_once('=')?;
        (key == name).then(|| rest.split(';').next().unwrap().to_owned())
    })
}

pub async fn start(fixture: &Fixture, user: &str, secret: &str) -> String {
    let reply = password(fixture, user, secret, &[]).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    cookie_value(&reply, "astrum_signin").expect("the ceremony cookie")
}

pub async fn step_on(state: &AppState, handle: Option<&str>, method: &str, code: &str) -> Reply {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(SECOND)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(handle) = handle {
        request = request.header(header::COOKIE, format!("astrum_signin={handle}"));
    }
    let body = json!({"method": method, "code": code});
    let response = router(state.clone())
        .oneshot(
            request
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
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

pub async fn step(fixture: &Fixture, handle: &str, method: &str, code: &str) -> Reply {
    step_on(&fixture.state, Some(handle), method, code).await
}

pub fn session_headers(reply: &Reply) -> (String, String) {
    let session = cookie_value(reply, "astrum_session").expect("a session cookie");
    let csrf = cookie_value(reply, "astrum_csrf").expect("a csrf cookie");
    (
        format!("astrum_session={session}; astrum_csrf={csrf}"),
        csrf,
    )
}

pub async fn with_session(
    fixture: &Fixture,
    method: Method,
    path: &str,
    login: &Reply,
    body: Option<Value>,
) -> Reply {
    let (cookie, csrf) = session_headers(login);
    fixture
        .send(
            method,
            path,
            Auth::None,
            body,
            &[
                ("cookie", &cookie),
                ("x-astrum-csrf", &csrf),
                ("sec-fetch-site", "same-origin"),
            ],
        )
        .await
}

pub async fn events(fixture: &Fixture, domain: &str) -> Vec<Value> {
    let mut found: Vec<Value> = fixture
        .call(Method::GET, AUDIT, Auth::Admin, None)
        .await
        .json()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["domain"] == domain)
        .cloned()
        .collect();
    found.reverse();
    found
}

pub fn same_answer(left: &Reply, right: &Reply) {
    assert_eq!(left.status, right.status);
    assert_eq!(left.body, right.body);
    assert_eq!(set_cookies(left), set_cookies(right));
}

pub fn error_code(reply: &Reply) -> String {
    reply.json()["code"].as_str().unwrap_or_default().to_owned()
}
