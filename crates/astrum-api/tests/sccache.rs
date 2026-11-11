mod support;

use std::sync::Arc;

use astrum_security::Scope;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use serde_json::json;
use support::{Auth, Fixture, Reply};

const REPOSITORY: &str = "sccache-cache";
const ENTRY: &str = "/a/b/c/abc0123";
const OCTET_STREAM: &str = "application/octet-stream";
const CHALLENGE: &str = "Basic realm=\"Astrum Repository\"";

fn propfind() -> Method {
    Method::from_bytes(b"PROPFIND").unwrap()
}

fn mkcol() -> Method {
    Method::from_bytes(b"MKCOL").unwrap()
}

async fn booted() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_sccache::Sccache));
    fixture
}

fn document(name: &str, write_policy: &str) -> serde_json::Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": write_policy},
    })
}

async fn repository(fixture: &Fixture, name: &str, write_policy: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/sccache/hosted",
            Auth::Admin,
            Some(document(name, write_policy)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn ready() -> Fixture {
    let fixture = booted().await;
    repository(&fixture, REPOSITORY, "allow").await;
    fixture
}

async fn credentials(fixture: &Fixture, scope: Scope) -> String {
    let token = fixture
        .state
        .security()
        .create_token("admin", "sccache", scope, None)
        .await
        .unwrap()
        .1;
    format!("Bearer {token}")
}

async fn sccache(
    fixture: &Fixture,
    method: Method,
    path: &str,
    authorization: Option<&str>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    sccache_in(
        fixture,
        REPOSITORY,
        method,
        path,
        authorization,
        body,
        headers,
    )
    .await
}

async fn sccache_in(
    fixture: &Fixture,
    repository: &str,
    method: Method,
    path: &str,
    authorization: Option<&str>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    let mut all = vec![("content-type", OCTET_STREAM)];
    if let Some(value) = authorization {
        all.push(("authorization", value));
    }
    all.extend_from_slice(headers);
    fixture
        .send_body(
            method,
            &format!("/repository/{repository}{path}"),
            Auth::None,
            Body::from(body),
            &all,
        )
        .await
}

#[tokio::test]
async fn a_miss_answers_not_found() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let missing = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND, "{}", missing.text());
}

#[tokio::test]
async fn an_entry_goes_up_and_comes_back() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let stored = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"object".to_vec(),
        &[],
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    let fetched = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, b"object");
}

#[tokio::test]
async fn a_second_upload_replaces_the_entry() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"first".to_vec(),
        &[],
    )
    .await;
    assert_eq!(first.status, StatusCode::CREATED);
    let again = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"second".to_vec(),
        &[],
    )
    .await;
    assert_eq!(again.status, StatusCode::NO_CONTENT, "{}", again.text());
    let fetched = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(fetched.body, b"second");
}

#[tokio::test]
async fn a_second_upload_keeps_the_first_when_writes_are_allowed_once() {
    let fixture = booted().await;
    repository(&fixture, "sccache-once", "allow_once").await;
    let writer = credentials(&fixture, Scope::Write).await;
    let first = sccache_in(
        &fixture,
        "sccache-once",
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"first".to_vec(),
        &[],
    )
    .await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.text());
    let again = sccache_in(
        &fixture,
        "sccache-once",
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"second".to_vec(),
        &[],
    )
    .await;
    assert_eq!(again.status, StatusCode::NO_CONTENT, "{}", again.text());
    let fetched = sccache_in(
        &fixture,
        "sccache-once",
        Method::GET,
        ENTRY,
        Some(&writer),
        vec![],
        &[],
    )
    .await;
    assert_eq!(fetched.body, b"first");
}

#[tokio::test]
async fn propfind_of_a_stored_file_describes_it() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"object".to_vec(),
        &[],
    )
    .await;
    let described = sccache(
        &fixture,
        propfind(),
        ENTRY,
        Some(&writer),
        vec![],
        &[("depth", "0")],
    )
    .await;
    assert_eq!(
        described.status,
        StatusCode::MULTI_STATUS,
        "{}",
        described.text()
    );
    assert_eq!(
        described.headers[header::CONTENT_TYPE],
        "application/xml; charset=utf-8"
    );
    let body = described.text();
    assert!(
        body.contains("<D:getcontentlength>6</D:getcontentlength>"),
        "{body}"
    );
    assert!(body.contains("<D:getlastmodified>"), "{body}");
    assert!(body.contains("<D:getetag>\""), "{body}");
    assert!(!body.contains("<D:collection/>"), "{body}");
}

