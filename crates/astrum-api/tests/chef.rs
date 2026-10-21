mod support;

use std::io::Write;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BOUNDARY: &str = "chefboundary0123456789";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_chef::Chef::default()));
    fixture
}

async fn create(fixture: &Fixture, kind: &str, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("/service/rest/v1/repositories/chef/{kind}"),
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

fn tar_gz(entries: &[(&str, Option<&[u8]>)]) -> Vec<u8> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in entries {
        let mut header = tar::Header::new_ustar();
        match data {
            Some(data) => {
                header.set_entry_type(tar::EntryType::Regular);
                header.set_size(data.len() as u64);
                header.set_mode(0o644);
                header.set_mtime(1_790_000_000);
                header.set_cksum();
                builder.append_data(&mut header, path, *data).unwrap();
            }
            None => {
                header.set_entry_type(tar::EntryType::Directory);
                header.set_size(0);
                header.set_mode(0o755);
                header.set_mtime(1_790_000_000);
                header.set_cksum();
                builder
                    .append_data(&mut header, path, std::io::empty())
                    .unwrap();
            }
        }
    }
    builder.into_inner().unwrap().finish().unwrap()
}

fn metadata_rb(name: &str, version: &str, dependencies: &[(&str, &str)]) -> String {
    let mut text = format!(
        "name '{name}'\nmaintainer 'Acme Ops'\nmaintainer_email 'ops@example.org'\nlicense 'Apache-2.0'\ndescription 'Installs and configures {name}'\nversion '{version}'\nchef_version '>= 17.0'\nsupports 'ubuntu', '>= 22.04'\n"
    );
    for (dependency, constraint) in dependencies {
        text.push_str(&format!("depends '{dependency}', '{constraint}'\n"));
    }
    text
}

fn metadata_json(name: &str, version: &str, dependencies: &[(&str, &str)]) -> String {
    let depends: serde_json::Map<String, Value> = dependencies
        .iter()
        .map(|(dependency, constraint)| ((*dependency).to_owned(), json!(constraint)))
        .collect();
    serde_json::to_string_pretty(&json!({
        "name": name,
        "description": format!("Installs and configures {name}"),
        "long_description": "",
        "maintainer": "Acme Ops",
        "maintainer_email": "ops@example.org",
        "license": "Apache-2.0",
        "platforms": {"ubuntu": ">= 22.04"},
        "dependencies": depends,
        "providing": {},
        "recipes": {},
        "version": version,
        "source_url": format!("https://git.example.org/{name}"),
        "issues_url": format!("https://git.example.org/{name}/issues"),
        "privacy": false,
        "chef_versions": [[">= 17.0"]],
        "ohai_versions": [],
        "gems": [],
        "eager_load_libraries": true
    }))
    .unwrap()
}

fn cookbook(name: &str, version: &str, dependencies: &[(&str, &str)]) -> Vec<u8> {
    let rb = metadata_rb(name, version, dependencies);
    let json = metadata_json(name, version, dependencies);
    let recipe = format!("package '{name}'\n");
    let readme = format!("# {name}\n\nInstalls {name}.\n");
    tar_gz(&[
        (&format!("{name}/"), None),
        (&format!("{name}/README.md"), Some(readme.as_bytes())),
        (&format!("{name}/metadata.rb"), Some(rb.as_bytes())),
        (&format!("{name}/metadata.json"), Some(json.as_bytes())),
        (&format!("{name}/recipes/"), None),
        (
            &format!("{name}/recipes/default.rb"),
            Some(recipe.as_bytes()),
        ),
    ])
}

fn ruby_only(name: &str, version: &str) -> Vec<u8> {
    let rb = metadata_rb(name, version, &[]);
    tar_gz(&[
        (&format!("{name}/"), None),
        (&format!("{name}/metadata.rb"), Some(rb.as_bytes())),
        (&format!("{name}/recipes/default.rb"), Some(b"log 'hi'\n")),
    ])
}

fn share_form(category: Option<&str>, tarball: Option<(&str, &[u8])>) -> Vec<u8> {
    let mut body = Vec::new();
    if let Some(category) = category {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"cookbook\"\r\n\r\n{}\r\n",
            json!({"category": category})
        )
        .unwrap();
    }
    if let Some((file_name, data)) = tarball {
        write!(
            body,
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"tarball\"; filename=\"{file_name}\"\r\nContent-Type: application/x-gzip\r\n\r\n"
        )
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    write!(body, "--{BOUNDARY}--\r\n").unwrap();
    body
}

