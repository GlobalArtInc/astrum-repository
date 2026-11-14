mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::{get, ready, selector, selector_privilege, user, user_with};

const POLICIES: &str = "/service/rest/v1/security/policies";
const REPORT: &str = "/service/rest/v1/security/licenses";
const EXPORT: &str = "/service/rest/v1/security/licenses/export.csv";
const HEADER: &str = "repository,format,namespace,name,version,declared,expression,state,verdict\n";

async fn run(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(sql)
        .await
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

async fn number(fixture: &Fixture, sql: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}

async fn repository(fixture: &Fixture, name: &str, policy: Option<&str>) {
    let mut body = json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "ATTACHMENT"}
    });
    if let Some(policy) = policy {
        body["policy"] = json!({"name": policy});
    }
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn component(fixture: &Fixture, repository: &str, name: &str, paths: &[&str]) -> i64 {
    let id = number(
        fixture,
        &format!(
            "INSERT INTO component (repository, format, name, version) \
             VALUES ('{repository}', 'raw', '{name}', '1') RETURNING id AS n"
        ),
    )
    .await;
    for path in paths {
        run(
            fixture,
            &format!(
                "INSERT INTO asset (repository, format, path, component_id, blob_store, blob_id, \
                 size, content_type, sha1, sha256, sha512, md5) \
                 VALUES ('{repository}', 'raw', '{path}', {id}, 'default', gen_random_uuid(), 1, \
                 'text/plain', '', '', '', '')"
            ),
        )
        .await;
    }
    id
}

async fn licence(
    fixture: &Fixture,
    component: i64,
    declared: &str,
    expression: Option<&str>,
    ids: &[&str],
    state: &str,
) {
    let expression = expression.map_or("NULL".to_owned(), |text| format!("'{text}'"));
    let ids = ids
        .iter()
        .map(|id| format!("'{id}'"))
        .collect::<Vec<_>>()
        .join(", ");
    run(
        fixture,
        &format!(
            "INSERT INTO component_license (component_id, declared, source, expression, ids, state) \
             VALUES ({component}, '{declared}', 'npm.packument', {expression}, ARRAY[{ids}]::text[], '{state}')"
        ),
    )
    .await;
}

async fn verdict(
    fixture: &Fixture,
    repository: &str,
    component: i64,
    blocked: bool,
    state: &str,
    revision_offset: i64,
) {
    run(
        fixture,
        &format!(
            "INSERT INTO policy_verdict (repository, subject, policy, component_id, blocked, state, policy_revision) \
             VALUES ('{repository}', 'c:{component}', 'strict', {component}, {blocked}, '{state}', \
             (SELECT revision FROM content_policy WHERE name = 'strict') - {revision_offset})"
        ),
    )
    .await;
}

