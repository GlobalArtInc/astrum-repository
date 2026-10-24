use std::collections::BTreeMap;
use std::time::Duration;

use astrum_blob::ContentStream;
use astrum_content::proxy::{self, Freshness};
use astrum_content::store::{self, PutAsset};
use astrum_content::{Asset, ContentError, Ctx};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::form_urlencoded;

use crate::crx::{artifact_path, compare_versions};
use crate::gupdate::{self, RemoteApp};
use crate::hosted::{component_key, read_package};
use crate::{ARTIFACT, CRX_TYPE, ID};

pub const DEFAULT_PRODVERSION: &str = "156.0.0.0";
const RECORD: &str = "update";
const RECORD_FILE: &str = "updatecheck.json";
const BATCH: usize = 50;
const MAX_ANSWER: usize = 8 * 1024 * 1024;
const MAX_VERSIONS: usize = 16;
const DEFAULT_NEGATIVE_MINUTES: u64 = 1440;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Codebase {
    pub url: String,
    pub sha256: Option<String>,
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Record {
    pub latest: Option<String>,
    pub versions: BTreeMap<String, Codebase>,
}

impl Record {
    fn of(asset: &Asset) -> Option<Record> {
        serde_json::from_value(asset.attributes.get(ID)?.clone()).ok()
    }

    fn learn(&mut self, app: Option<&RemoteApp>) {
        let offered = app.and_then(|app| {
            app.offered()
                .map(|(version, codebase)| (app, version, codebase))
        });
        let Some((app, version, codebase)) = offered else {
            self.latest = None;
            return;
        };
        self.latest = Some(version.to_owned());
        self.versions.insert(
            version.to_owned(),
            Codebase {
                url: codebase.to_owned(),
                sha256: app.sha256.clone(),
                size: app.size,
            },
        );
        while self.versions.len() > MAX_VERSIONS {
            let oldest = self
                .versions
                .keys()
                .min_by(|left, right| compare_versions(left, right))
                .cloned()
                .expect("a map over its limit has keys");
            self.versions.remove(&oldest);
        }
    }

    fn offer(&self) -> Option<(String, Codebase)> {
        let latest = self.latest.clone()?;
        let codebase = self.versions.get(&latest)?.clone();
        Some((latest, codebase))
    }
}

pub fn record_path(id: &str) -> String {
    format!("/{id}/{RECORD_FILE}")
}

fn negative_ttl(ctx: &Ctx) -> Option<Duration> {
    let negative = ctx.repository.attributes.negative_cache.as_ref();
    if !negative
        .and_then(|negative| negative.enabled)
        .unwrap_or(true)
    {
        return None;
    }
    let minutes = negative
        .and_then(|negative| negative.time_to_live)
        .map(|minutes| minutes.max(0) as u64)
        .unwrap_or(DEFAULT_NEGATIVE_MINUTES);
    (minutes > 0).then(|| Duration::from_secs(minutes * 60))
}

fn fresh(ctx: &Ctx, asset: &Asset, record: &Record) -> bool {
    if record.latest.is_some() {
        return proxy::is_fresh(ctx, asset, Freshness::Metadata);
    }
    let (Some(ttl), Some(verified)) = (negative_ttl(ctx), asset.last_verified_at) else {
        return false;
    };
    let age = Utc::now().fixed_offset() - verified;
    age.to_std().is_ok_and(|age| age < ttl)
}

fn base_url(ctx: &Ctx) -> Result<url::Url, ContentError> {
    let remote = ctx
        .repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .ok_or_else(|| {
            ContentError::internal(std::io::Error::other(format!(
                "the proxy {} has no remote URL",
                ctx.name()
            )))
        })?;
    url::Url::parse(remote)
        .map_err(|error| ContentError::BadGateway(format!("the remote URL is not valid: {error}")))
}

pub fn update_query(ids: &[String], prodversion: &str) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("response", "updatecheck");
    serializer.append_pair("acceptformat", "crx3");
    serializer.append_pair("prodversion", prodversion);
    for id in ids {
        serializer.append_pair("x", &format!("id={id}&v=0.0.0.0&uc"));
    }
    serializer.finish()
}

pub type Offer = Option<(String, Codebase)>;

#[derive(Default)]
pub struct Remote;

