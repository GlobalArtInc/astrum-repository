use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use astrum_db::entity::asset;
use astrum_policy::key;
use astrum_policy::policy::Policy;
use astrum_policy::verdict::clean;
use astrum_signature::{
    Claim, MAX_BUNDLE_BYTES, MAX_PAYLOAD_BYTES, Outcome, PublicKey, verify_bundle,
    verify_simple_signing,
};
use sea_orm::{ConnectionTrait, DbBackend, EntityTrait, Statement};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use sha2::{Digest as _, Sha256};

use crate::reference::{Algorithm, Digest};
use crate::upstream::Upstreams;
use crate::{MANIFEST_LIMIT, storage};

pub const MAX_EXAMINED: usize = 200;
pub const MAX_VERIFIED: usize = MAX_EXAMINED;
pub const MAX_REMOTE: usize = 20;
pub const MAX_REMOTE_BLOBS: usize = 60;
pub const MAX_LISTED: usize = 500;
pub const MAX_LAYERS: usize = 20;
pub const MAX_PARENTS: usize = 20;
const SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";
const BUNDLE: &str = "application/vnd.dev.sigstore.bundle";
const NOTATION: &str = "application/vnd.cncf.notary.signature";
const SIGNATURE_ANNOTATION: &str = "dev.cosignproject.cosign/signature";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Signed,
    Unsigned,
    Invalid,
    Unsupported,
    Error,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Signed => "signed",
            State::Unsigned => "unsigned",
            State::Invalid => "invalid",
            State::Unsupported => "unsupported",
            State::Error => "error",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [
            State::Signed,
            State::Unsigned,
            State::Invalid,
            State::Unsupported,
            State::Error,
        ]
        .into_iter()
        .find(|state| state.as_str() == text)
    }
}

pub struct Keys {
    keys: Vec<PublicKey>,
    names: HashMap<String, String>,
    fingerprint: String,
}

impl Keys {
    pub fn new(trusted: Vec<(String, PublicKey)>) -> Self {
        let mut fingerprints: Vec<&str> =
            trusted.iter().map(|(_, key)| key.fingerprint()).collect();
        fingerprints.sort_unstable();
        let fingerprint = hex::encode(Sha256::digest(fingerprints.join("\n")));
        let names = trusted
            .iter()
            .map(|(name, key)| (key.fingerprint().to_owned(), name.clone()))
            .collect();
        Self {
            keys: trusted.into_iter().map(|(_, key)| key).collect(),
            names,
            fingerprint,
        }
    }

    pub async fn load(db: &impl ConnectionTrait, policy: &Policy) -> Result<Self, ContentError> {
        let stored = key::list(db).await.map_err(ContentError::internal)?;
        let mut trusted = Vec::new();
        for name in &policy.signatures.keys {
            let Some(row) = stored.iter().find(|row| &row.name == name) else {
                tracing::warn!(policy = %policy.name, key = %name, "a trusted key of the policy does not exist, so it is skipped");
                continue;
            };
            match PublicKey::from_pem(&row.public_key) {
                Ok(parsed) => trusted.push((name.clone(), parsed)),
                Err(error) => {
                    tracing::warn!(policy = %policy.name, key = %name, %error, "a trusted key of the policy cannot be read, so it is skipped");
                }
            }
        }
        Ok(Self::new(trusted))
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    fn name_of(&self, fingerprint: &str) -> String {
        self.names
            .get(fingerprint)
            .cloned()
            .unwrap_or_else(|| fingerprint.to_owned())
    }
}

struct Referrer {
    digest: String,
    path: String,
}

pub struct Subject {
    pub digest: Digest,
    referrers: Vec<Referrer>,
    legacy: Option<Asset>,
    truncated: bool,
}

pub struct Local {
    subjects: Vec<Subject>,
    truncated: bool,
}

pub fn identity(asset: &Asset) -> String {
    storage::digest_of(asset)
        .map(|digest| digest.to_string())
        .unwrap_or_else(|| format!("sha256:{}", asset.sha256))
}

impl Local {
    pub fn bare(digest: Digest) -> Self {
        Self {
            subjects: vec![Subject {
                digest,
                referrers: Vec::new(),
                legacy: None,
                truncated: false,
            }],
            truncated: false,
        }
    }

