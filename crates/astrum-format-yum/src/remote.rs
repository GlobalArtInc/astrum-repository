use astrum_content::proxy;
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness};
use serde_json::json;

use crate::repodata::Repomd;
use crate::rpm::FileName;
use crate::{OTHER, REPODATA, REPOMD, RPM, RPM_TYPE, Yum, files, hosted, paths};

pub fn classify(path: &str) -> (&'static str, Freshness) {
    if paths::base_of_metadata(path).is_some() {
        return match paths::file_name(path) == paths::REPOMD {
            true => (REPOMD, Freshness::Metadata),
            false => (REPODATA, Freshness::Metadata),
        };
    }
    match paths::is_rpm(path) {
        true => (RPM, Freshness::Content),
        false => (OTHER, Freshness::Content),
    }
}

fn describe(path: &str, kind: &'static str) -> PutAsset {
    let put = PutAsset::cached(path).kind(kind);
    match kind {
        RPM => {
            let put = put.content_type(RPM_TYPE);
            match FileName::parse(paths::file_name(path)) {
                Some(name) => put.component(hosted::file_component(&name)),
                None => put,
            }
        }
        _ => put,
    }
}

pub async fn get(yum: &Yum, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
    let (kind, freshness) = classify(path);
    let found = proxy::fetch(ctx, path, None, freshness, || describe(path, kind)).await?;
    let Some(found) = found else {
        return Ok(None);
    };
    match kind {
        REPOMD => {
            if yum.first_sight(ctx.name(), &found) {
                invalidate(ctx, &found).await;
            }
        }
        RPM if found.attributes.get("yum").is_none() => {
            match files::read_asset_package(ctx, &found).await {
                Ok(package) => {
                    let mut attributes = found.attributes.clone();
                    if let Some(map) = attributes.as_object_mut() {
                        map.insert("yum".to_owned(), package.attributes());
                    } else {
                        attributes = json!({"yum": package.attributes()});
                    }
                    store::set_attributes(ctx.db(), found.id, attributes.clone()).await?;
                    return Ok(Some(Asset {
                        attributes,
                        ..found
                    }));
                }
                Err(error) => {
                    tracing::debug!(repository = ctx.name(), path, %error, "the header of a proxied package cannot be read");
                }
            }
        }
        _ => {}
    }
    Ok(Some(found))
}

fn matches(found: &Asset, kind: &str, checksum: &str) -> bool {
    let stored = match kind.to_ascii_lowercase().as_str() {
        "sha" | "sha1" => &found.sha1,
        "sha256" => &found.sha256,
        "sha512" => &found.sha512,
        "md5" => &found.md5,
        _ => return false,
    };
    stored.eq_ignore_ascii_case(checksum)
}

async fn invalidate(ctx: &Ctx, repomd: &Asset) {
    let Some(base) = paths::base_of_metadata(&repomd.path) else {
        return;
    };
    let parsed = match files::bytes(ctx, repomd)
        .await
        .map(|bytes| Repomd::parse(&bytes))
    {
        Ok(Ok(parsed)) => parsed,
        Ok(Err(error)) => {
            tracing::debug!(repository = ctx.name(), path = %repomd.path, %error, "a proxied repomd.xml cannot be read");
            return;
        }
        Err(error) => {
            tracing::debug!(repository = ctx.name(), path = %repomd.path, %error, "a proxied repomd.xml cannot be read");
            return;
        }
    };
    for data in parsed
        .data
        .iter()
        .filter(|data| data.location_base.is_none())
    {
        let Some(path) = paths::join(&base, &data.location) else {
            continue;
        };
        let Ok(Some(cached)) = store::find(ctx.db(), ctx.name(), &path).await else {
            continue;
        };
        if !matches(&cached, &data.checksum_type, &data.checksum)
            && let Err(error) = files::mark_stale(ctx.db(), cached.id).await
        {
            tracing::debug!(repository = ctx.name(), path, %error, "cached metadata was not marked stale");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_classified_like_the_recipe() {
        assert_eq!(classify("/repodata/repomd.xml").0, REPOMD);
        assert_eq!(classify("/el9/repodata/repomd.xml.asc").0, REPODATA);
        assert_eq!(classify("/el9/repodata/x-primary.xml.gz").0, REPODATA);
        assert_eq!(classify("/el9/Packages/a-1-1.x86_64.rpm").0, RPM);
        assert_eq!(classify("/RPM-GPG-KEY").0, OTHER);
        assert_eq!(classify("/repodata/repomd.xml").1, Freshness::Metadata);
    }
}
