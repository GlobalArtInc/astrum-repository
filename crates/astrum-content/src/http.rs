use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use astrum_repo::config::Repository;
use astrum_security::ssrf::{Network, SsrfProtection};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const MAX_REDIRECTS: usize = 10;

#[derive(Clone, Debug)]
pub(crate) struct Guard {
    enabled: bool,
    networks: Vec<Network>,
    domains: Vec<String>,
}

impl Guard {
    pub(crate) fn new(settings: &SsrfProtection) -> Self {
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

    pub(crate) fn host_allowed(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        self.domains
            .iter()
            .any(|domain| match domain.strip_prefix("*.") {
                Some(parent) => host.ends_with(&format!(".{parent}")),
                None => *domain == host,
            })
    }

    pub(crate) fn ip_allowed(&self, ip: IpAddr, host_allowed: bool) -> bool {
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

struct GuardedResolver {
    guard: Arc<Guard>,
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

pub(crate) fn literal_allowed(guard: &Guard, url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => guard.ip_allowed(IpAddr::V4(ip), false),
        Some(url::Host::Ipv6(ip)) => guard.ip_allowed(IpAddr::V6(ip), false),
        Some(url::Host::Domain(_)) => true,
        None => false,
    }
}

pub(crate) fn build(
    repository: &Repository,
    guard: Arc<Guard>,
) -> Result<reqwest::Client, reqwest::Error> {
    let connection = repository
        .attributes
        .http_client
        .as_ref()
        .and_then(|client| client.connection.as_ref());
    let timeout = connection
        .and_then(|connection| connection.timeout)
        .filter(|seconds| *seconds > 0)
        .map(|seconds| seconds as u64)
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS);
    let suffix = connection
        .and_then(|connection| connection.user_agent_suffix.clone())
        .filter(|suffix| !suffix.trim().is_empty());
    let agent = match suffix {
        Some(suffix) => format!("Astrum/{} {suffix}", env!("CARGO_PKG_VERSION")),
        None => format!("Astrum/{}", env!("CARGO_PKG_VERSION")),
    };
    let redirect_guard = guard.clone();
    let redirects = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            attempt.error("too many redirects")
        } else if !literal_allowed(&redirect_guard, attempt.url()) {
            attempt.error("the redirect goes to an address the SSRF protection blocks")
        } else {
            attempt.follow()
        }
    });
    reqwest::Client::builder()
        .user_agent(agent)
        .connect_timeout(Duration::from_secs(timeout))
        .read_timeout(Duration::from_secs(timeout))
        .redirect(redirects)
        .no_proxy()
        .dns_resolver(Arc::new(GuardedResolver { guard }))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

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
