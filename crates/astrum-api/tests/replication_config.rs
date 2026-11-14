mod accounts;
mod support;

use accounts::{audit, create_repository, create_role, create_user};
use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REMOTES: &str = "/service/rest/v1/replication/remotes";
const REPLICATIONS: &str = "/service/rest/v1/replications";
const FIRST_TOKEN: &str = "first-secret-token-0001";
const SECOND_TOKEN: &str = "second-secret-token-0002";
type Operation = (
    &'static str,
    Method,
    String,
    Option<Value>,
    &'static [&'static str],
);

const UNKNOWN: &str = "00000000-0000-0000-0000-000000000001";

async fn api(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    body: Option<Value>,
) -> Reply {
    let reply = fixture.call(method, path, auth, body).await;
    let text = reply.text();
    for secret in [FIRST_TOKEN, SECOND_TOKEN] {
        assert!(!text.contains(secret), "a reply carries a token: {text}");
    }
    reply
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Option<Value>) -> Reply {
    api(fixture, method, path, Auth::Admin, body).await
}

fn remote(name: &str) -> Value {
    json!({"name": name, "url": "https://203.0.113.10", "token": FIRST_TOKEN})
}

async fn make_remote(fixture: &Fixture, name: &str) -> String {
    let reply = admin(fixture, Method::POST, REMOTES, Some(remote(name))).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["id"].as_str().unwrap().to_owned()
}

fn replication(name: &str, remote_id: &str) -> Value {
    json!({"name": name, "sourceRepository": "src", "remoteId": remote_id, "remoteRepository": "dst"})
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    create_repository(&fixture, "src", "raw").await;
    fixture
}

fn fields(reply: &Reply) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["id"].as_str().unwrap().to_owned())
        .collect()
}

async fn stored_token(fixture: &Fixture, id: &str) -> String {
    let row = fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT token FROM replication_remote WHERE id = '{id}'"),
        ))
        .await
        .unwrap()
        .unwrap();
    row.try_get("", "token").unwrap()
}

#[tokio::test]
async fn a_remote_keeps_its_token_sealed_and_never_shows_it() {
    let fixture = ready().await;
    let created = admin(&fixture, Method::POST, REMOTES, Some(remote("Mirror"))).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let body = created.json();
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["hasToken"], true);
    assert!(body.get("token").is_none());
    assert_eq!(body["maxConcurrency"], 2);
    assert_eq!(body["url"], "https://203.0.113.10");
    let sealed = stored_token(&fixture, &id).await;
    assert!(astrum_security::SecretBox::is_sealed(&sealed));
    assert_eq!(fixture.state.secrets().open(&sealed).unwrap(), FIRST_TOKEN);
    let list = admin(&fixture, Method::GET, REMOTES, None).await;
    assert_eq!(list.json().as_array().unwrap().len(), 1);
    let one = admin(&fixture, Method::GET, &format!("{REMOTES}/{id}"), None).await;
    assert_eq!(one.json()["name"], "Mirror");

    let kept = admin(
        &fixture,
        Method::PUT,
        &format!("{REMOTES}/{id}"),
        Some(json!({"name": "Mirror 2", "url": "https://203.0.113.10/", "token": null, "maxConcurrency": 4})),
    )
    .await;
    assert_eq!(kept.status, StatusCode::OK, "{}", kept.text());
    assert_eq!(kept.json()["name"], "Mirror 2");
    assert_eq!(kept.json()["maxConcurrency"], 4);
    assert_eq!(stored_token(&fixture, &id).await, sealed);

    let replaced = admin(
        &fixture,
        Method::PUT,
        &format!("{REMOTES}/{id}"),
        Some(json!({"name": "Mirror 2", "url": "https://203.0.113.10", "token": SECOND_TOKEN})),
    )
    .await;
    assert_eq!(replaced.status, StatusCode::OK, "{}", replaced.text());
    let now = stored_token(&fixture, &id).await;
    assert_eq!(fixture.state.secrets().open(&now).unwrap(), SECOND_TOKEN);

    let blank = admin(
        &fixture,
        Method::PUT,
        &format!("{REMOTES}/{id}"),
        Some(json!({"name": "Mirror 2", "url": "https://203.0.113.10", "token": ""})),
    )
    .await;
    assert_eq!(blank.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&blank), ["token"]);

    let moved = admin(
        &fixture,
        Method::PUT,
        &format!("{REMOTES}/{id}"),
        Some(json!({"name": "Mirror 2", "url": "https://203.0.113.11"})),
    )
    .await;
    assert_eq!(moved.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&moved), ["token"]);

    let events = audit(&fixture, "replication").await;
    let kinds: Vec<&str> = events.iter().map(|event| event.0.as_str()).collect();
    assert_eq!(
        kinds,
        ["remote.created", "remote.updated", "remote.updated"]
    );
    let changed: Vec<&Value> = events
        .iter()
        .map(|event| &event.3["tokenChanged"])
        .collect();
    assert_eq!(changed, [&json!(true), &json!(false), &json!(true)]);
    for event in &events {
        let text = event.3.to_string();
        assert!(
            !text.contains(FIRST_TOKEN)
                && !text.contains(SECOND_TOKEN)
                && !text.contains("sealed:")
        );
        assert_eq!(event.2.split('/').next(), Some("admin"));
    }

    let gone = admin(&fixture, Method::DELETE, &format!("{REMOTES}/{id}"), None).await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT);
    let again = admin(&fixture, Method::GET, &format!("{REMOTES}/{id}"), None).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let events = audit(&fixture, "replication").await;
    assert_eq!(events.last().unwrap().0, "remote.deleted");
}

