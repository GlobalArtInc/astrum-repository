mod support;

use std::collections::BTreeMap;
use std::sync::Arc;

use astrum_logging::{Level, LogFiles, Logging};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture};

const LOGS: &str = "/service/rest/internal/logging/logs";
const MARK: &str = "/service/rest/internal/logging/log/mark";
const LEVELS: &str = "/service/rest/internal/ui/loggingConfiguration";
const CHECKS: &str = "/service/rest/v1/status/check";
const UI_CHECKS: &str = "/service/rest/internal/ui/status-check";
const SYSINFO: &str = "/service/rest/atlas/system-information";
const ZIP_PATH: &str = "/service/rest/v1/support/supportzippath";
const ZIP: &str = "/service/rest/v1/support/supportzip";
const DOWNLOAD: &str = "/service/rest/wonderland/download";

fn with_files(fixture: &Fixture) -> Arc<LogFiles> {
    let files = astrum_logging::open_files(fixture.data.path(), "node-1").unwrap();
    fixture.state.set_logging(Logging::new(
        "node-1",
        "info,sqlx=warn",
        Some(files.clone()),
        |_| Ok(()),
    ));
    files
}

#[tokio::test]
async fn lists_tails_downloads_and_marks_the_log() {
    let fixture = Fixture::new().await;
    let files = with_files(&fixture);
    files
        .write(b"first line\nsecond line\nthird line\n")
        .unwrap();
    let reply = fixture.call(Method::GET, LOGS, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    let listed = reply.json();
    assert_eq!(listed[0]["fileName"], "astrum-node-1.log");
    assert_eq!(listed[0]["size"], 34);
    assert_eq!(listed[0]["node"], "node-1");
    assert_eq!(listed[0]["current"], true);
    assert!(listed[0]["lastModified"].as_i64().unwrap() > 0);

    let tail = fixture
        .call(
            Method::GET,
            &format!("{LOGS}/astrum-node-1.log?bytesCount=-15"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(tail.status, StatusCode::OK);
    assert_eq!(tail.text(), "third line\n");
    assert_eq!(tail.headers["x-astrum-log-size"], "34");
    assert!(
        tail.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );

    let whole = fixture
        .call(
            Method::GET,
            &format!("{LOGS}/astrum-node-1.log"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(whole.text(), "first line\nsecond line\nthird line\n");
    assert_eq!(
        whole.headers["content-disposition"],
        "attachment; filename=\"astrum-node-1.log\""
    );

    let marked = fixture
        .send_body(
            Method::POST,
            MARK,
            Auth::Admin,
            Body::from("before the upgrade"),
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(marked.status, StatusCode::NO_CONTENT);
    let text = std::fs::read_to_string(fixture.data.path().join("log/astrum-node-1.log")).unwrap();
    assert!(
        text.contains("MARK - before the upgrade (by admin)"),
        "{text}"
    );

    let empty = fixture
        .send_body(Method::POST, MARK, Auth::Admin, Body::from("  "), &[])
        .await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);

    for bad in ["..%2Fsecrets.key", "other.txt", "astrum-node-2.log"] {
        let reply = fixture
            .call(Method::GET, &format!("{LOGS}/{bad}"), Auth::Admin, None)
            .await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{bad}");
    }
}

#[tokio::test]
async fn logs_need_the_logging_privileges() {
    let fixture = Fixture::new().await;
    with_files(&fixture);
    fixture.add_user("reader", "secret-1", &[]).await;
    assert_eq!(
        fixture
            .call(Method::GET, LOGS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        fixture
            .call(Method::GET, LOGS, Auth::Basic("reader", "secret-1"), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .call(Method::GET, LEVELS, Auth::Basic("reader", "secret-1"), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .call(Method::GET, CHECKS, Auth::Basic("reader", "secret-1"), None)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        fixture
            .call(
                Method::GET,
                SYSINFO,
                Auth::Basic("reader", "secret-1"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn log_levels_change_at_runtime_and_survive_a_restart() {
    let fixture = Fixture::new().await;
    let applied = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let seen = applied.clone();
    fixture
        .state
        .set_logging(Logging::new("n1", "info,sqlx=warn", None, move |filter| {
            seen.lock().unwrap().push(filter.to_owned());
            Ok(())
        }));
    let listed = fixture
        .call(Method::GET, LEVELS, Auth::Admin, None)
        .await
        .json();
    let root = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|logger| logger["name"] == "ROOT")
        .unwrap();
    assert_eq!(
        root,
        &json!({ "name": "ROOT", "level": "INFO", "override": false })
    );

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{LEVELS}/astrum_content"),
            Auth::Admin,
            Some(json!({ "name": "astrum_content", "level": "DEBUG" })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(
        applied.lock().unwrap().last().unwrap(),
        "info,sqlx=warn,astrum_content=debug"
    );
    let one = fixture
        .call(
            Method::GET,
            &format!("{LEVELS}/astrum_content"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(
        one,
        json!({ "name": "astrum_content", "level": "DEBUG", "override": true })
    );
    let stored: BTreeMap<String, Level> =
        astrum_db::setting::get(fixture.state.db().conn(), "logging.levels")
            .await
            .unwrap()
            .unwrap();
    assert_eq!(
        stored,
        BTreeMap::from([("astrum_content".to_owned(), Level::Debug)])
    );

    let replica = astrum_api::AppState::load(
        fixture.test.migrated().await,
        fixture.data.path().to_owned(),
        astrum_security::SecretBox::from_key([7; 32]),
    )
    .await
    .unwrap();
    astrum_api::routes::support::sync_logging(&replica)
        .await
        .unwrap();
    assert_eq!(
        replica.logging().logger("astrum_content").level,
        Level::Debug
    );

    for (path, body) in [
        (format!("{LEVELS}/org.example"), json!({ "level": "DEBUG" })),
        (format!("{LEVELS}/sqlx"), json!({ "level": "LOUD" })),
        (
            format!("{LEVELS}/sqlx"),
            json!({ "name": "other", "level": "INFO" }),
        ),
    ] {
        let reply = fixture
            .call(Method::PUT, &path, Auth::Admin, Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
    }

    let reset = fixture
        .call(
            Method::POST,
            &format!("{LEVELS}/astrum_content/reset"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    let one = fixture
        .call(
            Method::GET,
            &format!("{LEVELS}/astrum_content"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(one["override"], false);
    assert_eq!(one["level"], "INFO");

    fixture
        .call(
            Method::PUT,
            &format!("{LEVELS}/ROOT"),
            Auth::Admin,
            Some(json!({ "level": "warn" })),
        )
        .await;
    assert_eq!(fixture.state.logging().filter(), "warn,sqlx=warn");
    let reset = fixture
        .call(Method::POST, &format!("{LEVELS}/reset"), Auth::Admin, None)
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    assert_eq!(fixture.state.logging().filter(), "info,sqlx=warn");
    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=logging",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(audit.to_string().contains("astrum_content"));
}

#[tokio::test]
async fn health_checks_report_each_component() {
    let fixture = Fixture::new().await;
    with_files(&fixture);
    let reply = fixture.call(Method::GET, CHECKS, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    let checks = reply.json();
    for name in [
        "Available CPUs",
        "Available Disk Space",
        "Blob Stores Quota",
        "Blob Stores Ready",
        "Database",
        "Database Writes",
        "Default Admin Credentials",
        "Default Role Realm",
        "Log Files",
        "Scheduler",
    ] {
        let check = &checks[name];
        assert!(check["healthy"].is_boolean(), "{name}: {check}");
        assert!(check["timestamp"].is_string(), "{name}");
        assert!(check["time"].is_i64(), "{name}");
    }
    assert_eq!(checks["Database"]["healthy"], true);
    assert_eq!(checks["Log Files"]["healthy"], true);
    assert_eq!(checks["Default Admin Credentials"]["healthy"], false);
    assert_eq!(checks["Scheduler"]["healthy"], false);
    std::fs::remove_file(fixture.data.path().join("admin.password")).unwrap();
    astrum_db::read_only::freeze(fixture.state.db().conn(), "backup", false)
        .await
        .unwrap();
    let checks = fixture
        .call(Method::GET, UI_CHECKS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(checks["Default Admin Credentials"]["healthy"], true);
    assert_eq!(checks["Database Writes"]["healthy"], false);
    assert!(
        checks["Database Writes"]["message"]
            .as_str()
            .unwrap()
            .contains("backup")
    );
}

#[tokio::test]
async fn system_information_shows_versions_without_secrets() {
    let fixture = Fixture::new().await;
    with_files(&fixture);
    fixture.state.set_configuration(vec![
        (
            "database.url".to_owned(),
            "postgres://astrum:***@db/astrum".to_owned(),
        ),
        ("listen".to_owned(), "0.0.0.0:8081".to_owned()),
    ]);
    let reply = fixture.call(Method::GET, SYSINFO, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    let info = reply.json();
    assert_eq!(info["astrum-status"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(info["astrum-node"]["node-id"], "node-1");
    assert_eq!(info["astrum-properties"]["listen"], "0.0.0.0:8081");
    assert_eq!(info["astrum-properties"]["log.filter"], "info,sqlx=warn");
    assert_eq!(info["system-database"]["product"], "PostgreSQL");
    assert!(info["system-database"]["version"].is_string());
    assert!(
        info["system-database"]["appliedMigrations"]
            .as_i64()
            .unwrap()
            > 0
    );
    assert!(
        info["system-runtime"]["availableProcessors"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(info["system-filestores"]["dataDirectory"]["totalSpace"].is_u64());
    assert!(info.get("system-environment").is_none());
    assert!(!reply.text().contains(&fixture.admin_password));
}

#[tokio::test]
async fn support_zips_are_written_downloaded_and_streamed() {
    let fixture = Fixture::new().await;
    let files = with_files(&fixture);
    files.write(b"a log line\n").unwrap();
    let options = json!({
        "systemInformation": true, "threadDump": true, "configuration": true, "security": true,
        "log": true, "taskLog": true, "auditLog": true, "metrics": true, "jmx": true,
        "replication": true, "archivedLog": 0, "limitFileSizes": true, "limitZipSize": true
    });
    let reply = fixture
        .call(Method::POST, ZIP_PATH, Auth::Admin, Some(options.clone()))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let created = reply.json();
    let name = created["name"].as_str().unwrap().to_owned();
    assert!(name.starts_with("support-") && name.ends_with(".zip"));
    assert_eq!(created["truncated"], false);
    assert!(
        created["file"]
            .as_str()
            .unwrap()
            .ends_with(&format!("downloads/{name}"))
    );
    let downloaded = fixture
        .call(
            Method::GET,
            &format!("{DOWNLOAD}/{name}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(&downloaded.body[..2], b"PK");
    assert_eq!(
        downloaded.body.len() as u64,
        created["size"].as_u64().unwrap()
    );
    let listing = String::from_utf8_lossy(&downloaded.body);
    for entry in [
        "info/sysinfo.json",
        "info/configuration.json",
        "info/security.json",
        "info/health.json",
        "log/astrum-node-1.log",
        "log/tasks.log",
        "log/audit.jsonl",
    ] {
        assert!(listing.contains(entry), "{entry}");
    }
    let streamed = fixture
        .call(Method::POST, ZIP, Auth::Admin, Some(options))
        .await;
    assert_eq!(streamed.status, StatusCode::OK);
    assert_eq!(streamed.headers["content-type"], "application/zip");
    assert_eq!(&streamed.body[..2], b"PK");
    let missing = fixture
        .call(
            Method::GET,
            &format!("{DOWNLOAD}/secrets.key"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let anonymous = fixture
        .call(Method::GET, &format!("{DOWNLOAD}/{name}"), Auth::None, None)
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
}