impl Remote {
    async fn ask(
        &self,
        ctx: &Ctx,
        ids: &[String],
        prodversion: &str,
    ) -> Result<BTreeMap<String, RemoteApp>, ContentError> {
        let mut url = base_url(ctx)?;
        url.set_query(Some(&update_query(ids, prodversion)));
        let remote = proxy::request(ctx, url, &[]).await?;
        match remote.status {
            StatusCode::OK => {
                let bytes = remote.bytes(MAX_ANSWER).await?;
                gupdate::parse_answer(&bytes).map_err(|message| {
                    ContentError::BadGateway(format!(
                        "the remote sent a bad update answer: {message}"
                    ))
                })
            }
            StatusCode::NOT_FOUND | StatusCode::GONE => Ok(BTreeMap::new()),
            status => Err(ContentError::BadGateway(format!(
                "the remote answered {status} to the update check"
            ))),
        }
    }

    async fn save(
        &self,
        ctx: &Ctx,
        id: &str,
        known: Option<&Asset>,
        before: Option<&Record>,
        record: &Record,
    ) -> Result<(), ContentError> {
        if let (Some(asset), Some(before)) = (known, before)
            && before == record
        {
            return store::touch(ctx, asset.id, None, None).await;
        }
        if known.is_none() && record.latest.is_none() && negative_ttl(ctx).is_none() {
            return Ok(());
        }
        let put = PutAsset::cached(record_path(id))
            .kind(RECORD)
            .content_type("application/json")
            .attributes(json!({ ID: record }));
        store::put(ctx, put, astrum_blob::content_of(Vec::new())).await?;
        Ok(())
    }

    pub async fn lookup(
        &self,
        ctx: &Ctx,
        ids: &[String],
        prodversion: Option<&str>,
    ) -> Result<BTreeMap<String, Offer>, ContentError> {
        let mut offers: BTreeMap<String, Offer> = BTreeMap::new();
        let mut stale: Vec<(String, Option<(Asset, Record)>)> = Vec::new();
        for id in ids {
            let found = store::find(ctx.db(), ctx.name(), &record_path(id))
                .await?
                .and_then(|asset| Record::of(&asset).map(|record| (asset, record)));
            match found {
                Some((asset, record)) if fresh(ctx, &asset, &record) => {
                    offers.insert(id.clone(), record.offer());
                }
                other => stale.push((id.clone(), other)),
            }
        }
        let prodversion = prodversion.unwrap_or(DEFAULT_PRODVERSION);
        let mut unanswered = Vec::new();
        let mut failure = None;
        for batch in stale.chunks(BATCH) {
            let wanted: Vec<String> = batch.iter().map(|(id, _)| id.clone()).collect();
            match self.ask(ctx, &wanted, prodversion).await {
                Ok(apps) => {
                    for (id, known) in batch {
                        let before = known.as_ref().map(|(_, record)| record);
                        let mut record = before.cloned().unwrap_or_default();
                        record.learn(apps.get(id));
                        self.save(
                            ctx,
                            id,
                            known.as_ref().map(|(asset, _)| asset),
                            before,
                            &record,
                        )
                        .await?;
                        offers.insert(id.clone(), record.offer());
                    }
                }
                Err(error) => {
                    for (id, known) in batch {
                        match known {
                            Some((_, record)) => {
                                tracing::info!(repository = ctx.name(), id, %error, "answering from a stale update record");
                                offers.insert(id.clone(), record.offer());
                            }
                            None => unanswered.push(id.clone()),
                        }
                    }
                    failure = Some(error);
                }
            }
        }
        if let Some(error) = failure
            && offers.is_empty()
        {
            return Err(error);
        }
        for id in unanswered {
            offers.insert(id, None);
        }
        Ok(offers)
    }

    async fn codebase(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
    ) -> Result<Option<Codebase>, ContentError> {
        let found = store::find(ctx.db(), ctx.name(), &record_path(id))
            .await?
            .and_then(|asset| Record::of(&asset).map(|record| (asset, record)));
        if let Some((_, record)) = &found
            && let Some(codebase) = record.versions.get(version)
        {
            return Ok(Some(codebase.clone()));
        }
        if let Some((asset, record)) = &found
            && fresh(ctx, asset, record)
        {
            return Ok(None);
        }
        self.lookup(ctx, &[id.to_owned()], None).await?;
        Ok(store::find(ctx.db(), ctx.name(), &record_path(id))
            .await?
            .and_then(|asset| Record::of(&asset))
            .and_then(|record| record.versions.get(version).cloned()))
    }

