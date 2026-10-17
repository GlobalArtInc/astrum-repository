mod support;

use std::sync::{Arc, Mutex};

use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use astrum_testkit::tls::TestCertificates;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use base64::Engine;
use serde_json::{Value, json};
use support::{Auth, Fixture};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const HTTP: &str = "/service/rest/v1/http";
const TRUSTSTORE: &str = "/service/rest/v1/security/ssl/truststore";
const PLACEHOLDER: &str = "#~ASTRUM~PLACEHOLDER~PASSWORD~#";

async fn ready(loopback: bool) -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    if loopback {
        astrum_db::setting::put(
            fixture.state.db().conn(),
            SSRF_KEY,
            &SsrfProtection {
                enabled: true,
                allowed_ips: vec!["127.0.0.1".to_owned(), "::1".to_owned()],
                allowed_domains: vec![],
            },
        )
        .await
        .unwrap();
    }
    fixture
}

type Heads = Arc<Mutex<Vec<String>>>;

async fn read_head<S: AsyncReadExt + Unpin>(stream: &mut S) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).await.unwrap_or(0) == 0 {
            break;
        }
        head.push(byte[0]);
    }
    String::from_utf8_lossy(&head).into_owned()
}

fn reply(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

async fn plain_server(body: &'static str) -> (u16, Heads) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let heads: Heads = Arc::default();
    let seen = heads.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let seen = seen.clone();
            tokio::spawn(async move {
                let head = read_head(&mut stream).await;
                seen.lock().unwrap().push(head);
                let _ = stream.write_all(reply(body).as_bytes()).await;
                let _ = stream.shutdown().await;
            });
        }
    });
    (port, heads)
}

async fn tls_server(certificates: &TestCertificates) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acceptor = tokio_rustls::TlsAcceptor::from(certificates.server_config());
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(mut stream) = acceptor.accept(stream).await else {
                    return;
                };
                read_head(&mut stream).await;
                let _ = stream.write_all(reply("secure content").as_bytes()).await;
                let _ = stream.shutdown().await;
            });
        }
    });
    port
}

fn proxy(name: &str, remote: &str, trust: bool) -> Value {
    json!({
        "name": name,
        "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false},
        "proxy": {"remoteUrl": remote, "contentMaxAge": 1440, "metadataMaxAge": 1440},
        "negativeCache": {"enabled": false, "timeToLive": 0},
        "httpClient": {"blocked": false, "autoBlock": false,
            "connection": {"retries": 0, "useTrustStore": trust, "userAgentSuffix": "Mirror"}},
        "raw": {"contentDisposition": "ATTACHMENT"}
    })
}

async fn create_proxy(fixture: &Fixture, body: Value) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/proxy",
            Auth::Admin,
            Some(body),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn put_text(fixture: &Fixture, path: &str, text: &str) -> support::Reply {
    fixture
        .send_body(
            Method::POST,
            path,
            Auth::Admin,
            Body::from(text.to_owned()),
            &[("content-type", "text/plain")],
        )
        .await
}

