use crate::config::{Section, WritePolicy};
use crate::format::{self, Format, RepositoryType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    Required,
    Defaulted,
    Nullable,
    Omitted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub format: &'static Format,
    pub kind: RepositoryType,
}

const WRITABLE_MEMBER: &[&str] = &["cargo", "docker", "swift"];
const PROXY_WRITE_POLICY: &[&str] = &["maven2", "npm", "raw"];

impl Spec {
    pub fn new(format: &'static Format, kind: RepositoryType) -> Option<Self> {
        format.supports(kind).then_some(Self { format, kind })
    }

    pub fn from_api(format: &str, kind: &str) -> Option<Self> {
        Self::new(
            format::format_by_api_name(format)?,
            RepositoryType::parse(kind)?,
        )
    }

    pub fn all() -> impl Iterator<Item = Spec> {
        format::FORMATS.iter().flat_map(|format| {
            format.types.iter().map(move |kind| Spec {
                format,
                kind: *kind,
            })
        })
    }

    pub fn is(&self, format: &str, kind: RepositoryType) -> bool {
        self.format.id == format && self.kind == kind
    }

    pub fn presence(&self, section: Section) -> Option<Presence> {
        use Presence::{Defaulted, Nullable, Omitted, Required};
        use RepositoryType::{Group, Hosted, Proxy};
        let format = self.format.id;
        let kind = self.kind;
        let when = |condition: bool, presence: Presence| condition.then_some(presence);
        match section {
            Section::Storage => Some(Required),
            Section::Cleanup => when(kind != Group, Nullable),
            Section::Component => when(kind == Hosted, Defaulted),
            Section::Proxy | Section::NegativeCache | Section::HttpClient => {
                when(kind == Proxy, Required)
            }
            Section::RoutingRuleName => match kind {
                Proxy => Some(Nullable),
                Group => Some(Omitted),
                Hosted => None,
            },
            Section::Firewall => when(kind == Proxy, Nullable),
            Section::Replication => when(kind == Proxy, Defaulted),
            Section::Group => when(kind == Group, Required),
            Section::Maven => when(format == "maven2" && kind != Group, Required),
            Section::Docker => when(format == "docker", Defaulted),
            Section::DockerProxy => when(format == "docker" && kind == Proxy, Defaulted),
            Section::Apt => when(format == "apt", Required),
            Section::AptSigning => match kind {
                Hosted => when(format == "apt", Required),
                _ => when(format == "apt", Nullable),
            },
            Section::Yum => when(format == "yum" && kind == Hosted, Defaulted),
            Section::YumSigning => when(format == "yum", Omitted),
            Section::Npm => when(format == "npm" && kind == Proxy, Defaulted),
            Section::NugetProxy => when(format == "nuget" && kind == Proxy, Defaulted),
            Section::Raw => when(format == "raw", Defaulted),
            Section::Terraform => when(format == "terraform", Defaulted),
            Section::TerraformSigning => when(format == "terraform" && kind == Hosted, Omitted),
            Section::Cargo => when(format == "cargo", Defaulted),
            Section::ConanProxy => when(format == "conan" && kind == Proxy, Defaulted),
            Section::AlpineSigning => when(format == "alpine", Nullable),
            Section::Nix => when(format == "nix", Defaulted),
            Section::HexSigning => when(format == "hex" && kind != Proxy, Defaulted),
            Section::PacmanSigning => when(format == "pacman", Nullable),
            Section::Domain => Some(Omitted),
        }
    }

    pub fn has_writable_member(&self) -> bool {
        self.kind == RepositoryType::Group && WRITABLE_MEMBER.contains(&self.format.id)
    }

    pub fn has_preemptive_authentication(&self) -> bool {
        self.is("maven2", RepositoryType::Proxy)
    }

    pub fn has_latest_policy(&self) -> bool {
        self.is("docker", RepositoryType::Hosted)
    }

    pub fn fixed_write_policy(&self) -> Option<WritePolicy> {
        let fixed = match self.kind {
            RepositoryType::Hosted => false,
            RepositoryType::Proxy => PROXY_WRITE_POLICY.contains(&self.format.id),
            RepositoryType::Group => self.format.id == "raw",
        };
        fixed.then_some(WritePolicy::Allow)
    }

    pub fn pem_signing_keys(&self) -> bool {
        matches!(self.format.id, "alpine" | "hex")
    }

    pub fn plain_signing_keys(&self) -> bool {
        self.format.id == "hex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(format: &str, kind: &str) -> Spec {
        Spec::from_api(format, kind).unwrap()
    }

