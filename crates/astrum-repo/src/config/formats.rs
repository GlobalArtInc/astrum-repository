use serde::{Deserialize, Serialize};

use crate::RepositoryType;
use crate::config::problems::{Problems, blank_to_none, web_url};
use crate::config::{Check, Context};

pub const MAX_PORT: i32 = 65535;
pub const MAX_REPODATA_DEPTH: i32 = 5;
pub const DEFAULT_QUERY_CACHE_MAX_AGE: i32 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VersionPolicy {
    Release,
    Snapshot,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutPolicy {
    Strict,
    Permissive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentDisposition {
    Inline,
    Attachment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexType {
    Hub,
    Registry,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NugetVersion {
    V2,
    V3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConanVersion {
    V1,
    V2,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maven {
    pub version_policy: Option<VersionPolicy>,
    pub layout_policy: Option<LayoutPolicy>,
    pub content_disposition: Option<ContentDisposition>,
}

impl Check for Maven {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        if self.version_policy.is_none() {
            problems.add("versionPolicy", "must not be empty");
        }
        if self.layout_policy.is_none() {
            problems.add("layoutPolicy", "must not be empty");
        }
        self.content_disposition
            .get_or_insert(ContentDisposition::Attachment);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Docker {
    pub v1_enabled: Option<bool>,
    pub force_basic_auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_enabled: Option<bool>,
}

impl Docker {
    pub fn ports(&self) -> impl Iterator<Item = (&'static str, i32)> + '_ {
        [("httpPort", self.http_port), ("httpsPort", self.https_port)]
            .into_iter()
            .filter_map(|(field, port)| port.map(|port| (field, port)))
    }
}

impl Check for Docker {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        self.v1_enabled.get_or_insert(false);
        self.force_basic_auth.get_or_insert(true);
        problems.range("httpPort", self.http_port, 1, MAX_PORT);
        problems.range("httpsPort", self.https_port, 1, MAX_PORT);
        if self.http_port.is_some() && self.http_port == self.https_port {
            problems.add("httpsPort", "must differ from the HTTP port");
        }
        blank_to_none(&mut self.subdomain);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProxy {
    pub index_type: Option<IndexType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_url: Option<String>,
    pub cache_foreign_layers: Option<bool>,
    pub foreign_layer_url_whitelist: Option<Vec<String>>,
}

impl Check for DockerProxy {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        let index_type = *self.index_type.get_or_insert(IndexType::Registry);
        blank_to_none(&mut self.index_url);
        match &self.index_url {
            _ if index_type == IndexType::Registry => self.index_url = None,
            Some(url) if !web_url(url) => {
                problems.add("indexUrl", "must be an http or https URL with a host")
            }
            _ => {}
        }
        self.cache_foreign_layers.get_or_insert(false);
        self.foreign_layer_url_whitelist
            .get_or_insert_with(Vec::new)
            .retain(|pattern| !pattern.trim().is_empty());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apt {
    pub distribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_distribution: Option<bool>,
}

impl Check for Apt {
    fn check(&mut self, context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        blank_to_none(&mut self.distribution);
        match context.spec.kind {
            RepositoryType::Hosted => {
                problems.require_text("distribution", &self.distribution);
                self.flat = None;
                self.enforce_distribution = None;
            }
            _ => {
                problems.require("flat", &self.flat);
                self.enforce_distribution.get_or_insert(false);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeployPolicy {
    Strict,
    Permissive,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Yum {
    pub repodata_depth: Option<i32>,
    pub deploy_policy: Option<DeployPolicy>,
}

impl Check for Yum {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        problems.range("repodataDepth", self.repodata_depth, 0, MAX_REPODATA_DEPTH);
        self.repodata_depth.get_or_insert(0);
        self.deploy_policy.get_or_insert(DeployPolicy::Strict);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Npm {
    pub remove_quarantined: Option<bool>,
    pub remove_non_cataloged: Option<bool>,
}

impl Check for Npm {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.remove_quarantined.get_or_insert(false);
        self.remove_non_cataloged.get_or_insert(false);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NugetProxy {
    pub query_cache_item_max_age: Option<i32>,
    pub nuget_version: Option<NugetVersion>,
}

impl Check for NugetProxy {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        problems.range(
            "queryCacheItemMaxAge",
            self.query_cache_item_max_age,
            0,
            i32::MAX,
        );
        self.query_cache_item_max_age
            .get_or_insert(DEFAULT_QUERY_CACHE_MAX_AGE);
        self.nuget_version.get_or_insert(NugetVersion::V3);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Raw {
    pub content_disposition: Option<ContentDisposition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_query_parameters: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded_query_parameters: Option<Vec<String>>,
}

impl Check for Raw {
    fn check(&mut self, context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.content_disposition
            .get_or_insert(ContentDisposition::Attachment);
        if context.spec.kind != RepositoryType::Proxy {
            self.forward_query_parameters = None;
            self.excluded_query_parameters = None;
            return;
        }
        self.forward_query_parameters.get_or_insert(false);
        self.excluded_query_parameters
            .get_or_insert_with(Vec::new)
            .retain(|name| !name.trim().is_empty());
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Terraform {
    pub require_authentication: Option<bool>,
    pub registry_host: Option<String>,
    pub upstream_host: Option<String>,
}

impl Check for Terraform {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.require_authentication.get_or_insert(false);
        blank_to_none(&mut self.registry_host);
        blank_to_none(&mut self.upstream_host);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cargo {
    pub require_authentication: Option<bool>,
}

impl Check for Cargo {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.require_authentication.get_or_insert(false);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConanProxy {
    pub conan_version: Option<ConanVersion>,
}

impl Check for ConanProxy {
    fn check(&mut self, _context: &Context<'_>, _stored: Option<&Self>, _problems: &mut Problems) {
        self.conan_version.get_or_insert(ConanVersion::V2);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::config::Spec;

    fn check<T: Check + Serialize + serde::de::DeserializeOwned>(
        format: &str,
        kind: &str,
        value: Value,
    ) -> (Value, Vec<String>) {
        let mut section: T = serde_json::from_value(value).unwrap();
        let context = Context::new(Spec::from_api(format, kind).unwrap(), &str::to_owned);
        let mut problems = Problems::default();
        section.check(&context, None, &mut problems);
        let messages = problems
            .finish()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|problem| format!("{}: {}", problem.id, problem.message))
            .collect();
        (serde_json::to_value(section).unwrap(), messages)
    }

    #[test]
    fn maven_needs_its_policies() {
        let (_, messages) = check::<Maven>("maven", "hosted", json!({"versionPolicy": "RELEASE"}));
        assert_eq!(messages, ["PARAMETER layoutPolicy: must not be empty"]);
        let (maven, messages) = check::<Maven>(
            "maven",
            "hosted",
            json!({"versionPolicy": "SNAPSHOT", "layoutPolicy": "STRICT"}),
        );
        assert!(messages.is_empty());
        assert_eq!(
            maven,
            json!({"versionPolicy": "SNAPSHOT", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"})
        );
        assert!(serde_json::from_value::<Maven>(json!({"versionPolicy": "release"})).is_err());
    }

    #[test]
    fn docker_fills_defaults_and_bounds_ports() {
        let (docker, messages) = check::<Docker>("docker", "hosted", json!({}));
        assert!(messages.is_empty());
        assert_eq!(docker, json!({"v1Enabled": false, "forceBasicAuth": true}));
        let (docker, messages) = check::<Docker>(
            "docker",
            "group",
            json!({"httpPort": 5000, "httpsPort": 70000, "subdomain": "", "pathEnabled": true}),
        );
        assert_eq!(
            messages,
            ["PARAMETER httpsPort: must be less than or equal to 65535"]
        );
        assert_eq!(docker["pathEnabled"], true);
        assert!(docker.get("subdomain").is_none());
        let (_, messages) = check::<Docker>(
            "docker",
            "hosted",
            json!({"httpPort": 5000, "httpsPort": 5000}),
        );
        assert_eq!(
            messages,
            ["PARAMETER httpsPort: must differ from the HTTP port"]
        );
    }

    #[test]
    fn a_docker_proxy_keeps_an_index_url_only_off_the_registry() {
        let (proxy, _) = check::<DockerProxy>(
            "docker",
            "proxy",
            json!({"indexUrl": "https://index.example.org/"}),
        );
        assert_eq!(
            proxy,
            json!({"indexType": "REGISTRY", "cacheForeignLayers": false, "foreignLayerUrlWhitelist": []})
        );
        let (proxy, _) = check::<DockerProxy>(
            "docker",
            "proxy",
            json!({"indexType": "HUB", "indexUrl": "https://index.example.org/"}),
        );
        assert_eq!(proxy["indexUrl"], "https://index.example.org/");
        let (_, messages) = check::<DockerProxy>(
            "docker",
            "proxy",
            json!({"indexType": "CUSTOM", "indexUrl": "not a url"}),
        );
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn apt_differs_between_hosted_and_proxy() {
        let (_, messages) = check::<Apt>("apt", "hosted", json!({"flat": true}));
        assert_eq!(messages, ["PARAMETER distribution: must not be empty"]);
        let (apt, _) = check::<Apt>(
            "apt",
            "hosted",
            json!({"distribution": "bookworm", "flat": true}),
        );
        assert_eq!(apt, json!({"distribution": "bookworm"}));
        let (_, messages) = check::<Apt>("apt", "proxy", json!({}));
        assert_eq!(messages, ["PARAMETER flat: must not be null"]);
        let (apt, _) = check::<Apt>("apt", "proxy", json!({"flat": false}));
        assert_eq!(
            apt,
            json!({"distribution": null, "flat": false, "enforceDistribution": false})
        );
    }

    #[test]
    fn small_format_sections_have_defaults() {
        assert_eq!(
            check::<Yum>("yum", "hosted", json!({})).0,
            json!({"repodataDepth": 0, "deployPolicy": "STRICT"})
        );
        assert_eq!(
            check::<Yum>("yum", "hosted", json!({"repodataDepth": 6})).1,
            ["PARAMETER repodataDepth: must be less than or equal to 5"]
        );
        assert_eq!(
            check::<Npm>("npm", "proxy", json!({})).0,
            json!({"removeQuarantined": false, "removeNonCataloged": false})
        );
        assert_eq!(
            check::<NugetProxy>("nuget", "proxy", json!({})).0,
            json!({"queryCacheItemMaxAge": 3600, "nugetVersion": "V3"})
        );
        assert!(serde_json::from_value::<NugetProxy>(json!({"nugetVersion": "V4"})).is_err());
        assert_eq!(
            check::<Raw>("raw", "proxy", json!({})).0,
            json!({"contentDisposition": "ATTACHMENT", "forwardQueryParameters": false, "excludedQueryParameters": []})
        );
        for kind in ["hosted", "group"] {
            assert_eq!(
                check::<Raw>(
                    "raw",
                    kind,
                    json!({"forwardQueryParameters": true, "excludedQueryParameters": ["a"]})
                )
                .0,
                json!({"contentDisposition": "ATTACHMENT"})
            );
        }
        assert_eq!(
            check::<Terraform>("terraform", "group", json!({"registryHost": ""})).0,
            json!({"requireAuthentication": false, "registryHost": null, "upstreamHost": null})
        );
        assert_eq!(
            check::<Cargo>("cargo", "hosted", json!({})).0,
            json!({"requireAuthentication": false})
        );
        assert_eq!(
            check::<ConanProxy>("conan", "proxy", json!({})).0,
            json!({"conanVersion": "V2"})
        );
    }
}