#[tokio::test]
async fn a_remote_address_must_be_a_bare_server_address() {
    let fixture = ready().await;
    let counter = std::cell::Cell::new(0);
    let attempt = |url: &str, private: bool| {
        counter.set(counter.get() + 1);
        json!({"name": format!("remote {}", counter.get()), "url": url, "token": FIRST_TOKEN, "allowPrivateNetwork": private})
    };
    let create = |body: Value| admin(&fixture, Method::POST, REMOTES, Some(body));
    for (url, private) in [
        ("http://203.0.113.10", false),
        ("https://user:pass@203.0.113.10", false),
        ("https://203.0.113.10/path", false),
        ("https://203.0.113.10/?a=1", false),
        ("https://203.0.113.10/#frag", false),
        ("ftp://203.0.113.10", true),
        ("", false),
        ("not a url", false),
        ("https://10.0.0.5", false),
        ("https://127.0.0.1", true),
        ("http://127.0.0.1:8081", true),
        ("http://203.0.113.7:8081", true),
        ("http://[2001:db8::1]:8081", true),
        ("https://169.254.10.10", true),
        ("https://169.254.169.254", true),
        ("https://[::1]", true),
        ("https://[fe80::1]", true),
    ] {
        let reply = create(attempt(url, private)).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{url}: {}",
            reply.text()
        );
        assert_eq!(fields(&reply), ["url"], "{url}");
    }
    for (url, private) in [
        ("http://10.0.0.5:8080", true),
        ("https://192.168.1.2", true),
        ("https://[fd12::1]:8443", true),
        ("https://100.64.0.9", true),
        ("https://astrum.unresolvable.invalid", false),
        ("https://203.0.113.20:8443/", false),
    ] {
        let reply = create(attempt(url, private)).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{url}: {}", reply.text());
    }
    let listed = admin(&fixture, Method::GET, REMOTES, None).await;
    let urls: Vec<String> = listed
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["url"].as_str().unwrap().to_owned())
        .collect();
    assert!(urls.contains(&"https://203.0.113.20:8443".to_owned()));
}

