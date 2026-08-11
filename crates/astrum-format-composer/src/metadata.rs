use serde_json::{Map, Value, json};

pub const MINIFIED: &str = "composer/2.0";
const UNSET: &str = "__unset";

pub fn valid_part(part: &str) -> bool {
    !part.is_empty()
        && part.len() <= 128
        && part
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
        && !part.starts_with('.')
}

pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 128
        && !version.contains('/')
        && !version.starts_with('.')
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+'))
}

pub fn dist_path(vendor: &str, project: &str, version: &str) -> String {
    format!("/{vendor}/{project}/{version}/{vendor}-{project}-{version}.zip")
}

pub fn expand(entries: &[Value]) -> Vec<Value> {
    let mut expanded = Vec::with_capacity(entries.len());
    let mut previous: Option<Map<String, Value>> = None;
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let mut current = previous.clone().unwrap_or_default();
        for (key, value) in object {
            if value.as_str() == Some(UNSET) {
                current.remove(key);
            } else {
                current.insert(key.clone(), value.clone());
            }
        }
        expanded.push(Value::Object(current.clone()));
        previous = Some(current);
    }
    expanded
}

pub fn minify(entries: &[Value]) -> Vec<Value> {
    let mut minified = Vec::with_capacity(entries.len());
    let mut previous: Option<&Map<String, Value>> = None;
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };
        match previous {
            None => minified.push(entry.clone()),
            Some(before) => {
                let mut changed = Map::new();
                for (key, value) in object {
                    if before.get(key) != Some(value) {
                        changed.insert(key.clone(), value.clone());
                    }
                }
                for key in before.keys() {
                    if !object.contains_key(key) {
                        changed.insert(key.clone(), json!(UNSET));
                    }
                }
                minified.push(Value::Object(changed));
            }
        }
        previous = Some(object);
    }
    minified
}

pub fn version_key(version: &str) -> (Vec<u64>, String) {
    let bare = version.trim_start_matches(['v', 'V']);
    let (numbers, rest) = match bare.find(|c: char| !(c.is_ascii_digit() || c == '.')) {
        Some(at) => (&bare[..at], &bare[at..]),
        None => (bare, ""),
    };
    let parts = numbers
        .split('.')
        .filter(|part| !part.is_empty())
        .map(|part| part.parse().unwrap_or(0))
        .collect();
    (parts, rest.to_owned())
}

pub fn newest_first(entries: &mut [Value]) {
    entries.sort_by(|left, right| {
        let left = version_key(
            left.get("version")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        let right = version_key(
            right
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        );
        let stable = |key: &(Vec<u64>, String)| key.1.is_empty();
        right
            .0
            .cmp(&left.0)
            .then_with(|| stable(&right).cmp(&stable(&left)))
            .then_with(|| right.1.cmp(&left.1))
    });
}

pub fn is_dev(version: &str) -> bool {
    version.starts_with("dev-") || version.ends_with("-dev")
}

pub fn rewrite_dists(document: &mut Value, rewrite: &dyn Fn(&str, &str, &str) -> Option<String>) {
    let Some(packages) = document.get_mut("packages").and_then(Value::as_object_mut) else {
        return;
    };
    for (package, entries) in packages.iter_mut() {
        let Some(entries) = entries.as_array_mut() else {
            continue;
        };
        let mut version = String::new();
        for entry in entries.iter_mut() {
            if let Some(found) = entry.get("version").and_then(Value::as_str) {
                version = found.to_owned();
            }
            let Some(dist) = entry.get_mut("dist").and_then(Value::as_object_mut) else {
                continue;
            };
            let original = dist
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if let Some(url) = rewrite(package, &version, &original) {
                dist.insert("url".to_owned(), json!(url));
            }
        }
    }
}

pub fn dist_of(document: &Value, package: &str, version: &str) -> Option<(String, Option<String>)> {
    let entries = document.get("packages")?.get(package)?.as_array()?;
    let entries = match document.get("minified").and_then(Value::as_str) {
        Some(MINIFIED) => expand(entries),
        _ => entries.clone(),
    };
    let entry = entries
        .iter()
        .find(|entry| entry.get("version").and_then(Value::as_str) == Some(version))?;
    let dist = entry.get("dist")?;
    let url = dist.get("url")?.as_str()?.to_owned();
    let shasum = dist
        .get("shasum")
        .and_then(Value::as_str)
        .filter(|shasum| !shasum.is_empty())
        .map(str::to_owned);
    Some((url, shasum))
}

pub fn entries_of(document: &Value, package: &str) -> Vec<Value> {
    let Some(entries) = document
        .get("packages")
        .and_then(|packages| packages.get(package))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    match document.get("minified").and_then(Value::as_str) {
        Some(MINIFIED) => expand(entries),
        _ => entries.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minified_lists_expand_and_minify_back() {
        let entries = vec![
            json!({"name": "a/b", "version": "2.0.0", "require": {"php": ">=8"}, "dist": {"url": "u2"}}),
            json!({"name": "a/b", "version": "1.0.0", "dist": {"url": "u1"}}),
        ];
        let minified = minify(&entries);
        assert_eq!(
            minified[1],
            json!({"version": "1.0.0", "dist": {"url": "u1"}, "require": "__unset"})
        );
        assert_eq!(expand(&minified), entries);
        let document = json!({"packages": {"a/b": minified}, "minified": MINIFIED});
        assert_eq!(
            dist_of(&document, "a/b", "1.0.0"),
            Some(("u1".to_owned(), None))
        );
    }

    #[test]
    fn versions_sort_newest_first_and_dists_are_rewritten() {
        let mut entries = vec![
            json!({"version": "1.10.0"}),
            json!({"version": "1.9.0"}),
            json!({"version": "2.0.0-RC1"}),
            json!({"version": "2.0.0"}),
        ];
        newest_first(&mut entries);
        let order: Vec<&str> = entries
            .iter()
            .map(|entry| entry["version"].as_str().unwrap())
            .collect();
        assert_eq!(order, ["2.0.0", "2.0.0-RC1", "1.10.0", "1.9.0"]);
        let mut document = json!({"packages": {"a/b": [
            {"name": "a/b", "version": "1.0.0", "dist": {"url": "https://github.example/1"}},
            {"version": "0.9.0", "dist": {"url": "https://github.example/0"}}
        ]}, "minified": MINIFIED});
        rewrite_dists(&mut document, &|package, version, _| {
            Some(format!("http://h/{package}/{version}.zip"))
        });
        assert_eq!(
            document["packages"]["a/b"][1]["dist"]["url"],
            "http://h/a/b/0.9.0.zip"
        );
        assert!(valid_part("monolog"));
        assert!(!valid_part("Monolog"));
        assert!(valid_version("1.0.0-beta1"));
        assert!(!valid_version("../1"));
        assert_eq!(dist_path("a", "b", "1.0"), "/a/b/1.0/a-b-1.0.zip");
    }
}