    pub fn chain(&self) -> Vec<Digest> {
        self.subjects
            .iter()
            .map(|subject| subject.digest.clone())
            .collect()
    }

    pub fn fingerprint(&self, keys: &Keys) -> String {
        let mut lines = Vec::new();
        for subject in &self.subjects {
            lines.push(format!("subject {}", subject.digest));
            for referrer in &subject.referrers {
                lines.push(format!("referrer {} {}", subject.digest, referrer.digest));
            }
            if let Some(legacy) = &subject.legacy {
                lines.push(format!("legacy {} {}", subject.digest, identity(legacy)));
            }
        }
        lines.sort_unstable();
        if self.truncated {
            lines.push("truncated".to_owned());
        }
        lines.push(format!("keys {}", keys.fingerprint()));
        hex::encode(Sha256::digest(lines.join("\n")))
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

pub async fn local(ctx: &Ctx, image: &str, digest: &Digest) -> Result<Local, ContentError> {
    let (parents, mut truncated) = parents_of(ctx, image, digest).await?;
    let mut subjects = Vec::with_capacity(parents.len() + 1);
    for subject in std::iter::once(digest.clone()).chain(parents) {
        let subject = subject_of(ctx, image, subject).await?;
        truncated |= subject.truncated;
        subjects.push(subject);
    }
    Ok(Local {
        subjects,
        truncated,
    })
}

async fn subject_of(ctx: &Ctx, image: &str, digest: Digest) -> Result<Subject, ContentError> {
    let rows = ctx
        .db()
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT attributes #>> '{docker,digest}' AS digest, path FROM asset \
             WHERE repository = $1 AND format = 'docker' AND kind = 'manifest' \
             AND attributes #>> '{docker,name}' = $2 \
             AND attributes #>> '{docker,subject}' = $3 \
             AND attributes #>> '{docker,artifactKind}' = 'signature' \
             ORDER BY created_at DESC, id DESC LIMIT $4",
            vec![
                ctx.name().into(),
                image.into(),
                digest.to_string().into(),
                (MAX_LISTED as i64 + 1).into(),
            ],
        ))
        .await?;
    let truncated = rows.len() > MAX_LISTED;
    let mut referrers = Vec::with_capacity(rows.len().min(MAX_LISTED));
    for row in rows.into_iter().take(MAX_LISTED) {
        let found: Option<String> = row.try_get("", "digest")?;
        if let Some(found) = found {
            referrers.push(Referrer {
                digest: found,
                path: row.try_get("", "path")?,
            });
        }
    }
    let tag = format!("{}.sig", digest.referrers_tag());
    let legacy = storage::find(ctx, &storage::manifest_path(image, &tag)).await?;
    Ok(Subject {
        digest,
        referrers,
        legacy,
        truncated,
    })
}

async fn parents_of(
    ctx: &Ctx,
    image: &str,
    child: &Digest,
) -> Result<(Vec<Digest>, bool), ContentError> {
    let rows = asset::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT DISTINCT ON (a.attributes #>> '{docker,digest}') a.* \
             FROM docker_manifest_child c \
             JOIN asset a ON a.repository = c.repository AND a.format = 'docker' \
              AND a.kind IN ('manifest', 'tag') \
              AND a.attributes #>> '{docker,name}' = $3 \
              AND a.attributes #>> '{docker,digest}' = c.parent \
             WHERE c.repository = $1 AND c.child = $2 \
             ORDER BY a.attributes #>> '{docker,digest}', a.kind DESC, a.path \
             LIMIT $4",
            vec![
                ctx.name().into(),
                child.to_string().into(),
                image.into(),
                (MAX_PARENTS as i64 + 1).into(),
            ],
        ))
        .all(ctx.db())
        .await?;
    let truncated = rows.len() > MAX_PARENTS;
    let parents = rows
        .iter()
        .take(MAX_PARENTS)
        .filter_map(storage::digest_of)
        .filter(|digest| digest.algorithm() == Algorithm::Sha256)
        .collect();
    Ok((parents, truncated))
}

