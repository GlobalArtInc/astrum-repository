mod document;
mod domain;
mod formats;
mod group;
mod problems;
mod proxy;
mod signing;
mod spec;
mod storage;

use serde::{Deserialize, Serialize};

use crate::format::{Format, RepositoryType};

pub use domain::{Domain, DomainRouting, normalize_host, valid_host, valid_path};
pub use formats::{
    Apt, Cargo, ConanProxy, ConanVersion, ContentDisposition, DEFAULT_NIX_PRIORITY,
    DEFAULT_NIX_STORE_DIR, DeployPolicy, Docker, DockerProxy, IndexType, LayoutPolicy, Maven, Nix,
    Npm, NugetProxy, NugetVersion, Raw, Terraform, VersionPolicy, Yum, valid_store_dir,
};
pub use group::Group;
pub use problems::{Problem, Problems};
pub use proxy::{
    Authentication, AuthenticationType, Connection, Firewall, FirewallMode, HttpClient,
    NegativeCache, Proxy, Replication,
};
pub use signing::{Signing, is_armored_pgp_key, is_pem_private_key};
pub use spec::{Presence, Spec};
pub use storage::{Cleanup, Component, Storage, WritePolicy};

pub const MAX_NAME_LENGTH: usize = 200;

pub struct Context<'a> {
    pub spec: Spec,
    seal: &'a dyn Fn(&str) -> String,
}

impl<'a> Context<'a> {
    pub fn new(spec: Spec, seal: &'a dyn Fn(&str) -> String) -> Self {
        Self { spec, seal }
    }

    pub fn secret(&self, given: Option<String>, stored: Option<&String>) -> Option<String> {
        match given.filter(|plain| !plain.is_empty()) {
            Some(plain) => Some((self.seal)(&plain)),
            None => stored.cloned(),
        }
    }
}

pub trait Check {
    fn check(&mut self, context: &Context<'_>, stored: Option<&Self>, problems: &mut Problems);
}

impl Check for String {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {}
}

macro_rules! sections {
    ($($variant:ident: $field:ident: $section:ty = $key:literal),* $(,)?) => {
        #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Attributes {
            $(
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub $field: Option<$section>,
            )*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Section {
            $($variant),*
        }

        impl Section {
            pub const ALL: &'static [Section] = &[$(Section::$variant),*];

            pub fn key(self) -> &'static str {
                match self {
                    $(Section::$variant => $key),*
                }
            }
        }

        impl Attributes {
            pub fn has(&self, section: Section) -> bool {
                match section {
                    $(Section::$variant => self.$field.is_some()),*
                }
            }

            pub fn value(&self, section: Section) -> Option<serde_json::Value> {
                match section {
                    $(Section::$variant => self
                        .$field
                        .as_ref()
                        .and_then(|value| serde_json::to_value(value).ok())),*
                }
            }

            fn remove(&mut self, section: Section) {
                match section {
                    $(Section::$variant => self.$field = None),*
                }
            }

            fn fill(&mut self, section: Section) {
                match section {
                    $(Section::$variant => {
                        self.$field.get_or_insert_with(Default::default);
                    }),*
                }
            }

            fn check_sections(
                &mut self,
                context: &Context<'_>,
                stored: Option<&Attributes>,
                problems: &mut Problems,
            ) {
                $(
                    if let Some(section) = &mut self.$field {
                        let stored = stored.and_then(|stored| stored.$field.as_ref());
                        section.check(context, stored, problems);
                    }
                )*
            }
        }
    };
}

sections! {
    Storage: storage: Storage = "storage",
    Cleanup: cleanup: Cleanup = "cleanup",
    Component: component: Component = "component",
    Proxy: proxy: Proxy = "proxy",
    NegativeCache: negative_cache: NegativeCache = "negativeCache",
    HttpClient: http_client: HttpClient = "httpClient",
    RoutingRuleName: routing_rule_name: String = "routingRuleName",
    Replication: replication: Replication = "replication",
    Firewall: firewall: Firewall = "firewall",
    Group: group: Group = "group",
    Maven: maven: Maven = "maven",
    Docker: docker: Docker = "docker",
    DockerProxy: docker_proxy: DockerProxy = "dockerProxy",
    Apt: apt: Apt = "apt",
    AptSigning: apt_signing: Signing = "aptSigning",
    Yum: yum: Yum = "yum",
    YumSigning: yum_signing: Signing = "yumSigning",
    Npm: npm: Npm = "npm",
    NugetProxy: nuget_proxy: NugetProxy = "nugetProxy",
    Raw: raw: Raw = "raw",
    Terraform: terraform: Terraform = "terraform",
    TerraformSigning: terraform_signing: Signing = "terraformSigning",
    Cargo: cargo: Cargo = "cargo",
    ConanProxy: conan_proxy: ConanProxy = "conanProxy",
    AlpineSigning: alpine_signing: Signing = "alpineSigning",
    Nix: nix: Nix = "nix",
    HexSigning: hex_signing: Signing = "hexSigning",
    PacmanSigning: pacman_signing: Signing = "pacmanSigning",
    Domain: domain: Domain = "domain",
}

