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
    content.register(Arc::new(astrum_format_npm::Npm::default()));
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
            json!({"source": "source", "target": "target", "items": [named()], "force": true}),
        ),
        (
            "a string for distTags",
            json!({"source": "source", "target": "target", "items": [named()], "distTags": "yes"}),
        ),
        (
            "a number for distTags",
            json!({"source": "source", "target": "target", "items": [named()], "distTags": 1}),
        ),
        (
            "a null for distTags",
            json!({"source": "source", "target": "target", "items": [named()], "distTags": null}),
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

fn npm_repository(name: &str, policy: &str) -> Value {
    plain_repository(name, policy)
}

async fn npm_repositories(fixture: &Fixture, names: &[(&str, &str)]) {
    for (name, policy) in names {
        create(fixture, "npm", "hosted", npm_repository(name, policy)).await;
    }
}

fn npm_tarball(name: &str, version: &str, flavor: &str) -> Vec<u8> {
    let mut data = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x03".to_vec();
    data.extend_from_slice(format!("package {name} {version} {flavor}").as_bytes());
    data
}

async fn npm_publish(
    fixture: &Fixture,
    repository: &str,
    name: &str,
    version: &str,
    tag: &str,
    flavor: &str,
) {
    use base64::Engine;
    let data = npm_tarball(name, version, flavor);
    let bare = name.rsplit('/').next().unwrap();
    let body = json!({
        "_id": name,
        "name": name,
        "dist-tags": {tag: version},
        "versions": {version: {
            "name": name,
            "version": version,
            "description": format!("{bare} does things"),
            "dist": {"tarball": format!("http://client.example/{name}/-/{bare}-{version}.tgz")}
        }},
        "_attachments": {format!("{name}-{version}.tgz"): {
            "content_type": "application/octet-stream",
            "data": base64::engine::general_purpose::STANDARD.encode(&data),
            "length": data.len()
        }}
    });
    let reply = fixture
        .call(
            Method::PUT,
            &format!("/repository/{repository}/{}", name.replace('/', "%2f")),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

async fn npm_doc(fixture: &Fixture, repository: &str, name: &str) -> Reply {
    served(fixture, repository, &format!("/{name}")).await
}

fn npm_item(name: &str, version: &str) -> Value {
    match name.strip_prefix('@').and_then(|name| name.split_once('/')) {
        Some((scope, bare)) => json!({"namespace": scope, "name": bare, "version": version}),
        None => json!({"name": name, "version": version}),
    }
}

async fn npm_promote(fixture: &Fixture, target: &str, name: &str, version: &str) -> Vec<String> {
    let reply = promote(
        fixture,
        Auth::Admin,
        request("source", target, vec![npm_item(name, version)]),
    )
    .await;
    statuses(&reply)
}

async fn stored_blobs(fixture: &Fixture) -> usize {
    fixture
        .state
        .content()
        .blob_stores()
        .get("default")
        .await
        .unwrap()
        .list(None, 1000)
        .await
        .unwrap()
        .len()
}

fn npm_tarball_path(name: &str, version: &str) -> String {
    format!(
        "/{name}/-/{}-{version}.tgz",
        name.rsplit('/').next().unwrap()
    )
}

#[tokio::test]
async fn npm_versions_are_promoted_with_their_packument_entry_into_an_empty_target() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    for name in ["plain", "@team/tool"] {
        npm_publish(&fixture, "source", name, "1.0.0", "latest", "a").await;
        assert_eq!(
            npm_promote(&fixture, "target", name, "1.0.0").await,
            ["copied"]
        );
        let from = npm_doc(&fixture, "source", name).await.json();
        let reply = npm_doc(&fixture, "target", name).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
        let doc = reply.json();
        let (before, after) = (&from["versions"]["1.0.0"], &doc["versions"]["1.0.0"]);
        for field in ["shasum", "integrity"] {
            assert_eq!(
                after["dist"][field], before["dist"][field],
                "{name} {field}"
            );
        }
        assert_eq!(
            after["dist"]["tarball"],
            format!(
                "http://localhost/repository/target{}",
                npm_tarball_path(name, "1.0.0")
            )
        );
        assert_eq!(after["description"], before["description"]);
        assert_eq!(doc["dist-tags"], json!({"latest": "1.0.0"}));
        assert_eq!(doc["time"]["1.0.0"], from["time"]["1.0.0"]);
        assert_eq!(doc["time"]["created"], from["time"]["1.0.0"]);
        assert!(doc["time"]["modified"].is_string());
        assert_eq!(
            served(&fixture, "target", &npm_tarball_path(name, "1.0.0"))
                .await
                .body,
            npm_tarball(name, "1.0.0", "a")
        );
    }
}

#[tokio::test]
async fn a_promoted_version_joins_a_populated_packument_without_touching_the_others() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    let promoted = "1.5.0-rc.1+build.5";
    npm_publish(&fixture, "source", "plain", "0.1.0", "latest", "a").await;
    npm_publish(&fixture, "source", "plain", promoted, "beta", "a").await;
    for version in ["1.0.0", "2.0.0"] {
        npm_publish(&fixture, "target", "plain", version, "latest", "a").await;
    }
    let before = npm_doc(&fixture, "target", "plain").await.json();
    assert_eq!(
        npm_promote(&fixture, "target", "plain", promoted).await,
        ["copied"]
    );
    let doc = npm_doc(&fixture, "target", "plain").await.json();
    for version in ["1.0.0", "2.0.0"] {
        assert_eq!(doc["versions"][version], before["versions"][version]);
        assert_eq!(doc["time"][version], before["time"][version]);
    }
    assert!(doc["versions"][promoted].is_object());
    assert_eq!(
        doc["dist-tags"],
        json!({"latest": "2.0.0", "beta": promoted})
    );
    assert_eq!(doc["time"]["created"], before["time"]["created"]);
    assert_ne!(doc["time"]["modified"], before["time"]["modified"]);
    assert_eq!(doc["_rev"], "3");
    assert_eq!(
        served(&fixture, "target", &npm_tarball_path("plain", promoted))
            .await
            .body,
        npm_tarball("plain", promoted, "a")
    );
}

#[tokio::test]
async fn dist_tags_move_only_when_the_source_lists_them_and_the_option_is_on() {
    let fixture = world().await;
    npm_repositories(
        &fixture,
        &[
            ("source", "ALLOW"),
            ("tags-on", "ALLOW"),
            ("tags-off", "ALLOW"),
            ("tags-unlisted", "ALLOW"),
            ("tags-high", "ALLOW"),
            ("tags-high-off", "ALLOW"),
        ],
    )
    .await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    npm_publish(&fixture, "source", "plain", "1.1.0", "beta", "a").await;
    assert_eq!(
        npm_promote(&fixture, "tags-on", "plain", "1.1.0").await,
        ["copied"]
    );
    let doc = npm_doc(&fixture, "tags-on", "plain").await.json();
    assert_eq!(doc["dist-tags"]["beta"], "1.1.0");
    let off = promote(
        &fixture,
        Auth::Admin,
        json!({"source": "source", "target": "tags-off", "items": [npm_item("plain", "1.1.0")], "distTags": false}),
    )
    .await;
    assert_eq!(statuses(&off), ["copied"]);
    let doc = npm_doc(&fixture, "tags-off", "plain").await.json();
    assert_eq!(doc["dist-tags"], json!({"latest": "1.1.0"}));
    npm_publish(&fixture, "tags-unlisted", "plain", "0.9.0", "beta", "a").await;
    assert_eq!(
        npm_promote(&fixture, "tags-unlisted", "plain", "1.0.0").await,
        ["copied"]
    );
    let doc = npm_doc(&fixture, "tags-unlisted", "plain").await.json();
    assert_eq!(doc["dist-tags"]["beta"], "0.9.0");
    assert_eq!(doc["dist-tags"]["latest"], "1.0.0");
    let stable = fixture
        .call(
            Method::PUT,
            "/repository/source/-/package/plain/dist-tags/stable",
            Auth::Admin,
            Some(json!("1.0.0")),
        )
        .await;
    assert_eq!(stable.status, StatusCode::OK, "{}", stable.text());
    for (target, on, moved) in [("tags-high", true, true), ("tags-high-off", false, false)] {
        npm_publish(&fixture, target, "plain", "3.0.0", "latest", "a").await;
        let reply = promote(
            &fixture,
            Auth::Admin,
            json!({"source": "source", "target": target, "items": [npm_item("plain", "1.0.0")], "distTags": on}),
        )
        .await;
        assert_eq!(statuses(&reply), ["copied"]);
        let doc = npm_doc(&fixture, target, "plain").await.json();
        assert_eq!(doc["dist-tags"]["latest"], "3.0.0", "{target}");
        assert_eq!(doc["dist-tags"].get("stable").is_some(), moved, "{target}");
        assert!(doc["versions"]["1.0.0"].is_object());
    }
}

async fn trusted(fixture: &Fixture, name: &str) -> astrum_content::Ctx {
    let ctx = support::repository_ctx(fixture, name).await;
    let repository = ctx.repository.clone();
    ctx.member(repository)
}

async fn npm_receive(
    fixture: &Fixture,
    target: &str,
    bundle: &astrum_content::transfer::Bundle,
) -> Result<astrum_content::transfer::Outcome, astrum_content::ContentError> {
    use astrum_content::transfer::{Conflict, LocalSource, PROMOTED, Terms};
    let format = fixture.state.content().format("npm").unwrap();
    format
        .transfer()
        .unwrap()
        .receive(
            &trusted(fixture, target).await,
            bundle,
            &LocalSource::new(trusted(fixture, "source").await),
            Terms::new(Conflict::Fail, PROMOTED),
        )
        .await
}

#[tokio::test]
async fn a_tarball_that_does_not_match_its_manifest_is_refused_and_nothing_is_written() {
    use astrum_content::ContentError;
    let fixture = world().await;
    npm_repositories(
        &fixture,
        &[("source", "ALLOW"), ("target", "ALLOW"), ("kept", "ALLOW")],
    )
    .await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    npm_publish(&fixture, "kept", "plain", "0.5.0", "latest", "a").await;
    let found = store::find(
        fixture.state.db().conn(),
        "source",
        &npm_tarball_path("plain", "1.0.0"),
    )
    .await
    .unwrap()
    .unwrap();
    let component = store::component_by_id(fixture.state.db().conn(), found.component_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    let format = fixture.state.content().format("npm").unwrap();
    let genuine = format
        .transfer()
        .unwrap()
        .bundle(&trusted(&fixture, "source").await, &component)
        .await
        .unwrap();
    let wrong_integrity = format!("sha512-{}==", "A".repeat(86));
    let packument_before = npm_doc(&fixture, "kept", "plain").await.body;
    let stored_before = stored_blobs(&fixture).await;
    for (field, value) in [("integrity", wrong_integrity), ("shasum", "0".repeat(40))] {
        for target in ["target", "kept"] {
            let mut bundle = genuine.clone();
            bundle.meta["manifest"]["dist"][field] = json!(value);
            let refused = npm_receive(&fixture, target, &bundle).await;
            let Err(ContentError::BadRequest(message)) = refused else {
                panic!("{field} into {target}: {refused:?}");
            };
            assert!(message.contains("does not match"), "{message}");
            assert!(!exists(&fixture, target, &npm_tarball_path("plain", "1.0.0")).await);
        }
    }
    assert_eq!(
        npm_doc(&fixture, "target", "plain").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        npm_doc(&fixture, "kept", "plain").await.body,
        packument_before
    );
    assert_eq!(stored_blobs(&fixture).await, stored_before);
    npm_receive(&fixture, "target", &genuine).await.unwrap();
    assert!(npm_doc(&fixture, "target", "plain").await.json()["versions"]["1.0.0"].is_object());
}

#[tokio::test]
async fn a_rerun_is_unchanged_and_a_missing_packument_entry_is_repaired() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["copied"]
    );
    let packument = npm_doc(&fixture, "target", "plain").await.body;
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["unchanged"]
    );
    assert_eq!(npm_doc(&fixture, "target", "plain").await.body, packument);
    store::delete(&support::repository_ctx(&fixture, "target").await, "/plain")
        .await
        .unwrap();
    assert_eq!(
        npm_doc(&fixture, "target", "plain").await.status,
        StatusCode::NOT_FOUND
    );
    let mut dry = request("source", "target", vec![npm_item("plain", "1.0.0")]);
    dry["dryRun"] = json!(true);
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, dry).await),
        ["copied"]
    );
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["copied"]
    );
    let doc = npm_doc(&fixture, "target", "plain").await.json();
    assert!(doc["versions"]["1.0.0"].is_object());
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["unchanged"]
    );
}

