mod support;

use astrum_api::{AppState, STATUS, STATUS_WRITABLE};
use astrum_db::read_only;
use axum::http::{Method, StatusCode};
use support::{Auth, Fixture};

#[tokio::test]
async fn a_healthy_server_reads_and_writes() {
    let fixture = Fixture::new().await;
    let reply = fixture.call(Method::GET, STATUS, Auth::None, None).await;
    assert_eq!((reply.status, reply.body.len()), (StatusCode::OK, 0));
    let reply = fixture
        .call(Method::GET, STATUS_WRITABLE, Auth::None, None)
        .await;
    assert_eq!((reply.status, reply.body.len()), (StatusCode::OK, 0));
    assert_eq!(
        fixture
            .call(Method::HEAD, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
    let left: Vec<_> = std::fs::read_dir(fixture.data.path().join("tmp"))
        .unwrap()
        .collect();
    assert!(left.is_empty(), "the write probe leaves nothing behind");
}

#[tokio::test]
async fn read_only_mode_stops_writes_but_not_reads() {
    let fixture = Fixture::new().await;
    read_only::freeze(fixture.state.db().conn(), "backup", false)
        .await
        .unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        fixture
            .call(Method::GET, STATUS_WRITABLE, Auth::None, None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    read_only::release(fixture.state.db().conn()).await.unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, STATUS_WRITABLE, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_data_directory_that_cannot_be_written_stops_writes() {
    let fixture = Fixture::new().await;
    let state = AppState::load(
        fixture.state.db().clone(),
        fixture.data.path().join("missing"),
        astrum_security::SecretBox::from_key([1; 32]),
    )
    .await
    .unwrap();
    let moved = Fixture { state, ..fixture };
    assert_eq!(
        moved
            .call(Method::GET, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        moved
            .call(Method::GET, STATUS_WRITABLE, Auth::None, None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn a_lost_database_makes_the_server_unavailable() {
    let fixture = Fixture::new().await;
    fixture.state.db().clone().close().await.unwrap();
    assert_eq!(
        fixture
            .call(Method::GET, STATUS, Auth::None, None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        fixture
            .call(Method::GET, STATUS_WRITABLE, Auth::None, None)
            .await
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn unknown_paths_are_not_found() {
    let fixture = Fixture::new().await;
    for path in [
        "/",
        "/service/rest/v1/nothing",
        "/repository/maven-releases/",
    ] {
        assert_eq!(
            fixture
                .call(Method::GET, path, Auth::None, None)
                .await
                .status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
}