async fn share(
    fixture: &Fixture,
    repository: &str,
    auth: Auth<'_>,
    extra: &[(&str, &str)],
    body: Vec<u8>,
) -> Reply {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let mut headers = vec![("content-type", content_type.as_str())];
    headers.extend_from_slice(extra);
    fixture
        .send_body(
            Method::POST,
            &format!("/repository/{repository}/api/v1/cookbooks"),
            auth,
            Body::from(body),
            &headers,
        )
        .await
}

async fn publish(fixture: &Fixture, repository: &str, category: Option<&str>, data: &[u8]) {
    let reply = share(
        fixture,
        repository,
        Auth::Admin,
        &[],
        share_form(category, Some(("cookbook.tgz", data))),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn local(address: &str) -> String {
    address.replacen("http://localhost", "", 1)
}

async fn follow(fixture: &Fixture, address: &str) -> Reply {
    assert!(address.starts_with("http://localhost/"), "{address}");
    get(fixture, &local(address)).await
}

#[tokio::test]
async fn hosted_cookbooks_publish_and_answer_the_supermarket_api() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("chef-hosted", "ALLOW_ONCE")).await;
    let base = "http://localhost/repository/chef-hosted";

    let page = fixture
        .call(Method::GET, "/repository/chef-hosted/", Auth::None, None)
        .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("default_source :supermarket"));
    let empty = get(&fixture, "/repository/chef-hosted/universe").await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.json(), json!({}));

    let first = cookbook("acme", "1.0.0", &[]);
    let shared = share(
        &fixture,
        "chef-hosted",
        Auth::Admin,
        &[],
        share_form(Some("Web Servers"), Some(("acme.tgz", &first))),
    )
    .await;
    assert_eq!(shared.status, StatusCode::CREATED, "{}", shared.text());
    let shared = shared.json();
    assert_eq!(shared["name"], "acme");
    assert_eq!(shared["category"], "Web Servers");
    assert_eq!(
        shared["latest_version"],
        format!("{base}/api/v1/cookbooks/acme/versions/1.0.0")
    );
    let second = cookbook("acme", "1.10.0", &[("apt", "~> 7.0"), ("yum", ">= 0.0.0")]);
    publish(&fixture, "chef-hosted", None, &second).await;
    let upload = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/components?repository=chef-hosted",
            Auth::Admin,
            Body::from({
                let data = cookbook("base", "0.1.0", &[("acme", ">= 1.0")]);
                let mut body = Vec::new();
                write!(
                    body,
                    "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"chef.asset\"; filename=\"base.tgz\"\r\nContent-Type: application/octet-stream\r\n\r\n"
                )
                .unwrap();
                body.extend_from_slice(&data);
                write!(body, "\r\n--{BOUNDARY}--\r\n").unwrap();
                body
            }),
            &[(
                "content-type",
                &format!("multipart/form-data; boundary={BOUNDARY}"),
            )],
        )
        .await;
    assert_eq!(upload.status, StatusCode::NO_CONTENT, "{}", upload.text());

    let again = share(
        &fixture,
        "chef-hosted",
        Auth::Admin,
        &[],
        share_form(None, Some(("acme.tgz", &first))),
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json()["error_code"], "CONFLICT");
    let ruby = share(
        &fixture,
        "chef-hosted",
        Auth::Admin,
        &[],
        share_form(None, Some(("legacy.tgz", &ruby_only("legacy", "1.0.0")))),
    )
    .await;
    assert_eq!(ruby.status, StatusCode::BAD_REQUEST);
    let ruby = ruby.json();
    assert_eq!(ruby["error_code"], "INVALID_DATA");
    assert!(
        ruby["error_messages"][0]
            .as_str()
            .unwrap()
            .contains("knife cookbook metadata from file legacy/metadata.rb"),
        "{ruby}"
    );
    let no_tarball = share(
        &fixture,
        "chef-hosted",
        Auth::Admin,
        &[],
        share_form(Some("Other"), None),
    )
    .await;
    assert_eq!(no_tarball.status, StatusCode::BAD_REQUEST);
    assert!(no_tarball.text().contains("tarball"));
    let not_gzip = share(
        &fixture,
        "chef-hosted",
        Auth::Admin,
        &[],
        share_form(None, Some(("x.tgz", b"PK\x03\x04 not a tarball"))),
    )
    .await;
    assert_eq!(not_gzip.status, StatusCode::BAD_REQUEST);
    assert!(not_gzip.text().contains("gzip"));
    let bad_json = share(&fixture, "chef-hosted", Auth::Admin, &[], {
        let mut body = format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"cookbook\"\r\n\r\nnot json\r\n"
        )
        .into_bytes();
        body.extend(share_form(
            None,
            Some(("acme.tgz", &cookbook("acme", "3.0.0", &[]))),
        ));
        body
    })
    .await;
    assert_eq!(bad_json.status, StatusCode::BAD_REQUEST);
    let anonymous = share(
        &fixture,
        "chef-hosted",
        Auth::None,
        &[],
        share_form(None, Some(("acme.tgz", &cookbook("acme", "2.0.0", &[])))),
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["error_code"], "UNAUTHORIZED");
    let signed = share(
        &fixture,
        "chef-hosted",
        Auth::None,
        &[
            ("x-ops-userid", "jane"),
            ("x-ops-sign", "algorithm=sha1;version=1.0;"),
            ("x-ops-timestamp", "2026-10-20T10:00:00Z"),
            ("x-ops-content-hash", "2jmj7l5rSw0yVb/vlWAYkK/YBwk="),
            ("x-ops-authorization-1", "c2lnbmF0dXJl"),
        ],
        share_form(
            Some("Other"),
            Some(("acme.tgz", &cookbook("acme", "2.0.0", &[]))),
        ),
    )
    .await;
    assert_eq!(signed.status, StatusCode::UNAUTHORIZED);
    let message = signed.json()["error_messages"][0]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        message.contains("signed with a Chef client key"),
        "{message}"
    );
    assert!(
        message.contains(&format!("{base}/api/v1/cookbooks")),
        "{message}"
    );

    let universe = get(&fixture, "/repository/chef-hosted/universe").await;
    assert_eq!(universe.status, StatusCode::OK);
    assert_eq!(universe.headers["content-type"], "application/json");
    let etag = universe.headers[header::ETAG].to_str().unwrap().to_owned();
    let universe = universe.json();
    assert_eq!(
        universe["acme"]["1.10.0"],
        json!({
            "location_type": "opscode",
            "location_path": format!("{base}/api/v1"),
            "download_url": format!("{base}/api/v1/cookbooks/acme/versions/1.10.0/download"),
            "dependencies": {"apt": "~> 7.0", "yum": ">= 0.0.0"},
        })
    );
    assert_eq!(universe["acme"]["1.0.0"]["dependencies"], json!({}));
    assert_eq!(universe["base"]["0.1.0"]["dependencies"]["acme"], ">= 1.0");
    assert!(universe.get("legacy").is_none());
    let unchanged = fixture
        .send(
            Method::GET,
            "/repository/chef-hosted/universe",
            Auth::Admin,
            None,
            &[("if-none-match", &etag)],
        )
        .await;
    assert_eq!(unchanged.status, StatusCode::NOT_MODIFIED);

    let entry = &universe["acme"]["1.10.0"];
    let berkshelf = follow(
        &fixture,
        &format!(
            "{}/cookbooks/acme/versions/1_10_0",
            entry["location_path"].as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(berkshelf.status, StatusCode::OK, "{}", berkshelf.text());
    let version = berkshelf.json();
    assert_eq!(version["version"], "1.10.0");
    assert_eq!(version["license"], "Apache-2.0");
    assert_eq!(version["cookbook"], format!("{base}/api/v1/cookbooks/acme"));
    assert_eq!(version["dependencies"]["apt"], "~> 7.0");
    assert_eq!(version["platforms"]["ubuntu"], ">= 22.04");
    assert_eq!(version["tarball_file_size"], second.len());
    assert!(version["published_at"].as_str().unwrap().ends_with('Z'));
    let downloaded = follow(&fixture, version["file"].as_str().unwrap()).await;
    assert_eq!(downloaded.status, StatusCode::OK);
    assert_eq!(downloaded.body, second);
    let policyfile = follow(&fixture, entry["download_url"].as_str().unwrap()).await;
    assert_eq!(policyfile.body, second);

    let found = get(&fixture, "/repository/chef-hosted/api/v1/cookbooks/acme").await;
    assert_eq!(found.status, StatusCode::OK);
    let found = found.json();
    assert_eq!(found["maintainer"], "Acme Ops");
    assert_eq!(found["description"], "Installs and configures acme");
    assert_eq!(found["category"], "Web Servers");
    assert_eq!(found["source_url"], "https://git.example.org/acme");
    assert_eq!(found["deprecated"], false);
    assert_eq!(
        found["versions"],
        json!([
            format!("{base}/api/v1/cookbooks/acme/versions/1.10.0"),
            format!("{base}/api/v1/cookbooks/acme/versions/1.0.0"),
        ])
    );
    assert!(found["metrics"]["downloads"]["total"].is_number());
    assert!(found["metrics"]["downloads"]["versions"]["1.10.0"].is_number());
    let knife = follow(&fixture, found["latest_version"].as_str().unwrap())
        .await
        .json();
    assert_eq!(knife["version"], "1.10.0");
    let latest = get(
        &fixture,
        "/repository/chef-hosted/api/v1/cookbooks/acme/versions/latest",
    )
    .await
    .json();
    assert_eq!(latest["version"], "1.10.0");
    let file = get(
        &fixture,
        "/repository/chef-hosted/cookbooks/acme/1.0.0/acme-1.0.0.tar.gz",
    )
    .await;
    assert_eq!(file.body, first);
    let missing = get(
        &fixture,
        "/repository/chef-hosted/api/v1/cookbooks/acme/versions/9.9.9",
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.json()["error_code"], "NOT_FOUND");
    assert_eq!(
        get(&fixture, "/repository/chef-hosted/api/v1/cookbooks/nothing")
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    let search = get(&fixture, "/repository/chef-hosted/api/v1/search?q=acme")
        .await
        .json();
    assert_eq!(search["total"], 2, "{search}");
    assert_eq!(search["items"][0]["cookbook_name"], "acme");
    assert_eq!(
        search["items"][0]["cookbook"],
        format!("{base}/api/v1/cookbooks/acme")
    );
    assert_eq!(search["items"][0]["cookbook_maintainer"], "Acme Ops");
    assert_eq!(search["items"][1]["cookbook_name"], "base");
    let none = get(&fixture, "/repository/chef-hosted/api/v1/search?q=zzz")
        .await
        .json();
    assert_eq!(none, json!({"start": 0, "total": 0, "items": []}));
    let list = get(
        &fixture,
        "/repository/chef-hosted/api/v1/cookbooks?start=1&items=1",
    )
    .await
    .json();
    assert_eq!(list["total"], 2);
    assert_eq!(list["start"], 1);
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["cookbook_name"], "base");
    let by_attribute = get(
        &fixture,
        "/service/rest/v1/search?format=chef&chef.category=Web%20Servers",
    )
    .await
    .json();
    assert_eq!(
        by_attribute["items"].as_array().unwrap().len(),
        2,
        "{by_attribute}"
    );
    let by_keyword = get(&fixture, "/service/rest/v1/search?format=chef&q=configures")
        .await
        .json();
    assert!(!by_keyword["items"].as_array().unwrap().is_empty());

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/chef-hosted/api/v1/cookbooks/acme/versions/1_10_0",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK, "{}", deleted.text());
    assert_eq!(deleted.json()["version"], "1.10.0");
    let universe = get(&fixture, "/repository/chef-hosted/universe")
        .await
        .json();
    assert!(universe["acme"].get("1.10.0").is_none(), "{universe}");
    assert!(universe["acme"].get("1.0.0").is_some());
    let found = get(&fixture, "/repository/chef-hosted/api/v1/cookbooks/acme")
        .await
        .json();
    assert_eq!(
        found["latest_version"],
        format!("{base}/api/v1/cookbooks/acme/versions/1.0.0")
    );

    let component = get(
        &fixture,
        "/service/rest/v1/search?repository=chef-hosted&name=base",
    )
    .await
    .json();
    let id = component["items"][0]["id"].as_str().unwrap().to_owned();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/components/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    let universe = get(&fixture, "/repository/chef-hosted/universe")
        .await
        .json();
    assert!(universe.get("base").is_none(), "{universe}");

    let unshared = fixture
        .call(
            Method::DELETE,
            "/repository/chef-hosted/api/v1/cookbooks/acme",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unshared.status, StatusCode::OK, "{}", unshared.text());
    assert_eq!(unshared.json()["name"], "acme");
    assert_eq!(
        get(&fixture, "/repository/chef-hosted/api/v1/cookbooks/acme")
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, "/repository/chef-hosted/universe")
            .await
            .json(),
        json!({})
    );
}

