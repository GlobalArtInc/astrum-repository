mod support;

use std::sync::{Arc, Mutex};

use astrum_testkit::tls::TestCertificates;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use support::{Auth, Fixture};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

const EMAIL: &str = "/service/rest/v1/email";
const PLACEHOLDER: &str = "#~ASTRUM~PLACEHOLDER~PASSWORD~#";

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

#[derive(Default)]
struct Received {
    commands: Vec<String>,
    data: Vec<String>,
    secure: bool,
}

type Sink = Arc<Mutex<Received>>;

async fn conversation(stream: Box<dyn Io>, tls: Option<tokio_rustls::TlsAcceptor>, sink: Sink) {
    let mut reader = BufReader::new(stream);
    let _ = reader.get_mut().write_all(b"220 sink ready\r\n").await;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
            return;
        }
        let line = line.trim_end().to_owned();
        sink.lock().unwrap().commands.push(line.clone());
        let upper = line.to_ascii_uppercase();
        let answer: &[u8] = if upper.starts_with("EHLO") {
            if tls.is_some() && !sink.lock().unwrap().secure {
                b"250-sink\r\n250-AUTH PLAIN LOGIN\r\n250 STARTTLS\r\n"
            } else {
                b"250-sink\r\n250 AUTH PLAIN LOGIN\r\n"
            }
        } else if upper == "STARTTLS" {
            let _ = reader.get_mut().write_all(b"220 go ahead\r\n").await;
            let Some(acceptor) = tls.clone() else {
                return;
            };
            let Ok(secure) = acceptor.accept(reader.into_inner()).await else {
                return;
            };
            sink.lock().unwrap().secure = true;
            reader = BufReader::new(Box::new(secure));
            continue;
        } else if upper.starts_with("AUTH") {
            b"235 accepted\r\n"
        } else if upper == "DATA" {
            let _ = reader.get_mut().write_all(b"354 send it\r\n").await;
            let mut data = String::new();
            loop {
                let mut part = String::new();
                if reader.read_line(&mut part).await.unwrap_or(0) == 0 {
                    return;
                }
                if part == ".\r\n" {
                    break;
                }
                data.push_str(&part);
            }
            sink.lock().unwrap().data.push(data);
            b"250 queued\r\n"
        } else if upper == "QUIT" {
            let _ = reader.get_mut().write_all(b"221 bye\r\n").await;
            return;
        } else {
            b"250 ok\r\n"
        };
        if reader.get_mut().write_all(answer).await.is_err() {
            return;
        }
    }
}

async fn smtp_sink(tls: Option<&TestCertificates>) -> (u16, Sink) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let sink: Sink = Arc::default();
    let acceptor =
        tls.map(|certificates| tokio_rustls::TlsAcceptor::from(certificates.server_config()));
    let shared = sink.clone();
    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(conversation(
                Box::new(stream),
                acceptor.clone(),
                shared.clone(),
            ));
        }
    });
    (port, sink)
}

async fn verify(fixture: &Fixture, address: &str) -> (StatusCode, Value) {
    let reply = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/email/verify",
            Auth::Admin,
            Body::from(address.to_owned()),
            &[("content-type", "text/plain")],
        )
        .await;
    (reply.status, reply.json())
}

