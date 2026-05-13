use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;

use crate::documents::{self, MAX_DOCUMENT};
use crate::packument;
use crate::path::{self, Route};

pub fn plan(_repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match path::route(path) {
        Route::Packument { .. } | Route::Tarball { .. } => Plan::Import,
        _ => Plan::Skip("the path is neither a package document nor a tarball".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    match path::route(&file.path) {
        Route::Packument { name } => {
            let put = documents::packument_put(&name, name.packument_path());
            if file.cached() {
                return Ok(Some(Described::source(put)));
            }
            let bytes = file
                .bytes(MAX_DOCUMENT as u64)
                .await
                .map_err(|error| error.to_string())?;
            let doc: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|error| format!("the package document is not valid JSON: {error}"))?;
            let mut put = put.attributes(packument::search_attributes(&doc));
            put.verified = false;
            Ok(Some(Described::source(put)))
        }
        Route::Tarball {
            name,
            file: tarball,
        } => {
            let path = name.tarball_path(&tarball);
            let version = name.version_of_file(&tarball);
            Ok(Some(Described::source(documents::tarball_put(
                &name,
                file.put(path),
                version,
            ))))
        }
        _ => Err("the path is neither a package document nor a tarball".to_owned()),
    }
}
