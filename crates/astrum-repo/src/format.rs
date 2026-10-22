use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Hosted,
    Proxy,
    Group,
}

impl RepositoryType {
    pub const ALL: [RepositoryType; 3] = [Self::Hosted, Self::Proxy, Self::Group];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Proxy => "proxy",
            Self::Group => "group",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

impl fmt::Display for RepositoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub id: &'static str,
    pub api_name: &'static str,
    pub types: &'static [RepositoryType],
}

impl Format {
    pub fn supports(&self, kind: RepositoryType) -> bool {
        self.types.contains(&kind)
    }
}

use RepositoryType::{Group as G, Hosted as H, Proxy as P};

const HPG: &[RepositoryType] = &[H, P, G];
const HP: &[RepositoryType] = &[H, P];

pub const FORMATS: &[Format] = &[
    Format {
        id: "alpine",
        api_name: "alpine",
        types: HPG,
    },
    Format {
        id: "ansible",
        api_name: "ansible",
        types: HPG,
    },
    Format {
        id: "apt",
        api_name: "apt",
        types: HP,
    },
    Format {
        id: "bazel",
        api_name: "bazel",
        types: HPG,
    },
    Format {
        id: "cargo",
        api_name: "cargo",
        types: HPG,
    },
    Format {
        id: "chef",
        api_name: "chef",
        types: HPG,
    },
    Format {
        id: "cocoapods",
        api_name: "cocoapods",
        types: &[P],
    },
    Format {
        id: "composer",
        api_name: "composer",
        types: HPG,
    },
    Format {
        id: "conan",
        api_name: "conan",
        types: HPG,
    },
    Format {
        id: "conda",
        api_name: "conda",
        types: HPG,
    },
    Format {
        id: "docker",
        api_name: "docker",
        types: HPG,
    },
    Format {
        id: "gitlfs",
        api_name: "gitlfs",
        types: &[H],
    },
    Format {
        id: "go",
        api_name: "go",
        types: HPG,
    },
    Format {
        id: "helm",
        api_name: "helm",
        types: HPG,
    },
    Format {
        id: "hex",
        api_name: "hex",
        types: HPG,
    },
    Format {
        id: "huggingface",
        api_name: "huggingface",
        types: &[P],
    },
    Format {
        id: "jetbrains",
        api_name: "jetbrains",
        types: HPG,
    },
    Format {
        id: "maven2",
        api_name: "maven",
        types: HPG,
    },
    Format {
        id: "nix",
        api_name: "nix",
        types: HPG,
    },
    Format {
        id: "npm",
        api_name: "npm",
        types: HPG,
    },
    Format {
        id: "nuget",
        api_name: "nuget",
        types: HPG,
    },
    Format {
        id: "nx",
        api_name: "nx",
        types: &[H],
    },
    Format {
        id: "openvsx",
        api_name: "openvsx",
        types: HPG,
    },
    Format {
        id: "p2",
        api_name: "p2",
        types: &[P],
    },
    Format {
        id: "pacman",
        api_name: "pacman",
        types: HPG,
    },
    Format {
        id: "pub",
        api_name: "pub",
        types: HPG,
    },
    Format {
        id: "puppet",
        api_name: "puppet",
        types: HPG,
    },
    Format {
        id: "pypi",
        api_name: "pypi",
        types: HPG,
    },
    Format {
        id: "r",
        api_name: "r",
        types: HPG,
    },
    Format {
        id: "raw",
        api_name: "raw",
        types: HPG,
    },
    Format {
        id: "rubygems",
        api_name: "rubygems",
        types: HPG,
    },
    Format {
        id: "swift",
        api_name: "swift",
        types: HPG,
    },
    Format {
        id: "terraform",
        api_name: "terraform",
        types: HPG,
    },
    Format {
        id: "turbo",
        api_name: "turbo",
        types: &[H],
    },
    Format {
        id: "vagrant",
        api_name: "vagrant",
        types: HPG,
    },
    Format {
        id: "yum",
        api_name: "yum",
        types: HPG,
    },
];

pub fn format(id: &str) -> Option<&'static Format> {
    FORMATS.iter().find(|format| format.id == id)
}

pub fn format_by_api_name(name: &str) -> Option<&'static Format> {
    FORMATS.iter().find(|format| format.api_name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_are_sorted_and_unique() {
        let ids: Vec<&str> = FORMATS.iter().map(|format| format.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids, sorted);
        assert_eq!(ids.len(), 36);
    }

    #[test]
    fn maven_is_maven2_inside_and_maven_in_the_api() {
        assert_eq!(format_by_api_name("maven").unwrap().id, "maven2");
        assert_eq!(format("maven2").unwrap().api_name, "maven");
        assert!(format_by_api_name("maven2").is_none());
    }

    #[test]
    fn repository_types_round_trip() {
        for kind in RepositoryType::ALL {
            assert_eq!(RepositoryType::parse(kind.as_str()), Some(kind));
        }
        assert!(RepositoryType::parse("virtual").is_none());
        assert!(format("p2").unwrap().supports(RepositoryType::Proxy));
        assert!(!format("p2").unwrap().supports(RepositoryType::Hosted));
    }
}
