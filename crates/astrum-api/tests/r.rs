mod support;

use std::io::{Read, Write};
use std::sync::Arc;

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use md5::Digest;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "rboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_r::R::default()));
    fixture
        .state
        .tasks()
        .register(Arc::new(astrum_format_r::task::RebuildMetadata::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/r/{kind}"),
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

fn description(name: &str, version: &str, extra: &str) -> String {
    format!(
        "Package: {name}\nType: Package\nTitle: A Test Package\nVersion: {version}\nAuthor: Test Author\nMaintainer: Test Author <test@example.org>\nDescription: A package of the API tests.\nLicense: MIT + file LICENSE\nEncoding: UTF-8\n{extra}"
    )
}

fn tar_of(folder: &str, text: &str) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (name, body) in [
        (format!("{folder}/DESCRIPTION"), text.to_owned()),
        (format!("{folder}/NAMESPACE"), "export(hello)\n".to_owned()),
        (
            format!("{folder}/R/hello.R"),
            "hello <- function() \"hi\"\n".to_owned(),
        ),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, name, body.as_bytes())
            .unwrap();
    }
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&builder.into_inner().unwrap()).unwrap();
    encoder.finish().unwrap()
}

fn source(name: &str, version: &str, extra: &str) -> Vec<u8> {
    tar_of(name, &description(name, version, extra))
}

fn windows(name: &str, version: &str) -> Vec<u8> {
    let text = description(
        name,
        version,
        "Depends: R (>= 4.0.0)\nBuilt: R 4.4.1; ; 2026-10-01 10:00:00 UTC; windows\n",
    );
    astrum_content::zip::write(&[
        (&format!("{name}/DESCRIPTION"), text.as_bytes()),
        (&format!("{name}/NAMESPACE"), b"export(hello)\n".as_slice()),
    ])
}

fn md5(bytes: &[u8]) -> String {
    hex::encode(md5::Md5::digest(bytes))
}

fn gunzip(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes)
        .read_to_end(&mut out)
        .unwrap();
    out
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

