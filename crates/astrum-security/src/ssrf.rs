use std::error::Error;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use serde::{Deserialize, Serialize};

pub const SSRF_KEY: &str = "security.ssrf";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SsrfProtection {
    pub enabled: bool,
    #[serde(rename = "allowedIPs", default)]
    pub allowed_ips: Vec<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

impl Default for SsrfProtection {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_ips: Vec::new(),
            allowed_domains: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Network {
    address: IpAddr,
    prefix: u8,
}

impl Network {
    pub fn parse(value: &str) -> Option<Self> {
        let (address, prefix) = match value.split_once('/') {
            Some((address, prefix)) => (address, Some(prefix)),
            None => (value, None),
        };
        let address: IpAddr = address.parse().ok()?;
        let bits = match address {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        let prefix = match prefix {
            Some(prefix) if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) => {
                prefix.parse::<u8>().ok().filter(|prefix| *prefix <= bits)?
            }
            Some(_) => return None,
            None => bits,
        };
        Some(Self { address, prefix })
    }

    pub fn contains(&self, address: IpAddr) -> bool {
        let address = match address {
            IpAddr::V6(v6) => v6
                .to_ipv4_mapped()
                .map(IpAddr::V4)
                .unwrap_or(IpAddr::V6(v6)),
            v4 => v4,
        };
        match (self.address, address) {
            (IpAddr::V4(network), IpAddr::V4(candidate)) => {
                let mask = u32::MAX
                    .checked_shl(32 - u32::from(self.prefix))
                    .unwrap_or(0);
                u32::from(network) & mask == u32::from(candidate) & mask
            }
            (IpAddr::V6(network), IpAddr::V6(candidate)) => {
                let mask = u128::MAX
                    .checked_shl(128 - u32::from(self.prefix))
                    .unwrap_or(0);
                u128::from(network) & mask == u128::from(candidate) & mask
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}/{}", self.address, self.prefix)
    }
}

pub fn valid_domain(value: &str) -> bool {
    let name = value.strip_prefix("*.").unwrap_or(value);
    let name = name.strip_suffix('.').unwrap_or(name);
    !name.is_empty()
        && name.len() <= 253
        && name.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        })
}

#[derive(Clone, Debug)]
pub struct Guard {
    enabled: bool,
    networks: Vec<Network>,
    domains: Vec<String>,
}

impl Guard {
    pub fn new(settings: &SsrfProtection) -> Self {
        Self {
            enabled: settings.enabled,
            networks: settings
                .allowed_ips
                .iter()
                .filter_map(|value| Network::parse(value))
                .collect(),
            domains: settings
                .allowed_domains
                .iter()
                .map(|domain| domain.trim_end_matches('.').to_ascii_lowercase())
                .collect(),
        }
    }

    pub fn host_allowed(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        self.domains
            .iter()
            .any(|domain| match domain.strip_prefix("*.") {
                Some(parent) => host.ends_with(&format!(".{parent}")),
                None => *domain == host,
            })
    }

    pub fn ip_allowed(&self, ip: IpAddr, host_allowed: bool) -> bool {
        let ip = canonical(ip);
        if is_metadata(ip) {
            return false;
        }
        if !self.enabled || host_allowed {
            return true;
        }
        self.networks.iter().any(|network| network.contains(ip)) || !is_internal(ip)
    }

    pub fn remote_ip_allowed(&self, ip: IpAddr, host_allowed: bool, private_network: bool) -> bool {
        self.ip_allowed(ip, host_allowed) || (private_network && permitted_for_private_remote(ip))
    }
}

pub fn permitted_for_private_remote(ip: IpAddr) -> bool {
    match canonical(ip) {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            v4.is_private() || (a == 100 && (64..128).contains(&b))
        }
        IpAddr::V6(v6) => (v6.segments()[0] & 0xfe00) == 0xfc00 && !is_metadata(IpAddr::V6(v6)),
    }
}

pub fn cleartext_permitted(ip: IpAddr) -> bool {
    permitted_for_private_remote(ip) || canonical(ip).is_loopback()
}

fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(v6)),
        v4 => v4,
    }
}

fn is_metadata(ip: IpAddr) -> bool {
    ip == IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))
        || ip == IpAddr::V6(Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254))
}

fn is_internal(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || (a == 100 && (64..128).contains(&b))
                || a == 0
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || (first & 0xffc0) == 0xfec0
        }
    }
}

