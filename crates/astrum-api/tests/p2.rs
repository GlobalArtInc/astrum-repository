mod support;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use astrum_content::zip;
use astrum_format_p2::testing::{unxz, xz};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const BROWSE: &str = "/service/rest/internal/ui/browse";

#[derive(Clone, Default)]
struct Upstream {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Arc<Mutex<Vec<(String, String, bool)>>>,
}

impl Upstream {
    fn put(&self, path: &str, body: Vec<u8>) {
        self.files.lock().unwrap().insert(path.to_owned(), body);
    }

    fn hits(&self, path: &str) -> usize {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, hit, _)| hit == path)
            .count()
    }

    fn authorized(&self) -> Vec<String> {
        self.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, _, authorized)| *authorized)
            .map(|(_, path, _)| path.clone())
            .collect()
    }

    async fn start(&self) -> SocketAddr {
        let app = Router::new().fallback(serve_file).with_state(self.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        address
    }
}

async fn serve_file(State(upstream): State<Upstream>, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    upstream.hits.lock().unwrap().push((
        request.method().to_string(),
        path.clone(),
        request.headers().contains_key(header::AUTHORIZATION),
    ));
    let Some(body) = upstream.files.lock().unwrap().get(&path).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let etag = format!("\"{}\"", etag_of(&body));
    let unchanged = request
        .headers()
        .get(header::IF_NONE_MATCH)
        .is_some_and(|value| value.as_bytes() == etag.as_bytes());
    let headers = [
        (
            header::LAST_MODIFIED,
            "Wed, 06 Mar 2024 20:02:43 GMT".to_owned(),
        ),
        (header::ETAG, etag),
    ];
    match unchanged {
        true => (StatusCode::NOT_MODIFIED, headers).into_response(),
        false => (headers, body).into_response(),
    }
}

fn etag_of(body: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hasher);
    hasher.finish()
}

fn jar(entry: &str, text: &str) -> Vec<u8> {
    zip::write(&[(entry, text.as_bytes())])
}

fn unjar(bytes: &[u8]) -> String {
    let source = zip::Memory(bytes.to_vec());
    let entries = futures_ready(zip::entries(&source));
    let data = futures_ready(zip::read(&source, &entries[0], 1 << 30));
    String::from_utf8(data).unwrap()
}

fn futures_ready<T, E: std::fmt::Debug>(
    future: impl std::future::Future<Output = Result<T, E>>,
) -> T {
    use futures_util::FutureExt;
    future
        .now_or_never()
        .expect("an archive in memory is read at once")
        .unwrap()
}

fn content(name: &str, units: &[(&str, &str)]) -> String {
    let mut out = format!(
        "<?xml version='1.0' encoding='UTF-8'?>\n<?metadataRepository version='1.1.0'?>\n<repository name='{name}' type='org.eclipse.equinox.internal.p2.metadata.repository.LocalMetadataRepository' version='1'>\n  <properties size='1'>\n    <property name='p2.timestamp' value='1709755143787'/>\n  </properties>\n  <units size='{}'>\n",
        units.len()
    );
    for (id, version) in units {
        out.push_str(&format!(
            "    <unit id='{id}' version='{version}'>\n      <artifacts size='1'>\n        <artifact classifier='osgi.bundle' id='{id}' version='{version}'/>\n      </artifacts>\n    </unit>\n"
        ));
    }
    out.push_str("  </units>\n</repository>\n");
    out
}

fn artifacts(name: &str, items: &[(&str, &str, &str)], mirrored: bool) -> String {
    let mirrors = match mirrored {
        true => {
            "    <property name='p2.mirrorsURL' value='https://www.example.org/downloads/download.php?format=xml&amp;file=/x'/>\n    <property name='p2.statsURI' value='https://download.example.org/stats/x'/>\n"
        }
        false => "",
    };
    let mut out = format!(
        "<?xml version='1.0' encoding='UTF-8'?>\n<?artifactRepository version='1.1.0'?>\n<repository name='{name}' type='org.eclipse.equinox.p2.artifact.repository.simpleRepository' version='1'>\n  <properties size='{}'>\n    <property name='p2.timestamp' value='1709794724040'/>\n    <property name='p2.compressed' value='true'/>\n{mirrors}  </properties>\n  <mappings size='3'>\n    <rule filter='(&amp; (classifier=osgi.bundle))' output='${{repoUrl}}/plugins/${{id}}_${{version}}.jar'/>\n    <rule filter='(&amp; (classifier=binary))' output='${{repoUrl}}/binary/${{id}}_${{version}}'/>\n    <rule filter='(&amp; (classifier=org.eclipse.update.feature))' output='${{repoUrl}}/features/${{id}}_${{version}}.jar'/>\n  </mappings>\n  <artifacts size='{}'>\n",
        if mirrored { 4 } else { 2 },
        items.len()
    );
    for (classifier, id, version) in items {
        out.push_str(&format!(
            "    <artifact classifier='{classifier}' id='{id}' version='{version}'>\n      <properties size='2'>\n        <property name='artifact.size' value='512'/>\n        <property name='maven-groupId' value='org.example'/>\n      </properties>\n    </artifact>\n"
        ));
    }
    out.push_str("  </artifacts>\n</repository>\n");
    out
}

