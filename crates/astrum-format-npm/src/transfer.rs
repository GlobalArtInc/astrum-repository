use std::cmp::Ordering;

use astrum_content::import::{Described, Imported};
use astrum_content::store::{self, Component};
use astrum_content::transfer::{
    AssetStatus, BlobSource, Bundle, BundleAsset, GenericTransfer, Outcome, Status, Terms, Transfer,
};
use astrum_content::{Asset, ContentError, Ctx, View};
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::DateTime;
use futures_util::future::BoxFuture;
use serde_json::{Value, json};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

use crate::attachment;
use crate::documents::{self, Stored};
use crate::hosted::{self, MAX_PUBLISH};
use crate::import;
use crate::packument;
use crate::path::Name;
use crate::{ID, Npm, TARBALL};

static TARBALLS: GenericTransfer = GenericTransfer {
    plan: import::plan,
    describe: verified,
    after: None,
    admit: None,
    replaceable: None,
    check_content_type: true,
};

fn verified(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(check_stored(file))
}

fn wanted<'a>(expected: &'a Value, field: &str, path: &str) -> Result<&'a str, String> {
    expected
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{path} was staged without its expected {field}"))
}

async fn check_stored(file: &Imported) -> Result<Option<Described>, String> {
    let expected = file
        .attributes
        .pointer("/npm/expected")
        .ok_or_else(|| format!("{} was staged without its expected digests", file.path))?;
    let sha1 = wanted(expected, "sha1", &file.path)?;
    let sha512 = wanted(expected, "sha512", &file.path)?;
    let data = file
        .bytes(MAX_PUBLISH as u64)
        .await
        .map_err(|error| error.to_string())?;
    for (algorithm, wanted, found) in [
        (
            "SHA-256",
            file.sha256.as_str(),
            hex::encode(Sha256::digest(&data)),
        ),
        ("SHA-1", sha1, hex::encode(Sha1::digest(&data))),
        ("SHA-512", sha512, hex::encode(Sha512::digest(&data))),
    ] {
        if !wanted.eq_ignore_ascii_case(&found) {
            return Err(format!(
                "the stored tarball {} has {algorithm} {found}, the source recorded {wanted}",
                file.path
            ));
        }
    }
    let dist = expected.get("dist").cloned().unwrap_or_else(|| json!({}));
    attachment::verify(&dist, &data).map_err(|error| {
        format!(
            "the stored tarball {} does not match the manifest of its version: {error}",
            file.path
        )
    })?;
    import::describe(file).await
}

fn invalid(what: impl std::fmt::Display) -> ContentError {
    ContentError::BadRequest(format!("the npm bundle is not valid: {what}"))
}

fn package_name(namespace: &str, name: &str) -> Result<Name, ContentError> {
    let full = match namespace.is_empty() {
        true => name.to_owned(),
        false => format!("@{namespace}/{name}"),
    };
    Name::parse(&full)
        .filter(|parsed| {
            parsed.scope.as_deref().unwrap_or_default() == namespace && parsed.bare == name
        })
        .ok_or_else(|| invalid(format!("{full} is not a package name")))
}

struct Parcel {
    name: Name,
    version: String,
    manifest: Value,
    time: Option<String>,
    dist_tags: Vec<String>,
    path: String,
    sha1: String,
    sha512: String,
}

