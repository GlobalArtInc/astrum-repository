mod support;
mod ui_support;

use astrum_db::entity::audit_event;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::{create, ready, user_with};

const POLICIES: &str = "/service/rest/v1/security/policies";
const EXCEPTIONS: &str = "/service/rest/v1/security/exceptions";
const KEYS: &str = "/service/rest/v1/security/keys";
const FEEDS: &str = "/service/rest/v1/security/advisories/feeds";
const IMPORT: &str = "/service/rest/v1/security/advisories/import?ecosystem=npm";

const P256: &str = "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEjlbYB6ZMqNQZi+NyFytE68YWZvvD\njMVTs80I9GOxS4MNCnsT0S5/H7Y/0u0pII+w1TvGjJLD1qCKvJb5FhDpLw==\n-----END PUBLIC KEY-----\n";
const ED25519: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAJE3dInZfUxV36BT4nDCDzkPU0/gziiEq+fMBP78UtCo=\n-----END PUBLIC KEY-----\n";
const RSA_1024: &str = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgIz1akZhyY8JRIOJUIA9Zw0jT\n1q2KTkbrHjqLeoXYJtnGaoCzc5SDEgDdzCa+81k0A7R4Ff1bUrCwz5lnZvRMnEF6\nzF84xRxVl0vyhrKnZ9IVZIOSaV00Dc5yl9Xaoym+6JjlgNsp/c6lA3DxoOl8lVTX\nR4UqticodfNpJCjYawIDAQAB\n-----END PUBLIC KEY-----\n";
const P521: &str = "-----BEGIN PUBLIC KEY-----\nMIGbMBAGByqGSM49AgEGBSuBBAAjA4GGAAQBBtWove4ZX4R42HovrAtG2SZHnhPU\nZkckR+5WCm8mbATRb2xPK4GDsoWPSYPD0agQqnXaNssUzPcJTe4byWrFfXcBmxQP\nkNbek663U324EViVD+zkWyn0AgOPreRthNAaHeU3vniasyKnRYq9+YW1P571S4uK\nwBNqzxUA8FSjZHttTLU=\n-----END PUBLIC KEY-----\n";
const P384: &str = "-----BEGIN PUBLIC KEY-----\nMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEaJzCiv8oWh0vCWTJ45x8e5qCET81e29y\nnTPaKgPZavUkdKBfkAIbZhnnzRAW03dQ0Vmw41WIpUXtVGukF6X7CA0GnBIhSErY\nXHUF9WUZ3YrC1VeYPKRKy17oQVy+NokL\n-----END PUBLIC KEY-----\n";
const PRIVATE: &str =
    "-----BEGIN PRIVATE KEY-----\nc2VjcmV0LW1hdGVyaWFs\n-----END PRIVATE KEY-----\n";

const PASSWORD: &str = "password-1";
const PRINCIPALS: [(&str, &str); 8] = [
    ("nobody", ""),
    ("reader", "as-policies-read"),
    ("creator", "as-policies-create"),
    ("updater", "as-policies-update"),
    ("deleter", "as-policies-delete"),
    ("xreader", "as-policyexceptions-read"),
    ("xcreator", "as-policyexceptions-create"),
    ("xdeleter", "as-policyexceptions-delete"),
];

async fn principals(fixture: &Fixture) {
    for (user, privilege) in PRINCIPALS {
        match privilege.is_empty() {
            true => fixture.add_user(user, PASSWORD, &[]).await,
            false => user_with(fixture, user, &[privilege]).await,
        }
    }
}

async fn admin(fixture: &Fixture, method: Method, path: &str, body: Value) -> Reply {
    fixture.call(method, path, Auth::Admin, Some(body)).await
}

async fn made(fixture: &Fixture, path: &str, body: Value) -> Value {
    let reply = admin(fixture, Method::POST, path, body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()
}

async fn matrix(
    fixture: &Fixture,
    method: Method,
    path: &str,
    body: Option<Value>,
    allowed: &[&str],
    ok: StatusCode,
) {
    let label = format!("{method} {path}");
    let anonymous = fixture
        .call(method.clone(), path, Auth::None, body.clone())
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED, "{label}");
    for (user, _) in PRINCIPALS {
        if allowed.contains(&user) {
            continue;
        }
        let auth = Auth::Basic(user, PASSWORD);
        let reply = fixture.call(method.clone(), path, auth, body.clone()).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{label} as {user}");
    }
    for user in allowed {
        let auth = Auth::Basic(user, PASSWORD);
        let reply = fixture.call(method.clone(), path, auth, body.clone()).await;
        assert_eq!(reply.status, ok, "{label} as {user}: {}", reply.text());
    }
}