#[tokio::test]
async fn email_settings_are_validated_masked_and_reset() {
    let fixture = Fixture::new().await;
    let initial = fixture.call(Method::GET, EMAIL, Auth::Admin, None).await;
    assert_eq!(initial.json()["host"], "localhost");
    assert_eq!(initial.json()["port"], 25);
    let invalid = fixture
        .call(
            Method::PUT,
            EMAIL,
            Auth::Admin,
            Some(json!({"enabled": true, "host": "", "port": 0, "fromAddress": "bad"})),
        )
        .await;
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid.json().as_array().map(Vec::len), Some(3));
    let saved = fixture
        .call(
            Method::PUT,
            EMAIL,
            Auth::Admin,
            Some(json!({"enabled": true, "host": "mail.example.invalid", "port": 587,
                "username": "robot", "password": "mail-secret", "fromAddress": "repo@example.invalid",
                "startTlsEnabled": true})),
        )
        .await;
    assert_eq!(saved.status, StatusCode::NO_CONTENT, "{}", saved.text());
    let read = fixture.call(Method::GET, EMAIL, Auth::Admin, None).await;
    assert_eq!(read.json()["password"], PLACEHOLDER);
    assert!(!read.text().contains("mail-secret"));
    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=email",
            Auth::Admin,
            None,
        )
        .await;
    assert!(!audit.text().contains("mail-secret"));
    let reset = fixture.call(Method::DELETE, EMAIL, Auth::Admin, None).await;
    assert_eq!(reset.status, StatusCode::NO_CONTENT);
    let read = fixture.call(Method::GET, EMAIL, Auth::Admin, None).await;
    assert_eq!(read.json(), initial.json());

    fixture.add_user("jane", "secret-1", &[]).await;
    let denied = fixture
        .call(Method::GET, EMAIL, Auth::Basic("jane", "secret-1"), None)
        .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_test_mail_reaches_a_plain_smtp_server() {
    let fixture = Fixture::new().await;
    let (port, sink) = smtp_sink(None).await;
    let saved = fixture
        .call(
            Method::PUT,
            EMAIL,
            Auth::Admin,
            Some(json!({"enabled": true, "host": "127.0.0.1", "port": port,
                "username": "robot", "password": "mail-secret", "fromAddress": "repo@example.invalid",
                "subjectPrefix": "[Astrum]"})),
        )
        .await;
    assert_eq!(saved.status, StatusCode::NO_CONTENT, "{}", saved.text());
    let (status, body) = verify(&fixture, "admin@example.invalid").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"success": true, "reason": null}));
    {
        let received = sink.lock().unwrap();
        assert!(
            received
                .commands
                .contains(&"MAIL FROM:<repo@example.invalid>".to_owned())
        );
        assert!(
            received
                .commands
                .contains(&"RCPT TO:<admin@example.invalid>".to_owned())
        );
        let token = STANDARD.encode("\0robot\0mail-secret");
        assert!(received.commands.contains(&format!("AUTH PLAIN {token}")));
        let data = &received.data[0];
        assert!(data.contains("Subject: [Astrum] Email configuration test\r\n"));
        assert!(data.contains("To: admin@example.invalid\r\n"));
    }

    let (status, body) = verify(&fixture, "not an address").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    let mailer = fixture.state.mailer();
    mailer
        .send(&astrum_mail::Message {
            to: vec!["ops@example.invalid".to_owned()],
            subject: "Task failed".to_owned(),
            text: "The cleanup task failed.".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(sink.lock().unwrap().data.len(), 2);
}

#[tokio::test]
async fn starttls_uses_the_truststore_when_asked() {
    let fixture = Fixture::new().await;
    let certificates = TestCertificates::generate(&["localhost"]);
    let (port, sink) = smtp_sink(Some(&certificates)).await;
    let settings = json!({"enabled": true, "host": "localhost", "port": port,
        "fromAddress": "repo@example.invalid", "startTlsEnabled": true, "startTlsRequired": true,
        "sslServerIdentityCheckEnabled": true, "astrumTrustStoreEnabled": true});
    let saved = fixture
        .call(Method::PUT, EMAIL, Auth::Admin, Some(settings.clone()))
        .await;
    assert_eq!(saved.status, StatusCode::NO_CONTENT);
    let (status, body) = verify(&fixture, "admin@example.invalid").await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "an unknown authority is refused"
    );
    assert!(body["reason"].as_str().unwrap().contains("TLS"), "{body}");

    let added = fixture
        .send_body(
            Method::POST,
            "/service/rest/v1/security/ssl/truststore",
            Auth::Admin,
            Body::from(certificates.ca_pem.clone()),
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED);
    let (status, body) = verify(&fixture, "admin@example.invalid").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(sink.lock().unwrap().secure);
    assert_eq!(sink.lock().unwrap().data.len(), 1);
}

#[tokio::test]
async fn notification_emails_go_through_the_email_server() {
    let fixture = Fixture::new().await;
    let channel = json!({
        "name": "Mail",
        "type": "email",
        "recipients": ["team@example.invalid"],
        "events": ["published"],
        "language": "en",
    });
    let test = "/service/rest/internal/ui/notifications/test";
    let refused = fixture
        .call(Method::POST, test, Auth::Admin, Some(channel.clone()))
        .await
        .json();
    assert_eq!(refused["success"], false);
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("Email is not set up"),
        "{refused}"
    );

    let (port, sink) = smtp_sink(None).await;
    let saved = fixture
        .call(
            Method::PUT,
            EMAIL,
            Auth::Admin,
            Some(json!({"enabled": true, "host": "127.0.0.1", "port": port,
                "fromAddress": "repo@example.invalid", "subjectPrefix": "[Astrum]"})),
        )
        .await;
    assert_eq!(saved.status, StatusCode::NO_CONTENT, "{}", saved.text());
    let sent = fixture
        .call(Method::POST, test, Auth::Admin, Some(channel))
        .await
        .json();
    assert_eq!(sent["success"], true, "{sent}");
    let received = sink.lock().unwrap();
    assert!(
        received
            .commands
            .contains(&"RCPT TO:<team@example.invalid>".to_owned())
    );
    assert_eq!(received.data.len(), 1);
    assert!(
        received.data[0].contains("Subject: [Astrum] "),
        "{}",
        received.data[0]
    );
}