#[tokio::test]
async fn different_npm_content_follows_the_conflict_option_and_the_write_policy() {
    let fixture = world().await;
    npm_repositories(
        &fixture,
        &[
            ("source", "ALLOW"),
            ("target", "ALLOW"),
            ("once", "ALLOW_ONCE"),
            ("frozen", "DENY"),
        ],
    )
    .await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "source").await;
    for target in ["target", "once"] {
        npm_publish(&fixture, target, "plain", "1.0.0", "latest", "target").await;
    }
    let source_doc = npm_doc(&fixture, "source", "plain").await.json();
    for (target, option, expected) in [
        ("target", "fail", "conflict"),
        ("once", "fail", "conflict"),
        ("once", "replace", "conflict"),
    ] {
        let before = npm_doc(&fixture, target, "plain").await.body;
        let mut body = request("source", target, vec![npm_item("plain", "1.0.0")]);
        body["conflict"] = json!(option);
        assert_eq!(
            statuses(&promote(&fixture, Auth::Admin, body).await),
            [expected],
            "{target} {option}"
        );
        assert_eq!(npm_doc(&fixture, target, "plain").await.body, before);
        assert_eq!(
            served(&fixture, target, &npm_tarball_path("plain", "1.0.0"))
                .await
                .body,
            npm_tarball("plain", "1.0.0", "target")
        );
    }
    let mut body = request("source", "target", vec![npm_item("plain", "1.0.0")]);
    body["conflict"] = json!("replace");
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, body).await),
        ["copied"]
    );
    assert_eq!(
        served(&fixture, "target", &npm_tarball_path("plain", "1.0.0"))
            .await
            .body,
        npm_tarball("plain", "1.0.0", "source")
    );
    let doc = npm_doc(&fixture, "target", "plain").await.json();
    for field in ["shasum", "integrity"] {
        assert_eq!(
            doc["versions"]["1.0.0"]["dist"][field],
            source_doc["versions"]["1.0.0"]["dist"][field]
        );
    }
    assert_eq!(
        npm_promote(&fixture, "frozen", "plain", "1.0.0").await,
        ["failed"]
    );
    assert_eq!(
        npm_doc(&fixture, "frozen", "plain").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn an_npm_dry_run_writes_nothing_and_reports_the_plan() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    let mut body = request("source", "target", vec![npm_item("plain", "1.0.0")]);
    body["dryRun"] = json!(true);
    let planned = promote(&fixture, Auth::Admin, body.clone()).await;
    assert_eq!(statuses(&planned), ["copied"]);
    assert!(!exists(&fixture, "target", &npm_tarball_path("plain", "1.0.0")).await);
    assert!(!exists(&fixture, "target", "/plain").await);
    body["dryRun"] = json!(false);
    let real = promote(&fixture, Auth::Admin, body).await;
    for field in ["bytesCopied", "bytesReused", "assetCount"] {
        assert_eq!(
            planned.json()["results"][0][field],
            real.json()["results"][0][field]
        );
    }
}

