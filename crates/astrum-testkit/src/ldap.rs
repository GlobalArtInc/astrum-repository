use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use ldap3_lber::common::TagClass;
use ldap3_lber::structure::{PL, StructureTag};
use ldap3_proto::LdapCodec;
use ldap3_proto::proto::{
    LdapBindCred, LdapBindResponse, LdapExtendedResponse, LdapFilter, LdapMsg, LdapOp,
    LdapPartialAttribute, LdapResult, LdapResultCode, LdapSearchRequest, LdapSearchResultEntry,
    LdapSearchScope,
};
use md5::{Digest, Md5};
use rand::RngCore;
use rustls::pki_types::CertificateDer;
use sha1::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::Framed;

use crate::tls::TestCertificates;

pub const BASE: &str = "dc=example,dc=com";
pub const SYSTEM_DN: &str = "cn=svc,dc=example,dc=com";
pub const SYSTEM_USER: &str = "svc";
pub const SYSTEM_PASSWORD: &str = "svc-secret";
pub const SASL_REALM: &str = "example.com";
const STARTTLS: &str = "1.3.6.1.4.1.1466.20037";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub dn: String,
    pub attributes: Vec<(String, Vec<String>)>,
}

impl Entry {
    pub fn new(dn: &str, attributes: &[(&str, &[&str])]) -> Self {
        Self {
            dn: dn.to_owned(),
            attributes: attributes
                .iter()
                .map(|(name, values)| {
                    (
                        (*name).to_owned(),
                        values.iter().map(|value| (*value).to_owned()).collect(),
                    )
                })
                .collect(),
        }
    }

    fn values(&self, name: &str) -> Vec<&str> {
        self.attributes
            .iter()
            .filter(|(attribute, _)| attribute.eq_ignore_ascii_case(name))
            .flat_map(|(_, values)| values.iter().map(String::as_str))
            .collect()
    }
}

pub fn ssha(password: &str) -> String {
    let salt = b"astrum-salt";
    let mut digest = Sha1::new();
    sha1::Digest::update(&mut digest, password.as_bytes());
    sha1::Digest::update(&mut digest, salt);
    let mut bytes = digest.finalize().to_vec();
    bytes.extend_from_slice(salt);
    format!("{{SSHA}}{}", STANDARD.encode(bytes))
}

fn person(uid: &str, name: &str, mail: &str, password: &str, ou: &str, groups: &[&str]) -> Entry {
    let dn = format!("uid={uid},{ou},{BASE}");
    let member_of: Vec<String> = groups
        .iter()
        .map(|group| format!("cn={group},ou=groups,{BASE}"))
        .collect();
    let mut entry = Entry::new(
        &dn,
        &[
            ("objectClass", &["top", "person", "inetOrgPerson"]),
            ("uid", &[uid]),
            ("cn", &[name]),
            ("sn", &[name.split(' ').next_back().unwrap_or(name)]),
            ("mail", &[mail]),
            ("userPassword", &[password]),
        ],
    );
    if !member_of.is_empty() {
        entry.attributes.push(("memberOf".to_owned(), member_of));
    }
    entry
}