#[derive(Debug)]
enum Finding {
    Signed(String),
    Attestation(String),
    Invalid(String),
    Unsupported(String),
    Failed(String),
}

fn finding_of(outcome: Outcome, keys: &Keys) -> Finding {
    match outcome {
        Outcome::Verified {
            key,
            claim: Claim::ImageSignature,
        } => Finding::Signed(keys.name_of(&key)),
        Outcome::Verified {
            key,
            claim: Claim::Attestation,
        } => Finding::Attestation(keys.name_of(&key)),
        Outcome::Invalid(reason) => Finding::Invalid(reason),
        Outcome::Unsupported(reason) => Finding::Unsupported(reason),
    }
}

struct Signed {
    signer: String,
    subject: Digest,
}

#[derive(Default)]
pub struct Tally {
    limited: bool,
    signed: Option<Signed>,
    attestation: Option<String>,
    invalid: Option<String>,
    unsupported: Option<String>,
    failed: Option<String>,
}

pub struct Verdict {
    pub state: State,
    pub signer: Option<String>,
    pub detail: Value,
}

impl Tally {
    fn absorb(&mut self, subject: &Digest, finding: Finding) {
        match finding {
            Finding::Signed(signer) => {
                self.signed.get_or_insert(Signed {
                    signer,
                    subject: subject.clone(),
                });
            }
            Finding::Attestation(signer) => {
                self.attestation.get_or_insert(signer);
            }
            Finding::Invalid(reason) => {
                self.invalid.get_or_insert(reason);
            }
            Finding::Unsupported(reason) => {
                self.unsupported.get_or_insert(reason);
            }
            Finding::Failed(reason) => {
                self.failed.get_or_insert(reason);
            }
        }
    }

    pub fn verified(&self) -> Option<&str> {
        self.signed.as_ref().map(|signed| signed.signer.as_str())
    }

    pub fn failed(&self) -> bool {
        self.failed.is_some()
    }

    pub fn invalid(&self) -> bool {
        self.invalid.is_some()
    }

    pub fn conclude(&self, digest: &Digest, truncated: bool) -> Verdict {
        let mut detail = Map::new();
        if truncated || self.limited {
            detail.insert("truncated".to_owned(), true.into());
        }
        if let Some(signer) = &self.attestation {
            detail.insert("attestation".to_owned(), clean(signer).into());
        }
        let (state, signer, reason) = if let Some(signed) = &self.signed {
            if &signed.subject != digest {
                detail.insert("via".to_owned(), signed.subject.to_string().into());
            }
            (State::Signed, Some(signed.signer.clone()), None)
        } else if let Some(reason) = &self.failed {
            (State::Error, None, Some(reason.clone()))
        } else if let Some(reason) = &self.invalid {
            (State::Invalid, None, Some(reason.clone()))
        } else if let Some(reason) = &self.unsupported {
            (State::Unsupported, None, Some(reason.clone()))
        } else if let Some(signer) = &self.attestation {
            (
                State::Unsigned,
                None,
                Some(format!(
                    "only an attestation signed by {} was found",
                    clean(signer)
                )),
            )
        } else {
            (State::Unsigned, None, None)
        };
        if let Some(reason) = reason {
            detail.insert("reason".to_owned(), clean(&reason).into());
        }
        Verdict {
            state,
            signer,
            detail: Value::Object(detail),
        }
    }
}

pub fn verdict_without_keys() -> Verdict {
    Verdict {
        state: State::Unsigned,
        signer: None,
        detail: json!({"reason": "the policy names no usable trusted key"}),
    }
}

pub fn verdict_for_unsupported_digest() -> Verdict {
    Verdict {
        state: State::Unsupported,
        signer: None,
        detail: json!({"reason": "only sha256 manifest digests can be verified"}),
    }
}

