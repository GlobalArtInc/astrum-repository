use std::sync::{Arc, OnceLock};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ldap3_proto::LdapCodec;
use ldap3_proto::proto::{
    LdapBindCred, LdapBindRequest, LdapDerefAliases, LdapExtendedRequest, LdapFilter, LdapMsg,
    LdapOp, LdapResult, LdapResultCode, LdapSearchRequest, LdapSearchScope, SaslCredentials,
};
use rustls::pki_types::{CertificateDer, ServerName};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::codec::Framed;

use crate::tls;

const STARTTLS: &str = "1.3.6.1.4.1.1466.20037";
const MAX_MESSAGE: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Plain,
    Ldaps,
    StartTls,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum ClientError {
    #[error("{0}")]
    Connection(String),
    #[error("{message}")]
    Result {
        code: LdapResultCode,
        message: String,
    },
}

impl ClientError {
    pub fn code(&self) -> Option<&LdapResultCode> {
        match self {
            ClientError::Result { code, .. } => Some(code),
            ClientError::Connection(_) => None,
        }
    }
}

#[derive(Clone)]
pub struct TlsClient {
    roots: Vec<CertificateDer<'static>>,
    config: Result<Arc<rustls::ClientConfig>, String>,
}

impl std::fmt::Debug for TlsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TlsClient")
    }
}

impl TlsClient {
    pub fn system() -> Self {
        static SYSTEM: OnceLock<TlsClient> = OnceLock::new();
        SYSTEM.get_or_init(|| Self::with_roots(Vec::new())).clone()
    }

    pub fn with_roots(extra: Vec<CertificateDer<'static>>) -> Self {
        let config = tls::client_config(&extra, true)
            .map(Arc::new)
            .map_err(|error| format!("no certificate can be trusted: {error}"));
        Self {
            roots: extra,
            config,
        }
    }

    pub fn trusting(&self, more: &[CertificateDer<'static>]) -> Self {
        if more.is_empty() {
            return self.clone();
        }
        Self::with_roots(self.roots.iter().chain(more).cloned().collect())
    }

    async fn wrap<S>(&self, host: &str, stream: S) -> Result<Box<dyn Io>, String>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let config = self.config.clone()?;
        let name = ServerName::try_from(host.to_owned())
            .map_err(|_| format!("{host} is not a valid TLS server name"))?;
        let stream = TlsConnector::from(config)
            .connect(name, stream)
            .await
            .map_err(|error| error.to_string())?;
        Ok(Box::new(stream))
    }
}

pub trait Io: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub dn: String,
    pub attributes: Vec<(String, Vec<Vec<u8>>)>,
}

impl Entry {
    pub fn values(&self, name: &str) -> Vec<&[u8]> {
        self.attributes
            .iter()
            .filter(|(attribute, _)| same_attribute(attribute, name))
            .flat_map(|(_, values)| values.iter().map(Vec::as_slice))
            .collect()
    }

    pub fn texts(&self, name: &str) -> Vec<String> {
        self.values(name)
            .into_iter()
            .filter_map(|value| std::str::from_utf8(value).ok())
            .map(str::to_owned)
            .collect()
    }

    pub fn text(&self, name: &str) -> Option<String> {
        self.texts(name).into_iter().next()
    }

    pub fn first_bytes(&self, name: &str) -> Option<Vec<u8>> {
        self.values(name).first().map(|value| value.to_vec())
    }
}

fn same_attribute(attribute: &str, wanted: &str) -> bool {
    let base = attribute.split(';').next().unwrap_or(attribute);
    base.eq_ignore_ascii_case(wanted)
}

pub struct Search<'a> {
    pub base: &'a str,
    pub subtree: Scope,
    pub filter: LdapFilter,
    pub attributes: Vec<String>,
    pub limit: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Base,
    One,
    Subtree,
}

pub struct Connection {
    framed: Framed<Box<dyn Io>, LdapCodec>,
    address: String,
    next_id: i32,
    timeout: Duration,
}

fn codec() -> LdapCodec {
    LdapCodec::new(Some(MAX_MESSAGE), None)
}

pub fn address(host: &str, port: u16) -> String {
    match host.contains(':') && !host.starts_with('[') {
        true => format!("[{host}]:{port}"),
        false => format!("{host}:{port}"),
    }
}

