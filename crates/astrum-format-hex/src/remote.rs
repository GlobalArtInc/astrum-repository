use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use serde_json::json;

use crate::{
    ID, KEY, OTHER, REGISTRY, REGISTRY_TYPE, Route, TAR_TYPE, TARBALL, files, route, tarball,
};

pub fn classify(path: &str) -> (&'static str, Freshness) {
    match route(path) {
        Route::Resource(_) => (REGISTRY, Freshness::Metadata),
        Route::PublicKey => (KEY, Freshness::Metadata),
        Route::Tarball(_, _) => (TARBALL, Freshness::Content),
        _ if path.ends_with(".csv") || path.ends_with(".signed") => (OTHER, Freshness::Metadata),
        _ => (OTHER, Freshness::Content),
    }
}

pub fn describe(path: &str, kind: &'static str) -> PutAsset {
    let put = PutAsset::cached(path).kind(kind);
    match (kind, route(path)) {
        (TARBALL, Route::Tarball(name, version)) => {
            put.content_type(TAR_TYPE).component(ComponentKey {
                namespace: String::new(),
                name,
                version,
                kind: String::new(),
                attributes: json!({}),
            })
        }
        (REGISTRY, _) => put.content_type(REGISTRY_TYPE),
        (KEY, _) => put.content_type(crate::keys::KEY_TYPE),
        _ => put,
    }
}

pub async fn get(ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let (kind, freshness) = classify(path);
    let found = proxy::fetch(ctx, path, None, freshness, || describe(path, kind)).await?;
    let Some(found) = found else {
        return Ok(None);
    };
    if kind != TARBALL || found.attributes.get(ID).is_some() {
        return Ok(Some(found));
    }
    let read = match files::bytes(ctx, &found, tarball::MAX_TARBALL).await {
        Ok(bytes) => tokio::task::spawn_blocking(move || tarball::read(&bytes))
            .await
            .map_err(ContentError::internal)?,
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path, %error, "a proxied tarball cannot be read");
            return Ok(Some(found));
        }
    };
    match read {
        Ok(read) => {
            let mut attributes = found.attributes.clone();
            match attributes.as_object_mut() {
                Some(map) => {
                    map.insert(ID.to_owned(), read.attributes());
                }
                None => attributes = json!({ ID: read.attributes() }),
            }
            store::set_attributes(ctx.db(), found.id, attributes.clone()).await?;
            if let Some(component_id) = found.component_id {
                store::set_component_attributes(
                    ctx.db(),
                    component_id,
                    json!({ ID: read.metadata.summary() }),
                )
                .await?;
            }
            Ok(Some(Asset {
                attributes,
                ..found
            }))
        }
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path, %error, "the metadata of a proxied tarball cannot be read");
            Ok(Some(found))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxied_paths_are_classified() {
        assert_eq!(classify("/names"), (REGISTRY, Freshness::Metadata));
        assert_eq!(classify("/packages/jason"), (REGISTRY, Freshness::Metadata));
        assert_eq!(classify("/public_key"), (KEY, Freshness::Metadata));
        assert_eq!(
            classify("/tarballs/jason-1.4.4.tar"),
            (TARBALL, Freshness::Content)
        );
        assert_eq!(
            classify("/installs/hex-1.x.csv"),
            (OTHER, Freshness::Metadata)
        );
        assert_eq!(
            classify("/installs/1.17.0/hex-2.1.1.ez"),
            (OTHER, Freshness::Content)
        );
        let put = describe("/tarballs/jason-1.4.4.tar", TARBALL);
        let component = put.component.unwrap();
        assert_eq!(
            (component.name.as_str(), component.version.as_str()),
            ("jason", "1.4.4")
        );
    }
}
