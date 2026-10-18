use serde::{Deserialize, Deserializer, Serialize};

use crate::config::problems::Problems;
use crate::config::{Check, Context};

pub const MAX_HOST_LENGTH: usize = 253;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DomainRouting {
    #[default]
    Root,
    Path,
}

impl DomainRouting {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            "ROOT" => Some(Self::Root),
            "PATH" => Some(Self::Path),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for DomainRouting {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "\"{value}\" is not a domain routing: use ROOT or PATH"
            ))
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub host: Option<String>,
    pub routing: Option<DomainRouting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl Domain {
    pub fn routing(&self) -> DomainRouting {
        self.routing.unwrap_or_default()
    }
}

pub const MAX_PATH_LENGTH: usize = 100;

pub fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= MAX_PATH_LENGTH
        && path != "."
        && path != ".."
        && !path.eq_ignore_ascii_case("v2")
        && !path.eq_ignore_ascii_case("repository")
        && !path.eq_ignore_ascii_case("service")
        && path.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

impl Check for Domain {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        self.host = self
            .host
            .as_deref()
            .map(normalize_host)
            .filter(|host| !host.is_empty());
        if let Some(host) = &self.host
            && !valid_host(host)
        {
            problems.add(
                "domainHost",
                format!("{host} is not a domain name such as docker.example.com"),
            );
        }
        self.routing.get_or_insert_with(DomainRouting::default);
        self.path = self
            .path
            .as_deref()
            .map(|path| path.trim().trim_matches('/').to_owned())
            .filter(|path| !path.is_empty() && self.routing == Some(DomainRouting::Path));
        if let Some(path) = &self.path
            && !valid_path(path)
        {
            problems.add(
                "domainPath",
                format!(
                    "{path} is not a path segment: use letters, digits, '-', '_' and '.', and not v2, repository or service"
                ),
            );
        }
    }
}

pub fn normalize_host(value: &str) -> String {
    value.trim().trim_end_matches('.').to_ascii_lowercase()
}

pub fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > MAX_HOST_LENGTH {
        return false;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    let label_ok = |label: &&str| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
    };
    labels.iter().all(label_ok)
        && labels
            .last()
            .is_some_and(|top| !top.chars().all(|character| character.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Spec;

    fn checked(host: Option<&str>, routing: Option<DomainRouting>) -> (Domain, Vec<String>) {
        let spec = Spec::from_api("docker", "hosted").unwrap();
        let context = Context::new(spec, &|plain| plain.to_owned());
        let mut problems = Problems::default();
        let mut domain = Domain {
            host: host.map(str::to_owned),
            routing,
            path: None,
        };
        domain.check(&context, None, &mut problems);
        let ids = problems
            .finish()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|problem| problem.id)
            .collect();
        (domain, ids)
    }

    #[test]
    fn hosts_are_lowercased_trimmed_and_checked() {
        let (domain, ids) = checked(Some(" Docker-Prem.GlobalArt.hu. "), None);
        assert!(ids.is_empty(), "{ids:?}");
        assert_eq!(domain.host.as_deref(), Some("docker-prem.globalart.hu"));
        assert_eq!(domain.routing, Some(DomainRouting::Root));

        for bad in [
            "localhost",
            "https://images.globalart.hu",
            "images.globalart.hu:8443",
            "-bad.example.com",
            "bad-.example.com",
            "under_score.example.com",
            "10.0.0.1",
            "a..example.com",
        ] {
            let (_, ids) = checked(Some(bad), None);
            assert_eq!(ids, ["PARAMETER domainHost"], "{bad}");
        }
    }

    #[test]
    fn a_blank_host_means_no_domain() {
        let (domain, ids) = checked(Some("   "), Some(DomainRouting::Path));
        assert!(ids.is_empty());
        assert_eq!(domain.host, None);
    }

    #[test]
    fn a_path_belongs_to_path_routing_and_is_one_segment() {
        let spec = Spec::from_api("docker", "hosted").unwrap();
        let context = Context::new(spec, &|plain| plain.to_owned());
        let check = |routing: DomainRouting, path: &str| {
            let mut problems = Problems::default();
            let mut domain = Domain {
                host: Some("images.globalart.hu".to_owned()),
                routing: Some(routing),
                path: Some(path.to_owned()),
            };
            domain.check(&context, None, &mut problems);
            let ids: Vec<String> = problems
                .finish()
                .err()
                .unwrap_or_default()
                .into_iter()
                .map(|problem| problem.id)
                .collect();
            (domain.path, ids)
        };
        assert_eq!(
            check(DomainRouting::Path, " /prod/ "),
            (Some("prod".to_owned()), Vec::new())
        );
        assert_eq!(check(DomainRouting::Root, "prod"), (None, Vec::new()));
        for bad in ["a/b", "v2", "Repository", "service", "..", "with space"] {
            assert_eq!(
                check(DomainRouting::Path, bad).1,
                ["PARAMETER domainPath"],
                "{bad}"
            );
        }
    }

    #[test]
    fn routing_reads_in_any_case() {
        let parsed: Domain =
            serde_json::from_str(r#"{"host":"images.globalart.hu","routing":"path"}"#).unwrap();
        assert_eq!(parsed.routing(), DomainRouting::Path);
        assert_eq!(serde_json::to_value(DomainRouting::Root).unwrap(), "ROOT");
        let error = serde_json::from_str::<Domain>(r#"{"routing":"SIDEWAYS"}"#).unwrap_err();
        assert!(error.to_string().contains("ROOT or PATH"), "{error}");
        assert_eq!(Domain::default().routing(), DomainRouting::Root);
    }
}