#[tokio::test]
async fn remote_fields_are_checked_and_names_are_unique() {
    let fixture = ready().await;
    make_remote(&fixture, "Alpha").await;
    let clash = admin(&fixture, Method::POST, REMOTES, Some(remote("alpha"))).await;
    assert_eq!(clash.status, StatusCode::CONFLICT);
    let mut body = remote("beta");
    body["maxConcurrency"] = json!(17);
    body["maxBytesPerSecond"] = json!(0);
    body["token"] = json!("has space");
    let bad = admin(&fixture, Method::POST, REMOTES, Some(body)).await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        fields(&bad),
        ["token", "maxConcurrency", "maxBytesPerSecond"]
    );
    let mut body = remote("gamma");
    body.as_object_mut().unwrap().remove("token");
    let bad = admin(&fixture, Method::POST, REMOTES, Some(body)).await;
    assert_eq!(fields(&bad), ["token"]);
    let long = admin(
        &fixture,
        Method::POST,
        REMOTES,
        Some(remote(&"n".repeat(101))),
    )
    .await;
    assert_eq!(fields(&long), ["name"]);
    let control = admin(&fixture, Method::POST, REMOTES, Some(remote("a\u{7}b"))).await;
    assert_eq!(fields(&control), ["name"]);
    let mut limited = remote("delta");
    limited["maxBytesPerSecond"] = json!(1_000_000);
    let reply = admin(&fixture, Method::POST, REMOTES, Some(limited)).await;
    assert_eq!(reply.json()["maxBytesPerSecond"], 1_000_000);
}

#[tokio::test]
async fn replications_are_validated_and_unique() {
    let fixture = ready().await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    let created = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("Nightly", &remote_id)),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let body = created.json();
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["propagateDeletes"], false);
    assert_eq!(body["settleSeconds"], 30);
    assert_eq!(body["reconcileMinutes"], 360);
    assert_eq!(body["alertState"], "ok");
    assert_eq!(body["alertChannels"], json!([]));

    let same_name = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("nightly", &remote_id)),
    )
    .await;
    assert_eq!(same_name.status, StatusCode::CONFLICT);
    let same_target = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("Other", &remote_id)),
    )
    .await;
    assert_eq!(same_target.status, StatusCode::CONFLICT);

    let mut read_only = replication("Nightly", &remote_id);
    read_only["alertState"] = json!("failing");
    read_only["lastError"] = json!("boom");
    read_only["enabled"] = json!(false);
    read_only["nameFilter"] = json!("lib-.*");
    read_only["settleSeconds"] = json!(0);
    let updated = admin(
        &fixture,
        Method::PUT,
        &format!("{REPLICATIONS}/{id}"),
        Some(read_only),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    assert_eq!(updated.json()["alertState"], "ok");
    assert_eq!(updated.json()["lastError"], Value::Null);
    assert_eq!(updated.json()["enabled"], false);
    assert_eq!(updated.json()["nameFilter"], "lib-.*");
    let listed = admin(&fixture, Method::GET, REPLICATIONS, None).await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);

    let mut bad = replication("Bad", &remote_id);
    bad["remoteRepository"] = json!("bad name!");
    bad["settleSeconds"] = json!(3601);
    bad["reconcileMinutes"] = json!(4);
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(bad)).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        fields(&reply),
        ["remoteRepository", "settleSeconds", "reconcileMinutes"]
    );

    let events = audit(&fixture, "replication").await;
    let kinds: Vec<&str> = events.iter().map(|event| event.0.as_str()).collect();
    assert_eq!(
        kinds,
        [
            "remote.created",
            "replication.created",
            "replication.updated"
        ]
    );
}

#[tokio::test]
async fn filters_are_compiled_with_a_size_limit() {
    let fixture = ready().await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    for (field, pattern) in [
        ("nameFilter", "(unclosed".to_owned()),
        ("versionFilter", "a)|(b".to_owned()),
        ("nameFilter", "a".repeat(10 * 1024 + 1)),
        ("versionFilter", "((a{1000}){1000}){1000}".to_owned()),
        ("nameFilter", "(x{100}){100}{100}".repeat(2000)),
    ] {
        let mut body = replication("Filtered", &remote_id);
        body[field] = json!(pattern);
        let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(body)).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{field}: {}",
            reply.text()
        );
        assert_eq!(fields(&reply), [field]);
    }
    let mut body = replication("Filtered", &remote_id);
    body["nameFilter"] = json!("a".repeat(10 * 1024));
    body["versionFilter"] = json!(r"\d+\.\d+\..*");
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(body)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