#[tokio::test]
async fn moving_an_npm_version_removes_it_from_the_source_packument() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    npm_publish(&fixture, "source", "plain", "1.1.0", "beta", "a").await;
    let mut body = request("source", "target", vec![npm_item("plain", "1.1.0")]);
    body["move"] = json!(true);
    let reply = promote(&fixture, Auth::Admin, body.clone()).await;
    assert_eq!(statuses(&reply), ["copied"]);
    assert_eq!(reply.json()["results"][0]["sourceKept"], false);
    let doc = npm_doc(&fixture, "source", "plain").await.json();
    assert!(doc["versions"].get("1.1.0").is_none());
    assert!(doc["time"].get("1.1.0").is_none());
    assert_eq!(doc["dist-tags"], json!({"latest": "1.0.0"}));
    assert_eq!(
        served(&fixture, "source", &npm_tarball_path("plain", "1.1.0"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert!(npm_doc(&fixture, "target", "plain").await.json()["versions"]["1.1.0"].is_object());
    body["items"] = json!([npm_item("plain", "1.0.0")]);
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, body).await),
        ["copied"]
    );
    assert_eq!(
        npm_doc(&fixture, "source", "plain").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn concurrent_promotions_of_one_package_all_reach_its_packument() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    let versions: Vec<String> = (0..8).map(|minor| format!("1.{minor}.0")).collect();
    for version in &versions {
        npm_publish(&fixture, "source", "plain", version, "next", "a").await;
    }
    let runs = versions
        .iter()
        .map(|version| npm_promote(&fixture, "target", "plain", version));
    for result in futures_util::future::join_all(runs).await {
        assert_eq!(result, ["copied"]);
    }
    let doc = npm_doc(&fixture, "target", "plain").await.json();
    let listed: Vec<&String> = versions
        .iter()
        .filter(|version| doc["versions"].get(version.as_str()).is_some())
        .collect();
    assert_eq!(listed.len(), versions.len(), "{listed:?}");
    assert_eq!(doc["_rev"], "8");
}

