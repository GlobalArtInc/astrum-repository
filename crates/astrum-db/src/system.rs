use serde::{Deserialize, Serialize};

pub const BASE_URL_KEY: &str = "system.baseUrl";
pub const UI_SETTINGS_KEY: &str = "ui.settings";
pub const BRANDING_KEY: &str = "ui.branding";
pub const DEFAULT_TITLE: &str = "GlobalArt Astrum Repository";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BaseUrl {
    pub url: String,
    pub enabled: bool,
    pub notes: String,
}

impl Default for BaseUrl {
    fn default() -> Self {
        Self {
            url: String::new(),
            enabled: true,
            notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UiSettings {
    pub title: String,
    pub debug_allowed: bool,
    pub status_interval_authenticated: u32,
    pub status_interval_anonymous: u32,
    pub request_timeout: u32,
    pub long_request_timeout: u32,
    pub enabled: bool,
    pub notes: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            title: DEFAULT_TITLE.to_owned(),
            debug_allowed: true,
            status_interval_authenticated: 5,
            status_interval_anonymous: 60,
            request_timeout: 60,
            long_request_timeout: 180,
            enabled: true,
            notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Branding {
    pub header_enabled: bool,
    pub header_html: String,
    pub footer_enabled: bool,
    pub footer_html: String,
    pub enabled: bool,
    pub notes: String,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            header_enabled: true,
            header_html: String::new(),
            footer_enabled: true,
            footer_html: String::new(),
            enabled: true,
            notes: String::new(),
        }
    }
}

pub const PROMETHEUS_KEY: &str = "monitoring.prometheus";
pub const SPACE_ALERT_KEY: &str = "monitoring.blobStoreSpace";
pub const SPACE_ALERT_STATE_KEY: &str = "monitoring.blobStoreSpace.state";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prometheus {
    pub repositories: bool,
    pub enabled: bool,
    pub notes: String,
}

impl Default for Prometheus {
    fn default() -> Self {
        Self {
            repositories: true,
            enabled: true,
            notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpaceAlert {
    pub minimum_free_gigabytes: u32,
    pub interval_minutes: u32,
    pub repeat_hours: u32,
    pub quota: bool,
    pub unavailable: bool,
    pub recovery: bool,
    pub channels: Vec<String>,
    pub enabled: bool,
    pub notes: String,
}

impl Default for SpaceAlert {
    fn default() -> Self {
        Self {
            minimum_free_gigabytes: 10,
            interval_minutes: 5,
            repeat_hours: 24,
            quota: true,
            unavailable: true,
            recovery: true,
            channels: Vec::new(),
            enabled: true,
            notes: String::new(),
        }
    }
}