fn composite(kind: &str, children: &[String]) -> String {
    let (instruction, kind_type) = match kind {
        "content" => (
            "compositeMetadataRepository",
            "org.eclipse.equinox.internal.p2.metadata.repository.CompositeMetadataRepository",
        ),
        _ => (
            "compositeArtifactRepository",
            "org.eclipse.equinox.internal.p2.artifact.repository.CompositeArtifactRepository",
        ),
    };
    let mut out = format!(
        "<?xml version='1.0' encoding='UTF-8'?>\n<?{instruction} version='1.0.0'?>\n<repository name='Release' type='{kind_type}' version='1.0.0'>\n  <properties size='2'>\n    <property name='p2.timestamp' value='1709794749794'/>\n    <property name='p2.atomic.composite.loading' value='true'/>\n  </properties>\n  <children size='{}'>\n",
        children.len()
    );
    for child in children {
        out.push_str(&format!("    <child location='{child}'/>\n"));
    }
    out.push_str("  </children>\n</repository>\n");
    out
}

fn plugin(name: &str) -> Vec<u8> {
    zip::write(&[
        (
            "META-INF/MANIFEST.MF",
            b"Manifest-Version: 1.0\r\nBundle-ManifestVersion: 2\r\nBundle-Name: %pluginName\r\nBundle-SymbolicName: org.example\r\n\r\n",
        ),
        (
            "OSGI-INF/l10n/bundle.properties",
            format!("pluginName = {name}\n").as_bytes(),
        ),
        ("org/example/Core.class", b"\xca\xfe\xba\xbe"),
    ])
}

fn feature(label: &str) -> Vec<u8> {
    zip::write(&[
        (
            "feature.xml",
            b"<?xml version='1.0' encoding='UTF-8'?>\n<feature id='org.example.feature' label='%featureName' version='1.0.0'>\n</feature>\n",
        ),
        (
            "feature.properties",
            format!("featureName={label}\n").as_bytes(),
        ),
    ])
}

const SIMPLE_INDEX: &str = "version=1\nmetadata.repository.factory.order=content.xml.xz,content.xml,\\!\nartifact.repository.factory.order=artifacts.xml.xz,artifacts.xml,\\!\n";
const COMPOSITE_INDEX: &str = "version=1\nmetadata.repository.factory.order=compositeContent.xml,\\!\nartifact.repository.factory.order=compositeArtifacts.xml,\\!\n";

struct World {
    main: Upstream,
    other: Upstream,
    main_address: SocketAddr,
    other_address: SocketAddr,
}