#[tokio::test]
async fn the_dist_tags_option_is_ignored_by_other_formats() {
    let fixture = world().await;
    raw_pair(&fixture, "ALLOW", "ALLOW").await;
    put(&fixture, "source", "/a.txt", b"alpha").await;
    let item = component_of(&fixture, "source", "/a.txt").await;
    let mut body = request("source", "target", vec![item]);
    body["distTags"] = json!(false);
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, body).await),
        ["copied"]
    );
}

#[tokio::test]
async fn a_content_selector_that_leaves_the_packument_out_forbids_the_promotion() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "tarballs-only", "description": "", "expression": format!("format == \"npm\" and path == \"{}\"", npm_tarball_path("plain", "1.0.0"))}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "tarballs-add", "description": "", "actions": ["ADD"], "format": "npm", "repository": "target", "contentSelector": "tarballs-only"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "tarball-adder", "name": "tarball-adder", "description": "", "privileges": ["tarballs-add", "as-repository-view-npm-source-read"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("adder", PASSWORD, &["tarball-adder"])
        .await;
    let reply = promote(
        &fixture,
        Auth::Basic("adder", PASSWORD),
        request("source", "target", vec![npm_item("plain", "1.0.0")]),
    )
    .await;
    assert_eq!(statuses(&reply), ["forbidden"]);
    assert!(!exists(&fixture, "target", &npm_tarball_path("plain", "1.0.0")).await);
    assert!(!exists(&fixture, "target", "/plain").await);
}

