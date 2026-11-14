mod support;

use std::sync::Arc;
use std::time::Duration;

use astrum_blob::content_of;
use astrum_content::store::{ComponentKey, PutAsset};
use astrum_content::{ids, store};
use astrum_db::entity::audit_event;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, TransactionTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const PROMOTIONS: &str = "/service/rest/v1/promotions";
const TARGETS: &str = "/service/rest/internal/ui/promotion-targets";
const JAR: &[u8] = b"PK\x03\x04\x14\x00\x08\x00\x08\x00jar-content";
const PASSWORD: &str = "password-1234";

async fn world() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_helm::Helm::default()));
    content.register(Arc::new(astrum_format_pypi::PyPi));
    fixture
}

async fn create(fixture: &Fixture, format: &str, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/{format}/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn storage(policy: &str) -> Value {
    json!({"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy})
}

fn raw_repository(name: &str, policy: &str) -> Value {
    json!({"name": name, "online": true, "storage": storage(policy), "raw": {"contentDisposition": "INLINE"}})
}

fn maven_repository(name: &str, policy: &str) -> Value {
    json!({"name": name, "online": true, "storage": storage(policy), "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"}})
}

fn plain_repository(name: &str, policy: &str) -> Value {
    json!({"name": name, "online": true, "storage": storage(policy)})
}

async fn raw_pair(fixture: &Fixture, source_policy: &str, target_policy: &str) {
    create(
        fixture,
        "raw",
        "hosted",
        raw_repository("source", source_policy),
    )
    .await;
    create(
        fixture,
        "raw",
        "hosted",
        raw_repository("target", target_policy),
    )
    .await;
}

async fn put(fixture: &Fixture, repository: &str, path: &str, body: &[u8]) {
    let reply = fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}{path}"),
            Auth::Admin,
            Body::from(body.to_vec()),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert!(reply.status.is_success(), "{path}: {}", reply.text());
}

async fn served(fixture: &Fixture, repository: &str, path: &str) -> Reply {
    fixture
        .call(
            Method::GET,
            &format!("/repository/{repository}{path}"),
            Auth::Admin,
            None,
        )
        .await
}

async fn component_of(fixture: &Fixture, repository: &str, path: &str) -> Value {
    let found = store::find(fixture.state.db().conn(), repository, path)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("{repository}{path} exists"));
    json!({"componentId": ids::encode(repository, found.component_id.unwrap())})
}

async fn exists(fixture: &Fixture, repository: &str, path: &str) -> bool {
    store::find(fixture.state.db().conn(), repository, path)
        .await
        .unwrap()
        .is_some()
}

async fn promote(fixture: &Fixture, auth: Auth<'_>, body: Value) -> Reply {
    fixture
        .call(Method::POST, PROMOTIONS, auth, Some(body))
        .await
}

fn request(source: &str, target: &str, items: Vec<Value>) -> Value {
    json!({"source": source, "target": target, "items": items})
}

fn statuses(reply: &Reply) -> Vec<String> {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["status"].as_str().unwrap().to_owned())
        .collect()
}

async fn grant(fixture: &Fixture, user: &str, grants: &[(&str, &str, &str)]) {
    let privileges: Vec<String> = grants
        .iter()
        .map(|(format, repository, action)| {
            format!("as-repository-view-{format}-{repository}-{action}")
        })
        .collect();
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": user, "name": user, "description": "", "privileges": privileges, "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture.add_user(user, PASSWORD, &[user]).await;
}

async fn audited(fixture: &Fixture) -> Vec<audit_event::Model> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("repository.content"))
        .all(fixture.state.db().conn())
        .await
        .unwrap()
}

async fn allow_loopback(fixture: &Fixture) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn anonymous_callers_are_asked_to_sign_in() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let reply = promote(
        &fixture,
        Auth::None,
        request(
            "source",
            "target",
            vec![json!({"name": "a", "version": ""})],
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    let listed = fixture
        .call(
            Method::GET,
            &format!("{TARGETS}?source=source"),
            Auth::None,
            None,
        )
        .await;
    assert_eq!(listed.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_read_scope_token_cannot_promote() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/internal/access-tokens",
            Auth::Admin,
            Some(json!({"name": "reader", "scope": "READ", "expiresInDays": 1})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let token = created.json()["token"].as_str().unwrap().to_owned();
    let item = component_of(&fixture, "source", "/a.txt").await;
    let reply = promote(
        &fixture,
        Auth::Basic("admin", &token),
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert!(!exists(&fixture, "target", "/a.txt").await);
}

#[tokio::test]
async fn a_caller_who_cannot_see_the_source_gets_not_found() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    grant(&fixture, "blind", &[("raw", "target", "add")]).await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let reply = promote(
        &fixture,
        Auth::Basic("blind", PASSWORD),
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let missing = promote(
        &fixture,
        Auth::Basic("blind", PASSWORD),
        request(
            "nowhere",
            "target",
            vec![json!({"name": "a", "version": ""})],
        ),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.body, reply.body);
}

#[tokio::test]
async fn read_on_the_source_and_add_on_the_target_are_enough() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    grant(
        &fixture,
        "courier",
        &[("raw", "source", "read"), ("raw", "target", "add")],
    )
    .await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let reply = promote(
        &fixture,
        Auth::Basic("courier", PASSWORD),
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&reply), ["copied"]);
    assert_eq!(served(&fixture, "target", "/a.txt").await.body, b"alpha");
    assert!(exists(&fixture, "source", "/a.txt").await);
}

#[tokio::test]
async fn without_add_on_the_target_the_item_is_forbidden_and_nothing_is_written() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    grant(
        &fixture,
        "peeker",
        &[("raw", "source", "read"), ("raw", "target", "read")],
    )
    .await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let reply = promote(
        &fixture,
        Auth::Basic("peeker", PASSWORD),
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&reply), ["forbidden"]);
    assert!(
        reply.json()["results"][0]["error"]
            .as_str()
            .unwrap()
            .contains("add access")
    );
    assert!(!exists(&fixture, "target", "/a.txt").await);
    let holds = fixture
        .state
        .db()
        .conn()
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT count(*) AS n FROM blob_hold",
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(holds.try_get::<i64>("", "n").unwrap(), 0);
}

