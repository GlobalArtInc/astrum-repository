use std::sync::{Arc, OnceLock};

use rustls::client::WebPkiServerVerifier;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, RootCertStore, SignatureScheme};

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
            Err(_)
                if self
                    .pinned
                    .iter()
                    .any(|pinned| pinned.as_ref() == end_entity.as_ref()) =>
            {
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