pub struct Env<'a> {
    pub ctx: &'a Ctx,
    pub image: &'a str,
    pub keys: &'a Keys,
    pub upstream: Option<&'a Upstreams>,
    blob_budget: AtomicUsize,
}

impl<'a> Env<'a> {
    pub fn new(
        ctx: &'a Ctx,
        image: &'a str,
        keys: &'a Keys,
        upstream: Option<&'a Upstreams>,
    ) -> Self {
        Self {
            ctx,
            image,
            keys,
            upstream,
            blob_budget: AtomicUsize::new(MAX_REMOTE_BLOBS),
        }
    }
}

pub async fn verify(env: &Env<'_>, local: &Local) -> Tally {
    let mut tally = Tally::default();
    for subject in &local.subjects {
        examine_subject(env, subject, &mut tally).await;
        if tally.signed.is_some() {
            break;
        }
    }
    tally
}

pub async fn examine_asset(env: &Env<'_>, subject: &Digest, asset: &Asset) -> Tally {
    let mut tally = Tally::default();
    match storage::read_all(env.ctx, asset, MANIFEST_LIMIT).await {
        Ok(bytes) => match serde_json::from_slice::<ArtifactManifest>(&bytes) {
            Ok(parsed) if parsed.carries_signature() => {
                consider_parsed(env, subject, &parsed, &mut tally, &mut Counts::default()).await;
            }
            Ok(_) => tally.absorb(
                subject,
                Finding::Unsupported("the artifact holds no signature layer".to_owned()),
            ),
            Err(_) => tally.absorb(
                subject,
                Finding::Invalid("the signature manifest is not valid JSON".to_owned()),
            ),
        },
        Err(error) => {
            tracing::warn!(repository = env.ctx.name(), path = %asset.path, %error, "a signature manifest cannot be read");
            tally.absorb(
                subject,
                Finding::Failed("a signature manifest cannot be read".to_owned()),
            );
        }
    }
    tally
}

#[derive(Default)]
struct Counts {
    read: usize,
    verified: usize,
}

async fn consider(
    env: &Env<'_>,
    subject: &Digest,
    bytes: &[u8],
    tally: &mut Tally,
    counts: &mut Counts,
) {
    if let Ok(parsed) = serde_json::from_slice::<ArtifactManifest>(bytes) {
        consider_parsed(env, subject, &parsed, tally, counts).await;
    }
}

async fn consider_parsed(
    env: &Env<'_>,
    subject: &Digest,
    parsed: &ArtifactManifest,
    tally: &mut Tally,
    counts: &mut Counts,
) {
    if parsed.artifact_type.as_deref() == Some(NOTATION) {
        tally.absorb(
            subject,
            Finding::Unsupported("Notation signatures are not verified".to_owned()),
        );
        return;
    }
    if !parsed.carries_signature() {
        return;
    }
    let (items, findings) = gather(env, subject, parsed).await;
    for finding in findings {
        tally.absorb(subject, finding);
    }
    if items.is_empty() {
        return;
    }
    if counts.verified >= MAX_VERIFIED {
        tally.limited = true;
        return;
    }
    counts.verified += 1;
    for item in &items {
        tally.absorb(subject, verify_item(env, subject, item));
        if tally.signed.is_some() {
            return;
        }
    }
}

async fn read_stored(
    env: &Env<'_>,
    subject: &Digest,
    asset: &Asset,
    tally: &mut Tally,
    counts: &mut Counts,
) {
    counts.read += 1;
    match storage::read_all(env.ctx, asset, MANIFEST_LIMIT).await {
        Ok(bytes) => consider(env, subject, &bytes, tally, counts).await,
        Err(error) => {
            tracing::warn!(repository = env.ctx.name(), path = %asset.path, %error, "a signature manifest cannot be read");
            tally.absorb(
                subject,
                Finding::Failed("a signature manifest cannot be read".to_owned()),
            );
        }
    }
}

