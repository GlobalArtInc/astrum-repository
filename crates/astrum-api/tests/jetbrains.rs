mod support;

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use astrum_format_jetbrains::testing::{bundle, descriptor, jar};
use astrum_format_jetbrains::xml::{self, Entry};
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "jetbrainsboundary0123456789";
const TOOL: &str = "org.acme.tool";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_jetbrains::Jetbrains::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/jetbrains/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str, policy: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy}
    })
}

fn group(name: &str, members: &[&str]) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "group": {"memberNames": members},
    })
}

fn proxy(name: &str, remote: &str, metadata_max_age: u32) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": metadata_max_age},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {"blocked": false, "autoBlock": false},
    })
}

async fn token(fixture: &Fixture, scope: Scope) -> String {
    format!(
        "Bearer {}",
        fixture
            .state
            .security()
            .create_token("admin", "jetbrains", scope, None)
            .await
            .unwrap()
            .1
    )
}

fn multipart(parts: &[(&str, Option<&str>, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, file, data) in parts {
        match file {
            Some(file) => write!(
                body,
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
            ),
            None => write!(
                body,
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n"
            ),
        }
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

fn form_type() -> String {
    format!("multipart/form-data; boundary={BOUNDARY}")
}

async fn put(fixture: &Fixture, path: &str, authorization: Option<&str>, data: &[u8]) -> Reply {
    let mut headers = vec![("content-type", "application/octet-stream")];
    if let Some(authorization) = authorization {
        headers.push(("authorization", authorization));
    }
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::None,
            Body::from(data.to_vec()),
            &headers,
        )
        .await
}

async fn upload(
    fixture: &Fixture,
    repository: &str,
    authorization: &str,
    fields: &[(&str, &str)],
    data: &[u8],
) -> Reply {
    let mut parts: Vec<(&str, Option<&str>, &[u8])> = fields
        .iter()
        .map(|(name, value)| (*name, None, value.as_bytes()))
        .collect();
    parts.push(("file", Some("plugin.jar"), data));
    let content_type = form_type();
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/updates/upload"),
            Auth::None,
            Body::from(multipart(&parts)),
            &[
                ("content-type", content_type.as_str()),
                ("authorization", authorization),
            ],
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn entries(reply: &Reply) -> Vec<Entry> {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    xml::parse_entries(&reply.body).unwrap_or_else(|error| panic!("{error}: {}", reply.text()))
}

fn shown(entries: &[Entry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| format!("{}@{}", entry.id, entry.version))
        .collect()
}

fn header_of<'a>(reply: &'a Reply, name: &str) -> Option<&'a str> {
    reply
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn plugin_path(id: &str, version: &str, extension: &str) -> String {
    format!("/{id}/{version}/{id}-{version}.{extension}")
}

fn tool(version: &str, since: &str, until: &str) -> Vec<u8> {
    bundle("tool", &descriptor(TOOL, version, since, until))
}

#[tokio::test]
async fn hosted_plugins_are_uploaded_and_listed_for_the_ide() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plugins", "ALLOW_ONCE")).await;
    let writer = token(&fixture, Scope::Write).await;
    let reader = token(&fixture, Scope::Read).await;

    let page = fixture
        .call(Method::GET, "/repository/plugins/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("updatePlugins.xml"));
    assert!(page.text().contains("idea.plugins.host"));

    let first = tool("1.0.0", "241.0", "243.*");
    let stored = put(
        &fixture,
        &format!("/repository/plugins{}", plugin_path(TOOL, "1.0.0", "zip")),
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    assert_eq!(stored.json()["pluginXmlId"], TOOL);
    assert_eq!(stored.json()["version"], "1.0.0");
    assert!(
        header_of(&stored, "location")
            .unwrap()
            .ends_with("/repository/plugins/org.acme.tool/1.0.0/org.acme.tool-1.0.0.zip")
    );

    let again = put(
        &fixture,
        &format!("/repository/plugins{}", plugin_path(TOOL, "1.0.0", "zip")),
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    assert!(again.text().contains("redeploy is not allowed"));

    let misplaced = put(
        &fixture,
        &format!("/repository/plugins{}", plugin_path(TOOL, "1.0.1", "zip")),
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(misplaced.status, StatusCode::BAD_REQUEST);
    assert!(
        misplaced.text().contains("belongs at"),
        "{}",
        misplaced.text()
    );
    let garbage = put(
        &fixture,
        "/repository/plugins/x.zip",
        Some(&writer),
        b"nope",
    )
    .await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert!(garbage.text().contains("not a plugin archive"));
    let plain = astrum_content::zip::write(&[("readme.txt", b"hello")]);
    let without = put(&fixture, "/repository/plugins/y.zip", Some(&writer), &plain).await;
    assert_eq!(without.status, StatusCode::BAD_REQUEST);
    assert!(without.text().contains("META-INF/plugin.xml"));
    let broken = put(
        &fixture,
        "/repository/plugins/z.jar",
        Some(&writer),
        &jar("<idea-plugin><id>a/b</id><version>1</version></idea-plugin>"),
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    let denied = put(&fixture, "/repository/plugins/w.zip", Some(&reader), &first).await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let anonymous = put(&fixture, "/repository/plugins/w.zip", None, &first).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));

    let newest = jar(&descriptor(TOOL, "1.10.0", "241.0", "243.*"));
    let published = upload(&fixture, "plugins", &writer, &[("xmlId", TOOL)], &newest).await;
    assert_eq!(published.status, StatusCode::OK, "{}", published.text());
    assert_eq!(published.json()["version"], "1.10.0");
    assert!(
        published.json()["url"]
            .as_str()
            .unwrap()
            .ends_with("/repository/plugins/org.acme.tool/1.10.0/org.acme.tool-1.10.0.jar")
    );
    let numeric = upload(
        &fixture,
        "plugins",
        &writer,
        &[("pluginId", "5047")],
        &tool("1.9.0", "241.0", "243.*"),
    )
    .await;
    assert_eq!(numeric.status, StatusCode::OK, "{}", numeric.text());
    let wrong_id = upload(
        &fixture,
        "plugins",
        &writer,
        &[("xmlId", "org.acme.other")],
        &newest,
    )
    .await;
    assert_eq!(wrong_id.status, StatusCode::BAD_REQUEST);
    assert!(wrong_id.text().contains("declares the id org.acme.tool"));
    let no_file = fixture
        .send_body(
            Method::POST,
            "/repository/plugins/api/updates/upload",
            Auth::None,
            Body::from(multipart(&[("xmlId", None, TOOL.as_bytes())])),
            &[
                ("content-type", form_type().as_str()),
                ("authorization", writer.as_str()),
            ],
        )
        .await;
    assert_eq!(no_file.status, StatusCode::BAD_REQUEST);
    assert!(no_file.text().contains("no file part"));
    let unauthorised = upload(&fixture, "plugins", &reader, &[], &newest).await;
    assert_eq!(unauthorised.status, StatusCode::FORBIDDEN);

    let newer_ide = bundle(
        "other",
        &descriptor("org.acme.other", "2.0.0", "250.0", "251.*"),
    );
    let other = upload(&fixture, "plugins", &writer, &[], &newer_ide).await;
    assert_eq!(other.status, StatusCode::OK, "{}", other.text());
    let spaced = bundle(
        "spaced",
        &descriptor("Acme Spaced", "0.1+b1", "100.0", "999.*"),
    );
    let spaced_reply = upload(&fixture, "plugins", &writer, &[], &spaced).await;
    assert_eq!(
        spaced_reply.status,
        StatusCode::OK,
        "{}",
        spaced_reply.text()
    );
    let preview = jar(&descriptor(TOOL, "3.0.0-eap1", "241.0", "243.*"));
    let channel = upload(
        &fixture,
        "plugins",
        &writer,
        &[("channel", "EAP")],
        &preview,
    )
    .await;
    assert_eq!(channel.status, StatusCode::OK, "{}", channel.text());
    assert_eq!(channel.json()["channel"], "eap");

    let documented = get(&fixture, "/repository/plugins/updatePlugins.xml").await;
    assert_eq!(
        header_of(&documented, "content-type"),
        Some("text/xml;charset=UTF-8")
    );
    let listed = entries(&documented);
    assert_eq!(
        shown(&listed),
        [
            "Acme Spaced@0.1+b1",
            "org.acme.other@2.0.0",
            "org.acme.tool@1.10.0"
        ]
    );
    let text = documented.text();
    assert!(text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plugins>\n"));
    assert!(text.contains(
        "/repository/plugins/org.acme.tool/1.10.0/org.acme.tool-1.10.0.jar\" version=\"1.10.0\">"
    ));
    assert!(
        text.contains("/repository/plugins/Acme%20Spaced/0.1%2Bb1/Acme%20Spaced-0.1%2Bb1.zip\"")
    );
    assert!(text.contains("<idea-version since-build=\"241.0\" until-build=\"243.*\"/>"));
    assert!(text.contains("<name>org.acme.tool name</name>"));
    assert!(text.contains(
        "<vendor email=\"dev@example.test\" url=\"https://example.test/\">Example Vendor</vendor>"
    ));
    assert!(
        text.contains("<description><![CDATA[<p>The org.acme.tool plugin.</p>]]></description>")
    );
    assert!(text.contains("<change-notes><![CDATA[Version 1.10.0.]]></change-notes>"));
    assert!(text.contains("<depends>com.intellij.modules.platform</depends>"));

    let build =
        |build: &str| format!("/repository/plugins/updatePlugins.xml?build={build}&uuid=anything");
    assert_eq!(
        shown(&entries(&get(&fixture, &build("IU-243.21565.193")).await)),
        ["Acme Spaced@0.1+b1", "org.acme.tool@1.10.0"]
    );
    assert_eq!(
        shown(&entries(&get(&fixture, &build("GO-251.1")).await)),
        ["Acme Spaced@0.1+b1", "org.acme.other@2.0.0"]
    );
    assert_eq!(
        shown(&entries(&get(&fixture, &build("IU-200.5")).await)),
        ["Acme Spaced@0.1+b1"]
    );
    assert_eq!(
        shown(&entries(
            &get(
                &fixture,
                "/repository/plugins/updatePlugins.xml?channel=eap"
            )
            .await
        )),
        ["org.acme.tool@3.0.0-eap1"]
    );

    let url = listed[2]
        .xml
        .split("url=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let path = &url[url.find("/repository/").unwrap()..];
    let downloaded = get(&fixture, path).await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(downloaded.body, newest);
    assert_eq!(
        header_of(&downloaded, "content-disposition"),
        Some("attachment; filename=\"org.acme.tool-1.10.0.jar\"")
    );

    let legacy = get(
        &fixture,
        "/repository/plugins/plugins/list?build=IU-243.21565.193&pluginId=org.acme.tool",
    )
    .await;
    let legacy_entries = entries(&legacy);
    assert_eq!(shown(&legacy_entries), ["org.acme.tool@1.10.0"]);
    assert_eq!(legacy_entries[0].category, "Tools");
    let text = legacy.text();
    assert!(text.starts_with("<?xml version='1.0' encoding='UTF-8'?><plugin-repository><category name=\"Tools\"><idea-plugin downloads=\"0\""));
    assert!(text.contains(
        "<idea-version min=\"n/a\" max=\"n/a\" since-build=\"241.0\" until-build=\"243.*\"/>"
    ));
    assert!(text.contains(
        "<vendor email=\"dev@example.test\" url=\"https://example.test/\">Example Vendor</vendor>"
    ));
    let everything = get(&fixture, "/repository/plugins/plugins/list/?build=IU-243.1").await;
    assert_eq!(
        shown(&entries(&everything)),
        ["Acme Spaced@0.1+b1", "org.acme.tool@1.10.0"]
    );
    let early = get(
        &fixture,
        "/repository/plugins/plugins/eap/list?build=IU-243.1",
    )
    .await;
    assert_eq!(shown(&entries(&early)), ["org.acme.tool@3.0.0-eap1"]);
    let nothing = get(
        &fixture,
        "/repository/plugins/plugins/list?pluginId=unknown.plugin",
    )
    .await;
    assert_eq!(nothing.status, StatusCode::OK);
    assert_eq!(
        nothing.text(),
        "<?xml version='1.0' encoding='UTF-8'?><plugin-repository/>"
    );

    let managed = get(
        &fixture,
        "/repository/plugins/pluginManager?action=download&id=org.acme.tool&build=IU-243.21565.193",
    )
    .await;
    assert_eq!(managed.status, StatusCode::OK, "{}", managed.text());
    assert_eq!(managed.body, newest);
    assert_eq!(
        header_of(&managed, "content-disposition"),
        Some("attachment; filename=\"org.acme.tool-1.10.0.jar\"")
    );
    let slashed = get(
        &fixture,
        "/repository/plugins/pluginManager/?action=download&id=Acme%20Spaced&build=IU-243.1",
    )
    .await;
    assert_eq!(slashed.status, StatusCode::OK);
    assert_eq!(slashed.body, spaced);
    let incompatible = get(
        &fixture,
        "/repository/plugins/pluginManager?action=download&id=org.acme.tool&build=IU-251.1",
    )
    .await;
    assert_eq!(incompatible.status, StatusCode::NOT_FOUND);
    assert!(
        incompatible
            .text()
            .contains("Can't find update for plugin \"org.acme.tool\"")
    );
    let bounded = get(
        &fixture,
        "/repository/plugins/pluginManager?action=download&id=org.acme.tool&build=IU-243.1&channel=eap",
    )
    .await;
    assert_eq!(bounded.body, preview);
    for bad in [
        "/repository/plugins/pluginManager?action=download&build=IU-243.1",
        "/repository/plugins/pluginManager?action=upload&id=org.acme.tool",
    ] {
        assert_eq!(
            get(&fixture, bad).await.status,
            StatusCode::BAD_REQUEST,
            "{bad}"
        );
    }

    let by_version = get(
        &fixture,
        "/repository/plugins/plugin/download?pluginId=org.acme.tool&version=1.0.0",
    )
    .await;
    assert_eq!(by_version.status, StatusCode::OK);
    assert_eq!(by_version.body, first);
    assert_eq!(
        get(
            &fixture,
            "/repository/plugins/plugin/download?pluginId=org.acme.tool&version=9.9.9"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/plugins/plugin/download?pluginId=org.acme.tool"
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );

    let direct = "/repository/plugins/org.acme.tool/1.0.0/org.acme.tool-1.0.0.zip";
    let head = fixture.call(Method::HEAD, direct, Auth::Admin, None).await;
    assert_eq!(head.status, StatusCode::OK);
    assert_eq!(
        header_of(&head, "content-length"),
        Some(first.len().to_string().as_str())
    );
    assert!(head.body.is_empty());
    let part = fixture
        .send(
            Method::GET,
            direct,
            Auth::Admin,
            None,
            &[("range", "bytes=0-3")],
        )
        .await;
    assert_eq!(part.status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(part.body, first[..4]);
    assert_eq!(
        get(
            &fixture,
            "/repository/plugins/org.acme.tool/1.0.0/other-1.0.0.zip"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let removed = fixture
        .call(Method::DELETE, direct, Auth::Admin, None)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(get(&fixture, direct).await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        fixture
            .call(Method::DELETE, direct, Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let restored = put(
        &fixture,
        &format!("/repository/plugins{}", plugin_path(TOOL, "1.0.0", "zip")),
        Some(&writer),
        &first,
    )
    .await;
    assert_eq!(restored.status, StatusCode::CREATED, "{}", restored.text());
}

#[tokio::test]
async fn components_search_and_the_upload_form_cover_plugins() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plugins", "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    let archive = tool("1.0.0", "241.0", "243.*");
    let reply = put(
        &fixture,
        "/repository/plugins/first.zip",
        Some(&writer),
        &archive,
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let overwritten = put(
        &fixture,
        "/repository/plugins/second.zip",
        Some(&writer),
        &archive,
    )
    .await;
    assert_eq!(
        overwritten.status,
        StatusCode::CREATED,
        "{}",
        overwritten.text()
    );

    let content_type = form_type();
    let upload_form = |data: &[u8], channel: &str| {
        multipart(&[
            ("jetbrains.channel", None, channel.as_bytes()),
            ("jetbrains.asset", Some("plugin.jar"), data),
        ])
    };
    let sent = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=plugins",
            Auth::Admin,
            Body::from(upload_form(
                &jar(&descriptor("org.acme.preview", "0.1.0", "241.0", "243.*")),
                "Beta",
            )),
            &[("content-type", content_type.as_str())],
        )
        .await;
    assert_eq!(sent.status, StatusCode::NO_CONTENT, "{}", sent.text());
    let refused = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=plugins",
            Auth::Admin,
            Body::from(upload_form(b"not an archive", "")),
            &[("content-type", content_type.as_str())],
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let components = get(&fixture, "/service/rest/v1/components?repository=plugins").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    let names: Vec<(&str, &str)> = items
        .iter()
        .map(|item| {
            (
                item["name"].as_str().unwrap(),
                item["version"].as_str().unwrap(),
            )
        })
        .collect();
    assert!(names.contains(&(TOOL, "1.0.0")), "{names:?}");
    assert!(names.contains(&("org.acme.preview", "0.1.0")), "{names:?}");
    assert_eq!(items[0]["format"], "jetbrains");

    let by_vendor = get(
        &fixture,
        "/service/rest/v1/search?repository=plugins&jetbrains.vendor=Example%20Vendor",
    )
    .await;
    assert_eq!(
        by_vendor.json()["items"].as_array().unwrap().len(),
        2,
        "{}",
        by_vendor.text()
    );
    let by_channel = get(
        &fixture,
        "/service/rest/v1/search?repository=plugins&jetbrains.channel=beta",
    )
    .await;
    let found = by_channel.json()["items"].as_array().unwrap().clone();
    assert_eq!(found.len(), 1, "{}", by_channel.text());
    assert_eq!(found[0]["name"], "org.acme.preview");
    let by_category = get(
        &fixture,
        "/service/rest/v1/search?repository=plugins&jetbrains.category=Tools",
    )
    .await;
    assert_eq!(by_category.json()["items"].as_array().unwrap().len(), 2);
    let by_keyword = get(
        &fixture,
        "/service/rest/v1/search?repository=plugins&q=preview",
    )
    .await;
    assert_eq!(by_keyword.json()["items"][0]["name"], "org.acme.preview");
}

#[tokio::test]
async fn content_selectors_decide_which_plugins_a_reader_sees() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("plugins", "ALLOW")).await;
    let writer = token(&fixture, Scope::Write).await;
    for (id, dir) in [("org.acme.public", "public"), ("org.acme.secret", "secret")] {
        let reply = upload(
            &fixture,
            "plugins",
            &writer,
            &[],
            &bundle(dir, &descriptor(id, "1.0.0", "241.0", "243.*")),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    }
    for (path, body) in [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "public-plugins", "description": "", "expression": "format == \"jetbrains\" and path =^ \"/org.acme.public/\""}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "public-plugins-read", "description": "", "actions": ["READ"], "format": "jetbrains", "repository": "plugins", "contentSelector": "public-plugins"}),
        ),
        (
            "/service/rest/v1/security/roles",
            json!({"id": "plugin-readers", "name": "plugin-readers", "description": "", "privileges": ["public-plugins-read"], "roles": []}),
        ),
    ] {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{}", reply.text());
    }
    fixture
        .add_user("reader", "reader-pass", &["plugin-readers"])
        .await;
    let reader = || Auth::Basic("reader", "reader-pass");
    let call = |path: &'static str| {
        let fixture = &fixture;
        async move { fixture.call(Method::GET, path, reader(), None).await }
    };
    assert_eq!(
        shown(&entries(
            &call("/repository/plugins/updatePlugins.xml").await
        )),
        ["org.acme.public@1.0.0"]
    );
    assert_eq!(
        shown(&entries(
            &call("/repository/plugins/plugins/list?build=IU-243.1").await
        )),
        ["org.acme.public@1.0.0"]
    );
    assert_eq!(
        call("/repository/plugins/pluginManager?action=download&id=org.acme.public&build=IU-243.1")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        call("/repository/plugins/pluginManager?action=download&id=org.acme.secret&build=IU-243.1")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call("/repository/plugins/org.acme.secret/1.0.0/org.acme.secret-1.0.0.zip")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call("/repository/plugins/plugin/download?pluginId=org.acme.secret&version=1.0.0")
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[derive(Clone, Default)]
struct Upstream {
    seen: Arc<Mutex<Vec<String>>>,
    bodies: Arc<AtomicUsize>,
    conditional: Arc<AtomicUsize>,
    plugin: Arc<Vec<u8>>,
    other: Arc<Vec<u8>>,
}

impl Upstream {
    fn count(&self, prefix: &str) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|seen| seen.starts_with(prefix))
            .count()
    }

    fn last(&self, prefix: &str) -> Option<String> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|seen| seen.starts_with(prefix))
            .cloned()
    }
}

fn marketplace_entry(id: &str, version: &str) -> String {
    format!(
        "<idea-plugin downloads=\"5\" size=\"10\" date=\"1\" updatedDate=\"2\" url=\"\"><name>{id} name</name><id>{id}</id><description><![CDATA[Remote {id}]]></description><version>{version}</version><vendor email=\"\" url=\"\">Remote</vendor><change-notes><![CDATA[]]></change-notes><idea-version min=\"n/a\" max=\"n/a\" since-build=\"241.0\" until-build=\"243.*\"/></idea-plugin>"
    )
}

fn json_error(status: StatusCode, message: &str) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        json!({"statusCode": status.as_u16(), "message": message}).to_string(),
    )
        .into_response()
}

async fn marketplace(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
) -> Response {
    let query = query.unwrap_or_default();
    upstream
        .seen
        .lock()
        .unwrap()
        .push(format!("{path}?{query}"));
    let redirect = |location: &str| {
        (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, location.to_owned())],
        )
            .into_response()
    };
    match path.as_str() {
        "plugins/list" if query.contains("pluginId=org.up.tool") => (
            [(header::CONTENT_TYPE, "text/xml;charset=UTF-8")],
            format!(
                "<?xml version='1.0' encoding='UTF-8'?><plugin-repository><category name=\"Tools\">{}</category></plugin-repository>",
                marketplace_entry("org.up.tool", "1.2.0")
            ),
        )
            .into_response(),
        "plugins/list" if query.contains("pluginId=") => {
            json_error(StatusCode::NOT_FOUND, "Can't find Plugin with id")
        }
        "plugins/list" => (
            [(header::CONTENT_TYPE, "text/xml;charset=UTF-8")],
            format!(
                "<?xml version='1.0' encoding='UTF-8'?><plugin-repository><category name=\"Tools\">{}{}</category></plugin-repository>",
                marketplace_entry("org.acme.tool", "5.0.0"),
                marketplace_entry("org.up.tool", "1.2.0")
            ),
        )
            .into_response(),
        "pluginManager" if query.contains("id=org.up.tool") => {
            redirect("/files/164/9/tool-1.2.0.zip?updateId=9")
        }
        "pluginManager" if query.contains("id=org.up.broken") => redirect("/files/1/2/broken.zip"),
        "pluginManager" if query.contains("id=org.acme.tool") => {
            redirect("/files/1/3/acme-tool-5.0.0.zip")
        }
        "plugin/download" if query.contains("pluginId=org.up.tool&version=1.2.0") => {
            redirect("/files/164/9/tool-1.2.0.zip?updateId=9")
        }
        "files/164/9/tool-1.2.0.zip" => {
            let etag = "\"tool-1.2.0\"";
            if headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) == Some(etag) {
                upstream.conditional.fetch_add(1, Ordering::SeqCst);
                return (StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response();
            }
            upstream.bodies.fetch_add(1, Ordering::SeqCst);
            (
                [
                    (header::CONTENT_TYPE, "application/zip"),
                    (header::ETAG, etag),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"tool-1.2.0.zip\"",
                    ),
                ],
                upstream.plugin.as_ref().clone(),
            )
                .into_response()
        }
        "files/1/3/acme-tool-5.0.0.zip" => {
            upstream.bodies.fetch_add(1, Ordering::SeqCst);
            (
                [(header::CONTENT_TYPE, "application/zip")],
                upstream.other.as_ref().clone(),
            )
                .into_response()
        }
        "files/1/2/broken.zip" => (
            [(header::CONTENT_TYPE, "application/zip")],
            astrum_content::zip::write(&[("readme.txt", b"hello")]),
        )
            .into_response(),
        "pluginManager" | "plugin/download" => json_error(
            StatusCode::NOT_FOUND,
            "Can't find update for the plugin",
        ),
        _ => json_error(StatusCode::NOT_FOUND, "Not found"),
    }
}