impl Connection {
    pub async fn open(
        host: &str,
        port: u16,
        transport: Transport,
        tls: &TlsClient,
        timeout: Duration,
    ) -> Result<Self, ClientError> {
        let address = address(host, port);
        let tcp = tokio::time::timeout(timeout, TcpStream::connect((host, port)))
            .await
            .map_err(|_| {
                ClientError::Connection(format!(
                    "could not connect to {address} within {} seconds",
                    timeout.as_secs()
                ))
            })?
            .map_err(|error| {
                ClientError::Connection(format!("could not connect to {address}: {error}"))
            })?;
        let _ = tcp.set_nodelay(true);
        let handshake = |reason: String| {
            ClientError::Connection(format!("the TLS handshake with {address} failed: {reason}"))
        };
        let io: Box<dyn Io> = match transport {
            Transport::Plain => Box::new(tcp),
            Transport::Ldaps => tokio::time::timeout(timeout, tls.wrap(host, tcp))
                .await
                .map_err(|_| handshake("it took too long".to_owned()))?
                .map_err(handshake)?,
            Transport::StartTls => {
                let mut plain = Framed::new(tcp, codec());
                let request = LdapMsg::new(
                    1,
                    LdapOp::ExtendedRequest(LdapExtendedRequest {
                        name: STARTTLS.to_owned(),
                        value: None,
                    }),
                );
                let reply = tokio::time::timeout(timeout, async {
                    plain.send(request).await?;
                    plain.next().await.transpose()
                })
                .await
                .map_err(|_| {
                    ClientError::Connection(format!(
                        "{address} did not answer the StartTLS request"
                    ))
                })?
                .map_err(|error| {
                    ClientError::Connection(format!("the connection to {address} broke: {error}"))
                })?;
                match reply.map(|message| message.op) {
                    Some(LdapOp::ExtendedResponse(response))
                        if response.res.code == LdapResultCode::Success => {}
                    Some(LdapOp::ExtendedResponse(response)) => {
                        return Err(ClientError::Connection(format!(
                            "{address} refused StartTLS: {}",
                            describe(&response.res)
                        )));
                    }
                    _ => {
                        return Err(ClientError::Connection(format!(
                            "{address} did not accept StartTLS"
                        )));
                    }
                }
                let parts = plain.into_parts();
                if !parts.read_buf.is_empty() {
                    return Err(ClientError::Connection(format!(
                        "{address} sent data before the TLS handshake"
                    )));
                }
                tokio::time::timeout(timeout, tls.wrap(host, parts.io))
                    .await
                    .map_err(|_| handshake("it took too long".to_owned()))?
                    .map_err(handshake)?
            }
        };
        Ok(Self {
            framed: Framed::new(io, codec()),
            address,
            next_id: 2,
            timeout,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    async fn send(&mut self, op: LdapOp) -> Result<i32, ClientError> {
        let id = self.next_id;
        self.next_id += 1;
        tokio::time::timeout(self.timeout, self.framed.send(LdapMsg::new(id, op)))
            .await
            .map_err(|_| self.timed_out())?
            .map_err(|error| self.broken(&error))?;
        Ok(id)
    }

    async fn receive(&mut self, id: i32) -> Result<LdapOp, ClientError> {
        loop {
            let message = tokio::time::timeout(self.timeout, self.framed.next())
                .await
                .map_err(|_| self.timed_out())?
                .ok_or_else(|| {
                    ClientError::Connection(format!("{} closed the connection", self.address))
                })?
                .map_err(|error| self.broken(&error))?;
            if message.msgid == id {
                return Ok(message.op);
            }
            if message.msgid == 0 {
                return Err(ClientError::Connection(format!(
                    "{} ended the session",
                    self.address
                )));
            }
        }
    }

    fn timed_out(&self) -> ClientError {
        ClientError::Connection(format!(
            "{} did not answer within {} seconds",
            self.address,
            self.timeout.as_secs()
        ))
    }

    fn broken(&self, error: &std::io::Error) -> ClientError {
        ClientError::Connection(format!("the connection to {} broke: {error}", self.address))
    }

    pub async fn simple_bind(&mut self, dn: &str, password: &str) -> Result<(), ClientError> {
        let id = self
            .send(LdapOp::BindRequest(LdapBindRequest {
                dn: dn.to_owned(),
                cred: LdapBindCred::Simple(password.to_owned()),
            }))
            .await?;
        match self.receive(id).await? {
            LdapOp::BindResponse(response) if response.res.code == LdapResultCode::Success => {
                Ok(())
            }
            LdapOp::BindResponse(response) => Err(refused(&response.res)),
            _ => Err(self.unexpected()),
        }
    }

    pub async fn sasl_bind(
        &mut self,
        mechanism: &str,
        credentials: Vec<u8>,
    ) -> Result<(LdapResult, Option<Vec<u8>>), ClientError> {
        let id = self
            .send(LdapOp::BindRequest(LdapBindRequest {
                dn: String::new(),
                cred: LdapBindCred::SASL(SaslCredentials {
                    mechanism: mechanism.to_owned(),
                    credentials,
                }),
            }))
            .await?;
        match self.receive(id).await? {
            LdapOp::BindResponse(response) => Ok((response.res, response.saslcreds)),
            _ => Err(self.unexpected()),
        }
    }

    pub async fn search(&mut self, search: Search<'_>) -> Result<Vec<Entry>, ClientError> {
        let attributes = match search.attributes.is_empty() {
            true => vec!["1.1".to_owned()],
            false => search.attributes,
        };
        let id = self
            .send(LdapOp::SearchRequest(LdapSearchRequest {
                base: search.base.to_owned(),
                scope: match search.subtree {
                    Scope::Base => LdapSearchScope::Base,
                    Scope::One => LdapSearchScope::OneLevel,
                    Scope::Subtree => LdapSearchScope::Subtree,
                },
                aliases: LdapDerefAliases::Never,
                sizelimit: i32::try_from(search.limit).unwrap_or(i32::MAX),
                timelimit: i32::try_from(self.timeout.as_secs()).unwrap_or(i32::MAX),
                typesonly: false,
                filter: search.filter,
                attrs: attributes,
            }))
            .await?;
        let mut entries = Vec::new();
        loop {
            match self.receive(id).await? {
                LdapOp::SearchResultEntry(entry) => {
                    if search.limit == 0 || entries.len() < search.limit {
                        entries.push(Entry {
                            dn: entry.dn,
                            attributes: entry
                                .attributes
                                .into_iter()
                                .map(|attribute| (attribute.atype, attribute.vals))
                                .collect(),
                        });
                    }
                }
                LdapOp::SearchResultReference(_) => {}
                LdapOp::SearchResultDone(result) => {
                    return match result.code {
                        LdapResultCode::Success | LdapResultCode::SizeLimitExceeded => Ok(entries),
                        _ => Err(refused(&result)),
                    };
                }
                _ => return Err(self.unexpected()),
            }
        }
    }

    fn unexpected(&self) -> ClientError {
        ClientError::Connection(format!("{} sent an unexpected answer", self.address))
    }

    pub async fn close(mut self) {
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            self.framed
                .send(LdapMsg::new(self.next_id, LdapOp::UnbindRequest)),
        )
        .await;
    }
}

pub fn refused(result: &LdapResult) -> ClientError {
    ClientError::Result {
        code: result.code.clone(),
        message: describe(result),
    }
}

pub fn describe(result: &LdapResult) -> String {
    let code = result.code.clone() as i64;
    let name = format!("{:?}", result.code);
    match result.message.trim().is_empty() {
        true => format!("{name} ({code})"),
        false => format!("{name} ({code}): {}", result.message.trim()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attributes_match_without_case_or_options() {
        let entry = Entry {
            dn: "uid=a,dc=x".to_owned(),
            attributes: vec![
                ("CN".to_owned(), vec![b"Alice".to_vec()]),
                (
                    "memberOf;range=0-1".to_owned(),
                    vec![b"cn=a".to_vec(), b"cn=b".to_vec()],
                ),
            ],
        };
        assert_eq!(entry.text("cn"), Some("Alice".to_owned()));
        assert_eq!(entry.texts("memberof"), ["cn=a", "cn=b"]);
        assert_eq!(entry.text("mail"), None);
    }

    #[test]
    fn addresses_bracket_ipv6() {
        assert_eq!(address("::1", 389), "[::1]:389");
        assert_eq!(address("ldap.example.org", 636), "ldap.example.org:636");
    }

    #[test]
    fn results_describe_themselves() {
        let result = LdapResult {
            code: LdapResultCode::InvalidCredentials,
            matcheddn: String::new(),
            message: " bad ".to_owned(),
            referral: Vec::new(),
        };
        assert_eq!(describe(&result), "InvalidCredentials (49): bad");
    }
}
