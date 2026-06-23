mod support;

use std::io::Write;
use std::sync::Arc;

use astrum_api::router_with;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const BOUNDARY: &str = "terraformboundary0123456789";
const SECRET_KEY: &str = include_str!("../../astrum-format-terraform/tests/data/secret.asc");

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_terraform::Terraform::default()));
    fixture
}

async fn send(
    fixture: &Fixture,
    method: Method,
    path: &str,
    auth: Auth<'_>,
    body: Body,
    headers: &[(&str, &str)],
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    match auth {
        Auth::None => {}
        Auth::Admin => {
            let encoded = STANDARD.encode(format!("admin:{}", fixture.admin_password));
            request = request.header(header::AUTHORIZATION, format!("Basic {encoded}"));
        }
        Auth::Basic(user, password) => {
            let encoded = STANDARD.encode(format!("{user}:{password}"));
            request = request.header(header::AUTHORIZATION, format!("Basic {encoded}"));
        }
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let response = router_with(fixture.state.clone(), astrum_format_terraform::routes())
        .oneshot(request.body(body).unwrap())
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

async fn get(fixture: &Fixture, path: &str, auth: Auth<'_>) -> Reply {
    send(fixture, Method::GET, path, auth, Body::empty(), &[]).await
}

async fn put(fixture: &Fixture, path: &str, data: Vec<u8>) -> Reply {
    send(
        fixture,
        Method::PUT,
        path,
        Auth::Admin,
        Body::from(data),
        &[],
    )
    .await
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/terraform/{kind}"),
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

fn module_package(top: &str) -> Vec<u8> {
    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    let data = b"output \"greeting\" {\n  value = \"hello\"\n}\n";
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, format!("{top}/main.tf"), &data[..])
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

fn provider_zip(name: &str) -> Vec<u8> {
    let content = format!("#!/bin/sh\necho {name}\n");
    let crc = {
        let mut table = [0u32; 256];
        for (i, entry) in table.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 == 1 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            *entry = c;
        }
        let mut crc = 0xffff_ffffu32;
        for byte in content.as_bytes() {
            crc = table[((crc ^ *byte as u32) & 0xff) as usize] ^ (crc >> 8);
        }
        !crc
    };
    let file = name.as_bytes();
    let mut zip = Vec::new();
    let local = |zip: &mut Vec<u8>| {
        zip.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        zip.extend_from_slice(&crc.to_le_bytes());
        zip.extend_from_slice(&(content.len() as u32).to_le_bytes());
        zip.extend_from_slice(&(content.len() as u32).to_le_bytes());
        zip.extend_from_slice(&(file.len() as u16).to_le_bytes());
        zip.extend_from_slice(&[0, 0]);
        zip.extend_from_slice(file);
        zip.extend_from_slice(content.as_bytes());
    };
    local(&mut zip);
    let central = zip.len() as u32;
    zip.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02, 20, 3, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    zip.extend_from_slice(&crc.to_le_bytes());
    zip.extend_from_slice(&(content.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(content.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(file.len() as u16).to_le_bytes());
    zip.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);
    zip.extend_from_slice(&(0o100755u32 << 16).to_le_bytes());
    zip.extend_from_slice(&0u32.to_le_bytes());
    zip.extend_from_slice(file);
    let size = zip.len() as u32 - central;
    zip.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0]);
    zip.extend_from_slice(&size.to_le_bytes());
    zip.extend_from_slice(&central.to_le_bytes());
    zip.extend_from_slice(&[0, 0]);
    zip
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn multipart(fields: &[(&str, &str)], files: &[(&str, &str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
        )
        .unwrap();
    }
    for (name, file, data) in files {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn upload(
    fixture: &Fixture,
    repository: &str,
    fields: &[(&str, &str)],
    files: &[(&str, &str, &[u8])],
) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    send(
        fixture,
        Method::POST,
        &format!("/service/rest/v1/components?repository={repository}"),
        Auth::Admin,
        Body::from(multipart(fields, files)),
        &[("content-type", content_type.as_str())],
    )
    .await
}

fn query(location: &str) -> &str {
    location.split_once('?').map(|(_, query)| query).unwrap()
}

#[tokio::test]
async fn hosted_modules_are_listed_and_downloaded_through_signed_links() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("tf-hosted", "allow_once")).await;
    let base = "/repository/tf-hosted/v1/modules/acme/vpc/aws";
    let stored = put(
        &fixture,
        &format!("{base}/1.2.0/package.tar.gz"),
        module_package("vpc"),
    )
    .await;
    assert_eq!(stored.status, StatusCode::CREATED, "{}", stored.text());
    let second = put(
        &fixture,
        &format!("{base}/1.10.0/vpc.zip"),
        module_package("vpc"),
    )
    .await;
    assert_eq!(second.status, StatusCode::BAD_REQUEST, "{}", second.text());
    let again = put(
        &fixture,
        &format!("{base}/1.2.0/x.tar.gz"),
        module_package("vpc"),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.text());
    put(
        &fixture,
        &format!("{base}/1.10.0/vpc.tgz"),
        module_package("vpc"),
    )
    .await;

    let versions = get(&fixture, &format!("{base}/versions"), Auth::Admin).await;
    assert_eq!(
        versions.json(),
        json!({"modules": [{"versions": [{"version": "1.2.0"}, {"version": "1.10.0"}]}]})
    );
    let anonymous = get(&fixture, &format!("{base}/versions"), Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let missing = get(
        &fixture,
        "/repository/tf-hosted/v1/modules/acme/x/aws/versions",
        Auth::Admin,
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.json()["errors"][0], "Module acme/x/aws not found");

    let download = get(&fixture, &format!("{base}/1.2.0/download"), Auth::Admin).await;
    assert_eq!(download.status, StatusCode::NO_CONTENT);
    let location = download.headers["x-terraform-get"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        location.starts_with("./vpc-aws-1.2.0.tar.gz//vpc?expires="),
        "{location}"
    );
    let archive = format!("{base}/1.2.0/vpc-aws-1.2.0.tar.gz?{}", query(&location));
    let fetched = get(&fixture, &archive, Auth::None).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, module_package("vpc"));
    let forged = get(
        &fixture,
        &format!("{base}/1.10.0/vpc-aws-1.10.0.tar.gz?{}", query(&location)),
        Auth::None,
    )
    .await;
    assert_eq!(forged.status, StatusCode::UNAUTHORIZED);
    let not_archive = put(
        &fixture,
        &format!("{base}/2.0.0/vpc.tar.gz"),
        b"plain text".to_vec(),
    )
    .await;
    assert_eq!(not_archive.status, StatusCode::BAD_REQUEST);

    let discovery = get(
        &fixture,
        "/repository/tf-hosted/.well-known/terraform.json",
        Auth::None,
    )
    .await;
    assert_eq!(
        discovery.json(),
        json!({"modules.v1": "../v1/modules/", "providers.v1": "../v1/providers/"})
    );
    let search = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=tf-hosted&terraform.kind=module&terraform.provider=aws",
            Auth::Admin,
            None,
        )
        .await;
    let items = search.json()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 2, "{}", search.text());
    assert_eq!(items[0]["group"], "acme");
    assert_eq!(items[0]["name"], "vpc/aws");
}

