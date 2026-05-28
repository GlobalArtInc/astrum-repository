use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use serde_json::{Value, json};

use crate::hosted::Stored;
use crate::urls::Urls;
use crate::version::Version;

pub const PAGE_SIZE: usize = 64;

pub fn time(value: DateTime<FixedOffset>) -> String {
    value
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

pub fn catalog_entry(stored: &Stored, urls: &Urls) -> Value {
    let metadata = &stored.metadata;
    let id = &metadata.id;
    let version = &metadata.normalized_version;
    let catalog = urls.catalog_leaf(id, version);
    let groups: Vec<Value> = metadata
        .ranges_normalized()
        .into_iter()
        .map(|group| {
            let framework = group.target_framework.clone().unwrap_or_default();
            let group_id = format!("{catalog}#dependencygroup/{}", framework.to_lowercase());
            let dependencies: Vec<Value> = group
                .dependencies
                .iter()
                .map(|dependency| {
                    json!({
                        "@id": format!("{group_id}/{}", dependency.id.to_lowercase()),
                        "@type": "PackageDependency",
                        "id": dependency.id,
                        "range": dependency.range,
                        "registration": urls.registration_index(&dependency.id),
                    })
                })
                .collect();
            let mut value = json!({
                "@id": group_id,
                "@type": "PackageDependencyGroup",
                "dependencies": dependencies,
            });
            if let Some(framework) = group.target_framework {
                value["targetFramework"] = Value::String(framework);
            }
            value
        })
        .collect();
    json!({
        "@id": catalog,
        "@type": "PackageDetails",
        "authors": text(&metadata.authors),
        "dependencyGroups": groups,
        "description": text(&metadata.description),
        "iconUrl": text(&metadata.icon_url),
        "id": id,
        "language": text(&metadata.language),
        "licenseExpression": text(&metadata.license_expression),
        "licenseUrl": text(&metadata.license_url),
        "listed": true,
        "minClientVersion": text(&metadata.min_client_version),
        "packageContent": urls.package_content(id, version),
        "packageTypes": metadata.package_types.iter().map(|name| json!({"name": name})).collect::<Vec<_>>(),
        "projectUrl": text(&metadata.project_url),
        "published": time(stored.published()),
        "requireLicenseAcceptance": metadata.require_license_acceptance,
        "summary": text(&metadata.summary),
        "tags": metadata.tags,
        "title": text(&metadata.title),
        "version": metadata.version,
    })
}

pub fn item(stored: &Stored, urls: &Urls) -> Value {
    let metadata = &stored.metadata;
    json!({
        "@id": urls.registration_leaf(&metadata.id, &metadata.normalized_version),
        "@type": "Package",
        "catalogEntry": catalog_entry(stored, urls),
        "packageContent": urls.package_content(&metadata.id, &metadata.normalized_version),
        "registration": urls.registration_index(&metadata.id),
    })
}

pub fn leaf(stored: &Stored, urls: &Urls) -> Value {
    let metadata = &stored.metadata;
    json!({
        "@id": urls.registration_leaf(&metadata.id, &metadata.normalized_version),
        "@type": ["Package", "http://schema.nuget.org/catalog#Permalink"],
        "catalogEntry": urls.catalog_leaf(&metadata.id, &metadata.normalized_version),
        "listed": true,
        "packageContent": urls.package_content(&metadata.id, &metadata.normalized_version),
        "published": time(stored.published()),
        "registration": urls.registration_index(&metadata.id),
    })
}

pub fn item_version(item: &Value) -> Option<Version> {
    item.pointer("/catalogEntry/version")
        .and_then(Value::as_str)
        .and_then(Version::parse)
        .or_else(|| {
            item.get("@id")
                .and_then(Value::as_str)
                .and_then(|id| id.rsplit('/').next())
                .and_then(|leaf| leaf.strip_suffix(".json"))
                .and_then(Version::parse)
        })
}

pub fn index(id: &str, items: Vec<Value>, urls: &Urls) -> Option<Value> {
    if items.is_empty() {
        return None;
    }
    let index_url = urls.registration_index(id);
    let pages: Vec<Value> = items
        .chunks(PAGE_SIZE)
        .map(|chunk| {
            let bound = |item: Option<&Value>| {
                item.and_then(item_version)
                    .map(|version| version.lower())
                    .unwrap_or_default()
            };
            let lower = bound(chunk.first());
            let upper = bound(chunk.last());
            json!({
                "@id": format!("{index_url}#page/{lower}/{upper}"),
                "@type": "catalog:CatalogPage",
                "count": chunk.len(),
                "items": chunk,
                "lower": lower,
                "parent": index_url,
                "upper": upper,
            })
        })
        .collect();
    Some(json!({
        "@id": index_url,
        "@type": ["catalog:CatalogRoot", "PackageRegistration", "catalog:Permalink"],
        "count": pages.len(),
        "items": pages,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::stored;

    #[test]
    fn registrations_inline_their_versions() {
        let urls = Urls::new("http://h/repository/n");
        let one = stored(
            r#"<package><metadata><id>Astrum.Probe</id><version>1.0</version><authors>A</authors><tags>x y</tags>
            <dependencies><group targetFramework="net8.0"><dependency id="Dep" version="2.0"/></group></dependencies></metadata></package>"#,
        );
        let entry = catalog_entry(&one, &urls);
        assert_eq!(entry["version"], "1.0.0");
        assert_eq!(entry["published"], "2026-10-12T19:32:38.698Z");
        assert_eq!(
            entry["packageContent"],
            "http://h/repository/n/v3/content/astrum.probe/1.0.0/astrum.probe.1.0.0.nupkg"
        );
        assert_eq!(
            entry["dependencyGroups"][0]["dependencies"][0]["range"],
            "[2.0.0, )"
        );
        assert_eq!(
            entry["dependencyGroups"][0]["dependencies"][0]["registration"],
            "http://h/repository/n/v3/registration/dep/index.json"
        );
        assert_eq!(entry["tags"], serde_json::json!(["x", "y"]));
        let two = stored(
            "<package><metadata><id>Astrum.Probe</id><version>2.0.0-RC.1</version></metadata></package>",
        );
        let document = index(
            "Astrum.Probe",
            vec![item(&one, &urls), item(&two, &urls)],
            &urls,
        )
        .unwrap();
        assert_eq!(document["count"], 1);
        assert_eq!(document["items"][0]["count"], 2);
        assert_eq!(document["items"][0]["lower"], "1.0.0");
        assert_eq!(document["items"][0]["upper"], "2.0.0-rc.1");
        assert_eq!(
            document["items"][0]["@id"],
            "http://h/repository/n/v3/registration/astrum.probe/index.json#page/1.0.0/2.0.0-rc.1"
        );
        assert!(index("x", Vec::new(), &urls).is_none());
        let single = leaf(&two, &urls);
        assert_eq!(
            single["@id"],
            "http://h/repository/n/v3/registration/astrum.probe/2.0.0-rc.1.json"
        );
        assert_eq!(
            item_version(&item(&two, &urls)).unwrap().normalized(),
            "2.0.0-RC.1"
        );
    }
}
