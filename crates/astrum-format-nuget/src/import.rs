use astrum_content::import::{Described, Imported, Plan};
use astrum_content::store::ComponentKey;
use astrum_repo::RepositoryType;
use serde_json::json;

use crate::archive;
use crate::hosted;
use crate::metadata::parse_nuspec;
use crate::remote::{self, INDEX_PATH};
use crate::route::{package_path, symbols_path};
use crate::{ID, PACKAGE_KIND, SYMBOLS_KIND, ZIP};

enum Stored {
    Index,
    Document(&'static str),
    Package,
    Symbols,
}

fn classify(path: &str) -> Option<Stored> {
    if path == INDEX_PATH {
        return Some(Stored::Index);
    }
    if path.starts_with("/v3/registration/") && path.ends_with(".json") {
        return Some(Stored::Document("application/json"));
    }
    if let Some(rest) = path.strip_prefix("/v3/content/") {
        let parts: Vec<&str> = rest.split('/').collect();
        return match parts.as_slice() {
            [_, "index.json"] => Some(Stored::Document("application/json")),
            [id, _, file] if *file == format!("{id}.nuspec") => {
                Some(Stored::Document("application/xml"))
            }
            _ => None,
        };
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [id, version, file] if *file == format!("{id}.{version}.nupkg") => Some(Stored::Package),
        [id, version, file] if *file == format!("{id}.{version}.snupkg") => Some(Stored::Symbols),
        _ => None,
    }
}

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match (repository, classify(path)) {
        (_, Some(Stored::Package | Stored::Symbols)) => Plan::Import,
        (RepositoryType::Proxy, Some(_)) => Plan::Import,
        (RepositoryType::Hosted, _) => Plan::Derived,
        _ => Plan::Skip("a cached remote answer that the Rust proxy fetches again".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    let stored = classify(&file.path)
        .ok_or_else(|| "the path is not a package or a cached document".to_owned())?;
    match stored {
        Stored::Index => {
            let remote = remote::index_url_of(&file.config).map_err(|error| error.to_string())?;
            Ok(Some(Described::source(
                file.put(INDEX_PATH)
                    .content_type("application/json")
                    .attributes(json!({ ID: { "remote": remote.to_string() } })),
            )))
        }
        Stored::Document(content_type) => Ok(Some(Described::source(
            file.put(file.path.clone()).content_type(content_type),
        ))),
        Stored::Package | Stored::Symbols => {
            let source = archive::Blob {
                store: file.store.clone(),
                id: file.blob,
                size: file.size,
            };
            let nuspec = archive::nuspec(&source)
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "the package holds no .nuspec file".to_owned())?;
            let mut metadata = parse_nuspec(&nuspec).map_err(|error| error.to_string())?;
            if let Some(known) = &file.component
                && known.name.eq_ignore_ascii_case(&metadata.id)
            {
                metadata.id = known.name.clone();
            }
            let symbols = matches!(stored, Stored::Symbols) || metadata.is_symbols();
            let key = ComponentKey {
                namespace: String::new(),
                name: metadata.id.clone(),
                version: metadata.normalized_version.clone(),
                kind: PACKAGE_KIND.to_owned(),
                attributes: hosted::attributes(&metadata),
            };
            let (path, kind) = match symbols {
                true => (
                    symbols_path(&metadata.id, &metadata.normalized_version),
                    SYMBOLS_KIND,
                ),
                false => (
                    package_path(&metadata.id, &metadata.normalized_version),
                    PACKAGE_KIND,
                ),
            };
            let path = match path.eq_ignore_ascii_case(&file.path) {
                true => file.path.clone(),
                false => {
                    return Err(format!(
                        "the package inside is {} {}, which belongs at {path}",
                        metadata.id, metadata.normalized_version
                    ));
                }
            };
            let put = file
                .put(path)
                .component(key)
                .kind(kind)
                .content_type(ZIP)
                .attributes(json!({ ID: {} }));
            let described = Described::source(put);
            Ok(Some(match symbols {
                true => described.keeping_component(),
                false => described,
            }))
        }
    }
}
