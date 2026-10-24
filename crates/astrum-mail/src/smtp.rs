use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rustls::pki_types::{CertificateDer, ServerName};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use crate::{MailError, Message, settings::EmailSettings};

const STEP_TIMEOUT: Duration = Duration::from_secs(30);

pub trait Io: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

struct Session {
    reader: BufReader<Box<dyn Io>>,
}

struct Reply {
    code: u16,
    lines: Vec<String>,
}

impl Reply {
    fn text(&self) -> String {
        format!("{} {}", self.code, self.lines.join(" "))
    }
}

async fn timed<T>(
    future: impl Future<Output = Result<T, MailError>>,
    what: &str,
) -> Result<T, MailError> {
    tokio::time::timeout(STEP_TIMEOUT, future)
        .await
        .map_err(|_| {
            MailError::Connection(format!("the mail server did not answer {what} in time"))
        })?
}

impl Session {
    async fn read(&mut self) -> Result<Reply, MailError> {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let read = self
                .reader
                .read_line(&mut line)
                .await
                .map_err(|error| MailError::Connection(error.to_string()))?;
            if read == 0 {
                return Err(MailError::Connection(
                    "the mail server closed the connection".to_owned(),
                ));
            }
            let line = line.trim_end();
            let code = line
                .get(..3)
                .and_then(|code| code.parse::<u16>().ok())
                .ok_or_else(|| MailError::Protocol(format!("unexpected answer: {line}")))?;
            lines.push(line.get(4..).unwrap_or_default().to_owned());
            if line.as_bytes().get(3) != Some(&b'-') {
                return Ok(Reply { code, lines });
            }
        }
    }

    async fn expect(&mut self, what: &str, accepted: &[u16]) -> Result<Reply, MailError> {
        let reply = timed(self.read(), what).await?;
        if accepted.contains(&reply.code) {
            Ok(reply)
        } else {
            Err(MailError::Protocol(format!(
                "the mail server refused {what}: {}",
                reply.text()
            )))
        }
    }

    async fn send(&mut self, line: &str) -> Result<(), MailError> {
        let stream = self.reader.get_mut();
        timed(
            async {
                stream
                    .write_all(format!("{line}\r\n").as_bytes())
                    .await
                    .map_err(|error| MailError::Connection(error.to_string()))?;
                stream
                    .flush()
                    .await
                    .map_err(|error| MailError::Connection(error.to_string()))
            },
            "a command",
        )
        .await
    }

    async fn command(
        &mut self,
        line: &str,
        what: &str,
        accepted: &[u16],
    ) -> Result<Reply, MailError> {
        self.send(line).await?;
        self.expect(what, accepted).await
    }
}

async fn wrap(
    stream: Box<dyn Io>,
    settings: &EmailSettings,
    trust: &[CertificateDer<'static>],
) -> Result<Box<dyn Io>, MailError> {
    let extra: &[CertificateDer<'static>] = if settings.astrum_trust_store_enabled {
        trust
    } else {
        &[]
    };
    let config =
        astrum_security::tls::client_config(extra, settings.ssl_server_identity_check_enabled)
            .map_err(|error| MailError::Tls(error.to_string()))?;
    let name = ServerName::try_from(settings.host.clone())
        .map_err(|_| MailError::Tls(format!("{} is not a valid TLS server name", settings.host)))?;
    let stream = timed(
        async {
            TlsConnector::from(Arc::new(config))
                .connect(name, stream)
                .await
                .map_err(|error| MailError::Tls(error.to_string()))
        },
        "the TLS handshake",
    )
    .await?;
    Ok(Box::new(stream))
}

fn capabilities(reply: &Reply) -> Vec<String> {
    reply
        .lines
        .iter()
        .skip(1)
        .map(|line| line.to_ascii_uppercase())
        .collect()
}

fn encode_header(text: &str) -> String {
    if text.is_ascii() && !text.contains(['\r', '\n']) {
        return text.to_owned();
    }
    let mut words = Vec::new();
    let mut chunk = String::new();
    for c in text.chars().filter(|c| *c != '\r' && *c != '\n') {
        if chunk.len() + c.len_utf8() > 45 {
            words.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(&chunk)));
            chunk.clear();
        }
        chunk.push(c);
    }
    if !chunk.is_empty() {
        words.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(&chunk)));
    }
    words.join("\r\n ")
}

