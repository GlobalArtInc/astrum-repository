use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::Value;

use crate::MANIFEST_LIMIT;
use crate::manifest;
use crate::reference::Digest;
use crate::storage;

enum Stored {
    Blob(Digest),
    Manifest { image: String, reference: String },
}

fn classify(path: &str) -> Option<Stored> {
    if let Some(digest) = path.strip_prefix("/v2/-/blobs/") {
        return Digest::parse(digest).map(Stored::Blob);
    }
    let rest = path.strip_prefix("/v2/")?;
    let (image, reference) = rest.rsplit_once("/manifests/")?;
    (!image.is_empty() && !reference.is_empty() && !reference.contains('/')).then(|| {
        Stored::Manifest {
            image: image.to_owned(),
            reference: reference.to_owned(),
        }
    })
}

pub fn plan(_repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match classify(path) {
        Some(_) => Plan::Import,
        None => Plan::Skip("the path is neither a blob nor a manifest".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    match classify(&file.path) {
        Some(Stored::Blob(digest)) => {
            let mut put = storage::blob_put(&digest);
            put.verified = true;
            Ok(Some(Described::source(put)))
        }
        Some(Stored::Manifest { image, reference }) => {
            let bytes = file
                .bytes(MANIFEST_LIMIT as u64)
                .await
                .map_err(|error| error.to_string())?;
            let declared = file
                .attributes
                .pointer("/docker/mediaType")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| Some(file.content_type.clone()).filter(|value| !value.is_empty()));
            let parsed = manifest::parse(&bytes, declared.as_deref())?;
            let (digest, tag) = match Digest::parse(&reference) {
                Some(digest) => {
                    if !digest.matches_bytes(&bytes) {
                        return Err(format!("the manifest does not match its digest {digest}"));
                    }
                    (digest, None)
                }
                None => {
                    let recorded = file
                        .attributes
                        .pointer("/docker/digest")
                        .and_then(Value::as_str)
                        .and_then(Digest::parse)
                        .filter(|digest| digest.matches_bytes(&bytes));
                    (
                        recorded.unwrap_or_else(|| Digest::sha256(&bytes)),
                        Some(reference.clone()),
                    )
                }
            };
            let mut put =
                storage::manifest_put(&image, &reference, &digest, &parsed, tag.as_deref());
            put.verified = file.cached();
            Ok(Some(Described::source(put)))
        }
        None => Err("the path is neither a blob nor a manifest".to_owned()),
    }
}
