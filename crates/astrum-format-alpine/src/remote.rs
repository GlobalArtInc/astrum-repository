use std::sync::Arc;

use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use serde_json::json;

use crate::archive;
use crate::paths::{self, Route};
use crate::signing::SigningKey;
use crate::{APK, APK_TYPE, Alpine, ID, INDEX, INDEX_TYPE, KEY, OTHER, files, package};

pub fn classify(path: &str) -> (&'static str, Freshness) {
    match paths::route(path) {
        Route::Index(_) => (INDEX, Freshness::Metadata),
        Route::Key(_) => (KEY, Freshness::Metadata),
        Route::Package => (APK, Freshness::Content),
        Route::File | Route::Directory => (OTHER, Freshness::Content),
    }
}

pub fn describe(path: &str, kind: &'static str) -> PutAsset {
    let put = PutAsset::cached(path).kind(kind);
    match kind {
        APK => {
            let put = put.content_type(APK_TYPE);
            match package::parse_file_name(paths::file_name(path)) {
                Some((name, version)) => put.component(ComponentKey {
                    namespace: paths::last_segment(paths::directory(path)).to_owned(),
                    name,
                    version,
                    kind: String::new(),
                    attributes: json!({}),
                }),
                None => put,
            }
        }
        INDEX => put.content_type(INDEX_TYPE),
        _ => put,
    }
}

pub async fn get(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let (kind, freshness) = classify(path);
    let found = proxy::fetch(ctx, path, None, freshness, || describe(path, kind)).await?;
    let Some(found) = found else {
        return Ok(None);
    };
    if kind != APK || found.attributes.get(ID).is_some() {
        return Ok(Some(found));
    }
    match files::read_asset_package(ctx, &found).await {
        Ok(package) => {
            let mut entry = package.entry(found.size.max(0) as u64);
            entry.arch = crate::hosted::index_arch(&package, paths::directory(path));
            let mut attributes = found.attributes.clone();
            match attributes.as_object_mut() {
                Some(map) => {
                    map.insert(ID.to_owned(), entry.attributes());
                }
                None => attributes = json!({ ID: entry.attributes() }),
            }
            store::set_attributes(ctx.db(), found.id, attributes.clone()).await?;
            Ok(Some(Asset {
                attributes,
                ..found
            }))
        }
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path, %error, "the control segment of a proxied package cannot be read");
            Ok(Some(found))
        }
    }
}

impl Alpine {
    pub(crate) async fn resigned(
        &self,
        ctx: &Ctx,
        key: Arc<SigningKey>,
        found: &Asset,
    ) -> Result<Vec<u8>, ContentError> {
        let bytes = files::bytes(ctx, found).await?;
        let cache_key = (ctx.name().to_owned(), found.path.clone());
        let cached = self
            .signatures
            .lock()
            .expect("the signature cache lock is never poisoned")
            .get(&cache_key)
            .filter(|(blob, signer, _)| *blob == found.blob_id && *signer == key.fingerprint)
            .map(|(_, _, member)| member.clone());
        let (mut file, bytes) = match cached {
            Some(member) => (member, bytes),
            None => {
                let signer = key.clone();
                let mtime = chrono::Utc::now().timestamp().max(0) as u64;
                let (bytes, member) = tokio::task::spawn_blocking(move || {
                    let member = archive::split(&bytes, files::MAX_INDEX)
                        .map(|split| signer.signature_member(split.rest, mtime));
                    (bytes, member)
                })
                .await
                .map_err(ContentError::internal)?;
                let member = member.map_err(|error| {
                    ContentError::BadGateway(format!("{} of the remote: {error}", found.path))
                })?;
                self.signatures
                    .lock()
                    .expect("the signature cache lock is never poisoned")
                    .insert(
                        cache_key,
                        (found.blob_id, key.fingerprint.clone(), member.clone()),
                    );
                (member, bytes)
            }
        };
        file.extend_from_slice(&bytes);
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxied_paths_are_classified_like_the_recipe() {
        assert_eq!(classify("/v3.20/main/x86_64/APKINDEX.tar.gz").0, INDEX);
        assert_eq!(
            classify("/v3.20/main/x86_64/APKINDEX.tar.gz").1,
            Freshness::Metadata
        );
        assert_eq!(classify("/v3.20/main/x86_64/musl-1.2.5-r3.apk").0, APK);
        assert_eq!(classify("/alpine-devel.rsa.pub").0, KEY);
        assert_eq!(classify("/v3.20/README").0, OTHER);
        let put = describe("/v3.20/main/x86_64/musl-1.2.5-r3.apk", APK);
        let component = put.component.unwrap();
        assert_eq!(
            (
                component.namespace.as_str(),
                component.name.as_str(),
                component.version.as_str()
            ),
            ("x86_64", "musl", "1.2.5-r3")
        );
    }
}