async fn world() -> World {
    let main = Upstream::default();
    let other = Upstream::default();
    let main_address = main.start().await;
    let other_address = other.start().await;
    let children = vec![
        "simple".to_owned(),
        "../../technology/pkgs/".to_owned(),
        format!("http://{main_address}/releases/r1/abs"),
        format!("http://{other_address}/updates/x"),
    ];
    main.put("/releases/r1/p2.index", COMPOSITE_INDEX.into());
    main.put(
        "/releases/r1/compositeContent.jar",
        jar("compositeContent.xml", &composite("content", &children)),
    );
    main.put(
        "/releases/r1/compositeArtifacts.jar",
        jar("compositeArtifacts.xml", &composite("artifacts", &children)),
    );

    let core = [("osgi.bundle", "org.example.core", "1.0.0.v1")];
    let simple_artifacts = artifacts(
        "simple",
        &[
            core[0],
            ("org.eclipse.update.feature", "org.example.feature", "1.0.0"),
        ],
        true,
    );
    main.put("/releases/r1/simple/p2.index", SIMPLE_INDEX.into());
    main.put(
        "/releases/r1/simple/content.xml.xz",
        xz(content("simple", &[("org.example.core", "1.0.0.v1")]).as_bytes()),
    );
    main.put(
        "/releases/r1/simple/content.jar",
        jar(
            "content.xml",
            &content("simple", &[("org.example.core", "1.0.0.v1")]),
        ),
    );
    main.put(
        "/releases/r1/simple/artifacts.xml.xz",
        xz(simple_artifacts.as_bytes()),
    );
    main.put(
        "/releases/r1/simple/artifacts.jar",
        jar("artifacts.xml", &simple_artifacts),
    );
    main.put(
        "/releases/r1/simple/plugins/org.example.core_1.0.0.v1.jar",
        plugin("Example Core"),
    );
    main.put(
        "/releases/r1/simple/features/org.example.feature_1.0.0.jar",
        feature("Example Feature"),
    );

    main.put(
        "/technology/pkgs/content.jar",
        jar(
            "content.xml",
            &content("pkgs", &[("org.example.pkg", "2.0.0")]),
        ),
    );
    main.put(
        "/technology/pkgs/artifacts.jar",
        jar(
            "artifacts.xml",
            &artifacts(
                "pkgs",
                &[("osgi.bundle", "org.example.pkg", "2.0.0")],
                false,
            ),
        ),
    );
    main.put(
        "/technology/pkgs/plugins/org.example.pkg_2.0.0.jar",
        plugin("Example Package"),
    );
    main.put("/technology/secret/plugins/a_1.0.0.jar", plugin("Secret"));

    main.put(
        "/releases/r1/abs/artifacts.xml",
        artifacts(
            "abs",
            &[("binary", "org.example.launcher_root.linux", "1.0.0")],
            false,
        )
        .into_bytes(),
    );
    main.put(
        "/releases/r1/abs/content.xml",
        content("abs", &[]).into_bytes(),
    );
    main.put(
        "/releases/r1/abs/binary/org.example.launcher_root.linux_1.0.0",
        b"#!/bin/sh\n".to_vec(),
    );

    other.put(
        "/updates/x/artifacts.jar",
        jar(
            "artifacts.xml",
            &artifacts("x", &[("osgi.bundle", "org.other.tool", "3.1.0")], true),
        ),
    );
    other.put(
        "/updates/x/content.jar",
        jar("content.xml", &content("x", &[("org.other.tool", "3.1.0")])),
    );
    other.put(
        "/updates/x/plugins/org.other.tool_3.1.0.jar",
        plugin("Other Tool"),
    );
    World {
        main,
        other,
        main_address,
        other_address,
    }
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_p2::P2::default()));
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
    fixture
}

fn proxy(name: &str, remote: &str) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": true, "timeToLive": 1440},
        "httpClient": {
            "blocked": false,
            "autoBlock": true,
            "authentication": {"type": "username", "username": "mirror", "password": "mirror-secret"}
        }
    })
}

async fn create(fixture: &Fixture, body: Value) -> Reply {
    fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/p2/proxy",
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn get(fixture: &Fixture, path: &str) -> Reply {
    fixture.call(Method::GET, path, Auth::Admin, None).await
}

fn children(xml: &str) -> Vec<String> {
    xml.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("<child location='")
                .and_then(|rest| rest.split_once('\''))
                .map(|(location, _)| location.to_owned())
        })
        .collect()
}

fn join(location: &str, file: &str) -> String {
    format!(
        "/repository/eclipse/{}/{file}",
        location.trim_end_matches('/')
    )
}

