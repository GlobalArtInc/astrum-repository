mod support;

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REPOSITORIES: &str = "/service/rest/v1/repositories/raw";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    fixture
}

fn hosted(name: &str, quota: Option<Value>) -> Value {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    });
    if let Some(quota) = quota {
        body["storage"]["quota"] = quota;
    }
    body
}

fn hard(limit: i64) -> Value {
    json!({"limitBytes": limit, "enforce": true})
}

async fn create(fixture: &Fixture, name: &str, quota: Option<Value>) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/hosted"),
            Auth::Admin,
            Some(hosted(name, quota)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn set_quota(fixture: &Fixture, name: &str, quota: Option<Value>) {
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{REPOSITORIES}/hosted/{name}"),
            Auth::Admin,
            Some(hosted(name, quota)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn put_with(
    fixture: &Fixture,
    repository: &str,
    path: &str,
    size: usize,
    declare: bool,
) -> Reply {
    let length = size.to_string();
    let headers: Vec<(&str, &str)> = match declare {
        true => vec![("content-length", length.as_str())],
        false => Vec::new(),
    };
    fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}/{path}"),
            Auth::Admin,
            Body::from(vec![b'x'; size]),
            &headers,
        )
        .await
}

async fn put(fixture: &Fixture, repository: &str, path: &str, size: usize) -> StatusCode {
    put_with(fixture, repository, path, size, false)
        .await
        .status
}

async fn used(fixture: &Fixture, repository: &str) -> i64 {
    astrum_content::usage::current(fixture.state.db().conn(), repository)
        .await
        .unwrap()
        .unwrap()
        .bytes
}

fn blob_files(fixture: &Fixture) -> usize {
    fn walk(dir: &Path) -> usize {
        std::fs::read_dir(dir).map_or(0, |entries| {
            entries
                .flatten()
                .map(|entry| match entry.path().is_dir() {
                    true => walk(&entry.path()),
                    false => 1,
                })
                .sum()
        })
    }
    walk(&fixture.data.path().join("blobs/default/content"))
}

async fn blobs_settle_at(fixture: &Fixture, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while blob_files(fixture) != expected && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        blob_files(fixture),
        expected,
        "a refused upload leaves no blob"
    );
}

#[tokio::test]
async fn a_put_over_the_limit_is_refused_with_413_and_no_blob_stays() {
    let fixture = ready().await;
    create(&fixture, "capped", Some(hard(100))).await;
    assert_eq!(
        put(&fixture, "capped", "a.bin", 60).await,
        StatusCode::CREATED
    );
    assert_eq!(blob_files(&fixture), 1);

    let declared = put_with(&fixture, "capped", "b.bin", 60, true).await;
    assert_eq!(
        declared.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        declared.text()
    );
    let message = declared.text();
    for part in ["capped", "100", "60"] {
        assert!(message.contains(part), "{message}");
    }
    let counted = put_with(&fixture, "capped", "c.bin", 60, false).await;
    assert_eq!(
        counted.status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "{}",
        counted.text()
    );
    blobs_settle_at(&fixture, 1).await;
    assert_eq!(used(&fixture, "capped").await, 60);

    assert_eq!(
        put(&fixture, "capped", "d.bin", 40).await,
        StatusCode::CREATED
    );
    assert_eq!(used(&fixture, "capped").await, 100);
}