#[tokio::test]
async fn sources_remotes_and_channels_must_exist() {
    let fixture = ready().await;
    create_repository(&fixture, "mirror-of-central", "raw").await;
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared("UPDATE repository SET type = 'proxy' WHERE name = 'mirror-of-central'")
        .await
        .unwrap();
    let remote_id = make_remote(&fixture, "Mirror").await;

    let mut proxy = replication("Proxy", &remote_id);
    proxy["sourceRepository"] = json!("mirror-of-central");
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(proxy)).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&reply), ["sourceRepository"]);

    let mut missing = replication("Missing", &remote_id);
    missing["sourceRepository"] = json!("nope");
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(missing)).await;
    assert_eq!(fields(&reply), ["sourceRepository"]);

    let reply = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("NoRemote", UNKNOWN)),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&reply), ["remoteId"]);

    let mut unknown = replication("Channels", &remote_id);
    unknown["alertChannels"] = json!([UNKNOWN]);
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(unknown)).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(fields(&reply), ["alertChannels"]);

    let channel = "11111111-1111-1111-1111-111111111111";
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(&format!(
            "INSERT INTO notification_channel (id, name, type) VALUES ('{channel}', 'ops', 'webhook')"
        ))
        .await
        .unwrap();
    let mut known = replication("Channels", &remote_id);
    known["alertChannels"] = json!([channel, channel]);
    let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(known)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(reply.json()["alertChannels"], json!([channel]));
}

