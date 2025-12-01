use std::sync::Arc;

use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

pub struct TestCertificates {
    pub ca: CertificateDer<'static>,
    pub ca_pem: String,
    pub server: CertificateDer<'static>,
    key: Vec<u8>,
}

impl TestCertificates {
    pub fn generate(names: &[&str]) -> Self {
        let ca_key = KeyPair::generate().expect("a test key can be generated");
        let mut ca_params =
            CertificateParams::new(Vec::<String>::new()).expect("empty names are fine");
        ca_params
            .distinguished_name
            .push(DnType::CommonName, "Astrum test CA");
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca_cert = ca_params
            .self_signed(&ca_key)
            .expect("the test CA can be signed");
        let issuer = Issuer::new(ca_params, ca_key);
        let server_key = KeyPair::generate().expect("a test key can be generated");
        let mut server_params = CertificateParams::new(
            names
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
        )
        .expect("the test names are valid");
        server_params.distinguished_name.push(
            DnType::CommonName,
            names.first().copied().unwrap_or("localhost"),
        );
        let server_cert = server_params
            .signed_by(&server_key, &issuer)
            .expect("the test server certificate can be signed");
        Self {
            ca_pem: ca_cert.pem(),
            ca: ca_cert.der().clone(),
            server: server_cert.der().clone(),
            key: server_key.serialize_der(),
        }
    }

    pub fn server_config(&self) -> Arc<rustls::ServerConfig> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key.clone()));
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("the ring provider supports the default TLS versions")
            .with_no_client_auth()
            .with_single_cert(vec![self.server.clone(), self.ca.clone()], key)
            .expect("the test certificate and key belong together");
        Arc::new(config)
    }
}