#[derive(Debug)]
pub struct Blocked {
    host: String,
}

impl Blocked {
    pub fn host(&self) -> &str {
        &self.host
    }
}

impl fmt::Display for Blocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} resolves only to addresses the SSRF protection blocks",
            self.host
        )
    }
}

impl Error for Blocked {}

pub fn is_blocked(error: &(dyn Error + 'static)) -> bool {
    std::iter::successors(Some(error), |&error| error.source())
        .any(|error| error.downcast_ref::<Blocked>().is_some())
}

pub struct GuardedResolver {
    guard: Arc<Guard>,
    private_network: bool,
    cleartext: bool,
}

#[derive(Debug)]
pub enum Unverified {
    Blocked(Blocked),
    Lookup(String),
}

impl GuardedResolver {
    pub fn new(guard: Arc<Guard>) -> Self {
        Self {
            guard,
            private_network: false,
            cleartext: false,
        }
    }

    pub fn with_private_ranges(guard: Arc<Guard>) -> Self {
        Self {
            guard,
            private_network: true,
            cleartext: false,
        }
    }

    pub fn for_cleartext(mut self) -> Self {
        self.cleartext = true;
        self
    }

    pub fn permits(&self, ip: IpAddr, host_allowed: bool) -> bool {
        self.guard
            .remote_ip_allowed(ip, host_allowed, self.private_network)
            && (!self.cleartext || cleartext_permitted(ip))
    }

    pub async fn verify(&self, host: &str, port: u16) -> Result<(), Unverified> {
        let found: Vec<SocketAddr> =
            tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((host, port)))
                .await
                .map_err(|_| Unverified::Lookup(format!("{host} could not be resolved in time")))?
                .map_err(|error| Unverified::Lookup(format!("{host} cannot be resolved: {error}")))?
                .collect();
        if found.is_empty() {
            return Err(Unverified::Lookup(format!("{host} cannot be resolved")));
        }
        let host_allowed = self.guard.host_allowed(host);
        match found
            .iter()
            .all(|address| self.permits(address.ip(), host_allowed))
        {
            true => Ok(()),
            false => Err(Unverified::Blocked(Blocked {
                host: host.to_owned(),
            })),
        }
    }
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = GuardedResolver {
            guard: self.guard.clone(),
            private_network: self.private_network,
            cleartext: self.cleartext,
        };
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let found: Vec<SocketAddr> =
                tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let host_allowed = resolver.guard.host_allowed(&host);
            let allowed: Vec<SocketAddr> = found
                .into_iter()
                .filter(|address| resolver.permits(address.ip(), host_allowed))
                .collect();
            if allowed.is_empty() {
                return Err(Blocked { host }.into());
            }
            let addrs: Addrs = Box::new(allowed.into_iter());
            Ok(addrs)
        })
    }
}

pub const ADVICE: &str = "To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection.";

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

fn kind_of(ip: IpAddr) -> &'static str {
    let ip = canonical(ip);
    if is_metadata(ip) {
        return "restricted address";
    }
    match ip {
        IpAddr::V4(v4) if v4.is_loopback() => "loopback address",
        IpAddr::V4(v4) if v4.is_unspecified() => "wildcard address",
        IpAddr::V4(v4) if v4.is_private() => "private network address",
        IpAddr::V4(v4) if v4.is_link_local() => "link-local address",
        IpAddr::V6(v6) if v6.is_loopback() => "loopback address",
        IpAddr::V6(v6) if v6.is_unspecified() => "wildcard address",
        IpAddr::V6(v6) if (v6.segments()[0] & 0xffc0) == 0xfe80 => "link-local address",
        IpAddr::V6(v6)
            if (v6.segments()[0] & 0xfe00) == 0xfc00 || (v6.segments()[0] & 0xffc0) == 0xfec0 =>
        {
            "private network address"
        }
        _ => "restricted address",
    }
}

fn shown(ip: IpAddr) -> String {
    match canonical(ip) {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => v6
            .segments()
            .iter()
            .map(|segment| format!("{segment:x}"))
            .collect::<Vec<_>>()
            .join(":"),
    }
}

