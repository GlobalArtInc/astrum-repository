mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_conda::testing;
use astrum_security::SecurityModel;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "condaboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_conda::Conda));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/conda/{kind}"),
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

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn put(fixture: &Fixture, path: &str, data: Vec<u8>) -> Reply {
    fixture
        .send_body(Method::PUT, path, Auth::Admin, Body::from(data), &[])
        .await
}

fn form(fields: &[(&str, &str)], file: (&str, &str, &[u8])) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        )
        .unwrap();
    }
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
        file.0, file.1
    )
    .unwrap();
    body.extend_from_slice(file.2);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

fn unbz2(data: &[u8]) -> Vec<u8> {
    testing::unbzip2(data)
}

fn unzst(data: &[u8]) -> Vec<u8> {
    testing::unzstd(data)
}

fn files(document: &Value, key: &str) -> Vec<String> {
    document[key]
        .as_object()
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

#[tokio::test]
async fn hosted_channels_take_packages_and_answer_generated_repodata() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("conda-hosted", "ALLOW_ONCE")).await;
    let hello = testing::tar_bz2(
        &testing::index("hello", "1.0", "py_0", "noarch"),
        Some(&json!({"summary": "Say hello", "home": "https://example.org/hello"})),
    );
    let reply = put(
        &fixture,
        "/repository/conda-hosted/main/noarch/hello-1.0-py_0.tar.bz2",
        hello.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let again = put(
        &fixture,
        "/repository/conda-hosted/main/noarch/hello-1.0-py_0.tar.bz2",
        hello.clone(),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    let tool = testing::conda(&testing::index("tool", "2.0", "h1_0", "linux-64"), None);
    let reply = put(
        &fixture,
        "/repository/conda-hosted/main/linux-64/tool-2.0-h1_0.conda",
        tool.clone(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    for (path, data, wanted) in [
        (
            "/repository/conda-hosted/main/linux-64/other-2.0-h1_0.conda",
            tool.clone(),
            "does not match the package tool-2.0-h1_0",
        ),
        (
            "/repository/conda-hosted/main/osx-64/tool-2.0-h1_0.conda",
            tool.clone(),
            "built for linux-64",
        ),
        (
            "/repository/conda-hosted/main/noarch/broken-1-0.tar.bz2",
            b"not a package".to_vec(),
            "Not a valid conda package",
        ),
        (
            "/repository/conda-hosted/main/noarch/repodata.json",
            b"{}".to_vec(),
            "generated",
        ),
        (
            "/repository/conda-hosted/main/readme.txt",
            b"text".to_vec(),
            "not the place of a conda package",
        ),
    ] {
        let refused = put(&fixture, path, data).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{path}");
        assert!(
            refused.text().contains(wanted),
            "{path}: {}",
            refused.text()
        );
    }

    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=conda-hosted",
            Auth::Admin,
            Body::from(form(
                &[("conda.channel", "main")],
                (
                    "conda.asset",
                    "upload.conda",
                    &testing::conda(&testing::index("hello", "1.1", "py_0", "noarch"), None),
                ),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());

    let noarch = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/repodata.json",
    )
    .await;
    assert_eq!(noarch.status, StatusCode::OK);
    let document = noarch.json();
    assert_eq!(document["info"]["subdir"], "noarch");
    assert_eq!(document["repodata_version"], 1);
    assert_eq!(files(&document, "packages"), ["hello-1.0-py_0.tar.bz2"]);
    assert_eq!(files(&document, "packages.conda"), ["hello-1.1-py_0.conda"]);
    let record = &document["packages"]["hello-1.0-py_0.tar.bz2"];
    assert_eq!(record["name"], "hello");
    assert_eq!(record["build"], "py_0");
    assert_eq!(record["noarch"], "generic");
    assert_eq!(record["size"], hello.len());
    assert_eq!(record["sha256"], hex::encode(Sha256::digest(&hello)));
    assert_eq!(record["md5"].as_str().unwrap().len(), 32);
    let bz2 = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/repodata.json.bz2",
    )
    .await;
    assert_eq!(unbz2(&bz2.body), noarch.body);
    let zst = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/repodata.json.zst",
    )
    .await;
    assert_eq!(unzst(&zst.body), noarch.body);
    let current = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/current_repodata.json",
    )
    .await
    .json();
    assert!(files(&current, "packages").is_empty());
    assert_eq!(files(&current, "packages.conda"), ["hello-1.1-py_0.conda"]);
    let linux = get(
        &fixture,
        "/repository/conda-hosted/main/linux-64/repodata.json",
    )
    .await
    .json();
    assert_eq!(files(&linux, "packages.conda"), ["tool-2.0-h1_0.conda"]);

    let channeldata = get(&fixture, "/repository/conda-hosted/main/channeldata.json")
        .await
        .json();
    assert_eq!(channeldata["channeldata_version"], 1);
    assert_eq!(channeldata["subdirs"], json!(["linux-64", "noarch"]));
    assert_eq!(channeldata["packages"]["hello"]["version"], "1.1");
    assert_eq!(
        channeldata["packages"]["hello"]["subdirs"],
        json!(["noarch"])
    );
    assert_eq!(channeldata["packages"]["tool"]["license"], "MIT");

    let empty = get(
        &fixture,
        "/repository/conda-hosted/main/osx-arm64/repodata.json",
    )
    .await;
    assert_eq!(empty.status, StatusCode::OK);
    assert!(empty.json()["packages"].as_object().unwrap().is_empty());
    let empty_channel = get(
        &fixture,
        "/repository/conda-hosted/other/noarch/repodata.json",
    )
    .await;
    assert_eq!(empty_channel.status, StatusCode::OK);
    assert_eq!(
        get(
            &fixture,
            "/repository/conda-hosted/main/noarch/repodata.jlap"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let downloaded = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/hello-1.0-py_0.tar.bz2",
    )
    .await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(downloaded.body, hello);
    let page = get(&fixture, "/repository/conda-hosted/main/").await;
    assert!(
        page.text()
            .contains("conda install -c http://localhost/repository/conda-hosted/main")
    );

    let search = get(
        &fixture,
        "/service/rest/v1/search?repository=conda-hosted&conda.subdir=linux-64",
    )
    .await
    .json();
    let items = search["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "{search}");
    assert_eq!(items[0]["name"], "tool");
    assert_eq!(items[0]["group"], "main");
    assert_eq!(items[0]["version"], "2.0");
    assert_eq!(items[0]["assets"][0]["conda"]["build"], "h1_0");

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/conda-hosted/main/noarch/hello-1.1-py_0.conda",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let refused = fixture
        .call(
            Method::DELETE,
            "/repository/conda-hosted/main/noarch/repodata.json",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    let noarch = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/repodata.json",
    )
    .await
    .json();
    assert!(files(&noarch, "packages.conda").is_empty());
    let channeldata = get(&fixture, "/repository/conda-hosted/main/channeldata.json")
        .await
        .json();
    assert_eq!(channeldata["packages"]["hello"]["version"], "1.0");

    let found = get(
        &fixture,
        "/service/rest/v1/search/assets?repository=conda-hosted&name=tool",
    )
    .await
    .json();
    let tool_id = found["items"][0]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{found}"))
        .to_owned();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/assets/{tool_id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let linux = get(
        &fixture,
        "/repository/conda-hosted/main/linux-64/repodata.json",
    )
    .await
    .json();
    assert!(files(&linux, "packages.conda").is_empty());
    let channeldata = get(&fixture, "/repository/conda-hosted/main/channeldata.json")
        .await
        .json();
    assert_eq!(channeldata["subdirs"], json!(["noarch"]));
    let left = astrum_content::store::under(
        fixture.state.db().conn(),
        "conda-hosted",
        "/main/linux-64/",
        100,
    )
    .await
    .unwrap();
    assert!(left.is_empty(), "{left:?}");

    let repository = fixture
        .state
        .content()
        .repository("conda-hosted")
        .await
        .unwrap()
        .unwrap();
    let ctx = fixture.state.content().context(
        repository,
        Arc::new(SecurityModel::default()),
        None,
        "",
        "",
    );
    assert_eq!(astrum_format_conda::Conda.rebuild(&ctx).await.unwrap(), 1);
    let rebuilt = get(
        &fixture,
        "/repository/conda-hosted/main/noarch/repodata.json",
    )
    .await;
    assert_eq!(
        files(&rebuilt.json(), "packages"),
        ["hello-1.0-py_0.tar.bz2"]
    );
}

#[tokio::test]
async fn packages_at_the_root_and_permissions() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("root", "ALLOW")).await;
    let package = testing::conda(&testing::index("lib", "0.1", "0", "win-64"), None);
    let anonymous = fixture
        .send_body(
            Method::PUT,
            "/repository/root/win-64/lib-0.1-0.conda",
            Auth::None,
            Body::from(package.clone()),
            &[],
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        put(
            &fixture,
            "/repository/root/win-64/lib-0.1-0.conda",
            package.clone()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(&fixture, "/repository/root/win-64/lib-0.1-0.conda", package)
            .await
            .status,
        StatusCode::CREATED
    );
    let document = get(&fixture, "/repository/root/win-64/repodata.json")
        .await
        .json();
    assert_eq!(files(&document, "packages.conda"), ["lib-0.1-0.conda"]);
    let channeldata = get(&fixture, "/repository/root/channeldata.json")
        .await
        .json();
    assert_eq!(channeldata["subdirs"], json!(["win-64"]));
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=root",
            Auth::Admin,
            Body::from(form(
                &[("conda.channel", "../x")],
                ("conda.asset", "lib.conda", b"PK"),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::BAD_REQUEST, "{}", upload.text());
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<String>>>,
    package: Arc<Vec<u8>>,
}

async fn upstream_file(State(upstream): State<Upstream>, Path(path): Path<String>) -> Response {
    upstream.hits.lock().unwrap().push(path.clone());
    let record = |name: &str, version: &str| json!({"name": name, "version": version, "build": "0", "build_number": 0, "depends": [], "subdir": "noarch", "size": upstream.package.len(), "sha256": hex::encode(Sha256::digest(upstream.package.as_slice()))});
    match path.as_str() {
        "noarch/repodata.json" => json!({
            "info": {"subdir": "noarch"},
            "packages": {
                "far-1.0-0.tar.bz2": record("far", "1.0"),
                "shared-1.0-0.tar.bz2": record("shared", "upstream"),
            },
            "packages.conda": {"far-2.0-0.conda": record("far", "2.0")},
            "removed": [],
            "repodata_version": 1
        })
        .to_string()
        .into_response(),
        "channeldata.json" => json!({
            "channeldata_version": 1,
            "packages": {"far": {"version": "2.0", "subdirs": ["noarch"]}},
            "subdirs": ["noarch"]
        })
        .to_string()
        .into_response(),
        "noarch/far-1.0-0.tar.bz2" => upstream.package.as_ref().clone().into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::test]
async fn proxies_cache_channels_and_groups_merge_their_members() {
    let fixture = ready().await;
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
    let upstream = Upstream {
        package: Arc::new(testing::tar_bz2(
            &testing::index("far", "1.0", "0", "noarch"),
            None,
        )),
        ..Default::default()
    };
    let app = Router::new()
        .route("/channel/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "forge",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/channel/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "forge"]},
        }),
    )
    .await;

    let proxied = get(&fixture, "/repository/forge/noarch/repodata.json").await;
    assert_eq!(proxied.status, StatusCode::OK, "{}", proxied.text());
    assert_eq!(
        files(&proxied.json(), "packages.conda"),
        ["far-2.0-0.conda"]
    );
    let package = get(&fixture, "/repository/forge/noarch/far-1.0-0.tar.bz2").await;
    assert_eq!(package.body, *upstream.package);
    let again = get(&fixture, "/repository/forge/noarch/far-1.0-0.tar.bz2").await;
    assert_eq!(again.status, StatusCode::OK);
    let hits = upstream.hits.lock().unwrap().clone();
    assert_eq!(
        hits.iter()
            .filter(|hit| hit.as_str() == "noarch/far-1.0-0.tar.bz2")
            .count(),
        1,
        "{hits:?}"
    );
    let cached = get(
        &fixture,
        "/service/rest/v1/search?repository=forge&name=far",
    )
    .await
    .json();
    assert_eq!(cached["items"][0]["version"], "1.0", "{cached}");
    assert_eq!(cached["items"][0]["assets"][0]["conda"]["subdir"], "noarch");
    assert_eq!(
        get(&fixture, "/repository/forge/noarch/repodata.json.zst")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let only_own = get(&fixture, "/repository/all/linux-64/repodata.json").await;
    assert_eq!(only_own.status, StatusCode::OK);
    assert!(only_own.json()["packages"].as_object().unwrap().is_empty());

    let shared = testing::tar_bz2(&testing::index("shared", "1.0", "0", "noarch"), None);
    assert_eq!(
        put(
            &fixture,
            "/repository/own/noarch/shared-1.0-0.tar.bz2",
            shared.clone()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let merged = get(&fixture, "/repository/all/noarch/repodata.json").await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    let document = merged.json();
    assert_eq!(
        files(&document, "packages"),
        ["far-1.0-0.tar.bz2", "shared-1.0-0.tar.bz2"]
    );
    assert_eq!(files(&document, "packages.conda"), ["far-2.0-0.conda"]);
    assert_eq!(
        document["packages"]["shared-1.0-0.tar.bz2"]["sha256"],
        hex::encode(Sha256::digest(&shared))
    );
    let zst = get(&fixture, "/repository/all/noarch/repodata.json.zst").await;
    assert_eq!(unzst(&zst.body), merged.body);
    let bz2 = get(&fixture, "/repository/all/noarch/repodata.json.bz2").await;
    assert_eq!(unbz2(&bz2.body), merged.body);
    let current = get(&fixture, "/repository/all/noarch/current_repodata.json")
        .await
        .json();
    assert_eq!(files(&current, "packages.conda"), ["far-2.0-0.conda"]);
    assert_eq!(files(&current, "packages"), ["shared-1.0-0.tar.bz2"]);
    let channeldata = get(&fixture, "/repository/all/channeldata.json")
        .await
        .json();
    assert_eq!(channeldata["packages"]["far"]["version"], "2.0");
    assert_eq!(channeldata["packages"]["shared"]["version"], "1.0");

    let through = get(&fixture, "/repository/all/noarch/far-1.0-0.tar.bz2").await;
    assert_eq!(through.body, *upstream.package);
    let own = get(&fixture, "/repository/all/noarch/shared-1.0-0.tar.bz2").await;
    assert_eq!(own.body, shared);

    let second = testing::conda(&testing::index("shared", "1.1", "0", "noarch"), None);
    assert_eq!(
        put(
            &fixture,
            "/repository/own/noarch/shared-1.1-0.conda",
            second
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let remerged = get(&fixture, "/repository/all/noarch/repodata.json")
        .await
        .json();
    assert_eq!(
        files(&remerged, "packages.conda"),
        ["far-2.0-0.conda", "shared-1.1-0.conda"]
    );
    let refused = fixture
        .send_body(
            Method::PUT,
            "/repository/all/noarch/x-1-0.conda",
            Auth::Admin,
            Body::from(vec![1, 2, 3]),
            &[],
        )
        .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}