#[tokio::test]
async fn replacing_content_needs_edit_on_the_target() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"new").await;
    put(&fixture, "target", "/a.txt", b"old").await;
    grant(
        &fixture,
        "adder",
        &[("raw", "source", "read"), ("raw", "target", "add")],
    )
    .await;
    grant(
        &fixture,
        "editor",
        &[
            ("raw", "source", "read"),
            ("raw", "target", "add"),
            ("raw", "target", "edit"),
        ],
    )
    .await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["conflict"] = json!("replace");
    let refused = promote(&fixture, Auth::Basic("adder", PASSWORD), body.clone()).await;
    assert_eq!(statuses(&refused), ["forbidden"]);
    assert!(
        refused.json()["results"][0]["error"]
            .as_str()
            .unwrap()
            .contains("edit access")
    );
    assert_eq!(served(&fixture, "target", "/a.txt").await.body, b"old");
    let replaced = promote(&fixture, Auth::Basic("editor", PASSWORD), body).await;
    assert_eq!(statuses(&replaced), ["copied"]);
    assert_eq!(served(&fixture, "target", "/a.txt").await.body, b"new");
}

#[tokio::test]
async fn a_move_needs_delete_on_the_source() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    grant(
        &fixture,
        "keeper",
        &[("raw", "source", "read"), ("raw", "target", "add")],
    )
    .await;
    grant(
        &fixture,
        "mover",
        &[
            ("raw", "source", "read"),
            ("raw", "source", "delete"),
            ("raw", "target", "add"),
        ],
    )
    .await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["move"] = json!(true);
    let refused = promote(&fixture, Auth::Basic("keeper", PASSWORD), body.clone()).await;
    assert_eq!(statuses(&refused), ["forbidden"]);
    assert!(
        refused.json()["results"][0]["error"]
            .as_str()
            .unwrap()
            .contains("delete access")
    );
    assert!(exists(&fixture, "source", "/a.txt").await);
    assert!(!exists(&fixture, "target", "/a.txt").await);
    let moved = promote(&fixture, Auth::Basic("mover", PASSWORD), body).await;
    assert_eq!(statuses(&moved), ["copied"]);
    assert_eq!(moved.json()["results"][0]["sourceKept"], false);
    assert!(!exists(&fixture, "source", "/a.txt").await);
    assert_eq!(served(&fixture, "target", "/a.txt").await.body, b"alpha");
}

async fn deploy_maven(fixture: &Fixture, repository: &str, version: &str) {
    let directory = format!("/org/example/lib/{version}");
    put(
        fixture,
        repository,
        &format!("{directory}/lib-{version}.jar"),
        JAR,
    )
    .await;
    let pom = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>org.example</groupId>\n  <artifactId>lib</artifactId>\n  <version>{version}</version>\n  <packaging>jar</packaging>\n</project>\n"
    );
    put(
        fixture,
        repository,
        &format!("{directory}/lib-{version}.pom"),
        pom.as_bytes(),
    )
    .await;
}

#[tokio::test]
async fn a_content_selector_that_covers_part_of_a_component_forbids_it_without_naming_the_rest() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "maven", "hosted", maven_repository(name, "ALLOW")).await;
    }
    deploy_maven(&fixture, "source", "1.0").await;
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "jar-only", "description": "", "expression": "format == \"maven2\" and path == \"/org/example/lib/1.0/lib-1.0.jar\""}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "jar-only-read", "description": "", "actions": ["READ"], "format": "maven2", "repository": "source", "contentSelector": "jar-only"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "jar-reader", "name": "jar-reader", "description": "", "privileges": ["jar-only-read", "as-repository-view-maven2-target-add"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("jar-user", PASSWORD, &["jar-reader"])
        .await;
    let item = component_of(&fixture, "source", "/org/example/lib/1.0/lib-1.0.jar").await;
    let reply = promote(
        &fixture,
        Auth::Basic("jar-user", PASSWORD),
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&reply), ["forbidden"]);
    let text = reply.text();
    assert!(!text.contains(".pom"), "{text}");
    assert!(!exists(&fixture, "target", "/org/example/lib/1.0/lib-1.0.jar").await);
}

