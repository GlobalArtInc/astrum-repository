use astrum_content::proxy;
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use serde_json::json;

use crate::paths::{self, PackageFile, Route};
use crate::signing::SIGNATURE_TYPE;
use crate::{DATABASE, ID, OTHER, PACKAGE, SIGNATURE, files, hosted};

pub fn classify(path: &str) -> (&'static str, Freshness) {
    match paths::route(path) {
        Route::Database(_) | Route::DatabaseSignature(_) => (DATABASE, Freshness::Metadata),
        Route::Package(_) => (PACKAGE, Freshness::Content),
        Route::Signature(_) => (SIGNATURE, Freshness::Content),
        Route::Key(_) | Route::File | Route::Directory => (OTHER, Freshness::Metadata),
    }
}

pub fn content_type(file: &PackageFile) -> &'static str {
    match file.extension.as_str() {
        ".zst" => "application/zstd",
        ".xz" => "application/x-xz",
        ".gz" => "application/gzip",
        ".bz2" => "application/x-bzip2",
        "" => "application/x-tar",
        _ => "application/octet-stream",
    }
}

pub fn describe(path: &str, kind: &'static str) -> PutAsset {
    let put = PutAsset::cached(path).kind(kind);
    match paths::route(path) {
        Route::Package(file) => put
            .content_type(content_type(&file))
            .component(hosted::file_component(&file)),
        Route::Signature(file) => put
            .content_type(SIGNATURE_TYPE)
            .component(hosted::file_component(&file)),
        _ => put,
    }
}

pub async fn get(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let (kind, freshness) = classify(path);
    let found = proxy::fetch(ctx, path, None, freshness, || describe(path, kind)).await?;
    let Some(found) = found else {
        return Ok(None);
    };
    if kind != PACKAGE || found.attributes.get(ID).is_some() {
        return Ok(Some(found));
    }
    match files::read_head(ctx, &found).await {
        Ok(contents) => {
            let mut attributes = found.attributes.clone();
            match attributes.as_object_mut() {
                Some(map) => {
                    map.insert(ID.to_owned(), contents.info.attributes());
                }
                None => attributes = json!({ ID: contents.info.attributes() }),
            }
            store::set_attributes(ctx.db(), found.id, attributes.clone()).await?;
            Ok(Some(Asset {
                attributes,
                ..found
            }))
        }
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path, %error, "the .PKGINFO of a proxied package cannot be read");
            Ok(Some(found))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn databases_are_metadata_and_packages_content() {
        assert_eq!(
            classify("/core/os/x86_64/core.db"),
            (DATABASE, Freshness::Metadata)
        );
        assert_eq!(
            classify("/core/os/x86_64/core.files.tar.gz.sig"),
            (DATABASE, Freshness::Metadata)
        );
        assert_eq!(
            classify("/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst"),
            (PACKAGE, Freshness::Content)
        );
        assert_eq!(
            classify("/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst.sig"),
            (SIGNATURE, Freshness::Content)
        );
        assert_eq!(classify("/lastsync"), (OTHER, Freshness::Metadata));
        let put = describe("/core/os/x86_64/bash-5.2.037-1-x86_64.pkg.tar.zst", PACKAGE);
        let component = put.component.unwrap();
        assert_eq!(
            (
                component.namespace.as_str(),
                component.name.as_str(),
                component.version.as_str()
            ),
            ("x86_64", "bash", "5.2.037-1")
        );
        assert_eq!(put.content_type.as_deref(), Some("application/zstd"));
    }
}
