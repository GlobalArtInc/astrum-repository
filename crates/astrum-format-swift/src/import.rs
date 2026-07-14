use astrum_content::import::{Described, Imported, Plan};
use astrum_content::store::ComponentKey;
use astrum_repo::RepositoryType;
use serde_json::{Map, Value, json};

use crate::paths::{self, Package, Route};
use crate::{ARCHIVE, ID, MANIFEST_KIND};

const KEPT: &[&str] = &[
    "checksum",
    "publishedAt",
    "metadata",
    "description",
    "repositoryURLs",
    "signing",
    "metadataSigning",
];

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match (repository, paths::route(path)) {
        (RepositoryType::Hosted, Route::Archive(_) | Route::Manifest(..)) => Plan::Import,
        _ => Plan::Skip("the path is neither a source archive nor a manifest".to_owned()),
    }
}

fn component(file: &Imported, package: &Package) -> ComponentKey {
    let known = file
        .component
        .as_ref()
        .and_then(|component| component.attributes.get(ID))
        .and_then(Value::as_object);
    let mut swift = Map::new();
    if let Some(known) = known {
        for key in KEPT {
            if let Some(value) = known.get(*key).filter(|value| !value.is_null()) {
                swift.insert((*key).to_owned(), value.clone());
            }
        }
    }
    if !swift.contains_key("checksum") && !file.sha256.is_empty() && file.kind == ARCHIVE {
        swift.insert("checksum".to_owned(), Value::String(file.sha256.clone()));
    }
    if !swift.contains_key("metadata") {
        swift.insert("metadata".to_owned(), json!({}));
    }
    if !swift.contains_key("repositoryURLs")
        && let Some(listed) = swift
            .get("metadata")
            .and_then(|metadata| metadata.get("repositoryURLs"))
            .filter(|listed| listed.is_array())
            .cloned()
    {
        swift.insert("repositoryURLs".to_owned(), listed);
    }
    ComponentKey {
        namespace: package.scope.clone(),
        name: package.name.clone(),
        version: package.version.clone().unwrap_or_default(),
        kind: String::new(),
        attributes: json!({ ID: swift }),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    match paths::route(&file.path) {
        Route::Archive(package) => {
            let put = file
                .put(package.archive_path())
                .component(component(file, &package))
                .kind(ARCHIVE)
                .content_type("application/zip")
                .attributes(json!({}));
            Ok(Some(Described::source(put)))
        }
        Route::Manifest(package, name) => {
            let known = file
                .attributes
                .get(ID)
                .cloned()
                .filter(Value::is_object)
                .unwrap_or_else(|| json!({}));
            let mut swift = Map::new();
            if let Some(tools) = known.get("toolsVersion") {
                swift.insert("toolsVersion".to_owned(), tools.clone());
            }
            if let Some(alternates) = known.get("alternates") {
                swift.insert("alternates".to_owned(), alternates.clone());
            }
            let put = file
                .put(package.manifest_path(&name))
                .component(component(file, &package))
                .kind(MANIFEST_KIND)
                .content_type("text/x-swift")
                .attributes(json!({ ID: swift }));
            Ok(Some(Described::source(put).keeping_component()))
        }
        _ => Err("the path is neither a source archive nor a manifest".to_owned()),
    }
}