#[tokio::test]
async fn http_settings_are_validated_and_hide_passwords() {
    let fixture = ready(false).await;
    let initial = fixture.call(Method::GET, HTTP, Auth::Admin, None).await;
    assert_eq!(initial.status, StatusCode::OK);
    assert_eq!(
        initial.json()["httpProxy"]["authInfo"]["password"],
        Value::Null
    );
    assert_eq!(initial.json()["nonProxyHosts"], json!([]));

    let invalid = fixture
        .call(
            Method::PUT,
            HTTP,
            Auth::Admin,
            Some(json!({"timeout": 0, "retries": 11, "httpProxy": {"enabled": true, "host": "", "port": "x"}})),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    let ids: Vec<String> = invalid
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|problem| problem["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        ids,
        [
            "PARAMETER timeout",
            "PARAMETER retries",
            "httpProxy.host",
            "httpProxy.port"
        ]
    );

    let saved = fixture
        .call(
            Method::PUT,
            HTTP,
            Auth::Admin,
            Some(json!({"userAgent": "Corp/1", "timeout": 15, "retries": 3,
                "httpProxy": {"enabled": true, "host": "proxy.example", "port": 3128,
                    "authInfo": {"enabled": true, "username": "u", "password": "proxy-secret"}},
                "nonProxyHosts": ["*.example.com"]})),
        )
        .await;
    assert_eq!(saved.status, StatusCode::OK, "{}", saved.text());
    assert_eq!(saved.json()["httpProxy"]["port"], "3128");
    let read = fixture.call(Method::GET, HTTP, Auth::Admin, None).await;
    assert_eq!(
        read.json()["httpProxy"]["authInfo"]["password"],
        PLACEHOLDER
    );
    assert!(!read.text().contains("proxy-secret"));
    let mut again = read.json();
    again["timeout"] = json!(20);
    let kept = fixture
        .call(Method::PUT, HTTP, Auth::Admin, Some(again))
        .await;
    assert_eq!(kept.status, StatusCode::OK);
    let stored: Value = astrum_db::setting::get(fixture.state.db().conn(), "http.client")
        .await
        .unwrap()
        .unwrap();
    let sealed = stored["httpProxy"]["authInfo"]["password"]
        .as_str()
        .unwrap();
    assert!(sealed.starts_with("astrum:v1:"));
    assert_eq!(
        fixture.state.secrets().open(sealed).unwrap(),
        "proxy-secret"
    );

    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=httpclient",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(audit.json().as_array().map(Vec::len), Some(2));
    assert!(!audit.text().contains("proxy-secret"));

    fixture.add_user("jane", "secret-1", &[]).await;
    let denied = fixture
        .call(Method::GET, HTTP, Auth::Basic("jane", "secret-1"), None)
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn proxy_repositories_go_through_the_configured_http_proxy() {
    let fixture = ready(false).await;
    let (proxy_port, heads) = plain_server("via proxy").await;
    let reply = fixture
        .call(
            Method::PUT,
            HTTP,
            Auth::Admin,
            Some(json!({"userAgent": "Corp/1",
                "httpProxy": {"enabled": true, "host": "127.0.0.1", "port": proxy_port.to_string(),
                    "authInfo": {"enabled": true, "username": "u", "password": "p"}},
                "nonProxyHosts": ["*.direct.invalid"]})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    create_proxy(&fixture, proxy("mirror", "http://upstream.invalid/", false)).await;
    let fetched = fixture
        .call(Method::GET, "/repository/mirror/a.txt", Auth::Admin, None)
        .await;
    assert_eq!(
        (fetched.status, fetched.text()),
        (StatusCode::OK, "via proxy".to_owned())
    );
    let head = heads.lock().unwrap()[0].to_ascii_lowercase();
    assert!(
        head.starts_with("get http://upstream.invalid/a.txt http/1.1"),
        "{head}"
    );
    let credentials = base64::engine::general_purpose::STANDARD.encode("u:p");
    assert!(head.contains(&format!(
        "proxy-authorization: basic {}",
        credentials.to_ascii_lowercase()
    )));
    assert!(head.contains("user-agent: astrum/"));
    assert!(head.contains(" corp/1 mirror"), "{head}");

    create_proxy(
        &fixture,
        proxy("direct", "http://repo.direct.invalid/", false),
    )
    .await;
    let direct = fixture
        .call(Method::GET, "/repository/direct/a.txt", Auth::Admin, None)
        .await;
    assert_eq!(direct.status, StatusCode::BAD_GATEWAY, "{}", direct.text());
    assert_eq!(heads.lock().unwrap().len(), 1, "bypassed hosts go direct");
}

#[tokio::test]
async fn the_truststore_lets_proxies_reach_private_certificates() {
    let fixture = ready(true).await;
    let certificates = TestCertificates::generate(&["localhost"]);
    let port = tls_server(&certificates).await;
    let remote = format!("https://localhost:{port}/");
    create_proxy(&fixture, proxy("trusting", &remote, true)).await;
    create_proxy(&fixture, proxy("plain", &remote, false)).await;
    let path = "/repository/trusting/a.txt";
    assert_eq!(
        fixture
            .call(Method::GET, path, Auth::Admin, None)
            .await
            .status,
        StatusCode::BAD_GATEWAY
    );

    let fetched = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/security/ssl?host=localhost&port={port}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.text());
    let leaf = fetched.json();
    assert_eq!(leaf["subjectCommonName"], "localhost");
    assert_eq!(leaf["issuerCommonName"], "Astrum test CA");

    let added = put_text(&fixture, TRUSTSTORE, leaf["pem"].as_str().unwrap()).await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.text());
    assert_eq!(added.json()["id"], leaf["id"]);
    let duplicate = put_text(&fixture, TRUSTSTORE, leaf["pem"].as_str().unwrap()).await;
    assert_eq!(duplicate.status, StatusCode::CONFLICT);
    let listed = fixture
        .call(Method::GET, TRUSTSTORE, Auth::Admin, None)
        .await;
    assert_eq!(listed.json().as_array().map(Vec::len), Some(1));

    let trusted = fixture.call(Method::GET, path, Auth::Admin, None).await;
    assert_eq!(
        (trusted.status, trusted.text()),
        (StatusCode::OK, "secure content".to_owned())
    );
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/plain/a.txt", Auth::Admin, None)
            .await
            .status,
        StatusCode::BAD_GATEWAY,
        "repositories that do not use the truststore keep the system roots"
    );

    let id = leaf["id"].as_str().unwrap();
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("{TRUSTSTORE}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let missing = fixture
        .call(
            Method::DELETE,
            &format!("{TRUSTSTORE}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/trusting/b.txt", Auth::Admin, None)
            .await
            .status,
        StatusCode::BAD_GATEWAY
    );

    let ca = put_text(&fixture, TRUSTSTORE, &certificates.ca_pem).await;
    assert_eq!(ca.status, StatusCode::CREATED);
    assert_eq!(
        fixture
            .call(Method::GET, "/repository/trusting/c.txt", Auth::Admin, None)
            .await
            .status,
        StatusCode::OK,
        "a trusted authority covers the certificates it signed"
    );
}

#[tokio::test]
async fn certificates_are_checked_and_ssrf_protection_applies_to_retrieval() {
    let fixture = ready(false).await;
    let certificates = TestCertificates::generate(&["localhost"]);
    let port = tls_server(&certificates).await;
    let blocked = fixture
        .call(
            Method::GET,
            &format!("/service/rest/v1/security/ssl?host=127.0.0.1&port={port}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(blocked.status, StatusCode::BAD_REQUEST);
    assert!(blocked.json()["message"].as_str().unwrap().starts_with(
        "Host resolves to private/local IP address(es): 127.0.0.1 (loopback address)"
    ));
    let not_pem = put_text(&fixture, TRUSTSTORE, "hello").await;
    assert_eq!(not_pem.status, StatusCode::BAD_REQUEST);
    let details = put_text(
        &fixture,
        "/service/rest/internal/ui/ssl/certificate-details",
        &certificates.ca_pem,
    )
    .await;
    assert_eq!(details.status, StatusCode::OK, "{}", details.text());
    assert_eq!(details.json()["subjectCommonName"], "Astrum test CA");
    let listed = fixture
        .call(Method::GET, TRUSTSTORE, Auth::Admin, None)
        .await;
    assert_eq!(listed.json(), json!([]), "details do not store anything");

    fixture.add_user("jane", "secret-1", &[]).await;
    let denied = fixture
        .call(
            Method::GET,
            TRUSTSTORE,
            Auth::Basic("jane", "secret-1"),
            None,
        )
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}
