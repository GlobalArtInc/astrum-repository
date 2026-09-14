mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_format_pub::testing;
use astrum_security::Scope;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "dart-http-boundary-0123456789";
const V2: &str = "application/vnd.pub.v2+json";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_pub::Pub));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/pub/{kind}"),
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
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy},
    })
}

fn package(name: &str, version: &str, extra: &str) -> Vec<u8> {
    testing::archive(
        &format!(
            "name: {name}\nversion: {version}\ndescription: The {name} package for tests.\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n{extra}"
        ),
        &[
            ("lib/main.dart", format!("// {name} {version}\n").as_bytes()),
            ("README.md", b"# test\n"),
        ],
    )
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn form(field: &str, file: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    write!(
        body,
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"package.tar.gz\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    fixture
        .send(Method::GET, path, auth, None, &[("accept", V2)])
        .await
}

async fn get_bearer(fixture: &Fixture, path: &str, token: &str) -> Reply {
    let bearer = format!("Bearer {token}");
    fixture
        .send(
            Method::GET,
            path,
            Auth::None,
            None,
            &[("accept", V2), ("authorization", &bearer)],
        )
        .await
}

fn local(location: &str) -> String {
    location
        .strip_prefix("http://localhost")
        .unwrap_or(location)
        .to_owned()
}

struct Publish {
    upload: Reply,
    finalize: Option<Reply>,
}

async fn publish(fixture: &Fixture, repository: &str, auth: Auth<'_>, archive: &[u8]) -> Publish {
    let (user, password) = match auth {
        Auth::Admin => ("admin".to_owned(), fixture.admin_password.clone()),
        Auth::Basic(user, password) => (user.to_owned(), password.to_owned()),
        Auth::None => (String::new(), String::new()),
    };
    let auth = || match user.is_empty() {
        true => Auth::None,
        false => Auth::Basic(&user, &password),
    };
    let new = get(
        fixture,
        &format!("/repository/{repository}/api/packages/versions/new"),
        auth(),
    )
    .await;
    assert_eq!(new.status, StatusCode::OK, "{}", new.text());
    assert_eq!(new.headers["content-type"], V2);
    let parameters = new.json();
    assert_eq!(parameters["fields"], json!({}));
    let url = local(parameters["url"].as_str().unwrap());
    assert_eq!(
        url,
        format!("/repository/{repository}/api/packages/versions/upload")
    );
    let upload = fixture
        .send_body(
            Method::POST,
            &url,
            auth(),
            Body::from(form("file", archive)),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    let finalize = match upload.headers.get("location") {
        Some(location) => Some(get(fixture, &local(location.to_str().unwrap()), auth()).await),
        None => None,
    };
    Publish { upload, finalize }
}

fn versions(listing: &Value) -> Vec<String> {
    listing["versions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["version"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn hosted_repositories_take_dart_pub_publish_and_list_the_versions() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("dart", "ALLOW_ONCE")).await;
    let first = package("hello", "1.0.0", "dependencies:\n  path: ^1.8.0\n");
    let second = package("hello", "1.1.0-dev.1", "");
    let third = package("hello", "1.0.1", "");

    let missing = get(&fixture, "/repository/dart/api/packages/hello", Auth::Admin).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.headers["content-type"], V2);
    assert_eq!(missing.json()["error"]["code"], "NotFound");

    for archive in [&first, &second, &third] {
        let done = publish(&fixture, "dart", Auth::Admin, archive).await;
        assert_eq!(
            done.upload.status,
            StatusCode::NO_CONTENT,
            "{}",
            done.upload.text()
        );
        let finalize = done.finalize.unwrap();
        assert_eq!(finalize.status, StatusCode::OK, "{}", finalize.text());
        assert!(
            finalize.json()["success"]["message"]
                .as_str()
                .unwrap()
                .starts_with("Successfully uploaded package hello version "),
            "{}",
            finalize.text()
        );
    }

    let listing = get(&fixture, "/repository/dart/api/packages/hello", Auth::Admin).await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    assert_eq!(listing.headers["content-type"], V2);
    let document = listing.json();
    assert_eq!(document["name"], "hello");
    assert_eq!(versions(&document), ["1.0.0", "1.0.1", "1.1.0-dev.1"]);
    assert_eq!(document["latest"]["version"], "1.0.1");
    assert_eq!(
        document["versions"][0]["archive_url"],
        "http://localhost/repository/dart/packages/hello/versions/1.0.0.tar.gz"
    );
    assert_eq!(document["versions"][0]["archive_sha256"], sha256(&first));
    assert_eq!(
        document["versions"][0]["pubspec"]["dependencies"],
        json!({"path": "^1.8.0"})
    );
    assert!(
        document["versions"][0]["published"]
            .as_str()
            .unwrap()
            .ends_with('Z')
    );
    let etag = listing.headers["etag"].to_str().unwrap().to_owned();
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/dart/api/packages/hello",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let one = get(
        &fixture,
        "/repository/dart/api/packages/hello/versions/1.1.0-dev.1",
        Auth::Admin,
    )
    .await;
    assert_eq!(one.status, StatusCode::OK);
    assert_eq!(one.json()["archive_sha256"], sha256(&second));
    let none = get(
        &fixture,
        "/repository/dart/api/packages/hello/versions/9.0.0",
        Auth::Admin,
    )
    .await;
    assert_eq!(none.status, StatusCode::NOT_FOUND);

    for path in [
        "/repository/dart/packages/hello/versions/1.0.0.tar.gz",
        "/repository/dart/api/archives/hello-1.0.0.tar.gz",
    ] {
        let download = get(&fixture, path, Auth::Admin).await;
        assert_eq!(download.status, StatusCode::OK, "{path}");
        assert_eq!(download.body, first);
        assert_eq!(download.headers["content-type"], "application/gzip");
    }

    let again = publish(&fixture, "dart", Auth::Admin, &first).await;
    assert_eq!(again.upload.status, StatusCode::NO_CONTENT);
    let again = again.finalize.unwrap();
    assert_eq!(again.status, StatusCode::OK);
    assert!(
        again
            .text()
            .contains("already in dart with the same archive")
    );

    let changed = publish(
        &fixture,
        "dart",
        Auth::Admin,
        &package("hello", "1.0.0", "homepage: https://example.org\n"),
    )
    .await;
    assert_eq!(changed.upload.status, StatusCode::NO_CONTENT);
    let refused = changed.finalize.unwrap();
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.json()["error"]["code"], "PackageRejected");
    assert_eq!(
        refused.json()["error"]["message"],
        "version 1.0.0 of package hello already exists in dart, which does not allow redeployment"
    );

    for (archive, message) in [
        (testing::archive("name: hello\n", &[]), "no \"version\""),
        (b"not a package".to_vec(), "gzip"),
        (package("Hello", "1.0.0", ""), "lower case"),
        (crate_like(), "no pubspec.yaml"),
    ] {
        let done = publish(&fixture, "dart", Auth::Admin, &archive).await;
        let finalize = done.finalize.unwrap();
        assert_eq!(finalize.status, StatusCode::BAD_REQUEST);
        let text = finalize.json()["error"]["message"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(text.contains(message), "{text}");
    }

    let forged = get(
        &fixture,
        "/repository/dart/api/packages/versions/finalize?upload=astrum%3Av1%3AAAAA",
        Auth::Admin,
    )
    .await;
    assert_eq!(forged.status, StatusCode::BAD_REQUEST);
    assert_eq!(forged.headers["content-type"], V2);

    let completion = get(
        &fixture,
        "/repository/dart/api/package-name-completion-data",
        Auth::Admin,
    )
    .await;
    assert_eq!(completion.json(), json!({"packages": ["hello"]}));
    let advisories = get(
        &fixture,
        "/repository/dart/api/packages/hello/advisories",
        Auth::Admin,
    )
    .await;
    assert_eq!(advisories.json(), json!({"advisories": []}));

    let search = get(
        &fixture,
        "/service/rest/v1/search?repository=dart&name=hello&version=1.0.0",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(search["items"][0]["format"], "pub");
    assert_eq!(
        search["items"][0]["assets"][0]["path"],
        "/packages/hello/versions/1.0.0.tar.gz"
    );
    assert_eq!(
        search["items"][0]["assets"][0]["pub"]["description"],
        "The hello package for tests."
    );
    assert_eq!(
        search["items"][0]["assets"][0]["pub"]["sdk"],
        ">=3.0.0 <4.0.0"
    );
    let by_words = get(
        &fixture,
        "/service/rest/v1/search?q=tests&format=pub",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(by_words["items"].as_array().unwrap().len(), 3);

    let browse = get(
        &fixture,
        "/service/rest/internal/ui/browse/dart?path=/hello/",
        Auth::Admin,
    )
    .await;
    assert_eq!(browse.status, StatusCode::OK, "{}", browse.text());
    let children: Vec<(String, String)> = browse.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        children,
        [
            ("component".to_owned(), "/hello/1.0.0/".to_owned()),
            ("component".to_owned(), "/hello/1.0.1/".to_owned()),
            ("component".to_owned(), "/hello/1.1.0-dev.1/".to_owned()),
        ]
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/dart/packages/hello/versions/1.0.1.tar.gz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let listing = get(&fixture, "/repository/dart/api/packages/hello", Auth::Admin)
        .await
        .json();
    assert_eq!(versions(&listing), ["1.0.0", "1.1.0-dev.1"]);
    assert_eq!(listing["latest"]["version"], "1.0.0");
}

fn crate_like() -> Vec<u8> {
    astrum_format_pub::archive::build(&[("hello/pubspec.yaml", b"name: hello\nversion: 1.0.0\n")])
}

#[tokio::test]
async fn tokens_challenges_and_privileges_follow_the_dart_client() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("closed", "ALLOW")).await;
    let archive = package("secret", "2.0.0", "");
    let done = publish(&fixture, "closed", Auth::Admin, &archive).await;
    assert_eq!(done.finalize.unwrap().status, StatusCode::OK);

    let anonymous = get(
        &fixture,
        "/repository/closed/api/packages/secret",
        Auth::None,
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let challenges: Vec<&str> = anonymous
        .headers
        .get_all("www-authenticate")
        .iter()
        .map(|value| value.to_str().unwrap())
        .collect();
    assert_eq!(challenges.len(), 2, "{challenges:?}");
    assert_eq!(
        challenges[0],
        "Bearer realm=\"pub\", message=\"Create an access token in Astrum Repository at http://localhost/user/access-tokens and add it with: dart pub token add http://localhost/repository/closed\""
    );
    assert_eq!(challenges[1], "Basic realm=\"Astrum Repository\"");
    assert_eq!(anonymous.json()["error"]["code"], "MissingAuthentication");
    let new = get(
        &fixture,
        "/repository/closed/api/packages/versions/new",
        Auth::None,
    )
    .await;
    assert_eq!(new.status, StatusCode::UNAUTHORIZED);
    let ui = fixture
        .send(
            Method::GET,
            "/repository/closed/api/packages/secret",
            Auth::None,
            None,
            &[("x-astrum-ui", "true")],
        )
        .await;
    assert_eq!(ui.status, StatusCode::UNAUTHORIZED);
    assert!(ui.headers.get("www-authenticate").is_none());

    let (_, token) = fixture
        .state
        .security()
        .create_token("admin", "dart", Scope::Write, None)
        .await
        .unwrap();
    let listed = get_bearer(&fixture, "/repository/closed/api/packages/secret", &token).await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.text());
    let download = get_bearer(
        &fixture,
        "/repository/closed/packages/secret/versions/2.0.0.tar.gz",
        &token,
    )
    .await;
    assert_eq!(download.body, archive);
    let revoked = get_bearer(
        &fixture,
        "/repository/closed/api/packages/secret",
        "astrum_0000000000000000000000000000000000000000",
    )
    .await;
    assert_eq!(revoked.status, StatusCode::UNAUTHORIZED);

    fixture.add_user("reader", "reader-password", &[]).await;
    let stranger = get(
        &fixture,
        "/repository/closed/api/packages/secret",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(stranger.status, StatusCode::FORBIDDEN);
    let challenge = stranger.headers["www-authenticate"].to_str().unwrap();
    assert!(
        challenge.starts_with("Bearer realm=\"pub\", message=\"Your user lacks the privileges"),
        "{challenge}"
    );
    assert_eq!(stranger.json()["error"]["code"], "InsufficientPermissions");
    let refused = get(
        &fixture,
        "/repository/closed/api/packages/versions/new",
        Auth::Basic("reader", "reader-password"),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "pub-readers", "name": "pub-readers", "description": "", "privileges": ["as-repository-view-pub-closed-read", "as-repository-view-pub-closed-browse"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("consumer", "consumer-password", &["pub-readers"])
        .await;
    let read = get(
        &fixture,
        "/repository/closed/api/packages/secret",
        Auth::Basic("consumer", "consumer-password"),
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{}", read.text());
    let new = get(
        &fixture,
        "/repository/closed/api/packages/versions/new",
        Auth::Basic("consumer", "consumer-password"),
    )
    .await;
    assert_eq!(new.status, StatusCode::FORBIDDEN);
    let upload = fixture
        .send_body(
            Method::POST,
            "/repository/closed/api/packages/versions/upload",
            Auth::Basic("consumer", "consumer-password"),
            Body::from(form("file", &package("other", "1.0.0", ""))),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::FORBIDDEN);

    let republished = publish(
        &fixture,
        "closed",
        Auth::Admin,
        &package("secret", "2.0.0", "homepage: https://example.org\n"),
    )
    .await;
    assert_eq!(republished.finalize.unwrap().status, StatusCode::OK);
    let listing = get(
        &fixture,
        "/repository/closed/api/packages/secret",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        listing["latest"]["pubspec"]["homepage"],
        "https://example.org"
    );
}

#[tokio::test]
async fn versions_can_be_retracted_and_packages_discontinued() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("flags", "ALLOW_ONCE")).await;
    for version in ["1.0.0", "1.1.0"] {
        let done = publish(
            &fixture,
            "flags",
            Auth::Admin,
            &package("lib_a", version, ""),
        )
        .await;
        assert_eq!(done.finalize.unwrap().status, StatusCode::OK);
    }
    let retract = fixture
        .call(
            Method::PUT,
            "/repository/flags/api/packages/lib_a/versions/1.1.0/options",
            Auth::Admin,
            Some(json!({"isRetracted": true})),
        )
        .await;
    assert_eq!(retract.status, StatusCode::OK, "{}", retract.text());
    assert_eq!(retract.json(), json!({"isRetracted": true}));
    let listing = get(
        &fixture,
        "/repository/flags/api/packages/lib_a",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(listing["latest"]["version"], "1.0.0");
    assert_eq!(listing["versions"][1]["retracted"], true);
    let options = get(
        &fixture,
        "/repository/flags/api/packages/lib_a/versions/1.1.0/options",
        Auth::Admin,
    )
    .await;
    assert_eq!(options.json(), json!({"isRetracted": true}));

    let discontinue = fixture
        .call(
            Method::PUT,
            "/repository/flags/api/packages/lib_a/options",
            Auth::Admin,
            Some(json!({"isDiscontinued": true, "replacedBy": "lib_b"})),
        )
        .await;
    assert_eq!(discontinue.status, StatusCode::OK, "{}", discontinue.text());
    let done = publish(
        &fixture,
        "flags",
        Auth::Admin,
        &package("lib_a", "1.2.0", ""),
    )
    .await;
    assert_eq!(done.finalize.unwrap().status, StatusCode::OK);
    let listing = get(
        &fixture,
        "/repository/flags/api/packages/lib_a",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(listing["isDiscontinued"], true);
    assert_eq!(listing["replacedBy"], "lib_b");
    assert_eq!(listing["latest"]["version"], "1.2.0");
    let bad = fixture
        .call(
            Method::PUT,
            "/repository/flags/api/packages/lib_a/options",
            Auth::Admin,
            Some(json!({"isDiscontinued": true, "replacedBy": "lib_a"})),
        )
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    let anonymous = fixture
        .call(
            Method::PUT,
            "/repository/flags/api/packages/lib_a/options",
            Auth::None,
            Some(json!({"isDiscontinued": false})),
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let back = fixture
        .call(
            Method::PUT,
            "/repository/flags/api/packages/lib_a/options",
            Auth::Admin,
            Some(json!({"isDiscontinued": false})),
        )
        .await;
    assert_eq!(back.json()["isDiscontinued"], false);
    let listing = get(
        &fixture,
        "/repository/flags/api/packages/lib_a",
        Auth::Admin,
    )
    .await
    .json();
    assert!(listing.get("isDiscontinued").is_none());
    assert!(listing.get("replacedBy").is_none());
}

#[tokio::test]
async fn the_components_api_takes_package_archives() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("uploads", "ALLOW_ONCE")).await;
    let spec = get(
        &fixture,
        "/service/rest/v1/formats/pub/upload-specs",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(spec["assetFields"][0]["name"], "asset");
    let archive = package("uploaded", "0.3.0+4", "");
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=uploads",
            Auth::Admin,
            Body::from(form("pub.asset", &archive)),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());
    let listing = get(
        &fixture,
        "/repository/uploads/api/packages/uploaded",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions(&listing), ["0.3.0+4"]);
    assert_eq!(
        listing["latest"]["archive_url"],
        "http://localhost/repository/uploads/packages/uploaded/versions/0.3.0+4.tar.gz"
    );
    let download = get(
        &fixture,
        "/repository/uploads/packages/uploaded/versions/0.3.0+4.tar.gz",
        Auth::Admin,
    )
    .await;
    assert_eq!(download.body, archive);
    let duplicate = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=uploads",
            Auth::Admin,
            Body::from(form(
                "pub.asset",
                &package("uploaded", "0.3.0+4", "homepage: https://x.example\n"),
            )),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(
        duplicate.status,
        StatusCode::BAD_REQUEST,
        "{}",
        duplicate.text()
    );
    let broken = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=uploads",
            Auth::Admin,
            Body::from(form("pub.asset", b"plain text")),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
}

#[derive(Clone, Default)]
struct Upstream {
    hits: Arc<Mutex<Vec<(String, String)>>>,
    address: Arc<Mutex<String>>,
    archive: Arc<Vec<u8>>,
}

async fn upstream_file(
    State(upstream): State<Upstream>,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> Response {
    let accept = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    upstream.hits.lock().unwrap().push((path.clone(), accept));
    let base = upstream.address.lock().unwrap().clone();
    let entry = |version: &str, sha: String| {
        json!({
            "version": version,
            "archive_url": format!("http://{base}/storage/far-{version}.tar.gz"),
            "archive_sha256": sha,
            "pubspec": {"name": "far", "version": version, "description": "From far away."},
            "published": "2025-01-01T00:00:00.000Z"
        })
    };
    match path.as_str() {
        "api/packages/far" => json!({
            "name": "far",
            "advisoriesUpdated": "2025-02-01T00:00:00.000Z",
            "latest": entry("1.0.0", sha256(&upstream.archive)),
            "versions": [
                entry("1.0.0", sha256(&upstream.archive)),
                entry("0.9.0", "00".repeat(32)),
            ]
        })
        .to_string()
        .into_response(),
        "api/packages/shared" => json!({
            "name": "shared",
            "versions": [{"version": "5.0.0", "archive_url": format!("http://{base}/storage/shared-5.0.0.tar.gz"), "pubspec": {"name": "shared", "version": "5.0.0"}}]
        })
        .to_string()
        .into_response(),
        "api/packages/far/advisories" => json!({
            "advisories": [{"id": "GHSA-0000-0000-0000", "summary": "Far is far"}],
            "advisoriesUpdated": "2025-02-01T00:00:00.000Z"
        })
        .to_string()
        .into_response(),
        "api/package-name-completion-data" => {
            json!({"packages": ["far", "shared"]}).to_string().into_response()
        }
        "storage/far-1.0.0.tar.gz" | "storage/far-0.9.0.tar.gz" | "storage/shared-5.0.0.tar.gz" => {
            upstream.archive.as_ref().clone().into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[tokio::test]
async fn proxies_rewrite_archive_urls_and_groups_merge_their_members() {
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
        archive: Arc::new(package("far", "1.0.0", "")),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream_file))
        .with_state(upstream.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    *upstream.address.lock().unwrap() = address.to_string();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "pub-dev",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("own", "ALLOW_ONCE")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "pub-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["own", "pub-dev"]},
        }),
    )
    .await;
    let own = package("shared", "5.0.0", "");
    assert_eq!(
        publish(&fixture, "own", Auth::Admin, &own)
            .await
            .finalize
            .unwrap()
            .status,
        StatusCode::OK
    );

    let listing = get(
        &fixture,
        "/repository/pub-dev/api/packages/far",
        Auth::Admin,
    )
    .await;
    assert_eq!(listing.status, StatusCode::OK, "{}", listing.text());
    let document = listing.json();
    assert_eq!(versions(&document), ["0.9.0", "1.0.0"]);
    assert_eq!(
        document["latest"]["archive_url"],
        "http://localhost/repository/pub-dev/packages/far/versions/1.0.0.tar.gz"
    );
    assert_eq!(document["advisoriesUpdated"], "2025-02-01T00:00:00.000Z");
    assert_eq!(document["latest"]["published"], "2025-01-01T00:00:00.000Z");
    let hits = upstream.hits.lock().unwrap().clone();
    assert_eq!(hits[0], ("api/packages/far".to_owned(), V2.to_owned()));

    for _ in 0..2 {
        let archive = get(
            &fixture,
            "/repository/pub-dev/packages/far/versions/1.0.0.tar.gz",
            Auth::Admin,
        )
        .await;
        assert_eq!(archive.status, StatusCode::OK, "{}", archive.text());
        assert_eq!(archive.body, *upstream.archive);
    }
    let fetched = upstream
        .hits
        .lock()
        .unwrap()
        .iter()
        .filter(|(path, _)| path == "storage/far-1.0.0.tar.gz")
        .count();
    assert_eq!(fetched, 1);
    let mismatch = get(
        &fixture,
        "/repository/pub-dev/packages/far/versions/0.9.0.tar.gz",
        Auth::Admin,
    )
    .await;
    assert_eq!(
        mismatch.status,
        StatusCode::BAD_GATEWAY,
        "{}",
        mismatch.text()
    );
    let cached = get(
        &fixture,
        "/service/rest/v1/search?repository=pub-dev&name=far",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(cached["items"].as_array().unwrap().len(), 1, "{cached}");
    assert_eq!(cached["items"][0]["version"], "1.0.0");
    assert_eq!(
        cached["items"][0]["assets"][0]["pub"]["description"],
        "From far away."
    );

    let advisories = get(
        &fixture,
        "/repository/pub-dev/api/packages/far/advisories",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(advisories["advisories"][0]["id"], "GHSA-0000-0000-0000");

    let merged = get(
        &fixture,
        "/repository/pub-all/api/packages/shared",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions(&merged), ["5.0.0"]);
    assert_eq!(merged["latest"]["archive_sha256"], sha256(&own));
    assert_eq!(
        merged["latest"]["archive_url"],
        "http://localhost/repository/pub-all/packages/shared/versions/5.0.0.tar.gz"
    );
    let from_group = get(
        &fixture,
        "/repository/pub-all/packages/shared/versions/5.0.0.tar.gz",
        Auth::Admin,
    )
    .await;
    assert_eq!(from_group.body, own);
    let far = get(
        &fixture,
        "/repository/pub-all/api/packages/far",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(versions(&far), ["0.9.0", "1.0.0"]);
    let far_archive = get(
        &fixture,
        "/repository/pub-all/packages/far/versions/1.0.0.tar.gz",
        Auth::Admin,
    )
    .await;
    assert_eq!(far_archive.body, *upstream.archive);
    let group_advisories = get(
        &fixture,
        "/repository/pub-all/api/packages/far/advisories",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        group_advisories["advisoriesUpdated"],
        "2025-02-01T00:00:00.000Z"
    );
    assert_eq!(group_advisories["advisories"].as_array().unwrap().len(), 1);
    let names = get(
        &fixture,
        "/repository/pub-all/api/package-name-completion-data",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(names, json!({"packages": ["far", "shared"]}));
    let new = get(
        &fixture,
        "/repository/pub-all/api/packages/versions/new",
        Auth::Admin,
    )
    .await;
    assert_eq!(new.status, StatusCode::BAD_REQUEST);
    assert!(
        new.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("hosted")
    );
    let missing = get(
        &fixture,
        "/repository/pub-all/api/packages/nothing",
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_rest_api_creates_and_checks_pub_repositories() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("pub-hosted", "ALLOW_ONCE")).await;
    let stored = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories/pub/hosted/pub-hosted",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(stored.status, StatusCode::OK);
    let document = stored.json();
    assert_eq!(document["format"], "pub");
    assert_eq!(document["type"], "hosted");
    assert_eq!(document["url"], "http://localhost/repository/pub-hosted");
    let mut changed = document.clone();
    changed["storage"]["writePolicy"] = json!("DENY");
    let update = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/pub/hosted/pub-hosted",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(update.status, StatusCode::NO_CONTENT, "{}", update.text());
    let done = publish(
        &fixture,
        "pub-hosted",
        Auth::Admin,
        &package("ro", "1.0.0", ""),
    )
    .await;
    let finalize = done.finalize.unwrap();
    assert_eq!(finalize.status, StatusCode::BAD_REQUEST);
    assert!(finalize.text().contains("read-only"));

    let proxy = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/pub/proxy",
            Auth::Admin,
            Some(json!({
                "name": "no-remote",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "negativeCache": {"enabled": true, "timeToLive": 1440},
                "httpClient": {"blocked": false, "autoBlock": true},
            })),
        )
        .await;
    assert_eq!(proxy.status, StatusCode::BAD_REQUEST);
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(hosted("files", "ALLOW")),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let group = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/pub/group",
            Auth::Admin,
            Some(json!({
                "name": "mixed",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["pub-hosted", "files"]},
            })),
        )
        .await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST, "{}", group.text());
    let listed = fixture
        .call(
            Method::GET,
            "/service/rest/v1/repositories",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|one| one["name"] == "pub-hosted" && one["format"] == "pub")
    );
}

#[tokio::test]
async fn cleanup_tells_pre_releases_from_releases() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("clean", "ALLOW_ONCE")).await;
    for version in ["1.0.0", "1.1.0-dev.1", "1.0.0+build-7", "2.0.0-beta+1"] {
        let done = publish(
            &fixture,
            "clean",
            Auth::Admin,
            &package("tidy", version, ""),
        )
        .await;
        assert_eq!(done.finalize.unwrap().status, StatusCode::OK);
    }
    let preview = |release: &'static str| {
        let fixture = &fixture;
        async move {
            let reply = fixture
                .call(
                    Method::POST,
                    "/service/rest/internal/cleanup-policies/preview/summary",
                    Auth::Admin,
                    Some(json!({"name": "pre", "format": "pub", "criteriaReleaseType": release, "repository": "clean"})),
                )
                .await;
            assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
            reply.json()["componentCount"].clone()
        }
    };
    assert_eq!(preview("PRERELEASES").await, 2);
    assert_eq!(preview("RELEASES").await, 2);
}