#[tokio::test]
async fn request_bodies_are_validated_strictly() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let named = || json!({"name": "a", "version": ""});
    let many: Vec<Value> = (0..101).map(|_| named()).collect();
    let cases: Vec<(&str, Value)> = vec![
        ("an empty body object", json!({})),
        ("no items", request("source", "target", vec![])),
        ("101 items", request("source", "target", many)),
        (
            "an unknown field",
            json!({"source": "source", "target": "target", "items": [named()], "distTags": true}),
        ),
        (
            "an unknown item field",
            request(
                "source",
                "target",
                vec![json!({"name": "a", "version": "", "x": 1})],
            ),
        ),
        (
            "an empty item",
            request("source", "target", vec![json!({})]),
        ),
        (
            "a name without a version",
            request("source", "target", vec![json!({"name": "a"})]),
        ),
        (
            "two forms in one item",
            request(
                "source",
                "target",
                vec![json!({"componentId": "abc", "name": "a", "version": ""})],
            ),
        ),
        (
            "an image item for a raw repository",
            request(
                "source",
                "target",
                vec![json!({"image": "app", "tag": "1"})],
            ),
        ),
        (
            "control characters in a name",
            request(
                "source",
                "target",
                vec![json!({"name": "a\u{1}", "version": ""})],
            ),
        ),
        (
            "a right-to-left override in a name",
            request(
                "source",
                "target",
                vec![json!({"name": "a\u{202e}b", "version": ""})],
            ),
        ),
        (
            "a zero-width space in a version",
            request(
                "source",
                "target",
                vec![json!({"name": "a", "version": "1\u{200b}"})],
            ),
        ),
        (
            "control characters in the source",
            request("sour\u{7}ce", "target", vec![named()]),
        ),
        (
            "a string for move",
            json!({"source": "source", "target": "target", "items": [named()], "move": "yes"}),
        ),
        (
            "an unknown conflict option",
            json!({"source": "source", "target": "target", "items": [named()], "conflict": "overwrite"}),
        ),
        (
            "a number for the name",
            request("source", "target", vec![json!({"name": 7, "version": ""})]),
        ),
    ];
    for (label, body) in cases {
        let reply = promote(&fixture, Auth::Admin, body).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{label}: {}",
            reply.text()
        );
    }
    let fine = promote(
        &fixture,
        Auth::Admin,
        request("source", "target", vec![named()]),
    )
    .await;
    assert_eq!(statuses(&fine), ["notFound"]);
}

#[tokio::test]
async fn repositories_must_be_hosted_online_distinct_and_of_one_format() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    create(
        &fixture,
        "raw",
        "group",
        json!({"name": "bundle", "online": true, "storage": {"blobStoreName": "default", "strictContentTypeValidation": true}, "group": {"memberNames": ["source"]}}),
    )
    .await;
    allow_loopback(&fixture).await;
    create(
        &fixture,
        "raw",
        "proxy",
        json!({
            "name": "mirror", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "proxy": {"remoteUrl": "http://127.0.0.1:9/", "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": true},
            "raw": {"contentDisposition": "ATTACHMENT"}
        }),
    )
    .await;
    let mut offline = raw_repository("offline", "ALLOW");
    offline["online"] = json!(false);
    create(&fixture, "raw", "hosted", offline).await;
    create(
        &fixture,
        "maven",
        "hosted",
        maven_repository("jars", "ALLOW"),
    )
    .await;
    let item = || vec![json!({"name": "a", "version": ""})];
    for (label, source, target) in [
        ("the same repository", "source", "source"),
        ("a group target", "source", "bundle"),
        ("a group source", "bundle", "target"),
        ("a proxy target", "source", "mirror"),
        ("a proxy source", "mirror", "target"),
        ("an offline target", "source", "offline"),
        ("an offline source", "offline", "target"),
        ("another format", "source", "jars"),
        ("an unknown target", "source", "nowhere"),
    ] {
        let reply = promote(&fixture, Auth::Admin, request(source, target, item())).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{label}: {}",
            reply.text()
        );
    }
    let unknown = promote(&fixture, Auth::Admin, request("nowhere", "target", item())).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn raw_components_are_copied_by_id_and_by_coordinates_and_items_fail_alone() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/docs/a.txt", b"alpha").await;
    put(&fixture, "source", "/b.txt", b"beta").await;
    let by_id = component_of(&fixture, "source", "/docs/a.txt").await;
    let by_coordinates = json!({"namespace": "/", "name": "/b.txt", "version": ""});
    let missing = json!({"name": "/nothing.txt", "version": ""});
    let reply = promote(
        &fixture,
        Auth::Admin,
        request(
            "source",
            "target",
            vec![by_id.clone(), missing, by_coordinates],
        ),
    )
    .await;
    assert_eq!(statuses(&reply), ["copied", "notFound", "copied"]);
    let results = reply.json();
    assert_eq!(results["dryRun"], false);
    let first = &results["results"][0];
    assert_eq!(first["item"], by_id);
    assert_eq!(first["bytesCopied"], 5);
    assert_eq!(first["bytesReused"], 0);
    assert_eq!(first["assetCount"], 1);
    assert_eq!(first["assets"][0]["path"], "/docs/a.txt");
    assert_eq!(first["assets"][0]["status"], "new");
    assert!(first["componentId"].is_string());
    assert_eq!(
        served(&fixture, "target", "/docs/a.txt").await.body,
        b"alpha"
    );
    assert_eq!(served(&fixture, "target", "/b.txt").await.body, b"beta");
}