#[tokio::test]
async fn a_composite_release_resolves_through_the_proxy() {
    let fixture = ready().await;
    let world = world().await;
    let remote = format!("http://{}/releases/r1/", world.main_address);
    let created = create(&fixture, proxy("eclipse", &remote)).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());

    let index = get(&fixture, "/repository/eclipse/p2.index").await;
    assert_eq!(index.status, StatusCode::OK);
    assert_eq!(index.text(), COMPOSITE_INDEX);
    assert_eq!(index.headers["content-type"], "text/plain");

    let other = format!(".remote/http/{}/updates/x", world.other_address);
    let technology = format!(".remote/http/{}/technology/pkgs/", world.main_address);
    let expected_children = vec![
        "simple".to_owned(),
        technology.clone(),
        "abs".to_owned(),
        other.clone(),
    ];
    for file in ["compositeContent.jar", "compositeArtifacts.jar"] {
        let reply = get(&fixture, &format!("/repository/eclipse/{file}")).await;
        assert_eq!(reply.status, StatusCode::OK, "{file}: {}", reply.text());
        assert_eq!(reply.headers["content-type"], "application/java-archive");
        let xml = unjar(&reply.body);
        assert_eq!(children(&xml), expected_children, "{file}");
        assert!(xml.contains("p2.atomic.composite.loading"));
    }

    let simple_content = get(&fixture, &join("simple", "content.xml.xz")).await;
    assert_eq!(simple_content.status, StatusCode::OK);
    assert_eq!(
        simple_content.body,
        world
            .main
            .files
            .lock()
            .unwrap()
            .get("/releases/r1/simple/content.xml.xz")
            .unwrap()
            .clone(),
        "content metadata passes through unchanged"
    );
    for file in ["artifacts.xml.xz", "artifacts.jar"] {
        let reply = get(&fixture, &join("simple", file)).await;
        assert_eq!(reply.status, StatusCode::OK, "{file}");
        let xml = match file.ends_with(".xz") {
            true => String::from_utf8(unxz(&reply.body)).unwrap(),
            false => unjar(&reply.body),
        };
        assert!(!xml.contains("p2.mirrorsURL"), "{file}");
        assert!(!xml.contains("p2.statsURI"), "{file}");
        assert!(xml.contains("<properties size='2'>"), "{file}");
        assert!(xml.contains("id='org.example.core' version='1.0.0.v1'"));
    }

    let core = get(
        &fixture,
        &join("simple", "plugins/org.example.core_1.0.0.v1.jar"),
    )
    .await;
    assert_eq!(core.status, StatusCode::OK);
    assert_eq!(core.body, plugin("Example Core"));
    assert_eq!(core.headers["content-type"], "application/java-archive");
    let feature_reply = get(
        &fixture,
        &join("simple", "features/org.example.feature_1.0.0.jar"),
    )
    .await;
    assert_eq!(feature_reply.status, StatusCode::OK);

    let pkgs = get(&fixture, &join(&technology, "artifacts.jar")).await;
    assert_eq!(pkgs.status, StatusCode::OK, "{}", pkgs.text());
    let pkg = get(
        &fixture,
        &join(&technology, "plugins/org.example.pkg_2.0.0.jar"),
    )
    .await;
    assert_eq!(pkg.status, StatusCode::OK);
    assert_eq!(pkg.body, plugin("Example Package"));

    let launcher = get(
        &fixture,
        &join("abs", "binary/org.example.launcher_root.linux_1.0.0"),
    )
    .await;
    assert_eq!(launcher.status, StatusCode::OK);
    assert_eq!(launcher.body, b"#!/bin/sh\n");

    let other_artifacts = get(&fixture, &join(&other, "artifacts.jar")).await;
    assert_eq!(other_artifacts.status, StatusCode::OK);
    assert!(!unjar(&other_artifacts.body).contains("mirrorsURL"));
    let tool = get(&fixture, &join(&other, "plugins/org.other.tool_3.1.0.jar")).await;
    assert_eq!(tool.status, StatusCode::OK);
    assert_eq!(tool.body, plugin("Other Tool"));

    let head = fixture
        .call(
            Method::HEAD,
            &join("simple", "plugins/org.example.core_1.0.0.v1.jar"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(head.status, StatusCode::OK);
    assert!(head.headers.contains_key("last-modified"));
    assert!(head.body.is_empty());
    let again = get(&fixture, "/repository/eclipse/compositeContent.jar").await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(world.main.hits("/releases/r1/compositeContent.jar"), 1);
    assert_eq!(
        world
            .main
            .hits("/releases/r1/simple/plugins/org.example.core_1.0.0.v1.jar"),
        1
    );

    let authorized = world.main.authorized();
    assert!(authorized.contains(&"/releases/r1/compositeContent.jar".to_owned()));
    assert!(
        world.other.authorized().is_empty(),
        "the remote's credentials stay with the remote"
    );

    let components = get(
        &fixture,
        "/service/rest/v1/search?repository=eclipse&sort=name",
    )
    .await
    .json();
    let found: Vec<(String, String, String)> = components["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["group"].as_str().unwrap().to_owned(),
                item["name"].as_str().unwrap().to_owned(),
                item["version"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            ("plugins", "org.example.core", "1.0.0.v1"),
            ("features", "org.example.feature", "1.0.0"),
            ("binary", "org.example.launcher_root.linux", "1.0.0"),
            ("plugins", "org.example.pkg", "2.0.0"),
            ("plugins", "org.other.tool", "3.1.0"),
        ]
        .map(|(a, b, c)| (a.to_owned(), b.to_owned(), c.to_owned()))
    );
    let core_item = components["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "org.example.core")
        .unwrap();
    let asset = &core_item["assets"][0];
    assert_eq!(
        asset["path"],
        "/simple/plugins/org.example.core_1.0.0.v1.jar"
    );
    assert_eq!(asset["p2"]["pluginName"], "Example Core");
    assert_eq!(asset["p2"]["classifier"], "osgi.bundle");
    assert_eq!(asset["p2"]["properties"]["artifact.size"], "512");

    let by_name = get(
        &fixture,
        "/service/rest/v1/search?format=p2&p2.pluginName=Example%20Feature",
    )
    .await
    .json();
    assert_eq!(by_name["items"].as_array().unwrap().len(), 1);
    assert_eq!(by_name["items"][0]["name"], "org.example.feature");
    let by_keyword = get(&fixture, "/service/rest/v1/search?q=Package")
        .await
        .json();
    assert_eq!(by_keyword["items"].as_array().unwrap().len(), 1);
    assert_eq!(by_keyword["items"][0]["name"], "org.example.pkg");

    let tree = get(&fixture, &format!("{BROWSE}/eclipse?path=/simple/plugins/"))
        .await
        .json();
    assert_eq!(tree["children"][0]["name"], "org.example.core");
    let version = get(
        &fixture,
        &format!("{BROWSE}/eclipse?path=/simple/plugins/org.example.core/"),
    )
    .await
    .json();
    assert_eq!(version["children"][0]["kind"], "component");
    assert_eq!(version["children"][0]["name"], "1.0.0.v1");
    let root = get(&fixture, &format!("{BROWSE}/eclipse?path=/"))
        .await
        .json();
    let names: Vec<&str> = root["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&".remote"), "{names:?}");
    assert!(names.contains(&"compositeContent.jar"), "{names:?}");
}

#[tokio::test]
async fn the_proxy_fetches_only_what_the_remote_names() {
    let fixture = ready().await;
    let world = world().await;
    let remote = format!("http://{}/releases/r1", world.main_address);
    assert_eq!(
        create(&fixture, proxy("eclipse", &remote)).await.status,
        StatusCode::CREATED
    );
    let unnamed = format!(
        "/repository/eclipse/.remote/http/{}/technology/secret/plugins/a_1.0.0.jar",
        world.main_address
    );
    assert_eq!(get(&fixture, &unnamed).await.status, StatusCode::NOT_FOUND);
    assert_eq!(world.main.hits("/technology/secret/plugins/a_1.0.0.jar"), 0);
    let before_composite = format!(
        "/repository/eclipse/.remote/http/{}/updates/x/artifacts.jar",
        world.other_address
    );
    assert_eq!(
        get(&fixture, &before_composite).await.status,
        StatusCode::NOT_FOUND
    );
    assert!(world.other.hits.lock().unwrap().is_empty());
    assert_eq!(
        get(&fixture, "/repository/eclipse/compositeArtifacts.jar")
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get(&fixture, &before_composite).await.status,
        StatusCode::OK
    );
    for path in [
        "/repository/eclipse/site.xml",
        "/repository/eclipse/simple/index.html",
        "/repository/eclipse/.remote/ftp/example.org/x/content.jar",
    ] {
        assert_eq!(
            get(&fixture, path).await.status,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    assert_eq!(world.main.hits("/releases/r1/site.xml"), 0);
    let page = get(&fixture, "/repository/eclipse/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text().contains("p2 proxy repository"));
    let refused = fixture
        .call(
            Method::PUT,
            "/repository/eclipse/simple/plugins/x_1.0.0.jar",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(refused.status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(refused.headers["allow"], "GET,HEAD,DELETE");
    let missing = get(&fixture, "/repository/eclipse/simple/artifacts.xml").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    let deleted = fixture
        .call(
            Method::DELETE,
            "/repository/eclipse/compositeArtifacts.jar",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn p2_repositories_are_proxies_only() {
    let fixture = ready().await;
    for kind in ["hosted", "group"] {
        let reply = fixture
            .call(
                Method::POST,
                &format!("/service/rest/v1/repositories/p2/{kind}"),
                Auth::Admin,
                Some(json!({"name": format!("p2-{kind}"), "online": true, "storage": {"blobStoreName": "default"}})),
            )
            .await;
        assert!(
            matches!(
                reply.status,
                StatusCode::NOT_FOUND | StatusCode::METHOD_NOT_ALLOWED | StatusCode::BAD_REQUEST
            ),
            "{kind}: {}",
            reply.status
        );
    }
    let missing_remote = create(
        &fixture,
        json!({
            "name": "no-remote",
            "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": true},
            "proxy": {"contentMaxAge": 1440, "metadataMaxAge": 1440},
            "negativeCache": {"enabled": true, "timeToLive": 1440},
            "httpClient": {"blocked": false, "autoBlock": true}
        }),
    )
    .await;
    assert_eq!(missing_remote.status, StatusCode::BAD_REQUEST);
    assert!(
        missing_remote.text().contains("remoteUrl"),
        "{}",
        missing_remote.text()
    );
    let created = create(
        &fixture,
        proxy("eclipse", "https://download.eclipse.org/releases/2024-03/"),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    let stored = get(&fixture, "/service/rest/v1/repositories/p2/proxy/eclipse")
        .await
        .json();
    assert_eq!(stored["format"], "p2");
    assert_eq!(stored["type"], "proxy");
    assert_eq!(
        stored["proxy"]["remoteUrl"],
        "https://download.eclipse.org/releases/2024-03/"
    );
    assert!(
        stored["httpClient"]["authentication"]
            .get("password")
            .is_none()
    );
    let mut changed = stored.clone();
    changed["proxy"]["metadataMaxAge"] = json!(60);
    let updated = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/repositories/p2/proxy/eclipse",
            Auth::Admin,
            Some(changed),
        )
        .await;
    assert_eq!(updated.status, StatusCode::NO_CONTENT, "{}", updated.text());
    let reread = get(&fixture, "/service/rest/v1/repositories/p2/proxy/eclipse")
        .await
        .json();
    assert_eq!(reread["proxy"]["metadataMaxAge"], 60);
    let recipes = get(&fixture, "/service/rest/internal/ui/repositories/recipes")
        .await
        .json();
    assert!(
        recipes
            .as_array()
            .unwrap()
            .iter()
            .any(|recipe| recipe["format"] == "p2" && recipe["type"] == "proxy"),
        "{recipes}"
    );
}

#[tokio::test]
async fn changed_remote_files_are_rewritten_again_and_unchanged_ones_are_kept() {
    let fixture = ready().await;
    let world = world().await;
    let remote = format!("http://{}/releases/r1/", world.main_address);
    let mut body = proxy("eager", &remote);
    body["proxy"]["metadataMaxAge"] = json!(0);
    assert_eq!(create(&fixture, body).await.status, StatusCode::CREATED);
    let path = "/repository/eager/simple/artifacts.jar";
    let first = get(&fixture, path).await;
    assert_eq!(first.status, StatusCode::OK);
    assert!(!unjar(&first.body).contains("p2.mirrorsURL"));
    let second = get(&fixture, path).await;
    assert_eq!(second.status, StatusCode::OK);
    assert_eq!(second.body, first.body, "a 304 keeps the rewritten copy");
    assert_eq!(world.main.hits("/releases/r1/simple/artifacts.jar"), 2);

    let changed = artifacts(
        "simple",
        &[("osgi.bundle", "org.example.core", "1.1.0")],
        true,
    );
    world.main.put(
        "/releases/r1/simple/artifacts.jar",
        jar("artifacts.xml", &changed),
    );
    let third = get(&fixture, path).await;
    assert_eq!(third.status, StatusCode::OK);
    let text = unjar(&third.body);
    assert!(text.contains("version='1.1.0'"), "{text}");
    assert!(!text.contains("p2.mirrorsURL"));
    assert!(!text.contains("p2.statsURI"));
    let assets = get(&fixture, "/service/rest/v1/assets?repository=eager")
        .await
        .json();
    let stored = assets["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["path"] == "/simple/artifacts.jar")
        .unwrap()
        .clone();
    assert_eq!(stored["p2"]["checked"], true);
}