#[tokio::test]
async fn write_policies_and_permissions() {
    let fixture = ready().await;
    create(&fixture, "hosted", hosted("open", "ALLOW")).await;
    create(&fixture, "hosted", hosted("closed", "DENY")).await;
    create(&fixture, "group", group("all", &["open"])).await;
    let data = cookbook("acme", "1.0.0", &[]);
    publish(&fixture, "open", Some("Other"), &data).await;
    publish(&fixture, "open", None, &data).await;
    let closed = share(
        &fixture,
        "closed",
        Auth::Admin,
        &[],
        share_form(None, Some(("acme.tgz", &data))),
    )
    .await;
    assert_eq!(closed.status, StatusCode::BAD_REQUEST);
    assert!(closed.text().contains("read-only"));
    let to_group = share(
        &fixture,
        "all",
        Auth::Admin,
        &[],
        share_form(None, Some(("acme.tgz", &data))),
    )
    .await;
    assert_eq!(to_group.status, StatusCode::BAD_REQUEST);
    assert!(to_group.text().contains("hosted"));

    fixture.add_user("nobody", "nobody-password-1", &[]).await;
    let refused = fixture
        .send(
            Method::GET,
            "/repository/open/universe",
            Auth::Basic("nobody", "nobody-password-1"),
            None,
            &[],
        )
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.json()["error_code"], "FORBIDDEN");
    let anonymous = fixture
        .call(Method::GET, "/repository/open/universe", Auth::None, None)
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    assert_eq!(anonymous.json()["error_code"], "UNAUTHORIZED");
    let download = fixture
        .call(
            Method::GET,
            "/repository/open/api/v1/cookbooks/acme/versions/1.0.0/download",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(download.status, StatusCode::UNAUTHORIZED);

    fixture
        .add_user("reader", "reader-password-1", &["as-anonymous"])
        .await;
    let reader = share(
        &fixture,
        "open",
        Auth::Basic("reader", "reader-password-1"),
        &[],
        share_form(None, Some(("acme.tgz", &cookbook("acme", "1.1.0", &[])))),
    )
    .await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN);
    assert_eq!(reader.json()["error_code"], "FORBIDDEN");
    let reader_delete = fixture
        .send(
            Method::DELETE,
            "/repository/open/api/v1/cookbooks/acme/versions/1.0.0",
            Auth::Basic("reader", "reader-password-1"),
            None,
            &[],
        )
        .await;
    assert_eq!(reader_delete.status, StatusCode::FORBIDDEN);

    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "deployer-role", "name": "deployer-role", "description": "", "privileges": ["as-repository-view-chef-open-add"], "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user("deployer", "deployer-password-1", &["deployer-role"])
        .await;
    let deployed = share(
        &fixture,
        "open",
        Auth::Basic("deployer", "deployer-password-1"),
        &[],
        share_form(None, Some(("acme.tgz", &cookbook("acme", "2.0.0", &[])))),
    )
    .await;
    assert_eq!(deployed.status, StatusCode::CREATED, "{}", deployed.text());
    assert_eq!(deployed.json()["versions"].as_array().unwrap().len(), 2);
    let overwrite = share(
        &fixture,
        "open",
        Auth::Basic("deployer", "deployer-password-1"),
        &[],
        share_form(None, Some(("acme.tgz", &data))),
    )
    .await;
    assert_eq!(overwrite.status, StatusCode::FORBIDDEN);
    let unread = fixture
        .send(
            Method::GET,
            "/repository/open/api/v1/cookbooks/acme",
            Auth::Basic("deployer", "deployer-password-1"),
            None,
            &[],
        )
        .await;
    assert_eq!(unread.status, StatusCode::FORBIDDEN);

    let enabled = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    assert_eq!(enabled.status, StatusCode::OK, "{}", enabled.text());
    let open = fixture
        .call(Method::GET, "/repository/all/universe", Auth::None, None)
        .await;
    assert_eq!(open.status, StatusCode::OK, "{}", open.text());
    assert_eq!(
        open.json()["acme"]["2.0.0"]["download_url"],
        "http://localhost/repository/all/api/v1/cookbooks/acme/versions/2.0.0/download"
    );
    let fetched = fixture
        .call(
            Method::GET,
            "/repository/all/api/v1/cookbooks/acme/versions/1.0.0/download",
            Auth::None,
            None,
        )
        .await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body, data);
    let still_closed = share(
        &fixture,
        "open",
        Auth::None,
        &[],
        share_form(None, Some(("acme.tgz", &cookbook("acme", "4.0.0", &[])))),
    )
    .await;
    assert_eq!(still_closed.status, StatusCode::UNAUTHORIZED);
    let group_delete = fixture
        .call(
            Method::DELETE,
            "/repository/all/api/v1/cookbooks/acme",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(group_delete.status, StatusCode::METHOD_NOT_ALLOWED);
    let head = fixture
        .call(Method::HEAD, "/repository/open/universe", Auth::Admin, None)
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.body.is_empty());
}

