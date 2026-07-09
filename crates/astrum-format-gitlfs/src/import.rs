use astrum_content::import::{Described, Imported, Plan};
use astrum_content::mime;
use astrum_content::store::ComponentKey;
use astrum_repo::RepositoryType;
use serde_json::json;

use crate::{OBJECT, oid_of_asset_path};

pub fn plan(_repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match oid_of_asset_path(path) {
        Some(_) => Plan::Import,
        None => Plan::Skip("the path is not a Git LFS object".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    let oid = oid_of_asset_path(&file.path)
        .ok_or_else(|| "the path is not a Git LFS object".to_owned())?
        .to_owned();
    if !file.sha256.is_empty() && !file.sha256.eq_ignore_ascii_case(&oid) {
        return Err(format!(
            "the object's SHA-256 is {}, not its id {oid}",
            file.sha256
        ));
    }
    let sample = file
        .head(mime::SAMPLE as u64)
        .await
        .map_err(|error| error.to_string())?;
    let content_type = mime::detect(&sample).unwrap_or_else(|| mime::OCTET_STREAM.to_owned());
    let put = file
        .put(file.path.clone())
        .kind(OBJECT)
        .content_type(content_type)
        .component(ComponentKey {
            namespace: String::new(),
            name: oid,
            version: String::new(),
            kind: String::new(),
            attributes: json!({}),
        });
    Ok(Some(Described::source(put)))
}