async fn serve_marketplace(fixture: &Fixture) -> (Upstream, String) {
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
    let state = Upstream {
        plugin: Arc::new(bundle(
            "tool",
            &descriptor("org.up.tool", "1.2.0", "241.0", "243.*"),
        )),
        other: Arc::new(bundle("acme", &descriptor(TOOL, "5.0.0", "241.0", "243.*"))),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(marketplace))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (state, format!("http://{address}"))
}

#[tokio::test]
async fn proxies_cache_marketplace_lists_and_plugins() {
    let fixture = ready().await;
    let (upstream, remote) = serve_marketplace(&fixture).await;
    create(&fixture, "proxy", proxy("market", &remote, 1440)).await;
    create(&fixture, "proxy", proxy("live", &remote, 0)).await;

    let list = "/repository/market/plugins/list?build=IU-243.1&pluginId=org.up.tool&uuid=1234";
    let first = get(&fixture, list).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text());
    assert_eq!(shown(&entries(&first)), ["org.up.tool@1.2.0"]);
    assert_eq!(header_of(&first, "content-type"), Some("text/xml"));
    assert_eq!(
        upstream.last("plugins/list").as_deref(),
        Some("plugins/list?build=IU-243.1&pluginId=org.up.tool")
    );
    let second = get(&fixture, list).await;
    assert_eq!(second.body, first.body);
    assert_eq!(upstream.count("plugins/list"), 1);
    let restyled = get(
        &fixture,
        "/repository/market/plugins/list/?pluginId=org.up.tool&build=IU-243.1",
    )
    .await;
    assert_eq!(restyled.status, StatusCode::OK);
    assert_eq!(upstream.count("plugins/list"), 1);

    let invalid = get(&fixture, "/repository/market/plugins/list?build=banana").await;
    assert_eq!(
        invalid.status,
        StatusCode::BAD_REQUEST,
        "{}",
        invalid.text()
    );
    assert_eq!(upstream.count("plugins/list"), 1);

    let missing = "/repository/market/plugins/list?build=IU-243.1&pluginId=no.such.plugin";
    assert_eq!(get(&fixture, missing).await.status, StatusCode::NOT_FOUND);
    assert_eq!(get(&fixture, missing).await.status, StatusCode::NOT_FOUND);
    assert_eq!(upstream.count("plugins/list"), 2);

    let managed = "/repository/market/pluginManager?action=download&id=org.up.tool&build=IU-243.1";
    let downloaded = get(&fixture, managed).await;
    assert_eq!(downloaded.status, StatusCode::OK, "{}", downloaded.text());
    assert_eq!(downloaded.body, *upstream.plugin);
    assert_eq!(
        header_of(&downloaded, "content-disposition"),
        Some("attachment; filename=\"org.up.tool-1.2.0.zip\"")
    );
    assert_eq!(upstream.bodies.load(Ordering::SeqCst), 1);
    let again = get(&fixture, managed).await;
    assert_eq!(again.body, *upstream.plugin);
    assert_eq!(upstream.count("pluginManager"), 1);
    assert_eq!(upstream.bodies.load(Ordering::SeqCst), 1);

    let cached = get(
        &fixture,
        "/repository/market/org.up.tool/1.2.0/org.up.tool-1.2.0.zip",
    )
    .await;
    assert_eq!(cached.status, StatusCode::OK);
    assert_eq!(cached.body, *upstream.plugin);
    let by_version = get(
        &fixture,
        "/repository/market/plugin/download?pluginId=org.up.tool&version=1.2.0",
    )
    .await;
    assert_eq!(by_version.body, *upstream.plugin);
    assert_eq!(upstream.count("plugin/download"), 0);

    let other_build = get(
        &fixture,
        "/repository/market/pluginManager?action=download&id=org.up.tool&build=GO-243.9",
    )
    .await;
    assert_eq!(other_build.body, *upstream.plugin);
    assert_eq!(upstream.count("pluginManager"), 2);
    let components = get(&fixture, "/service/rest/v1/components?repository=market").await;
    let items = components.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1, "{}", components.text());
    assert_eq!(items[0]["name"], "org.up.tool");
    assert_eq!(items[0]["version"], "1.2.0");

    let tree = get(&fixture, "/service/rest/internal/ui/browse/market?path=/").await;
    assert_eq!(tree.status, StatusCode::OK, "{}", tree.text());
    let names: Vec<String> = tree.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["org.up.tool"]);

    let unknown = "/repository/market/pluginManager?action=download&id=no.such&build=IU-243.1";
    assert_eq!(get(&fixture, unknown).await.status, StatusCode::NOT_FOUND);
    assert_eq!(get(&fixture, unknown).await.status, StatusCode::NOT_FOUND);
    assert_eq!(upstream.count("pluginManager"), 3);

    let absent = "/repository/market/plugin/download?pluginId=org.up.tool&version=9.9.9";
    assert_eq!(get(&fixture, absent).await.status, StatusCode::NOT_FOUND);
    assert_eq!(get(&fixture, absent).await.status, StatusCode::NOT_FOUND);
    assert_eq!(upstream.count("plugin/download"), 1);

    let broken = get(
        &fixture,
        "/repository/market/pluginManager?action=download&id=org.up.broken&build=IU-243.1",
    )
    .await;
    assert_eq!(broken.status, StatusCode::BAD_GATEWAY, "{}", broken.text());
    assert!(broken.text().contains("did not send a plugin archive"));

    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/market/org.up.tool/1.2.0/org.up.tool-1.2.0.zip",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let refetched = get(&fixture, managed).await;
    assert_eq!(refetched.status, StatusCode::OK);
    assert_eq!(refetched.body, *upstream.plugin);
    assert_eq!(upstream.bodies.load(Ordering::SeqCst), 3);

    let documented = get(&fixture, "/repository/market/updatePlugins.xml").await;
    assert_eq!(documented.status, StatusCode::NOT_FOUND);
    let upload = put(
        &fixture,
        "/repository/market/x.zip",
        Some(&token(&fixture, Scope::Write).await),
        &upstream.plugin,
    )
    .await;
    assert_eq!(upload.status, StatusCode::BAD_REQUEST);

    let live = "/repository/live/pluginManager?action=download&id=org.up.tool&build=IU-243.1";
    let before = upstream.bodies.load(Ordering::SeqCst);
    assert_eq!(get(&fixture, live).await.status, StatusCode::OK);
    assert_eq!(upstream.bodies.load(Ordering::SeqCst), before + 1);
    let revalidated = get(&fixture, live).await;
    assert_eq!(revalidated.status, StatusCode::OK);
    assert_eq!(revalidated.body, *upstream.plugin);
    assert_eq!(upstream.bodies.load(Ordering::SeqCst), before + 1);
    assert_eq!(upstream.conditional.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn groups_put_their_members_in_order() {
    let fixture = ready().await;
    let (upstream, remote) = serve_marketplace(&fixture).await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(&fixture, "proxy", proxy("market", &remote, 1440)).await;
    create(&fixture, "group", group("all", &["own", "market"])).await;
    let writer = token(&fixture, Scope::Write).await;
    let own = tool("1.0.0", "241.0", "243.*");
    assert_eq!(
        upload(&fixture, "own", &writer, &[], &own).await.status,
        StatusCode::OK
    );

    let list = get(&fixture, "/repository/all/plugins/list?build=IU-243.1").await;
    let merged = entries(&list);
    assert_eq!(shown(&merged), ["org.acme.tool@1.0.0", "org.up.tool@1.2.0"]);
    assert!(merged[0].xml.contains("Example Vendor"));
    assert!(merged[1].xml.contains("Remote"));
    assert_eq!(merged[1].category, "Tools");

    let documented = get(&fixture, "/repository/all/updatePlugins.xml?build=IU-243.1").await;
    assert_eq!(shown(&entries(&documented)), ["org.acme.tool@1.0.0"]);
    assert!(
        documented
            .text()
            .contains("/repository/all/org.acme.tool/1.0.0/org.acme.tool-1.0.0.zip\"")
    );

    let own_download = get(
        &fixture,
        "/repository/all/pluginManager?action=download&id=org.acme.tool&build=IU-243.1",
    )
    .await;
    assert_eq!(own_download.body, own);
    assert_eq!(upstream.count("pluginManager"), 0);
    let remote_download = get(
        &fixture,
        "/repository/all/pluginManager?action=download&id=org.up.tool&build=IU-243.1",
    )
    .await;
    assert_eq!(remote_download.body, *upstream.plugin);
    let missing = get(
        &fixture,
        "/repository/all/pluginManager?action=download&id=nobody&build=IU-243.1",
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let by_version = get(
        &fixture,
        "/repository/all/plugin/download?pluginId=org.up.tool&version=1.2.0",
    )
    .await;
    assert_eq!(by_version.body, *upstream.plugin);
    let direct = get(
        &fixture,
        "/repository/all/org.acme.tool/1.0.0/org.acme.tool-1.0.0.zip",
    )
    .await;
    assert_eq!(direct.body, own);
    let write = put(&fixture, "/repository/all/x.zip", Some(&writer), &own).await;
    assert_eq!(write.status, StatusCode::BAD_REQUEST);
    let publish = upload(&fixture, "all", &writer, &[], &own).await;
    assert_eq!(publish.status, StatusCode::BAD_REQUEST);
}