#[derive(Clone, Default)]
struct Upstream {
    address: Arc<Mutex<Option<SocketAddr>>>,
    apt: Arc<Vec<u8>>,
    nginx: Arc<Vec<u8>>,
    universes: Arc<AtomicUsize>,
    tarballs: Arc<AtomicUsize>,
    searches: Arc<Mutex<Vec<String>>>,
}

async fn upstream(
    State(upstream): State<Upstream>,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
) -> Response {
    let address = upstream.address.lock().unwrap().unwrap();
    let api = format!("http://{address}/api/v1");
    let not_found = || {
        (
            StatusCode::NOT_FOUND,
            json!({"error_messages": ["Resource does not exist."], "error_code": "NOT_FOUND"})
                .to_string(),
        )
            .into_response()
    };
    let entry = |name: &str, version: &str, dependencies: Value| {
        json!({
            "location_type": "opscode",
            "location_path": api,
            "download_url": format!("{api}/cookbooks/{name}/versions/{version}/download"),
            "dependencies": dependencies,
        })
    };
    let item = |name: &str| json!({"cookbook_name": name, "cookbook_maintainer": "sous-chefs", "cookbook_description": format!("The {name} cookbook"), "cookbook": format!("{api}/cookbooks/{name}")});
    match path.as_str() {
        "universe" => {
            upstream.universes.fetch_add(1, Ordering::SeqCst);
            json!({
                "apt": {
                    "7.4.0": entry("apt", "7.4.0", json!({})),
                    "8.0.0": entry("apt", "8.0.0", json!({})),
                    "0.1.0\n": entry("apt", "0.1.0\n", json!({})),
                },
                "nginx": {"12.3.4": entry("nginx", "12.3.4", json!({"apt": ">= 7.0"}))},
            })
            .to_string()
            .into_response()
        }
        "api/v1/cookbooks/apt" => json!({
            "name": "apt",
            "maintainer": "sous-chefs",
            "description": "Configures apt",
            "category": "Other",
            "latest_version": format!("{api}/cookbooks/apt/versions/8_0_0"),
            "external_url": "https://git.example.org/apt",
            "average_rating": null,
            "created_at": "2009-10-25T23:48:48.000Z",
            "updated_at": "2026-07-09T11:35:28.537Z",
            "deprecated": false,
            "versions": [format!("{api}/cookbooks/apt/versions/8_0_0"), format!("{api}/cookbooks/apt/versions/7.4.0")],
            "metrics": {"downloads": {"total": 30, "versions": {"7.4.0": 10, "8.0.0": 20}}, "followers": 407, "collaborators": 2},
        })
        .to_string()
        .into_response(),
        "api/v1/cookbooks/apt/versions/8.0.0" | "api/v1/cookbooks/apt/versions/7.4.0" => {
            let version = path.rsplit('/').next().unwrap();
            json!({
                "license": "Apache-2.0",
                "tarball_file_size": upstream.apt.len(),
                "version": version,
                "published_at": "2026-07-09T11:35:28Z",
                "average_rating": null,
                "cookbook": format!("{api}/cookbooks/apt"),
                "file": format!("{api}/cookbooks/apt/versions/{version}/download"),
                "supports": {"ubuntu": ">= 22.04"},
            })
            .to_string()
            .into_response()
        }
        "api/v1/cookbooks/nginx/versions/12.3.4" => json!({
            "license": "Apache-2.0",
            "version": "12.3.4",
            "cookbook": format!("{api}/cookbooks/nginx"),
            "file": "/files/nginx-12.3.4.tgz",
        })
        .to_string()
        .into_response(),
        "api/v1/cookbooks/apt/versions/8.0.0/download" => {
            Redirect::to(&format!("http://{address}/files/apt.tgz?1783596928")).into_response()
        }
        "files/apt.tgz" => {
            upstream.tarballs.fetch_add(1, Ordering::SeqCst);
            upstream.apt.as_ref().clone().into_response()
        }
        "files/nginx-12.3.4.tgz" => {
            upstream.tarballs.fetch_add(1, Ordering::SeqCst);
            upstream.nginx.as_ref().clone().into_response()
        }
        "api/v1/search" | "api/v1/cookbooks" => {
            let query = query.unwrap_or_default();
            upstream
                .searches
                .lock()
                .unwrap()
                .push(format!("{path}?{query}"));
            let wanted: String = url::form_urlencoded::parse(query.as_bytes())
                .find(|(key, _)| key == "q")
                .map(|(_, value)| value.into_owned())
                .unwrap_or_default();
            let items: Vec<Value> = ["apt", "nginx"]
                .iter()
                .filter(|name| name.contains(wanted.as_str()))
                .map(|name| item(name))
                .collect();
            json!({"start": 0, "total": items.len(), "items": items})
                .to_string()
                .into_response()
        }
        _ => not_found(),
    }
}