pub fn fixture() -> Vec<Entry> {
    vec![
        Entry::new(
            BASE,
            &[("objectClass", &["top", "domain"]), ("dc", &["example"])],
        ),
        Entry::new(
            &format!("ou=people,{BASE}"),
            &[
                ("objectClass", &["organizationalUnit"]),
                ("ou", &["people"]),
            ],
        ),
        Entry::new(
            &format!("ou=contractors,ou=people,{BASE}"),
            &[
                ("objectClass", &["organizationalUnit"]),
                ("ou", &["contractors"]),
            ],
        ),
        Entry::new(
            &format!("ou=groups,{BASE}"),
            &[
                ("objectClass", &["organizationalUnit"]),
                ("ou", &["groups"]),
            ],
        ),
        person(
            "alice",
            "Alice Smith",
            "alice@example.com",
            "alice-secret",
            "ou=people",
            &["developers", "admins"],
        ),
        person(
            "bob",
            "Bob Jones",
            "bob@example.com",
            &ssha("bob-secret"),
            "ou=people",
            &["developers"],
        ),
        person(
            "carol",
            "Carol White",
            "carol@other.org",
            "carol-secret",
            "ou=people",
            &[],
        ),
        person(
            "dave",
            "Dave Brown",
            "dave@example.com",
            "dave-secret",
            "ou=contractors,ou=people",
            &[],
        ),
        Entry::new(
            &format!("cn=developers,ou=groups,{BASE}"),
            &[
                ("objectClass", &["groupOfUniqueNames"]),
                ("cn", &["developers"]),
                (
                    "uniqueMember",
                    &[
                        "uid=alice,ou=people,dc=example,dc=com",
                        "uid=bob,ou=people,dc=example,dc=com",
                    ],
                ),
            ],
        ),
        Entry::new(
            &format!("cn=admins,ou=groups,{BASE}"),
            &[
                ("objectClass", &["groupOfUniqueNames"]),
                ("cn", &["admins"]),
                ("uniqueMember", &["uid=alice,ou=people,dc=example,dc=com"]),
            ],
        ),
    ]
}

#[derive(Default)]
struct Stats {
    connections: AtomicUsize,
    binds: AtomicUsize,
    searches: AtomicUsize,
}

struct Shared {
    entries: Mutex<Vec<Entry>>,
    stats: Stats,
    tls: TlsAcceptor,
}

pub struct LdapTestServer {
    pub port: u16,
    pub ldaps_port: u16,
    pub certificates: TestCertificates,
    shared: Arc<Shared>,
    tasks: Arc<Mutex<JoinSet<()>>>,
}

impl LdapTestServer {
    pub async fn start() -> Self {
        Self::start_with(fixture()).await
    }

    pub async fn start_with(entries: Vec<Entry>) -> Self {
        Self::start_on(entries, 0, 0).await
    }

    pub async fn start_on(entries: Vec<Entry>, port: u16, ldaps_port: u16) -> Self {
        let certificates = TestCertificates::generate(&["localhost", "127.0.0.1"]);
        let shared = Arc::new(Shared {
            entries: Mutex::new(entries),
            stats: Stats::default(),
            tls: TlsAcceptor::from(certificates.server_config()),
        });
        let plain = TcpListener::bind(("127.0.0.1", port))
            .await
            .expect("the test LDAP server can listen");
        let secure = TcpListener::bind(("127.0.0.1", ldaps_port))
            .await
            .expect("the test LDAP server can listen");
        let port = plain.local_addr().expect("a bound port").port();
        let ldaps_port = secure.local_addr().expect("a bound port").port();
        let tasks = Arc::new(Mutex::new(JoinSet::new()));
        {
            let mut set = tasks.lock().expect("the task lock is never poisoned");
            set.spawn(accept(plain, shared.clone(), tasks.clone(), false));
            set.spawn(accept(secure, shared.clone(), tasks.clone(), true));
        }
        Self {
            port,
            ldaps_port,
            certificates,
            shared,
            tasks,
        }
    }

