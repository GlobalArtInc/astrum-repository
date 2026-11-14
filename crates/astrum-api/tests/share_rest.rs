mod support;
mod ui_support;

use std::sync::Arc;

use astrum_db::entity::{audit_event, security_user_role, share_link};
use axum::http::{Method, StatusCode};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::{
    anonymous, create, names, put, raw_group, raw_hosted, raw_proxy, role, selector,
    selector_privilege, user, user_with,
};
use uuid::Uuid;

const LINKS: &str = "/service/rest/v1/share-links";
const TOKENS: &str = "/service/rest/internal/access-tokens";
const HOST: &str = "astrum.example";
const SHARER: [&str; 2] = ["as-sharelinks-create", "as-repository-view-raw-files-read"];

async fn ready() -> Fixture {
    let fixture = ui_support::ready().await;
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    for name in ["files", "other"] {
        raw_hosted(&fixture, name).await;
        put(
            &fixture,
            &format!("/repository/{name}/notes/one.txt"),
            "first file",
        )
        .await;
        put(
            &fixture,
            &format!("/repository/{name}/notes/deep/two.txt"),
            "second file",
        )
        .await;
        put(
            &fixture,
            &format!("/repository/{name}/docs/three.txt"),
            "third file",
        )
        .await;
    }
    fixture
}

async fn call(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    body: Option<Value>,
) -> Reply {
    fixture
        .send(method, path, auth, body, &[("host", HOST)])
        .await
}

async fn make(fixture: &Fixture, auth: Auth<'_>, body: Value) -> Reply {
    call(fixture, Method::POST, LINKS, auth, Some(body)).await
}

fn single(path: &str) -> Value {
    json!({"repository": "files", "path": path})
}

