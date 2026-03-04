mod support;
mod ui_support;

use std::io::Write;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};
use ui_support::{create, get};

const BROWSE: &str = "/service/rest/internal/ui/browse";
const BOUNDARY: &str = "layout-boundary";

async fn ready() -> Fixture {
    let fixture = ui_support::ready().await;
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_pypi::PyPi));
    content.register(Arc::new(astrum_format_helm::Helm::default()));
    content.register(Arc::new(astrum_format_nuget::Nuget::default()));
    fixture
}

fn hosted(name: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "docker": {"v1Enabled": false, "forceBasicAuth": true, "pathEnabled": true}
    })
}

async fn send(fixture: &Fixture, method: Method, path: &str, kind: &str, body: Vec<u8>) -> Reply {
    let reply = fixture
        .send_body(
            method,
            path,
            Auth::Admin,
            Body::from(body),
            &[("content-type", kind)],
        )
        .await;
    assert!(
        reply.status.is_success(),
        "{path}: {} {}",
        reply.status,
        reply.text()
    );
    reply
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn zip(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let directory = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&directory.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

fn form(fields: &[(&str, &str)], files: &[(&str, &str, &[u8])]) -> Vec<u8> {
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

fn multipart() -> String {
    format!("multipart/form-data; boundary={BOUNDARY}")
}

async fn listing(fixture: &Fixture, repository: &str, path: &str) -> Vec<(String, String, String)> {
    let reply = get(
        fixture,
        &format!("{BROWSE}/{repository}?path={path}"),
        Auth::Admin,
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.json()["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| {
            (
                node["kind"].as_str().unwrap().to_owned(),
                node["name"].as_str().unwrap().to_owned(),
                node["path"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn triples(expected: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    expected
        .iter()
        .map(|(kind, name, path)| ((*kind).to_owned(), (*name).to_owned(), (*path).to_owned()))
        .collect()
}

async fn push_image(fixture: &Fixture, repository: &str, image: &str, tags: &[&str]) {
    let prefix = format!("/repository/{repository}/v2/{image}");
    let config = format!("{{\"os\":\"linux\",\"image\":\"{image}\"}}").into_bytes();
    let layer = format!("layer of {image}").into_bytes();
    for data in [&config, &layer] {
        let digest = format!("sha256:{}", sha256(data));
        send(
            fixture,
            Method::POST,
            &format!("{prefix}/blobs/uploads/?digest={digest}"),
            "application/octet-stream",
            data.clone(),
        )
        .await;
    }
    let manifest = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {"mediaType": "application/vnd.oci.image.config.v1+json", "digest": format!("sha256:{}", sha256(&config)), "size": config.len()},
        "layers": [{"mediaType": "application/vnd.oci.image.layer.v1.tar+gzip", "digest": format!("sha256:{}", sha256(&layer)), "size": layer.len()}]
    });
    for tag in tags {
        send(
            fixture,
            Method::PUT,
            &format!("{prefix}/manifests/{tag}"),
            "application/vnd.oci.image.manifest.v1+json",
            serde_json::to_vec(&manifest).unwrap(),
        )
        .await;
    }
}

#[tokio::test]
async fn docker_shows_images_and_tags_without_blobs_and_digests() {
    let fixture = ready().await;
    create(&fixture, "docker", "hosted", hosted("images")).await;
    push_image(
        &fixture,
        "images",
        "team/app",
        &["1.0", "latest", "10.0", "2.0"],
    )
    .await;
    push_image(&fixture, "images", "solo", &["2"]).await;

    assert_eq!(
        listing(&fixture, "images", "/").await,
        triples(&[("folder", "solo", "/solo/"), ("folder", "team", "/team/")])
    );
    assert_eq!(
        listing(&fixture, "images", "/team/").await,
        triples(&[("folder", "app", "/team/app/")])
    );
    assert_eq!(
        listing(&fixture, "images", "/team/app/").await,
        triples(&[
            ("asset", "latest", "/team/app/latest"),
            ("asset", "1.0", "/team/app/1.0"),
            ("asset", "2.0", "/team/app/2.0"),
            ("asset", "10.0", "/team/app/10.0"),
        ])
    );
    let tags = get(
        &fixture,
        &format!("{BROWSE}/images?path=/team/app/"),
        Auth::Admin,
    )
    .await
    .json();
    let tag = &tags["children"][1];
    assert_eq!(tag["assetPath"], "/v2/team/app/manifests/1.0");
    assert_eq!(tag["component"]["name"], "team/app");
    assert_eq!(
        tag["downloadUrl"],
        "http://localhost/repository/images/v2/team/app/manifests/1.0"
    );

    let details = get(
        &fixture,
        &format!("{BROWSE}/images/asset?path=/team/app/1.0"),
        Auth::Admin,
    )
    .await;
    assert_eq!(details.status, StatusCode::OK, "{}", details.text());
    assert_eq!(details.json()["path"], "/v2/team/app/manifests/1.0");
    assert_eq!(details.json()["treePath"], "/team/app/1.0");
    assert!(
        details.json()["blobRef"]
            .as_str()
            .unwrap()
            .starts_with("default@")
    );
    let stored = get(
        &fixture,
        &format!("{BROWSE}/images/asset?path=/v2/team/app/manifests/1.0"),
        Auth::Admin,
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK);
    assert_eq!(stored.json()["treePath"], "/team/app/1.0");
    let missing = get(&fixture, &format!("{BROWSE}/images?path=/v2/"), Auth::Admin).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{BROWSE}/images?path=/team/"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK, "{}", deleted.text());
    assert_eq!(deleted.json()["deletedAssets"], 4);
    assert_eq!(
        listing(&fixture, "images", "/").await,
        triples(&[("folder", "solo", "/solo/")])
    );
}

async fn publish(fixture: &Fixture, repository: &str, name: &str, version: &str) {
    let mut data = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    data.extend_from_slice(format!("package {name} {version}").as_bytes());
    let bare = name.rsplit('/').next().unwrap();
    let encoded = STANDARD.encode(&data);
    let document = json!({
        "_id": name,
        "name": name,
        "dist-tags": {"latest": version},
        "versions": {version: {"name": name, "version": version, "dist": {"tarball": format!("http://client.invalid/{name}/-/{bare}-{version}.tgz"), "shasum": hex::encode(sha1_digest(&data))}}},
        "_attachments": {format!("{name}-{version}.tgz"): {"content_type": "application/octet-stream", "data": encoded, "length": data.len()}}
    });
    send(
        fixture,
        Method::PUT,
        &format!("/repository/{repository}/{}", name.replace('/', "%2f")),
        "application/json",
        serde_json::to_vec(&document).unwrap(),
    )
    .await;
}

fn sha1_digest(data: &[u8]) -> Vec<u8> {
    use sha1::Digest as _;
    sha1::Sha1::digest(data).to_vec()
}

#[tokio::test]
async fn npm_shows_packages_with_their_tarballs() {
    let fixture = ready().await;
    create(&fixture, "npm", "hosted", hosted("packages")).await;
    publish(&fixture, "packages", "@team/tool", "1.0.0").await;
    publish(&fixture, "packages", "left-pad", "1.3.0").await;
    publish(&fixture, "packages", "left-pad", "1.4.0").await;

    assert_eq!(
        listing(&fixture, "packages", "/").await,
        triples(&[
            ("folder", "@team", "/@team/"),
            ("folder", "left-pad", "/left-pad/")
        ])
    );
    assert_eq!(
        listing(&fixture, "packages", "/@team/tool/").await,
        triples(&[("asset", "tool-1.0.0.tgz", "/@team/tool/tool-1.0.0.tgz")])
    );
    assert_eq!(
        listing(&fixture, "packages", "/left-pad/").await,
        triples(&[
            (
                "asset",
                "left-pad-1.3.0.tgz",
                "/left-pad/left-pad-1.3.0.tgz"
            ),
            (
                "asset",
                "left-pad-1.4.0.tgz",
                "/left-pad/left-pad-1.4.0.tgz"
            ),
        ])
    );
    let details = get(
        &fixture,
        &format!("{BROWSE}/packages/asset?path=/left-pad/left-pad-1.4.0.tgz"),
        Auth::Admin,
    )
    .await;
    assert_eq!(details.status, StatusCode::OK, "{}", details.text());
    assert_eq!(details.json()["path"], "/left-pad/-/left-pad-1.4.0.tgz");

    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{BROWSE}/packages?path=/left-pad/"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK, "{}", deleted.text());
    assert_eq!(deleted.json()["deletedAssets"], 3);
    assert_eq!(
        listing(&fixture, "packages", "/").await,
        triples(&[("folder", "@team", "/@team/")])
    );
}

fn wheel(name: &str, version: &str) -> Vec<u8> {
    let info = format!("{name}-{version}.dist-info");
    zip(&[
        (&format!("{name}/__init__.py"), Vec::new()),
        (
            &format!("{info}/METADATA"),
            format!("Metadata-Version: 2.1\nName: {name}\nVersion: {version}\n").into_bytes(),
        ),
        (
            &format!("{info}/WHEEL"),
            b"Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n".to_vec(),
        ),
        (&format!("{info}/RECORD"), Vec::new()),
    ])
}

async fn upload_python(fixture: &Fixture, repository: &str, file: &str, data: &[u8], kind: &str) {
    let digest = sha256(data);
    let body = form(
        &[
            (":action", "file_upload"),
            ("protocol_version", "1"),
            ("metadata_version", "2.1"),
            ("name", "tool"),
            ("version", "0.3.0"),
            ("filetype", kind),
            ("sha256_digest", &digest),
        ],
        &[("content", file, data)],
    );
    send(
        fixture,
        Method::POST,
        &format!("/repository/{repository}/"),
        &multipart(),
        body,
    )
    .await;
}

#[tokio::test]
async fn pypi_shows_projects_and_versions_without_metadata_files() {
    let fixture = ready().await;
    create(&fixture, "pypi", "hosted", hosted("python")).await;
    let sdist = b"\x1f\x8b\x08\x00sdist of tool".to_vec();
    upload_python(&fixture, "python", "tool-0.3.0.tar.gz", &sdist, "sdist").await;
    let built = wheel("tool", "0.3.0");
    upload_python(
        &fixture,
        "python",
        "tool-0.3.0-py3-none-any.whl",
        &built,
        "bdist_wheel",
    )
    .await;

    assert_eq!(
        listing(&fixture, "python", "/").await,
        triples(&[("folder", "tool", "/tool/")])
    );
    assert_eq!(
        listing(&fixture, "python", "/tool/").await,
        triples(&[("component", "0.3.0", "/tool/0.3.0/")])
    );
    assert_eq!(
        listing(&fixture, "python", "/tool/0.3.0/").await,
        triples(&[
            (
                "asset",
                "tool-0.3.0-py3-none-any.whl",
                "/tool/0.3.0/tool-0.3.0-py3-none-any.whl"
            ),
            (
                "asset",
                "tool-0.3.0.tar.gz",
                "/tool/0.3.0/tool-0.3.0.tar.gz"
            ),
        ])
    );
}

fn chart(name: &str, version: &str) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let yaml = format!("apiVersion: v2\nname: {name}\nversion: {version}\ndescription: probe\n");
    let mut header = tar::Header::new_gnu();
    header.set_size(yaml.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, format!("{name}/Chart.yaml"), yaml.as_bytes())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

#[tokio::test]
async fn helm_shows_charts_by_name_and_version_and_the_index() {
    let fixture = ready().await;
    create(&fixture, "helm", "hosted", hosted("charts")).await;
    for (name, version) in [("demo", "0.1.0"), ("demo", "0.2.0"), ("other", "1.0.0")] {
        send(
            &fixture,
            Method::PUT,
            &format!("/repository/charts/{name}-{version}.tgz"),
            "application/gzip",
            chart(name, version),
        )
        .await;
    }
    assert_eq!(
        listing(&fixture, "charts", "/").await,
        triples(&[
            ("folder", "demo", "/demo/"),
            ("folder", "other", "/other/"),
            ("asset", "index.yaml", "/index.yaml"),
        ])
    );
    let root = get(&fixture, &format!("{BROWSE}/charts?path=/"), Auth::Admin)
        .await
        .json();
    assert_eq!(root["children"][2]["generated"], true);
    assert_eq!(
        root["children"][2]["downloadUrl"],
        "http://localhost/repository/charts/index.yaml"
    );
    assert_eq!(
        listing(&fixture, "charts", "/demo/").await,
        triples(&[
            ("component", "0.1.0", "/demo/0.1.0/"),
            ("component", "0.2.0", "/demo/0.2.0/"),
        ])
    );
    assert_eq!(
        listing(&fixture, "charts", "/demo/0.2.0/").await,
        triples(&[("asset", "demo-0.2.0.tgz", "/demo/0.2.0/demo-0.2.0.tgz")])
    );
}

fn nupkg(id: &str, version: &str) -> Vec<u8> {
    let nuspec = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<package xmlns=\"http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd\">\n  <metadata>\n    <id>{id}</id>\n    <version>{version}</version>\n    <authors>Probe</authors>\n    <description>Probe package.</description>\n  </metadata>\n</package>"
    );
    zip(&[
        ("_rels/.rels", b"<Relationships/>".to_vec()),
        (&format!("{id}.nuspec"), nuspec.into_bytes()),
    ])
}

#[tokio::test]
async fn nuget_shows_packages_under_their_own_names() {
    let fixture = ready().await;
    create(&fixture, "nuget", "hosted", hosted("nugets")).await;
    for version in ["1.0.0", "1.1.0"] {
        let body = form(
            &[],
            &[("package", "package.nupkg", &nupkg("Demo.Package", version))],
        );
        send(
            &fixture,
            Method::PUT,
            "/repository/nugets/",
            &multipart(),
            body,
        )
        .await;
    }
    assert_eq!(
        listing(&fixture, "nugets", "/").await,
        triples(&[("folder", "Demo.Package", "/Demo.Package/")])
    );
    assert_eq!(
        listing(&fixture, "nugets", "/Demo.Package/").await,
        triples(&[
            ("component", "1.0.0", "/Demo.Package/1.0.0/"),
            ("component", "1.1.0", "/Demo.Package/1.1.0/"),
        ])
    );
    assert_eq!(
        listing(&fixture, "nugets", "/Demo.Package/1.1.0/").await,
        triples(&[(
            "asset",
            "demo.package.1.1.0.nupkg",
            "/Demo.Package/1.1.0/demo.package.1.1.0.nupkg"
        )])
    );
}
