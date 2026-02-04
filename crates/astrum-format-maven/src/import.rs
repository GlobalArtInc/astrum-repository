use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;

use crate::metadata::{self, Metadata};
use crate::path::{self, Kind};
use crate::{DOCUMENT_LIMIT, describe as describe_path, hosted};

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    let parsed = path::parse(path);
    if parsed.hash.is_some() {
        return Plan::Derived;
    }
    match (repository, &parsed.kind) {
        (RepositoryType::Hosted, Kind::ArchetypeCatalog | Kind::Index) => Plan::Derived,
        _ => Plan::Import,
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    let parsed = path::parse(&file.path);
    let put = describe_path(&parsed, file.put(file.path.clone()));
    match (&parsed.kind, file.repository) {
        (Kind::Metadata, RepositoryType::Hosted) => {
            let bytes = file
                .bytes(DOCUMENT_LIMIT as u64)
                .await
                .map_err(|error| error.to_string())?;
            let Ok(document) = metadata::parse(&bytes) else {
                return Ok(None);
            };
            if document.plugins.is_empty() {
                return Ok(None);
            }
            let plugins = Metadata {
                plugins: document.plugins,
                ..Default::default()
            };
            Ok(Some(Described::bytes(
                put,
                metadata::write(&plugins).into_bytes(),
            )))
        }
        (Kind::Artifact(coordinates), RepositoryType::Hosted) if hosted::is_pom(coordinates) => {
            let bytes = file
                .bytes(DOCUMENT_LIMIT as u64)
                .await
                .map_err(|error| error.to_string())?;
            Ok(Some(Described::source(hosted::with_pom(put, &bytes))))
        }
        _ => Ok(Some(Described::source(put))),
    }
}