#[tokio::test]
async fn propfind_of_a_missing_path_or_the_root_is_a_collection() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    for path in ["/a/b/", "/a/b/c/abc0999", "", "/"] {
        let described = sccache(
            &fixture,
            propfind(),
            path,
            Some(&writer),
            vec![],
            &[("depth", "0")],
        )
        .await;
        assert_eq!(described.status, StatusCode::MULTI_STATUS, "{path}");
        let body = described.text();
        assert!(
            body.contains("<D:resourcetype><D:collection/></D:resourcetype>"),
            "{path}: {body}"
        );
        assert!(body.contains("<D:getlastmodified>"), "{path}: {body}");
        assert!(!body.contains("<D:getcontentlength>"), "{path}: {body}");
    }
}

#[tokio::test]
async fn propfind_with_depth_one_or_infinity_is_forbidden() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    for depth in ["1", "infinity"] {
        let listing = sccache(
            &fixture,
            propfind(),
            "/a/b/",
            Some(&writer),
            vec![],
            &[("depth", depth)],
        )
        .await;
        assert_eq!(listing.status, StatusCode::FORBIDDEN, "{depth}");
    }
}

#[tokio::test]
async fn mkcol_answers_created_and_stores_nothing() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let made = sccache(&fixture, mkcol(), "/a/b/c/", Some(&writer), vec![], &[]).await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let missing = sccache(&fixture, Method::GET, "/a/b/c", Some(&writer), vec![], &[]).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn options_announce_webdav_and_the_methods() {
    let fixture = ready().await;
    let reader = credentials(&fixture, Scope::Read).await;
    let answer = sccache(&fixture, Method::OPTIONS, "/", Some(&reader), vec![], &[]).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.text());
    assert_eq!(answer.headers["dav"], "1");
    assert_eq!(
        answer.headers[header::ALLOW],
        "GET,HEAD,PUT,DELETE,PROPFIND,MKCOL,OPTIONS"
    );
}

#[tokio::test]
async fn a_path_outside_the_grammar_is_refused() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let seventeen = format!("/{}", vec!["a"; 17].join("/"));
    let long = format!("/{}", "a".repeat(513));
    for path in [
        "/a%20b/c",
        "/a%2Ab",
        "/a/%2e%2e/b",
        seventeen.as_str(),
        long.as_str(),
    ] {
        let refused = sccache(
            &fixture,
            Method::PUT,
            path,
            Some(&writer),
            b"x".to_vec(),
            &[],
        )
        .await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{path}");
    }
    for path in ["/a%20b/c", seventeen.as_str(), long.as_str()] {
        let missing = sccache(&fixture, Method::GET, path, Some(&writer), vec![], &[]).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn a_read_token_cannot_upload_or_make_directories() {
    let fixture = ready().await;
    let reader = credentials(&fixture, Scope::Read).await;
    let refused = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&reader),
        b"entry".to_vec(),
        &[],
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.text());
    let made = sccache(&fixture, mkcol(), "/a/b/c/", Some(&reader), vec![], &[]).await;
    assert_eq!(made.status, StatusCode::FORBIDDEN, "{}", made.text());
}

#[tokio::test]
async fn anonymous_callers_are_challenged() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"entry".to_vec(),
        &[],
    )
    .await;
    let challenged = sccache(&fixture, Method::GET, ENTRY, None, vec![], &[]).await;
    assert_eq!(challenged.status, StatusCode::UNAUTHORIZED);
    assert_eq!(challenged.headers[header::WWW_AUTHENTICATE], CHALLENGE);
}

