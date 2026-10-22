use url::form_urlencoded;

use crate::builds::Build;

const STABLE: &[&str] = &["", "stable", "default"];
const PLATFORMS: &[&str] = &["Windows", "MacOS", "Linux", "FreeBSD", "Unix"];
const ARCHITECTURES: &[&str] = &["X86", "X86_64", "ARM32", "ARM64"];

#[derive(Debug, Default)]
pub struct Params(Vec<(String, String)>);

impl Params {
    pub fn of(query: Option<&str>) -> Self {
        Params(
            form_urlencoded::parse(query.unwrap_or_default().as_bytes())
                .map(|(name, value)| (name.into_owned(), value.into_owned()))
                .collect(),
        )
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(known, value)| known == name && !value.trim().is_empty())
            .map(|(_, value)| value.trim())
    }

    pub fn all(&self, name: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(known, _)| known == name)
            .flat_map(|(_, value)| value.split(','))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect()
    }
}

pub fn channel_of(name: Option<&str>) -> String {
    let name = name.unwrap_or_default().trim().to_ascii_lowercase();
    match STABLE.contains(&name.as_str()) {
        true => String::new(),
        false => name,
    }
}

#[derive(Debug, Default, Clone)]
pub struct Query {
    pub build: Option<String>,
    pub parsed: Option<Build>,
    pub ids: Vec<String>,
    pub channel: String,
    pub os: Option<String>,
    pub arch: Option<String>,
}

fn known(list: &[&str], value: Option<&str>) -> Option<String> {
    let value = value?;
    list.iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(value))
        .map(|candidate| (*candidate).to_owned())
}

impl Query {
    pub fn new(params: &Params, path_channel: Option<&str>) -> Self {
        let build = params.get("build").map(str::to_owned);
        let channel = path_channel.or_else(|| params.get("channel"));
        Query {
            parsed: build.as_deref().and_then(Build::parse),
            build,
            ids: params
                .all("pluginId")
                .into_iter()
                .chain(params.all("xmlId"))
                .map(str::to_owned)
                .collect(),
            channel: channel_of(channel),
            os: known(PLATFORMS, params.get("os")),
            arch: known(ARCHITECTURES, params.get("arch")),
        }
    }

    pub fn wants(&self, id: &str) -> bool {
        self.ids.is_empty() || self.ids.iter().any(|wanted| wanted == id)
    }

    pub fn accepts(&self, since: Option<&str>, until: Option<&str>) -> bool {
        match &self.parsed {
            Some(build) => build.compatible(since, until),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_are_decoded_and_repeated_ids_are_collected() {
        let params = Params::of(Some(
            "build=IU-243.1&pluginId=a%20b&pluginId=c,d&uuid=x&channel=EAP&empty=",
        ));
        assert_eq!(params.get("build"), Some("IU-243.1"));
        assert_eq!(params.get("empty"), None);
        assert_eq!(params.get("missing"), None);
        assert_eq!(params.all("pluginId"), ["a b", "c", "d"]);
        let query = Query::new(&params, None);
        assert_eq!(query.channel, "eap");
        assert_eq!(query.os, None);
        assert!(query.wants("c"));
        assert!(!query.wants("e"));
        assert!(query.accepts(Some("243.0"), Some("243.*")));
        assert!(!query.accepts(Some("244"), None));
    }

    #[test]
    fn the_channel_of_the_path_wins_and_stable_is_the_empty_channel() {
        let params = Params::of(Some("channel=beta"));
        assert_eq!(Query::new(&params, Some("EAP")).channel, "eap");
        assert_eq!(Query::new(&params, None).channel, "beta");
        assert_eq!(Query::new(&Params::default(), Some("Stable")).channel, "");
        assert_eq!(channel_of(None), "");
    }

    #[test]
    fn platforms_and_architectures_are_normalised_or_dropped() {
        let query = Query::new(&Params::of(Some("os=macos&arch=arm64")), None);
        assert_eq!(query.os.as_deref(), Some("MacOS"));
        assert_eq!(query.arch.as_deref(), Some("ARM64"));
        let query = Query::new(&Params::of(Some("os=plan9&arch=")), None);
        assert_eq!((query.os, query.arch), (None, None));
    }

    #[test]
    fn an_unparsable_build_filters_nothing() {
        let query = Query::new(&Params::of(Some("build=whatever")), None);
        assert!(query.parsed.is_none());
        assert!(query.accepts(Some("999"), Some("1")));
        assert!(Query::new(&Params::default(), None).wants("anything"));
    }
}