#[tokio::test]
async fn maven_artifacts_are_copied_and_the_target_metadata_lists_the_version() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "maven", "hosted", maven_repository(name, "ALLOW")).await;
    }
    deploy_maven(&fixture, "source", "1.0").await;
    let item = component_of(&fixture, "source", "/org/example/lib/1.0/lib-1.0.jar").await;
    let reply = promote(
        &fixture,
        Auth::Admin,
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&reply), ["copied"]);
    assert_eq!(reply.json()["results"][0]["assetCount"], 2);
    let jar = served(&fixture, "target", "/org/example/lib/1.0/lib-1.0.jar").await;
    assert_eq!(jar.body, JAR);
    let metadata = served(&fixture, "target", "/org/example/lib/maven-metadata.xml").await;
    assert!(
        metadata.text().contains("<version>1.0</version>"),
        "{}",
        metadata.text()
    );
    assert!(!exists(&fixture, "target", "/org/example/lib/1.0/lib-1.0.jar.sha1").await);
}

#[tokio::test]
async fn maven_targets_that_forbid_redeploying_or_writing_refuse() {
    let fixture = world().await;
    create(
        &fixture,
        "maven",
        "hosted",
        maven_repository("source", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "maven",
        "hosted",
        maven_repository("once", "ALLOW_ONCE"),
    )
    .await;
    create(
        &fixture,
        "maven",
        "hosted",
        maven_repository("frozen", "DENY"),
    )
    .await;
    deploy_maven(&fixture, "source", "1.0").await;
    let jar = "/org/example/lib/1.0/lib-1.0.jar";
    let item = component_of(&fixture, "source", jar).await;
    let denied = promote(
        &fixture,
        Auth::Admin,
        request("source", "frozen", vec![item.clone()]),
    )
    .await;
    assert_eq!(statuses(&denied), ["failed"]);
    assert!(denied.text().contains("read-only"), "{}", denied.text());
    assert!(!exists(&fixture, "frozen", jar).await);
    let first = promote(
        &fixture,
        Auth::Admin,
        request("source", "once", vec![item.clone()]),
    )
    .await;
    assert_eq!(statuses(&first), ["copied"]);
    put(
        &fixture,
        "source",
        jar,
        b"PK\x03\x04\x14\x00\x08\x00\x08\x00other",
    )
    .await;
    let mut again = request("source", "once", vec![item.clone()]);
    for conflict in ["fail", "replace"] {
        again["conflict"] = json!(conflict);
        let reply = promote(&fixture, Auth::Admin, again.clone()).await;
        assert_eq!(statuses(&reply), ["conflict"], "{conflict}");
        assert_eq!(reply.json()["results"][0]["retryable"], false, "{conflict}");
    }
    assert_eq!(served(&fixture, "once", jar).await.body, JAR);
}

#[tokio::test]
async fn helm_charts_are_copied_with_their_provenance_and_the_target_index_lists_them() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "helm", "hosted", plain_repository(name, "ALLOW")).await;
    }
    let chart = helm_chart("web", "1.2.3");
    put(&fixture, "source", "/web-1.2.3.tgz", &chart).await;
    let signed = helm_provenance("web", "1.2.3", &chart);
    put(&fixture, "source", "/web-1.2.3.tgz.prov", signed.as_bytes()).await;
    let item = component_of(&fixture, "source", "/web-1.2.3.tgz").await;
    let reply = promote(
        &fixture,
        Auth::Admin,
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&reply), ["copied"]);
    let index = served(&fixture, "target", "/index.yaml").await;
    assert!(index.text().contains("1.2.3"), "{}", index.text());
    assert_eq!(
        served(&fixture, "target", "/web-1.2.3.tgz.prov").await.body,
        signed.as_bytes()
    );
}

