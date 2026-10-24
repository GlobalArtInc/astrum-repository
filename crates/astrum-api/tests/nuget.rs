mod support;

use std::net::SocketAddr;
use std::sync::Arc;

use astrum_security::model::REALMS_KEY;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "nuget-test-boundary";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_nuget::Nuget::default()));
    fixture
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

fn nupkg(id: &str, version: &str, extra: &str) -> Vec<u8> {
    let nuspec = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
  <metadata>
    <id>{id}</id>
    <version>{version}</version>
    <authors>Astrum Team</authors>
    <description>The {id} package for tests.</description>
    <tags>astrum probe</tags>
    {extra}
  </metadata>
</package>"#
    );
    zip(&[
        ("_rels/.rels", b"<Relationships/>".to_vec()),
        (&format!("{id}.nuspec"), nuspec.into_bytes()),
        (
            "lib/net8.0/Probe.dll",
            format!("{id} {version}").into_bytes(),
        ),
    ])
}

fn symbols(id: &str, version: &str) -> Vec<u8> {
    nupkg(
        id,
        version,
        r#"<packageTypes><packageType name="SymbolsPackage" /></packageTypes>"#,
    )
}

fn form(package: &[u8]) -> Vec<u8> {
    let mut body = format!(
        "--{BOUNDARY}\r\nContent-Type: application/octet-stream\r\nContent-Disposition: form-data; name=package; filename=package.nupkg; filename*=utf-8''package.nupkg\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(package);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

async fn push(fixture: &Fixture, repository: &str, package: Vec<u8>, auth: Auth<'_>) -> Reply {
    push_with(
        fixture,
        &format!("/repository/{repository}/"),
        package,
        auth,
        &[],
    )
    .await
}

async fn push_with(
    fixture: &Fixture,
    path: &str,
    package: Vec<u8>,
    auth: Auth<'_>,
    headers: &[(&str, &str)],
) -> Reply {
    let content_type = format!("multipart/form-data; boundary=\"{BOUNDARY}\"");
    let mut all = vec![("content-type", content_type.as_str())];
    all.extend_from_slice(headers);
    fixture
        .send_body(Method::PUT, path, auth, Body::from(form(&package)), &all)
        .await
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/nuget/{kind}"),
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

async fn get_json(fixture: &Fixture, path: &str) -> Value {
    let reply = get(fixture, path).await;
    assert_eq!(reply.status, StatusCode::OK, "{path}: {}", reply.text());
    reply.json()
}

#[tokio::test]
async fn hosted_packages_are_pushed_listed_and_downloaded_over_v3() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("nuget-hosted", "ALLOW")).await;
    let first = nupkg(
        "Astrum.Probe",
        "1.0",
        r#"<dependencies><group targetFramework="net8.0"><dependency id="Newtonsoft.Json" version="13.0.1" /></group></dependencies>"#,
    );
    for package in [
        first.clone(),
        nupkg("Astrum.Probe", "1.1.0-beta.1", ""),
        nupkg("Astrum.Probe", "2.0.0", ""),
    ] {
        let reply = push(&fixture, "nuget-hosted", package, Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    let base = "http://localhost/repository/nuget-hosted";

    let index = get_json(&fixture, "/repository/nuget-hosted/index.json").await;
    assert_eq!(index["version"], "3.0.0");
    let kinds: Vec<(&str, &str)> = index["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| {
            (
                resource["@type"].as_str().unwrap(),
                resource["@id"].as_str().unwrap(),
            )
        })
        .collect();
    for (kind, id) in [
        ("PackageBaseAddress/3.0.0", format!("{base}/v3/content/")),
        ("PackagePublish/2.0.0", format!("{base}/")),
        ("SymbolPackagePublish/4.9.0", format!("{base}/")),
        (
            "RegistrationsBaseUrl/3.6.0",
            format!("{base}/v3/registration/"),
        ),
        ("SearchQueryService/3.5.0", format!("{base}/v3/query")),
        (
            "SearchAutocompleteService",
            format!("{base}/v3/autocomplete"),
        ),
        ("LegacyGallery/2.0.0", format!("{base}/")),
    ] {
        assert!(kinds.contains(&(kind, id.as_str())), "{kind} {id}");
    }

    let versions = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/content/ASTRUM.PROBE/index.json",
    )
    .await;
    assert_eq!(
        versions,
        json!({"versions": ["1.0.0", "1.1.0-beta.1", "2.0.0"]})
    );
    assert_eq!(
        get(
            &fixture,
            "/repository/nuget-hosted/v3/content/nothing/index.json"
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );

    let download = get(
        &fixture,
        "/repository/nuget-hosted/v3/content/astrum.probe/1.0.0/astrum.probe.1.0.0.nupkg",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK);
    assert_eq!(download.body, first);
    assert_eq!(download.headers[header::CONTENT_TYPE], "application/zip");
    let by_asset = get(
        &fixture,
        "/repository/nuget-hosted/astrum.probe/1.0.0/astrum.probe.1.0.0.nupkg",
    )
    .await;
    assert_eq!(by_asset.body, first);
    let v2 = get(
        &fixture,
        "/repository/nuget-hosted/package/Astrum.Probe/1.0.0",
    )
    .await;
    assert_eq!(v2.body, first);
    let nuspec = get(
        &fixture,
        "/repository/nuget-hosted/v3/content/astrum.probe/1.0.0/astrum.probe.nuspec",
    )
    .await;
    assert_eq!(nuspec.status, StatusCode::OK);
    assert!(nuspec.text().contains("<id>Astrum.Probe</id>"));

    let registration = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/registration/astrum.probe/index.json",
    )
    .await;
    assert_eq!(
        registration["@id"],
        format!("{base}/v3/registration/astrum.probe/index.json")
    );
    let page = &registration["items"][0];
    assert_eq!(page["count"], 3);
    assert_eq!(page["lower"], "1.0.0");
    assert_eq!(page["upper"], "2.0.0");
    let entry = &page["items"][0]["catalogEntry"];
    assert_eq!(entry["id"], "Astrum.Probe");
    assert_eq!(entry["version"], "1.0.0");
    assert_eq!(entry["authors"], "Astrum Team");
    assert_eq!(entry["tags"], json!(["astrum", "probe"]));
    assert_eq!(
        entry["packageContent"],
        format!("{base}/v3/content/astrum.probe/1.0.0/astrum.probe.1.0.0.nupkg")
    );
    assert_eq!(
        entry["dependencyGroups"][0]["dependencies"][0],
        json!({
            "@id": format!("{base}/v3/catalog/astrum.probe/1.0.0.json#dependencygroup/net8.0/newtonsoft.json"),
            "@type": "PackageDependency",
            "id": "Newtonsoft.Json",
            "range": "[13.0.1, )",
            "registration": format!("{base}/v3/registration/newtonsoft.json/index.json"),
        })
    );
    let leaf = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/registration/astrum.probe/2.0.0.json",
    )
    .await;
    assert_eq!(leaf["listed"], true);
    assert_eq!(
        leaf["catalogEntry"],
        format!("{base}/v3/catalog/astrum.probe/2.0.0.json")
    );
    let catalog = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/catalog/astrum.probe/2.0.0.json",
    )
    .await;
    assert_eq!(catalog["@type"], "PackageDetails");

    let query = get_json(&fixture, "/repository/nuget-hosted/v3/query?q=probe").await;
    assert_eq!(query["totalHits"], 1);
    let hit = &query["data"][0];
    assert_eq!(hit["id"], "Astrum.Probe");
    assert_eq!(hit["version"], "2.0.0");
    assert_eq!(hit["versions"].as_array().unwrap().len(), 2);
    let all = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/query?q=&prerelease=true&semVerLevel=2.0.0",
    )
    .await;
    assert_eq!(all["data"][0]["versions"].as_array().unwrap().len(), 3);
    let none = get_json(&fixture, "/repository/nuget-hosted/v3/query?q=nothing").await;
    assert_eq!(none, json!({"totalHits": 0, "data": []}));
    let ids = get_json(&fixture, "/repository/nuget-hosted/v3/autocomplete?q=astr").await;
    assert_eq!(ids, json!({"totalHits": 1, "data": ["Astrum.Probe"]}));
    let versions = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/autocomplete?id=astrum.probe&prerelease=true&semVerLevel=2.0.0",
    )
    .await;
    assert_eq!(versions["data"], json!(["1.0.0", "1.1.0-beta.1", "2.0.0"]));

    let components = get_json(
        &fixture,
        "/service/rest/v1/search?repository=nuget-hosted&nuget.id=Astrum.Probe",
    )
    .await;
    let found = components["items"].as_array().unwrap();
    assert_eq!(found.len(), 3);
    assert_eq!(found[0]["name"], "Astrum.Probe");
    assert_eq!(found[0]["group"], "");
    assert_eq!(
        found[0]["assets"][0]["path"],
        "/astrum.probe/1.0.0/astrum.probe.1.0.0.nupkg"
    );
    assert_eq!(found[0]["assets"][0]["contentType"], "application/zip");
    let tagged = get_json(&fixture, "/service/rest/v1/search?nuget.tags=probe").await;
    assert_eq!(tagged["items"].as_array().unwrap().len(), 3);
    let untagged = get_json(&fixture, "/service/rest/v1/search?nuget.tags=nothing").await;
    assert_eq!(untagged["items"].as_array().unwrap().len(), 0);

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/nuget-hosted/Astrum.Probe/1.1.0-beta.1",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let again = fixture
        .call(
            Method::DELETE,
            "/repository/nuget-hosted/Astrum.Probe/1.1.0-beta.1",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let versions = get_json(
        &fixture,
        "/repository/nuget-hosted/v3/content/astrum.probe/index.json",
    )
    .await;
    assert_eq!(versions, json!({"versions": ["1.0.0", "2.0.0"]}));
}

