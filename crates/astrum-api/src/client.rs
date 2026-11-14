use std::fmt;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};

use astrum_security::ssrf::Network;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;

const FORWARDED_FOR: &str = "x-forwarded-for";
const MAPPED_PREFIX: u8 = 96;
const MIN_V4_PREFIX: u8 = 8;
const MIN_V6_PREFIX: u8 = 16;

static PEER_WARNING: AtomicBool = AtomicBool::new(false);

tokio::task_local! {
    static CLIENT: Option<IpAddr>;
}

pub(crate) static NO_PROXIES: TrustedProxies = TrustedProxies {
    networks: Vec::new(),
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientAddress(pub Option<IpAddr>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedProxies {
    networks: Vec<Network>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidProxy {
    #[error("\"{0}\" is neither an IP address nor a range such as 10.0.0.0/8")]
    Unparsable(String),
    #[error(
        "\"{0}\" trusts every address; trusting every address defeats the purpose, list the proxies"
    )]
    EveryAddress(String),
    #[error(
        "\"{0}\" is an IPv4-mapped IPv6 range shorter than /96, which can never match; write it as an IPv4 range"
    )]
    MappedTooShort(String),
    #[error(
        "\"{entry}\" trusts too large a part of the address space; the shortest {family} range allowed is /{minimum}, list the proxies"
    )]
    TooBroad {
        entry: String,
        family: &'static str,
        minimum: u8,
    },
}

impl TrustedProxies {
    pub fn parse<'a>(entries: impl IntoIterator<Item = &'a str>) -> Result<Self, InvalidProxy> {
        let mut networks = Vec::new();
        for entry in entries {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            networks.push(network(entry)?);
        }
        Ok(Self { networks })
    }

    pub fn contains(&self, address: IpAddr) -> bool {
        self.networks
            .iter()
            .any(|network| network.contains(address))
    }

    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }
}

fn network(entry: &str) -> Result<Network, InvalidProxy> {
    let unparsable = || InvalidProxy::Unparsable(entry.to_owned());
    Network::parse(entry).ok_or_else(unparsable)?;
    let (address, prefix) = match entry.split_once('/') {
        Some((address, prefix)) => (address, Some(prefix)),
        None => (entry, None),
    };
    let address: IpAddr = address.parse().map_err(|_| unparsable())?;
    let prefix: Option<u8> = match prefix {
        Some(prefix) => Some(prefix.parse().map_err(|_| unparsable())?),
        None => None,
    };
    if prefix == Some(0) {
        return Err(InvalidProxy::EveryAddress(entry.to_owned()));
    }
    let (address, prefix) = match address {
        IpAddr::V4(_) => (address, prefix.unwrap_or(32)),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => {
                let prefix = prefix
                    .unwrap_or(128)
                    .checked_sub(MAPPED_PREFIX)
                    .ok_or_else(|| InvalidProxy::MappedTooShort(entry.to_owned()))?;
                (IpAddr::V4(v4), prefix)
            }
            None => (address, prefix.unwrap_or(128)),
        },
    };
    if prefix == 0 {
        return Err(InvalidProxy::EveryAddress(entry.to_owned()));
    }
    let (family, minimum) = match address {
        IpAddr::V4(_) => ("IPv4", MIN_V4_PREFIX),
        IpAddr::V6(_) => ("IPv6", MIN_V6_PREFIX),
    };
    if prefix < minimum {
        return Err(InvalidProxy::TooBroad {
            entry: entry.to_owned(),
            family,
            minimum,
        });
    }
    Network::parse(&format!("{address}/{prefix}")).ok_or_else(unparsable)
}

impl fmt::Display for TrustedProxies {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, network) in self.networks.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            write!(formatter, "{network}")?;
        }
        Ok(())
    }
}

pub fn current() -> Option<IpAddr> {
    CLIENT.try_with(|address| *address).ok().flatten()
}

pub(crate) async fn with_address<F: std::future::Future>(
    address: Option<IpAddr>,
    future: F,
) -> F::Output {
    CLIENT.scope(address, future).await
}

pub fn throttle_bucket() -> String {
    astrum_security::sessions::throttle_address(current())
}