fn helm_chart(name: &str, version: &str) -> Vec<u8> {
    let chart = format!(
        "apiVersion: v2\nname: {name}\ndescription: The {name} test chart\ntype: application\nversion: {version}\nappVersion: \"1.16.0\"\n"
    );
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in [
        (format!("{name}/Chart.yaml"), chart.into_bytes()),
        (format!("{name}/values.yaml"), b"replicaCount: 1\n".to_vec()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path, data.as_slice())
            .unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

fn helm_provenance(name: &str, version: &str, data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA512\n\napiVersion: v2\ndescription: The {name} test chart\nname: {name}\nversion: {version}\n\n...\nfiles:\n  {name}-{version}.tgz: sha256:{}\n-----BEGIN PGP SIGNATURE-----\n\nwsBcBAEBCgAQBQJqzTW/\n-----END PGP SIGNATURE-----\n",
        hex::encode(Sha256::digest(data))
    )
}

async fn three_files(fixture: &Fixture) -> Vec<Value> {
    put(fixture, "source", "/new.txt", b"fresh").await;
    put(fixture, "source", "/same.txt", b"same").await;
    put(fixture, "source", "/clash.txt", b"source side").await;
    put(fixture, "target", "/same.txt", b"same").await;
    put(fixture, "target", "/clash.txt", b"target side").await;
    vec![
        component_of(fixture, "source", "/new.txt").await,
        component_of(fixture, "source", "/same.txt").await,
        component_of(fixture, "source", "/clash.txt").await,
    ]
}

#[tokio::test]
async fn a_dry_run_writes_nothing_and_matches_the_real_run() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let items = three_files(&fixture).await;
    for (conflict, expected) in [
        ("fail", ["copied", "unchanged", "conflict"]),
        ("skip", ["copied", "unchanged", "skipped"]),
    ] {
        let mut body = request("source", "target", items.clone());
        body["conflict"] = json!(conflict);
        body["dryRun"] = json!(true);
        let planned = promote(&fixture, Auth::Admin, body.clone()).await;
        assert_eq!(statuses(&planned), expected, "{conflict}");
        assert_eq!(planned.json()["dryRun"], true);
        assert!(!exists(&fixture, "target", "/new.txt").await);
        assert_eq!(
            served(&fixture, "target", "/clash.txt").await.body,
            b"target side"
        );
        body["dryRun"] = json!(false);
        let real = promote(&fixture, Auth::Admin, body).await;
        assert_eq!(statuses(&real), expected, "{conflict}");
        let (dry, wet) = (planned.json(), real.json());
        for index in 0..3 {
            for field in ["bytesCopied", "bytesReused", "assetCount"] {
                assert_eq!(dry["results"][index][field], wet["results"][index][field]);
            }
        }
        store::delete(
            &support::repository_ctx(&fixture, "target").await,
            "/new.txt",
        )
        .await
        .unwrap();
    }
    assert!(
        audited(&fixture)
            .await
            .iter()
            .all(|row| row.kind == "promoted")
    );
}

#[tokio::test]
async fn conflicts_follow_the_option_and_the_target_write_policy() {
    let allow = world().await;
    raw_pair(&allow, "ALLOW", "ALLOW").await;
    let items = three_files(&allow).await;
    let mut body = request("source", "target", items);
    body["conflict"] = json!("replace");
    let replaced = promote(&allow, Auth::Admin, body).await;
    assert_eq!(statuses(&replaced), ["copied", "unchanged", "copied"]);
    assert_eq!(
        served(&allow, "target", "/clash.txt").await.body,
        b"source side"
    );
    let once = world().await;
    raw_pair(&once, "ALLOW", "ALLOW_ONCE").await;
    let items = three_files(&once).await;
    for conflict in ["fail", "replace"] {
        let mut body = request("source", "target", items.clone());
        body["conflict"] = json!(conflict);
        let reply = promote(&once, Auth::Admin, body).await;
        assert_eq!(
            statuses(&reply),
            [
                if conflict == "fail" {
                    "copied"
                } else {
                    "unchanged"
                },
                "unchanged",
                "conflict"
            ],
            "{conflict}"
        );
        assert_eq!(
            served(&once, "target", "/clash.txt").await.body,
            b"target side"
        );
    }
    let frozen = world().await;
    raw_pair(&frozen, "ALLOW", "DENY").await;
    put(&frozen, "source", "/new.txt", b"fresh").await;
    let item = component_of(&frozen, "source", "/new.txt").await;
    let denied = promote(
        &frozen,
        Auth::Admin,
        request("source", "target", vec![item]),
    )
    .await;
    assert_eq!(statuses(&denied), ["failed"]);
    assert!(!exists(&frozen, "target", "/new.txt").await);
}

#[tokio::test]
async fn a_rerun_finds_everything_unchanged() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let body = request("source", "target", vec![item]);
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, body.clone()).await),
        ["copied"]
    );
    let again = promote(&fixture, Auth::Admin, body).await;
    assert_eq!(statuses(&again), ["unchanged"]);
    let result = &again.json()["results"][0];
    assert_eq!(result["bytesCopied"], 0);
    assert_eq!(result["bytesReused"], 5);
    assert_eq!(audited(&fixture).await.len(), 1);
}

#[tokio::test]
async fn a_move_deletes_the_source_of_copied_and_unchanged_items_and_keeps_the_rest() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let items = three_files(&fixture).await;
    let mut body = request("source", "target", items);
    body["move"] = json!(true);
    let reply = promote(&fixture, Auth::Admin, body).await;
    assert_eq!(statuses(&reply), ["copied", "unchanged", "conflict"]);
    let results = reply.json();
    assert_eq!(results["results"][0]["sourceKept"], false);
    assert_eq!(results["results"][1]["sourceKept"], false);
    assert_eq!(results["results"][2]["sourceKept"], true);
    assert!(!exists(&fixture, "source", "/new.txt").await);
    assert!(!exists(&fixture, "source", "/same.txt").await);
    assert!(exists(&fixture, "source", "/clash.txt").await);
    assert!(exists(&fixture, "target", "/same.txt").await);
    let rows = audited(&fixture).await;
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.kind == "moved"));
}