#[tokio::test]
async fn proxies_rewrite_the_remote_and_groups_merge_members() {
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
    let state = Upstream {
        apt: Arc::new(cookbook("apt", "8.0.0", &[])),
        nginx: Arc::new(cookbook("nginx", "12.3.4", &[("apt", ">= 7.0")])),
        ..Default::default()
    };
    let app = Router::new()
        .route("/{*path}", routing::get(upstream))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    *state.address.lock().unwrap() = Some(address);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    create(
        &fixture,
        "proxy",
        json!({
            "name": "supermarket",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"remoteUrl": format!("http://{address}"), "contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true},
        }),
    )
    .await;
    let base = "http://localhost/repository/supermarket";
    let remote = address.to_string();

    let universe = get(&fixture, "/repository/supermarket/universe").await;
    assert_eq!(universe.status, StatusCode::OK, "{}", universe.text());
    assert!(!universe.text().contains(&remote), "{}", universe.text());
    let universe = universe.json();
    assert_eq!(
        universe["apt"]["8.0.0"],
        json!({
            "location_type": "opscode",
            "location_path": format!("{base}/api/v1"),
            "download_url": format!("{base}/api/v1/cookbooks/apt/versions/8.0.0/download"),
            "dependencies": {},
        })
    );
    assert_eq!(universe["apt"].as_object().unwrap().len(), 2);
    assert_eq!(universe["nginx"]["12.3.4"]["dependencies"]["apt"], ">= 7.0");
    get(&fixture, "/repository/supermarket/universe").await;
    assert_eq!(state.universes.load(Ordering::SeqCst), 1);

    let found = get(&fixture, "/repository/supermarket/api/v1/cookbooks/apt").await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.text());
    assert!(!found.text().contains(&remote));
    let found = found.json();
    assert_eq!(
        found["latest_version"],
        format!("{base}/api/v1/cookbooks/apt/versions/8.0.0")
    );
    assert_eq!(
        found["versions"],
        json!([
            format!("{base}/api/v1/cookbooks/apt/versions/8.0.0"),
            format!("{base}/api/v1/cookbooks/apt/versions/7.4.0"),
        ])
    );
    assert_eq!(found["metrics"]["followers"], 407);
    let version = get(
        &fixture,
        "/repository/supermarket/api/v1/cookbooks/apt/versions/latest",
    )
    .await;
    assert_eq!(version.status, StatusCode::OK, "{}", version.text());
    let version = version.json();
    assert_eq!(version["version"], "8.0.0");
    assert_eq!(version["license"], "Apache-2.0");
    assert_eq!(version["cookbook"], format!("{base}/api/v1/cookbooks/apt"));
    assert_eq!(
        version["file"],
        format!("{base}/api/v1/cookbooks/apt/versions/8.0.0/download")
    );
    let downloaded = follow(&fixture, version["file"].as_str().unwrap()).await;
    assert_eq!(downloaded.status, StatusCode::OK, "{}", downloaded.text());
    assert_eq!(downloaded.body, *state.apt);
    let again = follow(
        &fixture,
        universe["apt"]["8.0.0"]["download_url"].as_str().unwrap(),
    )
    .await;
    assert_eq!(again.body, *state.apt);
    assert_eq!(state.tarballs.load(Ordering::SeqCst), 1);
    let relative = get(
        &fixture,
        "/repository/supermarket/api/v1/cookbooks/nginx/versions/12_3_4/download",
    )
    .await;
    assert_eq!(relative.status, StatusCode::OK, "{}", relative.text());
    assert_eq!(relative.body, *state.nginx);
    let missing = get(
        &fixture,
        "/repository/supermarket/api/v1/cookbooks/apt/versions/1.0.0/download",
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let cached = get(
        &fixture,
        "/service/rest/v1/search?repository=supermarket&format=chef&name=apt",
    )
    .await
    .json();
    assert_eq!(cached["items"].as_array().unwrap().len(), 1, "{cached}");
    assert_eq!(cached["items"][0]["version"], "8.0.0");
    let licensed = get(
        &fixture,
        "/service/rest/v1/search?repository=supermarket&format=chef&chef.license=Apache-2.0",
    )
    .await
    .json();
    assert_eq!(licensed["items"].as_array().unwrap().len(), 2, "{licensed}");

    let search = get(
        &fixture,
        "/repository/supermarket/api/v1/search?q=ngi&items=5",
    )
    .await;
    assert_eq!(search.status, StatusCode::OK, "{}", search.text());
    let search = search.json();
    assert_eq!(search["total"], 1);
    assert_eq!(
        search["items"][0]["cookbook"],
        format!("{base}/api/v1/cookbooks/nginx")
    );
    assert!(
        state
            .searches
            .lock()
            .unwrap()
            .iter()
            .any(|asked| asked == "api/v1/search?q=ngi&start=0&items=5"),
        "{:?}",
        state.searches.lock().unwrap()
    );

    create(&fixture, "hosted", hosted("own", "ALLOW")).await;
    create(
        &fixture,
        "group",
        group("chef-all", &["own", "supermarket"]),
    )
    .await;
    let own_apt = cookbook("apt", "8.0.0", &[("own-base", ">= 1.0")]);
    publish(&fixture, "own", Some("Other"), &own_apt).await;
    publish(&fixture, "own", None, &cookbook("apt", "9.0.0", &[])).await;
    publish(&fixture, "own", None, &cookbook("own-base", "1.0.0", &[])).await;
    let all = "http://localhost/repository/chef-all";

    let merged = get(&fixture, "/repository/chef-all/universe").await;
    assert_eq!(merged.status, StatusCode::OK, "{}", merged.text());
    assert!(!merged.text().contains(&remote));
    let merged = merged.json();
    let apt_versions: Vec<&String> = merged["apt"].as_object().unwrap().keys().collect();
    assert_eq!(apt_versions, ["7.4.0", "8.0.0", "9.0.0"]);
    assert_eq!(merged["apt"]["8.0.0"]["dependencies"]["own-base"], ">= 1.0");
    assert_eq!(
        merged["apt"]["7.4.0"]["download_url"],
        format!("{all}/api/v1/cookbooks/apt/versions/7.4.0/download")
    );
    assert!(merged.get("nginx").is_some() && merged.get("own-base").is_some());
    assert_eq!(state.universes.load(Ordering::SeqCst), 1);

    let cookbook_doc = get(&fixture, "/repository/chef-all/api/v1/cookbooks/apt")
        .await
        .json();
    assert_eq!(
        cookbook_doc["latest_version"],
        format!("{all}/api/v1/cookbooks/apt/versions/9.0.0")
    );
    assert_eq!(cookbook_doc["versions"].as_array().unwrap().len(), 3);
    assert_eq!(cookbook_doc["maintainer"], "Acme Ops");
    let through = get(
        &fixture,
        "/repository/chef-all/api/v1/cookbooks/apt/versions/7_4_0",
    )
    .await
    .json();
    assert_eq!(
        through["file"],
        format!("{all}/api/v1/cookbooks/apt/versions/7.4.0/download")
    );
    let first_wins = get(
        &fixture,
        "/repository/chef-all/api/v1/cookbooks/apt/versions/8.0.0/download",
    )
    .await;
    assert_eq!(first_wins.body, own_apt);
    let from_proxy = get(
        &fixture,
        "/repository/chef-all/api/v1/cookbooks/nginx/versions/12.3.4/download",
    )
    .await;
    assert_eq!(from_proxy.body, *state.nginx);
    let searched = get(&fixture, "/repository/chef-all/api/v1/search?q=apt")
        .await
        .json();
    let names: Vec<&str> = searched["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["cookbook_name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["apt"], "{searched}");
    assert_eq!(
        searched["items"][0]["cookbook"],
        format!("{all}/api/v1/cookbooks/apt")
    );
    let listed = get(&fixture, "/repository/chef-all/api/v1/cookbooks")
        .await
        .json();
    assert_eq!(listed["total"], 3, "{listed}");

    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/own/api/v1/cookbooks/apt/versions/9.0.0",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::OK);
    let merged = get(&fixture, "/repository/chef-all/universe").await.json();
    assert!(merged["apt"].get("9.0.0").is_none(), "{merged}");
    let dropped = fixture
        .call(
            Method::DELETE,
            "/repository/supermarket/universe",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(dropped.status, StatusCode::NO_CONTENT);
    get(&fixture, "/repository/supermarket/universe").await;
    assert_eq!(state.universes.load(Ordering::SeqCst), 2);
}