async fn stored_packument(fixture: &Fixture, repository: &str, name: &str) -> Vec<u8> {
    use futures_util::TryStreamExt;
    let row = store::find(fixture.state.db().conn(), repository, &format!("/{name}"))
        .await
        .unwrap()
        .unwrap();
    let blobs = fixture.state.content().blob_stores();
    let content = blobs
        .get(&row.blob_store)
        .await
        .unwrap()
        .read(row.blob_id.into())
        .await
        .unwrap()
        .unwrap();
    content
        .stream
        .try_collect::<Vec<_>>()
        .await
        .unwrap()
        .concat()
}

async fn put_packument(fixture: &Fixture, repository: &str, name: &str, bytes: &[u8]) {
    let ctx = trusted(fixture, repository).await;
    let put = PutAsset {
        kind: "packument".to_owned(),
        content_type: Some("application/json".to_owned()),
        ..PutAsset::cached(format!("/{name}"))
    };
    store::put(&ctx, put, content_of(bytes.to_vec()))
        .await
        .unwrap();
}

#[tokio::test]
async fn a_failed_merge_leaves_no_new_tarball_in_the_target() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "2.0.0", "latest", "a").await;
    npm_publish(&fixture, "target", "plain", "1.0.0", "latest", "a").await;
    let good = stored_packument(&fixture, "target", "plain").await;
    put_packument(&fixture, "target", "plain", b"{not json").await;
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "2.0.0").await,
        ["failed"]
    );
    assert!(!exists(&fixture, "target", &npm_tarball_path("plain", "2.0.0")).await);
    put_packument(&fixture, "target", "plain", &good).await;
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "2.0.0").await,
        ["copied"]
    );
    assert!(npm_doc(&fixture, "target", "plain").await.json()["versions"]["2.0.0"].is_object());
}

