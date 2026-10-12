use astrum_api::{AppState, STATUS, STATUS_WRITABLE, router};
use astrum_db::read_only;
use astrum_testkit::TestDatabase;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;

struct Fixture {
    test: TestDatabase,
    data: TempDir,
    state: AppState,
}

async fn fixture() -> Fixture {
    let test = TestDatabase::create().await;
    let data = tempfile::tempdir().unwrap();
    std::fs::create_dir(data.path().join("tmp")).unwrap();
    let state = AppState::new(test.migrated().await, data.path().to_owned());
    Fixture { test, data, state }
}

async fn call(state: &AppState, method: Method, path: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let response = router(state.clone()).oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, body.to_vec())
}

#[tokio::test]
async fn a_healthy_server_reads_and_writes() {
    let fixture = fixture().await;
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS).await,
        (StatusCode::OK, vec![])
    );
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS_WRITABLE).await,
        (StatusCode::OK, vec![])
    );
    assert_eq!(
        call(&fixture.state, Method::HEAD, STATUS).await.0,
        StatusCode::OK
    );
    let left: Vec<_> = std::fs::read_dir(fixture.data.path().join("tmp"))
        .unwrap()
        .collect();
    assert!(left.is_empty(), "the write probe leaves nothing behind");
}

#[tokio::test]
async fn read_only_mode_stops_writes_but_not_reads() {
    let fixture = fixture().await;
    read_only::freeze(fixture.state.db().conn(), "backup", false)
        .await
        .unwrap();
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS_WRITABLE).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );

    read_only::release(fixture.state.db().conn()).await.unwrap();
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS_WRITABLE).await.0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_data_directory_that_cannot_be_written_stops_writes() {
    let fixture = fixture().await;
    let state = AppState::new(
        fixture.state.db().clone(),
        fixture.data.path().join("missing"),
    );
    assert_eq!(call(&state, Method::GET, STATUS).await.0, StatusCode::OK);
    assert_eq!(
        call(&state, Method::GET, STATUS_WRITABLE).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn a_lost_database_makes_the_server_unavailable() {
    let fixture = fixture().await;
    fixture.state.db().clone().close().await.unwrap();
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        call(&fixture.state, Method::GET, STATUS_WRITABLE).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    drop(fixture.test);
}

#[tokio::test]
async fn unknown_paths_are_not_found() {
    let fixture = fixture().await;
    for path in [
        "/",
        "/service/rest/v1/nothing",
        "/repository/maven-releases/",
    ] {
        assert_eq!(
            call(&fixture.state, Method::GET, path).await.0,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
}