async fn events(fixture: &Fixture, domain: &str) -> Vec<audit_event::Model> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq(domain))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn bad(fixture: &Fixture, path: &str, body: Value, needle: &str) {
    let reply = admin(fixture, Method::POST, path, body.clone()).await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "{body}: {}",
        reply.text()
    );
    assert!(reply.text().contains(needle), "{body}: {}", reply.text());
}

async fn import(fixture: &Fixture, user: &str) -> Reply {
    fixture
        .send_body(
            Method::POST,
            IMPORT,
            Auth::Basic(user, PASSWORD),
            Body::empty(),
            &[("content-type", "application/zip")],
        )
        .await
}

#[tokio::test]
async fn every_route_wants_its_own_privilege() {
    let fixture = Fixture::new().await;
    principals(&fixture).await;
    made(&fixture, POLICIES, json!({"name": "base"})).await;
    let held = made(
        &fixture,
        EXCEPTIONS,
        json!({"rule": "license", "license": "MIT", "reason": "approved"}),
    )
    .await;
    made(&fixture, KEYS, json!({"name": "k1", "publicKey": P256})).await;
    let id = held["id"].as_str().unwrap();
    let reads = ["reader", "creator", "updater", "deleter"];
    let exception_reads = ["xreader", "xcreator", "xdeleter"];
    let ok = StatusCode::OK;

    matrix(&fixture, Method::GET, POLICIES, None, &reads, ok).await;
    matrix(
        &fixture,
        Method::GET,
        &format!("{POLICIES}/base"),
        None,
        &reads,
        ok,
    )
    .await;
    let body = Some(json!({"name": "made"}));
    matrix(
        &fixture,
        Method::POST,
        POLICIES,
        body,
        &["creator"],
        StatusCode::CREATED,
    )
    .await;
    let body = Some(json!({"notes": "changed"}));
    matrix(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/base"),
        body,
        &["updater"],
        ok,
    )
    .await;
    matrix(
        &fixture,
        Method::DELETE,
        &format!("{POLICIES}/made"),
        None,
        &["deleter"],
        StatusCode::NO_CONTENT,
    )
    .await;

    matrix(
        &fixture,
        Method::GET,
        EXCEPTIONS,
        None,
        &exception_reads,
        ok,
    )
    .await;
    let body = Some(json!({"rule": "license", "license": "ISC", "reason": "approved"}));
    matrix(
        &fixture,
        Method::POST,
        EXCEPTIONS,
        body,
        &["xcreator"],
        StatusCode::CREATED,
    )
    .await;
    matrix(
        &fixture,
        Method::DELETE,
        &format!("{EXCEPTIONS}/{id}"),
        None,
        &["xdeleter"],
        StatusCode::NO_CONTENT,
    )
    .await;

    matrix(&fixture, Method::GET, KEYS, None, &reads, ok).await;
    let body = Some(json!({"name": "k2", "publicKey": ED25519}));
    matrix(
        &fixture,
        Method::POST,
        KEYS,
        body,
        &["creator"],
        StatusCode::CREATED,
    )
    .await;
    matrix(
        &fixture,
        Method::DELETE,
        &format!("{KEYS}/k1"),
        None,
        &["deleter"],
        StatusCode::NO_CONTENT,
    )
    .await;
}