#[tokio::test]
async fn a_remote_in_use_cannot_be_deleted() {
    let fixture = ready().await;
    create_repository(&fixture, "other", "raw").await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    let first = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("Zulu", &remote_id)),
    )
    .await;
    let mut second = replication("Alpha", &remote_id);
    second["sourceRepository"] = json!("other");
    admin(&fixture, Method::POST, REPLICATIONS, Some(second)).await;
    let refused = admin(
        &fixture,
        Method::DELETE,
        &format!("{REMOTES}/{remote_id}"),
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    let message = refused.json()["message"].as_str().unwrap().to_owned();
    assert!(message.ends_with("Alpha, Zulu"), "{message}");
    let id = first.json()["id"].as_str().unwrap().to_owned();
    let gone = admin(
        &fixture,
        Method::DELETE,
        &format!("{REPLICATIONS}/{id}"),
        None,
    )
    .await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT);
    let still = admin(
        &fixture,
        Method::DELETE,
        &format!("{REMOTES}/{remote_id}"),
        None,
    )
    .await;
    assert_eq!(still.status, StatusCode::CONFLICT);
    let missing = admin(
        &fixture,
        Method::DELETE,
        &format!("{REPLICATIONS}/{id}"),
        None,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn every_route_needs_its_privilege() {
    let fixture = ready().await;
    for (user, privilege) in [
        ("reader", "as-replication-read"),
        ("creator", "as-replication-create"),
        ("updater", "as-replication-update"),
        ("deleter", "as-replication-delete"),
        ("receiver", "as-replication-receive"),
    ] {
        create_role(&fixture, user, &[privilege], &[]).await;
        create_user(&fixture, user, "pw-pw-pw-1", &[user]).await;
    }
    let operations: [Operation; 10] = [
        (
            "remote list",
            Method::GET,
            REMOTES.to_owned(),
            None,
            &["reader", "creator", "updater", "deleter"],
        ),
        (
            "remote get",
            Method::GET,
            format!("{REMOTES}/{UNKNOWN}"),
            None,
            &["reader", "creator", "updater", "deleter"],
        ),
        (
            "remote create",
            Method::POST,
            REMOTES.to_owned(),
            Some(json!({})),
            &["creator"],
        ),
        (
            "remote update",
            Method::PUT,
            format!("{REMOTES}/{UNKNOWN}"),
            Some(json!({})),
            &["updater"],
        ),
        (
            "remote delete",
            Method::DELETE,
            format!("{REMOTES}/{UNKNOWN}"),
            None,
            &["deleter"],
        ),
        (
            "replication list",
            Method::GET,
            REPLICATIONS.to_owned(),
            None,
            &["reader", "creator", "updater", "deleter"],
        ),
        (
            "replication get",
            Method::GET,
            format!("{REPLICATIONS}/{UNKNOWN}"),
            None,
            &["reader", "creator", "updater", "deleter"],
        ),
        (
            "replication create",
            Method::POST,
            REPLICATIONS.to_owned(),
            Some(json!({})),
            &["creator"],
        ),
        (
            "replication update",
            Method::PUT,
            format!("{REPLICATIONS}/{UNKNOWN}"),
            Some(json!({})),
            &["updater"],
        ),
        (
            "replication delete",
            Method::DELETE,
            format!("{REPLICATIONS}/{UNKNOWN}"),
            None,
            &["deleter"],
        ),
    ];
    for (label, method, path, body, allowed) in operations {
        let anonymous = api(&fixture, method.clone(), &path, Auth::None, body.clone()).await;
        assert_eq!(
            anonymous.status,
            StatusCode::UNAUTHORIZED,
            "{label} anonymous"
        );
        for user in ["reader", "creator", "updater", "deleter", "receiver"] {
            let reply = api(
                &fixture,
                method.clone(),
                &path,
                Auth::Basic(user, "pw-pw-pw-1"),
                body.clone(),
            )
            .await;
            match allowed.contains(&user) {
                true => assert_ne!(reply.status, StatusCode::FORBIDDEN, "{label} {user}"),
                false => assert_eq!(reply.status, StatusCode::FORBIDDEN, "{label} {user}"),
            }
        }
    }
}

#[tokio::test]
async fn a_replication_also_needs_administrative_access_to_its_source() {
    let fixture = ready().await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    create_role(
        &fixture,
        "plain",
        &["as-replication-create", "as-replication-update"],
        &[],
    )
    .await;
    create_user(&fixture, "plain", "pw-pw-pw-1", &["plain"]).await;
    let plain = Auth::Basic("plain", "pw-pw-pw-1");
    let denied = api(
        &fixture,
        Method::POST,
        REPLICATIONS,
        plain,
        Some(replication("One", &remote_id)),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    create_role(
        &fixture,
        "plain-admin",
        &[
            "as-replication-create",
            "as-replication-update",
            "as-repository-admin-raw-src-read",
        ],
        &[],
    )
    .await;
    create_user(&fixture, "plain-admin", "pw-pw-pw-1", &["plain-admin"]).await;
    let allowed = api(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Auth::Basic("plain-admin", "pw-pw-pw-1"),
        Some(replication("One", &remote_id)),
    )
    .await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
    let id = allowed.json()["id"].as_str().unwrap().to_owned();
    let denied = api(
        &fixture,
        Method::PUT,
        &format!("{REPLICATIONS}/{id}"),
        Auth::Basic("plain", "pw-pw-pw-1"),
        Some(replication("One", &remote_id)),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn changing_or_removing_a_replication_needs_access_to_its_current_source() {
    let fixture = ready().await;
    create_repository(&fixture, "other", "raw").await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    let created = admin(
        &fixture,
        Method::POST,
        REPLICATIONS,
        Some(replication("One", &remote_id)),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    create_role(
        &fixture,
        "other-admin",
        &[
            "as-replication-update",
            "as-replication-delete",
            "as-repository-admin-raw-other-read",
        ],
        &[],
    )
    .await;
    create_user(&fixture, "other-admin", "pw-pw-pw-1", &["other-admin"]).await;
    let auth = || Auth::Basic("other-admin", "pw-pw-pw-1");
    let mut moved = replication("One", &remote_id);
    moved["sourceRepository"] = json!("other");
    let path = format!("{REPLICATIONS}/{id}");
    let update = api(&fixture, Method::PUT, &path, auth(), Some(moved)).await;
    assert_eq!(update.status, StatusCode::FORBIDDEN, "{}", update.text());
    let removal = api(&fixture, Method::DELETE, &path, auth(), None).await;
    assert_eq!(removal.status, StatusCode::FORBIDDEN, "{}", removal.text());
    let still = admin(&fixture, Method::GET, &path, None).await;
    assert_eq!(still.json()["sourceRepository"], "src");
    let unknown = api(
        &fixture,
        Method::DELETE,
        &format!("{REPLICATIONS}/{UNKNOWN}"),
        auth(),
        None,
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_pull_replication_is_validated_and_its_direction_is_fixed() {
    let fixture = ready().await;
    for name in ["locked", "grouped", "other"] {
        create_repository(&fixture, name, "raw").await;
    }
    execute(
        &fixture,
        "UPDATE repository SET attributes = '{\"storage\": {\"writePolicy\": \"DENY\"}}' WHERE name = 'locked'",
    )
    .await;
    execute(
        &fixture,
        "UPDATE repository SET type = 'group' WHERE name = 'grouped'",
    )
    .await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    let mut pull = replication("Pull", &remote_id);
    pull["direction"] = json!("pull");
    let created = admin(&fixture, Method::POST, REPLICATIONS, Some(pull.clone())).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["direction"], "pull");
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let mut push = replication("Push", &remote_id);
    push["sourceRepository"] = json!("other");
    let pushed = admin(&fixture, Method::POST, REPLICATIONS, Some(push)).await;
    assert_eq!(pushed.json()["direction"], "push");
    let listed = admin(&fixture, Method::GET, REPLICATIONS, None)
        .await
        .json();
    let mut directions: Vec<&str> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|found| found["direction"].as_str().unwrap())
        .collect();
    directions.sort();
    assert_eq!(directions, ["pull", "push"]);

    let path = format!("{REPLICATIONS}/{id}");
    let kept = admin(
        &fixture,
        Method::PUT,
        &path,
        Some(replication("Pull", &remote_id)),
    )
    .await;
    assert_eq!(kept.status, StatusCode::OK, "{}", kept.text());
    assert_eq!(kept.json()["direction"], "pull");
    for wrong in ["push", "sideways"] {
        let mut changed = pull.clone();
        changed["direction"] = json!(wrong);
        let refused = admin(&fixture, Method::PUT, &path, Some(changed)).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{wrong}");
        assert_eq!(fields(&refused), ["direction"]);
    }
    let same_key = replication("Again", &remote_id);
    let clash = admin(&fixture, Method::POST, REPLICATIONS, Some(same_key)).await;
    assert_eq!(clash.status, StatusCode::CONFLICT);
    assert!(
        clash.text().contains("both push to and pull from"),
        "{}",
        clash.text()
    );

    for source in ["locked", "grouped"] {
        let mut refused = pull.clone();
        refused["name"] = json!(source);
        refused["sourceRepository"] = json!(source);
        let reply = admin(&fixture, Method::POST, REPLICATIONS, Some(refused)).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{source}: {}",
            reply.text()
        );
        assert_eq!(fields(&reply), ["sourceRepository"], "{source}");
    }
    let mut elsewhere = replication("Locked push", &remote_id);
    elsewhere["sourceRepository"] = json!("locked");
    let allowed = admin(&fixture, Method::POST, REPLICATIONS, Some(elsewhere)).await;
    assert_eq!(allowed.status, StatusCode::CREATED, "{}", allowed.text());
}

#[tokio::test]
async fn a_pull_needs_edit_access_to_the_local_repository_where_a_push_needs_read() {
    let fixture = ready().await;
    create_repository(&fixture, "other", "raw").await;
    let remote_id = make_remote(&fixture, "Mirror").await;
    create_role(
        &fixture,
        "reader-admin",
        &[
            "as-replication-create",
            "as-repository-admin-raw-other-read",
        ],
        &[],
    )
    .await;
    create_role(
        &fixture,
        "editor-admin",
        &[
            "as-replication-create",
            "as-repository-admin-raw-other-read",
            "as-repository-admin-raw-other-edit",
        ],
        &[],
    )
    .await;
    for name in ["reader-admin", "editor-admin"] {
        create_user(&fixture, name, "pw-pw-pw-1", &[name]).await;
    }
    let mut body = replication("Pull", &remote_id);
    body["sourceRepository"] = json!("other");
    body["direction"] = json!("pull");
    let auth = |name| Auth::Basic(name, "pw-pw-pw-1");
    let denied = api(
        &fixture,
        Method::POST,
        REPLICATIONS,
        auth("reader-admin"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN, "{}", denied.text());
    body["direction"] = json!("push");
    let pushed = api(
        &fixture,
        Method::POST,
        REPLICATIONS,
        auth("reader-admin"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(pushed.status, StatusCode::CREATED, "{}", pushed.text());
    body["direction"] = json!("pull");
    body["name"] = json!("Pull 2");
    body["remoteRepository"] = json!("dst2");
    let pulled = api(
        &fixture,
        Method::POST,
        REPLICATIONS,
        auth("editor-admin"),
        Some(body),
    )
    .await;
    assert_eq!(pulled.status, StatusCode::CREATED, "{}", pulled.text());
}