#[tokio::test]
async fn a_skipped_item_keeps_its_source_in_a_move() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let items = three_files(&fixture).await;
    let mut body = request("source", "target", items);
    body["move"] = json!(true);
    body["conflict"] = json!("skip");
    let reply = promote(&fixture, Auth::Admin, body).await;
    assert_eq!(statuses(&reply), ["copied", "unchanged", "skipped"]);
    assert_eq!(reply.json()["results"][2]["sourceKept"], true);
    assert!(exists(&fixture, "source", "/clash.txt").await);
}

#[tokio::test]
async fn a_move_without_delete_on_the_source_leaves_an_unchanged_component_alone() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    put(&fixture, "target", "/a.txt", b"alpha").await;
    grant(
        &fixture,
        "keeper",
        &[("raw", "source", "read"), ("raw", "target", "add")],
    )
    .await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["move"] = json!(true);
    let reply = promote(&fixture, Auth::Basic("keeper", PASSWORD), body).await;
    assert_eq!(statuses(&reply), ["forbidden"]);
    assert!(exists(&fixture, "source", "/a.txt").await);
}

#[tokio::test]
async fn formats_without_transfer_are_unsupported() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "pypi", "hosted", plain_repository(name, "ALLOW")).await;
    }
    let reply = promote(
        &fixture,
        Auth::Admin,
        request(
            "source",
            "target",
            vec![json!({"name": "pkg", "version": "1"})],
        ),
    )
    .await;
    assert_eq!(statuses(&reply), ["unsupported"]);
    let listed = fixture
        .call(
            Method::GET,
            &format!("{TARGETS}?source=source"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(listed.json(), json!([]));
}

#[tokio::test]
async fn a_promotion_is_audited_with_its_route_and_without_secrets() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["dryRun"] = json!(true);
    promote(&fixture, Auth::Admin, body.clone()).await;
    assert!(audited(&fixture).await.is_empty());
    body["dryRun"] = json!(false);
    promote(&fixture, Auth::Admin, body).await;
    let rows = audited(&fixture).await;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.kind, "promoted");
    assert_eq!(row.context, "source -> target");
    assert_eq!(row.initiator.split('/').next(), Some("admin"));
    let attributes = row.attributes.as_object().unwrap();
    let mut keys: Vec<&str> = attributes.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "assets",
            "bytes",
            "component",
            "dryRun",
            "source",
            "target",
            "via"
        ]
    );
    assert_eq!(attributes["via"], "rest");
    assert_eq!(attributes["dryRun"], false);
    assert_eq!(attributes["bytes"], 5);
    assert!(!row.attributes.to_string().contains(&fixture.admin_password));
}

#[tokio::test]
async fn a_dropped_request_does_not_cut_the_transfer() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "maven", "hosted", maven_repository(name, "ALLOW")).await;
    }
    deploy_maven(&fixture, "source", "1.0").await;
    let item = component_of(&fixture, "source", "/org/example/lib/1.0/lib-1.0.jar").await;
    let lock = fixture.state.db().conn().begin().await.unwrap();
    lock.execute_unprepared("LOCK TABLE asset IN ACCESS EXCLUSIVE MODE")
        .await
        .unwrap();
    let abandoned = tokio::time::timeout(
        Duration::from_millis(500),
        promote(
            &fixture,
            Auth::Admin,
            request("source", "target", vec![item]),
        ),
    )
    .await;
    assert!(
        abandoned.is_err(),
        "the request was still waiting for the lock"
    );
    lock.rollback().await.unwrap();
    let mut seen = 0;
    for _ in 0..100 {
        let count = [
            "/org/example/lib/1.0/lib-1.0.jar",
            "/org/example/lib/1.0/lib-1.0.pom",
        ]
        .into_iter()
        .map(|path| store::find(fixture.state.db().conn(), "target", path))
        .collect::<Vec<_>>();
        let mut found = 0;
        for lookup in count {
            found += usize::from(lookup.await.unwrap().is_some());
        }
        assert_ne!(found, 1, "a half-written component");
        seen = found;
        if seen == 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(seen, 2, "the transfer finished after the client left");
    assert_eq!(audited(&fixture).await.len(), 1);
}