#[tokio::test]
async fn a_policy_is_created_changed_and_audited_under_a_new_revision() {
    let fixture = Fixture::new().await;
    let revision = || async {
        astrum_db::revision::current(fixture.state.db().conn(), astrum_db::revision::POLICIES)
            .await
            .unwrap()
    };
    let created = made(
        &fixture,
        POLICIES,
        json!({
            "name": "strict", "notes": "blocks the worst",
            "vulnerabilities": {"mode": "block", "severity": "high", "unscanned": "block"},
            "licenses": {"mode": "report", "list": "deny", "ids": ["GPL-3.0-only"]},
        }),
    )
    .await;
    assert_eq!(created["vulnerabilities"]["severity"], "high");
    assert_eq!(created["vulnerabilities"]["malicious"], "block");
    assert_eq!(created["signatures"]["mode"], "off");
    assert_eq!(created["revision"], 1);
    assert_eq!(revision().await, 1);

    let changed = admin(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/strict"),
        json!({"name": "strict", "notes": "softer", "vulnerabilities": {"mode": "report"}}),
    )
    .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.text());
    let changed = changed.json();
    assert_eq!(changed["notes"], "softer");
    assert_eq!(changed["vulnerabilities"]["mode"], "report");
    assert_eq!(changed["vulnerabilities"]["severity"], Value::Null);
    assert_eq!(changed["licenses"]["ids"], json!(["GPL-3.0-only"]));
    assert_eq!(changed["revision"], 2);
    assert_eq!(revision().await, 2);

    let renamed = admin(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/strict"),
        json!({"name": "lenient"}),
    )
    .await;
    assert_eq!(renamed.status, StatusCode::BAD_REQUEST);
    assert!(renamed.text().contains("not renamed"), "{}", renamed.text());
    let missing = admin(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/ghost"),
        json!({}),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let listed = fixture.call(Method::GET, POLICIES, Auth::Admin, None).await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);

    let rows = events(&fixture, "contentPolicy").await;
    let kinds: Vec<&str> = rows.iter().map(|row| row.kind.as_str()).collect();
    assert_eq!(kinds, ["created", "updated"]);
    assert_eq!(rows[0].context, "strict");
    assert_eq!(rows[1].attributes["vulnerabilities"]["mode"], "report");
    assert!(rows.iter().all(|row| row.attributes.get("via").is_none()));
    assert!(rows.iter().all(|row| row.initiator.starts_with("admin")));
}

#[tokio::test]
async fn policy_input_is_checked_strictly() {
    let fixture = Fixture::new().await;
    let long_name = "n".repeat(65);
    let long_notes = "x".repeat(1001);
    for (body, needle) in [
        (
            json!({"name": "a", "vulnerabilities": {"mdoe": "block"}}),
            "mdoe",
        ),
        (
            json!({"name": "a", "licenses": {"mode": "block", "idz": []}}),
            "idz",
        ),
        (json!({"name": "a", "signatures": {"key": ["k"]}}), "key"),
        (json!({"name": "a", "extra": 1}), "extra"),
        (
            json!({"name": "a", "vulnerabilities": {"mode": "blocc"}}),
            "blocc",
        ),
        (json!({"name": ""}), "must not be empty"),
        (json!({}), "must not be empty"),
        (json!({"name": long_name}), "at most 64"),
        (json!({"name": "_hidden"}), "letters, digits"),
        (json!({"name": "a", "notes": long_notes}), "at most 1000"),
        (
            json!({"name": "a", "licenses": {"mode": "report", "ids": ["MIT License"]}}),
            "licenses.ids",
        ),
        (
            json!({"name": "a", "vulnerabilities": {"severity": "critical"}}),
            "vulnerabilities.mode",
        ),
        (
            json!({"name": "a", "licenses": {"ids": ["MIT"]}}),
            "licenses.mode",
        ),
        (
            json!({"name": "a", "signatures": {"keys": []}}),
            "signatures.mode",
        ),
        (
            json!({"name": "a", "signatures": {"mode": "block"}}),
            "signatures.keys",
        ),
        (
            json!({"name": "a", "licenses": {"mode": "block", "list": "allow", "ids": []}}),
            "licenses.ids",
        ),
        (
            json!({"name": "a", "signatures": {"mode": "block", "keys": ["ghost"]}}),
            "no trusted key named ghost",
        ),
    ] {
        bad(&fixture, POLICIES, body, needle).await;
    }
    let listed = fixture.call(Method::GET, POLICIES, Auth::Admin, None).await;
    assert_eq!(listed.json(), json!([]));

    made(&fixture, POLICIES, json!({"name": "strict"})).await;
    for name in ["strict", "STRICT"] {
        let again = admin(&fixture, Method::POST, POLICIES, json!({"name": name})).await;
        assert_eq!(
            again.status,
            StatusCode::CONFLICT,
            "{name}: {}",
            again.text()
        );
    }
    assert!(events(&fixture, "contentPolicy").await.len() == 1);
}