async fn made(fixture: &Fixture, auth: Auth<'_>, body: Value) -> Value {
    let reply = make(fixture, auth, body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()
}

fn secret_of(url: &str) -> String {
    let (_, rest) = url.split_once("/service/share/").unwrap();
    rest.split('/').next().unwrap().to_owned()
}

fn local(url: &str) -> &str {
    &url[url.find("/service/share/").unwrap()..]
}

async fn download(fixture: &Fixture, path: &str, host: &str) -> Reply {
    fixture
        .send(Method::GET, path, Auth::None, None, &[("host", host)])
        .await
}

async fn token(fixture: &Fixture, scope: &str) -> String {
    let reply = call(
        fixture,
        Method::POST,
        TOKENS,
        Auth::Admin,
        Some(json!({"name": format!("share-{scope}"), "scope": scope, "expiresInDays": 30})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    reply.json()["token"].as_str().unwrap().to_owned()
}

async fn with_token(
    fixture: &Fixture,
    token: &str,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Reply {
    let bearer = format!("Bearer {token}");
    fixture
        .send(
            method,
            path,
            Auth::None,
            body,
            &[("host", HOST), ("authorization", bearer.as_str())],
        )
        .await
}

async fn events(fixture: &Fixture) -> Vec<audit_event::Model> {
    audit_event::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap()
        .into_iter()
        .filter(|event| event.domain == "security.sharelink")
        .collect()
}

#[tokio::test]
async fn a_link_made_with_read_access_and_the_privilege_downloads_through_the_public_route() {
    let fixture = ready().await;
    user_with(&fixture, "sharer", &SHARER).await;
    let body = json!({
        "repository": "files", "path": "/notes/one.txt",
        "expiresInHours": 2, "maxDownloads": 2, "note": "for acme"
    });
    let reply = make(&fixture, user("sharer"), body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert_eq!(reply.headers["cache-control"], "no-store");
    let created = reply.json();
    let url = created["url"].as_str().unwrap();
    assert!(
        url.starts_with("http://astrum.example/service/share/astrum_share_")
            && url.ends_with("/one.txt"),
        "{url}"
    );
    let mut keys: Vec<&str> = created
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "bytesServed",
            "createdAt",
            "createdBy",
            "downloads",
            "expired",
            "expiresAt",
            "id",
            "lastUsedAt",
            "maxDownloads",
            "note",
            "path",
            "prefix",
            "repository",
            "url"
        ]
    );
    assert_eq!(created["createdBy"], "sharer");
    assert_eq!(created["maxDownloads"], 2);
    let reply = download(&fixture, local(url), HOST).await;
    assert_eq!(
        (reply.status, reply.text()),
        (StatusCode::OK, "first file".to_owned())
    );
    let listed = call(&fixture, Method::GET, LINKS, user("sharer"), None).await;
    let items = listed.json();
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item["id"], created["id"]);
    assert_eq!(
        (item["downloads"].as_i64(), item["bytesServed"].as_i64()),
        (Some(1), Some(10))
    );
    let text = listed.text();
    assert!(!text.contains(&secret_of(url)) && !text.contains("url") && !text.contains("secret"));
}

#[tokio::test]
async fn every_route_wants_a_signed_in_caller_and_the_right_privilege() {
    let fixture = ready().await;
    let admin_link = made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let id = admin_link["id"].as_str().unwrap();
    let one = format!("{LINKS}/{id}");
    user_with(&fixture, "plain", &["as-repository-view-raw-files-read"]).await;
    for (method, path, body) in [
        (Method::POST, LINKS, Some(single("/notes/one.txt"))),
        (Method::GET, LINKS, None),
        (Method::DELETE, one.as_str(), None),
    ] {
        let reply = call(&fixture, method.clone(), path, Auth::None, body).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
    }
    let refused = make(&fixture, user("plain"), single("/notes/one.txt")).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    role(&fixture, "anon-sharer", &SHARER).await;
    anonymous(&fixture, true).await;
    security_user_role::ActiveModel {
        user_id: ActiveValue::Unchanged("anonymous".to_owned()),
        source: ActiveValue::Unchanged("default".to_owned()),
        roles: ActiveValue::Set(json!(["anon-sharer"])),
        ..Default::default()
    }
    .update(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let by_anonymous = make(&fixture, Auth::None, single("/notes/one.txt")).await;
    assert_eq!(by_anonymous.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn creating_needs_read_access_to_the_path_and_a_prefix_needs_the_whole_repository() {
    let fixture = ready().await;
    user_with(
        &fixture,
        "elsewhere",
        &["as-sharelinks-create", "as-repository-view-raw-other-read"],
    )
    .await;
    let denied = make(&fixture, user("elsewhere"), single("/notes/one.txt")).await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    selector(&fixture, "one", "path == \"/notes/one.txt\"").await;
    selector_privilege(&fixture, "files-one", "one", "raw", "files", &["READ"]).await;
    user_with(&fixture, "selected", &["as-sharelinks-create", "files-one"]).await;
    made(&fixture, user("selected"), single("/notes/one.txt")).await;
    let other = make(&fixture, user("selected"), single("/notes/deep/two.txt")).await;
    assert_eq!(other.status, StatusCode::FORBIDDEN);
    let prefix = make(&fixture, user("selected"), single("/notes/one.txt/")).await;
    assert_eq!(prefix.status, StatusCode::FORBIDDEN);
    let folder = make(
        &fixture,
        user("selected"),
        json!({"repository": "files", "path": "/notes/", "prefix": true}),
    )
    .await;
    assert_eq!(folder.status, StatusCode::FORBIDDEN, "{}", folder.text());

    user_with(&fixture, "whole", &SHARER).await;
    let created = made(
        &fixture,
        user("whole"),
        json!({"repository": "files", "path": "/notes/", "prefix": true}),
    )
    .await;
    assert_eq!(created["prefix"], true);
    assert!(created["url"].as_str().unwrap().ends_with('/'));
    let reply = download(&fixture, local(created["url"].as_str().unwrap()), HOST).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.text().contains("one.txt"));
}

#[tokio::test]
async fn only_online_hosted_repositories_of_other_formats_get_links() {
    let fixture = ready().await;
    raw_proxy(&fixture, "mirror").await;
    raw_group(&fixture, "everything", &["files"]).await;
    create(
        &fixture,
        "docker",
        "hosted",
        json!({
            "name": "images", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
            "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
        }),
    )
    .await;
    for name in ["mirror", "everything", "images"] {
        let reply = make(
            &fixture,
            Auth::Admin,
            json!({"repository": name, "path": "/a.txt"}),
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{name}");
        assert_eq!(reply.json()[0]["id"], "repository", "{name}");
    }
    let mut offline = json!({
        "name": "files", "online": false,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    });
    let reply = call(
        &fixture,
        Method::PUT,
        "/service/rest/v1/repositories/raw/hosted/files",
        Auth::Admin,
        Some(offline.take()),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = make(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = make(
        &fixture,
        Auth::Admin,
        json!({"repository": "nowhere", "path": "/a.txt"}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_lifetime_is_bounded_by_the_setting_and_a_prefix_has_no_download_limit() {
    let fixture = ready().await;
    let hours = |value: Value| {
        let mut body = single("/notes/one.txt");
        body["expiresInHours"] = value;
        body
    };
    let default = made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let expires = chrono::DateTime::parse_from_rfc3339(default["expiresAt"].as_str().unwrap())
        .unwrap()
        .with_timezone(&chrono::Utc);
    let span = expires - chrono::Utc::now();
    assert!(span > chrono::Duration::hours(167) && span <= chrono::Duration::hours(168));
    for bad in [json!(0), json!(721), json!(-5)] {
        let reply = make(&fixture, Auth::Admin, hours(bad.clone())).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(reply.json()[0]["id"], "expiresInHours");
    }
    made(&fixture, Auth::Admin, hours(json!(720))).await;

    astrum_db::setting::put(fixture.state.db().conn(), "shareLinks.maxDays", &2u32)
        .await
        .unwrap();
    let reply = make(&fixture, Auth::Admin, hours(json!(49))).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    made(&fixture, Auth::Admin, hours(json!(48))).await;
    let capped = made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let expires = chrono::DateTime::parse_from_rfc3339(capped["expiresAt"].as_str().unwrap())
        .unwrap()
        .with_timezone(&chrono::Utc);
    assert!(expires - chrono::Utc::now() <= chrono::Duration::hours(48));
    astrum_db::setting::put(fixture.state.db().conn(), "shareLinks.maxDays", &0u32)
        .await
        .unwrap();
    let reply = make(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    astrum_db::setting::put(fixture.state.db().conn(), "shareLinks.maxDays", &30u32)
        .await
        .unwrap();

    for (body, field) in [
        (
            json!({"repository": "files", "path": "/notes/", "maxDownloads": 3}),
            "maxDownloads",
        ),
        (
            json!({"repository": "files", "path": "/notes/one.txt", "maxDownloads": 0}),
            "maxDownloads",
        ),
        (
            json!({"repository": "files", "path": "/notes/one.txt", "prefix": true}),
            "prefix",
        ),
        (
            json!({"repository": "files", "path": "/notes/../one.txt"}),
            "path",
        ),
    ] {
        let reply = make(&fixture, Auth::Admin, body.clone()).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(reply.json()[0]["id"], field, "{body}");
    }
    let reply = make(
        &fixture,
        Auth::Admin,
        json!({"repository": "files", "path": "/notes/one.txt", "maxdownloads": 1}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn everyone_lists_own_links_and_all_needs_the_read_privilege() {
    let fixture = ready().await;
    user_with(&fixture, "sharer", &SHARER).await;
    user_with(&fixture, "auditor", &["as-sharelinks-read"]).await;
    user_with(&fixture, "plain", &[]).await;
    made(&fixture, user("sharer"), single("/notes/one.txt")).await;
    made(
        &fixture,
        user("sharer"),
        json!({"repository": "files", "path": "/docs/three.txt"}),
    )
    .await;
    made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let own = |auth| call(&fixture, Method::GET, LINKS, auth, None);
    let by = |reply: Reply| names(&reply.json(), "createdBy");
    assert_eq!(by(own(user("sharer")).await), ["sharer", "sharer"]);
    assert_eq!(by(own(user("plain")).await), Vec::<String>::new());
    let all = format!("{LINKS}?all=true");
    let denied = call(&fixture, Method::GET, &all, user("sharer"), None).await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let everything = call(&fixture, Method::GET, &all, user("auditor"), None).await;
    assert_eq!(everything.status, StatusCode::OK);
    assert_eq!(by(everything).len(), 3);
    let narrowed = call(
        &fixture,
        Method::GET,
        &format!("{LINKS}?all=true&repository=other"),
        user("auditor"),
        None,
    )
    .await;
    assert!(narrowed.json().as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_creator_revokes_own_links_and_revoking_others_needs_the_delete_privilege() {
    let fixture = ready().await;
    user_with(&fixture, "sharer", &SHARER).await;
    user_with(&fixture, "janitor", &["as-sharelinks-delete"]).await;
    let mine = made(&fixture, user("sharer"), single("/notes/one.txt")).await;
    let theirs = made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let path = |link: &Value| format!("{LINKS}/{}", link["id"].as_str().unwrap());
    let refused = call(
        &fixture,
        Method::DELETE,
        &path(&theirs),
        user("sharer"),
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let own = call(&fixture, Method::DELETE, &path(&mine), user("sharer"), None).await;
    assert_eq!(own.status, StatusCode::NO_CONTENT);
    let gone = download(&fixture, local(mine["url"].as_str().unwrap()), HOST).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let again = call(&fixture, Method::DELETE, &path(&mine), user("sharer"), None).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let others = call(
        &fixture,
        Method::DELETE,
        &path(&theirs),
        user("janitor"),
        None,
    )
    .await;
    assert_eq!(others.status, StatusCode::NO_CONTENT);
    let nonsense = call(
        &fixture,
        Method::DELETE,
        &format!("{LINKS}/nonsense"),
        Auth::Admin,
        None,
    )
    .await;
    assert_eq!(nonsense.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_read_scope_token_cannot_create_or_revoke_links() {
    let fixture = ready().await;
    let write = token(&fixture, "WRITE").await;
    let read = token(&fixture, "READ").await;
    let created = with_token(
        &fixture,
        &write,
        Method::POST,
        LINKS,
        Some(single("/notes/one.txt")),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let refused = with_token(
        &fixture,
        &read,
        Method::POST,
        LINKS,
        Some(single("/notes/one.txt")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let revoke = with_token(
        &fixture,
        &read,
        Method::DELETE,
        &format!("{LINKS}/{id}"),
        None,
    )
    .await;
    assert_eq!(revoke.status, StatusCode::FORBIDDEN);
    let listed = with_token(&fixture, &read, Method::GET, LINKS, None).await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
    let revoked = with_token(
        &fixture,
        &write,
        Method::DELETE,
        &format!("{LINKS}/{id}"),
        None,
    )
    .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn audit_rows_for_created_and_deleted_hold_no_secret_hash_or_url() {
    let fixture = ready().await;
    let created = made(
        &fixture,
        Auth::Admin,
        json!({"repository": "files", "path": "/notes/one.txt", "note": "confidential-note"}),
    )
    .await;
    let url = created["url"].as_str().unwrap().to_owned();
    let secret = secret_of(&url);
    let id = created["id"].as_str().unwrap();
    let revoked = call(
        &fixture,
        Method::DELETE,
        &format!("{LINKS}/{id}"),
        Auth::Admin,
        None,
    )
    .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    let rows = events(&fixture).await;
    let kinds: Vec<&str> = rows.iter().map(|row| row.kind.as_str()).collect();
    assert_eq!(kinds, ["created", "deleted"]);
    assert!(rows.iter().all(|row| row.context == id));
    let dump = format!("{rows:?}");
    for leaked in [
        secret.as_str(),
        astrum_content::share::hash(&secret).as_str(),
        url.as_str(),
        "/service/share/",
        "confidential-note",
    ] {
        assert!(!dump.contains(leaked), "{leaked}");
    }
    assert!(rows[0].attributes.get("via").is_none());
}

#[tokio::test]
async fn the_url_uses_the_domain_of_the_repository_and_the_base_url_otherwise() {
    let fixture = ready().await;
    let mut rooted = json!({
        "name": "rooted", "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"},
        "domain": {"host": "files.example.org", "routing": "ROOT"}
    });
    create(&fixture, "raw", "hosted", rooted.take()).await;
    put(&fixture, "/repository/rooted/notes/one.txt", "rooted file").await;
    let proxied = [("host", HOST), ("x-forwarded-proto", "https")];
    let reply = fixture
        .send(
            Method::POST,
            LINKS,
            Auth::Admin,
            Some(json!({"repository": "rooted", "path": "/notes/one.txt"})),
            &proxied,
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let url = reply.json()["url"].as_str().unwrap().to_owned();
    assert!(
        url.starts_with("https://files.example.org/service/share/astrum_share_"),
        "{url}"
    );
    let served = download(&fixture, local(&url), "files.example.org").await;
    assert_eq!(served.text(), "rooted file");
    let plain = fixture
        .send(
            Method::POST,
            LINKS,
            Auth::Admin,
            Some(single("/notes/one.txt")),
            &proxied,
        )
        .await;
    assert!(
        plain.json()["url"]
            .as_str()
            .unwrap()
            .starts_with("https://astrum.example/service/share/")
    );
    let with_port = fixture
        .send(
            Method::POST,
            LINKS,
            Auth::Admin,
            Some(json!({"repository": "rooted", "path": "/notes/one.txt"})),
            &[("host", "astrum.example:8081")],
        )
        .await;
    assert_eq!(
        with_port.status,
        StatusCode::CREATED,
        "{}",
        with_port.text()
    );
    assert!(
        with_port.json()["url"]
            .as_str()
            .unwrap()
            .starts_with("http://files.example.org:8081/service/share/astrum_share_")
    );
}

async fn rpc(fixture: &Fixture, token: &str, method: &str, params: Value) -> Value {
    let reply = with_token(
        fixture,
        token,
        Method::POST,
        "/mcp",
        Some(json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

async fn tool(fixture: &Fixture, token: &str, name: &str, arguments: Value) -> Value {
    let reply = rpc(
        fixture,
        token,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
    .await;
    assert!(reply["error"].is_null(), "{reply}");
    reply["result"].clone()
}

#[tokio::test]
async fn the_agent_tools_make_list_and_revoke_links_and_say_so_in_the_audit_log() {
    let fixture = ready().await;
    let write = token(&fixture, "WRITE").await;
    let read = token(&fixture, "READ").await;
    let listed = |reply: &Value| -> Vec<String> {
        reply["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_owned())
            .collect()
    };
    let for_write = listed(&rpc(&fixture, &write, "tools/list", json!({})).await);
    for name in ["create_share_link", "list_share_links", "revoke_share_link"] {
        assert!(for_write.contains(&name.to_owned()), "{name}");
    }
    let for_read = listed(&rpc(&fixture, &read, "tools/list", json!({})).await);
    assert!(for_read.contains(&"list_share_links".to_owned()));
    assert!(!for_read.contains(&"create_share_link".to_owned()));
    assert!(!for_read.contains(&"revoke_share_link".to_owned()));

    let created = tool(
        &fixture,
        &write,
        "create_share_link",
        json!({"repository": "files", "path": "/notes/one.txt", "expiresInHours": 3, "maxDownloads": 1}),
    )
    .await;
    assert_eq!(created["isError"], false, "{created}");
    let link = &created["structuredContent"];
    let url = link["url"].as_str().unwrap();
    assert!(
        url.starts_with("http://astrum.example/service/share/astrum_share_"),
        "{url}"
    );
    assert_eq!(
        download(&fixture, local(url), HOST).await.text(),
        "first file"
    );

    let all = tool(&fixture, &write, "list_share_links", json!({"all": true})).await;
    let items = all["structuredContent"]["links"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], link["id"]);
    assert!(!all.to_string().contains(&secret_of(url)));

    let revoked = tool(
        &fixture,
        &write,
        "revoke_share_link",
        json!({"id": link["id"]}),
    )
    .await;
    assert_eq!(revoked["isError"], false, "{revoked}");
    assert_eq!(
        download(&fixture, local(url), HOST).await.status,
        StatusCode::NOT_FOUND
    );

    let rows = events(&fixture).await;
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter().all(|row| row.attributes["via"] == "mcp"),
        "{rows:?}"
    );
    assert!(!format!("{rows:?}").contains(&secret_of(url)));
}

#[tokio::test]
async fn deleting_a_user_removes_the_links_of_that_user_and_source_only() {
    let fixture = ready().await;
    user_with(&fixture, "leaver", &SHARER).await;
    made(&fixture, user("leaver"), single("/notes/one.txt")).await;
    made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    insert(&fixture, "LDAP", "leaver", 1, 0).await;
    insert(&fixture, "OAuth2", "leaver", 1, 0).await;
    let remove = |realm: &'static str| {
        let path = format!("/service/rest/v1/security/users/leaver?realm={realm}");
        let fixture = &fixture;
        async move {
            let reply = call(fixture, Method::DELETE, &path, Auth::Admin, None).await;
            assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
        }
    };
    let pairs = || async {
        let mut pairs: Vec<(String, String)> = share_link::Entity::find()
            .all(fixture.state.db().conn())
            .await
            .unwrap()
            .into_iter()
            .map(|row| (row.created_source, row.created_by))
            .collect();
        pairs.sort();
        pairs
    };
    let pair = |source: &str, user: &str| (source.to_owned(), user.to_owned());
    remove("AstrumAuthenticatingRealm").await;
    assert_eq!(
        pairs().await,
        [
            pair("LDAP", "leaver"),
            pair("OAuth2", "leaver"),
            pair("default", "admin")
        ]
    );
    security_user_role::ActiveModel {
        user_id: ActiveValue::Set("leaver".to_owned()),
        source: ActiveValue::Set("OAuth2".to_owned()),
        roles: ActiveValue::Set(json!([])),
        updated_at: ActiveValue::Set(Utc::now().fixed_offset()),
    }
    .insert(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    remove("OAuth2Realm").await;
    assert_eq!(
        pairs().await,
        [pair("LDAP", "leaver"), pair("default", "admin")]
    );
}

#[tokio::test]
async fn users_of_two_sources_with_one_name_neither_see_nor_revoke_each_others_links() {
    let fixture = ready().await;
    user_with(&fixture, "alice", &SHARER).await;
    let mine = made(&fixture, user("alice"), single("/notes/one.txt")).await;
    let foreign = insert(&fixture, "LDAP", "alice", 1, 0).await[0];
    let listed = call(&fixture, Method::GET, LINKS, user("alice"), None).await;
    let items = listed.json();
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], mine["id"]);
    assert!(!listed.text().contains("bulk note"));
    let revoke = call(
        &fixture,
        Method::DELETE,
        &format!("{LINKS}/{foreign}"),
        user("alice"),
        None,
    )
    .await;
    assert_eq!(revoke.status, StatusCode::FORBIDDEN);
    assert!(
        share_link::Entity::find_by_id(foreign)
            .one(fixture.state.db().conn())
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn text_with_control_or_direction_characters_is_refused_with_the_field_name() {
    let fixture = ready().await;
    for (field, value) in [
        ("note", "a\u{0}b"),
        ("note", "line\nbreak"),
        ("note", "invoice \u{202e}fdp.exe"),
        ("note", "\u{2066}hidden\u{2069}"),
        ("note", "mark\u{200f}"),
        ("note", "arabic\u{061c}mark"),
        ("note", "line\u{2028}separator"),
        ("note", "paragraph\u{2029}separator"),
        ("repository", "files\u{0}"),
        ("path", "/notes/one.txt\u{0}"),
    ] {
        let mut body = single("/notes/one.txt");
        body[field] = json!(value);
        let reply = make(&fixture, Auth::Admin, body).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{field} {value:?}");
        assert_eq!(reply.json()[0]["id"], field, "{value:?}");
    }
    let reply = call(
        &fixture,
        Method::GET,
        &format!("{LINKS}?repository=%00"),
        Auth::Admin,
        None,
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()[0]["id"], "repository");
    let mut body = single("/notes/one.txt");
    body["note"] = json!("для acme: заказ №5");
    made(&fixture, Auth::Admin, body).await;
}

#[tokio::test]
async fn an_unreadable_lifetime_setting_falls_back_to_the_default() {
    let fixture = ready().await;
    astrum_db::setting::put(fixture.state.db().conn(), "shareLinks.maxDays", &"lots")
        .await
        .unwrap();
    made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let mut over = single("/notes/one.txt");
    over["expiresInHours"] = json!(721);
    assert_eq!(
        make(&fixture, Auth::Admin, over).await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn a_user_holds_at_most_a_thousand_active_links_and_lists_active_ones_first() {
    let fixture = ready().await;
    user_with(&fixture, "hoarder", &SHARER).await;
    let ids = insert(&fixture, "default", "hoarder", 995, 50).await;
    for _ in 0..5 {
        made(&fixture, user("hoarder"), single("/notes/one.txt")).await;
    }
    let refused = make(&fixture, user("hoarder"), single("/notes/one.txt")).await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    made(&fixture, Auth::Admin, single("/notes/one.txt")).await;
    let listed = call(&fixture, Method::GET, LINKS, user("hoarder"), None).await;
    let items = listed.json();
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1000);
    assert!(items.iter().all(|item| !item["expired"].as_bool().unwrap()));
    let listed_ids: Vec<&str> = items
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert!(!listed_ids.contains(&ids[0].to_string().as_str()));
    assert!(listed_ids.contains(&ids[ids.len() - 1].to_string().as_str()));
}

async fn insert(
    fixture: &Fixture,
    source: &str,
    user: &str,
    active: i64,
    expired: i64,
) -> Vec<Uuid> {
    let now = Utc::now();
    let models: Vec<share_link::ActiveModel> = (0..expired + active)
        .map(|index| share_link::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            secret_hash: ActiveValue::Set(format!("bulk-{}", Uuid::new_v4())),
            repository: ActiveValue::Set("files".to_owned()),
            path: ActiveValue::Set("/notes/one.txt".to_owned()),
            prefix: ActiveValue::Set(false),
            note: ActiveValue::Set("bulk note".to_owned()),
            created_by: ActiveValue::Set(user.to_owned()),
            created_source: ActiveValue::Set(source.to_owned()),
            created_at: ActiveValue::Set((now - Duration::seconds(10 + index)).fixed_offset()),
            expires_at: ActiveValue::Set(
                if index < expired {
                    now - Duration::days(1)
                } else {
                    now + Duration::hours(1)
                }
                .fixed_offset(),
            ),
            max_downloads: ActiveValue::Set(None),
            downloads: ActiveValue::Set(0),
            bytes_served: ActiveValue::Set(0),
            last_used_at: ActiveValue::Set(None),
            last_used_ip: ActiveValue::Set(None),
        })
        .collect();
    let ids = models
        .iter()
        .map(|model| model.id.clone().unwrap())
        .collect();
    share_link::Entity::insert_many(models)
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    ids
}