#[tokio::test]
async fn promotion_targets_are_the_hosted_repositories_the_caller_may_add_to() {
    let fixture = world().await;
    for name in ["source", "open", "reading", "hidden"] {
        create(&fixture, "raw", "hosted", raw_repository(name, "ALLOW")).await;
    }
    let mut offline = raw_repository("offline", "ALLOW");
    offline["online"] = json!(false);
    create(&fixture, "raw", "hosted", offline).await;
    create(
        &fixture,
        "maven",
        "hosted",
        maven_repository("jars", "ALLOW"),
    )
    .await;
    create(
        &fixture,
        "raw",
        "group",
        json!({"name": "bundle", "online": true, "storage": {"blobStoreName": "default", "strictContentTypeValidation": true}, "group": {"memberNames": ["source"]}}),
    )
    .await;
    grant(
        &fixture,
        "lister",
        &[
            ("raw", "source", "read"),
            ("raw", "open", "add"),
            ("raw", "reading", "read"),
            ("raw", "offline", "add"),
            ("maven2", "jars", "add"),
        ],
    )
    .await;
    let names = |reply: &Reply| -> Vec<String> {
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        reply
            .json()
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["name"].as_str().unwrap().to_owned())
            .collect()
    };
    let path = format!("{TARGETS}?source=source");
    let own = fixture
        .call(Method::GET, &path, Auth::Basic("lister", PASSWORD), None)
        .await;
    assert_eq!(names(&own), ["open"]);
    let admin = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(names(&admin), ["hidden", "open", "reading"]);
    let unknown = fixture
        .call(
            Method::GET,
            &format!("{TARGETS}?source=nowhere"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    let unseen = fixture
        .call(
            Method::GET,
            &format!("{TARGETS}?source=hidden"),
            Auth::Basic("lister", PASSWORD),
            None,
        )
        .await;
    assert_eq!(unseen.status, StatusCode::NOT_FOUND);
}

async fn upload(fixture: &Fixture, repository: &str, path: &str, body: &[u8]) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            &format!("/repository/{repository}{path}"),
            Auth::Admin,
            Body::from(body.to_vec()),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn hold_component_of(
    fixture: &Fixture,
    repository: &str,
    path: &str,
) -> sea_orm::DatabaseTransaction {
    let found = store::find(fixture.state.db().conn(), repository, path)
        .await
        .unwrap()
        .unwrap();
    support::hold_row(
        fixture,
        &format!(
            "SELECT id FROM component WHERE id = {} FOR UPDATE",
            found.component_id.unwrap()
        ),
    )
    .await
}

#[tokio::test]
async fn a_move_keeps_the_source_when_an_asset_arrives_while_it_waits_for_the_lock() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "maven", "hosted", maven_repository(name, "ALLOW")).await;
    }
    deploy_maven(&fixture, "source", "1.0").await;
    let jar = "/org/example/lib/1.0/lib-1.0.jar";
    let sources = "/org/example/lib/1.0/lib-1.0-sources.jar";
    let item = component_of(&fixture, "source", jar).await;
    let mut body = request("source", "target", vec![item]);
    body["move"] = json!(true);
    let hold = hold_component_of(&fixture, "source", jar).await;
    let (uploaded, moved, ()) = tokio::join!(
        upload(&fixture, "source", sources, JAR),
        async {
            support::wait_for_waiters(&fixture, 1).await;
            promote(&fixture, Auth::Admin, body.clone()).await
        },
        async {
            support::wait_for_waiters(&fixture, 2).await;
            hold.rollback().await.unwrap();
        },
    );
    assert!(uploaded.status.is_success(), "{}", uploaded.text());
    assert_eq!(statuses(&moved), ["copied"]);
    let result = &moved.json()["results"][0];
    assert_eq!(result["sourceKept"], true);
    assert!(
        result["error"]
            .as_str()
            .unwrap()
            .contains("changed in the source while the move ran"),
        "{result}"
    );
    assert!(exists(&fixture, "source", sources).await);
    assert!(exists(&fixture, "source", jar).await);
    assert!(!exists(&fixture, "target", sources).await);
    let again = promote(&fixture, Auth::Admin, body).await;
    assert_eq!(statuses(&again), ["copied"]);
    assert_eq!(again.json()["results"][0]["sourceKept"], false);
    assert!(!exists(&fixture, "source", sources).await);
    assert!(!exists(&fixture, "source", jar).await);
    assert_eq!(served(&fixture, "target", sources).await.body, JAR);
    let kinds: Vec<String> = audited(&fixture)
        .await
        .into_iter()
        .map(|row| row.kind)
        .collect();
    assert_eq!(kinds, ["promoted", "moved"]);
}

const GATE: i64 = 7_700_221;