#[tokio::test]
async fn a_policy_that_repositories_use_is_not_deleted_and_the_error_names_them() {
    let fixture = ready().await;
    made(&fixture, POLICIES, json!({"name": "strict"})).await;
    for name in ["docs-b", "docs-a"] {
        create(
            &fixture,
            "raw",
            "hosted",
            json!({
                "name": name, "online": true, "policy": {"name": "strict"},
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            }),
        )
        .await;
    }
    let mut scoped = Vec::new();
    for license in ["MIT", "BSD-3-Clause"] {
        let held = made(
            &fixture,
            EXCEPTIONS,
            json!({"policy": "strict", "rule": "license", "license": license, "reason": "own"}),
        )
        .await;
        scoped.push(held["id"].as_str().unwrap().to_owned());
    }
    made(
        &fixture,
        EXCEPTIONS,
        json!({"rule": "license", "license": "ISC", "reason": "everywhere"}),
    )
    .await;
    let path = format!("{POLICIES}/strict");
    let refused = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.text());
    let message = refused.json()["message"].as_str().unwrap().to_owned();
    assert!(message.ends_with("docs-a, docs-b"), "{message}");
    assert!(
        events(&fixture, "contentPolicy")
            .await
            .iter()
            .all(|row| row.kind == "created")
    );

    for name in ["docs-a", "docs-b"] {
        let removed = fixture
            .call(
                Method::DELETE,
                &format!("/service/rest/v1/repositories/{name}"),
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    }
    let deleted = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT, "{}", deleted.text());
    let again = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let held = fixture
        .call(Method::GET, EXCEPTIONS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(held.as_array().unwrap().len(), 1);
    assert_eq!(held[0]["policy"], Value::Null);
    let rows = events(&fixture, "contentPolicy").await;
    assert_eq!(rows.last().unwrap().kind, "deleted");
    let attributes = &rows.last().unwrap().attributes;
    assert_eq!(attributes["exceptions"], 2);
    let mut named: Vec<String> = attributes["exceptionIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect();
    named.sort();
    scoped.sort();
    assert_eq!(named, scoped);
}

#[tokio::test]
async fn an_exception_needs_a_target_a_reason_and_a_future_end() {
    let fixture = Fixture::new().await;
    made(&fixture, POLICIES, json!({"name": "strict"})).await;
    let license = json!({"rule": "license", "license": "MIT", "reason": "approved"});
    let with = |field: &str, value: Value| {
        let mut body = license.clone();
        body[field] = value;
        body
    };
    for (body, needle) in [
        (
            json!({"rule": "license", "reason": "approved"}),
            "advisory, license, name or digest",
        ),
        (json!({"rule": "license", "license": "MIT"}), "reason"),
        (with("reason", json!("   ")), "reason"),
        (
            with("expiresAt", json!("2020-01-01T00:00:00Z")),
            "expiresAt",
        ),
        (with("expiresAt", json!("tomorrow")), "expiresAt"),
        (
            with("policy", json!("ghost")),
            "no content policy named ghost",
        ),
        (with("licence", json!("MIT")), "licence"),
        (with("rule", json!("all")), "all"),
        (json!({"license": "MIT", "reason": "approved"}), "rule"),
        (with("digest", json!("sha256:XYZ")), "digest"),
        (with("format", json!("nonesuch")), "nonesuch"),
        (with("name", json!("  ")), "name"),
    ] {
        bad(&fixture, EXCEPTIONS, body, needle).await;
    }
    let listed = fixture
        .call(Method::GET, EXCEPTIONS, Auth::Admin, None)
        .await;
    assert_eq!(listed.json(), json!([]));

    let digest = format!("sha256:{}", "ab".repeat(32));
    let held = made(
        &fixture,
        EXCEPTIONS,
        json!({
            "policy": "strict", "rule": "vulnerability", "advisory": "GHSA-aaaa-bbbb-cccc",
            "format": "npm", "name": "left-pad", "digest": digest,
            "reason": " accepted until the fix ships ", "expiresAt": "2999-01-01T00:00:00Z"
        }),
    )
    .await;
    assert_eq!(held["reason"], "accepted until the fix ships");
    assert_eq!(held["createdBy"], "admin");
    assert_eq!(held["expiresAt"], "2999-01-01T00:00:00Z");
    assert_eq!(held["policy"], "strict");
    let id = held["id"].as_str().unwrap();
    let rows = events(&fixture, "policyException").await;
    assert_eq!(rows[0].kind, "created");
    assert_eq!(rows[0].context, id);
    assert!(rows[0].attributes.get("via").is_none());

    let path = format!("{EXCEPTIONS}/{id}");
    let removed = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    for path in [path, format!("{EXCEPTIONS}/not-an-id")] {
        let reply = fixture.call(Method::DELETE, &path, Auth::Admin, None).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
    }
    let rows = events(&fixture, "policyException").await;
    let kinds: Vec<&str> = rows.iter().map(|row| row.kind.as_str()).collect();
    assert_eq!(kinds, ["created", "deleted"]);
}

#[tokio::test]
async fn a_trusted_key_is_a_public_key_of_a_supported_algorithm() {
    let fixture = Fixture::new().await;
    let created = made(
        &fixture,
        KEYS,
        json!({"name": "release", "publicKey": P256, "notes": "ci"}),
    )
    .await;
    assert_eq!(created["algorithm"], "ecdsa-p256");
    assert_eq!(created["fingerprint"].as_str().unwrap().len(), 64);
    assert_eq!(created["publicKey"], P256);
    let ed = made(&fixture, KEYS, json!({"name": "ed", "publicKey": ED25519})).await;
    assert_eq!(ed["algorithm"], "ed25519");

    for (body, needle) in [
        (json!({"name": "k", "publicKey": PRIVATE}), "private key"),
        (json!({"name": "k", "publicKey": RSA_1024}), "2048"),
        (json!({"name": "k", "publicKey": P521}), "not supported"),
        (json!({"name": "k", "publicKey": "just text"}), "publicKey"),
        (json!({"name": "k"}), "publicKey"),
        (json!({"name": "", "publicKey": P256}), "name"),
        (
            json!({"name": "bad name", "publicKey": P256}),
            "letters, digits",
        ),
        (
            json!({"name": "k", "publicKey": P256, "algorithm": "rsa"}),
            "algorithm",
        ),
    ] {
        bad(&fixture, KEYS, body, needle).await;
    }
    let echoed = admin(
        &fixture,
        Method::POST,
        KEYS,
        json!({"name": "k", "publicKey": PRIVATE}),
    )
    .await;
    assert!(!echoed.text().contains("c2VjcmV0"), "{}", echoed.text());

    let clashes = [
        ("release", ED25519),
        ("second", P256),
        ("RELEASE", P384),
        ("release", P384),
    ];
    for (name, public_key) in clashes {
        let body = json!({"name": name, "publicKey": public_key});
        let again = admin(&fixture, Method::POST, KEYS, body).await;
        assert_eq!(
            again.status,
            StatusCode::CONFLICT,
            "{name}: {}",
            again.text()
        );
    }
    made(&fixture, KEYS, json!({"name": "third", "publicKey": P384})).await;
    let listed = fixture
        .call(Method::GET, KEYS, Auth::Admin, None)
        .await
        .json();
    assert_eq!(listed.as_array().unwrap().len(), 3);
    let rows = events(&fixture, "trustedKey").await;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].attributes["fingerprint"], created["fingerprint"]);
    assert!(!rows[0].attributes.to_string().contains("BEGIN"));
}