async fn examine_subject(env: &Env<'_>, subject: &Subject, tally: &mut Tally) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut counts = Counts::default();
    if let Some(asset) = &subject.legacy {
        seen.insert(identity(asset));
        read_stored(env, &subject.digest, asset, tally, &mut counts).await;
    }
    for referrer in &subject.referrers {
        if tally.signed.is_some() {
            return;
        }
        if counts.read >= MAX_EXAMINED {
            tally.limited = true;
            break;
        }
        if !seen.insert(referrer.digest.clone()) {
            continue;
        }
        match storage::find(env.ctx, &referrer.path).await {
            Ok(Some(asset)) => {
                read_stored(env, &subject.digest, &asset, tally, &mut counts).await;
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(repository = env.ctx.name(), path = %referrer.path, %error, "a signature manifest cannot be looked up");
                tally.absorb(
                    &subject.digest,
                    Finding::Failed("a signature manifest cannot be read".to_owned()),
                );
            }
        }
    }
    let Some(upstream) = env.upstream else {
        return;
    };
    if tally.signed.is_some() {
        return;
    }
    let budget = MAX_REMOTE.min(MAX_VERIFIED.saturating_sub(counts.verified));
    if budget == 0 {
        tally.limited = true;
        return;
    }
    match remote_manifests(env, upstream, &subject.digest, &seen, budget).await {
        Ok(found) => {
            for bytes in found {
                consider(env, &subject.digest, &bytes, tally, &mut counts).await;
                if tally.signed.is_some() {
                    return;
                }
            }
        }
        Err(error) => {
            tracing::warn!(repository = env.ctx.name(), image = env.image, subject = %subject.digest, %error, "the signatures cannot be fetched from the upstream registry");
            tally.absorb(
                &subject.digest,
                Finding::Failed(
                    "the signatures cannot be fetched from the upstream registry".to_owned(),
                ),
            );
        }
    }
}

fn signature_like(descriptor: &Value) -> bool {
    match descriptor
        .get("artifactType")
        .and_then(Value::as_str)
        .filter(|artifact| !artifact.is_empty())
    {
        None => true,
        Some(artifact) => {
            let lower = artifact.to_ascii_lowercase();
            ["cosign", "sigstore", "notary", "signature"]
                .iter()
                .any(|needle| lower.contains(needle))
        }
    }
}

async fn remote_manifests(
    env: &Env<'_>,
    upstream: &Upstreams,
    subject: &Digest,
    seen: &HashSet<String>,
    budget: usize,
) -> Result<Vec<Vec<u8>>, ContentError> {
    let mut found = Vec::new();
    let mut known = seen.clone();
    let tag = format!("{}.sig", subject.referrers_tag());
    if let Some(bytes) = upstream
        .manifest_bytes(env.ctx, env.image, &tag, MANIFEST_LIMIT)
        .await?
        && known.insert(Digest::sha256(&bytes).to_string())
    {
        found.push(bytes);
    }
    let mut wanted: Vec<Digest> = upstream
        .referrers(env.ctx, env.image, subject)
        .await?
        .iter()
        .filter(|descriptor| signature_like(descriptor))
        .filter_map(|descriptor| descriptor.get("digest").and_then(Value::as_str))
        .filter_map(Digest::parse)
        .collect();
    wanted.sort_by_key(ToString::to_string);
    wanted.dedup();
    for digest in wanted {
        if found.len() >= budget {
            break;
        }
        if known.contains(&digest.to_string()) {
            continue;
        }
        let Some(bytes) = upstream
            .manifest_bytes(env.ctx, env.image, &digest.to_string(), MANIFEST_LIMIT)
            .await?
        else {
            continue;
        };
        if !digest.matches_bytes(&bytes) {
            return Err(ContentError::BadGateway(format!(
                "the remote sent a manifest that does not match {digest}"
            )));
        }
        known.insert(digest.to_string());
        found.push(bytes);
    }
    found.truncate(budget);
    Ok(found)
}