#[tokio::test]
async fn provider_releases_are_checked_signed_and_mirrored() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("tf-providers", "allow")).await;
    let spec = get(
        &fixture,
        "/service/rest/v1/formats/terraform/upload-specs",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        spec["assetFields"],
        json!([{"name": "asset", "type": "FILE", "description": null, "optional": false,
            "group": null, "groupHelpText": null, "options": null}])
    );
    let package = provider_zip("terraform-provider-demo_v2.0.0");
    let archive = "terraform-provider-demo_2.0.0_linux_amd64.zip";
    let wrong = format!("{}  {archive}\n", "0".repeat(64));
    let refused = upload(
        &fixture,
        "tf-providers",
        &[
            ("terraform.namespace", "acme"),
            ("terraform.type", "demo"),
            ("terraform.version", "2.0.0"),
        ],
        &[
            ("terraform.asset1", archive, &package),
            (
                "terraform.asset2",
                "terraform-provider-demo_2.0.0_SHA256SUMS",
                wrong.as_bytes(),
            ),
        ],
    )
    .await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert!(
        refused.text().contains("does not match its checksum"),
        "{}",
        refused.text()
    );

    let accepted = upload(
        &fixture,
        "tf-providers",
        &[
            ("terraform.namespace", "acme"),
            ("terraform.type", "demo"),
            ("terraform.version", "v2.0.0"),
            ("terraform.protocols", "6"),
        ],
        &[("terraform.asset", archive, &package)],
    )
    .await;
    assert_eq!(
        accepted.status,
        StatusCode::NO_CONTENT,
        "{}",
        accepted.text()
    );
    let base = "/repository/tf-providers/v1/providers/acme/demo";
    let versions = get(&fixture, &format!("{base}/versions"), Auth::Admin).await;
    assert_eq!(
        versions.json(),
        json!({"versions": [{"version": "2.0.0", "protocols": ["6.0"], "platforms": [{"os": "linux", "arch": "amd64"}]}]})
    );
    let sums = get(
        &fixture,
        &format!("{base}/2.0.0/terraform-provider-demo_2.0.0_SHA256SUMS"),
        Auth::Admin,
    )
    .await;
    assert_eq!(sums.text(), format!("{}  {archive}\n", sha256(&package)));
    let unsigned = get(
        &fixture,
        &format!("{base}/2.0.0/download/linux/amd64"),
        Auth::Admin,
    )
    .await;
    assert_eq!(unsigned.status, StatusCode::NOT_FOUND);
    assert!(
        unsigned.text().contains("is not signed"),
        "{}",
        unsigned.text()
    );

    let mut signed = hosted("tf-providers", "allow");
    signed["terraformSigning"] = json!({"keypair": SECRET_KEY, "passphrase": "probe-passphrase"});
    let updated = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/terraform/hosted/tf-providers",
            Auth::Admin,
            Some(signed),
        )
        .await;
    assert_eq!(updated.status, StatusCode::NO_CONTENT, "{}", updated.text());
    let second = provider_zip("terraform-provider-demo_v2.0.0");
    let darwin = put(
        &fixture,
        &format!("{base}/2.0.0/terraform-provider-demo_2.0.0_darwin_arm64.zip"),
        second.clone(),
    )
    .await;
    assert_eq!(darwin.status, StatusCode::CREATED, "{}", darwin.text());
    let document = get(
        &fixture,
        &format!("{base}/2.0.0/download/linux/amd64"),
        Auth::Admin,
    )
    .await;
    assert_eq!(document.status, StatusCode::OK, "{}", document.text());
    let document = document.json();
    assert_eq!(document["shasum"], sha256(&package));
    assert_eq!(document["protocols"], json!(["6.0"]));
    assert_eq!(document["filename"], archive);
    let keys = document["signing_keys"]["gpg_public_keys"]
        .as_array()
        .unwrap();
    assert_eq!(keys.len(), 1);
    assert!(
        keys[0]["ascii_armor"]
            .as_str()
            .unwrap()
            .contains("BEGIN PGP PUBLIC KEY")
    );
    let download_url = document["download_url"].as_str().unwrap();
    assert!(
        download_url.starts_with(&format!("../../{archive}?expires=")),
        "{download_url}"
    );
    let signature_url = document["shasums_signature_url"].as_str().unwrap();
    let signature = get(
        &fixture,
        &format!(
            "{base}/2.0.0/terraform-provider-demo_2.0.0_SHA256SUMS.sig?{}",
            query(signature_url)
        ),
        Auth::None,
    )
    .await;
    assert_eq!(signature.status, StatusCode::OK);
    assert!(!signature.body.is_empty());

    let index = get(
        &fixture,
        "/repository/tf-providers/registry.example.org/acme/demo/index.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(index.json(), json!({"versions": {"2.0.0": {}}}));
    let mirror = get(
        &fixture,
        "/repository/tf-providers/registry.example.org/acme/demo/2.0.0.json",
        Auth::Admin,
    )
    .await
    .json();
    assert_eq!(
        mirror["archives"]["linux_amd64"]["hashes"],
        json!([format!("zh:{}", sha256(&package))])
    );
    assert!(
        mirror["archives"]["darwin_arm64"]["url"]
            .as_str()
            .unwrap()
            .starts_with("../../../v1/providers/acme/demo/2.0.0/terraform-provider-demo_2.0.0_darwin_arm64.zip?expires=")
    );

    let deleted = send(
        &fixture,
        Method::DELETE,
        &format!("{base}/2.0.0/terraform-provider-demo_2.0.0_darwin_arm64.zip"),
        Auth::Admin,
        Body::empty(),
        &[],
    )
    .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let sums = get(
        &fixture,
        &format!("{base}/2.0.0/terraform-provider-demo_2.0.0_SHA256SUMS"),
        Auth::Admin,
    )
    .await;
    assert_eq!(sums.text(), format!("{}  {archive}\n", sha256(&package)));
}