impl Parcel {
    fn of(bundle: &Bundle) -> Result<Self, ContentError> {
        if bundle.format != ID {
            return Err(invalid(format!("it is a {} bundle", bundle.format)));
        }
        let name = package_name(&bundle.namespace, &bundle.name)?;
        let version = bundle.version.clone();
        if !packument::valid_version(&version) {
            return Err(invalid(format!("{version} is not a semantic version")));
        }
        let manifest = bundle
            .meta
            .get("manifest")
            .filter(|manifest| manifest.is_object())
            .cloned()
            .ok_or_else(|| invalid("it carries no version manifest"))?;
        if manifest.get("name").and_then(Value::as_str) != Some(name.to_string().as_str()) {
            return Err(invalid(format!("the manifest does not name {name}")));
        }
        if manifest.get("version").and_then(Value::as_str) != Some(version.as_str()) {
            return Err(invalid("the manifest names another version"));
        }
        if manifest.get("dist").is_some_and(|dist| !dist.is_object()) {
            return Err(invalid("the manifest dist is not an object"));
        }
        let time = match bundle.meta.get("time") {
            None | Some(Value::Null) => None,
            Some(Value::String(time)) if DateTime::parse_from_rfc3339(time).is_ok() => {
                Some(time.clone())
            }
            Some(_) => None,
        };
        let dist_tags = match bundle.meta.get("distTags") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(tags)) => tags
                .iter()
                .map(|tag| {
                    tag.as_str()
                        .filter(|tag| packument::valid_tag(tag))
                        .map(str::to_owned)
                        .ok_or_else(|| invalid("a dist-tag is not valid"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err(invalid("the dist-tags are not a list")),
        };
        let [asset] = bundle.assets.as_slice() else {
            return Err(invalid("it must hold exactly one tarball"));
        };
        let path = name.tarball_path(&name.tarball_file(&version));
        if asset.kind != TARBALL || asset.path != path {
            return Err(invalid(format!("the tarball must be {path}")));
        }
        if asset.sha1.is_empty() || asset.sha512.is_empty() {
            return Err(invalid("the tarball has no SHA-1 or SHA-512"));
        }
        Ok(Self {
            name,
            version,
            manifest,
            time,
            dist_tags,
            path,
            sha1: asset.sha1.to_ascii_lowercase(),
            sha512: asset.sha512.to_ascii_lowercase(),
        })
    }

    fn staged(&self, bundle: &Bundle) -> Bundle {
        let mut staged = bundle.clone();
        let dist = self
            .manifest
            .get("dist")
            .cloned()
            .unwrap_or_else(|| json!({}));
        for asset in &mut staged.assets {
            asset.attributes = json!({"npm": {"expected": {
                "dist": dist,
                "sha1": self.sha1,
                "sha512": self.sha512,
            }}});
        }
        staged
    }

    fn integrity(&self) -> Result<String, ContentError> {
        let digest = hex::decode(&self.sha512)
            .map_err(|_| invalid("the SHA-512 of the tarball is not hexadecimal"))?;
        Ok(format!("sha512-{}", STANDARD.encode(digest)))
    }

    fn manifest_for(&self, target: &Ctx) -> Result<Value, ContentError> {
        let mut manifest = self.manifest.clone();
        let mut dist = manifest
            .get("dist")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        dist.insert(
            "tarball".to_owned(),
            Value::String(format!("{}{}", target.repository_url(), self.path)),
        );
        dist.insert("shasum".to_owned(), Value::String(self.sha1.clone()));
        dist.insert("integrity".to_owned(), Value::String(self.integrity()?));
        manifest["dist"] = Value::Object(dist);
        Ok(manifest)
    }

    fn current(&self, doc: &Value, row: &Asset) -> bool {
        let Some(entry) = packument::versions(doc).and_then(|versions| versions.get(&self.version))
        else {
            return false;
        };
        let shasum = entry
            .pointer("/dist/shasum")
            .and_then(Value::as_str)
            .is_some_and(|shasum| shasum.eq_ignore_ascii_case(&row.sha1));
        let Ok(digest) = hex::decode(&row.sha512) else {
            return false;
        };
        let wanted = format!("sha512-{}", STANDARD.encode(digest));
        let integrity = entry
            .pointer("/dist/integrity")
            .and_then(Value::as_str)
            .is_some_and(|integrity| integrity.split_whitespace().any(|part| part == wanted));
        shasum && integrity
    }

    async fn entry_matches_target(
        &self,
        target: &Ctx,
        stored: Option<&Stored>,
    ) -> Result<bool, ContentError> {
        let Some(stored) = stored else {
            return Ok(false);
        };
        let Some(row) = store::find(target.db(), target.name(), &self.path).await? else {
            return Ok(false);
        };
        Ok(self.current(&stored.doc, &row))
    }

    fn may_take_latest(&self, current: &serde_json::Map<String, Value>) -> bool {
        let Ok(mine) = semver::Version::parse(&self.version) else {
            return false;
        };
        match current.get("latest") {
            None => true,
            Some(Value::String(held)) => semver::Version::parse(held)
                .is_ok_and(|held| mine.cmp_precedence(&held) != Ordering::Less),
            Some(_) => false,
        }
    }

    fn tags_to_move(&self, doc: Option<&Value>, terms: Terms) -> serde_json::Map<String, Value> {
        if !terms.dist_tags {
            return serde_json::Map::new();
        }
        let current = doc.map(packument::dist_tags).unwrap_or_default();
        self.dist_tags
            .iter()
            .filter(|tag| tag.as_str() != "latest" || self.may_take_latest(&current))
            .map(|tag| (tag.clone(), Value::String(self.version.clone())))
            .collect()
    }
}

fn written(outcome: &Outcome) -> bool {
    matches!(outcome.status, Status::Copied | Status::Unchanged) && !outcome.assets.is_empty()
}

impl Npm {
    async fn planned(
        &self,
        target: &Ctx,
        parcel: &Parcel,
        mut outcome: Outcome,
    ) -> Result<Outcome, ContentError> {
        if outcome.status == Status::Unchanged && written(&outcome) {
            let stored = documents::hosted(target, &parcel.name).await?;
            if !parcel.entry_matches_target(target, stored.as_ref()).await? {
                outcome.status = Status::Copied;
            }
        }
        Ok(outcome)
    }

    async fn merge(
        &self,
        target: &Ctx,
        parcel: &Parcel,
        terms: Terms,
        mut outcome: Outcome,
    ) -> Result<Outcome, ContentError> {
        if !written(&outcome) {
            return Ok(outcome);
        }
        let stored = documents::hosted(target, &parcel.name).await?;
        if outcome.status == Status::Unchanged
            && parcel.entry_matches_target(target, stored.as_ref()).await?
        {
            return Ok(outcome);
        }
        let fresh = stored.is_none();
        let tags = parcel.tags_to_move(stored.as_ref().map(|stored| &stored.doc), terms);
        let incoming = json!({"dist-tags": tags});
        let manifest = parcel.manifest_for(target)?;
        let mut doc = packument::publish(
            stored.map(|stored| stored.doc),
            &parcel.name,
            &incoming,
            &[(parcel.version.clone(), manifest)],
            &hosted::now(),
        );
        if let Some(time) = &parcel.time {
            doc["time"][parcel.version.as_str()] = Value::String(time.clone());
            if fresh {
                doc["time"]["created"] = Value::String(time.clone());
            }
        }
        documents::save(target, &parcel.name, &doc).await?;
        outcome.status = Status::Copied;
        Ok(outcome)
    }

    async fn forget(&self, target: &Ctx, outcome: &Outcome) {
        for asset in &outcome.assets {
            if asset.status != AssetStatus::New {
                continue;
            }
            if let Err(error) = store::delete(target, &asset.path).await {
                tracing::warn!(repository = target.name(), path = asset.path, %error, "a tarball of a failed promotion stays");
            }
        }
    }
}

#[async_trait]
impl Transfer for Npm {
    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError> {
        if component.repository != source.name() {
            return Err(ContentError::not_found(format!(
                "the repository {} has no such component",
                source.name()
            )));
        }
        let name = package_name(&component.namespace, &component.name)?;
        let assets = store::assets_of(source.db(), component.id).await?;
        let reachable =
            |path: &str| source.allowed(View::Read, path) || source.allowed(View::Browse, path);
        if !assets.iter().any(|asset| reachable(&asset.path)) {
            return Err(ContentError::not_found(
                "there is no such component in the source repository",
            ));
        }
        source.require(View::Read, &name.packument_path())?;
        let [tarball] = assets.as_slice() else {
            return Err(invalid("the component is not one published version"));
        };
        let stored = documents::hosted(source, &name).await?.ok_or_else(|| {
            ContentError::BadRequest(format!(
                "the source repository has no package document for {name}"
            ))
        })?;
        let manifest = packument::versions(&stored.doc)
            .and_then(|versions| versions.get(&component.version))
            .cloned()
            .ok_or_else(|| {
                ContentError::BadRequest(format!(
                    "the package document of {name} does not list {}",
                    component.version
                ))
            })?;
        let time = stored
            .doc
            .get("time")
            .and_then(|time| time.get(&component.version))
            .and_then(Value::as_str)
            .filter(|time| DateTime::parse_from_rfc3339(time).is_ok())
            .map(str::to_owned);
        let dist_tags: Vec<String> = packument::dist_tags(&stored.doc)
            .into_iter()
            .filter(|(tag, version)| {
                version.as_str() == Some(component.version.as_str()) && packument::valid_tag(tag)
            })
            .map(|(tag, _)| tag)
            .collect();
        let bundle = Bundle {
            format: ID.to_owned(),
            namespace: component.namespace.clone(),
            name: component.name.clone(),
            version: component.version.clone(),
            assets: vec![BundleAsset {
                path: tarball.path.clone(),
                kind: tarball.kind.clone(),
                content_type: tarball.content_type.clone(),
                size: tarball.size as u64,
                sha1: tarball.sha1.clone(),
                sha256: tarball.sha256.clone(),
                sha512: tarball.sha512.clone(),
                md5: tarball.md5.clone(),
                attributes: tarball.attributes.clone(),
            }],
            meta: json!({"manifest": manifest, "time": time, "distTags": dist_tags}),
            origin: None,
        };
        Parcel::of(&bundle)?;
        Ok(bundle)
    }

    async fn receive(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        blobs: &dyn BlobSource,
        terms: Terms,
    ) -> Result<Outcome, ContentError> {
        let parcel = Parcel::of(bundle)?;
        hosted::writable(target)?;
        target.require(View::Add, &parcel.name.packument_path())?;
        let staged = parcel.staged(bundle);
        if terms.dry_run {
            let outcome = TARBALLS.receive(target, &staged, blobs, terms).await?;
            return self.planned(target, &parcel, outcome).await;
        }
        let _held = self.lock(target, &parcel.name).await;
        let outcome = TARBALLS.receive(target, &staged, blobs, terms).await?;
        match self.merge(target, &parcel, terms, outcome.clone()).await {
            Ok(merged) => Ok(merged),
            Err(error) => {
                self.forget(target, &outcome).await;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle() -> Bundle {
        Bundle {
            format: ID.to_owned(),
            namespace: "team".to_owned(),
            name: "tool".to_owned(),
            version: "1.0.0".to_owned(),
            assets: vec![BundleAsset {
                path: "/@team/tool/-/tool-1.0.0.tgz".to_owned(),
                kind: TARBALL.to_owned(),
                content_type: "application/x-tgz".to_owned(),
                size: 3,
                sha1: "a".repeat(40),
                sha256: "b".repeat(64),
                sha512: "c".repeat(128),
                md5: String::new(),
                attributes: json!({}),
            }],
            meta: json!({
                "manifest": {"name": "@team/tool", "version": "1.0.0"},
                "time": "2026-11-01T10:00:00.000Z",
                "distTags": ["latest", "beta"],
            }),
            origin: None,
        }
    }

    #[test]
    fn a_well_formed_bundle_is_read() {
        let parcel = Parcel::of(&bundle()).unwrap();
        assert_eq!(parcel.name.to_string(), "@team/tool");
        assert_eq!(parcel.dist_tags, ["latest", "beta"]);
        assert_eq!(parcel.path, "/@team/tool/-/tool-1.0.0.tgz");
    }

    #[test]
    fn bundles_of_another_shape_are_refused() {
        let breakers: [fn(&mut Bundle); 10] = [
            |bundle| bundle.format = "raw".to_owned(),
            |bundle| bundle.version = "1.0".to_owned(),
            |bundle| bundle.namespace = "@team".to_owned(),
            |bundle| bundle.name = "tool/extra".to_owned(),
            |bundle| bundle.meta["manifest"]["name"] = json!("other"),
            |bundle| bundle.meta["manifest"]["version"] = json!("2.0.0"),
            |bundle| bundle.meta["distTags"] = json!(["1.0.0"]),
            |bundle| bundle.assets[0].path = "/@team/tool/-/other-1.0.0.tgz".to_owned(),
            |bundle| bundle.assets[0].sha512.clear(),
            |bundle| bundle.assets.clear(),
        ];
        for (index, breaker) in breakers.into_iter().enumerate() {
            let mut broken = bundle();
            breaker(&mut broken);
            assert!(
                matches!(Parcel::of(&broken), Err(ContentError::BadRequest(_))),
                "breaker {index}"
            );
        }
    }

    #[test]
    fn a_publish_time_that_is_not_a_timestamp_falls_back_to_now() {
        for time in [json!("yesterday"), json!(5), Value::Null] {
            let mut odd = bundle();
            odd.meta["time"] = time;
            assert_eq!(Parcel::of(&odd).unwrap().time, None);
        }
    }

    #[test]
    fn latest_moves_up_or_stays_never_down() {
        let parcel = Parcel::of(&bundle()).unwrap();
        for (held, expected) in [
            (json!("0.9.0"), true),
            (json!("1.0.0"), true),
            (json!("1.0.0-rc.1"), true),
            (json!("1.0.1"), false),
            (json!("2.0.0"), false),
            (json!("not a version"), false),
            (json!(7), false),
        ] {
            let current = json!({"latest": held, "beta": "9.9.9"});
            let current = current.as_object().unwrap();
            assert_eq!(parcel.may_take_latest(current), expected, "{held}");
        }
        assert!(parcel.may_take_latest(&serde_json::Map::new()));
    }
}
