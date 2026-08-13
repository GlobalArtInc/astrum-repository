mod support;

use std::sync::Arc;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const RECIPE: &str = "/v2/conans/hello/1.0.0/_/_";
const RREV: &str = "30918f2669fd83ad91770cc857ca5f69";
const PKG: &str = "63fead0844576fc02943e16909f08fcdddd6f44b";
const PREV: &str = "62fca0bfb7d150a008f36f90514fb34f";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_conan::Conan));
    fixture
}

fn app(fixture: &Fixture) -> axum::Router {
    astrum_api::router_with(fixture.state.clone(), astrum_format_conan::routes())
}

async fn call(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    body: Vec<u8>,
    headers: &[(&str, &str)],
) -> Reply {
    use base64::Engine;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let mut request = axum::http::Request::builder().method(method).uri(path);
    let credentials = match auth {
        Auth::None => None,
        Auth::Admin => Some(("admin".to_owned(), fixture.admin_password.clone())),
        Auth::Basic(user, password) => Some((user.to_owned(), password.to_owned())),
    };
    if let Some((user, password)) = credentials {
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"));
        request = request.header("authorization", format!("Basic {encoded}"));
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = app(fixture)
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Reply {
        status,
        headers,
        body,
    }
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/conan/{kind}"),
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW_ONCE"}
    })
}

async fn upload(fixture: &Fixture, repository: &str) {
    let files: Vec<(String, Vec<u8>)> = vec![
        (
            format!("{RECIPE}/revisions/{RREV}/files/conanfile.py"),
            b"from conan import ConanFile\n".to_vec(),
        ),
        (
            format!("{RECIPE}/revisions/{RREV}/files/conanmanifest.txt"),
            b"123\nconanfile.py: abc\n".to_vec(),
        ),
        (
            format!(
                "{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions/{PREV}/files/conaninfo.txt"
            ),
            b"[settings]\narch=x86_64\nos=Linux\n".to_vec(),
        ),
        (
            format!(
                "{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions/{PREV}/files/conan_package.tgz"
            ),
            vec![0x1f, 0x8b, 8, 0],
        ),
    ];
    for (path, body) in files {
        let reply = call(
            fixture,
            Method::PUT,
            &format!("/repository/{repository}{path}"),
            Auth::Admin,
            body,
            &[],
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{path}: {}",
            reply.text()
        );
    }
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    call(fixture, Method::GET, path, Auth::Admin, Vec::new(), &[]).await
}