#[tokio::test]
async fn groups_merge_members_and_answer_discovery_and_tokens() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("tf-a", "allow")).await;
    create(&fixture, "hosted", hosted("tf-b", "allow")).await;
    create(
        &fixture,
        "group",
        json!({
            "name": "tf-group",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["tf-a", "tf-b"]},
            "terraform": {"registryHost": "registry.example.org", "requireAuthentication": true}
        }),
    )
    .await;
    put(
        &fixture,
        "/repository/tf-a/v1/modules/acme/vpc/aws/1.0.0/a.tar.gz",
        module_package("a"),
    )
    .await;
    put(
        &fixture,
        "/repository/tf-b/v1/modules/acme/vpc/aws/2.0.0/b.tgz",
        module_package("b"),
    )
    .await;
    let versions = get(
        &fixture,
        "/repository/tf-group/v1/modules/acme/vpc/aws/versions",
        Auth::Admin,
    )
    .await;
    assert_eq!(
        versions.json(),
        json!({"modules": [{"versions": [{"version": "1.0.0"}, {"version": "2.0.0"}]}]})
    );
    let download = get(
        &fixture,
        "/repository/tf-group/v1/modules/acme/vpc/aws/2.0.0/download",
        Auth::Admin,
    )
    .await;
    let location = download.headers["x-terraform-get"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        location.starts_with("./vpc-aws-2.0.0.tar.gz//b?expires="),
        "{location}"
    );
    let fetched = get(
        &fixture,
        &format!(
            "/repository/tf-group/v1/modules/acme/vpc/aws/2.0.0/vpc-aws-2.0.0.tar.gz?{}",
            query(&location)
        ),
        Auth::None,
    )
    .await;
    assert_eq!(fetched.status, StatusCode::OK);
    let member_link = get(
        &fixture,
        &format!(
            "/repository/tf-b/v1/modules/acme/vpc/aws/2.0.0/vpc-aws-2.0.0.tar.gz?{}",
            query(&location)
        ),
        Auth::None,
    )
    .await;
    assert_eq!(member_link.status, StatusCode::UNAUTHORIZED);

    let discovery = send(
        &fixture,
        Method::GET,
        "/.well-known/terraform.json",
        Auth::None,
        Body::empty(),
        &[("host", "Registry.Example.org:8443")],
    )
    .await;
    assert_eq!(
        discovery.json(),
        json!({"modules.v1": "/repository/tf-group/v1/modules/", "providers.v1": "/repository/tf-group/v1/providers/"})
    );
    let unknown = send(
        &fixture,
        Method::GET,
        "/.well-known/terraform.json",
        Auth::None,
        Body::empty(),
        &[("host", "other.example.org")],
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    let inactive = get(&fixture, "/repository/tf-group/token", Auth::Admin).await;
    assert_eq!(inactive.status, StatusCode::CONFLICT);
    fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/realms/active",
            Auth::Admin,
            Some(json!(["AstrumAuthenticatingRealm", "TerraformToken"])),
        )
        .await;
    let issued = get(&fixture, "/repository/tf-group/token", Auth::Admin).await;
    assert_eq!(issued.status, StatusCode::OK, "{}", issued.text());
    let token = issued.json()["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("TerraformToken."));
    let bearer = format!("Bearer {token}");
    let with_token = send(
        &fixture,
        Method::GET,
        "/repository/tf-group/v1/modules/acme/vpc/aws/versions",
        Auth::None,
        Body::empty(),
        &[("authorization", bearer.as_str())],
    )
    .await;
    assert_eq!(with_token.status, StatusCode::OK);
    let revoked = send(
        &fixture,
        Method::DELETE,
        "/repository/tf-group/token",
        Auth::Admin,
        Body::empty(),
        &[],
    )
    .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT);
    let after = send(
        &fixture,
        Method::GET,
        "/repository/tf-group/v1/modules/acme/vpc/aws/versions",
        Auth::None,
        Body::empty(),
        &[("authorization", bearer.as_str())],
    )
    .await;
    assert_eq!(after.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn proxies_cache_modules_and_provider_files_of_their_remote() {
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
    let mut signed = hosted("tf-upstream", "allow");
    signed["terraformSigning"] = json!({"keypair": SECRET_KEY, "passphrase": "probe-passphrase"});
    create(&fixture, "hosted", signed).await;
    put(
        &fixture,
        "/repository/tf-upstream/v1/modules/acme/vpc/aws/1.0.0/vpc.tar.gz",
        module_package("vpc"),
    )
    .await;
    let package = provider_zip("terraform-provider-demo_v1.0.0");
    put(
        &fixture,
        "/repository/tf-upstream/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_linux_amd64.zip",
        package.clone(),
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = router_with(fixture.state.clone(), astrum_format_terraform::routes());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "tf-proxy",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}/repository/tf-upstream/"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 60},
            "httpClient": {"blocked": false, "autoBlock": false,
                "authentication": {"type": "username", "username": "admin", "password": fixture.admin_password}}
        }),
    )
    .await;
    let versions = get(
        &fixture,
        "/repository/tf-proxy/v1/modules/acme/vpc/aws/versions",
        Auth::Admin,
    )
    .await;
    assert_eq!(
        versions.json(),
        json!({"modules": [{"versions": [{"version": "1.0.0"}]}]})
    );
    let download = get(
        &fixture,
        "/repository/tf-proxy/v1/modules/acme/vpc/aws/1.0.0/download",
        Auth::Admin,
    )
    .await;
    let location = download.headers["x-terraform-get"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(
        location.starts_with("./vpc-aws-1.0.0.tar.gz//vpc?expires="),
        "{location}"
    );
    let cached = get(
        &fixture,
        &format!(
            "/repository/tf-proxy/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz?{}",
            query(&location)
        ),
        Auth::None,
    )
    .await;
    assert_eq!(cached.body, module_package("vpc"));

    let document = get(
        &fixture,
        "/repository/tf-proxy/v1/providers/acme/demo/1.0.0/download/linux/amd64",
        Auth::Admin,
    )
    .await
    .json();
    let url = document["download_url"].as_str().unwrap();
    assert!(
        url.starts_with("../../terraform-provider-demo_1.0.0_linux_amd64.zip?expires="),
        "{url}"
    );
    let file = get(
        &fixture,
        &format!("/repository/tf-proxy/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_linux_amd64.zip?{}", query(url)),
        Auth::None,
    )
    .await;
    assert_eq!(file.status, StatusCode::OK);
    assert_eq!(file.body, package);
    let mirror = get(
        &fixture,
        &format!("/repository/tf-proxy/{address}/acme/demo/1.0.0.json"),
        Auth::Admin,
    )
    .await;
    assert_eq!(mirror.status, StatusCode::OK, "{}", mirror.text());
    assert_eq!(
        mirror.json()["archives"]["linux_amd64"]["hashes"],
        json!([format!("zh:{}", sha256(&package))])
    );
    let other = get(
        &fixture,
        "/repository/tf-proxy/registry.terraform.io/acme/demo/index.json",
        Auth::Admin,
    )
    .await;
    assert_eq!(other.status, StatusCode::NOT_FOUND);
}
