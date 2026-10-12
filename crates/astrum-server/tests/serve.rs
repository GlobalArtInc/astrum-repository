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
        web_dir: data_dir.join("web"),
        ldap_cache_ttl: std::time::Duration::from_secs(300),
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

async fn fetch(address: std::net::SocketAddr, path: &str) -> (u16, String) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
    let request = format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).await.unwrap();
    let status = reply
        .split_ascii_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap();
    (status, reply)
}

#[tokio::test]
async fn the_server_serves_the_ui_next_to_the_api() {
    let test = TestDatabase::create().await;
    let data = tempfile::tempdir().unwrap();
    let config = config(&test, data.path().join("data"));
    std::fs::create_dir_all(config.web_dir.join("assets")).unwrap();
    std::fs::write(
        config.web_dir.join("index.html"),
        "<!doctype html><title>astrum-ui-marker</title>",
    )
    .unwrap();
    std::fs::write(config.web_dir.join("assets/app-1.js"), "1").unwrap();

    let server = Server::start(&config).await.unwrap();
    let address = server.local_addr();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let running = tokio::spawn(server.run(async {
        let _ = stopped.await;
    }));
    check(address).await.unwrap();

    for path in ["/", "/browse/maven-releases/org", "/login?returnTo=%2F"] {
        let (status, reply) = fetch(address, path).await;
        assert_eq!(status, 200, "{path}");
        assert!(reply.contains("astrum-ui-marker"), "{path}");
        assert!(
            reply
                .to_ascii_lowercase()
                .contains("content-security-policy: default-src 'self'"),
            "{path}"
        );
    }
    let (status, reply) = fetch(address, "/assets/app-1.js").await;
    assert_eq!(status, 200);
    assert!(reply.to_ascii_lowercase().contains("immutable"));
    let (status, reply) = fetch(address, astrum_api::STATUS).await;
    assert_eq!(status, 200);
    assert!(!reply.contains("astrum-ui-marker"));
    for path in [
        "/service/rest/v1/nothing",
        "/repository/missing/a.txt",
        "/v2/missing/manifests/latest",
    ] {
        let (status, reply) = fetch(address, path).await;
        assert!(status >= 400, "{path}: {status}");
        assert!(!reply.contains("astrum-ui-marker"), "{path}");
    }

    stop.send(()).unwrap();
    running.await.unwrap().unwrap();
}