async fn put(fixture: &Fixture, path: &str, body: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            Auth::Admin,
            Body::from(body),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

fn form(fields: &[(&str, &str)], filename: &str, file: &[u8]) -> Vec<u8> {
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
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"r.asset\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .unwrap();
    body.extend_from_slice(file);
    write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn upload(fixture: &Fixture, repository: &str, body: Vec<u8>) -> Reply {
    fixture
        .send_body(
            Method::POST,
            &format!("/service/rest/v1/components?repository={repository}"),
            Auth::Admin,
            Body::from(body),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await
}

fn entries(text: &str) -> Vec<(String, String)> {
    astrum_format_r::dcf::parse(text)
        .into_iter()
        .map(|record| {
            (
                record.get("Package").unwrap().to_owned(),
                record.get("Version").unwrap().to_owned(),
            )
        })
        .collect()
}

#[tokio::test]
async fn hosted_repositories_generate_the_packages_files_of_each_directory() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("cran-own", "ALLOW_ONCE")).await;
    let hello_old = source("astrumhello", "1.0.0", "Depends: R (>= 3.5.0)\n");
    let hello = source(
        "astrumhello",
        "1.1.0",
        "Depends: R (>= 3.5.0)\nImports: utils\nSuggests: testthat\nNeedsCompilation: no\n",
    );
    let dep = source(
        "astrumdep",
        "0.2.1",
        "Imports:\n    astrumhello (>= 1.1.0)\nLinkingTo: Rcpp\nNeedsCompilation: no\n",
    );
    for (file, body) in [
        ("astrumhello_1.0.0.tar.gz", hello_old.clone()),
        ("astrumhello_1.1.0.tar.gz", hello.clone()),
        ("astrumdep_0.2.1.tar.gz", dep.clone()),
    ] {
        let reply = put(
            &fixture,
            &format!("/repository/cran-own/src/contrib/{file}"),
            body,
        )
        .await;
        assert_eq!(
            reply.status,
            StatusCode::CREATED,
            "{file}: {}",
            reply.text()
        );
    }
    let again = put(
        &fixture,
        "/repository/cran-own/src/contrib/astrumhello_1.0.0.tar.gz",
        hello_old.clone(),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    for (path, body, message) in [
        (
            "/repository/cran-own/src/contrib/astrumhello_2.0.0.tar.gz",
            hello.clone(),
            "the file name says astrumhello 2.0.0",
        ),
        (
            "/repository/cran-own/somewhere/astrumhello_1.1.0.tar.gz",
            hello.clone(),
            "R packages go to src/contrib/",
        ),
        (
            "/repository/cran-own/src/contrib/PACKAGES",
            b"Package: x".to_vec(),
            "are generated",
        ),
        (
            "/repository/cran-own/src/contrib/astrumhello_1.2.0.tar.gz",
            windows("astrumhello", "1.2.0"),
            "not a gzip archive",
        ),
        (
            "/repository/cran-own/bin/windows/contrib/4.4/astrumhello_1.1.0.zip",
            astrum_content::zip::write(&[(
                "astrumhello/DESCRIPTION",
                description("astrumhello", "1.1.0", "").as_bytes(),
            )]),
            "looks like a source package",
        ),
        (
            "/repository/cran-own/src/contrib/astrumhello_1.3.0.tar.gz",
            tar_of("other", &description("astrumhello", "1.3.0", "")),
            "not in astrumhello/",
        ),
        (
            "/repository/cran-own/src/contrib/astrumhello_1.4.0.tar.gz",
            b"not an archive".to_vec(),
            "not an R package archive",
        ),
    ] {
        let reply = put(&fixture, path, body).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{path}: {}",
            reply.text()
        );
        assert!(reply.text().contains(message), "{path}: {}", reply.text());
    }

    let plain = get(&fixture, "/repository/cran-own/src/contrib/PACKAGES").await;
    assert_eq!(plain.status, StatusCode::OK, "{}", plain.text());
    assert_eq!(
        plain.text(),
        format!(
            "Package: astrumdep\nVersion: 0.2.1\nImports: astrumhello (>= 1.1.0)\nLinkingTo: Rcpp\nLicense: MIT + file LICENSE\nMD5sum: {}\nNeedsCompilation: no\n\nPackage: astrumhello\nVersion: 1.1.0\nDepends: R (>= 3.5.0)\nImports: utils\nSuggests: testthat\nLicense: MIT + file LICENSE\nMD5sum: {}\nNeedsCompilation: no\n",
            md5(&dep),
            md5(&hello)
        )
    );
    let packed = get(&fixture, "/repository/cran-own/src/contrib/PACKAGES.gz").await;
    assert_eq!(packed.status, StatusCode::OK);
    assert_eq!(gunzip(&packed.body), plain.body);
    let rds = get(&fixture, "/repository/cran-own/src/contrib/PACKAGES.rds").await;
    assert_eq!(rds.status, StatusCode::OK);
    let serialized = gunzip(&rds.body);
    assert_eq!(&serialized[..6], b"X\n\0\0\0\x03");
    let text = String::from_utf8_lossy(&serialized).into_owned();
    assert!(text.contains("astrumhello (>= 1.1.0)") && text.contains("NeedsCompilation"));
    let file = get(
        &fixture,
        "/repository/cran-own/src/contrib/astrumhello_1.1.0.tar.gz",
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, hello);

    let binary = windows("astrumhello", "1.1.0");
    let uploaded = upload(
        &fixture,
        "cran-own",
        form(&[], "astrumhello_1.1.0.zip", &binary),
    )
    .await;
    assert_eq!(
        uploaded.status,
        StatusCode::NO_CONTENT,
        "{}",
        uploaded.text()
    );
    let archived = upload(
        &fixture,
        "cran-own",
        form(
            &[("r.asset.pathId", "src/contrib/Archive/astrumhello/")],
            "upload.bin",
            &source("astrumhello", "0.9.0", ""),
        ),
    )
    .await;
    assert_eq!(
        archived.status,
        StatusCode::NO_CONTENT,
        "{}",
        archived.text()
    );
    let refused = upload(
        &fixture,
        "cran-own",
        form(
            &[("r.asset.pathId", "src/contrib/Archive/astrumdep")],
            "x.tar.gz",
            &source("astrumhello", "0.8.0", ""),
        ),
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    let windows_index = get(
        &fixture,
        "/repository/cran-own/bin/windows/contrib/4.4/PACKAGES",
    )
    .await;
    assert_eq!(
        windows_index.text(),
        format!(
            "Package: astrumhello\nVersion: 1.1.0\nDepends: R (>= 4.0.0)\nLicense: MIT + file LICENSE\nMD5sum: {}\n",
            md5(&binary)
        )
    );
    let archive_index = get(
        &fixture,
        "/repository/cran-own/src/contrib/Archive/astrumhello/PACKAGES",
    )
    .await;
    assert_eq!(
        entries(&archive_index.text()),
        [("astrumhello".to_owned(), "0.9.0".to_owned())]
    );
    let components = get(&fixture, "/service/rest/v1/components?repository=cran-own")
        .await
        .json();
    let hello_component = components["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "astrumhello" && item["version"] == "1.1.0")
        .unwrap()
        .clone();
    assert_eq!(hello_component["group"], "");
    let paths: Vec<&str> = hello_component["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        [
            "/bin/windows/contrib/4.4/astrumhello_1.1.0.zip",
            "/src/contrib/astrumhello_1.1.0.tar.gz"
        ]
    );
    let asset = hello_component["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| asset["path"] == "/src/contrib/astrumhello_1.1.0.tar.gz")
        .unwrap();
    assert_eq!(asset["r"]["Title"], "A Test Package");
    assert_eq!(asset["r"]["Imports"], "utils");

    let empty = get(
        &fixture,
        "/repository/cran-own/bin/macosx/big-sur-arm64/contrib/4.4/PACKAGES",
    )
    .await;
    assert_eq!(empty.status, StatusCode::OK);
    assert!(empty.body.is_empty());
    let empty_rds = get(
        &fixture,
        "/repository/cran-own/bin/macosx/big-sur-arm64/contrib/4.4/PACKAGES.rds",
    )
    .await;
    assert_eq!(&gunzip(&empty_rds.body)[..2], b"X\n");
    assert_eq!(
        get(&fixture, "/repository/cran-own/elsewhere/PACKAGES")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/cran-own/src/contrib/astrumhello_1.1.0.tar.gz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let after = get(&fixture, "/repository/cran-own/src/contrib/PACKAGES").await;
    assert_eq!(
        entries(&after.text()),
        [
            ("astrumdep".to_owned(), "0.2.1".to_owned()),
            ("astrumhello".to_owned(), "1.0.0".to_owned())
        ]
    );
    let metadata = fixture
        .call(
            Method::DELETE,
            "/repository/cran-own/src/contrib/PACKAGES.gz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(metadata.status, StatusCode::BAD_REQUEST);
    let dep_id = components["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "astrumdep")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{dep_id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let after = get(&fixture, "/repository/cran-own/src/contrib/PACKAGES.gz").await;
    assert_eq!(
        entries(&String::from_utf8(gunzip(&after.body)).unwrap()),
        [("astrumhello".to_owned(), "1.0.0".to_owned())]
    );
    let page = get(&fixture, "/repository/cran-own/").await;
    assert!(page.text().contains("install.packages"));
    let post = fixture
        .call(
            Method::POST,
            "/repository/cran-own/src/contrib/x_1.0.tar.gz",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn uploads_need_write_access_and_readers_need_read_access() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("cran-private", "ALLOW")).await;
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "readers", "name": "readers", "description": "", "privileges": ["as-repository-view-r-cran-private-read"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("reader", "reader-pass", &["readers"])
        .await;
    let path = "/repository/cran-private/src/contrib/astrumhello_1.0.0.tar.gz";
    let anonymous = fixture
        .send_body(
            Method::PUT,
            path,
            Auth::None,
            Body::from(source("astrumhello", "1.0.0", "")),
            &[],
        )
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let reader = fixture
        .send_body(
            Method::PUT,
            path,
            Auth::Basic("reader", "reader-pass"),
            Body::from(source("astrumhello", "1.0.0", "")),
            &[],
        )
        .await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);
    assert_eq!(
        put(&fixture, path, source("astrumhello", "1.0.0", ""))
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        put(
            &fixture,
            path,
            source("astrumhello", "1.0.0", "Imports: stats\n")
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let index = fixture
        .call(
            Method::GET,
            "/repository/cran-private/src/contrib/PACKAGES",
            Auth::Basic("reader", "reader-pass"),
            None,
        )
        .await;
    assert_eq!(index.status, StatusCode::OK);
    assert!(index.text().contains("Imports: stats"));
    let stranger = fixture
        .call(
            Method::GET,
            "/repository/cran-private/src/contrib/PACKAGES",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(stranger.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn proxies_cache_a_remote_and_groups_merge_their_members() {
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
    create(&remote, "hosted", hosted("origin", "ALLOW")).await;
    let remote_hello = source("astrumhello", "1.1.0", "Imports: utils\n");
    let remote_new = source("astrumhello", "2.0.0", "Imports: utils, stats\n");
    let remote_other = source("astrumother", "0.1.0", "");
    for (file, body) in [
        ("astrumhello_1.1.0.tar.gz", remote_hello.clone()),
        ("astrumhello_2.0.0.tar.gz", remote_new.clone()),
        ("astrumother_0.1.0.tar.gz", remote_other.clone()),
    ] {
        let reply = put(
            &remote,
            &format!("/repository/origin/src/contrib/{file}"),
            body,
        )
        .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
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
    let app = astrum_api::router(remote.state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    create(
        &fixture,
        "proxy",
        json!({
            "name": "cran-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/repository/origin/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    create(&fixture, "hosted", hosted("cran-own", "ALLOW")).await;
    let own_hello = source("astrumhello", "1.1.0", "Imports: methods\n");
    assert_eq!(
        put(
            &fixture,
            "/repository/cran-own/src/contrib/astrumhello_1.1.0.tar.gz",
            own_hello.clone()
        )
        .await
        .status,
        StatusCode::CREATED
    );
    create(
        &fixture,
        "group",
        json!({
            "name": "cran-all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["cran-own", "cran-proxy"]},
        }),
    )
    .await;

    let rds = get(&fixture, "/repository/cran-proxy/src/contrib/PACKAGES.rds").await;
    assert_eq!(rds.status, StatusCode::OK, "{}", rds.text());
    let original = get(&remote, "/repository/origin/src/contrib/PACKAGES.rds").await;
    assert_eq!(rds.body, original.body);
    let proxied = get(
        &fixture,
        "/repository/cran-proxy/src/contrib/astrumhello_2.0.0.tar.gz",
    )
    .await;
    assert_eq!(proxied.status, StatusCode::OK);
    assert_eq!(proxied.body, remote_new);
    let components = get(
        &fixture,
        "/service/rest/v1/components?repository=cran-proxy",
    )
    .await
    .json();
    assert_eq!(components["items"][0]["name"], "astrumhello");
    assert_eq!(components["items"][0]["version"], "2.0.0");

    let merged = get(&fixture, "/repository/cran-all/src/contrib/PACKAGES").await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    let records = astrum_format_r::dcf::parse(&merged.text());
    let listed: Vec<(&str, &str, &str)> = records
        .iter()
        .map(|record| {
            (
                record.get("Package").unwrap(),
                record.get("Version").unwrap(),
                record.get("MD5sum").unwrap(),
            )
        })
        .collect();
    let own_md5 = md5(&own_hello);
    let new_md5 = md5(&remote_new);
    let other_md5 = md5(&remote_other);
    assert_eq!(
        listed,
        [
            ("astrumhello", "1.1.0", own_md5.as_str()),
            ("astrumhello", "2.0.0", new_md5.as_str()),
            ("astrumother", "0.1.0", other_md5.as_str()),
        ]
    );
    let packed = get(&fixture, "/repository/cran-all/src/contrib/PACKAGES.gz").await;
    assert_eq!(gunzip(&packed.body), merged.body);
    let merged_rds = get(&fixture, "/repository/cran-all/src/contrib/PACKAGES.rds").await;
    let serialized = gunzip(&merged_rds.body);
    assert_eq!(&serialized[..6], b"X\n\0\0\0\x03");
    let etag = merged_rds.headers["etag"].to_str().unwrap().to_owned();
    let cached = fixture
        .send(
            Method::GET,
            "/repository/cran-all/src/contrib/PACKAGES.rds",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(cached.status, StatusCode::NOT_MODIFIED);
    let head = fixture
        .call(
            Method::HEAD,
            "/repository/cran-all/src/contrib/PACKAGES",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());

    let own = get(
        &fixture,
        "/repository/cran-all/src/contrib/astrumhello_1.1.0.tar.gz",
    )
    .await;
    assert_eq!(own.body, own_hello);
    let through = get(
        &fixture,
        "/repository/cran-all/src/contrib/astrumother_0.1.0.tar.gz",
    )
    .await;
    assert_eq!(through.status, StatusCode::OK);
    assert_eq!(through.body, remote_other);

    let newer = source("astrumhello", "3.0.0", "");
    assert_eq!(
        put(
            &fixture,
            "/repository/cran-own/src/contrib/astrumhello_3.0.0.tar.gz",
            newer
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let merged = get(&fixture, "/repository/cran-all/src/contrib/PACKAGES").await;
    assert_eq!(
        entries(&merged.text()),
        [
            ("astrumhello".to_owned(), "3.0.0".to_owned()),
            ("astrumhello".to_owned(), "2.0.0".to_owned()),
            ("astrumother".to_owned(), "0.1.0".to_owned()),
        ]
    );
    let windows_index = get(
        &fixture,
        "/repository/cran-all/bin/windows/contrib/4.4/PACKAGES",
    )
    .await;
    assert_eq!(windows_index.status, StatusCode::OK);
    assert!(windows_index.body.is_empty());
    let refused = put(
        &fixture,
        "/repository/cran-all/src/contrib/astrumhello_4.0.0.tar.gz",
        source("astrumhello", "4.0.0", ""),
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    let refused = put(
        &fixture,
        "/repository/cran-proxy/src/contrib/astrumhello_4.0.0.tar.gz",
        source("astrumhello", "4.0.0", ""),
    )
    .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn the_rebuild_task_writes_the_packages_files_again() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("cran-own", "ALLOW")).await;
    let hello = source("astrumhello", "1.0.0", "");
    assert_eq!(
        put(
            &fixture,
            "/repository/cran-own/src/contrib/astrumhello_1.0.0.tar.gz",
            hello
        )
        .await
        .status,
        StatusCode::CREATED
    );
    use sea_orm::ConnectionTrait;
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(
            "UPDATE asset SET attributes = '{}'::jsonb WHERE path = '/src/contrib/PACKAGES'",
        )
        .await
        .unwrap();
    let created = fixture
        .call(
            Method::POST,
            "/service/rest/v1/tasks",
            Auth::Admin,
            Some(json!({"type": "repository.r.rebuild.metadata", "name": "Rebuild R", "properties": {"repositoryName": "cran-own"}})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let id = created.json()["id"].as_str().unwrap().to_owned();
    let run = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/tasks/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(run.status, StatusCode::NO_CONTENT, "{}", run.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let task = get(&fixture, &format!("/service/rest/v1/tasks/{id}"))
        .await
        .json();
    assert_eq!(task["lastRunResult"], "OK", "{task}");
    let index = get(&fixture, "/service/rest/v1/assets?repository=cran-own")
        .await
        .json();
    assert!(
        index["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["path"] == "/src/contrib/PACKAGES"),
        "{index}"
    );
    let stored = fixture
        .state
        .db()
        .conn()
        .query_one(sea_orm::Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "SELECT attributes::text AS a FROM asset WHERE path = '/src/contrib/PACKAGES'",
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<String>("", "a")
        .unwrap();
    assert!(stored.contains("fingerprint"), "{stored}");
    let templates = get(&fixture, "/service/rest/v1/tasks/templates")
        .await
        .json();
    assert!(
        templates
            .as_array()
            .unwrap()
            .iter()
            .any(|template| template["type"] == "repository.r.rebuild.metadata"),
        "{templates}"
    );
}

#[tokio::test]
async fn repositories_are_created_changed_and_validated_through_the_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("cran-own", "ALLOW_ONCE")).await;
    let document = get(&fixture, "/service/rest/v1/repositories/r/hosted/cran-own")
        .await
        .json();
    assert_eq!(document["format"], "r");
    assert_eq!(document["type"], "hosted");
    assert_eq!(document["storage"]["writePolicy"], "ALLOW_ONCE");
    assert_eq!(document["url"], "http://localhost/repository/cran-own");
    let mut changed = document.clone();
    changed["storage"]["writePolicy"] = json!("ALLOW");
    let updated = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/r/hosted/cran-own",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(updated.status, StatusCode::NO_CONTENT, "{}", updated.text());
    let document = get(&fixture, "/service/rest/v1/repositories/r/hosted/cran-own")
        .await
        .json();
    assert_eq!(document["storage"]["writePolicy"], "ALLOW");
    let proxy = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/r/proxy",
            Auth::Admin,
            Some(json!({
                "name": "cran-proxy",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "proxy": {"contentMaxAge": 1440, "metadataMaxAge": 1440},
                "negativeCache": {"enabled": true, "timeToLive": 1440},
                "httpClient": {"blocked": false, "autoBlock": true},
            })),
        )
        .await;
    assert_eq!(proxy.status, StatusCode::BAD_REQUEST, "{}", proxy.text());
    assert!(proxy.text().contains("remoteUrl"), "{}", proxy.text());
    let raw = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(hosted("files", "ALLOW")),
        )
        .await;
    assert_eq!(raw.status, StatusCode::CREATED, "{}", raw.text());
    let group = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/r/group",
            Auth::Admin,
            Some(json!({
                "name": "cran-all",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
                "group": {"memberNames": ["cran-own", "files"]},
            })),
        )
        .await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST, "{}", group.text());
    let specs = get(&fixture, "/service/rest/v1/formats/r/upload-specs")
        .await
        .json();
    assert_eq!(specs["format"], "r");
    let names: Vec<&str> = specs["assetFields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["pathId", "asset"]);
}