#[tokio::test]
async fn hosted_recipes_answer_the_conan_v2_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("conan")).await;
    let ping = call(
        &fixture,
        Method::GET,
        "/repository/conan/v1/ping",
        Auth::None,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(ping.status, StatusCode::OK);
    assert_eq!(ping.headers["x-conan-server-capabilities"], "revisions");
    upload(&fixture, "conan").await;
    let again = call(
        &fixture,
        Method::PUT,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/files/conanfile.py"),
        Auth::Admin,
        b"other".to_vec(),
        &[],
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    let deploy = call(
        &fixture,
        Method::PUT,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/files/conan_sources.tgz"),
        Auth::Admin,
        Vec::new(),
        &[("x-checksum-deploy", "true"), ("x-checksum-sha1", "0000")],
    )
    .await;
    assert_eq!(deploy.status, StatusCode::NOT_FOUND);

    let revisions = get(&fixture, &format!("/repository/conan{RECIPE}/revisions"))
        .await
        .json();
    assert_eq!(revisions["reference"], "hello/1.0.0@_/_");
    assert_eq!(revisions["revisions"][0]["revision"], RREV);
    assert!(
        revisions["revisions"][0]["time"]
            .as_str()
            .unwrap()
            .ends_with("+0000")
    );
    let latest = get(&fixture, &format!("/repository/conan{RECIPE}/latest"))
        .await
        .json();
    assert_eq!(latest["revision"], RREV);
    let files = get(
        &fixture,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/files"),
    )
    .await
    .json();
    assert_eq!(
        files,
        json!({"files": {"conanfile.py": {}, "conanmanifest.txt": {}}})
    );
    let file = get(
        &fixture,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/files/conanfile.py"),
    )
    .await;
    assert_eq!(file.body, b"from conan import ConanFile\n");
    let package_latest = get(
        &fixture,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/packages/{PKG}/latest"),
    )
    .await
    .json();
    assert_eq!(package_latest["revision"], PREV);
    let package_revisions = get(
        &fixture,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions"),
    )
    .await
    .json();
    assert_eq!(
        package_revisions["reference"],
        format!("hello/1.0.0@_/_#{RREV}:{PKG}")
    );
    let package_files = get(
        &fixture,
        &format!(
            "/repository/conan{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions/{PREV}/files"
        ),
    )
    .await
    .json();
    assert_eq!(
        package_files,
        json!({"files": {"conan_package.tgz": {}, "conaninfo.txt": {}}})
    );
    let packages = get(
        &fixture,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/search"),
    )
    .await
    .json();
    assert_eq!(
        packages[PKG]["settings"],
        json!({"arch": "x86_64", "os": "Linux"})
    );
    let latest_packages = get(&fixture, &format!("/repository/conan{RECIPE}/search"))
        .await
        .json();
    assert_eq!(latest_packages, packages);
    let search = get(&fixture, "/repository/conan/v2/conans/search?q=hel*")
        .await
        .json();
    assert_eq!(search, json!({"results": ["hello/1.0.0@_/_"]}));
    assert_eq!(
        get(
            &fixture,
            "/repository/conan/v2/conans/missing/1.0/_/_/latest"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let components = fixture
        .call(
            Method::GET,
            "/service/rest/v1/components?repository=conan",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(components["items"][0]["name"], "hello");
    assert_eq!(components["items"][0]["group"], "");
    assert_eq!(
        components["items"][0]["assets"].as_array().unwrap().len(),
        4
    );

    let removed = call(
        &fixture,
        Method::DELETE,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions/{PREV}"),
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(removed.status, StatusCode::OK);
    assert_eq!(
        get(
            &fixture,
            &format!("/repository/conan{RECIPE}/revisions/{RREV}/packages/{PKG}/latest")
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let removed = call(
        &fixture,
        Method::DELETE,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}"),
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(removed.status, StatusCode::OK);
    assert_eq!(
        get(&fixture, &format!("/repository/conan{RECIPE}/latest"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn tokens_come_from_the_authenticate_endpoint() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("conan")).await;
    let anonymous = call(
        &fixture,
        Method::GET,
        "/repository/conan/v2/users/authenticate",
        Auth::None,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let inactive = call(
        &fixture,
        Method::GET,
        "/repository/conan/v2/users/authenticate",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(inactive.status, StatusCode::FORBIDDEN);
    assert!(
        inactive
            .text()
            .contains("Conan Bearer Token Realm is not active")
    );
    let reply = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!(["AstrumAuthenticatingRealm", "ConanToken"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let token = call(
        &fixture,
        Method::GET,
        "/repository/conan/v2/users/authenticate",
        Auth::Admin,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(token.status, StatusCode::OK);
    let token = token.text();
    assert!(token.starts_with("ConanToken."));
    let bearer = format!("Bearer {token}");
    let checked = call(
        &fixture,
        Method::GET,
        "/repository/conan/v2/users/check_credentials",
        Auth::None,
        Vec::new(),
        &[("authorization", &bearer)],
    )
    .await;
    assert_eq!(checked.status, StatusCode::OK);
    assert_eq!(checked.text(), "admin");
    let refused = call(
        &fixture,
        Method::GET,
        "/repository/conan/v2/users/check_credentials",
        Auth::None,
        Vec::new(),
        &[],
    )
    .await;
    assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
    let put = call(
        &fixture,
        Method::PUT,
        &format!("/repository/conan{RECIPE}/revisions/{RREV}/files/conanfile.py"),
        Auth::None,
        b"x".to_vec(),
        &[("authorization", &bearer)],
    )
    .await;
    assert_eq!(put.status, StatusCode::CREATED);
}

#[tokio::test]
async fn proxies_cache_a_remote_and_groups_merge_members() {
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
    let remote = ready().await;
    create(&remote, "hosted", hosted("origin")).await;
    upload(&remote, "origin").await;
    let anonymous = remote
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::OK);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let served = app(&remote);
    tokio::spawn(async move { axum::serve(listener, served).await.unwrap() });

    create(
        &fixture,
        "proxy",
        json!({
            "name": "center",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/repository/origin"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
            "conanProxy": {"conanVersion": "V2"},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "center"]},
        }),
    )
    .await;
    let latest = get(&fixture, &format!("/repository/center{RECIPE}/latest")).await;
    assert_eq!(latest.status, StatusCode::OK, "{}", latest.text());
    assert_eq!(latest.json()["revision"], RREV);
    let package = get(&fixture, &format!("/repository/center{RECIPE}/revisions/{RREV}/packages/{PKG}/revisions/{PREV}/files/conan_package.tgz")).await;
    assert_eq!(package.status, StatusCode::OK);
    assert_eq!(package.body, vec![0x1f, 0x8b, 8, 0]);
    let cached = fixture
        .call(
            Method::GET,
            "/service/rest/v1/components?repository=center",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(cached["items"][0]["name"], "hello");

    let other =
        "/v2/conans/hello/1.0.0/_/_/revisions/ffffffffffffffffffffffffffffffff/files/conanfile.py";
    let put = call(
        &fixture,
        Method::PUT,
        &format!("/repository/own{other}"),
        Auth::Admin,
        b"own".to_vec(),
        &[],
    )
    .await;
    assert_eq!(put.status, StatusCode::CREATED);
    let merged = get(&fixture, &format!("/repository/all{RECIPE}/revisions"))
        .await
        .json();
    let revisions: Vec<&str> = merged["revisions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["revision"].as_str().unwrap())
        .collect();
    assert_eq!(revisions.len(), 2);
    assert!(revisions.contains(&RREV));
    let through = get(
        &fixture,
        &format!("/repository/all{RECIPE}/revisions/{RREV}/files/conanfile.py"),
    )
    .await;
    assert_eq!(through.body, b"from conan import ConanFile\n");
    let search = get(&fixture, "/repository/all/v2/conans/search?q=hello*")
        .await
        .json();
    assert_eq!(search, json!({"results": ["hello/1.0.0@_/_"]}));
    let refused = call(
        &fixture,
        Method::PUT,
        &format!("/repository/all{other}"),
        Auth::Admin,
        b"x".to_vec(),
        &[],
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn conan_search_keys_find_assets_by_revision() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("conan")).await;
    upload(&fixture, "conan").await;
    let found = fixture
        .call(
            Method::GET,
            &format!(
                "/service/rest/v1/search/assets?repository=conan&conan.packageRevision={PREV}"
            ),
            Auth::Admin,
            None,
        )
        .await
        .json();
    let paths: Vec<&str> = found["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths.len(), 2, "{paths:?}");
    assert!(paths.iter().all(|path| path.contains("/package/")));
    let by_revision = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/search?conan.revision={RREV}&conan.channel=_"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(by_revision["items"].as_array().unwrap().len(), 1);
}