async fn world() -> Fixture {
    let fixture = ready().await;
    let created = fixture
        .call(
            Method::POST,
            POLICIES,
            Auth::Admin,
            Some(json!({"name": "strict"})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    repository(&fixture, "files", Some("strict")).await;
    repository(&fixture, "other", Some("strict")).await;
    repository(&fixture, "bare", None).await;

    let alpha = component(&fixture, "files", "alpha", &["/a/b/one"]).await;
    licence(&fixture, alpha, "MIT", Some("MIT"), &["MIT"], "spdx").await;
    verdict(&fixture, "files", alpha, false, "pass", 0).await;
    let beta = component(&fixture, "files", "beta", &["/a/x", "/a/b/two"]).await;
    licence(
        &fixture,
        beta,
        "GPL-2.0",
        Some("GPL-2.0"),
        &["GPL-2.0"],
        "spdx",
    )
    .await;
    verdict(&fixture, "files", beta, true, "block", 0).await;
    let gamma = component(&fixture, "files", "gamma", &["/z/three"]).await;
    licence(
        &fixture,
        gamma,
        "Apache 2.0",
        Some("Apache-2.0"),
        &["Apache-2.0"],
        "mapped",
    )
    .await;
    verdict(&fixture, "files", gamma, false, "report", 0).await;
    let zeta = component(&fixture, "files", "zeta", &["/z/x"]).await;
    licence(
        &fixture,
        zeta,
        "Custom, \"tricky\"\nnext line",
        None,
        &[],
        "unknown",
    )
    .await;
    verdict(&fixture, "files", zeta, false, "pass", 1).await;
    let omega = component(&fixture, "files", "omega", &["/z/y"]).await;
    licence(&fixture, omega, "=1+1", None, &[], "unknown").await;
    let delta = component(&fixture, "other", "delta", &["/d/four"]).await;
    licence(&fixture, delta, "", None, &[], "none").await;
    component(&fixture, "bare", "epsilon", &["/e/five"]).await;
    fixture
}

async fn ok(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Value {
    let reply = get(fixture, path, auth).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.json()
}

async fn ask(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    get(fixture, path, auth).await
}

fn names(page: &Value) -> Vec<&str> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect()
}

async fn walk(fixture: &Fixture, query: &str) -> Vec<Value> {
    let mut items = Vec::new();
    let mut token: Option<String> = None;
    for _ in 0..20 {
        let path = match &token {
            Some(token) => format!("{REPORT}?{query}&continuationToken={token}"),
            None => format!("{REPORT}?{query}"),
        };
        let page = ok(fixture, &path, Auth::Admin).await;
        items.extend(page["items"].as_array().unwrap().clone());
        match page["continuationToken"].as_str() {
            Some(next) => token = Some(next.to_owned()),
            None => return items,
        }
    }
    panic!("the walk does not end");
}

async fn two_users(fixture: &Fixture) {
    user_with(fixture, "alice", &["as-repository-view-raw-files-browse"]).await;
    selector(fixture, "deep", "path =^ \"/a/b/\"").await;
    selector_privilege(fixture, "files-deep", "deep", "raw", "files", &["BROWSE"]).await;
    user_with(fixture, "deep", &["files-deep"]).await;
    selector(fixture, "nothing", "path =^ \"/nothing/\"").await;
    selector_privilege(
        fixture,
        "files-nothing",
        "nothing",
        "raw",
        "files",
        &["BROWSE"],
    )
    .await;
    user_with(fixture, "hidden", &["files-nothing"]).await;
}

#[tokio::test]
async fn anonymous_callers_get_401_on_the_list_and_the_export() {
    let fixture = world().await;
    for path in [REPORT, EXPORT] {
        let reply = ask(&fixture, path, Auth::None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
async fn the_list_has_a_row_per_component_with_a_licence_record() {
    let fixture = world().await;
    let page = ok(&fixture, REPORT, Auth::Admin).await;
    assert_eq!(
        names(&page),
        ["alpha", "beta", "gamma", "zeta", "omega", "delta"]
    );
    assert_eq!(page["continuationToken"], Value::Null);
    let rows = page["items"].as_array().unwrap();
    assert_eq!(
        rows[0],
        json!({
            "repository": "files", "format": "raw", "namespace": "", "name": "alpha",
            "version": "1", "declared": "MIT", "expression": "MIT", "ids": ["MIT"],
            "state": "spdx", "verdict": "pass"
        })
    );
    assert_eq!(rows[2]["state"], "mapped");
    assert_eq!(rows[2]["ids"], json!(["Apache-2.0"]));
    assert_eq!(rows[4]["expression"], Value::Null);
    assert_eq!(rows[4]["state"], "unknown");
    assert_eq!(rows[5]["state"], "none");
}

#[tokio::test]
async fn filters_combine_and_a_deprecated_id_matches_its_equivalent() {
    let fixture = world().await;
    let cases = [
        ("license=gpl-2.0-only", vec!["beta"]),
        ("license=GPL-2.0", vec!["beta"]),
        ("license=MIT&repository=files", vec!["alpha"]),
        ("repository=files&state=unknown", vec!["zeta", "omega"]),
        ("state=mapped", vec!["gamma"]),
        ("state=none", vec!["epsilon", "delta"]),
        ("state=none&repository=other&format=raw", vec!["delta"]),
        ("state=none&format=npm", vec![]),
        ("license=GPL-2.0&state=spdx&format=raw", vec!["beta"]),
    ];
    for (query, expected) in cases {
        let page = ok(&fixture, &format!("{REPORT}?{query}"), Auth::Admin).await;
        assert_eq!(names(&page), expected, "{query}");
    }
    for query in ["state=bogus", "format=", "continuationToken=not-a-token"] {
        let reply = ask(&fixture, &format!("{REPORT}?{query}"), Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{query}");
    }
}

#[tokio::test]
async fn pages_of_one_hundred_follow_a_keyset_in_repository_and_component_order() {
    let fixture = world().await;
    for index in 1..=150 {
        let path = format!("/bulk/{index:03}");
        let id = component(&fixture, "files", &format!("bulk-{index:03}"), &[&path]).await;
        licence(&fixture, id, "MIT", Some("MIT"), &["MIT"], "spdx").await;
    }
    let first = ok(&fixture, &format!("{REPORT}?license=MIT"), Auth::Admin).await;
    assert_eq!(first["items"].as_array().unwrap().len(), 100);
    assert!(first["continuationToken"].is_string());
    let all = walk(&fixture, "license=MIT").await;
    let listed: Vec<&str> = all
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    let mut expected = vec!["alpha".to_owned()];
    expected.extend((1..=150).map(|index| format!("bulk-{index:03}")));
    assert_eq!(listed, expected);
}

#[tokio::test]
async fn verdicts_read_pass_blocked_reported_and_unjudged() {
    let fixture = world().await;
    let page = ok(&fixture, REPORT, Auth::Admin).await;
    let verdicts: Vec<(&str, &str)> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["name"].as_str().unwrap(),
                row["verdict"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        verdicts,
        [
            ("alpha", "pass"),
            ("beta", "blocked"),
            ("gamma", "reported"),
            ("zeta", "unjudged"),
            ("omega", "unjudged"),
            ("delta", "unjudged"),
        ]
    );
    let bare = ok(
        &fixture,
        &format!("{REPORT}?state=none&repository=bare"),
        Auth::Admin,
    )
    .await;
    assert_eq!(bare["items"][0]["verdict"], "unjudged");
}

#[tokio::test]
async fn a_caller_sees_only_the_repositories_and_paths_it_may_browse() {
    let fixture = world().await;
    two_users(&fixture).await;

    let files = ok(&fixture, REPORT, user("alice")).await;
    assert_eq!(names(&files), ["alpha", "beta", "gamma", "zeta", "omega"]);
    let deep = ok(&fixture, REPORT, user("deep")).await;
    assert_eq!(names(&deep), ["alpha", "beta"]);
    let hidden = ok(&fixture, REPORT, user("hidden")).await;
    assert!(names(&hidden).is_empty());

    let export = ask(&fixture, EXPORT, user("deep")).await;
    assert_eq!(export.status, StatusCode::OK);
    assert_eq!(export.text().lines().count(), 3);
    assert!(export.text().contains("alpha") && !export.text().contains("gamma"));
}

#[tokio::test]
async fn the_export_streams_csv_with_quoting_and_a_formula_guard() {
    let fixture = world().await;
    let reply = get(&fixture, EXPORT, Auth::Admin).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.headers["content-type"], "text/csv; charset=utf-8");
    assert_eq!(
        reply.headers["content-disposition"],
        "attachment; filename=\"licenses.csv\""
    );
    let expected = format!(
        "{HEADER}\
         files,raw,,alpha,1,MIT,MIT,spdx,pass\n\
         files,raw,,beta,1,GPL-2.0,GPL-2.0,spdx,blocked\n\
         files,raw,,gamma,1,Apache 2.0,Apache-2.0,mapped,reported\n\
         files,raw,,zeta,1,\"Custom, \"\"tricky\"\"\nnext line\",,unknown,unjudged\n\
         files,raw,,omega,1,'=1+1,,unknown,unjudged\n\
         other,raw,,delta,1,,,none,unjudged\n"
    );
    assert_eq!(reply.text(), expected);
}

#[tokio::test]
async fn the_export_refuses_above_the_cap_before_streaming() {
    let fixture = world().await;
    fixture.state.set_licence_export_cap(1);
    let refused = ask(&fixture, EXPORT, Auth::Admin).await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert!(refused.text().contains("narrow the filter"));
    let narrow = ask(
        &fixture,
        &format!("{EXPORT}?license=Apache-2.0"),
        Auth::Admin,
    )
    .await;
    assert_eq!(narrow.status, StatusCode::OK);
    assert_eq!(narrow.text().lines().count(), 2);
    fixture
        .state
        .set_licence_export_cap(astrum_api::routes::licenses::EXPORT_ROW_CAP);
    let whole = ask(&fixture, EXPORT, Auth::Admin).await;
    assert_eq!(whole.status, StatusCode::OK);
}