    pub async fn fetch(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        let path = artifact_path(id, version);
        if let Some(cached) = store::find(ctx.db(), ctx.name(), &path).await? {
            return Ok(Some(cached));
        }
        let Some(codebase) = self.codebase(ctx, id, version).await? else {
            return Ok(None);
        };
        let url = url::Url::parse(&codebase.url).map_err(|error| {
            ContentError::BadGateway(format!("the remote named a bad codebase: {error}"))
        })?;
        let remote = proxy::request(ctx, url, &[]).await?;
        match remote.status {
            StatusCode::OK => self
                .adopt(ctx, id, version, &codebase, remote.body)
                .await
                .map(Some),
            StatusCode::NOT_FOUND | StatusCode::GONE => Ok(None),
            status => Err(ContentError::BadGateway(format!(
                "the remote answered {status} for {id} {version}"
            ))),
        }
    }

    async fn adopt(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
        codebase: &Codebase,
        body: ContentStream,
    ) -> Result<Asset, ContentError> {
        let written = store::write_blob(ctx, body).await?;
        let refuse = |message: String| {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            ContentError::BadGateway(message)
        };
        if let Some(expected) = &codebase.sha256
            && *expected != written.digests.sha256
        {
            return Err(refuse(format!(
                "the file the remote sent for {id} {version} does not match the hash_sha256 of its update answer"
            )));
        }
        if let Some(expected) = codebase.size
            && expected != written.size
        {
            return Err(refuse(format!(
                "the file the remote sent for {id} {version} has {} bytes, not the {expected} of its update answer",
                written.size
            )));
        }
        let package = match read_package(ctx, &written).await {
            Ok(package) => package,
            Err(refusal) => {
                return Err(refuse(format!(
                    "the remote did not send a CRX3 package: {}",
                    refusal.into_error()
                )));
            }
        };
        if package.id != id || package.manifest.version != version {
            return Err(refuse(format!(
                "the remote sent {} {} for {id} {version}",
                package.id, package.manifest.version
            )));
        }
        let path = artifact_path(id, version);
        if let Some(existing) = store::find(ctx.db(), ctx.name(), &path).await? {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Ok(existing);
        }
        let put = PutAsset::cached(path)
            .kind(ARTIFACT)
            .content_type(CRX_TYPE)
            .component(component_key(&package));
        store::adopt(ctx, put, &written).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered(version: &str) -> RemoteApp {
        RemoteApp {
            status: "ok".to_owned(),
            check_status: "ok".to_owned(),
            version: Some(version.to_owned()),
            codebase: Some(format!("https://cdn.test/{version}.crx")),
            sha256: None,
            size: Some(10),
        }
    }

    #[test]
    fn records_keep_the_newest_versions_and_forget_withdrawn_ones() {
        let mut record = Record::default();
        for minor in 0..20 {
            record.learn(Some(&offered(&format!("1.{minor}"))));
        }
        assert_eq!(record.latest.as_deref(), Some("1.19"));
        assert_eq!(record.versions.len(), MAX_VERSIONS);
        assert!(record.versions.contains_key("1.4"));
        assert!(!record.versions.contains_key("1.3"));
        let (version, codebase) = record.offer().unwrap();
        assert_eq!(version, "1.19");
        assert_eq!(codebase.url, "https://cdn.test/1.19.crx");
        record.learn(None);
        assert_eq!(record.offer(), None);
        assert!(record.versions.contains_key("1.19"));
    }

    #[test]
    fn the_remote_is_asked_for_the_newest_version_of_each_id() {
        let ids = vec!["a".repeat(32), "b".repeat(32)];
        assert_eq!(
            update_query(&ids, "131.0"),
            format!(
                "response=updatecheck&acceptformat=crx3&prodversion=131.0&x=id%3D{}%26v%3D0.0.0.0%26uc&x=id%3D{}%26v%3D0.0.0.0%26uc",
                ids[0], ids[1]
            )
        );
    }
}
