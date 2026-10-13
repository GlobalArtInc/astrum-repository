use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_security::ssrf::Guard;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use serde::Serialize;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use x509_parser::prelude::{FromDer, X509Certificate, X509Name};

const FETCH_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub id: String,
    pub fingerprint: String,
    pub pem: String,
    pub serial_number: String,
    pub subject_common_name: Option<String>,
    pub subject_organization: Option<String>,
    pub subject_organizational_unit: Option<String>,
    pub issuer_common_name: Option<String>,
    pub issuer_organization: Option<String>,
    pub issuer_organizational_unit: Option<String>,
    pub issued_on: i64,
    pub expires_on: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum CertificateError {
    #[error("the text is not a PEM encoded X.509 certificate")]
    NotPem,
    #[error("the certificate cannot be read: {0}")]
    Malformed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("{0}")]
    Blocked(String),
    #[error("{0}")]
    Failed(String),
}

pub fn fingerprint(der: &[u8]) -> String {
    Sha1::digest(der)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn pem_of(der: &[u8]) -> String {
    let encoded = STANDARD.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for line in encoded.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(line).unwrap_or_default());
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

pub fn der_from_pem(text: &str) -> Result<Vec<u8>, CertificateError> {
    let text = text.trim();
    let text = match serde_json::from_str::<String>(text) {
        Ok(unquoted) if text.starts_with('"') => unquoted,
        _ => text.to_owned(),
    };
    let text = text.replace("\\n", "\n");
    let start = text
        .find("-----BEGIN CERTIFICATE-----")
        .ok_or(CertificateError::NotPem)?;
    let (_, pem) = x509_parser::pem::parse_x509_pem(&text.as_bytes()[start..])
        .map_err(|_| CertificateError::NotPem)?;
    if pem.label != "CERTIFICATE" {
        return Err(CertificateError::NotPem);
    }
    X509Certificate::from_der(&pem.contents)
        .map_err(|error| CertificateError::Malformed(error.to_string()))?;
    Ok(pem.contents)
}

fn first<'a>(
    name: &'a X509Name<'_>,
    pick: impl Fn(&'a X509Name<'_>) -> Option<&'a x509_parser::x509::AttributeTypeAndValue<'a>>,
) -> Option<String> {
    pick(name)
        .and_then(|value| value.as_str().ok())
        .map(str::to_owned)
}

pub fn describe(der: &[u8]) -> Result<Certificate, CertificateError> {
    let (_, parsed) = X509Certificate::from_der(der)
        .map_err(|error| CertificateError::Malformed(error.to_string()))?;
    let subject = parsed.subject();
    let issuer = parsed.issuer();
    let fingerprint = fingerprint(der);
    Ok(Certificate {
        id: fingerprint.clone(),
        fingerprint,
        pem: pem_of(der),
        serial_number: parsed.tbs_certificate.serial.to_string(),
        subject_common_name: first(subject, |name| name.iter_common_name().next()),
        subject_organization: first(subject, |name| name.iter_organization().next()),
        subject_organizational_unit: first(subject, |name| name.iter_organizational_unit().next()),
        issuer_common_name: first(issuer, |name| name.iter_common_name().next()),
        issuer_organization: first(issuer, |name| name.iter_organization().next()),
        issuer_organizational_unit: first(issuer, |name| name.iter_organizational_unit().next()),
        issued_on: parsed.validity().not_before.timestamp() * 1000,
        expires_on: parsed.validity().not_after.timestamp() * 1000,
    })
}

#[derive(Debug)]
struct Capture {
    chain: Mutex<Vec<CertificateDer<'static>>>,
    schemes: Vec<SignatureScheme>,
}

impl ServerCertVerifier for Capture {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let mut chain = self
            .chain
            .lock()
            .expect("the capture lock is never poisoned");
        chain.push(end_entity.clone().into_owned());
        chain.extend(intermediates.iter().map(|der| der.clone().into_owned()));
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.schemes.clone()
    }
}

pub fn ip_description(ip: IpAddr) -> &'static str {
    match ip {
        IpAddr::V4(v4) if v4.is_loopback() => "loopback address",
        IpAddr::V6(v6) if v6.is_loopback() => "loopback address",
        IpAddr::V4(v4) if v4.is_private() => "private address",
        IpAddr::V4(v4) if v4.is_link_local() => "link-local address",
        _ => "internal address",
    }
}