pub async fn refused_destination(settings: &SsrfProtection, address: &str) -> Option<String> {
    let url = url::Url::parse(address.trim()).ok()?;
    let guard = Guard::new(settings);
    let (mut addresses, host_allowed): (Vec<IpAddr>, bool) = match url.host()? {
        url::Host::Ipv4(ip) => (vec![IpAddr::V4(ip)], false),
        url::Host::Ipv6(ip) => (vec![IpAddr::V6(ip)], false),
        url::Host::Domain(domain) => {
            let port = url.port_or_known_default().unwrap_or(80);
            let found =
                tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((domain, port)))
                    .await
                    .ok()?
                    .ok()?;
            (
                found.map(|found| found.ip()).collect(),
                guard.host_allowed(domain),
            )
        }
    };
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|ip| guard.ip_allowed(*ip, host_allowed))
    {
        return None;
    }
    addresses.sort_by_key(|ip| canonical(*ip).is_ipv6());
    let mut blocked: Vec<String> = Vec::new();
    for ip in addresses {
        let entry = format!("{} ({})", shown(ip), kind_of(ip));
        if !blocked.contains(&entry) {
            blocked.push(entry);
        }
    }
    Some(format!(
        "Host resolves to private/local IP address(es): {}. {ADVICE}",
        blocked.join(", ")
    ))
}

pub async fn refused_remote_destination(
    settings: &SsrfProtection,
    url: &url::Url,
    private_network: bool,
) -> Option<String> {
    if !private_network {
        return refused_destination(settings, url.as_str()).await;
    }
    let guard = Guard::new(settings);
    let (addresses, host_allowed): (Vec<IpAddr>, bool) = match url.host()? {
        url::Host::Ipv4(ip) => (vec![IpAddr::V4(ip)], false),
        url::Host::Ipv6(ip) => (vec![IpAddr::V6(ip)], false),
        url::Host::Domain(domain) => {
            let port = url.port_or_known_default().unwrap_or(80);
            let found =
                tokio::time::timeout(LOOKUP_TIMEOUT, tokio::net::lookup_host((domain, port)))
                    .await
                    .ok()?
                    .ok()?;
            (
                found.map(|found| found.ip()).collect(),
                guard.host_allowed(domain),
            )
        }
    };
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|ip| guard.remote_ip_allowed(*ip, host_allowed, true))
    {
        return None;
    }
    let mut shown: Vec<String> = addresses
        .iter()
        .map(|ip| canonical(*ip).to_string())
        .collect();
    shown.sort();
    shown.dedup();
    Some(format!(
        "Host resolves only to addresses a remote may not use even with private networks allowed (loopback, link-local, unspecified, multicast, broadcast): {}",
        shown.join(", ")
    ))
}

pub fn literal_allowed(guard: &Guard, url: &url::Url) -> bool {
    remote_literal_allowed(guard, url, false)
}

