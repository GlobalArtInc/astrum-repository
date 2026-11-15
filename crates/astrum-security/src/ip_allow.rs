use std::net::IpAddr;

use astrum_db::audit::{self, Event};
use astrum_db::{revision, setting};
use percent_encoding::percent_decode_str;
use sea_orm::{ConnectionTrait, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Error;
use crate::ssrf::Network;

pub const IP_ALLOW_KEY: &str = "security.ipAllow";
pub const MAX_ENTRIES: usize = 256;
pub const STATUS: &str = "/service/rest/v1/status";
pub const STATUS_WRITABLE: &str = "/service/rest/v1/status/writable";
pub const COMPONENTS: &str = "/service/rest/v1/components";

const MAPPED_PREFIX: u8 = 96;
const CLI_INITIATOR: &str = "*CLI";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Surface {
    Management,
    Content,
    Agents,
}

impl Surface {
    pub const ALL: [Surface; 3] = [Surface::Management, Surface::Content, Surface::Agents];

    pub fn as_str(self) -> &'static str {
        match self {
            Surface::Management => "management",
            Surface::Content => "content",
            Surface::Agents => "agents",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpAllowSettings {
    #[serde(default)]
    pub management: Vec<String>,
    #[serde(default)]
    pub content: Vec<String>,
    #[serde(default)]
    pub agents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub surface: Surface,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IpAllow {
    management: Vec<Network>,
    content: Vec<Network>,
    agents: Vec<Network>,
}

impl IpAllowSettings {
    pub fn entries(&self, surface: Surface) -> &[String] {
        match surface {
            Surface::Management => &self.management,
            Surface::Content => &self.content,
            Surface::Agents => &self.agents,
        }
    }

    pub fn compile(&self) -> Result<IpAllow, Vec<Problem>> {
        let mut problems = Vec::new();
        let mut compiled = IpAllow::default();
        for surface in Surface::ALL {
            let entries = self.entries(surface);
            if entries.len() > MAX_ENTRIES {
                problems.push(Problem {
                    surface,
                    message: format!(
                        "has {} entries; at most {MAX_ENTRIES} are allowed",
                        entries.len()
                    ),
                });
                continue;
            }
            let mut networks = Vec::with_capacity(entries.len());
            for entry in entries {
                match network(entry.trim()) {
                    Ok(network) => networks.push(network),
                    Err(message) => problems.push(Problem { surface, message }),
                }
            }
            *compiled.networks_mut(surface) = networks;
        }
        match problems.is_empty() {
            true => Ok(compiled),
            false => Err(problems),
        }
    }
}

impl IpAllow {
    fn networks(&self, surface: Surface) -> &[Network] {
        match surface {
            Surface::Management => &self.management,
            Surface::Content => &self.content,
            Surface::Agents => &self.agents,
        }
    }

    fn networks_mut(&mut self, surface: Surface) -> &mut Vec<Network> {
        match surface {
            Surface::Management => &mut self.management,
            Surface::Content => &mut self.content,
            Surface::Agents => &mut self.agents,
        }
    }

    pub fn restricts(&self, surface: Surface) -> bool {
        !self.networks(surface).is_empty()
    }

    pub fn permits(&self, surface: Surface, address: Option<IpAddr>) -> bool {
        let networks = self.networks(surface);
        if networks.is_empty() {
            return true;
        }
        address.is_some_and(|address| networks.iter().any(|network| network.contains(address)))
    }
}

fn network(entry: &str) -> Result<Network, String> {
    let unparsable =
        || format!("\"{entry}\" is neither an IP address nor a range such as 10.0.0.0/8");
    let whole_family = || {
        format!(
            "\"{entry}\": an entry that matches a whole address family is not allowed; clear the list to allow everybody"
        )
    };
    let parsed = Network::parse(entry).ok_or_else(unparsable)?;
    let (address, prefix) = match entry.split_once('/') {
        Some((address, prefix)) => (address, Some(prefix)),
        None => (entry, None),
    };
    let address: IpAddr = address.parse().map_err(|_| unparsable())?;
    let prefix: Option<u8> = match prefix {
        Some(prefix) => Some(prefix.parse().map_err(|_| unparsable())?),
        None => None,
    };
    let IpAddr::V6(v6) = address else {
        return match prefix {
            Some(0) => Err(whole_family()),
            _ => Ok(parsed),
        };
    };
    let Some(v4) = v6.to_ipv4_mapped() else {
        return match prefix {
            Some(0) => Err(whole_family()),
            _ => Ok(parsed),
        };
    };
    let prefix = prefix.unwrap_or(128).checked_sub(MAPPED_PREFIX).ok_or_else(|| {
        format!(
            "\"{entry}\" is an IPv4-mapped IPv6 range shorter than /{MAPPED_PREFIX}, which can never match; write it as an IPv4 range"
        )
    })?;
    if prefix == 0 {
        return Err(whole_family());
    }
    Network::parse(&format!("{v4}/{prefix}")).ok_or_else(unparsable)
}

pub fn decode_path(raw: &str) -> Option<String> {
    let decoded = percent_decode_str(raw).decode_utf8().ok()?;
    match decoded.contains('\0') {
        true => None,
        false => Some(decoded.into_owned()),
    }
}

pub fn surface_of(method: &str, raw_path: &str) -> Option<Surface> {
    surface_of_decoded(method, decode_path(raw_path).as_deref())
}

pub fn surface_of_decoded(method: &str, decoded: Option<&str>) -> Option<Surface> {
    let Some(path) = decoded else {
        return Some(Surface::Management);
    };
    if matches!(method, "GET" | "HEAD") && (path == STATUS || path == STATUS_WRITABLE) {
        return None;
    }
    if under(path, "/mcp") || path.starts_with("/oauth/") || path.starts_with("/.well-known/oauth-")
    {
        return Some(Surface::Agents);
    }
    if method == "POST" && path == COMPONENTS {
        return Some(Surface::Content);
    }
    let content = [
        "/repository",
        "/content",
        "/v2",
        "/service/local/repositories",
        "/service/local/repo_groups",
    ];
    if content.iter().any(|root| under(path, root)) {
        return Some(Surface::Content);
    }
    Some(Surface::Management)
}

fn under(path: &str, root: &str) -> bool {
    path.strip_prefix(root)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

pub async fn stored(db: &impl ConnectionTrait) -> Result<IpAllowSettings, Error> {
    match setting::get(db, IP_ALLOW_KEY).await {
        Ok(stored) => Ok(stored.unwrap_or_default()),
        Err(astrum_db::Error::Setting { source, .. }) => Err(Error::IpAllow(format!(
            "the stored value has an unexpected shape: {source}"
        ))),
        Err(error) => Err(error.into()),
    }
}

pub async fn reset(db: &DatabaseConnection) -> Result<(), Error> {
    let txn = db.begin().await?;
    let before: Option<serde_json::Value> = setting::get(&txn, IP_ALLOW_KEY).await?;
    let after = IpAllowSettings::default();
    setting::put(&txn, IP_ALLOW_KEY, &after).await?;
    revision::bump(&txn, revision::SECURITY).await?;
    audit::record(
        &txn,
        Event::new("security.ipAllow", "updated", "ip-allow")
            .with(json!({"before": before, "after": after}))
            .by(CLI_INITIATOR),
    )
    .await?;
    txn.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(management: &[&str], content: &[&str], agents: &[&str]) -> IpAllowSettings {
        let owned = |entries: &[&str]| entries.iter().map(|entry| (*entry).to_owned()).collect();
        IpAllowSettings {
            management: owned(management),
            content: owned(content),
            agents: owned(agents),
        }
    }

    fn ip(value: &str) -> Option<IpAddr> {
        Some(value.parse().unwrap())
    }

    #[test]
    fn requests_fall_into_one_surface_by_method_and_path() {
        use Surface::{Agents, Content, Management};
        let table = [
            ("GET", "/", Some(Management)),
            ("GET", "/assets/index-1a2b.js", Some(Management)),
            ("GET", "/user/authorize", Some(Management)),
            ("GET", "/service/rest/v1/repositories", Some(Management)),
            ("GET", "/service/rest/internal/ui/user", Some(Management)),
            ("GET", "/service/metrics/prometheus", Some(Management)),
            ("GET", "/service/rest/v1/components", Some(Management)),
            (
                "DELETE",
                "/service/rest/v1/components/abc",
                Some(Management),
            ),
            ("POST", "/service/rest/v1/components", Some(Content)),
            ("POST", "/service/rest/v1/assets", Some(Management)),
            (
                "GET",
                "/service/rest/v1/search/assets/download",
                Some(Management),
            ),
            ("GET", "/service/local/other", Some(Management)),
            ("GET", "/repository/raw/a.txt", Some(Content)),
            ("PUT", "/repository/raw/dir/a.txt", Some(Content)),
            ("GET", "/repository", Some(Content)),
            ("GET", "/repository/", Some(Content)),
            ("GET", "/repositorysettings", Some(Management)),
            ("GET", "/content/repositories/raw/a.txt", Some(Content)),
            ("GET", "/content/sites/raw/index.html", Some(Content)),
            (
                "GET",
                "/service/local/repositories/raw/content/a.txt",
                Some(Content),
            ),
            (
                "GET",
                "/service/local/repo_groups/group/content/a.txt",
                Some(Content),
            ),
            ("GET", "/service/local/repositoriesX/a", Some(Management)),
            ("GET", "/v2", Some(Content)),
            ("GET", "/v2/", Some(Content)),
            ("PATCH", "/v2/team/app/blobs/uploads/abc", Some(Content)),
            ("GET", "/v2token", Some(Management)),
            ("POST", "/mcp", Some(Agents)),
            ("PUT", "/mcp/upload/ticket", Some(Agents)),
            ("GET", "/mcp", Some(Agents)),
            ("GET", "/mcpx", Some(Management)),
            ("POST", "/oauth/token", Some(Agents)),
            ("GET", "/oauth/authorize", Some(Agents)),
            (
                "GET",
                "/.well-known/oauth-protected-resource/mcp",
                Some(Agents),
            ),
            (
                "GET",
                "/.well-known/oauth-authorization-server",
                Some(Agents),
            ),
            ("GET", "/.well-known/other", Some(Management)),
            ("GET", "/service/rest/v1/status", None),
            ("HEAD", "/service/rest/v1/status", None),
            ("GET", "/service/rest/v1/status/writable", None),
            ("POST", "/service/rest/v1/status", Some(Management)),
            ("GET", "/service/rest/v1/status/other", Some(Management)),
            ("GET", "//repository/raw/a.txt", Some(Management)),
            (
                "GET",
                "/service/local/%72epositories/files/a.txt",
                Some(Content),
            ),
            (
                "PUT",
                "/service/local/repo%5Fgroups/group/content/a.txt",
                Some(Content),
            ),
            ("GET", "/%72epository/raw/a.txt", Some(Content)),
            (
                "GET",
                "/service/local/other%2Frepositories/x",
                Some(Management),
            ),
            ("GET", "/service/local/repositories%2Fraw/a", Some(Content)),
            ("GET", "/service/local/%zz/repositories/a", Some(Management)),
            ("GET", "/repository/%ff/a.txt", Some(Management)),
            ("GET", "/repository/a%00b", Some(Management)),
        ];
        for (method, path, expected) in table {
            assert_eq!(surface_of(method, path), expected, "{method} {path}");
        }
    }

    #[test]
    fn a_stored_list_error_tells_the_operator_how_to_recover() {
        let text = Error::IpAllow("management: bad".to_owned()).to_string();
        assert!(text.contains("management: bad"));
        assert!(text.contains("stop the server and run `astrum ip-allow reset`"));
    }

    #[test]
    fn an_empty_list_allows_every_address_and_a_listed_one_only_its_networks() {
        let compiled = settings(&["10.0.0.0/8", "2001:db8::/32"], &[], &["192.0.2.7"])
            .compile()
            .unwrap();
        assert!(compiled.restricts(Surface::Management));
        assert!(!compiled.restricts(Surface::Content));
        for address in [ip("10.1.2.3"), ip("2001:db8::5"), ip("::ffff:10.9.9.9")] {
            assert!(
                compiled.permits(Surface::Management, address),
                "{address:?}"
            );
        }
        for address in [ip("11.0.0.1"), ip("2001:db9::1"), None] {
            assert!(
                !compiled.permits(Surface::Management, address),
                "{address:?}"
            );
        }
        assert!(compiled.permits(Surface::Content, ip("203.0.113.9")));
        assert!(compiled.permits(Surface::Content, None));
        assert!(compiled.permits(Surface::Agents, ip("192.0.2.7")));
        assert!(!compiled.permits(Surface::Agents, ip("192.0.2.8")));
    }

    #[test]
    fn mapped_ranges_are_stored_as_ipv4_and_too_short_ones_are_refused() {
        let compiled = settings(&["::ffff:10.0.0.0/104"], &[], &[])
            .compile()
            .unwrap();
        assert!(compiled.permits(Surface::Management, ip("10.2.3.4")));
        assert!(!compiled.permits(Surface::Management, ip("11.2.3.4")));
        let problems = settings(&["::ffff:10.0.0.0/64"], &[], &[])
            .compile()
            .unwrap_err();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].surface, Surface::Management);
        assert!(problems[0].message.contains("write it as an IPv4 range"));
    }

    #[test]
    fn an_entry_that_matches_a_whole_address_family_is_refused() {
        let text = "an entry that matches a whole address family is not allowed; clear the list to allow everybody";
        for entry in [
            "0.0.0.0/0",
            "10.0.0.0/0",
            "::/0",
            "2001:db8::/0",
            "::ffff:0.0.0.0/96",
            "::ffff:10.0.0.0/96",
        ] {
            let problems = settings(&[entry], &[], &[]).compile().unwrap_err();
            assert_eq!(problems.len(), 1, "{entry}");
            assert!(problems[0].message.contains(text), "{entry}");
        }
        for entry in ["0.0.0.0/1", "::/1", "::ffff:0.0.0.0/97", "0.0.0.0", "::"] {
            assert!(settings(&[entry], &[], &[]).compile().is_ok(), "{entry}");
        }
    }

    #[test]
    fn invalid_entries_and_too_long_lists_are_reported_per_surface() {
        let problems = settings(&["10.0.0.0/33", ""], &["gateway"], &[])
            .compile()
            .unwrap_err();
        let surfaces: Vec<Surface> = problems.iter().map(|problem| problem.surface).collect();
        assert_eq!(
            surfaces,
            [Surface::Management, Surface::Management, Surface::Content]
        );
        let entries: Vec<String> = (0..MAX_ENTRIES)
            .map(|index| format!("10.0.{index}.1"))
            .collect();
        let refs: Vec<&str> = entries.iter().map(String::as_str).collect();
        assert!(settings(&[], &[], &refs).compile().is_ok());
        let mut over = refs.clone();
        over.push("192.0.2.1");
        let problems = settings(&[], &[], &over).compile().unwrap_err();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].surface, Surface::Agents);
    }
}