pub async fn resolve_guarded(
    guard: &Guard,
    host: &str,
    port: u16,
) -> Result<Vec<SocketAddr>, FetchError> {
    let found: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| FetchError::Failed(format!("{host} cannot be resolved: {error}")))?
        .collect();
    if found.is_empty() {
        return Err(FetchError::Failed(format!("{host} cannot be resolved")));
    }
    let host_allowed = guard.host_allowed(host);
    let allowed: Vec<SocketAddr> = found
        .iter()
        .copied()
        .filter(|address| guard.ip_allowed(address.ip(), host_allowed))
        .collect();
    if allowed.is_empty() {
        let blocked: Vec<String> = found
            .iter()
            .map(|address| format!("{} ({})", address.ip(), ip_description(address.ip())))
            .collect();
        return Err(FetchError::Blocked(format!(
            "Host resolves to private/local IP address(es): {}. To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection.",
            blocked.join(", ")
        )));
    }
    Ok(allowed)
}

async fn expect<R: AsyncBufRead + Unpin>(reader: &mut R, code: &str) -> Result<(), String> {
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .await
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("the server closed the connection".to_owned());
        }
        if !line.starts_with(code) {
            return Err(format!("the server answered {}", line.trim()));
        }
        if line.as_bytes().get(3) != Some(&b'-') {
            return Ok(());
        }
    }
}

async fn starttls(stream: &mut TcpStream) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    expect(&mut reader, "220").await?;
    for (command, code) in [
        (&b"EHLO astrum\r\n"[..], "250"),
        (&b"STARTTLS\r\n"[..], "220"),
    ] {
        reader
            .get_mut()
            .write_all(command)
            .await
            .map_err(|error| error.to_string())?;
        expect(&mut reader, code).await?;
    }
    Ok(())
}

pub async fn fetch(
    guard: &Guard,
    host: &str,
    port: u16,
    protocol_hint: Option<&str>,
) -> Result<Vec<CertificateDer<'static>>, FetchError> {
    let addresses = resolve_guarded(guard, host, port).await?;
    let smtp = match protocol_hint.map(str::to_ascii_lowercase).as_deref() {
        Some("smtp") => true,
        Some(_) => false,
        None => port == 25 || port == 587,
    };
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let capture = Arc::new(Capture {
        chain: Mutex::new(Vec::new()),
        schemes: provider
            .signature_verification_algorithms
            .supported_schemes(),
    });
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| FetchError::Failed(error.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(capture.clone())
        .with_no_client_auth();
    let name = ServerName::try_from(
        host.trim_start_matches('[')
            .trim_end_matches(']')
            .to_owned(),
    )
    .map_err(|_| FetchError::Failed(format!("{host} is not a valid server name")))?;
    let attempt = async {
        let mut stream = TcpStream::connect(&addresses[..])
            .await
            .map_err(|error| format!("cannot connect to {host}:{port}: {error}"))?;
        if smtp {
            starttls(&mut stream).await?;
        }
        TlsConnector::from(Arc::new(config))
            .connect(name, stream)
            .await
            .map_err(|error| format!("the TLS handshake with {host}:{port} failed: {error}"))?;
        Ok::<_, String>(())
    };
    let outcome = tokio::time::timeout(FETCH_TIMEOUT, attempt)
        .await
        .map_err(|_| FetchError::Failed(format!("{host}:{port} did not answer in time")))?;
    let chain = std::mem::take(
        &mut *capture
            .chain
            .lock()
            .expect("the capture lock is never poisoned"),
    );
    if chain.is_empty() {
        return Err(FetchError::Failed(outcome.err().unwrap_or_else(|| {
            format!("{host}:{port} presented no certificate")
        })));
    }
    Ok(chain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated() -> (String, Vec<u8>) {
        let mut params = rcgen::CertificateParams::new(vec!["localhost".to_owned()]).unwrap();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "outbound-test.local");
        params
            .distinguished_name
            .push(rcgen::DnType::OrganizationName, "Outbound Test Org");
        let key = rcgen::KeyPair::generate().unwrap();
        let certificate = params.self_signed(&key).unwrap();
        (certificate.pem(), certificate.der().to_vec())
    }

    #[test]
    fn certificates_are_described_like_the_reference() {
        let (pem, der) = generated();
        let described = describe(&der_from_pem(&pem).unwrap()).unwrap();
        assert_eq!(
            described.subject_common_name.as_deref(),
            Some("outbound-test.local")
        );
        assert_eq!(
            described.issuer_organization.as_deref(),
            Some("Outbound Test Org")
        );
        assert_eq!(described.subject_organizational_unit, None);
        assert_eq!(described.id, described.fingerprint);
        assert_eq!(described.fingerprint.len(), 59);
        assert_eq!(described.fingerprint, fingerprint(&der));
        assert!(described.expires_on > described.issued_on);
        assert_eq!(der_from_pem(&described.pem).unwrap(), der);
        let quoted = serde_json::to_string(&pem).unwrap();
        assert_eq!(der_from_pem(&quoted).unwrap(), der);
    }

    #[test]
    fn other_text_is_refused() {
        assert!(matches!(
            der_from_pem("hello"),
            Err(CertificateError::NotPem)
        ));
        assert!(
            der_from_pem("-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n").is_err()
        );
    }
}