impl Attributes {
    pub fn redacted(&self) -> Attributes {
        let mut attributes = self.clone();
        if let Some(authentication) = attributes
            .http_client
            .as_mut()
            .and_then(|client| client.authentication.as_mut())
        {
            authentication.redact();
        }
        for (_, signing) in attributes.signing_mut() {
            if let Some(signing) = signing {
                signing.redact();
            }
        }
        attributes
    }

    fn signing_mut(&mut self) -> [(Section, &mut Option<Signing>); 5] {
        [
            (Section::AptSigning, &mut self.apt_signing),
            (Section::YumSigning, &mut self.yum_signing),
            (Section::TerraformSigning, &mut self.terraform_signing),
            (Section::AlpineSigning, &mut self.alpine_signing),
            (Section::HexSigning, &mut self.hex_signing),
            (Section::PacmanSigning, &mut self.pacman_signing),
        ]
    }

    pub fn signing_of(&mut self, section: Section) -> Option<&mut Option<Signing>> {
        self.signing_mut()
            .into_iter()
            .find(|(candidate, _)| *candidate == section)
            .map(|(_, signing)| signing)
    }

    fn settle_signing(&mut self, spec: Spec, problems: &mut Problems) {
        for (section, signing) in self.signing_mut() {
            if signing
                .as_ref()
                .is_some_and(|signing| signing.keypair.is_none())
            {
                match spec.presence(section) {
                    Some(Presence::Required) => problems.add("keypair", "must not be empty"),
                    _ => *signing = None,
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: Option<String>,
    pub online: Option<bool>,
    #[serde(flatten)]
    pub attributes: Attributes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub name: String,
    pub format: &'static Format,
    pub kind: RepositoryType,
    pub online: bool,
    pub attributes: Attributes,
}

impl Repository {
    pub fn prepare(
        spec: Spec,
        request: Request,
        stored: Option<&Repository>,
        seal: &dyn Fn(&str) -> String,
    ) -> Result<Self, Vec<Problem>> {
        let mut problems = Problems::default();
        check_name(request.name.as_deref(), &mut problems);
        problems.require("online", &request.online);
        let mut attributes = request.attributes;
        problems::blank_to_none(&mut attributes.routing_rule_name);
        for section in Section::ALL {
            match spec.presence(*section) {
                None => attributes.remove(*section),
                Some(Presence::Required) if !attributes.has(*section) => {
                    problems.add(section.key(), "must not be null")
                }
                Some(Presence::Defaulted) => attributes.fill(*section),
                _ => {}
            }
        }
        let stored = stored
            .filter(|stored| stored.spec() == spec)
            .map(|stored| &stored.attributes);
        attributes.check_sections(&Context::new(spec, seal), stored, &mut problems);
        attributes.settle_signing(spec, &mut problems);
        if attributes
            .domain
            .as_ref()
            .is_some_and(|domain| domain.host.is_none())
        {
            attributes.domain = None;
        }
        problems.finish()?;
        Ok(Self {
            name: request.name.unwrap_or_default(),
            format: spec.format,
            kind: spec.kind,
            online: request.online.unwrap_or(true),
            attributes,
        })
    }

    pub fn spec(&self) -> Spec {
        Spec {
            format: self.format,
            kind: self.kind,
        }
    }

    pub fn blob_store(&self) -> Option<&str> {
        self.attributes
            .storage
            .as_ref()
            .and_then(|storage| storage.blob_store_name.as_deref())
    }

    pub fn group(&self) -> Option<&Group> {
        self.attributes.group.as_ref()
    }

    pub fn domain(&self) -> Option<(&str, DomainRouting)> {
        self.attributes
            .domain
            .as_ref()
            .and_then(|domain| Some((domain.host.as_deref()?, domain.routing())))
    }

    pub fn domain_path(&self) -> &str {
        self.attributes
            .domain
            .as_ref()
            .and_then(|domain| domain.path.as_deref())
            .unwrap_or(&self.name)
    }

    pub fn is_named(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

pub fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric() || first == '-')
        && chars.all(|next| next.is_ascii_alphanumeric() || matches!(next, '-' | '_' | '.'))
}

fn check_name(name: Option<&str>, problems: &mut Problems) {
    match name {
        None | Some("") => problems.add("name", "must not be empty"),
        Some(name) if !valid_name(name) => problems.add(
            "name",
            "may hold only letters, digits, '-', '_' and '.', and must not start with '_' or '.'",
        ),
        Some(name) if name.len() > MAX_NAME_LENGTH => problems.add(
            "name",
            format!("must be at most {MAX_NAME_LENGTH} characters long"),
        ),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