pub fn address(
    peer: Option<IpAddr>,
    headers: &HeaderMap,
    trusted: &TrustedProxies,
) -> Option<IpAddr> {
    let peer = peer?.to_canonical();
    if !trusted.contains(peer) {
        return Some(peer);
    }
    let entries: Vec<&[u8]> = headers
        .get_all(FORWARDED_FOR)
        .iter()
        .flat_map(|value| value.as_bytes().split(|byte| *byte == b','))
        .collect();
    let mut client = peer;
    for entry in entries.into_iter().rev() {
        let Some(hop) = hop(entry) else {
            return Some(client);
        };
        client = hop;
        if !trusted.contains(hop) {
            break;
        }
    }
    Some(client)
}

fn hop(entry: &[u8]) -> Option<IpAddr> {
    let entry = std::str::from_utf8(entry.trim_ascii()).ok()?;
    if let Ok(address) = entry.parse::<IpAddr>() {
        return Some(address.to_canonical());
    }
    if let Ok(socket) = entry.parse::<SocketAddr>() {
        return Some(socket.ip().to_canonical());
    }
    let inner = entry.strip_prefix('[')?.strip_suffix(']')?;
    let address: Ipv6Addr = inner.parse().ok()?;
    Some(IpAddr::V6(address).to_canonical())
}