    #[test]
    fn every_format_and_type_the_registry_offers_has_a_spec() {
        assert_eq!(Spec::all().count(), 102);
        assert!(Spec::from_api("maven", "hosted").is_some());
        assert!(Spec::from_api("maven2", "hosted").is_none());
        assert!(Spec::from_api("apt", "group").is_none());
        assert!(Spec::from_api("raw", "virtual").is_none());
        for spec in Spec::all() {
            assert_eq!(spec.presence(Section::Storage), Some(Presence::Required));
            for section in Section::ALL {
                if let Some(presence) = spec.presence(*section) {
                    assert!(
                        presence != Presence::Required
                            || matches!(
                                section,
                                Section::Storage
                                    | Section::Proxy
                                    | Section::NegativeCache
                                    | Section::HttpClient
                                    | Section::Group
                                    | Section::Maven
                                    | Section::Apt
                                    | Section::AptSigning
                            ),
                        "{section:?} in {spec:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn shared_sections_follow_the_type() {
        let hosted = spec("npm", "hosted");
        assert_eq!(
            hosted.presence(Section::Component),
            Some(Presence::Defaulted)
        );
        assert_eq!(hosted.presence(Section::Cleanup), Some(Presence::Nullable));
        assert_eq!(hosted.presence(Section::Proxy), None);
        let proxy = spec("npm", "proxy");
        assert_eq!(proxy.presence(Section::Component), None);
        assert_eq!(
            proxy.presence(Section::HttpClient),
            Some(Presence::Required)
        );
        assert_eq!(proxy.presence(Section::Npm), Some(Presence::Defaulted));
        assert_eq!(proxy.presence(Section::Firewall), Some(Presence::Nullable));
        let group = spec("npm", "group");
        assert_eq!(group.presence(Section::Group), Some(Presence::Required));
        assert_eq!(group.presence(Section::Cleanup), None);
        assert_eq!(group.presence(Section::Npm), None);
    }

    #[test]
    fn format_sections_belong_to_their_formats() {
        assert_eq!(
            spec("maven", "proxy").presence(Section::Maven),
            Some(Presence::Required)
        );
        assert_eq!(spec("maven", "group").presence(Section::Maven), None);
        assert_eq!(
            spec("apt", "hosted").presence(Section::AptSigning),
            Some(Presence::Required)
        );
        assert_eq!(
            spec("apt", "proxy").presence(Section::AptSigning),
            Some(Presence::Nullable)
        );
        assert_eq!(
            spec("yum", "group").presence(Section::YumSigning),
            Some(Presence::Omitted)
        );
        assert_eq!(spec("yum", "proxy").presence(Section::Yum), None);
        assert_eq!(
            spec("terraform", "proxy").presence(Section::TerraformSigning),
            None
        );
        assert_eq!(
            spec("docker", "group").presence(Section::Docker),
            Some(Presence::Defaulted)
        );
        assert_eq!(spec("docker", "group").presence(Section::DockerProxy), None);
        for kind in ["hosted", "proxy", "group"] {
            assert_eq!(
                spec("nix", kind).presence(Section::Nix),
                Some(Presence::Defaulted)
            );
        }
        assert_eq!(spec("raw", "hosted").presence(Section::Nix), None);
        assert_eq!(
            spec("hex", "hosted").presence(Section::HexSigning),
            Some(Presence::Defaulted)
        );
        assert_eq!(
            spec("hex", "group").presence(Section::HexSigning),
            Some(Presence::Defaulted)
        );
        assert_eq!(spec("hex", "proxy").presence(Section::HexSigning), None);
        assert_eq!(
            spec("pacman", "proxy").presence(Section::PacmanSigning),
            Some(Presence::Nullable)
        );
        assert_eq!(
            spec("alpine", "hosted").presence(Section::PacmanSigning),
            None
        );
    }

    #[test]
    fn small_rules_per_format() {
        assert!(spec("docker", "group").has_writable_member());
        assert!(spec("cargo", "group").has_writable_member());
        assert!(spec("swift", "group").has_writable_member());
        assert!(!spec("npm", "group").has_writable_member());
        assert!(spec("maven", "proxy").has_preemptive_authentication());
        assert!(!spec("raw", "proxy").has_preemptive_authentication());
        assert!(spec("docker", "hosted").has_latest_policy());
        assert_eq!(
            spec("maven", "proxy").fixed_write_policy(),
            Some(WritePolicy::Allow)
        );
        assert_eq!(
            spec("raw", "group").fixed_write_policy(),
            Some(WritePolicy::Allow)
        );
        assert_eq!(spec("pypi", "proxy").fixed_write_policy(), None);
        assert_eq!(spec("raw", "hosted").fixed_write_policy(), None);
        assert!(spec("alpine", "hosted").pem_signing_keys());
        assert!(spec("hex", "group").pem_signing_keys());
        assert!(spec("hex", "hosted").plain_signing_keys());
        assert!(!spec("alpine", "hosted").plain_signing_keys());
    }
}
