use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::chart::{self, Chart};

pub type Entries = BTreeMap<String, Vec<Chart>>;

#[derive(Clone, Debug, Default)]
pub struct Listing {
    pub entries: Entries,
    pub generated: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct IndexFile {
    #[serde(default)]
    pub entries: Option<BTreeMap<String, Option<Vec<Chart>>>>,
    #[serde(default)]
    pub generated: Option<String>,
}

#[derive(Serialize)]
struct Rendered<'a> {
    #[serde(rename = "apiVersion")]
    api_version: &'static str,
    entries: &'a Entries,
    generated: String,
}

pub fn timestamp(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

pub fn compare_versions(left: &Chart, right: &Chart) -> Ordering {
    match (left.semver(), right.semver()) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => left.version.cmp(&right.version),
    }
}

pub fn sort(entries: &mut Entries) {
    for charts in entries.values_mut() {
        charts.sort_by(|left, right| compare_versions(right, left));
    }
}

pub fn merge(listings: Vec<Listing>) -> Listing {
    let mut merged = Listing::default();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for listing in listings {
        merged.generated = merged.generated.max(listing.generated);
        for (name, charts) in listing.entries {
            for chart in charts {
                if seen.insert((name.clone(), chart.version.clone())) {
                    merged.entries.entry(name.clone()).or_default().push(chart);
                }
            }
        }
    }
    sort(&mut merged.entries);
    merged
}

pub fn render(listing: &Listing) -> Result<String, serde_norway::Error> {
    serde_norway::to_string(&Rendered {
        api_version: "v1",
        entries: &listing.entries,
        generated: timestamp(listing.generated.unwrap_or_else(Utc::now)),
    })
}

#[derive(Clone, Debug)]
pub struct Located {
    pub urls: Vec<url::Url>,
    pub chart: Chart,
}

#[derive(Debug, Default)]
pub struct Upstream {
    pub listing: Listing,
    pub files: HashMap<String, Located>,
    pub skipped: usize,
}