    pub fn ca(&self) -> CertificateDer<'static> {
        self.certificates.ca.clone()
    }

    pub fn stop(&self) {
        self.tasks
            .lock()
            .expect("the task lock is never poisoned")
            .abort_all();
    }

    pub fn connections(&self) -> usize {
        self.shared.stats.connections.load(Ordering::SeqCst)
    }

    pub fn binds(&self) -> usize {
        self.shared.stats.binds.load(Ordering::SeqCst)
    }

    pub fn searches(&self) -> usize {
        self.shared.stats.searches.load(Ordering::SeqCst)
    }

    pub fn add(&self, entry: Entry) {
        self.shared
            .entries
            .lock()
            .expect("the entry lock is never poisoned")
            .push(entry);
    }

    pub fn remove(&self, dn: &str) {
        self.shared
            .entries
            .lock()
            .expect("the entry lock is never poisoned")
            .retain(|entry| !same_dn(&entry.dn, dn));
    }

    pub fn set_attribute(&self, dn: &str, name: &str, values: &[&str]) {
        let mut entries = self
            .shared
            .entries
            .lock()
            .expect("the entry lock is never poisoned");
        if let Some(entry) = entries.iter_mut().find(|entry| same_dn(&entry.dn, dn)) {
            entry
                .attributes
                .retain(|(attribute, _)| !attribute.eq_ignore_ascii_case(name));
            entry.attributes.push((
                name.to_owned(),
                values.iter().map(|value| (*value).to_owned()).collect(),
            ));
        }
    }
}