#[derive(Deserialize)]
struct ArtifactManifest {
    #[serde(rename = "artifactType", default)]
    artifact_type: Option<String>,
    #[serde(default)]
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
struct Layer {
    #[serde(rename = "mediaType", default)]
    media_type: String,
    #[serde(default)]
    digest: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    annotations: Map<String, Value>,
    #[serde(default)]
    urls: Vec<String>,
}

impl Layer {
    fn annotation(&self, name: &str) -> Option<&str> {
        self.annotations.get(name).and_then(Value::as_str)
    }

    fn is_signature(&self) -> bool {
        self.urls.is_empty()
            && (self.media_type == SIMPLE_SIGNING || self.media_type.starts_with(BUNDLE))
    }
}

impl ArtifactManifest {
    fn layers(&self) -> &[Layer] {
        &self.layers[..self.layers.len().min(MAX_LAYERS)]
    }

    fn carries_signature(&self) -> bool {
        self.artifact_type.as_deref() == Some(NOTATION)
            || self.layers().iter().any(Layer::is_signature)
    }
}

enum Item {
    Simple {
        signature: String,
        payload: Vec<u8>,
    },
    Bundle {
        bundle: Vec<u8>,
        message: Option<Vec<u8>>,
    },
}

fn names_subject(payload: &[u8], subject: &Digest) -> bool {
    serde_json::from_slice::<Value>(payload)
        .ok()
        .and_then(|parsed| {
            parsed
                .pointer("/critical/image/docker-manifest-digest")
                .and_then(Value::as_str)
                .map(|named| named == subject.to_string())
        })
        .unwrap_or(false)
}

type Fetched = HashMap<String, Vec<u8>>;

async fn load(env: &Env<'_>, fetched: &mut Fetched, digest: &str, limit: usize) -> Blob {
    if let Some(known) = fetched.get(digest) {
        return Blob::Found(known.clone());
    }
    let found = fetch_blob(
        env.ctx,
        env.image,
        env.upstream,
        Some(&env.blob_budget),
        digest,
        limit,
    )
    .await;
    if let Blob::Found(bytes) = &found {
        fetched.insert(digest.to_owned(), bytes.clone());
    }
    found
}

async fn gather(
    env: &Env<'_>,
    subject: &Digest,
    parsed: &ArtifactManifest,
) -> (Vec<Item>, Vec<Finding>) {
    let layers = parsed.layers();
    let has_bundle = layers
        .iter()
        .any(|layer| layer.is_signature() && layer.media_type.starts_with(BUNDLE));
    let payload = layers
        .iter()
        .find(|layer| layer.is_signature() && layer.media_type == SIMPLE_SIGNING);
    let mut fetched = Fetched::new();
    let message = match payload {
        Some(layer) if has_bundle && layer.size <= MAX_PAYLOAD_BYTES as u64 => {
            match load(env, &mut fetched, &layer.digest, MAX_PAYLOAD_BYTES).await {
                Blob::Found(message) => Some(message),
                Blob::Bad(_) | Blob::Lost | Blob::Failed => None,
            }
        }
        _ => None,
    };
    let mut items = Vec::new();
    let mut findings = Vec::new();
    for layer in layers.iter().filter(|layer| layer.is_signature()) {
        let gathered = if layer.media_type.starts_with(BUNDLE) {
            bundle_item(env, &mut fetched, layer, message.clone()).await
        } else if !has_bundle {
            simple_item(env, &mut fetched, subject, layer).await
        } else {
            continue;
        };
        match gathered {
            Ok(item) => items.push(item),
            Err(finding) => findings.push(finding),
        }
    }
    (items, findings)
}

async fn simple_item(
    env: &Env<'_>,
    fetched: &mut Fetched,
    subject: &Digest,
    layer: &Layer,
) -> Result<Item, Finding> {
    let Some(signature) = layer.annotation(SIGNATURE_ANNOTATION) else {
        return Err(Finding::Invalid(
            "a payload layer has no signature annotation".to_owned(),
        ));
    };
    if layer.size > MAX_PAYLOAD_BYTES as u64 {
        return Err(Finding::Invalid(format!(
            "a payload is larger than {MAX_PAYLOAD_BYTES} bytes"
        )));
    }
    match load(env, fetched, &layer.digest, MAX_PAYLOAD_BYTES).await {
        Blob::Found(payload) if names_subject(&payload, subject) => Ok(Item::Simple {
            signature: signature.to_owned(),
            payload,
        }),
        Blob::Found(_) => Err(Finding::Invalid(format!(
            "the payload does not name {subject}"
        ))),
        Blob::Bad(reason) => Err(Finding::Invalid(reason)),
        Blob::Lost | Blob::Failed => Err(Finding::Failed(
            "a signature payload cannot be read".to_owned(),
        )),
    }
}

async fn bundle_item(
    env: &Env<'_>,
    fetched: &mut Fetched,
    layer: &Layer,
    message: Option<Vec<u8>>,
) -> Result<Item, Finding> {
    if layer.size > MAX_BUNDLE_BYTES as u64 {
        return Err(Finding::Invalid(format!(
            "a bundle is larger than {MAX_BUNDLE_BYTES} bytes"
        )));
    }
    match load(env, fetched, &layer.digest, MAX_BUNDLE_BYTES).await {
        Blob::Found(bundle) => Ok(Item::Bundle { bundle, message }),
        Blob::Bad(reason) => Err(Finding::Invalid(reason)),
        Blob::Lost | Blob::Failed => Err(Finding::Failed(
            "a signature bundle cannot be read".to_owned(),
        )),
    }
}

fn verify_item(env: &Env<'_>, subject: &Digest, item: &Item) -> Finding {
    let outcome = match item {
        Item::Simple { signature, payload } => {
            verify_simple_signing(payload, signature, &env.keys.keys, &subject.to_string())
        }
        Item::Bundle { bundle, message } => verify_bundle(
            bundle,
            message.as_deref(),
            &env.keys.keys,
            &subject.to_string(),
        ),
    };
    finding_of(outcome, env.keys)
}

pub enum Blob {
    Found(Vec<u8>),
    Bad(String),
    Lost,
    Failed,
}

pub async fn fetch_blob(
    ctx: &Ctx,
    image: &str,
    upstream: Option<&Upstreams>,
    budget: Option<&AtomicUsize>,
    digest: &str,
    limit: usize,
) -> Blob {
    let Some(digest) = Digest::parse(digest) else {
        return Blob::Bad("a layer names a digest that is not supported".to_owned());
    };
    let stored = match storage::find(ctx, &storage::blob_path(&digest)).await {
        Ok(found) => found,
        Err(error) => {
            tracing::warn!(repository = ctx.name(), %digest, %error, "a signature blob cannot be looked up");
            return Blob::Failed;
        }
    };
    let bytes = match (stored, upstream) {
        (Some(asset), _) => match storage::read_all(ctx, &asset, limit).await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(repository = ctx.name(), %digest, %error, "a signature blob cannot be read");
                return Blob::Failed;
            }
        },
        (None, Some(upstream)) => {
            let spent = budget.is_some_and(|budget| {
                budget
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                        left.checked_sub(1)
                    })
                    .is_err()
            });
            if spent {
                tracing::warn!(repository = ctx.name(), %digest, "the limit of upstream fetches of one verification is reached");
                return Blob::Failed;
            }
            match upstream.blob_bytes(ctx, image, &digest, limit).await {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return Blob::Lost,
                Err(error) => {
                    tracing::warn!(repository = ctx.name(), %digest, %error, "a signature blob cannot be fetched from the upstream registry");
                    return Blob::Failed;
                }
            }
        }
        (None, None) => return Blob::Lost,
    };
    if !digest.matches_bytes(&bytes) {
        tracing::warn!(repository = ctx.name(), %digest, "a signature blob does not match its digest");
        return Blob::Failed;
    }
    Blob::Found(bytes)
}