#[tokio::test]
async fn an_administrator_deletes_an_entry() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"entry".to_vec(),
        &[],
    )
    .await;
    let deleted = fixture
        .send_body(
            Method::DELETE,
            &format!("/repository/{REPOSITORY}{ENTRY}"),
            Auth::Admin,
            Body::empty(),
            &[],
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let missing = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_large_upload_to_a_stored_key_is_read_to_the_end() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"small".to_vec(),
        &[],
    )
    .await;
    let big = vec![7u8; 3 * 1024 * 1024];
    let replaced = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        big.clone(),
        &[],
    )
    .await;
    assert_eq!(
        replaced.status,
        StatusCode::NO_CONTENT,
        "{}",
        replaced.text()
    );
    let fetched = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(fetched.body, big);
}

#[tokio::test]
async fn a_read_only_repository_refuses_writes_and_keeps_its_entries() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"first".to_vec(),
        &[],
    )
    .await;
    let closed = fixture
        .call(
            Method::PUT,
            &format!("/service/rest/v1/repositories/sccache/hosted/{REPOSITORY}"),
            Auth::Admin,
            Some(document(REPOSITORY, "deny")),
        )
        .await;
    assert_eq!(closed.status, StatusCode::NO_CONTENT, "{}", closed.text());
    let refused = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"second".to_vec(),
        &[],
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    let new_entry = sccache(
        &fixture,
        Method::PUT,
        "/a/b/c/abc0999",
        Some(&writer),
        b"new".to_vec(),
        &[],
    )
    .await;
    assert_eq!(
        new_entry.status,
        StatusCode::BAD_REQUEST,
        "{}",
        new_entry.text()
    );
    let fetched = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, b"first");
    let missing = sccache(
        &fixture,
        Method::GET,
        "/a/b/c/abc0999",
        Some(&writer),
        vec![],
        &[],
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_refused_large_upload_is_read_to_the_end() {
    let fixture = ready().await;
    let reader = credentials(&fixture, Scope::Read).await;
    let writer = credentials(&fixture, Scope::Write).await;
    repository(&fixture, "sccache-closed", "deny").await;
    let big = vec![7u8; 4 * 1024 * 1024];
    let read_only_token = sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&reader),
        big.clone(),
        &[],
    )
    .await;
    assert_eq!(
        read_only_token.status,
        StatusCode::FORBIDDEN,
        "{}",
        read_only_token.text()
    );
    let closed = sccache_in(
        &fixture,
        "sccache-closed",
        Method::PUT,
        ENTRY,
        Some(&writer),
        big,
        &[],
    )
    .await;
    assert_eq!(closed.status, StatusCode::BAD_REQUEST, "{}", closed.text());
}

#[tokio::test]
async fn an_entry_is_served_as_octet_stream_whatever_its_bytes() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    sccache(
        &fixture,
        Method::PUT,
        ENTRY,
        Some(&writer),
        b"<html><script>alert(1)</script></html>".to_vec(),
        &[],
    )
    .await;
    let fetched = sccache(&fixture, Method::GET, ENTRY, Some(&writer), vec![], &[]).await;
    assert_eq!(fetched.headers[header::CONTENT_TYPE], OCTET_STREAM);
}

#[tokio::test]
async fn the_client_start_up_check_runs_in_order() {
    let fixture = ready().await;
    let writer = credentials(&fixture, Scope::Write).await;
    let probe = "/.sccache_check";
    let before = sccache(&fixture, Method::GET, probe, Some(&writer), vec![], &[]).await;
    assert_eq!(before.status, StatusCode::NOT_FOUND);
    let written = sccache(
        &fixture,
        Method::PUT,
        probe,
        Some(&writer),
        b"Hello, World!".to_vec(),
        &[],
    )
    .await;
    assert_eq!(written.status, StatusCode::CREATED, "{}", written.text());
    let parent = sccache(
        &fixture,
        propfind(),
        "/",
        Some(&writer),
        vec![],
        &[("depth", "0")],
    )
    .await;
    assert_eq!(parent.status, StatusCode::MULTI_STATUS, "{}", parent.text());
    let after = sccache(&fixture, Method::GET, probe, Some(&writer), vec![], &[]).await;
    assert_eq!(after.status, StatusCode::OK);
    assert_eq!(after.body, b"Hello, World!");
}
