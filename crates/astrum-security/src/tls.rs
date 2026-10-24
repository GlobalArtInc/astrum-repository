use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::{WebPkiServerVerifier, verify_server_name};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::server::ParsedCertificate;
use rustls::{CertificateError, DigitallySignedStruct, RootCertStore, SignatureScheme};

pub type Certificates<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<CertificateDer<'static>>, String>> + Send + 'a>>;

pub trait TrustStore: Send + Sync {
    fn certificates(&self) -> Certificates<'_>;
}

fn native() -> &'static Vec<CertificateDer<'static>> {
    static NATIVE: OnceLock<Vec<CertificateDer<'static>>> = OnceLock::new();
    NATIVE.get_or_init(|| {
        let loaded = rustls_native_certs::load_native_certs();
        for error in &loaded.errors {
            tracing::debug!(%error, "a system certificate could not be loaded");
        }
        loaded.certs
    })
}

#[derive(Debug)]
struct Verifier {
    inner: Arc<WebPkiServerVerifier>,
    pinned: Vec<CertificateDer<'static>>,
    check_identity: bool,
}

impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        ) {
            Ok(verified) => Ok(verified),
            Err(rustls::Error::InvalidCertificate(
                CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. },
            )) if !self.check_identity => Ok(ServerCertVerified::assertion()),
            Err(error)
                if self
                    .pinned
                    .iter()
                    .any(|pinned| pinned.as_ref() == end_entity.as_ref()) =>
            {
                if !self.check_identity {
                    return Ok(ServerCertVerified::assertion());
                }
                let parsed = ParsedCertificate::try_from(end_entity).map_err(|_| error)?;
                verify_server_name(&parsed, server_name)?;
                Ok(ServerCertVerified::assertion())
            }
            Err(error) => Err(error),
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

pub fn client_config(
    extra: &[CertificateDer<'static>],
    check_identity: bool,
) -> Result<rustls::ClientConfig, rustls::Error> {
    let mut roots = RootCertStore::empty();
    roots.add_parsable_certificates(native().iter().cloned());
    roots.add_parsable_certificates(extra.iter().cloned());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let inner = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider.clone())
        .build()
        .map_err(|error| rustls::Error::General(error.to_string()))?;
    Ok(rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Verifier {
            inner,
            pinned: extra.to_vec(),
            check_identity,
        }))
        .with_no_client_auth())
}

pub fn http_client_config(
    extra: &[CertificateDer<'static>],
) -> Result<rustls::ClientConfig, rustls::Error> {
    let mut config = client_config(extra, true)?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

#[cfg(test)]
mod tests {
    use astrum_testkit::tls::TestCertificates;
    use tokio_rustls::{TlsAcceptor, TlsConnector};

    use super::*;

    async fn handshake(
        certificates: &TestCertificates,
        extra: &[CertificateDer<'static>],
        check_identity: bool,
        name: &str,
    ) -> bool {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let acceptor = TlsAcceptor::from(certificates.server_config());
        tokio::spawn(async move {
            let _ = acceptor.accept(server).await;
        });
        let config = client_config(extra, check_identity).unwrap();
        TlsConnector::from(Arc::new(config))
            .connect(ServerName::try_from(name.to_owned()).unwrap(), client)
            .await
            .is_ok()
    }

    #[tokio::test]
    async fn stored_certificates_are_trusted_and_names_are_checked_when_asked() {
        let certificates = TestCertificates::generate(&["localhost"]);
        let leaf = [certificates.server.clone()];
        let authority = [certificates.ca.clone()];
        assert!(!handshake(&certificates, &[], true, "localhost").await);
        assert!(handshake(&certificates, &leaf, true, "localhost").await);
        assert!(handshake(&certificates, &authority, true, "localhost").await);
        assert!(!handshake(&certificates, &leaf, true, "other.invalid").await);
        assert!(!handshake(&certificates, &authority, true, "other.invalid").await);
        assert!(handshake(&certificates, &leaf, false, "other.invalid").await);
        assert!(handshake(&certificates, &authority, false, "other.invalid").await);
    }
}