#[tokio::test]
async fn a_rerun_rewrites_a_packument_entry_that_no_longer_matches_the_replaced_tarball() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "source").await;
    npm_publish(&fixture, "target", "plain", "1.0.0", "latest", "target").await;
    let stale = stored_packument(&fixture, "target", "plain").await;
    put_packument(&fixture, "target", "plain", b"{not json").await;
    let mut body = request("source", "target", vec![npm_item("plain", "1.0.0")]);
    body["conflict"] = json!("replace");
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, body).await),
        ["failed"]
    );
    assert_eq!(
        served(&fixture, "target", &npm_tarball_path("plain", "1.0.0"))
            .await
            .body,
        npm_tarball("plain", "1.0.0", "source")
    );
    put_packument(&fixture, "target", "plain", &stale).await;
    let mut dry = request("source", "target", vec![npm_item("plain", "1.0.0")]);
    dry["dryRun"] = json!(true);
    assert_eq!(
        statuses(&promote(&fixture, Auth::Admin, dry).await),
        ["copied"]
    );
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["copied"]
    );
    let from = npm_doc(&fixture, "source", "plain").await.json();
    let doc = npm_doc(&fixture, "target", "plain").await.json();
    for field in ["shasum", "integrity"] {
        assert_eq!(
            doc["versions"]["1.0.0"]["dist"][field],
            from["versions"]["1.0.0"]["dist"][field]
        );
    }
    assert_eq!(
        npm_promote(&fixture, "target", "plain", "1.0.0").await,
        ["unchanged"]
    );
}

#[tokio::test]
async fn the_stored_entry_names_the_target_tarball_and_the_digests_the_target_kept() {
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "@team/tool", "1.0.0", "latest", "a").await;
    assert_eq!(
        npm_promote(&fixture, "target", "@team/tool", "1.0.0").await,
        ["copied"]
    );
    let stored: Value =
        serde_json::from_slice(&stored_packument(&fixture, "target", "@team/tool").await).unwrap();
    let from = npm_doc(&fixture, "source", "@team/tool").await.json();
    let dist = &stored["versions"]["1.0.0"]["dist"];
    assert_eq!(
        dist["tarball"],
        "/repository/target/@team/tool/-/tool-1.0.0.tgz"
    );
    for field in ["shasum", "integrity"] {
        assert_eq!(dist[field], from["versions"]["1.0.0"]["dist"][field]);
    }
}

#[tokio::test]
async fn a_caller_who_cannot_reach_an_npm_package_gets_the_answer_for_a_package_that_is_not_there()
{
    let fixture = world().await;
    npm_repositories(&fixture, &[("source", "ALLOW"), ("target", "ALLOW")]).await;
    npm_publish(&fixture, "source", "plain", "1.0.0", "latest", "a").await;
    npm_publish(&fixture, "source", "other", "1.0.0", "latest", "a").await;
    for (name, actions, path) in [
        ("other-read", "READ", "/other/-/other-1.0.0.tgz"),
        ("plain-browse", "BROWSE", "/plain/-/plain-1.0.0.tgz"),
    ] {
        for (path_url, body) in [
            (
                "/service/rest/v1/security/content-selectors",
                json!({"name": name, "description": "", "expression": format!("format == \"npm\" and path == \"{path}\"")}),
            ),
            (
                "/service/rest/v1/security/privileges/repository-content-selector",
                json!({"name": format!("{name}-privilege"), "description": "", "actions": [actions], "format": "npm", "repository": "source", "contentSelector": name}),
            ),
            (
                "/service/rest/v1/security/roles",
                json!({"id": name, "name": name, "description": "", "privileges": [format!("{name}-privilege"), "as-repository-view-npm-target-add"], "roles": []}),
            ),
        ] {
            let reply = fixture
                .call(Method::POST, path_url, Auth::Admin, Some(body))
                .await;
            assert!(reply.status.is_success(), "{}", reply.text());
        }
        fixture.add_user(name, PASSWORD, &[name]).await;
    }
    let ask = |user: &'static str, package: &'static str| {
        let fixture = &fixture;
        async move {
            let reply = promote(
                fixture,
                Auth::Basic(user, PASSWORD),
                request("source", "target", vec![npm_item(package, "1.0.0")]),
            )
            .await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
            let result = reply.json()["results"][0].clone();
            (result["status"].clone(), result["error"].clone())
        }
    };
    let hidden = ask("other-read", "plain").await;
    let missing = ask("other-read", "ghost").await;
    assert_eq!(hidden, missing);
    assert_eq!(hidden.0, "notFound");
    let browse_only = ask("plain-browse", "plain").await;
    assert_eq!(browse_only.0, "forbidden");
    assert!(!exists(&fixture, "target", &npm_tarball_path("plain", "1.0.0")).await);
}