#[tokio::test]
async fn the_v2_feed_answers_like_a_gallery() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("nuget-v2", "ALLOW")).await;
    for (id, version) in [
        ("Astrum.Probe", "1.0.0"),
        ("Astrum.Probe", "1.1.0-beta.1"),
        ("Astrum.Probe", "2.0.0"),
        ("Other.Tool", "0.1.0"),
    ] {
        let reply = push(&fixture, "nuget-v2", nupkg(id, version, ""), Auth::Admin).await;
        assert_eq!(reply.status, StatusCode::CREATED);
    }
    let service = get(&fixture, "/repository/nuget-v2/").await;
    assert_eq!(
        service.headers[header::CONTENT_TYPE],
        "application/xml;charset=utf-8"
    );
    assert!(service.text().contains(r#"<collection href="Packages">"#));
    let metadata = get(&fixture, "/repository/nuget-v2/$metadata").await;
    assert!(
        metadata
            .text()
            .contains(r#"<FunctionImport Name="FindPackagesById""#)
    );

    let by_id = get(
        &fixture,
        "/repository/nuget-v2/FindPackagesById()?id='astrum.probe'",
    )
    .await;
    assert_eq!(
        by_id.headers[header::CONTENT_TYPE],
        "application/atom+xml;type=feed;charset=utf-8"
    );
    let text = by_id.text();
    assert!(text.contains("<m:count>2</m:count>"), "{text}");
    assert!(
        text.contains(r#"src="http://localhost/repository/nuget-v2/package/Astrum.Probe/2.0.0""#)
    );
    assert!(text.contains(r#"<d:IsLatestVersion m:type="Edm.Boolean">true</d:IsLatestVersion>"#));
    assert!(text.contains("<d:PackageHashAlgorithm>SHA512</d:PackageHashAlgorithm>"));
    let semver2 = get(
        &fixture,
        "/repository/nuget-v2/FindPackagesById()?id='Astrum.Probe'&semVerLevel=2.0.0",
    )
    .await;
    assert!(semver2.text().contains("<m:count>3</m:count>"));
    let count = get(
        &fixture,
        "/repository/nuget-v2/FindPackagesById()/$count?id='Astrum.Probe'",
    )
    .await;
    assert_eq!(
        (
            count.text().as_str(),
            count.headers[header::CONTENT_TYPE].to_str().unwrap()
        ),
        ("2", "text/plain")
    );

    let search = get(
        &fixture,
        "/repository/nuget-v2/Search()?searchTerm='astrum.probe'&targetFramework=''&includePrerelease=false",
    )
    .await;
    let text = search.text();
    assert!(text.contains("<m:count>1</m:count>"), "{text}");
    assert!(text.contains("<d:Version>2.0.0</d:Version>"));
    let latest = get(
        &fixture,
        "/repository/nuget-v2/Packages()?$filter=IsLatestVersion&$orderby=Id%20desc",
    )
    .await;
    let text = latest.text();
    assert!(text.find("Other.Tool").unwrap() < text.find("Astrum.Probe").unwrap());
    let filtered = get(
        &fixture,
        "/repository/nuget-v2/Packages()?$filter=tolower(Id)%20eq%20'other.tool'",
    )
    .await;
    assert!(filtered.text().contains("<m:count>1</m:count>"));
    let entry = get(
        &fixture,
        "/repository/nuget-v2/Packages(Id='Astrum.Probe',Version='1.0.0')",
    )
    .await;
    assert_eq!(entry.status, StatusCode::OK);
    assert_eq!(
        entry.headers[header::CONTENT_TYPE],
        "application/atom+xml;type=entry;charset=utf-8"
    );
    assert!(entry.text().contains("<d:Version>1.0.0</d:Version>"));
    let missing = get(
        &fixture,
        "/repository/nuget-v2/Packages(Id='Astrum.Probe',Version='9.9.9')",
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let paged = get(&fixture, "/repository/nuget-v2/Packages()?$top=1&$skip=1").await;
    assert_eq!(paged.text().matches("<entry>").count(), 1);

    let pushed_v2 = push_with(
        &fixture,
        "/repository/nuget-v2/api/v2/package/",
        nupkg("Pushed.ByV2", "1.0.0", ""),
        Auth::Admin,
        &[],
    )
    .await;
    assert_eq!(pushed_v2.status, StatusCode::CREATED);
    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/nuget-v2/api/v2/package/Pushed.ByV2/1.0.0",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn pushes_need_rights_and_take_nuget_api_keys() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("keys", "ALLOW")).await;
    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;

    let anonymous = push(&fixture, "keys", nupkg("A", "1.0.0", ""), Auth::None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    let reader = push(
        &fixture,
        "keys",
        nupkg("A", "1.0.0", ""),
        Auth::Basic("reader", "reader-password-1"),
    )
    .await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);

    let revealed = fixture
        .call(
            Method::POST,
            "/service/rest/internal/ui/user/nuget-api-key",
            Auth::Admin,
            Some(json!({"password": fixture.admin_password})),
        )
        .await;
    assert_eq!(revealed.status, StatusCode::OK, "{}", revealed.text());
    let key = revealed.json()["apiKey"].as_str().unwrap().to_owned();
    assert_eq!(key.len(), 36);

    let inactive = push_with(
        &fixture,
        "/repository/keys/",
        nupkg("A", "1.0.0", ""),
        Auth::None,
        &[("x-nuget-apikey", &key)],
    )
    .await;
    assert_eq!(inactive.status, StatusCode::UNAUTHORIZED);
    let state = get_json(&fixture, "/service/rest/internal/ui/state").await;
    assert_eq!(state["nugetApiKeyRealm"], false);

    astrum_db::setting::put(
        fixture.state.db().conn(),
        REALMS_KEY,
        &["AstrumAuthenticatingRealm", "NuGetApiKey"],
    )
    .await
    .unwrap();
    fixture.state.security().changed().await.unwrap();
    let state = get_json(&fixture, "/service/rest/internal/ui/state").await;
    assert_eq!(state["nugetApiKeyRealm"], true);
    let with_key = push_with(
        &fixture,
        "/repository/keys/",
        nupkg("A", "1.0.0", ""),
        Auth::None,
        &[("x-nuget-apikey", &key)],
    )
    .await;
    assert_eq!(with_key.status, StatusCode::CREATED, "{}", with_key.text());
    let as_password = fixture
        .call(
            Method::GET,
            "/repository/keys/index.json",
            Auth::Basic("admin", &key),
            None,
        )
        .await;
    assert_eq!(as_password.status, StatusCode::OK);
    let someone_elses = fixture
        .call(
            Method::GET,
            "/repository/keys/index.json",
            Auth::Basic("reader", &key),
            None,
        )
        .await;
    assert_eq!(someone_elses.status, StatusCode::UNAUTHORIZED);
    let wrong = push_with(
        &fixture,
        "/repository/keys/",
        nupkg("A", "1.0.1", ""),
        Auth::None,
        &[("x-nuget-apikey", "00000000-0000-0000-0000-000000000000")],
    )
    .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let deleted_with_key = fixture
        .send(
            Method::DELETE,
            "/repository/keys/A/1.0.0",
            Auth::None,
            None,
            &[("x-nuget-apikey", &key)],
        )
        .await;
    assert_eq!(deleted_with_key.status, StatusCode::NO_CONTENT);

    let status = get_json(&fixture, "/service/rest/internal/ui/user/nuget-api-key").await;
    assert_eq!(status["realmActive"], true);
    assert!(status["createdAt"].is_string());
    assert!(status["lastUsedAt"].is_string());
    let wrong_password = fixture
        .call(
            Method::POST,
            "/service/rest/internal/ui/user/nuget-api-key/reset",
            Auth::Admin,
            Some(json!({"password": "wrong"})),
        )
        .await;
    assert_eq!(wrong_password.status, StatusCode::BAD_REQUEST);
    assert_eq!(wrong_password.json()[0]["id"], "password");
    let reset = fixture
        .call(
            Method::POST,
            "/service/rest/internal/ui/user/nuget-api-key/reset",
            Auth::Admin,
            Some(json!({"password": fixture.admin_password})),
        )
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    let status = get_json(&fixture, "/service/rest/internal/ui/user/nuget-api-key").await;
    assert_eq!(status["createdAt"], Value::Null);
    let again = fixture
        .call(
            Method::POST,
            "/service/rest/internal/ui/user/nuget-api-key",
            Auth::Admin,
            Some(json!({"password": fixture.admin_password})),
        )
        .await;
    let fresh = again.json()["apiKey"].as_str().unwrap().to_owned();
    assert_ne!(fresh, key);
    let stale = push_with(
        &fixture,
        "/repository/keys/",
        nupkg("A", "2.0.0", ""),
        Auth::None,
        &[("x-nuget-apikey", &key)],
    )
    .await;
    assert_eq!(stale.status, StatusCode::UNAUTHORIZED);
    let audit = get_json(
        &fixture,
        "/service/rest/internal/audit?domain=security.nugetapikey",
    )
    .await;
    let kinds: Vec<&str> = audit["items"]
        .as_array()
        .or_else(|| audit.as_array())
        .unwrap()
        .iter()
        .filter_map(|event| event["kind"].as_str().or_else(|| event["type"].as_str()))
        .collect();
    assert!(
        kinds.contains(&"created") && kinds.contains(&"deleted"),
        "{audit}"
    );

    let reader_key = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/user/nuget-api-key",
            Auth::Basic("reader", "reader-password-1"),
            None,
        )
        .await;
    assert_eq!(reader_key.status, StatusCode::FORBIDDEN);
    let stranger = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/user/nuget-api-key",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(stranger.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_ticket_endpoints_of_the_old_ui_still_issue_keys() {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    let fixture = ready().await;
    let credentials = json!({
        "u": STANDARD.encode("admin"),
        "p": STANDARD.encode(&fixture.admin_password),
    });
    let ticket = fixture
        .send(
            Method::POST,
            "/service/rest/wonderland/authenticate",
            Auth::Admin,
            Some(credentials),
            &[("accept", "application/json")],
        )
        .await;
    assert_eq!(ticket.status, StatusCode::OK, "{}", ticket.text());
    let token = STANDARD.encode(ticket.json()["t"].as_str().unwrap());
    let token: String = url::form_urlencoded::byte_serialize(token.as_bytes()).collect();
    let missing = get(&fixture, "/service/rest/internal/nuget-api-key").await;
    assert_eq!(missing.status, StatusCode::FORBIDDEN);
    let forged = get(
        &fixture,
        "/service/rest/internal/nuget-api-key?authToken=Zm9v",
    )
    .await;
    assert_eq!(forged.status, StatusCode::FORBIDDEN);
    let key = get_json(
        &fixture,
        &format!("/service/rest/internal/nuget-api-key?authToken={token}"),
    )
    .await;
    assert_eq!(key["apiKey"].as_str().unwrap().len(), 36);
    let reset = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/internal/nuget-api-key?authToken={token}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    let fresh = get_json(
        &fixture,
        &format!("/service/rest/internal/nuget-api-key?authToken={token}"),
    )
    .await;
    assert_ne!(fresh["apiKey"], key["apiKey"]);
    let wrong = fixture
        .send(
            Method::POST,
            "/service/rest/wonderland/authenticate",
            Auth::Admin,
            Some(json!({"u": STANDARD.encode("admin"), "p": STANDARD.encode("nope")})),
            &[],
        )
        .await;
    assert_eq!(wrong.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn write_policies_symbols_and_broken_packages() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("once", "ALLOW_ONCE")).await;
    let early = push(&fixture, "once", symbols("Sym", "1.0.0"), Auth::Admin).await;
    assert_eq!(early.status, StatusCode::BAD_REQUEST);
    let package = nupkg("Sym", "1.0.0", "");
    assert_eq!(
        push(&fixture, "once", package.clone(), Auth::Admin)
            .await
            .status,
        StatusCode::CREATED
    );
    let twice = push(&fixture, "once", nupkg("Sym", "1.0.0", ""), Auth::Admin).await;
    assert_eq!(twice.status, StatusCode::CONFLICT);
    let symbol_package = symbols("Sym", "1.0.0");
    assert_eq!(
        push(&fixture, "once", symbol_package.clone(), Auth::Admin)
            .await
            .status,
        StatusCode::CREATED
    );
    let served = get(
        &fixture,
        "/repository/once/v3/content/sym/1.0.0/sym.1.0.0.snupkg",
    )
    .await;
    assert_eq!(served.body, symbol_package);
    let package_still = get(
        &fixture,
        "/repository/once/v3/content/sym/1.0.0/sym.1.0.0.nupkg",
    )
    .await;
    assert_eq!(package_still.body, package);
    let assets = get_json(&fixture, "/service/rest/v1/search?repository=once").await;
    assert_eq!(assets["items"][0]["assets"].as_array().unwrap().len(), 2);

    let garbage = fixture
        .send_body(
            Method::PUT,
            "/repository/once/",
            Auth::Admin,
            Body::from(form(b"not a zip at all, sorry")),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert!(
        garbage.text().starts_with("Invalid NuGet package"),
        "{}",
        garbage.text()
    );
    let no_nuspec = push(
        &fixture,
        "once",
        zip(&[("lib/a.dll", b"x".to_vec())]),
        Auth::Admin,
    )
    .await;
    assert_eq!(
        no_nuspec.text(),
        "Invalid NuGet package: The package does not contain a .nuspec file"
    );
    let raw_body = fixture
        .send_body(
            Method::PUT,
            "/repository/once/",
            Auth::Admin,
            Body::from(nupkg("Raw.Body", "1.0.0", "")),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(raw_body.status, StatusCode::CREATED);
    let bad_id = push(&fixture, "once", nupkg("bad id", "1.0.0", ""), Auth::Admin).await;
    assert_eq!(bad_id.status, StatusCode::BAD_REQUEST);

    let blobs = astrum_content::store::count(fixture.state.db().conn(), "once")
        .await
        .unwrap();
    assert_eq!(blobs, 3);
}

async fn serve(fixture: &Fixture) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = astrum_api::router(fixture.state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    address
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

fn proxy(name: &str, remote: &str, version: &str, password: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {
            "blocked": false,
            "autoBlock": false,
            "authentication": {"type": "username", "username": "admin", "password": password}
        },
        "nugetProxy": {"queryCacheItemMaxAge": 3600, "nugetVersion": version}
    })
}

#[tokio::test]
async fn proxies_rewrite_a_v3_upstream_and_cache_its_packages() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let address = serve(&fixture).await;
    create(&fixture, "hosted", hosted("origin", "ALLOW")).await;
    let package = nupkg("Remote.Lib", "1.2.3", "");
    assert_eq!(
        push(&fixture, "origin", package.clone(), Auth::Admin)
            .await
            .status,
        StatusCode::CREATED
    );
    create(
        &fixture,
        "proxy",
        proxy(
            "mirror",
            &format!("http://{address}/repository/origin/index.json"),
            "V3",
            &fixture.admin_password,
        ),
    )
    .await;
    let base = "http://localhost/repository/mirror";

    let index = get_json(&fixture, "/repository/mirror/index.json").await;
    let kinds: Vec<&str> = index["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["@type"].as_str().unwrap())
        .collect();
    assert!(!kinds.contains(&"PackagePublish/2.0.0"));
    assert!(!kinds.contains(&"LegacyGallery/2.0.0"));
    assert!(
        index["resources"]
            .as_array()
            .unwrap()
            .iter()
            .all(|resource| resource["@id"].as_str().unwrap().starts_with(base))
    );

    let versions = get_json(
        &fixture,
        "/repository/mirror/v3/content/remote.lib/index.json",
    )
    .await;
    assert_eq!(versions, json!({"versions": ["1.2.3"]}));
    let registration = get_json(
        &fixture,
        "/repository/mirror/v3/registration/remote.lib/index.json",
    )
    .await;
    assert_eq!(
        registration["@id"],
        format!("{base}/v3/registration/remote.lib/index.json")
    );
    assert_eq!(
        registration["items"][0]["items"][0]["packageContent"],
        format!("{base}/v3/content/remote.lib/1.2.3/remote.lib.1.2.3.nupkg")
    );
    let query = get_json(&fixture, "/repository/mirror/v3/query?q=remote").await;
    assert_eq!(query["totalHits"], 1);
    assert_eq!(
        query["data"][0]["registration"],
        format!("{base}/v3/registration/remote.lib/index.json")
    );

    let download = get(
        &fixture,
        "/repository/mirror/v3/content/remote.lib/1.2.3/remote.lib.1.2.3.nupkg",
    )
    .await;
    assert_eq!(download.status, StatusCode::OK, "{}", download.text());
    assert_eq!(download.body, package);
    let cached = get_json(
        &fixture,
        "/service/rest/v1/search?repository=mirror&nuget.id=Remote.Lib",
    )
    .await;
    assert_eq!(cached["items"][0]["name"], "Remote.Lib");
    assert_eq!(cached["items"][0]["version"], "1.2.3");
    assert_eq!(
        get(&fixture, "/repository/mirror/v3/content/missing/index.json")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let refused = push(&fixture, "mirror", nupkg("X", "1.0.0", ""), Auth::Admin).await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);

    create(&fixture, "hosted", hosted("elsewhere", "ALLOW")).await;
    push(
        &fixture,
        "elsewhere",
        nupkg("Moved.Lib", "1.0.0", ""),
        Auth::Admin,
    )
    .await;
    let moved = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/nuget/proxy/mirror",
            Auth::Admin,
            Some(proxy(
                "mirror",
                &format!("http://{address}/repository/elsewhere/index.json"),
                "V3",
                &fixture.admin_password,
            )),
        )
        .await;
    assert_eq!(moved.status, StatusCode::NO_CONTENT, "{}", moved.text());
    let versions = get_json(
        &fixture,
        "/repository/mirror/v3/content/moved.lib/index.json",
    )
    .await;
    assert_eq!(versions, json!({"versions": ["1.0.0"]}));
}

#[tokio::test]
async fn v2_proxies_pass_the_feed_through_and_cache_downloads() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let address = serve(&fixture).await;
    create(&fixture, "hosted", hosted("v2-origin", "ALLOW")).await;
    let package = nupkg("Old.School", "3.0.0", "");
    push(&fixture, "v2-origin", package.clone(), Auth::Admin).await;
    create(
        &fixture,
        "proxy",
        proxy(
            "v2-mirror",
            &format!("http://{address}/repository/v2-origin/"),
            "V2",
            &fixture.admin_password,
        ),
    )
    .await;
    let feed = get(
        &fixture,
        "/repository/v2-mirror/FindPackagesById()?id='Old.School'",
    )
    .await;
    assert_eq!(feed.status, StatusCode::OK, "{}", feed.text());
    let text = feed.text();
    assert!(
        text.contains(r#"src="http://localhost/repository/v2-mirror/package/Old.School/3.0.0""#),
        "{text}"
    );
    assert!(!text.contains(&address.to_string()));
    let count = get(
        &fixture,
        "/repository/v2-mirror/FindPackagesById()/$count?id='Old.School'",
    )
    .await;
    assert_eq!(count.text(), "1");
    let download = get(&fixture, "/repository/v2-mirror/package/Old.School/3.0.0").await;
    assert_eq!(download.body, package);
    assert_eq!(
        get(&fixture, "/repository/v2-mirror/index.json")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn groups_merge_their_members_and_the_first_one_wins() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let address = serve(&fixture).await;
    create(&fixture, "hosted", hosted("first", "ALLOW")).await;
    create(&fixture, "hosted", hosted("second", "ALLOW")).await;
    create(&fixture, "hosted", hosted("far", "ALLOW")).await;
    let winner = nupkg("Shared", "2.0.0", "<title>from first</title>");
    push(&fixture, "first", nupkg("Shared", "1.0.0", ""), Auth::Admin).await;
    push(&fixture, "first", winner.clone(), Auth::Admin).await;
    push(
        &fixture,
        "second",
        nupkg("Shared", "2.0.0", "<title>from second</title>"),
        Auth::Admin,
    )
    .await;
    push(
        &fixture,
        "second",
        nupkg("Shared", "3.0.0", ""),
        Auth::Admin,
    )
    .await;
    push(
        &fixture,
        "second",
        nupkg("Only.Second", "1.0.0", ""),
        Auth::Admin,
    )
    .await;
    push(&fixture, "far", nupkg("Far.Away", "5.0.0", ""), Auth::Admin).await;
    create(
        &fixture,
        "proxy",
        proxy(
            "far-mirror",
            &format!("http://{address}/repository/far/index.json"),
            "V3",
            &fixture.admin_password,
        ),
    )
    .await;
    create(
        &fixture,
        "group",
        json!({
            "name": "all",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "group": {"memberNames": ["first", "second", "far-mirror"]}
        }),
    )
    .await;
    let base = "http://localhost/repository/all";

    let index = get_json(&fixture, "/repository/all/index.json").await;
    let kinds: Vec<&str> = index["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["@type"].as_str().unwrap())
        .collect();
    assert!(!kinds.contains(&"PackagePublish/2.0.0"));
    assert!(kinds.contains(&"LegacyGallery/2.0.0"));

    let versions = get_json(&fixture, "/repository/all/v3/content/shared/index.json").await;
    assert_eq!(versions, json!({"versions": ["1.0.0", "2.0.0", "3.0.0"]}));
    let registration = get_json(
        &fixture,
        "/repository/all/v3/registration/shared/index.json",
    )
    .await;
    let items = registration["items"][0]["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[1]["catalogEntry"]["title"], "from first");
    assert_eq!(
        items[1]["packageContent"],
        format!("{base}/v3/content/shared/2.0.0/shared.2.0.0.nupkg")
    );
    let download = get(
        &fixture,
        "/repository/all/v3/content/shared/2.0.0/shared.2.0.0.nupkg",
    )
    .await;
    assert_eq!(download.body, winner);
    let far = get_json(
        &fixture,
        "/repository/all/v3/registration/far.away/index.json",
    )
    .await;
    assert_eq!(
        far["items"][0]["items"][0]["packageContent"],
        format!("{base}/v3/content/far.away/5.0.0/far.away.5.0.0.nupkg")
    );
    let far_package = get(
        &fixture,
        "/repository/all/v3/content/far.away/5.0.0/far.away.5.0.0.nupkg",
    )
    .await;
    assert_eq!(far_package.status, StatusCode::OK);

    let query = get_json(&fixture, "/repository/all/v3/query?q=").await;
    let ids: Vec<&str> = query["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(query["totalHits"], 3, "{query}");
    for id in ["Shared", "Only.Second", "Far.Away"] {
        assert!(ids.contains(&id), "{ids:?}");
    }
    let v2 = get(&fixture, "/repository/all/FindPackagesById()?id='Shared'").await;
    let text = v2.text();
    assert!(text.contains("<m:count>3</m:count>"), "{text}");
    assert!(text.contains("<d:Title>from first</d:Title>"));
    assert!(!text.contains("from second"));
    let refused = push(&fixture, "all", nupkg("X", "1.0.0", ""), Auth::Admin).await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn content_selectors_judge_the_request_path() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("guarded", "ALLOW")).await;
    push(
        &fixture,
        "guarded",
        nupkg("Open.Pkg", "1.0.0", ""),
        Auth::Admin,
    )
    .await;
    let selector = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/content-selectors",
            Auth::Admin,
            Some(json!({"name": "open-only", "expression": "format == \"nuget\" and path =^ \"/open.pkg/\""})),
        )
        .await;
    assert!(selector.status.is_success(), "{}", selector.text());
    let privilege = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/privileges/repository-content-selector",
            Auth::Admin,
            Some(json!({"name": "open-read", "description": "x", "format": "nuget", "repository": "guarded", "contentSelector": "open-only", "actions": ["READ"]})),
        )
        .await;
    assert!(privilege.status.is_success(), "{}", privilege.text());
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "open-role", "name": "open-role", "privileges": ["open-read"], "roles": []})),
        )
        .await;
    assert!(role.status.is_success(), "{}", role.text());
    fixture
        .add_user("limited", "limited-password-1", &["open-role"])
        .await;
    let limited = Auth::Basic("limited", "limited-password-1");
    let asset = fixture
        .call(
            Method::GET,
            "/repository/guarded/open.pkg/1.0.0/open.pkg.1.0.0.nupkg",
            limited,
            None,
        )
        .await;
    assert_eq!(asset.status, StatusCode::OK);
    let index = fixture
        .call(
            Method::GET,
            "/repository/guarded/index.json",
            Auth::Basic("limited", "limited-password-1"),
            None,
        )
        .await;
    assert_eq!(index.status, StatusCode::FORBIDDEN);
}