pub fn render(settings: &EmailSettings, message: &Message) -> String {
    let subject = match &settings.subject_prefix {
        Some(prefix) => format!("{prefix} {}", message.subject),
        None => message.subject.clone(),
    };
    let body = STANDARD.encode(message.text.replace("\r\n", "\n").replace('\n', "\r\n"));
    let mut rendered = String::new();
    for (name, value) in [
        ("Date", chrono::Utc::now().to_rfc2822()),
        ("From", settings.from_address.clone()),
        ("To", message.to.join(", ")),
        ("Subject", encode_header(&subject)),
        (
            "Message-ID",
            format!("<{}@astrum>", uuid::Uuid::new_v4().simple()),
        ),
        ("MIME-Version", "1.0".to_owned()),
        ("Content-Type", "text/plain; charset=UTF-8".to_owned()),
        ("Content-Transfer-Encoding", "base64".to_owned()),
    ] {
        rendered.push_str(&format!("{name}: {value}\r\n"));
    }
    rendered.push_str("\r\n");
    for line in body.as_bytes().chunks(76) {
        rendered.push_str(std::str::from_utf8(line).unwrap_or_default());
        rendered.push_str("\r\n");
    }
    rendered
}

pub async fn deliver(
    settings: &EmailSettings,
    password: Option<&str>,
    trust: &[CertificateDer<'static>],
    message: &Message,
) -> Result<(), MailError> {
    let address = format!("{}:{}", settings.host, settings.port);
    let tcp = timed(
        async {
            TcpStream::connect(&address).await.map_err(|error| {
                MailError::Connection(format!("cannot connect to {address}: {error}"))
            })
        },
        "the connection",
    )
    .await?;
    let mut stream: Box<dyn Io> = Box::new(tcp);
    let mut secure = false;
    if settings.ssl_on_connect_enabled {
        stream = wrap(stream, settings, trust).await?;
        secure = true;
    }
    let mut session = Session {
        reader: BufReader::new(stream),
    };
    session.expect("the connection", &[220]).await?;
    let mut hello = session.command("EHLO localhost", "EHLO", &[250]).await?;
    if !secure && settings.start_tls_enabled {
        let offered = capabilities(&hello).iter().any(|line| line == "STARTTLS");
        if offered {
            session.command("STARTTLS", "STARTTLS", &[220]).await?;
            let plain = session.reader.into_inner();
            session = Session {
                reader: BufReader::new(wrap(plain, settings, trust).await?),
            };
            hello = session.command("EHLO localhost", "EHLO", &[250]).await?;
        } else if settings.start_tls_required {
            return Err(MailError::Protocol(
                "STARTTLS is required, but the mail server does not offer it".to_owned(),
            ));
        }
    }
    if !settings.username.is_empty() {
        let password = password.unwrap_or_default();
        let methods: Vec<String> = capabilities(&hello)
            .iter()
            .filter_map(|line| line.strip_prefix("AUTH").map(str::to_owned))
            .flat_map(|rest| {
                rest.split([' ', '='])
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect();
        if methods.iter().any(|method| method == "LOGIN")
            && !methods.iter().any(|method| method == "PLAIN")
        {
            session.command("AUTH LOGIN", "AUTH LOGIN", &[334]).await?;
            session
                .command(
                    &STANDARD.encode(&settings.username),
                    "the user name",
                    &[334],
                )
                .await?;
            session
                .command(&STANDARD.encode(password), "the credentials", &[235])
                .await?;
        } else {
            let token = STANDARD.encode(format!("\0{}\0{password}", settings.username));
            session
                .command(&format!("AUTH PLAIN {token}"), "the credentials", &[235])
                .await?;
        }
    }
    session
        .command(
            &format!("MAIL FROM:<{}>", settings.from_address),
            "the sender",
            &[250],
        )
        .await?;
    for recipient in &message.to {
        session
            .command(
                &format!("RCPT TO:<{recipient}>"),
                "the recipient",
                &[250, 251],
            )
            .await?;
    }
    session.command("DATA", "DATA", &[354]).await?;
    let mut data = String::new();
    for line in render(settings, message).split("\r\n") {
        if line.starts_with('.') {
            data.push('.');
        }
        data.push_str(line);
        data.push_str("\r\n");
    }
    let data = data.trim_end_matches("\r\n").to_owned();
    session.send(&format!("{data}\r\n.")).await?;
    session.expect("the message", &[250]).await?;
    let _ = session.command("QUIT", "QUIT", &[221]).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_mime_with_encoded_subjects() {
        let settings = EmailSettings {
            subject_prefix: Some("[Astrum]".to_owned()),
            ..Default::default()
        };
        let message = Message {
            to: vec!["a@example.invalid".to_owned()],
            subject: "Проверка".to_owned(),
            text: "line one\nline two".to_owned(),
        };
        let rendered = render(&settings, &message);
        assert!(rendered.contains("From: astrum@example.org\r\n"));
        assert!(rendered.contains("To: a@example.invalid\r\n"));
        assert!(rendered.contains("Subject: =?UTF-8?B?"));
        let body = rendered
            .split("\r\n\r\n")
            .nth(1)
            .unwrap()
            .replace("\r\n", "");
        assert_eq!(
            STANDARD.decode(body).unwrap(),
            b"line one\r\nline two".to_vec()
        );
        assert_eq!(encode_header("plain"), "plain");
    }
}