fn fetchable(url: &url::Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

pub fn parse_upstream(body: &[u8], base: &url::Url) -> Result<Upstream, serde_norway::Error> {
    let file: IndexFile = serde_norway::from_slice(body)?;
    let mut upstream = Upstream {
        listing: Listing {
            generated: file
                .generated
                .as_deref()
                .and_then(|text| DateTime::parse_from_rfc3339(text.trim()).ok())
                .map(|time| time.with_timezone(&Utc)),
            ..Default::default()
        },
        ..Default::default()
    };
    for (name, charts) in file.entries.unwrap_or_default() {
        for mut chart in charts.unwrap_or_default() {
            if chart.name.is_empty() {
                chart.name = name.clone();
            }
            if chart.name != name
                || !chart::is_valid_name(&chart.name)
                || !chart::is_valid_version(&chart.version)
            {
                upstream.skipped += 1;
                continue;
            }
            let file = chart.file_name();
            if upstream.files.contains_key(&file) {
                continue;
            }
            let urls: Vec<url::Url> = chart
                .urls
                .iter()
                .filter_map(|url| base.join(url.trim()).ok())
                .filter(fetchable)
                .collect();
            if urls.is_empty() {
                upstream.skipped += 1;
                continue;
            }
            upstream.files.insert(
                file.clone(),
                Located {
                    urls,
                    chart: chart.metadata(),
                },
            );
            chart.urls = vec![file];
            upstream
                .listing
                .entries
                .entry(name.clone())
                .or_default()
                .push(chart);
        }
    }
    sort(&mut upstream.listing.entries);
    Ok(upstream)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(name: &str, version: &str) -> Chart {
        Chart {
            name: name.to_owned(),
            version: version.to_owned(),
            urls: vec![chart::file_name(name, version)],
            ..Default::default()
        }
    }

    const UPSTREAM: &str = r#"apiVersion: v1
entries:
  demo:
  - apiVersion: v2
    name: demo
    version: 0.2.0
    created: "2026-10-01T10:00:00Z"
    digest: aa
    urls:
    - https://files.example.org/releases/demo-0.2.0.tgz
  - name: demo
    version: 0.10.0
    urls:
    - charts/demo-0.10.0.tgz
    - https://mirror.example.org/demo-0.10.0.tgz
  - name: demo
    version: 0.1.0
    urls:
    - /abs/demo-0.1.0.tgz
  - name: demo
    version: 0.0.1
    urls:
    - ftp://files.example.org/demo-0.0.1.tgz
  - name: other
    version: 1.0.0
    urls:
    - other-1.0.0.tgz
  nameless:
  - version: 2.0.0
    urls: [nameless-2.0.0.tgz]
  empty:
generated: "2026-10-02T11:00:00.5Z"
serverInfo: {}
"#;

    #[test]
    fn upstream_urls_are_resolved_and_rewritten() {
        let base = url::Url::parse("https://charts.example.org/stable/index.yaml").unwrap();
        let upstream = parse_upstream(UPSTREAM.as_bytes(), &base).unwrap();
        let demo = &upstream.listing.entries["demo"];
        let versions: Vec<&str> = demo.iter().map(|chart| chart.version.as_str()).collect();
        assert_eq!(versions, ["0.10.0", "0.2.0", "0.1.0"]);
        assert!(demo.iter().all(|chart| chart.urls == [chart.file_name()]));
        assert_eq!(demo[1].digest.as_deref(), Some("aa"));
        let located = |file: &str| -> Vec<String> {
            upstream.files[file]
                .urls
                .iter()
                .map(|url| url.to_string())
                .collect()
        };
        assert_eq!(
            located("demo-0.2.0.tgz"),
            ["https://files.example.org/releases/demo-0.2.0.tgz"]
        );
        assert_eq!(
            located("demo-0.10.0.tgz"),
            [
                "https://charts.example.org/stable/charts/demo-0.10.0.tgz",
                "https://mirror.example.org/demo-0.10.0.tgz"
            ]
        );
        assert_eq!(
            located("demo-0.1.0.tgz"),
            ["https://charts.example.org/abs/demo-0.1.0.tgz"]
        );
        assert!(!upstream.files.contains_key("demo-0.0.1.tgz"));
        assert!(!upstream.files.contains_key("other-1.0.0.tgz"));
        assert!(upstream.files.contains_key("nameless-2.0.0.tgz"));
        assert_eq!(upstream.skipped, 2);
        assert_eq!(
            upstream.listing.generated.map(timestamp).as_deref(),
            Some("2026-10-02T11:00:00.500Z")
        );
    }

    #[test]
    fn broken_upstream_indexes_are_errors() {
        let base = url::Url::parse("https://charts.example.org/index.yaml").unwrap();
        assert!(parse_upstream(b"entries: [1, 2", &base).is_err());
        assert!(parse_upstream(b"entries:\n  a:\n  - maintainers: 3\n", &base).is_err());
        let empty = parse_upstream(b"apiVersion: v1\n", &base).unwrap();
        assert!(empty.listing.entries.is_empty());
    }

    #[test]
    fn groups_keep_the_first_member_per_version() {
        let mut first = chart("demo", "1.0.0");
        first.description = Some("first".to_owned());
        let mut second = chart("demo", "1.0.0");
        second.description = Some("second".to_owned());
        let merged = merge(vec![
            Listing {
                entries: [("demo".to_owned(), vec![first])].into(),
                generated: None,
            },
            Listing {
                entries: [
                    (
                        "demo".to_owned(),
                        vec![second, chart("demo", "2.0.0-rc.1"), chart("demo", "2.0.0")],
                    ),
                    ("other".to_owned(), vec![chart("other", "latest")]),
                ]
                .into(),
                generated: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                    .ok()
                    .map(|time| time.with_timezone(&Utc)),
            },
        ]);
        let demo = &merged.entries["demo"];
        let versions: Vec<&str> = demo.iter().map(|chart| chart.version.as_str()).collect();
        assert_eq!(versions, ["2.0.0", "2.0.0-rc.1", "1.0.0"]);
        assert_eq!(demo[2].description.as_deref(), Some("first"));
        assert_eq!(merged.entries["other"].len(), 1);
        assert_eq!(
            merged.generated.map(timestamp).as_deref(),
            Some("2026-01-01T00:00:00Z")
        );
    }

    #[test]
    fn indexes_render_as_helm_reads_them() {
        let mut demo = chart("demo", "1.10.0");
        demo.api_version = Some("v2".to_owned());
        demo.app_version = Some("1.16".to_owned());
        demo.created = Some("2026-10-12T19:31:18.212Z".to_owned());
        demo.digest = Some("ab".repeat(32));
        let listing = Listing {
            entries: [("demo".to_owned(), vec![demo])].into(),
            generated: DateTime::parse_from_rfc3339("2026-10-12T19:31:18.212Z")
                .ok()
                .map(|time| time.with_timezone(&Utc)),
        };
        let yaml = render(&listing).unwrap();
        assert_eq!(
            yaml,
            format!(
                "apiVersion: v1\nentries:\n  demo:\n  - apiVersion: v2\n    appVersion: '1.16'\n    created: 2026-10-12T19:31:18.212Z\n    digest: {}\n    name: demo\n    urls:\n    - demo-1.10.0.tgz\n    version: 1.10.0\ngenerated: 2026-10-12T19:31:18.212Z\n",
                "ab".repeat(32)
            )
        );
        let back: IndexFile = serde_norway::from_str(&yaml).unwrap();
        let entries = back.entries.unwrap();
        assert_eq!(
            entries["demo"].as_ref().unwrap()[0].app_version.as_deref(),
            Some("1.16")
        );
        let empty = render(&Listing::default()).unwrap();
        assert!(
            empty.starts_with("apiVersion: v1\nentries: {}\ngenerated: "),
            "{empty}"
        );
    }
}
