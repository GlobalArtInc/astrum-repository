use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

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
        }
    }
}

pub struct GuardedResolver {
    guard: Arc<Guard>,
}

impl GuardedResolver {
    pub fn new(guard: Arc<Guard>) -> Self {
        Self { guard }
    }
}

impl Resolve for GuardedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let guard = self.guard.clone();
        let host = name.as_str().to_owned();
        Box::pin(async move {
            let found: Vec<SocketAddr> =
                tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            let host_allowed = guard.host_allowed(&host);
            let allowed: Vec<SocketAddr> = found
                .into_iter()
                .filter(|address| guard.ip_allowed(address.ip(), host_allowed))
                .collect();
            if allowed.is_empty() {
                return Err(format!(
                    "{host} resolves only to addresses the SSRF protection blocks"
                )
                .into());
            }
            let addrs: Addrs = Box::new(allowed.into_iter());
            Ok(addrs)
        })
    }
}

pub const ADVICE: &str = "To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection.";

fn kind_of(ip: IpAddr) -> &'static str {
    match canonical(ip) {
        IpAddr::V4(v4) if v4.is_loopback() => "loopback address",
        IpAddr::V6(v6) if v6.is_loopback() => "loopback address",
        IpAddr::V4(v4) if v4.is_private() => "private network address",
        IpAddr::V6(v6) if (v6.segments()[0] & 0xfe00) == 0xfc00 => "private network address",
        _ => "restricted address",
    }
}

pub async fn blocked_destination(settings: &SsrfProtection, address: &str) -> Option<String> {
    let url = url::Url::parse(address.trim()).ok()?;
    let host = url.host_str()?.trim_matches(|c| c == '[' || c == ']').to_owned();
    let guard = Guard::new(settings);
    let host_allowed = guard.host_allowed(&host);
    let addresses: Vec<IpAddr> = match url.host()? {
        url::Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
        url::Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
        url::Host::Domain(domain) => {
            let port = url.port_or_known_default().unwrap_or(80);
            tokio::net::lookup_host((domain, port))
                .await
                .ok()?
                .map(|found| found.ip())
                .collect()
        }
    };
    let mut blocked: Vec<String> = Vec::new();
    for ip in addresses {
        if !guard.ip_allowed(ip, host_allowed) {
            let shown = format!("{} ({})", canonical(ip), kind_of(ip));
            if !blocked.contains(&shown) {
                blocked.push(shown);
            }
        }
    }
    (!blocked.is_empty()).then(|| {
        format!(
            "Host resolves to private/local IP address(es): {}. {ADVICE}",
            blocked.join(", ")
        )
    })
}

pub fn literal_allowed(guard: &Guard, url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => guard.ip_allowed(IpAddr::V4(ip), false),
        Some(url::Host::Ipv6(ip)) => guard.ip_allowed(IpAddr::V6(ip), false),
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
        let domains = guard(true, &[], &["nexus.corp", "*.svc.local"]);
        assert!(domains.host_allowed("nexus.corp"));
        assert!(domains.host_allowed("NEXUS.corp."));
        assert!(domains.host_allowed("a.b.svc.local"));
        assert!(!domains.host_allowed("svc.local"));
        assert!(!domains.host_allowed("evilnexus.corp"));
    }

    #[tokio::test]
    async fn destinations_are_judged_like_the_old_server() {
        let on = SsrfProtection::default();
        let blocked = blocked_destination(&on, "http://127.0.0.1:9/x/").await.unwrap();
        assert_eq!(
            blocked,
            format!("Host resolves to private/local IP address(es): 127.0.0.1 (loopback address). {ADVICE}")
        );
        for (url, kind) in [
            ("http://10.1.2.3/", "10.1.2.3 (private network address)"),
            ("http://169.254.169.254/", "169.254.169.254 (restricted address)"),
            ("http://[::1]:8081/", "::1 (loopback address)"),
        ] {
            let blocked = blocked_destination(&on, url).await.unwrap();
            assert!(blocked.contains(kind), "{blocked}");
        }
        assert!(blocked_destination(&on, "https://93.184.215.14/").await.is_none());
        assert!(blocked_destination(&on, "http://no-such-host.invalid/").await.is_none());
        assert!(blocked_destination(&on, "not a url").await.is_none());
        let allowed = SsrfProtection {
            allowed_ips: vec!["127.0.0.1".to_owned()],
            ..SsrfProtection::default()
        };
        assert!(blocked_destination(&allowed, "http://127.0.0.1:9/").await.is_none());
        let off = SsrfProtection {
            enabled: false,
            ..SsrfProtection::default()
        };
        assert!(blocked_destination(&off, "http://10.1.2.3/").await.is_none());
        assert!(blocked_destination(&off, "http://169.254.169.254/").await.is_some());
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
}
