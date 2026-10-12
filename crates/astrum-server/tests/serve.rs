use std::path::PathBuf;

use astrum_server::Server;
use astrum_server::check::{check, status};
use astrum_server::config::{Config, LogConfig};
use astrum_testkit::TestDatabase;

fn config(test: &TestDatabase, data_dir: PathBuf) -> Config {
    Config {
        listen: "127.0.0.1:0".parse().unwrap(),
        data_dir: data_dir.clone(),
        database: Some(test.config()),
        log: LogConfig::default(),
        secrets_key: data_dir.join("etc/secrets.key"),
        file: data_dir.join("etc/astrum.toml"),
        file_found: false,
    }
}

#[tokio::test]
async fn the_server_starts_answers_and_stops() {
    let test = TestDatabase::create().await;
    let data = tempfile::tempdir().unwrap();
    let config = config(&test, data.path().join("data"));

    let server = Server::start(&config).await.unwrap();
    let address = server.local_addr();
    assert!(config.data_dir.join("tmp").is_dir());
    assert!(config.data_dir.join("blobs/default/content").is_dir());

    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let running = tokio::spawn(server.run(async {
        let _ = stopped.await;
    }));

    check(address).await.unwrap();
    assert_eq!(
        status(address, astrum_api::STATUS_WRITABLE).await.unwrap(),
        200
    );
    assert_eq!(status(address, "/nothing-here").await.unwrap(), 404);

    stop.send(()).unwrap();
    running.await.unwrap().unwrap();
    assert!(check(address).await.is_err());
}

#[tokio::test]
async fn a_second_start_finds_the_schema_ready() {
    let test = TestDatabase::create().await;
    let data = tempfile::tempdir().unwrap();
    let config = config(&test, data.path().to_owned());

    let applied = astrum_server::migrate(&config).await.unwrap();
    assert!(!applied.is_empty());
    assert!(astrum_server::migrate(&config).await.unwrap().is_empty());
    let server = Server::start(&config).await.unwrap();
    drop(server);
}

#[tokio::test]
async fn without_a_database_the_server_does_not_start() {
    let data = tempfile::tempdir().unwrap();
    let config = Config {
        database: None,
        ..config(&TestDatabase::create().await, data.path().to_owned())
    };
    let error = Server::start(&config).await.err().unwrap().to_string();
    assert!(error.contains("ASTRUM_DATABASE_URL"), "{error}");
}

#[test]
fn the_version_line_names_the_product() {
    assert!(astrum_server::version_line().starts_with("Astrum Repository 3."));
}