pub fn remote_literal_allowed(guard: &Guard, url: &url::Url, private_network: bool) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => {
            guard.remote_ip_allowed(IpAddr::V4(ip), false, private_network)
        }
        Some(url::Host::Ipv6(ip)) => {
            guard.remote_ip_allowed(IpAddr::V6(ip), false, private_network)
        }
        Some(url::Host::Domain(_)) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn networks_and_addresses() {
        let network = Network::parse("10.0.0.0/8").unwrap();
        assert!(network.contains("10.1.2.3".parse().unwrap()));
        assert!(!network.contains("11.0.0.1".parse().unwrap()));
        assert!(network.contains("::ffff:10.0.0.1".parse().unwrap()));
        let single = Network::parse("192.168.1.100").unwrap();
        assert!(single.contains("192.168.1.100".parse().unwrap()));
        assert!(!single.contains("192.168.1.101".parse().unwrap()));
        assert_eq!(single.to_string(), "192.168.1.100/32");
        assert_eq!(network.to_string(), "10.0.0.0/8");
        let v6 = Network::parse("fd00::/8").unwrap();
        assert!(v6.contains("fd12::1".parse().unwrap()));
        assert!(!v6.contains("fe80::1".parse().unwrap()));
        assert!(
            Network::parse("0.0.0.0/0")
                .unwrap()
                .contains("8.8.8.8".parse().unwrap())
        );
        for bad in [
            "",
            "not-an-ip",
            "10.0.0.0/33",
            "10.0.0.0/",
            "10.0.0.0/+8",
            "::/129",
            "10.0.0",
        ] {
            assert!(Network::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn clear_text_stays_on_private_networks_and_the_machine_itself() {
        let ip = |text: &str| text.parse::<IpAddr>().unwrap();
        for private in [
            "10.1.2.3",
            "192.168.0.9",
            "100.64.0.1",
            "fd12::1",
            "::ffff:10.0.0.1",
            "127.0.0.1",
            "::1",
        ] {
            assert!(cleartext_permitted(ip(private)), "{private}");
        }
        for public in [
            "203.0.113.7",
            "8.8.8.8",
            "2001:db8::1",
            "169.254.169.254",
            "fe80::1",
            "0.0.0.0",
        ] {
            assert!(!cleartext_permitted(ip(public)), "{public}");
        }
        let off = SsrfProtection {
            enabled: false,
            ..SsrfProtection::default()
        };
        let guard = Arc::new(Guard::new(&off));
        let plain = GuardedResolver::with_private_ranges(guard.clone());
        let clear = GuardedResolver::with_private_ranges(guard).for_cleartext();
        assert!(plain.permits(ip("203.0.113.7"), false));
        assert!(!clear.permits(ip("203.0.113.7"), false));
        assert!(clear.permits(ip("10.0.0.1"), false));
        let on = Arc::new(Guard::new(&SsrfProtection::default()));
        let strict = GuardedResolver::with_private_ranges(on).for_cleartext();
        assert!(strict.permits(ip("10.0.0.1"), false));
        assert!(!strict.permits(ip("127.0.0.1"), false));
        assert!(!strict.permits(ip("203.0.113.7"), false));
    }

    #[test]
    fn domain_names() {
        for good in [
            "registry.local",
            "internal.corp.com",
            "*.corp.com",
            "localhost",
            "a_b.example.",
        ] {
            assert!(valid_domain(good), "{good}");
        }
        for bad in [
            "",
            "bad domain!",
            "-a.com",
            "a..b",
            "*.",
            "http://a.com",
            "a.com/x",
            "a.*.com",
        ] {
            assert!(!valid_domain(bad), "{bad}");
        }
    }

    #[test]
    fn protection_is_on_by_default() {
        let stored: SsrfProtection = serde_json::from_str(r#"{"enabled":true}"#).unwrap();
        assert_eq!(stored, SsrfProtection::default());
        assert_eq!(
            serde_json::to_value(SsrfProtection::default()).unwrap(),
            serde_json::json!({"enabled": true, "allowedIPs": [], "allowedDomains": []})
        );
    }

    fn guard(enabled: bool, ips: &[&str], domains: &[&str]) -> Guard {
        Guard::new(&SsrfProtection {
            enabled,
            allowed_ips: ips.iter().map(|ip| (*ip).to_owned()).collect(),
            allowed_domains: domains.iter().map(|domain| (*domain).to_owned()).collect(),
        })
    }

    #[test]
    fn internal_addresses_are_blocked_unless_allowed() {
        let strict = guard(true, &[], &[]);
        for blocked in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.0.10",
            "172.20.0.1",
            "169.254.1.1",
            "::1",
            "fd12::1",
            "fe80::1",
            "100.64.0.1",
            "0.0.0.0",
            "::ffff:127.0.0.1",
            "fec0::1",
        ] {
            assert!(
                !strict.ip_allowed(blocked.parse().unwrap(), false),
                "{blocked}"
            );
        }
        for open in ["8.8.8.8", "2606:4700::1111", "100.128.0.1"] {
            assert!(strict.ip_allowed(open.parse().unwrap(), false), "{open}");
        }
        let lab = guard(true, &["10.0.0.0/8", "127.0.0.1"], &[]);
        assert!(lab.ip_allowed("10.9.9.9".parse().unwrap(), false));
        assert!(lab.ip_allowed("127.0.0.1".parse().unwrap(), false));
        assert!(!lab.ip_allowed("192.168.1.1".parse().unwrap(), false));
    }

    #[test]
    fn metadata_addresses_are_always_blocked() {
        let off = guard(false, &["169.254.0.0/16"], &["*.internal"]);
        assert!(!off.ip_allowed("169.254.169.254".parse().unwrap(), true));
        assert!(off.ip_allowed("10.0.0.1".parse().unwrap(), false));
    }

    #[test]
    fn allowed_domains_match_exactly_or_by_suffix() {
        let domains = guard(true, &[], &["repo.corp", "*.svc.local"]);
        assert!(domains.host_allowed("repo.corp"));
        assert!(domains.host_allowed("REPO.corp."));
        assert!(domains.host_allowed("a.b.svc.local"));
        assert!(!domains.host_allowed("svc.local"));
        assert!(!domains.host_allowed("evilrepo.corp"));
    }

    #[tokio::test]
    async fn destinations_are_judged_like_the_old_server() {
        let on = SsrfProtection::default();
        let refused = refused_destination(&on, "http://127.0.0.1:9/x/")
            .await
            .unwrap();
        assert_eq!(
            refused,
            format!(
                "Host resolves to private/local IP address(es): 127.0.0.1 (loopback address). {ADVICE}"
            )
        );
        for (url, kind) in [
            ("http://10.1.2.3/", "10.1.2.3 (private network address)"),
            ("http://172.16.0.1/", "172.16.0.1 (private network address)"),
            (
                "http://192.168.1.1/",
                "192.168.1.1 (private network address)",
            ),
            ("http://127.1.2.3/", "127.1.2.3 (loopback address)"),
            ("http://2130706433/", "127.0.0.1 (loopback address)"),
            (
                "http://169.254.169.254/",
                "169.254.169.254 (restricted address)",
            ),
            ("http://169.254.1.1/", "169.254.1.1 (link-local address)"),
            ("http://0.0.0.0/", "0.0.0.0 (wildcard address)"),
            ("http://[::1]:8081/", "0:0:0:0:0:0:0:1 (loopback address)"),
            ("http://[::]/", "0:0:0:0:0:0:0:0 (wildcard address)"),
            (
                "http://[fd00::1]/",
                "fd00:0:0:0:0:0:0:1 (private network address)",
            ),
            (
                "http://[fec0::1]/",
                "fec0:0:0:0:0:0:0:1 (private network address)",
            ),
            (
                "http://[fe80::1]/",
                "fe80:0:0:0:0:0:0:1 (link-local address)",
            ),
            (
                "http://[fd00:ec2::254]/",
                "fd00:ec2:0:0:0:0:0:254 (restricted address)",
            ),
            (
                "http://[::ffff:10.0.0.1]/",
                "10.0.0.1 (private network address)",
            ),
        ] {
            let refused = refused_destination(&on, url).await.unwrap();
            assert!(refused.contains(&format!(": {kind}. ")), "{url}: {refused}");
        }
        let both = refused_destination(&on, "http://localhost:9/")
            .await
            .unwrap();
        assert!(
            both.starts_with(
                "Host resolves to private/local IP address(es): 127.0.0.1 (loopback address)"
            ),
            "{both}"
        );
        assert!(
            refused_destination(&on, "https://93.184.215.14/")
                .await
                .is_none()
        );
        assert!(
            refused_destination(&on, "http://no-such-host.invalid/")
                .await
                .is_none()
        );
        assert!(refused_destination(&on, "not a url").await.is_none());
        let allowed = SsrfProtection {
            allowed_ips: vec!["127.0.0.1".to_owned(), "10.0.0.0/8".to_owned()],
            ..SsrfProtection::default()
        };
        assert!(
            refused_destination(&allowed, "http://127.0.0.1:9/")
                .await
                .is_none()
        );
        assert!(
            refused_destination(&allowed, "http://10.9.8.7/")
                .await
                .is_none()
        );
        assert!(
            refused_destination(&allowed, "http://192.168.1.1/")
                .await
                .is_some()
        );
        let named = SsrfProtection {
            allowed_domains: vec!["localhost".to_owned()],
            ..SsrfProtection::default()
        };
        assert!(
            refused_destination(&named, "http://localhost:9/")
                .await
                .is_none()
        );
        assert!(
            refused_destination(&named, "http://127.0.0.1:9/")
                .await
                .is_some()
        );
        let off = SsrfProtection {
            enabled: false,
            ..SsrfProtection::default()
        };
        assert!(
            refused_destination(&off, "http://10.1.2.3/")
                .await
                .is_none()
        );
        assert!(
            refused_destination(&off, "http://169.254.169.254/")
                .await
                .is_some()
        );
    }

    #[test]
    fn literal_hosts_are_checked_before_sending() {
        let strict = guard(true, &[], &[]);
        assert!(!literal_allowed(
            &strict,
            &"http://127.0.0.1:8080/x".parse().unwrap()
        ));
        assert!(!literal_allowed(
            &strict,
            &"http://[::1]/x".parse().unwrap()
        ));
        assert!(literal_allowed(
            &strict,
            &"https://repo1.maven.org/x".parse().unwrap()
        ));
    }

    fn ip(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    const PRIVATE: [&str; 5] = [
        "10.1.2.3",
        "172.16.0.1",
        "192.168.1.1",
        "fd00::1",
        "100.64.0.1",
    ];
    const NEVER: [&str; 11] = [
        "127.0.0.1",
        "::1",
        "169.254.1.1",
        "169.254.169.254",
        "fe80::1",
        "fd00:ec2::254",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "::ffff:127.0.0.1",
        "::",
    ];

    #[test]
    fn private_ranges_open_only_for_a_remote_that_allows_them() {
        let strict = guard(true, &[], &[]);
        for private in PRIVATE {
            assert!(permitted_for_private_remote(ip(private)), "{private}");
            assert!(
                !strict.remote_ip_allowed(ip(private), false, false),
                "{private}"
            );
            assert!(
                strict.remote_ip_allowed(ip(private), false, true),
                "{private}"
            );
        }
        for never in NEVER {
            assert!(!permitted_for_private_remote(ip(never)), "{never}");
            assert!(!strict.remote_ip_allowed(ip(never), false, true), "{never}");
        }
        assert!(strict.remote_ip_allowed(ip("8.8.8.8"), false, false));
        assert!(!permitted_for_private_remote(ip("8.8.8.8")));
    }

    #[test]
    fn a_literal_remote_address_follows_the_same_rule() {
        let strict = guard(true, &[], &[]);
        let url = |text: &str| -> url::Url { text.parse().unwrap() };
        assert!(!remote_literal_allowed(
            &strict,
            &url("http://10.0.0.5:8081"),
            false
        ));
        assert!(remote_literal_allowed(
            &strict,
            &url("http://10.0.0.5:8081"),
            true
        ));
        assert!(remote_literal_allowed(
            &strict,
            &url("http://[fd00::1]:8081"),
            true
        ));
        assert!(!remote_literal_allowed(
            &strict,
            &url("http://127.0.0.1:8081"),
            true
        ));
        assert!(!remote_literal_allowed(
            &strict,
            &url("http://[::1]:8081"),
            true
        ));
    }

    async fn resolved(resolver: &GuardedResolver, host: &str) -> Result<Vec<SocketAddr>, String> {
        let name: Name = host.parse().unwrap();
        match resolver.resolve(name).await {
            Ok(addresses) => Ok(addresses.collect()),
            Err(error) => {
                assert!(is_blocked(&*error), "{host}: {error}");
                Err(error.to_string())
            }
        }
    }

    #[tokio::test]
    async fn the_resolver_takes_private_ranges_only_when_built_for_them() {
        let global = Arc::new(guard(true, &[], &[]));
        let plain = GuardedResolver::new(global.clone());
        let private = GuardedResolver::with_private_ranges(global);
        for address in PRIVATE {
            assert!(resolved(&plain, address).await.is_err(), "{address}");
            assert!(resolved(&private, address).await.is_ok(), "{address}");
        }
        for address in NEVER {
            assert!(resolved(&plain, address).await.is_err(), "{address}");
            assert!(resolved(&private, address).await.is_err(), "{address}");
        }
        assert!(resolved(&plain, "8.8.8.8").await.is_ok());
    }

    #[test]
    fn only_the_blocked_error_counts_as_blocked() {
        let blocked: Box<dyn Error + Send + Sync> = Blocked {
            host: "a.example".to_owned(),
        }
        .into();
        assert!(is_blocked(&*blocked));
        let other: Box<dyn Error + Send + Sync> = "connection refused".into();
        assert!(!is_blocked(&*other));
    }

    #[tokio::test]
    async fn a_remote_destination_with_private_networks_still_refuses_loopback() {
        let strict = SsrfProtection::default();
        let url = |text: &str| -> url::Url { text.parse().unwrap() };
        assert!(
            refused_remote_destination(&strict, &url("http://10.1.2.3:8081"), false)
                .await
                .is_some()
        );
        assert!(
            refused_remote_destination(&strict, &url("http://10.1.2.3:8081"), true)
                .await
                .is_none()
        );
        for refused in [
            "http://127.0.0.1:8081",
            "http://[::1]:8081",
            "http://169.254.169.254",
        ] {
            assert!(
                refused_remote_destination(&strict, &url(refused), true)
                    .await
                    .is_some(),
                "{refused}"
            );
        }
    }
}
