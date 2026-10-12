use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::hosted::{self, Stored};
use crate::route::param;
use crate::urls::Urls;

pub const DEFAULT_TAKE: usize = 20;
pub const MAX_TAKE: usize = 1000;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub q: String,
    pub skip: usize,
    pub take: usize,
    pub prerelease: bool,
    pub semver2: bool,
    pub package_type: Option<String>,
    pub id: Option<String>,
}

fn flag(pairs: &[(String, String)], name: &str) -> bool {
    param(pairs, name).is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn number(pairs: &[(String, String)], name: &str) -> Option<usize> {
    param(pairs, name).and_then(|value| value.trim().parse().ok())
}

impl Query {
    pub fn parse(pairs: &[(String, String)]) -> Self {
        Query {
            q: param(pairs, "q").unwrap_or_default().trim().to_owned(),
            skip: number(pairs, "skip").unwrap_or(0),
            take: number(pairs, "take").unwrap_or(DEFAULT_TAKE).min(MAX_TAKE),
            prerelease: flag(pairs, "prerelease"),
            semver2: param(pairs, "semVerLevel").is_some_and(|level| level.trim().starts_with('2')),
            package_type: param(pairs, "packageType").filter(|value| !value.trim().is_empty()),
            id: param(pairs, "id").filter(|value| !value.trim().is_empty()),
        }
    }

    pub fn terms(&self) -> Vec<Term> {
        terms(&self.q)
    }

    pub fn free_text(&self) -> Vec<String> {
        self.terms()
            .into_iter()
            .filter_map(|term| match term {
                Term::Any(text) => Some(text),
                Term::Field(field, text) if field != "version" => Some(text),
                _ => None,
            })
            .collect()
    }

    pub fn wants(&self, stored: &Stored) -> bool {
        let metadata = &stored.metadata;
        (self.prerelease || !metadata.is_prerelease)
            && (self.semver2 || !metadata.semver2)
            && self.package_type.as_ref().is_none_or(|wanted| {
                metadata
                    .package_types
                    .iter()
                    .any(|kind| kind.eq_ignore_ascii_case(wanted))
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Any(String),
    Field(String, String),
}

pub fn terms(q: &str) -> Vec<Term> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut words: Vec<String> = Vec::new();
    for c in q.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    for word in words {
        match word.split_once(':') {
            Some((field, value)) if !value.is_empty() && is_field(field) => found.push(
                Term::Field(field.to_ascii_lowercase(), value.to_lowercase()),
            ),
            _ => found.push(Term::Any(word.to_lowercase())),
        }
    }
    found
}

fn is_field(field: &str) -> bool {
    matches!(
        field.to_ascii_lowercase().as_str(),
        "id" | "packageid"
            | "version"
            | "title"
            | "tag"
            | "tags"
            | "author"
            | "authors"
            | "description"
            | "summary"
            | "owner"
            | "owners"
    )
}

fn contains(value: &Option<String>, term: &str) -> bool {
    value
        .as_deref()
        .is_some_and(|value| value.to_lowercase().contains(term))
}

pub fn matches(stored: &Stored, terms: &[Term]) -> bool {
    let metadata = &stored.metadata;
    let id = metadata.id.to_lowercase();
    let tags = |term: &str| {
        metadata
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(term))
    };
    terms.iter().all(|term| match term {
        Term::Field(field, value) => match field.as_str() {
            "packageid" => id == *value,
            "id" => id.contains(value.as_str()),
            "version" => metadata.normalized_version.to_lowercase() == *value,
            "title" => contains(&metadata.title, value),
            "tag" | "tags" => tags(value),
            "author" | "authors" => contains(&metadata.authors, value),
            "description" => contains(&metadata.description, value),
            "summary" => contains(&metadata.summary, value),
            "owner" | "owners" => contains(&metadata.owners, value),
            _ => false,
        },
        Term::Any(value) => {
            id.contains(value.as_str())
                || contains(&metadata.title, value)
                || contains(&metadata.description, value)
                || contains(&metadata.summary, value)
                || contains(&metadata.authors, value)
                || tags(value)
        }
    })
}

fn rank(id: &str, terms: &[Term]) -> u8 {
    let id = id.to_lowercase();
    let words: Vec<&str> = terms
        .iter()
        .filter_map(|term| match term {
            Term::Any(value) => Some(value.as_str()),
            Term::Field(field, value) if field == "id" || field == "packageid" => {
                Some(value.as_str())
            }
            _ => None,
        })
        .collect();
    if words.iter().any(|word| id == *word) {
        0
    } else if words.iter().any(|word| id.starts_with(word)) {
        1
    } else if words.iter().any(|word| id.contains(word)) {
        2
    } else {
        3
    }
}

pub struct Group {
    pub id: String,
    pub versions: Vec<Stored>,
}

pub fn group(found: Vec<Stored>, query: &Query) -> Vec<Group> {
    let terms = query.terms();
    let mut groups: BTreeMap<String, Vec<Stored>> = BTreeMap::new();
    for stored in found {
        if query.wants(&stored) && matches(&stored, &terms) {
            groups
                .entry(stored.metadata.id.to_lowercase())
                .or_default()
                .push(stored);
        }
    }
    let mut groups: Vec<Group> = groups
        .into_values()
        .map(|mut versions| {
            versions.sort_by(hosted::compare);
            Group {
                id: versions
                    .last()
                    .map(|stored| stored.metadata.id.clone())
                    .unwrap_or_default(),
                versions,
            }
        })
        .collect();
    groups.sort_by(|left, right| {
        rank(&left.id, &terms)
            .cmp(&rank(&right.id, &terms))
            .then_with(|| downloads(right).cmp(&downloads(left)))
            .then_with(|| left.id.to_lowercase().cmp(&right.id.to_lowercase()))
    });
    groups
}

fn downloads(group: &Group) -> i64 {
    group.versions.iter().map(Stored::downloads).sum()
}

pub fn item(group: &Group, urls: &Urls) -> Value {
    let latest = group
        .versions
        .last()
        .expect("a search group holds at least one version");
    let metadata = &latest.metadata;
    let registration = urls.registration_index(&metadata.id);
    let versions: Vec<Value> = group
        .versions
        .iter()
        .map(|stored| {
            json!({
                "version": stored.metadata.version,
                "downloads": stored.downloads(),
                "@id": urls.registration_leaf(&stored.metadata.id, &stored.metadata.normalized_version),
            })
        })
        .collect();
    json!({
        "@id": registration,
        "@type": "Package",
        "registration": registration,
        "id": metadata.id,
        "version": metadata.version,
        "description": metadata.description.clone().unwrap_or_default(),
        "summary": metadata.summary.clone().unwrap_or_default(),
        "title": metadata.title.clone().unwrap_or_default(),
        "iconUrl": metadata.icon_url.clone().unwrap_or_default(),
        "licenseUrl": metadata.license_url.clone().unwrap_or_default(),
        "projectUrl": metadata.project_url.clone().unwrap_or_default(),
        "tags": metadata.tags,
        "authors": metadata.author_list(),
        "owners": metadata.owners.as_deref().map(|owners| owners.split(',').map(str::trim).filter(|owner| !owner.is_empty()).collect::<Vec<_>>()).unwrap_or_default(),
        "totalDownloads": downloads(group),
        "verified": false,
        "packageTypes": metadata.package_types.iter().map(|name| json!({"name": name})).collect::<Vec<_>>(),
        "versions": versions,
    })
}

#[derive(Clone, Debug, Default)]
pub struct Found {
    pub total: usize,
    pub data: Vec<Value>,
}

impl Found {
    pub fn to_json(&self) -> Value {
        json!({"totalHits": self.total, "data": self.data})
    }

    pub fn from_json(value: &Value) -> Found {
        let data = value
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Found {
            total: value
                .get("totalHits")
                .and_then(Value::as_u64)
                .map(|total| total as usize)
                .unwrap_or(data.len()),
            data,
        }
    }
}

pub fn hosted(found: Vec<Stored>, query: &Query, urls: &Urls) -> Found {
    let groups = group(found, query);
    Found {
        total: groups.len(),
        data: groups
            .iter()
            .skip(query.skip)
            .take(query.take)
            .map(|group| item(group, urls))
            .collect(),
    }
}

pub fn autocomplete_ids(found: Vec<Stored>, query: &Query) -> Found {
    let needle = query.q.to_lowercase();
    let mut groups = group(
        found,
        &Query {
            q: String::new(),
            ..query.clone()
        },
    );
    groups.retain(|group| needle.is_empty() || group.id.to_lowercase().contains(&needle));
    groups.sort_by(|left, right| {
        let left_starts = !left.id.to_lowercase().starts_with(&needle);
        let right_starts = !right.id.to_lowercase().starts_with(&needle);
        left_starts
            .cmp(&right_starts)
            .then_with(|| left.id.to_lowercase().cmp(&right.id.to_lowercase()))
    });
    Found {
        total: groups.len(),
        data: groups
            .iter()
            .skip(query.skip)
            .take(query.take)
            .map(|group| Value::String(group.id.clone()))
            .collect(),
    }
}

pub fn autocomplete_versions(mut found: Vec<Stored>, query: &Query) -> Found {
    found.retain(|stored| query.wants(stored));
    found.sort_by(hosted::compare);
    Found {
        total: found.len(),
        data: found
            .iter()
            .map(|stored| Value::String(stored.metadata.version.clone()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::stored;

    fn sample() -> Vec<Stored> {
        vec![
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>1.0.0</version><description>A probe</description><tags>tool</tags></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>1.1.0-beta.1</version><description>A probe</description></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>1.1.0-beta2</version><description>A probe</description></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Other</id><version>3.0.0</version><description>Uses astrum.probe</description><authors>Jane</authors></metadata></package>",
            ),
        ]
    }

    fn query(text: &str) -> Query {
        Query::parse(&crate::route::query_pairs(Some(text)))
    }

    #[test]
    fn queries_read_their_parameters() {
        let parsed = query("q=json&skip=2&take=5000&prerelease=true&semVerLevel=2.0.0");
        assert_eq!(parsed.q, "json");
        assert_eq!((parsed.skip, parsed.take), (2, MAX_TAKE));
        assert!(parsed.prerelease && parsed.semver2);
        let plain = query("");
        assert_eq!((plain.skip, plain.take), (0, DEFAULT_TAKE));
        assert!(!plain.prerelease && !plain.semver2);
        assert_eq!(
            terms("packageid:A.B \"two words\" Tags:x"),
            vec![
                Term::Field("packageid".to_owned(), "a.b".to_owned()),
                Term::Any("two words".to_owned()),
                Term::Field("tags".to_owned(), "x".to_owned())
            ]
        );
    }

    #[test]
    fn results_group_versions_by_id_and_hide_what_was_not_asked_for() {
        let urls = Urls::new("http://h/repository/n");
        let found = hosted(sample(), &query("q=probe"), &urls);
        assert_eq!(found.total, 2);
        assert_eq!(found.data[0]["id"], "Astrum.Probe");
        assert_eq!(found.data[0]["versions"].as_array().unwrap().len(), 1);
        assert_eq!(found.data[1]["id"], "Other");
        let all = hosted(
            sample(),
            &query("q=probe&prerelease=true&semVerLevel=2.0.0"),
            &urls,
        );
        assert_eq!(all.data[0]["version"], "1.1.0-beta2");
        assert_eq!(all.data[0]["versions"].as_array().unwrap().len(), 3);
        let semver1 = hosted(sample(), &query("q=probe&prerelease=true"), &urls);
        assert_eq!(semver1.data[0]["versions"].as_array().unwrap().len(), 2);
        let exact = hosted(sample(), &query("q=packageid:astrum.probe"), &urls);
        assert_eq!(exact.total, 1);
        let tagged = hosted(sample(), &query("q=tags:tool"), &urls);
        assert_eq!(tagged.total, 1);
        let author = hosted(sample(), &query("q=jane"), &urls);
        assert_eq!(author.data[0]["id"], "Other");
        let paged = hosted(sample(), &query("q=&skip=1&take=1"), &urls);
        assert_eq!((paged.total, paged.data.len()), (2, 1));
        assert_eq!(
            paged.data[0]["registration"],
            "http://h/repository/n/v3/registration/other/index.json"
        );
    }

    #[test]
    fn autocomplete_gives_ids_or_versions() {
        let ids = autocomplete_ids(sample(), &query("q=ot"));
        assert_eq!(ids.data, vec![Value::String("Other".to_owned())]);
        let everything = autocomplete_ids(sample(), &query("q="));
        assert_eq!(everything.total, 2);
        let versions = autocomplete_versions(
            sample()
                .into_iter()
                .filter(|stored| stored.metadata.id == "Astrum.Probe")
                .collect(),
            &query("id=astrum.probe&prerelease=true"),
        );
        assert_eq!(
            versions.data,
            vec![
                Value::String("1.0.0".to_owned()),
                Value::String("1.1.0-beta2".to_owned())
            ]
        );
    }
}