#[tokio::test]
async fn a_key_that_a_policy_trusts_is_not_deleted() {
    let fixture = Fixture::new().await;
    made(
        &fixture,
        KEYS,
        json!({"name": "release", "publicKey": P256}),
    )
    .await;
    made(
        &fixture,
        KEYS,
        json!({"name": "spare", "publicKey": ED25519}),
    )
    .await;
    made(
        &fixture,
        POLICIES,
        json!({"name": "signed", "signatures": {"mode": "block", "keys": ["release"]}}),
    )
    .await;
    let twice = admin(
        &fixture,
        Method::POST,
        POLICIES,
        json!({"name": "twice", "signatures": {"mode": "report", "keys": ["release", "release"]}}),
    )
    .await;
    assert_eq!(twice.status, StatusCode::BAD_REQUEST, "{}", twice.text());
    let refused = fixture
        .call(
            Method::DELETE,
            &format!("{KEYS}/release"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.text());
    let message = refused.json()["message"].as_str().unwrap().to_owned();
    assert!(message.ends_with(": signed"), "{message}");
    let spare = fixture
        .call(Method::DELETE, &format!("{KEYS}/spare"), Auth::Admin, None)
        .await;
    assert_eq!(spare.status, StatusCode::NO_CONTENT);
    let freed = admin(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/signed"),
        json!({"signatures": {"mode": "off"}}),
    )
    .await;
    assert_eq!(freed.status, StatusCode::OK, "{}", freed.text());
    let released = fixture
        .call(
            Method::DELETE,
            &format!("{KEYS}/release"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(released.status, StatusCode::NO_CONTENT);
    let gone = fixture
        .call(
            Method::DELETE,
            &format!("{KEYS}/release"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_advisory_routes_honour_the_new_policy_privileges() {
    let fixture = Fixture::new().await;
    principals(&fixture).await;
    for user in ["reader", "creator", "updater", "deleter"] {
        let feeds = fixture
            .call(Method::GET, FEEDS, Auth::Basic(user, PASSWORD), None)
            .await;
        assert_eq!(feeds.status, StatusCode::OK, "{user}");
    }
    for user in ["reader", "creator", "deleter", "xcreator"] {
        assert_eq!(
            import(&fixture, user).await.status,
            StatusCode::FORBIDDEN,
            "{user}"
        );
    }
    let allowed = import(&fixture, "updater").await;
    assert_eq!(
        allowed.status,
        StatusCode::BAD_REQUEST,
        "{}",
        allowed.text()
    );
    assert!(
        allowed.text().contains("body is empty"),
        "{}",
        allowed.text()
    );
}

#[tokio::test]
async fn a_rule_block_changes_only_with_its_mode_and_never_blocks_everything() {
    let fixture = Fixture::new().await;
    made(
        &fixture,
        POLICIES,
        json!({"name": "strict", "vulnerabilities": {"mode": "block", "severity": "high"}}),
    )
    .await;
    let path = format!("{POLICIES}/strict");
    for (body, needle) in [
        (
            json!({"vulnerabilities": {"severity": "critical"}}),
            "vulnerabilities.mode",
        ),
        (json!({"licenses": {"ids": ["MIT"]}}), "licenses.mode"),
        (json!({"signatures": {"keys": []}}), "signatures.mode"),
        (json!({"vulnerabilities": {"mdoe": "off"}}), "mdoe"),
        (
            json!({"signatures": {"mode": "block", "keys": []}}),
            "signatures.keys",
        ),
        (
            json!({"licenses": {"mode": "block", "list": "allow"}}),
            "licenses.ids",
        ),
    ] {
        let reply = admin(&fixture, Method::PUT, &path, body.clone()).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{body}: {}",
            reply.text()
        );
        assert!(reply.text().contains(needle), "{body}: {}", reply.text());
    }
    let kept = fixture
        .call(Method::GET, &path, Auth::Admin, None)
        .await
        .json();
    assert_eq!(kept["vulnerabilities"]["mode"], "block");
    assert_eq!(kept["vulnerabilities"]["severity"], "high");
    assert_eq!(kept["revision"], 1);

    for mode in ["off", "report"] {
        let body = json!({
            "name": mode,
            "signatures": {"mode": mode, "keys": []},
            "licenses": {"mode": mode, "list": "allow", "ids": []},
        });
        let reply = admin(&fixture, Method::POST, POLICIES, body).await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{mode}: {}",
            reply.text()
        );
    }
}

#[tokio::test]
async fn a_put_that_changes_nothing_leaves_the_revision_and_the_audit_alone() {
    let fixture = Fixture::new().await;
    made(
        &fixture,
        POLICIES,
        json!({"name": "strict", "vulnerabilities": {"mode": "block", "severity": "high"}}),
    )
    .await;
    let path = format!("{POLICIES}/strict");
    let revision = || async {
        astrum_db::revision::current(fixture.state.db().conn(), astrum_db::revision::POLICIES)
            .await
            .unwrap()
    };
    let body = json!({"notes": "changed"});
    let first = admin(&fixture, Method::PUT, &path, body.clone()).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert_eq!(first.json()["revision"], 2);
    for body in [
        body,
        json!({}),
        json!({"vulnerabilities": {"mode": "block", "severity": "high"}}),
    ] {
        let again = admin(&fixture, Method::PUT, &path, body.clone()).await;
        assert_eq!(again.status, StatusCode::OK, "{body}: {}", again.text());
        assert_eq!(again.json()["revision"], 2, "{body}");
    }
    assert_eq!(revision().await, 2);
    assert_eq!(events(&fixture, "contentPolicy").await.len(), 2);
}

#[tokio::test]
async fn an_exception_target_must_fit_its_rule() {
    let fixture = Fixture::new().await;
    let digest = format!("sha256:{}", "ab".repeat(32));
    for (body, needle) in [
        (
            json!({"rule": "license", "name": "left-pad", "reason": "r"}),
            "license exception needs a license",
        ),
        (
            json!({"rule": "license", "license": "MIT", "advisory": "CVE-2024-0001", "reason": "r"}),
            "cannot target an advisory",
        ),
        (
            json!({"rule": "vulnerability", "license": "MIT", "reason": "r"}),
            "vulnerability exception cannot target a license",
        ),
        (
            json!({"rule": "signature", "advisory": "CVE-2024-0001", "digest": digest, "reason": "r"}),
            "signature exception cannot target an advisory",
        ),
        (
            json!({"rule": "signature", "license": "MIT", "digest": digest, "reason": "r"}),
            "signature exception cannot target a license",
        ),
        (
            json!({"rule": "cooldown", "name": "x", "reason": "r"}),
            "cooldown",
        ),
    ] {
        bad(&fixture, EXCEPTIONS, body, needle).await;
    }
    for body in [
        json!({"rule": "license", "license": "MIT", "name": "left-pad", "reason": "r"}),
        json!({"rule": "vulnerability", "advisory": "CVE-2024-0001", "reason": "r"}),
        json!({"rule": "signature", "digest": digest, "reason": "r"}),
    ] {
        made(&fixture, EXCEPTIONS, body).await;
    }
}

#[tokio::test]
async fn a_policy_or_exception_change_reaches_the_policy_cache_at_once() {
    let fixture = Fixture::new().await;
    let cache = fixture.state.policy_cache();
    let conn = fixture.state.db().conn();
    made(
        &fixture,
        POLICIES,
        json!({"name": "strict", "vulnerabilities": {"mode": "block"}}),
    )
    .await;
    let first = cache.get(conn).await.unwrap();
    assert_eq!(first.policies["strict"].revision, 1);

    let changed = admin(
        &fixture,
        Method::PUT,
        &format!("{POLICIES}/strict"),
        json!({"vulnerabilities": {"mode": "report"}}),
    )
    .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.text());
    let second = cache.get(conn).await.unwrap();
    assert_eq!(second.policies["strict"].revision, 2);

    let exception = made(
        &fixture,
        EXCEPTIONS,
        json!({"rule": "vulnerability", "advisory": "CVE-1", "reason": "accepted"}),
    )
    .await;
    assert_eq!(cache.get(conn).await.unwrap().exceptions.len(), 1);

    let id = exception["id"].as_str().unwrap();
    let removed = admin(
        &fixture,
        Method::DELETE,
        &format!("{EXCEPTIONS}/{id}"),
        json!(null),
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert!(cache.get(conn).await.unwrap().exceptions.is_empty());
}

#[tokio::test]
async fn creating_and_deleting_a_policy_reach_the_policy_cache_at_once() {
    let fixture = Fixture::new().await;
    let cache = fixture.state.policy_cache();
    let conn = fixture.state.db().conn();
    assert!(cache.get(conn).await.unwrap().policies.is_empty());

    made(
        &fixture,
        POLICIES,
        json!({"name": "strict", "vulnerabilities": {"mode": "block"}}),
    )
    .await;
    assert!(
        cache
            .get(conn)
            .await
            .unwrap()
            .policies
            .contains_key("strict")
    );

    let removed = admin(
        &fixture,
        Method::DELETE,
        &format!("{POLICIES}/strict"),
        json!(null),
    )
    .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert!(cache.get(conn).await.unwrap().policies.is_empty());
}