#[tokio::test]
async fn an_asset_that_appears_in_the_target_after_the_plan_is_never_overwritten() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/clash.txt", b"source side").await;
    grant(
        &fixture,
        "adder",
        &[("raw", "source", "read"), ("raw", "target", "add")],
    )
    .await;
    let conn = fixture.state.db().conn();
    conn.execute_unprepared(&format!(
        "CREATE FUNCTION promotion_gate() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN \
         PERFORM pg_advisory_xact_lock({GATE}); RETURN NEW; END $$; \
         CREATE TRIGGER promotion_gate BEFORE INSERT ON blob_hold \
         FOR EACH ROW EXECUTE FUNCTION promotion_gate()"
    ))
    .await
    .unwrap();
    let gate = conn.begin().await.unwrap();
    gate.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({GATE})"))
        .await
        .unwrap();
    let item = component_of(&fixture, "source", "/clash.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["conflict"] = json!("replace");
    let (reply, ()) = tokio::join!(
        promote(&fixture, Auth::Basic("adder", PASSWORD), body),
        async {
            support::wait_for_waiters(&fixture, 1).await;
            put(&fixture, "target", "/clash.txt", b"target side").await;
            gate.commit().await.unwrap();
        },
    );
    assert_eq!(statuses(&reply), ["conflict"]);
    assert_eq!(reply.json()["results"][0]["retryable"], true);
    assert_eq!(
        served(&fixture, "target", "/clash.txt").await.body,
        b"target side"
    );
    conn.execute_unprepared(
        "DROP TRIGGER promotion_gate ON blob_hold; DROP FUNCTION promotion_gate()",
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn conflict_options_leave_a_different_asset_in_the_target_untouched() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    let items = three_files(&fixture).await;
    for conflict in ["fail", "skip"] {
        let mut body = request("source", "target", items[2..].to_vec());
        body["conflict"] = json!(conflict);
        let reply = promote(&fixture, Auth::Admin, body).await;
        let expected = match conflict {
            "fail" => "conflict",
            _ => "skipped",
        };
        assert_eq!(statuses(&reply), [expected], "{conflict}");
        assert_eq!(
            served(&fixture, "target", "/clash.txt").await.body,
            b"target side",
            "{conflict}"
        );
    }
    let once = world().await;
    raw_pair(&once, "ALLOW", "ALLOW_ONCE").await;
    let items = three_files(&once).await;
    let mut body = request("source", "target", items[2..].to_vec());
    body["conflict"] = json!("replace");
    let reply = promote(&once, Auth::Admin, body).await;
    assert_eq!(statuses(&reply), ["conflict"]);
    assert_eq!(reply.json()["results"][0]["retryable"], false);
    assert_eq!(
        served(&once, "target", "/clash.txt").await.body,
        b"target side"
    );
}

#[tokio::test]
async fn a_move_that_found_and_copied_nothing_keeps_the_source() {
    let fixture = world().await;
    for name in ["source", "target"] {
        create(&fixture, "maven", "hosted", maven_repository(name, "ALLOW")).await;
    }
    let ctx = support::repository_ctx(&fixture, "source").await;
    let checksum = "/org/example/lib/1.0/lib-1.0.jar.sha1";
    let key = ComponentKey {
        namespace: "org.example".to_owned(),
        name: "lib".to_owned(),
        version: "1.0".to_owned(),
        ..ComponentKey::default()
    };
    store::put(
        &ctx,
        PutAsset::hosted(&ctx, checksum).component(key),
        content_of(&b"da39a3ee5e6b4b0d3255bfef95601890afd80709"[..]),
    )
    .await
    .unwrap();
    let item = json!({"namespace": "org.example", "name": "lib", "version": "1.0"});
    let mut body = request("source", "target", vec![item]);
    body["move"] = json!(true);
    let reply = promote(&fixture, Auth::Admin, body).await;
    assert_eq!(statuses(&reply), ["unchanged"]);
    let result = &reply.json()["results"][0];
    assert_eq!(result["assetCount"], 0);
    assert_eq!(result["sourceKept"], true);
    assert!(result["error"].as_str().unwrap().contains("nothing"));
    assert!(exists(&fixture, "source", checksum).await);
    assert!(audited(&fixture).await.is_empty());
}

#[tokio::test]
async fn an_exhausted_budget_fails_every_item_without_deleting_anything() {
    let fixture = world().await;
    fixture.state.set_promotion_budget(Duration::ZERO);
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    put(&fixture, "source", "/b.txt", b"beta").await;
    let items = vec![
        component_of(&fixture, "source", "/a.txt").await,
        component_of(&fixture, "source", "/b.txt").await,
    ];
    for (dry_run, kept) in [(false, Some(true)), (true, None)] {
        let mut body = request("source", "target", items.clone());
        body["move"] = json!(true);
        body["dryRun"] = json!(dry_run);
        let reply = promote(&fixture, Auth::Admin, body).await;
        assert_eq!(statuses(&reply), ["failed", "failed"], "{dry_run}");
        for result in reply.json()["results"].as_array().unwrap() {
            assert_eq!(result["retryable"], true);
            assert!(result["error"].as_str().unwrap().starts_with("timeout"));
            assert_eq!(result.get("sourceKept").cloned(), kept.map(Value::from));
        }
    }
    for path in ["/a.txt", "/b.txt"] {
        assert!(exists(&fixture, "source", path).await);
        assert!(!exists(&fixture, "target", path).await);
    }
    assert!(audited(&fixture).await.is_empty());
}

#[tokio::test]
async fn items_that_start_after_the_budget_ran_out_fail_while_the_running_one_finishes() {
    let fixture = world().await;
    fixture.state.set_promotion_budget(Duration::from_secs(1));
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    put(&fixture, "source", "/b.txt", b"beta").await;
    let items = vec![
        component_of(&fixture, "source", "/a.txt").await,
        component_of(&fixture, "source", "/b.txt").await,
    ];
    let mut body = request("source", "target", items);
    body["move"] = json!(true);
    let hold = hold_component_of(&fixture, "source", "/a.txt").await;
    let (reply, ()) = tokio::join!(promote(&fixture, Auth::Admin, body), async {
        support::wait_for_waiters(&fixture, 1).await;
        tokio::time::sleep(Duration::from_millis(1300)).await;
        hold.rollback().await.unwrap();
    });
    assert_eq!(statuses(&reply), ["copied", "failed"]);
    let results = reply.json();
    assert_eq!(results["results"][0]["sourceKept"], false);
    assert_eq!(results["results"][1]["retryable"], true);
    assert_eq!(results["results"][1]["sourceKept"], true);
    assert!(!exists(&fixture, "source", "/a.txt").await);
    assert!(exists(&fixture, "source", "/b.txt").await);
    assert!(!exists(&fixture, "target", "/b.txt").await);
}