impl Drop for LdapTestServer {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn accept(
    listener: TcpListener,
    shared: Arc<Shared>,
    tasks: Arc<Mutex<JoinSet<()>>>,
    tls: bool,
) {
    while let Ok((stream, _)) = listener.accept().await {
        shared.stats.connections.fetch_add(1, Ordering::SeqCst);
        let shared = shared.clone();
        tasks
            .lock()
            .expect("the task lock is never poisoned")
            .spawn(connection(stream, shared, tls));
    }
}

async fn connection(stream: TcpStream, shared: Arc<Shared>, tls: bool) {
    if tls {
        if let Ok(stream) = shared.tls.accept(stream).await {
            let mut framed = Framed::new(stream, LdapCodec::default());
            let _ = session(&mut framed, &shared, true).await;
        }
        return;
    }
    let mut framed = Framed::new(stream, LdapCodec::default());
    if session(&mut framed, &shared, false).await == Next::StartTls {
        let parts = framed.into_parts();
        if let Ok(stream) = shared.tls.accept(parts.io).await {
            let mut framed = Framed::new(stream, LdapCodec::default());
            let _ = session(&mut framed, &shared, true).await;
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Next {
    Close,
    StartTls,
}

enum Sasl {
    None,
    Cram(String),
    Digest(String),
}

async fn session<S>(framed: &mut Framed<S, LdapCodec>, shared: &Shared, secure: bool) -> Next
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut sasl = Sasl::None;
    while let Some(Ok(message)) = framed.next().await {
        let id = message.msgid;
        match message.op {
            LdapOp::BindRequest(request) => {
                shared.stats.binds.fetch_add(1, Ordering::SeqCst);
                let (code, creds) = match request.cred {
                    LdapBindCred::Simple(password) => {
                        sasl = Sasl::None;
                        (simple_bind(shared, &request.dn, &password), None)
                    }
                    LdapBindCred::SASL(credentials) => sasl_step(
                        shared,
                        &mut sasl,
                        &credentials.mechanism,
                        &credentials.credentials,
                    ),
                };
                let reply = LdapOp::BindResponse(LdapBindResponse {
                    res: result(code),
                    saslcreds: creds,
                });
                if send_bind_response(framed, LdapMsg::new(id, reply))
                    .await
                    .is_err()
                {
                    return Next::Close;
                }
            }
            LdapOp::ExtendedRequest(request) if request.name == STARTTLS && !secure => {
                let reply = LdapOp::ExtendedResponse(LdapExtendedResponse {
                    res: result(LdapResultCode::Success),
                    name: None,
                    value: None,
                });
                if framed.send(LdapMsg::new(id, reply)).await.is_err() {
                    return Next::Close;
                }
                return Next::StartTls;
            }
            LdapOp::ExtendedRequest(_) => {
                let reply = LdapOp::ExtendedResponse(LdapExtendedResponse {
                    res: result(LdapResultCode::UnwillingToPerform),
                    name: None,
                    value: None,
                });
                if framed.send(LdapMsg::new(id, reply)).await.is_err() {
                    return Next::Close;
                }
            }
            LdapOp::SearchRequest(request) => {
                shared.stats.searches.fetch_add(1, Ordering::SeqCst);
                let (entries, code) = search(shared, &request);
                for entry in entries {
                    if framed
                        .send(LdapMsg::new(id, LdapOp::SearchResultEntry(entry)))
                        .await
                        .is_err()
                    {
                        return Next::Close;
                    }
                }
                if framed
                    .send(LdapMsg::new(id, LdapOp::SearchResultDone(result(code))))
                    .await
                    .is_err()
                {
                    return Next::Close;
                }
            }
            LdapOp::UnbindRequest => return Next::Close,
            _ => return Next::Close,
        }
    }
    Next::Close
}

async fn send_bind_response<S>(
    framed: &mut Framed<S, LdapCodec>,
    message: LdapMsg,
) -> std::io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut tag: StructureTag = message.into();
    if let PL::C(parts) = &mut tag.payload
        && let Some(op) = parts.get_mut(1)
        && let PL::C(fields) = &mut op.payload
        && fields.len() == 4
    {
        fields[3].class = TagClass::Context;
        fields[3].id = 7;
    }
    let mut buffer = BytesMut::new();
    ldap3_lber::write::encode_into(&mut buffer, tag)?;
    framed.flush().await?;
    framed.get_mut().write_all(&buffer).await?;
    framed.get_mut().flush().await
}

fn result(code: LdapResultCode) -> LdapResult {
    LdapResult {
        code,
        matcheddn: String::new(),
        message: String::new(),
        referral: Vec::new(),
    }
}

fn password_of(shared: &Shared, dn: &str) -> Option<String> {
    if same_dn(dn, SYSTEM_DN) {
        return Some(SYSTEM_PASSWORD.to_owned());
    }
    let entries = shared
        .entries
        .lock()
        .expect("the entry lock is never poisoned");
    entries
        .iter()
        .find(|entry| same_dn(&entry.dn, dn))
        .and_then(|entry| {
            entry
                .values("userPassword")
                .first()
                .map(|value| (*value).to_owned())
        })
}

fn password_of_user(shared: &Shared, user: &str) -> Option<String> {
    if user == SYSTEM_USER {
        return Some(SYSTEM_PASSWORD.to_owned());
    }
    let entries = shared
        .entries
        .lock()
        .expect("the entry lock is never poisoned");
    entries
        .iter()
        .find(|entry| entry.values("uid").contains(&user))
        .and_then(|entry| {
            entry
                .values("userPassword")
                .first()
                .map(|value| (*value).to_owned())
        })
}

fn simple_bind(shared: &Shared, dn: &str, password: &str) -> LdapResultCode {
    if dn.is_empty() && password.is_empty() {
        return LdapResultCode::Success;
    }
    if password.is_empty() {
        return LdapResultCode::UnwillingToPerform;
    }
    match password_of(shared, dn) {
        Some(stored) if stored == password || stored == ssha(password) => LdapResultCode::Success,
        _ => LdapResultCode::InvalidCredentials,
    }
}

fn nonce() -> String {
    let mut bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut bytes);
    hex(&bytes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sasl_step(
    shared: &Shared,
    state: &mut Sasl,
    mechanism: &str,
    credentials: &[u8],
) -> (LdapResultCode, Option<Vec<u8>>) {
    let current = std::mem::replace(state, Sasl::None);
    match (mechanism, current) {
        ("CRAM-MD5", Sasl::Cram(challenge)) if !credentials.is_empty() => {
            let text = String::from_utf8_lossy(credentials).into_owned();
            let Some((user, digest)) = text.rsplit_once(' ') else {
                return (LdapResultCode::InvalidCredentials, None);
            };
            let expected = password_of_user(shared, user).map(|password| {
                let mut mac = Hmac::<Md5>::new_from_slice(password.as_bytes())
                    .expect("HMAC accepts keys of any length");
                mac.update(challenge.as_bytes());
                hex(&mac.finalize().into_bytes())
            });
            match expected {
                Some(expected) if expected == digest => (LdapResultCode::Success, None),
                _ => (LdapResultCode::InvalidCredentials, None),
            }
        }
        ("CRAM-MD5", _) => {
            let challenge = format!("<{}@astrum.test>", nonce());
            *state = Sasl::Cram(challenge.clone());
            (
                LdapResultCode::SaslBindInProgress,
                Some(challenge.into_bytes()),
            )
        }
        ("DIGEST-MD5", Sasl::Digest(nonce)) if !credentials.is_empty() => {
            digest_check(shared, &nonce, credentials)
        }
        ("DIGEST-MD5", _) => {
            let nonce = nonce();
            *state = Sasl::Digest(nonce.clone());
            let challenge = format!(
                "realm=\"{SASL_REALM}\",nonce=\"{nonce}\",qop=\"auth\",charset=utf-8,algorithm=md5-sess"
            );
            (
                LdapResultCode::SaslBindInProgress,
                Some(challenge.into_bytes()),
            )
        }
        _ => (LdapResultCode::AuthMethodNotSupported, None),
    }
}

fn digest_check(
    shared: &Shared,
    nonce: &str,
    credentials: &[u8],
) -> (LdapResultCode, Option<Vec<u8>>) {
    let text = String::from_utf8_lossy(credentials).into_owned();
    let fields = directives(&text);
    let get = |name: &str| {
        fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    };
    let (Some(user), Some(cnonce), Some(nc), Some(uri), Some(response)) = (
        get("username"),
        get("cnonce"),
        get("nc"),
        get("digest-uri"),
        get("response"),
    ) else {
        return (LdapResultCode::InvalidCredentials, None);
    };
    if get("nonce").as_deref() != Some(nonce) {
        return (LdapResultCode::InvalidCredentials, None);
    }
    let realm = get("realm").unwrap_or_default();
    let Some(password) = password_of_user(shared, &user) else {
        return (LdapResultCode::InvalidCredentials, None);
    };
    let inner = Md5::digest(format!("{user}:{realm}:{password}").as_bytes());
    let mut a1 = inner.to_vec();
    a1.extend_from_slice(format!(":{nonce}:{cnonce}").as_bytes());
    let ha1 = hex(&Md5::digest(&a1));
    let kd = |data: String| hex(&Md5::digest(format!("{ha1}:{data}").as_bytes()));
    let expected = kd(format!(
        "{nonce}:{nc}:{cnonce}:auth:{}",
        hex(&Md5::digest(format!("AUTHENTICATE:{uri}").as_bytes()))
    ));
    if expected != response {
        return (LdapResultCode::InvalidCredentials, None);
    }
    let rspauth = kd(format!(
        "{nonce}:{nc}:{cnonce}:auth:{}",
        hex(&Md5::digest(format!(":{uri}").as_bytes()))
    ));
    (
        LdapResultCode::Success,
        Some(format!("rspauth={rspauth}").into_bytes()),
    )
}

fn directives(text: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        rest = rest.trim_start_matches([',', ' ']);
        let Some((name, after)) = rest.split_once('=') else {
            break;
        };
        let (value, remaining) = match after.strip_prefix('"') {
            Some(quoted) => {
                let end = quoted.find('"').unwrap_or(quoted.len());
                (
                    quoted[..end].to_owned(),
                    quoted.get(end + 1..).unwrap_or(""),
                )
            }
            None => {
                let end = after.find(',').unwrap_or(after.len());
                (after[..end].to_owned(), &after[end..])
            }
        };
        found.push((name.trim().to_owned(), value));
        rest = remaining;
    }
    found
}

fn normalize_dn(dn: &str) -> String {
    dn.split(',')
        .map(|part| part.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join(",")
}

fn same_dn(left: &str, right: &str) -> bool {
    normalize_dn(left) == normalize_dn(right)
}

fn in_scope(dn: &str, base: &str, scope: &LdapSearchScope) -> bool {
    let dn = normalize_dn(dn);
    let base = normalize_dn(base);
    match scope {
        LdapSearchScope::Base => dn == base,
        LdapSearchScope::OneLevel => dn.split_once(',').is_some_and(|(_, parent)| parent == base),
        LdapSearchScope::Subtree | LdapSearchScope::Children => {
            dn == base || dn.ends_with(&format!(",{base}"))
        }
    }
}

fn matches(entry: &Entry, filter: &LdapFilter) -> bool {
    match filter {
        LdapFilter::And(parts) => parts.iter().all(|part| matches(entry, part)),
        LdapFilter::Or(parts) => parts.iter().any(|part| matches(entry, part)),
        LdapFilter::Not(inner) => !matches(entry, inner),
        LdapFilter::Equality(attribute, value) | LdapFilter::Approx(attribute, value) => entry
            .values(attribute)
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(value)),
        LdapFilter::Present(attribute) => {
            attribute.eq_ignore_ascii_case("objectClass") || !entry.values(attribute).is_empty()
        }
        LdapFilter::Substring(attribute, parts) => entry.values(attribute).iter().any(|value| {
            let value = value.to_lowercase();
            let mut rest = value.as_str();
            if let Some(initial) = &parts.initial {
                match rest.strip_prefix(&initial.to_lowercase()) {
                    Some(after) => rest = after,
                    None => return false,
                }
            }
            for any in &parts.any {
                match rest.find(&any.to_lowercase()) {
                    Some(at) => rest = &rest[at + any.len()..],
                    None => return false,
                }
            }
            parts
                .final_
                .as_ref()
                .is_none_or(|last| rest.ends_with(&last.to_lowercase()))
        }),
        LdapFilter::GreaterOrEqual(attribute, value) => entry
            .values(attribute)
            .iter()
            .any(|candidate| *candidate >= value.as_str()),
        LdapFilter::LessOrEqual(attribute, value) => entry
            .values(attribute)
            .iter()
            .any(|candidate| *candidate <= value.as_str()),
        LdapFilter::Extensible(_) => false,
    }
}

fn search(
    shared: &Shared,
    request: &LdapSearchRequest,
) -> (Vec<LdapSearchResultEntry>, LdapResultCode) {
    let entries = shared
        .entries
        .lock()
        .expect("the entry lock is never poisoned");
    if !entries
        .iter()
        .any(|entry| same_dn(&entry.dn, &request.base))
    {
        return (Vec::new(), LdapResultCode::NoSuchObject);
    }
    let limit = usize::try_from(request.sizelimit).unwrap_or_default();
    let wanted: Vec<String> = request
        .attrs
        .iter()
        .map(|name| name.to_lowercase())
        .collect();
    let all = wanted.is_empty() || wanted.iter().any(|name| name == "*");
    let none = wanted.len() == 1 && wanted[0] == "1.1";
    let mut found = Vec::new();
    for entry in entries
        .iter()
        .filter(|entry| in_scope(&entry.dn, &request.base, &request.scope))
        .filter(|entry| matches(entry, &request.filter))
    {
        if limit > 0 && found.len() == limit {
            return (found, LdapResultCode::SizeLimitExceeded);
        }
        let attributes = entry
            .attributes
            .iter()
            .filter(|(name, _)| !none && (all || wanted.contains(&name.to_lowercase())))
            .map(|(name, values)| LdapPartialAttribute {
                atype: name.clone(),
                vals: values
                    .iter()
                    .map(|value| value.as_bytes().to_vec())
                    .collect(),
            })
            .collect();
        found.push(LdapSearchResultEntry {
            dn: entry.dn.clone(),
            attributes,
        });
    }
    (found, LdapResultCode::Success)
}