#[tokio::test]
async fn shrinking_replacements_deletes_and_reads_pass_while_over_the_limit() {
    let fixture = ready().await;
    create(&fixture, "tight", None).await;
    assert_eq!(
        put(&fixture, "tight", "a.bin", 100).await,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, "tight", "b.bin", 100).await,
        StatusCode::CREATED
    );
    set_quota(&fixture, "tight", Some(hard(150))).await;

    assert_eq!(
        put(&fixture, "tight", "c.bin", 1).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        put(&fixture, "tight", "a.bin", 101).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let read = fixture
        .call(Method::GET, "/repository/tight/a.bin", Auth::Admin, None)
        .await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(read.body.len(), 100);

    assert_eq!(
        put(&fixture, "tight", "a.bin", 50).await,
        StatusCode::CREATED
    );
    assert_eq!(used(&fixture, "tight").await, 150);
    assert_eq!(
        put(&fixture, "tight", "a.bin", 51).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        put(&fixture, "tight", "b.bin", 100).await,
        StatusCode::CREATED
    );

    let deleted = fixture
        .call(Method::DELETE, "/repository/tight/b.bin", Auth::Admin, None)
        .await;
    assert!(deleted.status.is_success(), "{}", deleted.text());
    assert_eq!(used(&fixture, "tight").await, 50);
    assert_eq!(
        put(&fixture, "tight", "e.bin", 100).await,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, "tight", "f.bin", 1).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_uploads_that_fit_alone_but_not_together_give_exactly_one_413() {
    let fixture = ready().await;
    for round in 0..20 {
        let name = format!("race{round}");
        create(&fixture, &name, Some(hard(100))).await;
        let (first, second) = tokio::join!(
            put(&fixture, &name, "one.bin", 60),
            put(&fixture, &name, "two.bin", 60)
        );
        let mut statuses = [first, second];
        statuses.sort();
        assert_eq!(
            statuses,
            [StatusCode::CREATED, StatusCode::PAYLOAD_TOO_LARGE],
            "round {round}"
        );
        assert_eq!(used(&fixture, &name).await, 60, "round {round}");
    }
}

#[tokio::test]
async fn a_repository_without_a_hard_limit_takes_any_upload() {
    let fixture = ready().await;
    create(&fixture, "free", None).await;
    create(
        &fixture,
        "soft",
        Some(json!({"limitBytes": 10, "warnPercent": 50, "enforce": false})),
    )
    .await;
    for name in ["free", "soft"] {
        assert_eq!(
            put(&fixture, name, "big.bin", 5000).await,
            StatusCode::CREATED
        );
        assert_eq!(used(&fixture, name).await, 5000);
    }
    let stored = fixture
        .call(
            Method::GET,
            &format!("{REPOSITORIES}/hosted/soft"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.json()["storage"]["quota"]["warnPercent"], 50);
}

#[tokio::test]
async fn quota_settings_are_validated() {
    let fixture = ready().await;
    let cases = [
        (json!({"limitBytes": 0}), "PARAMETER quota.limitBytes"),
        (json!({"limitBytes": -5}), "PARAMETER quota.limitBytes"),
        (json!({"enforce": true}), "PARAMETER quota.limitBytes"),
        (
            json!({"limitBytes": 5, "warnPercent": 0}),
            "PARAMETER quota.warnPercent",
        ),
        (
            json!({"limitBytes": 5, "warnPercent": 101}),
            "PARAMETER quota.warnPercent",
        ),
    ];
    for (quota, id) in cases {
        let reply = fixture
            .call(
                Method::POST,
                &format!("{REPOSITORIES}/hosted"),
                Auth::Admin,
                Some(hosted("bad", Some(quota.clone()))),
            )
            .await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{quota}: {}",
            reply.text()
        );
        assert_eq!(reply.json()[0]["id"], id, "{quota}");
    }
    create(
        &fixture,
        "edge",
        Some(json!({"limitBytes": 1, "warnPercent": 100})),
    )
    .await;
    let stored = fixture
        .call(
            Method::GET,
            &format!("{REPOSITORIES}/hosted/edge"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.json()["storage"]["quota"]["limitBytes"], 1);

    let mut proxy = json!({
        "name": "mirror",
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "quota": hard(10)},
        "proxy": {"remoteUrl": "https://example.org/", "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 60},
        "httpClient": {"blocked": false, "autoBlock": true},
        "raw": {"contentDisposition": "ATTACHMENT"}
    });
    let refused = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/proxy"),
            Auth::Admin,
            Some(proxy.clone()),
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert!(
        refused
            .json()
            .as_array()
            .unwrap()
            .iter()
            .any(|problem| problem["id"] == "PARAMETER quota.enforce"),
        "{}",
        refused.text()
    );
    proxy["storage"]["quota"]["enforce"] = false.into();
    let accepted = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/proxy"),
            Auth::Admin,
            Some(proxy),
        )
        .await;
    assert_eq!(accepted.status, StatusCode::CREATED, "{}", accepted.text());
}
