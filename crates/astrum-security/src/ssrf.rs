use std::net::IpAddr;

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
}
