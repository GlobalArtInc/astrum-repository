mod support;

use astrum_db::entity::{audit_event, blob_store, repository, security_role};
use astrum_repo::{RepositoryType, Spec};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::http::{Method, StatusCode, header};
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const REPOSITORIES: &str = "/service/rest/v1/repositories";
const SETTINGS: &str = "/service/rest/v1/repositorySettings";
const PGP: &str = "-----BEGIN PGP PRIVATE KEY BLOCK-----\n\nlQPGBGrM3DIBCAC\n=XjSL\n-----END PGP PRIVATE KEY BLOCK-----\n";
const PEM: &str =
    "-----BEGIN PRIVATE KEY-----\nMIIEvAIBADANBgkqhkiG9w0BAQEFAASC\n-----END PRIVATE KEY-----\n";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    add_blob_store(&fixture, "default").await;
    fixture
}

async fn add_blob_store(fixture: &Fixture, name: &str) {
    let now = chrono::Utc::now().fixed_offset();
    blob_store::Entity::insert(blob_store::ActiveModel {
        name: ActiveValue::Set(name.to_owned()),
        kind: ActiveValue::Set("File".to_owned()),
        attributes: ActiveValue::Set(json!({"file": {"path": name}})),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
}

async fn add_role(fixture: &Fixture, id: &str, privileges: &[&str]) {
    let now = chrono::Utc::now().fixed_offset();
    security_role::Entity::insert(security_role::ActiveModel {
        id: ActiveValue::Set(id.to_owned()),
        source: ActiveValue::Set("default".to_owned()),
        name: ActiveValue::Set(id.to_owned()),
        description: ActiveValue::Set(String::new()),
        privileges: ActiveValue::Set(json!(privileges)),
        roles: ActiveValue::Set(json!([])),
        read_only: ActiveValue::Set(false),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
}

async fn user_with(fixture: &Fixture, user: &str, privileges: &[&str]) {
    let role = format!("{user}-role");
    add_role(fixture, &role, privileges).await;
    fixture.add_user(user, "secret-pw", &[&role]).await;
}

fn path(spec: Spec) -> String {
    format!("{REPOSITORIES}/{}/{}", spec.format.api_name, spec.kind)
}

fn spec(format: &str, kind: &str) -> Spec {
    Spec::from_api(format, kind).unwrap()
}

fn body(spec: Spec, name: &str, member: Option<&str>) -> Value {
    let mut body = json!({
        "name": name,
        "online": false,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
    });
    match spec.kind {
        RepositoryType::Hosted => {
            body["storage"]["writePolicy"] = json!("ALLOW_ONCE");
            body["cleanup"] = json!({"policyNames": []});
            body["component"] = json!({"proprietaryComponents": true});
        }
        RepositoryType::Proxy => {
            body["cleanup"] = json!({"policyNames": []});
            body["proxy"] = json!({"remoteUrl": "https://upstream.example.org/m1/", "contentMaxAge": 11, "metadataMaxAge": 12, "preserveEncodedCharacters": true});
            body["negativeCache"] = json!({"enabled": false, "timeToLive": 13});
            body["httpClient"] = json!({
                "blocked": true, "autoBlock": false,
                "connection": {"retries": 3, "userAgentSuffix": "m1", "timeout": 30, "enableCircularRedirects": true, "enableCookies": true, "useTrustStore": true},
            });
            body["replication"] = json!({"preemptivePullEnabled": true, "assetPathRegex": ".*m1"});
            body["firewall"] = json!({"mode": "AUDIT"});
        }
        RepositoryType::Group => {
            let member = member.expect("a group needs a member");
            body["group"] = json!({"memberNames": [member]});
            if matches!(spec.format.id, "docker" | "cargo" | "swift") {
                body["group"]["writableMember"] = json!(member);
            }
        }
    }
    let kind = spec.kind;
    match spec.format.id {
        "alpine" => body["alpineSigning"] = json!({"keypair": PEM, "passphrase": "m1-pass"}),
        "apt" if kind == RepositoryType::Hosted => {
            body["apt"] = json!({"distribution": "bookworm"});
            body["aptSigning"] = json!({"keypair": PGP, "passphrase": "m1-pass"});
        }
        "apt" => {
            body["apt"] =
                json!({"distribution": "bookworm", "flat": true, "enforceDistribution": true})
        }
        "cargo" => body["cargo"] = json!({"requireAuthentication": true}),
        "conan" if kind == RepositoryType::Proxy => {
            body["conanProxy"] = json!({"conanVersion": "V1"})
        }
        "docker" => {
            body["docker"] = json!({"v1Enabled": true, "forceBasicAuth": false, "subdomain": format!("{name}-sub"), "pathEnabled": true});
            if kind == RepositoryType::Proxy {
                body["dockerProxy"] = json!({"indexType": "CUSTOM", "indexUrl": "https://index.example.org/", "cacheForeignLayers": true, "foreignLayerUrlWhitelist": [".*m1"]});
            }
            if kind == RepositoryType::Hosted {
                body["storage"]["latestPolicy"] = json!(true);
            }
        }
        "maven2" if kind != RepositoryType::Group => {
            body["maven"] = json!({"versionPolicy": "MIXED", "layoutPolicy": "PERMISSIVE", "contentDisposition": "INLINE"});
        }
        "nix" => {
            body["nix"] =
                json!({"storeDir": "/opt/nix/store", "priority": 25, "wantMassQuery": false})
        }
        "npm" if kind == RepositoryType::Proxy => {
            body["npm"] = json!({"removeQuarantined": true, "removeNonCataloged": true})
        }
        "nuget" if kind == RepositoryType::Proxy => {
            body["nugetProxy"] = json!({"queryCacheItemMaxAge": 100, "nugetVersion": "V2"})
        }
        "raw" => {
            body["raw"] = json!({"contentDisposition": "INLINE", "forwardQueryParameters": true, "excludedQueryParameters": ["token"]})
        }
        "terraform" => {
            body["terraform"] = json!({"requireAuthentication": true, "registryHost": format!("{name}.example.org"), "upstreamHost": "registry.example.org"});
            if kind == RepositoryType::Hosted {
                body["terraformSigning"] = json!({"keypair": PGP, "passphrase": "m1-pass"});
            }
        }
        "yum" => {
            if kind == RepositoryType::Hosted {
                body["yum"] = json!({"repodataDepth": 2, "deployPolicy": "PERMISSIVE"});
            }
            body["yumSigning"] = json!({"keypair": PGP, "passphrase": "m1-pass"});
        }
        _ => {}
    }
    body
}

async fn post(fixture: &Fixture, spec: Spec, body: Value) -> Reply {
    fixture
        .call(Method::POST, &path(spec), Auth::Admin, Some(body))
        .await
}

async fn get(fixture: &Fixture, spec: Spec, name: &str) -> Reply {
    fixture
        .call(
            Method::GET,
            &format!("{}/{name}", path(spec)),
            Auth::Admin,
            None,
        )
        .await
}

async fn put(fixture: &Fixture, spec: Spec, name: &str, body: Value) -> Reply {
    fixture
        .call(
            Method::PUT,
            &format!("{}/{name}", path(spec)),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn created(fixture: &Fixture, spec: Spec, body: Value) {
    let reply = post(fixture, spec, body).await;
    assert_eq!(
        reply.status,
        StatusCode::CREATED,
        "{spec:?}: {}",
        reply.text()
    );
}

fn ids(reply: &Reply) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap_or_else(|| panic!("{}", reply.text()))
        .iter()
        .map(|error| {
            format!(
                "{}: {}",
                error["id"].as_str().unwrap(),
                error["message"].as_str().unwrap()
            )
        })
        .collect()
}

fn renamed(mut document: Value, name: &str) -> Value {
    document["name"] = json!(name);
    document["url"] = json!(format!("http://localhost/repository/{name}"));
    document
}

fn with_own_hosts(mut document: Value, name: &str) -> Value {
    if document["docker"]["subdomain"].is_string() {
        document["docker"]["subdomain"] = json!(format!("{name}-sub"));
    }
    if document["terraform"]["registryHost"].is_string() {
        document["terraform"]["registryHost"] = json!(format!("{name}.example.org"));
    }
    document
}

#[tokio::test]
async fn every_format_and_type_round_trips_through_get_post_and_put() {
    let fixture = ready().await;
    for spec in Spec::all() {
        let member = match spec.kind {
            RepositoryType::Group => {
                let member_spec = Spec::all()
                    .find(|other| {
                        other.format == spec.format && other.kind != RepositoryType::Group
                    })
                    .unwrap();
                Some(format!("{}-{}-a", member_spec.format.id, member_spec.kind))
            }
            _ => None,
        };
        let first = format!("{}-{}-a", spec.format.id, spec.kind);
        created(&fixture, spec, body(spec, &first, member.as_deref())).await;
        let reply = get(&fixture, spec, &first).await;
        assert_eq!(reply.status, StatusCode::OK);
        let document = reply.json();
        assert_eq!(document["name"], first);
        assert_eq!(document["format"], spec.format.id);
        assert_eq!(document["type"], spec.kind.as_str());
        assert_eq!(
            document["url"],
            format!("http://localhost/repository/{first}")
        );
        assert_eq!(document["online"], false);

        let second = format!("{}-{}-b", spec.format.id, spec.kind);
        let mut copy = with_own_hosts(renamed(document.clone(), &second), &second);
        for signing in [
            "aptSigning",
            "yumSigning",
            "terraformSigning",
            "alpineSigning",
        ] {
            if copy[signing].is_object() {
                copy[signing]["keypair"] =
                    json!(if spec.format.id == "alpine" { PEM } else { PGP });
            }
        }
        created(&fixture, spec, copy).await;
        let expected = with_own_hosts(renamed(document.clone(), &second), &second);
        assert_eq!(
            get(&fixture, spec, &second).await.json(),
            expected,
            "{spec:?}"
        );

        let reply = put(&fixture, spec, &first, document.clone()).await;
        assert_eq!(
            reply.status,
            StatusCode::NO_CONTENT,
            "{spec:?}: {}",
            reply.text()
        );
        assert_eq!(
            get(&fixture, spec, &first).await.json(),
            document,
            "{spec:?}"
        );
    }
    let listed = fixture
        .call(Method::GET, REPOSITORIES, Auth::Admin, None)
        .await
        .json();
    assert_eq!(listed.as_array().unwrap().len(), 69 * 2);
    let settings = fixture.call(Method::GET, SETTINGS, Auth::Admin, None).await;
    assert_eq!(settings.json().as_array().unwrap().len(), 69 * 2);
}

#[tokio::test]
async fn documents_match_the_reference_shapes() {
    let fixture = ready().await;
    let maven = spec("maven", "hosted");
    created(
        &fixture,
        maven,
        json!({
            "name": "maven-releases", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "allow_once"},
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"},
        }),
    )
    .await;
    assert_eq!(
        get(&fixture, maven, "maven-releases").await.json(),
        json!({
            "name": "maven-releases",
            "url": "http://localhost/repository/maven-releases",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW_ONCE"},
            "cleanup": null,
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"},
            "component": {"proprietaryComponents": false},
            "format": "maven2",
            "type": "hosted"
        })
    );
    let proxy = spec("maven", "proxy");
    created(
        &fixture,
        proxy,
        json!({
            "name": "maven-central", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "proxy": {"remoteUrl": "https://repo1.maven.org/maven2/", "contentMaxAge": -1, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {},
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "PERMISSIVE"},
        }),
    )
    .await;
    assert_eq!(
        get(&fixture, proxy, "maven-central").await.json(),
        json!({
            "name": "maven-central",
            "url": "http://localhost/repository/maven-central",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "cleanup": null,
            "proxy": {"remoteUrl": "https://repo1.maven.org/maven2/", "contentMaxAge": -1, "metadataMaxAge": 1440, "preserveEncodedCharacters": false},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {
                "blocked": false, "autoBlock": true,
                "connection": {"retries": null, "userAgentSuffix": null, "timeout": null, "enableCircularRedirects": false, "enableCookies": false, "useTrustStore": false},
                "authentication": null
            },
            "routingRuleName": null,
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "PERMISSIVE", "contentDisposition": "ATTACHMENT"},
            "replication": {"preemptivePullEnabled": false, "assetPathRegex": null},
            "format": "maven2",
            "type": "proxy",
            "firewall": null
        })
    );
    let group = spec("maven", "group");
    created(
        &fixture,
        group,
        json!({
            "name": "maven-public", "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "group": {"memberNames": ["maven-releases", "maven-central"]},
        }),
    )
    .await;
    assert_eq!(
        get(&fixture, group, "maven-public").await.json(),
        json!({
            "name": "maven-public",
            "format": "maven2",
            "url": "http://localhost/repository/maven-public",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
            "group": {"memberNames": ["maven-releases", "maven-central"]},
            "type": "group"
        })
    );
    let listed = fixture
        .call(Method::GET, REPOSITORIES, Auth::Admin, None)
        .await
        .json();
    assert_eq!(
        listed,
        json!([
            {"name": "maven-central", "format": "maven2", "type": "proxy", "url": "http://localhost/repository/maven-central", "online": true, "size": 0, "attributes": {"proxy": {"remoteUrl": "https://repo1.maven.org/maven2/"}}},
            {"name": "maven-public", "format": "maven2", "type": "group", "url": "http://localhost/repository/maven-public", "online": true, "size": 0, "attributes": {}},
            {"name": "maven-releases", "format": "maven2", "type": "hosted", "url": "http://localhost/repository/maven-releases", "online": true, "size": 0, "attributes": {}},
        ])
    );
    let single = fixture
        .call(
            Method::GET,
            &format!("{REPOSITORIES}/MAVEN-CENTRAL"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(single.status, StatusCode::OK);
    assert_eq!(single.json()["name"], "maven-central");
    assert!(single.json().get("size").is_none());
}

#[tokio::test]
async fn urls_follow_the_forwarded_headers() {
    let fixture = ready().await;
    created(
        &fixture,
        spec("raw", "hosted"),
        body(spec("raw", "hosted"), "files", None),
    )
    .await;
    let reply = fixture
        .send(
            Method::GET,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            None,
            &[
                ("Host", "10.0.0.5:8081"),
                ("X-Forwarded-Proto", "https"),
                ("X-Forwarded-Host", "astrum.example.org"),
            ],
        )
        .await;
    assert_eq!(
        reply.json()["url"],
        "https://astrum.example.org/repository/files"
    );
    let reply = fixture
        .send(
            Method::GET,
            &format!("{}/files", path(spec("raw", "hosted"))),
            Auth::Admin,
            None,
            &[("Host", "repo.example.org:8081")],
        )
        .await;
    assert_eq!(
        reply.json()["url"],
        "http://repo.example.org:8081/repository/files"
    );
}

#[tokio::test]
async fn names_are_checked_and_unique_ignoring_case() {
    let fixture = ready().await;
    let raw = spec("raw", "hosted");
    for bad in ["bad name", "_x", ".x", "a/b"] {
        let reply = post(&fixture, raw, body(raw, bad, None)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
        assert!(ids(&reply)[0].starts_with("PARAMETER name: "), "{bad}");
    }
    let reply = post(&fixture, raw, body(raw, &"x".repeat(201), None)).await;
    assert_eq!(
        ids(&reply),
        ["PARAMETER name: must be at most 200 characters long"]
    );
    created(&fixture, raw, body(raw, "Files", None)).await;
    let reply = post(
        &fixture,
        spec("npm", "hosted"),
        body(spec("npm", "hosted"), "FILES", None),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        ids(&reply),
        ["PARAMETER name: is already used by another repository (names ignore case)"]
    );
    assert_eq!(get(&fixture, raw, "files").await.json()["name"], "Files");
}

#[tokio::test]
async fn requests_are_validated_like_the_reference() {
    let fixture = ready().await;
    let reply = post(&fixture, spec("maven", "hosted"), json!({})).await;
    assert_eq!(
        ids(&reply),
        [
            "PARAMETER name: must not be empty",
            "PARAMETER online: must not be null",
            "PARAMETER storage: must not be null",
            "PARAMETER maven: must not be null"
        ]
    );
    let raw = spec("raw", "hosted");
    let mut missing_store = body(raw, "x", None);
    missing_store["storage"]["blobStoreName"] = json!("nope");
    let reply = post(&fixture, raw, missing_store).await;
    assert_eq!(
        ids(&reply),
        ["PARAMETER blobStoreName: the blob store \"nope\" does not exist"]
    );
    let mut wrong_case = body(raw, "x", None);
    wrong_case["storage"]["blobStoreName"] = json!("DEFAULT");
    assert_eq!(
        post(&fixture, raw, wrong_case).await.status,
        StatusCode::BAD_REQUEST
    );
    let mut no_policy = body(raw, "x", None);
    no_policy["storage"]
        .as_object_mut()
        .unwrap()
        .remove("writePolicy");
    assert_eq!(
        ids(&post(&fixture, raw, no_policy).await),
        ["PARAMETER writePolicy: must not be null"]
    );
    let mut bad_policy = body(raw, "x", None);
    bad_policy["storage"]["writePolicy"] = json!("WHATEVER");
    let reply = post(&fixture, raw, bad_policy).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("ALLOW_ONCE"), "{}", reply.text());
    let mut cleanup = body(raw, "x", None);
    cleanup["cleanup"] = json!({"policyNames": ["weekly"]});
    assert_eq!(
        ids(&post(&fixture, raw, cleanup).await),
        ["PARAMETER policyNames: the cleanup policy \"weekly\" does not exist"]
    );

    let proxy = spec("raw", "proxy");
    let mut ftp = body(proxy, "x", None);
    ftp["proxy"]["remoteUrl"] = json!("ftp://upstream.example.org/");
    assert_eq!(
        ids(&post(&fixture, proxy, ftp).await),
        ["PARAMETER remoteUrl: must be an http or https URL with a host"]
    );
    let mut rule = body(proxy, "x", None);
    rule["routingRuleName"] = json!("internal-only");
    assert_eq!(
        ids(&post(&fixture, proxy, rule).await),
        ["PARAMETER routingRuleName: the routing rule \"internal-only\" does not exist"]
    );
    let mut ranges = body(proxy, "x", None);
    ranges["negativeCache"]["timeToLive"] = json!(-1);
    ranges["httpClient"]["connection"]["retries"] = json!(11);
    assert_eq!(
        ids(&post(&fixture, proxy, ranges).await),
        [
            "PARAMETER timeToLive: must be greater than or equal to 0",
            "PARAMETER retries: must be less than or equal to 10"
        ]
    );
    let mut no_password = body(proxy, "x", None);
    no_password["httpClient"]["authentication"] = json!({"type": "username", "username": "u"});
    assert_eq!(
        ids(&post(&fixture, proxy, no_password).await),
        ["PARAMETER password: must not be null"]
    );

    let reply = fixture
        .send(
            Method::POST,
            &path(raw),
            Auth::Admin,
            None,
            &[("Content-Type", "application/json")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = fixture
        .send(Method::POST, &path(raw), Auth::Admin, None, &[])
        .await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    for unknown in ["apt/group", "maven2/hosted", "nope/hosted", "raw/virtual"] {
        let reply = fixture
            .call(
                Method::POST,
                &format!("{REPOSITORIES}/{unknown}"),
                Auth::Admin,
                Some(body(raw, "x", None)),
            )
            .await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{unknown}");
    }
    let nothing: Vec<repository::Model> = repository::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert!(nothing.is_empty());
}

#[tokio::test]
async fn updates_cannot_rename_or_move_a_repository() {
    let fixture = ready().await;
    add_blob_store(&fixture, "other").await;
    let raw = spec("raw", "hosted");
    created(&fixture, raw, body(raw, "files", None)).await;
    let document = get(&fixture, raw, "files").await.json();

    let reply = put(&fixture, raw, "files", renamed(document.clone(), "papers")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()[0]["message"], "a repository cannot be renamed");

    let mut moved = document.clone();
    moved["storage"]["blobStoreName"] = json!("other");
    let reply = put(&fixture, raw, "files", moved).await;
    assert_eq!(
        reply.json()[0]["message"],
        "the blob store of a repository cannot be changed"
    );

    let reply = put(&fixture, spec("raw", "proxy"), "files", document.clone()).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reply.json()[0]["message"],
        "the repository \"files\" is a raw hosted repository; its format and type cannot change"
    );
    let reply = put(&fixture, spec("maven", "hosted"), "files", document.clone()).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    let reply = put(&fixture, raw, "nope", renamed(document.clone(), "nope")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let mut changed = document.clone();
    changed["online"] = json!(true);
    changed["raw"]["contentDisposition"] = json!("ATTACHMENT");
    let reply = put(&fixture, raw, "FILES", changed.clone()).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert_eq!(get(&fixture, raw, "files").await.json(), changed);
    assert_eq!(
        get(&fixture, spec("raw", "proxy"), "files").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn groups_hold_existing_members_of_their_format_without_cycles() {
    let fixture = ready().await;
    let hosted = spec("raw", "hosted");
    let group = spec("raw", "group");
    created(&fixture, hosted, body(hosted, "files", None)).await;
    created(
        &fixture,
        spec("npm", "hosted"),
        body(spec("npm", "hosted"), "npm-files", None),
    )
    .await;

    let reply = post(&fixture, group, body(group, "all", Some("nope"))).await;
    assert_eq!(
        ids(&reply),
        ["PARAMETER memberNames: the repository \"nope\" does not exist"]
    );
    let reply = post(&fixture, group, body(group, "all", Some("npm-files"))).await;
    assert_eq!(
        ids(&reply),
        ["PARAMETER memberNames: the repository \"npm-files\" is a npm repository, not raw"]
    );
    let reply = post(&fixture, group, body(group, "all", Some("all"))).await;
    assert_eq!(
        ids(&reply),
        ["PARAMETER memberNames: the group \"all\" cannot be its own member"]
    );
    let mut empty = body(group, "all", Some("files"));
    empty["group"]["memberNames"] = json!([]);
    assert_eq!(
        ids(&post(&fixture, group, empty).await),
        ["PARAMETER memberNames: must not be empty"]
    );

    created(&fixture, group, body(group, "inner", Some("FILES"))).await;
    created(&fixture, group, body(group, "outer", Some("inner"))).await;
    let mut cycle = get(&fixture, group, "inner").await.json();
    cycle["group"]["memberNames"] = json!(["files", "outer"]);
    let reply = put(&fixture, group, "inner", cycle).await;
    assert_eq!(
        ids(&reply),
        [
            "PARAMETER memberNames: the group \"outer\" contains \"inner\", so it cannot be its member"
        ]
    );

    let docker_hosted = spec("docker", "hosted");
    let docker_group = spec("docker", "group");
    created(&fixture, docker_hosted, body(docker_hosted, "images", None)).await;
    created(
        &fixture,
        docker_hosted,
        body(docker_hosted, "images-2", None),
    )
    .await;
    let mut docker_proxy = body(spec("docker", "proxy"), "hub", None);
    docker_proxy["docker"]["subdomain"] = json!("hub-sub");
    created(&fixture, spec("docker", "proxy"), docker_proxy).await;
    let mut outside = body(docker_group, "docker-all", Some("images"));
    outside["group"]["writableMember"] = json!("images-2");
    assert_eq!(
        ids(&post(&fixture, docker_group, outside).await),
        ["PARAMETER writableMember: must be one of the members of the group"]
    );
    let mut proxy_writes = body(docker_group, "docker-all", Some("hub"));
    proxy_writes["group"]["writableMember"] = json!("hub");
    assert_eq!(
        ids(&post(&fixture, docker_group, proxy_writes).await),
        ["PARAMETER writableMember: must be a hosted repository"]
    );
    let mut fine = body(docker_group, "docker-all", Some("images"));
    fine["group"]["memberNames"] = json!(["images", "hub", "images-2"]);
    created(&fixture, docker_group, fine).await;

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{REPOSITORIES}/IMAGES"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let remaining = get(&fixture, docker_group, "docker-all").await.json();
    assert_eq!(
        remaining["group"],
        json!({"memberNames": ["hub", "images-2"]})
    );

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&fixture, group, "inner").await.json()["group"]["memberNames"],
        json!([])
    );
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn docker_ports_belong_to_one_repository() {
    let fixture = ready().await;
    let hosted = spec("docker", "hosted");
    let mut first = body(hosted, "images", None);
    first["docker"]["httpPort"] = json!(18080);
    created(&fixture, hosted, first).await;
    let mut second = body(hosted, "images-2", None);
    second["docker"]["httpsPort"] = json!(18080);
    second["docker"]["subdomain"] = json!("other");
    assert_eq!(
        ids(&post(&fixture, hosted, second).await),
        ["PARAMETER httpsPort: the port 18080 is used by the repository \"images\""]
    );
    let document = get(&fixture, hosted, "images").await.json();
    assert_eq!(document["docker"]["httpPort"], 18080);
    assert_eq!(
        put(&fixture, hosted, "images", document).await.status,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn the_list_reports_the_bytes_each_repository_holds() {
    let fixture = ready().await;
    let hosted = spec("raw", "hosted");
    created(&fixture, hosted, body(hosted, "files", None)).await;
    created(&fixture, hosted, body(hosted, "empty", None)).await;
    let group = spec("raw", "group");
    created(&fixture, group, body(group, "all", Some("files"))).await;
    fixture
        .state
        .db()
        .conn()
        .execute_unprepared(
            "INSERT INTO asset (repository, format, path, blob_store, blob_id, size, content_type, sha1, sha256, sha512, md5) VALUES \
             ('files', 'raw', '/a.txt', 'default', '00000000-0000-4000-8000-0000000000a1', 3000, 'text/plain', '', '', '', ''), \
             ('files', 'raw', '/b.txt', 'default', '00000000-0000-4000-8000-0000000000a2', 4, 'text/plain', '', '', '', '')",
        )
        .await
        .unwrap();
    let listed = fixture
        .call(Method::GET, REPOSITORIES, Auth::Admin, None)
        .await
        .json();
    let size = |name: &str| {
        listed
            .as_array()
            .unwrap()
            .iter()
            .find(|repository| repository["name"] == name)
            .unwrap_or_else(|| panic!("{name} is listed"))["size"]
            .clone()
    };
    assert_eq!(size("files"), 3004);
    assert_eq!(size("empty"), 0);
    assert_eq!(size("all"), 0);
}

async fn protect(fixture: &Fixture, enabled: bool, allowed_ips: &[&str]) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled,
            allowed_ips: allowed_ips.iter().map(|ip| (*ip).to_owned()).collect(),
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

fn remote(name: &str, url: &str) -> Value {
    let raw = spec("raw", "proxy");
    let mut document = body(raw, name, None);
    document["proxy"]["remoteUrl"] = json!(url);
    document
}

#[tokio::test]
async fn proxies_cannot_point_at_internal_addresses_unless_allowed() {
    let fixture = ready().await;
    let raw = spec("raw", "proxy");
    let refused = post(&fixture, raw, remote("inside", "http://10.1.2.3/")).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        refused.json(),
        json!([{
            "id": "PARAMETER remoteUrl",
            "message": "Proxy URL blocked: Host resolves to private/local IP address(es): 10.1.2.3 (private network address). To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection."
        }])
    );
    for (url, kind) in [
        ("http://127.0.0.1:8081/", "127.0.0.1 (loopback address)"),
        (
            "http://[fd00::1]/",
            "fd00:0:0:0:0:0:0:1 (private network address)",
        ),
        (
            "http://169.254.169.254/",
            "169.254.169.254 (restricted address)",
        ),
    ] {
        let reply = post(&fixture, raw, remote("inside", url)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{url}");
        assert!(reply.text().contains(kind), "{}", reply.text());
    }
    created(
        &fixture,
        raw,
        remote("unresolved", "http://upstream.invalid/"),
    )
    .await;

    protect(&fixture, true, &["10.0.0.0/8"]).await;
    created(&fixture, raw, remote("inside", "http://10.1.2.3/")).await;
    protect(&fixture, true, &[]).await;
    let mut document = get(&fixture, raw, "inside").await.json();
    document["online"] = json!(true);
    assert_eq!(
        put(&fixture, raw, "inside", document.clone()).await.status,
        StatusCode::NO_CONTENT,
        "a repository whose remote did not change can still be edited"
    );
    document["proxy"]["remoteUrl"] = json!("http://10.1.2.4/");
    let reply = put(&fixture, raw, "inside", document).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        ids(&reply),
        [
            "PARAMETER remoteUrl: Proxy URL blocked: Host resolves to private/local IP address(es): 10.1.2.4 (private network address). To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection."
        ]
    );
    assert_eq!(
        get(&fixture, raw, "inside").await.json()["proxy"]["remoteUrl"],
        "http://10.1.2.3/"
    );

    protect(&fixture, false, &[]).await;
    created(&fixture, raw, remote("open", "http://192.168.1.1/")).await;
    let reply = post(&fixture, raw, remote("metadata", "http://169.254.169.254/")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn secrets_are_never_returned_and_survive_updates_without_them() {
    let fixture = ready().await;
    let proxy = spec("maven", "proxy");
    let mut with_password = body(proxy, "central", None);
    with_password["httpClient"]["authentication"] = json!({"type": "username", "username": "reader", "password": "proxy-password", "preemptive": true});
    created(&fixture, proxy, with_password).await;
    let raw_proxy = spec("raw", "proxy");
    let mut with_token = body(raw_proxy, "tokens", None);
    with_token["httpClient"]["authentication"] =
        json!({"type": "bearerToken", "bearerToken": "proxy-token"});
    created(&fixture, raw_proxy, with_token).await;
    let yum = spec("yum", "hosted");
    created(&fixture, yum, body(yum, "rpms", None)).await;

    let document = get(&fixture, proxy, "central").await.json();
    assert_eq!(
        document["httpClient"]["authentication"],
        json!({"type": "username", "username": "reader", "ntlmHost": null, "ntlmDomain": null, "preemptive": true})
    );
    assert_eq!(
        get(&fixture, raw_proxy, "tokens").await.json()["httpClient"]["authentication"],
        json!({"type": "bearerToken", "username": null, "ntlmHost": null, "ntlmDomain": null})
    );
    assert_eq!(
        get(&fixture, yum, "rpms").await.json()["yumSigning"],
        json!({})
    );
    for path in [
        REPOSITORIES.to_owned(),
        SETTINGS.to_owned(),
        format!("{}/central", path(proxy)),
        format!("{}/tokens", path(raw_proxy)),
        format!("{}/rpms", path(yum)),
    ] {
        let text = fixture
            .call(Method::GET, &path, Auth::Admin, None)
            .await
            .text();
        for secret in [
            "proxy-password",
            "proxy-token",
            "m1-pass",
            "PGP",
            "astrum:v1:",
        ] {
            assert!(!text.contains(secret), "{secret} in {path}: {text}");
        }
    }

    let reply = put(&fixture, proxy, "central", document.clone()).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = put(
        &fixture,
        yum,
        "rpms",
        get(&fixture, yum, "rpms").await.json(),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());

    let rows: Vec<repository::Model> = repository::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    let stored = |name: &str| {
        rows.iter()
            .find(|row| row.name == name)
            .unwrap()
            .attributes
            .clone()
    };
    let secrets = fixture.state.secrets();
    let central = stored("central");
    let sealed = central["httpClient"]["authentication"]["password"]
        .as_str()
        .unwrap();
    assert!(!central.to_string().contains("proxy-password"));
    assert_eq!(secrets.open(sealed).unwrap(), "proxy-password");
    let token = stored("tokens")["httpClient"]["authentication"]["bearerToken"].clone();
    assert_eq!(
        secrets.open(token.as_str().unwrap()).unwrap(),
        "proxy-token"
    );
    let rpms = stored("rpms");
    assert_eq!(
        secrets
            .open(rpms["yumSigning"]["keypair"].as_str().unwrap())
            .unwrap(),
        PGP
    );
    assert_eq!(
        secrets
            .open(rpms["yumSigning"]["passphrase"].as_str().unwrap())
            .unwrap(),
        "m1-pass"
    );

    let mut changed = document.clone();
    changed["httpClient"]["authentication"]["password"] = json!("new-password");
    assert_eq!(
        put(&fixture, proxy, "central", changed).await.status,
        StatusCode::NO_CONTENT
    );
    let mut removed = document;
    removed["httpClient"]["authentication"] = Value::Null;
    assert_eq!(
        put(&fixture, proxy, "central", removed).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&fixture, proxy, "central").await.json()["httpClient"]["authentication"],
        Value::Null
    );

    let audit: Vec<audit_event::Model> = audit_event::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    assert!(!audit.is_empty());
    for event in audit {
        assert!(!event.attributes.to_string().contains("password"));
    }
}

#[tokio::test]
async fn changes_are_audited_and_create_repository_privileges() {
    let fixture = ready().await;
    let raw = spec("raw", "hosted");
    created(&fixture, raw, body(raw, "files", None)).await;
    let model = fixture.state.security().model();
    assert!(
        model
            .privileges
            .contains_key("as-repository-view-raw-files-read")
    );
    assert!(
        model
            .privileges
            .contains_key("as-repository-admin-raw-files-edit")
    );
    let document = get(&fixture, raw, "files").await.json();
    assert_eq!(
        put(&fixture, raw, "files", document).await.status,
        StatusCode::NO_CONTENT
    );
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{REPOSITORIES}/files"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(
        !fixture
            .state
            .security()
            .model()
            .privileges
            .contains_key("as-repository-view-raw-files-read")
    );
    let events: Vec<(String, String, String, String, Value)> = audit_event::Entity::find()
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
        .into_iter()
        .filter(|event| event.domain == "repository")
        .map(|event| {
            (
                event.kind,
                event.context,
                event.initiator,
                event.domain,
                event.attributes,
            )
        })
        .collect();
    let attributes = json!({"format": "raw", "type": "hosted"});
    assert_eq!(
        events,
        [
            (
                "created".to_owned(),
                "files".to_owned(),
                "admin".to_owned(),
                "repository".to_owned(),
                attributes.clone()
            ),
            (
                "updated".to_owned(),
                "files".to_owned(),
                "admin".to_owned(),
                "repository".to_owned(),
                attributes.clone()
            ),
            (
                "deleted".to_owned(),
                "files".to_owned(),
                "admin".to_owned(),
                "repository".to_owned(),
                attributes
            ),
        ]
    );
}

#[tokio::test]
async fn callers_need_the_repository_permissions() {
    let fixture = ready().await;
    let raw = spec("raw", "hosted");
    created(&fixture, raw, body(raw, "mine", None)).await;
    created(&fixture, raw, body(raw, "theirs", None)).await;
    created(&fixture, raw, body(raw, "browsed", None)).await;
    user_with(
        &fixture,
        "keeper",
        &[
            "as-repository-admin-raw-mine-read",
            "as-repository-admin-raw-mine-edit",
            "as-repository-view-raw-browsed-browse",
        ],
    )
    .await;
    user_with(&fixture, "reader", &["as-repository-admin-raw-theirs-read"]).await;
    user_with(&fixture, "creator", &["as-repository-admin-*-*-add"]).await;
    let keeper = Auth::Basic("keeper", "secret-pw");
    let reader = Auth::Basic("reader", "secret-pw");
    let creator = Auth::Basic("creator", "secret-pw");

    for (method, url, body) in [
        (Method::GET, REPOSITORIES.to_owned(), None),
        (Method::GET, SETTINGS.to_owned(), None),
        (Method::GET, format!("{REPOSITORIES}/mine"), None),
        (Method::GET, format!("{}/mine", path(raw)), None),
        (Method::POST, path(raw), Some(body(raw, "new", None))),
        (Method::DELETE, format!("{REPOSITORIES}/mine"), None),
    ] {
        let reply = fixture.call(method.clone(), &url, Auth::None, body).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {url}");
        assert!(reply.headers.contains_key(header::WWW_AUTHENTICATE));
    }

    let names = |reply: Reply| -> Vec<String> {
        reply
            .json()
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["name"].as_str().unwrap().to_owned())
            .collect()
    };
    let list = |path: &'static str, user: &'static str| {
        fixture.call(Method::GET, path, Auth::Basic(user, "secret-pw"), None)
    };
    assert_eq!(
        names(list(REPOSITORIES, "keeper").await),
        ["browsed", "mine"]
    );
    assert_eq!(names(list(SETTINGS, "keeper").await), ["mine"]);
    assert_eq!(
        names(list(REPOSITORIES, "creator").await),
        Vec::<String>::new()
    );
    let status = |reply: Reply| reply.status;
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{REPOSITORIES}/browsed"),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::OK
    );
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{REPOSITORIES}/theirs"),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{REPOSITORIES}/nope"),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{}/mine", path(raw)),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::OK
    );
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{}/browsed", path(raw)),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(
            fixture
                .call(
                    Method::GET,
                    &format!("{}/theirs", path(raw)),
                    Auth::Basic("keeper", "secret-pw"),
                    None
                )
                .await
        ),
        StatusCode::NOT_FOUND
    );

    let mine = get(&fixture, raw, "mine").await.json();
    let theirs = get(&fixture, raw, "theirs").await.json();
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{}/mine", path(raw)),
            keeper,
            Some(mine),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{}/theirs", path(raw)),
            Auth::Basic("keeper", "secret-pw"),
            Some(theirs.clone()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{}/theirs", path(raw)),
            reader,
            Some(theirs),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::POST,
            &path(raw),
            Auth::Basic("keeper", "secret-pw"),
            Some(body(raw, "new", None)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    let reply = fixture
        .call(
            Method::POST,
            &path(raw),
            Auth::Basic("keeper", "secret-pw"),
            Some(json!({})),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::BAD_REQUEST,
        "validation comes before permissions"
    );
    let reply = fixture
        .call(
            Method::POST,
            &path(raw),
            creator,
            Some(body(raw, "new", None)),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED);

    for (name, expected) in [
        ("mine", StatusCode::FORBIDDEN),
        ("theirs", StatusCode::NOT_FOUND),
    ] {
        let reply = fixture
            .call(
                Method::DELETE,
                &format!("{REPOSITORIES}/{name}"),
                Auth::Basic("keeper", "secret-pw"),
                None,
            )
            .await;
        assert_eq!(reply.status, expected, "{name}");
    }
    let reply = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/mine/rebuild-index"),
            Auth::Basic("keeper", "secret-pw"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let reply = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/theirs/rebuild-index"),
            Auth::Basic("reader", "secret-pw"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(get(&fixture, raw, "mine").await.status, StatusCode::OK);
}

#[tokio::test]
async fn maintenance_requests_are_accepted_for_the_right_types() {
    let fixture = ready().await;
    let hosted = spec("raw", "hosted");
    created(&fixture, hosted, body(hosted, "files", None)).await;
    created(
        &fixture,
        spec("raw", "proxy"),
        body(spec("raw", "proxy"), "remote", None),
    )
    .await;
    created(
        &fixture,
        spec("raw", "group"),
        body(spec("raw", "group"), "all", Some("files")),
    )
    .await;
    for (name, action, expected) in [
        ("files", "invalidate-cache", StatusCode::BAD_REQUEST),
        ("files", "rebuild-index", StatusCode::NO_CONTENT),
        ("remote", "invalidate-cache", StatusCode::NO_CONTENT),
        ("remote", "rebuild-index", StatusCode::NO_CONTENT),
        ("all", "invalidate-cache", StatusCode::NO_CONTENT),
        ("all", "rebuild-index", StatusCode::BAD_REQUEST),
        ("nope", "invalidate-cache", StatusCode::NOT_FOUND),
        ("nope", "rebuild-index", StatusCode::NOT_FOUND),
    ] {
        let reply = fixture
            .call(
                Method::POST,
                &format!("{REPOSITORIES}/{name}/{action}"),
                Auth::Admin,
                None,
            )
            .await;
        assert_eq!(reply.status, expected, "{name} {action}");
    }
    let reply = fixture
        .call(
            Method::POST,
            &format!("{REPOSITORIES}/files/invalidate-cache"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json()[0]["message"],
        "the cache can be invalidated only for proxy and group repositories"
    );
}