pub async fn scope(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    if peer.is_none() && !PEER_WARNING.swap(true, Ordering::Relaxed) {
        tracing::warn!(
            "a request has no peer address, so client addresses are unknown and failed-password throttling puts such requests into one shared bucket"
        );
    }
    let address = address(peer, request.headers(), state.trusted_proxies());
    request.extensions_mut().insert(ClientAddress(address));
    CLIENT.scope(address, next.run(request)).await
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn trusted(entries: &[&str]) -> TrustedProxies {
        TrustedProxies::parse(entries.iter().copied()).unwrap()
    }

    fn lines(values: &[&str]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for value in values {
            headers.append(FORWARDED_FOR, HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    fn derive(peer: &str, forwarded: &[&str], trusted: &TrustedProxies) -> Option<String> {
        address(Some(peer.parse().unwrap()), &lines(forwarded), trusted)
            .map(|address| address.to_string())
    }

    #[test]
    fn an_untrusted_peer_is_the_client_whatever_the_headers_say() {
        let proxies = trusted(&["10.0.0.0/8"]);
        let mut headers = lines(&["198.51.100.1"]);
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.2"));
        headers.insert(
            "forwarded",
            HeaderValue::from_static("for=198.51.100.3;proto=https"),
        );
        let peer = Some("203.0.113.7".parse().unwrap());
        assert_eq!(
            address(peer, &headers, &proxies),
            Some("203.0.113.7".parse().unwrap())
        );
        assert_eq!(
            address(peer, &headers, &TrustedProxies::default()),
            Some("203.0.113.7".parse().unwrap())
        );
    }

    #[test]
    fn a_trusted_peer_gives_the_rightmost_untrusted_entry() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(
            derive("10.0.0.1", &["198.51.100.9, 203.0.113.5"], &proxies).as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            derive(
                "10.0.0.1",
                &["198.51.100.9, 203.0.113.5, 10.2.2.2, 10.3.3.3"],
                &proxies
            )
            .as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            derive("10.0.0.1", &[], &proxies).as_deref(),
            Some("10.0.0.1")
        );
    }

    #[test]
    fn every_header_line_counts_in_order() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(
            derive(
                "10.0.0.1",
                &["198.51.100.9", "203.0.113.5, 10.2.2.2", "10.3.3.3"],
                &proxies
            )
            .as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            derive("10.0.0.1", &["198.51.100.9", "10.4.4.4"], &proxies).as_deref(),
            Some("198.51.100.9")
        );
    }

    #[test]
    fn when_every_hop_is_trusted_the_leftmost_wins() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(
            derive("10.0.0.1", &["10.9.9.9, 10.8.8.8"], &proxies).as_deref(),
            Some("10.9.9.9")
        );
    }

    #[test]
    fn an_unparsable_entry_stops_the_walk_at_the_last_good_hop() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(
            derive("10.0.0.1", &["198.51.100.9, garbage, 10.2.2.2"], &proxies).as_deref(),
            Some("10.2.2.2")
        );
        assert_eq!(
            derive("10.0.0.1", &["198.51.100.9, unknown"], &proxies).as_deref(),
            Some("10.0.0.1")
        );
        assert_eq!(
            derive("10.0.0.1", &["198.51.100.9,, 10.2.2.2"], &proxies).as_deref(),
            Some("10.2.2.2")
        );
        let mut headers = HeaderMap::new();
        headers.append(FORWARDED_FOR, HeaderValue::from_static("198.51.100.9"));
        headers.append(FORWARDED_FOR, HeaderValue::from_bytes(b"\xff").unwrap());
        assert_eq!(
            address(Some("10.0.0.1".parse().unwrap()), &headers, &proxies),
            Some("10.0.0.1".parse().unwrap())
        );
    }

    #[test]
    fn addresses_are_canonical() {
        let proxies = trusted(&["10.0.0.0/8", "fd00::/16"]);
        assert_eq!(
            derive("::ffff:10.0.0.1", &["::ffff:203.0.113.5"], &proxies).as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            derive("fd00::1", &["2001:db8::7, fd00::2"], &proxies).as_deref(),
            Some("2001:db8::7")
        );
        assert_eq!(
            derive("10.0.0.1", &["[2001:db8::7]:4000"], &proxies).as_deref(),
            Some("2001:db8::7")
        );
        assert_eq!(
            derive("10.0.0.1", &["[2001:db8::7]"], &proxies).as_deref(),
            Some("2001:db8::7")
        );
        assert_eq!(
            derive("10.0.0.1", &["203.0.113.5:51000"], &proxies).as_deref(),
            Some("203.0.113.5")
        );
        assert_eq!(
            derive("::ffff:192.0.2.1", &["198.51.100.9"], &proxies).as_deref(),
            Some("192.0.2.1")
        );
    }

    fn raw_lines(values: &[&[u8]]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for value in values {
            headers.append(FORWARDED_FOR, HeaderValue::from_bytes(value).unwrap());
        }
        headers
    }

    fn derive_raw(forwarded: &[&[u8]], trusted: &TrustedProxies) -> Option<String> {
        address(
            Some("10.0.0.1".parse().unwrap()),
            &raw_lines(forwarded),
            trusted,
        )
        .map(|address| address.to_string())
    }

    #[test]
    fn garbage_spoils_only_its_own_entry() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(
            derive_raw(&[b"\x80, 203.0.113.9"], &proxies).as_deref(),
            Some("203.0.113.9")
        );
        assert_eq!(
            derive_raw(&[b"\xff\xfe,203.0.113.9 , 10.2.2.2"], &proxies).as_deref(),
            Some("203.0.113.9")
        );
        assert_eq!(
            derive_raw(&[b"\x80", b"203.0.113.9"], &proxies).as_deref(),
            Some("203.0.113.9")
        );
        assert_eq!(
            derive_raw(&[b"203.0.113.9, \x80"], &proxies).as_deref(),
            Some("10.0.0.1")
        );
        assert_eq!(
            derive_raw(&[b"203.0.113.9, \x80, 10.2.2.2"], &proxies).as_deref(),
            Some("10.2.2.2")
        );
        assert_eq!(
            derive_raw(&[b"\x80, 203.0.113.9, 10.2.2.2"], &proxies).as_deref(),
            Some("203.0.113.9")
        );
    }

    #[tokio::test]
    async fn limiter_buckets_follow_the_canonical_address_and_the_ipv6_prefix() {
        let bucket =
            |text: &str| CLIENT.scope(Some(text.parse().unwrap()), async { throttle_bucket() });
        assert_eq!(
            bucket("2001:db8:1:2::1").await,
            bucket("2001:db8:1:2:ffff:ffff:ffff:ffff").await
        );
        assert_ne!(
            bucket("2001:db8:1:2::1").await,
            bucket("2001:db8:1:3::1").await
        );
        assert_eq!(
            bucket("::ffff:203.0.113.9").await,
            bucket("203.0.113.9").await
        );
        assert_ne!(bucket("203.0.113.9").await, bucket("203.0.113.10").await);
        assert_eq!(throttle_bucket(), "unknown");
    }

    #[test]
    fn no_peer_means_no_address() {
        let proxies = trusted(&["10.0.0.0/8"]);
        assert_eq!(address(None, &lines(&["198.51.100.9"]), &proxies), None);
    }

    #[test]
    fn a_range_that_trusts_every_address_is_refused() {
        for every in ["0.0.0.0/0", "::/0", "10.0.0.0/0", "::ffff:0.0.0.0/96"] {
            let error = TrustedProxies::parse(["10.0.0.0/8", every]).unwrap_err();
            assert_eq!(error, InvalidProxy::EveryAddress(every.to_owned()));
            assert!(error.to_string().contains("list the proxies"), "{error}");
        }
        assert!(TrustedProxies::parse(["10.0.0.0/8", "fc00::/16", "10.0.0.1/32"]).is_ok());
    }

    #[test]
    fn a_range_that_trusts_a_huge_part_of_the_address_space_is_refused() {
        for (broad, family, minimum) in [
            ("0.0.0.0/1", "IPv4", 8),
            ("128.0.0.0/1", "IPv4", 8),
            ("10.0.0.0/7", "IPv4", 8),
            ("::/1", "IPv6", 16),
            ("8000::/1", "IPv6", 16),
            ("fc00::/15", "IPv6", 16),
            ("::ffff:0.0.0.0/100", "IPv4", 8),
        ] {
            let error = TrustedProxies::parse([broad]).unwrap_err();
            assert_eq!(
                error,
                InvalidProxy::TooBroad {
                    entry: broad.to_owned(),
                    family,
                    minimum
                }
            );
            let text = error.to_string();
            assert!(text.contains(broad), "{text}");
            assert!(text.contains(&format!("/{minimum}")), "{text}");
        }
        let edge = trusted(&["10.0.0.0/8", "fc00::/16", "::ffff:11.0.0.0/104"]);
        assert_eq!(edge.to_string(), "10.0.0.0/8, fc00::/16, 11.0.0.0/8");
    }

    #[test]
    fn ipv4_mapped_networks_are_stored_as_ipv4() {
        let proxies = trusted(&["::ffff:10.0.0.0/104", "::ffff:192.0.2.1"]);
        assert_eq!(proxies.to_string(), "10.0.0.0/8, 192.0.2.1/32");
        assert!(proxies.contains("10.9.9.9".parse().unwrap()));
        assert!(proxies.contains("::ffff:10.9.9.9".parse().unwrap()));
        assert!(proxies.contains("192.0.2.1".parse().unwrap()));
        assert!(!proxies.contains("192.0.2.2".parse().unwrap()));
        assert_eq!(
            derive("10.0.0.1", &["203.0.113.5"], &proxies).as_deref(),
            Some("203.0.113.5")
        );
        for short in ["::ffff:10.0.0.0/95", "::ffff:10.0.0.0/64"] {
            let error = TrustedProxies::parse([short]).unwrap_err();
            assert_eq!(error, InvalidProxy::MappedTooShort(short.to_owned()));
            assert!(error.to_string().contains("shorter than /96"), "{error}");
        }
    }

    #[test]
    fn the_proxy_list_refuses_bad_entries() {
        assert!(TrustedProxies::parse(["10.0.0.0/8", " ", "fd00::1"]).is_ok());
        assert!(TrustedProxies::parse([""]).unwrap().is_empty());
        for bad in ["gateway", "10.0.0.0/33", "10.0.0.0/", "300.1.1.1"] {
            let error = TrustedProxies::parse(["10.0.0.0/8", bad]).unwrap_err();
            assert_eq!(error, InvalidProxy::Unparsable(bad.to_owned()));
        }
        assert_eq!(
            trusted(&["10.0.0.0/8", "fd00::1"]).to_string(),
            "10.0.0.0/8, fd00::1/128"
        );
    }
}
