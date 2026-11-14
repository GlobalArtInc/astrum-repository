use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Duration;

use astrum_content::store::Asset;
use astrum_content::{ContentError, Ctx};
use astrum_policy::cache::{Cache, Snapshot};
use astrum_policy::policy::{Mode, Policy};
use astrum_policy::refusal;
use astrum_policy::verdict::{Target, clean, signature_exception};
use astrum_repo::RepositoryType;
use chrono::Utc;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::Value;
use tokio::sync::Mutex as Flight;

use crate::evidence::{self, Blob, Env, Keys, Local, State, Verdict};
use crate::exempt;
use crate::reference::{Algorithm, Digest, Reference};
use crate::upstream::Upstreams;
use crate::{ID, MANIFEST_LIMIT, storage};

const UNAVAILABLE: &str = "the content policy could not be checked; try again";
const PROXY_RECHECK: Duration = Duration::from_secs(300);
const ERROR_RECHECK: Duration = Duration::from_secs(30);
const EXEMPT_STATE: &str = "exempt";
const NOT_EXEMPT_STATE: &str = "not-exempt";
const MEMO_RULES: &str = "content-1";
const EXEMPT_KINDS: [&str; 3] = ["signature", "attestation", "sbom"];

#[derive(Default)]
pub struct SignedOnly {
    policies: Cache,
    keys: Mutex<KeySlot>,
    flights: Mutex<HashMap<String, Weak<Flight<()>>>>,
}

#[derive(Default)]
struct KeySlot {
    revision: Option<i64>,
    resolved: HashMap<String, (i64, Arc<Keys>)>,
}

struct Decision {
    state: State,
    detail: Value,
    chain: Vec<Digest>,
}

struct Row {
    fingerprint: String,
    state: String,
    detail: Value,
    age: f64,
}

struct Query<'a> {
    upstreams: &'a Upstreams,
    holder: &'a Ctx,
    image: &'a str,
    digest: &'a Digest,
    snapshot: &'a Snapshot,
    policy: &'a Policy,
}

impl Row {
    fn usable(&self, fingerprint: &str, kind: RepositoryType, mode: Mode) -> bool {
        let errored = self.state == State::Error.as_str();
        let settled = !errored || (mode == Mode::Report && self.age < ERROR_RECHECK.as_secs_f64());
        let stale_negative = kind == RepositoryType::Proxy
            && !errored
            && self.state != State::Signed.as_str()
            && self.age >= PROXY_RECHECK.as_secs_f64();
        self.fingerprint == fingerprint && settled && !stale_negative
    }
}

fn unchecked(error: astrum_policy::Error) -> ContentError {
    tracing::error!(%error, "the content policies could not be read");
    ContentError::Unavailable(UNAVAILABLE.to_owned())
}

fn short(digest: &Digest) -> String {
    format!("{}:{}", digest.algorithm().name(), &digest.hex()[..12])
}

fn refusal_of(state: State, policy: &str, digest: &Digest, reason: Option<&str>) -> ContentError {
    let (short, policy) = (short(digest), clean(policy));
    match (state, reason) {
        (State::Error, reason) => ContentError::Unavailable(format!(
            "signature: {short} could not be checked against policy {policy}{}; try again",
            reason
                .map(|reason| format!(": {reason}"))
                .unwrap_or_default()
        )),
        (_, Some(reason)) => ContentError::Blocked(format!(
            "signature: {short} is not signed by a trusted key of policy {policy} ({reason})"
        )),
        (_, None) => ContentError::Blocked(format!(
            "signature: {short} is not signed by a trusted key of policy {policy}"
        )),
    }
}

fn unfinished(policy: &str, digest: &Digest) -> ContentError {
    ContentError::Unavailable(format!(
        "signature: {} could not be checked against policy {}; try again",
        short(digest),
        clean(policy)
    ))
}

fn digest_of(reference: &Reference, asset: &Asset) -> Result<Digest, ContentError> {
    match reference {
        Reference::Digest(digest) => Ok(digest.clone()),
        Reference::Tag(_) => Digest::parse(&evidence::identity(asset)).ok_or_else(|| {
            ContentError::internal(std::io::Error::other(format!(
                "{} names no digest",
                asset.path
            )))
        }),
    }
}

fn target(image: &str, version: &str, digest: &Digest) -> Target {
    Target {
        format: ID.to_owned(),
        namespace: String::new(),
        name: image.to_owned(),
        version: version.to_owned(),
        digest: Some(digest.to_string()),
    }
}

async fn read_row(
    db: &impl ConnectionTrait,
    repository: &str,
    key: &str,
) -> Result<Option<Row>, ContentError> {
    let Some(row) = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT fingerprint, state, detail, \
             EXTRACT(EPOCH FROM (now() - checked_at))::float8 AS age \
             FROM oci_signature_check WHERE repository = $1 AND digest = $2",
            vec![repository.into(), key.into()],
        ))
        .await?
    else {
        return Ok(None);
    };
    Ok(Some(Row {
        fingerprint: row.try_get("", "fingerprint")?,
        state: row.try_get("", "state")?,
        detail: row.try_get("", "detail")?,
        age: row.try_get("", "age")?,
    }))
}

enum Memo {
    Exempt,
    NotExempt,
}

enum Content {
    Exempt,
    Rejected,
    Unsure,
}

fn memo_key(digest: &Digest) -> String {
    format!("content:{digest}")
}

async fn read_memo(
    db: &impl ConnectionTrait,
    repository: &str,
    digest: &Digest,
    kind: RepositoryType,
) -> Result<Option<Memo>, ContentError> {
    let Some(row) = read_row(db, repository, &memo_key(digest)).await? else {
        return Ok(None);
    };
    if row.fingerprint != MEMO_RULES {
        return Ok(None);
    }
    Ok(match row.state.as_str() {
        EXEMPT_STATE => Some(Memo::Exempt),
        NOT_EXEMPT_STATE
            if kind != RepositoryType::Proxy || row.age < PROXY_RECHECK.as_secs_f64() =>
        {
            Some(Memo::NotExempt)
        }
        _ => None,
    })
}

async fn write_memo(
    db: &impl ConnectionTrait,
    repository: &str,
    digest: &Digest,
    state: &str,
    revision: i64,
) {
    let written = db
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO oci_signature_check \
             (repository, digest, fingerprint, keys_revision, state, detail, checked_at) \
             VALUES ($1, $2, $3, $4, $5, '{}', now()) \
             ON CONFLICT (repository, digest) DO UPDATE SET \
             fingerprint = EXCLUDED.fingerprint, keys_revision = EXCLUDED.keys_revision, \
             state = EXCLUDED.state, signer = NULL, detail = EXCLUDED.detail, \
             checked_at = now()",
            vec![
                repository.into(),
                memo_key(digest).into(),
                MEMO_RULES.into(),
                revision.into(),
                state.into(),
            ],
        ))
        .await;
    if let Err(error) = written {
        tracing::error!(repository, %digest, %error, "the content check of a signature artifact was not recorded");
    }
}

impl SignedOnly {
    fn flight(&self, key: String) -> Arc<Flight<()>> {
        let mut flights = self.flights.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = flights.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        flights.retain(|_, weak| weak.strong_count() > 0);
        let fresh = Arc::new(Flight::new(()));
        flights.insert(key, Arc::downgrade(&fresh));
        fresh
    }

    fn slot(&self) -> MutexGuard<'_, KeySlot> {
        self.keys.lock().unwrap_or_else(PoisonError::into_inner)
    }

    async fn snapshot(&self, ctx: &Ctx, name: &str) -> Result<Arc<Snapshot>, ContentError> {
        let mut snapshot = self.policies.get(ctx.db()).await.map_err(unchecked)?;
        if !snapshot.policies.contains_key(name) {
            snapshot = self.policies.fresh(ctx.db()).await.map_err(unchecked)?;
        }
        Ok(snapshot)
    }

    async fn keys_for(
        &self,
        db: &impl ConnectionTrait,
        revision: i64,
        policy: &Policy,
    ) -> Result<Arc<Keys>, ContentError> {
        {
            let slot = self.slot();
            if slot.revision == Some(revision)
                && let Some((held, keys)) = slot.resolved.get(&policy.name)
                && *held == policy.revision
            {
                return Ok(keys.clone());
            }
        }
        let loaded = Arc::new(Keys::load(db, policy).await?);
        let mut slot = self.slot();
        if slot.revision != Some(revision) {
            slot.revision = Some(revision);
            slot.resolved.clear();
        }
        slot.resolved
            .insert(policy.name.clone(), (policy.revision, loaded.clone()));
        Ok(loaded)
    }

    pub async fn require(
        &self,
        upstreams: &Upstreams,
        holder: &Ctx,
        image: &str,
        reference: &Reference,
        asset: &Asset,
    ) -> Result<(), ContentError> {
        let Some(name) = holder.repository.policy_name() else {
            return Ok(());
        };
        let snapshot = self.snapshot(holder, name).await?;
        let Some(Ok(policy)) = snapshot.policies.get(name).map(|stored| &stored.policy) else {
            return Err(ContentError::blocked_by(&clean(name), refusal::UNREADABLE));
        };
        let mode = policy.signatures.mode;
        if mode == Mode::Off {
            return Ok(());
        }
        let digest = digest_of(reference, asset)?;
        let query = Query {
            upstreams,
            holder,
            image,
            digest: &digest,
            snapshot: &snapshot,
            policy,
        };
        let decision = match self.judge(&query, asset).await {
            Ok(Some(decision)) => decision,
            Ok(None) => return Ok(()),
            Err(error) => {
                tracing::error!(repository = holder.name(), image, %digest, %error, "the signature check could not be completed");
                return match mode {
                    Mode::Block => Err(unfinished(name, &digest)),
                    _ => Ok(()),
                };
            }
        };
        if decision.state == State::Signed || mode != Mode::Block {
            return Ok(());
        }
        let tag = match reference {
            Reference::Tag(tag) => tag.as_str(),
            Reference::Digest(_) => "",
        };
        let now = Utc::now();
        for (index, candidate) in decision.chain.iter().enumerate() {
            let version = if index == 0 { tag } else { "" };
            let covered = target(image, version, candidate);
            if signature_exception(&snapshot.exceptions, name, &covered, now).is_some() {
                return Ok(());
            }
        }
        let reason = decision
            .detail
            .get("reason")
            .and_then(Value::as_str)
            .map(str::to_owned);
        Err(refusal_of(decision.state, name, &digest, reason.as_deref()))
    }

    async fn exempt_artifact(
        &self,
        query: &Query<'_>,
        asset: &Asset,
    ) -> Result<bool, ContentError> {
        let Query {
            holder,
            digest,
            snapshot,
            ..
        } = *query;
        let kind = asset
            .attributes
            .pointer("/docker/artifactKind")
            .and_then(Value::as_str);
        if !kind.is_some_and(|kind| EXEMPT_KINDS.contains(&kind)) {
            return Ok(false);
        }
        let db = holder.db();
        let remembered = read_memo(db, holder.name(), digest, holder.kind()).await?;
        if let Some(memo) = remembered {
            return Ok(matches!(memo, Memo::Exempt));
        }
        let flight = (holder.kind() == RepositoryType::Proxy)
            .then(|| self.flight(format!("content|{}|{digest}", holder.name())));
        let _held = match &flight {
            Some(flight) => Some(flight.lock().await),
            None => None,
        };
        if flight.is_some() {
            let again = read_memo(db, holder.name(), digest, holder.kind()).await?;
            if let Some(memo) = again {
                return Ok(matches!(memo, Memo::Exempt));
            }
        }
        match self.check_content(query, asset).await? {
            Content::Exempt => {
                write_memo(db, holder.name(), digest, EXEMPT_STATE, snapshot.revision).await;
                Ok(true)
            }
            Content::Rejected => {
                write_memo(
                    db,
                    holder.name(),
                    digest,
                    NOT_EXEMPT_STATE,
                    snapshot.revision,
                )
                .await;
                Ok(false)
            }
            Content::Unsure => Ok(false),
        }
    }

    async fn check_content(
        &self,
        query: &Query<'_>,
        asset: &Asset,
    ) -> Result<Content, ContentError> {
        let Query {
            upstreams,
            holder,
            image,
            ..
        } = *query;
        let bytes = storage::read_all(holder, asset, MANIFEST_LIMIT).await?;
        let Some(layers) = exempt::signature_artifact(&bytes) else {
            return Ok(Content::Rejected);
        };
        if layers
            .iter()
            .any(|layer| layer.size > exempt::MAX_LAYER_BYTES)
        {
            return Ok(Content::Rejected);
        }
        let proxy = holder.kind() == RepositoryType::Proxy;
        let upstream = proxy.then_some(upstreams);
        let limit = exempt::MAX_LAYER_BYTES as usize;
        let mut seen = HashSet::new();
        for layer in &layers {
            if !seen.insert((layer.digest.as_str(), layer.media_type.as_str())) {
                continue;
            }
            let found =
                evidence::fetch_blob(holder, image, upstream, None, &layer.digest, limit).await;
            match found {
                Blob::Found(content)
                    if content.len() as u64 == layer.size
                        && exempt::json_content(&layer.media_type, &content) => {}
                Blob::Found(_) | Blob::Bad(_) => return Ok(Content::Rejected),
                Blob::Lost => {
                    return Ok(match proxy {
                        true => Content::Rejected,
                        false => Content::Unsure,
                    });
                }
                Blob::Failed => return Ok(Content::Unsure),
            }
        }
        Ok(Content::Exempt)
    }

    async fn judge(
        &self,
        query: &Query<'_>,
        asset: &Asset,
    ) -> Result<Option<Decision>, ContentError> {
        if self.exempt_artifact(query, asset).await? {
            return Ok(None);
        }
        self.decide(query).await.map(Some)
    }

    async fn decide(&self, query: &Query<'_>) -> Result<Decision, ContentError> {
        let Query {
            upstreams,
            holder,
            image,
            digest,
            snapshot,
            policy,
        } = *query;
        let db = holder.db();
        let keys = self.keys_for(db, snapshot.revision, policy).await?;
        let sha256 = digest.algorithm() == Algorithm::Sha256;
        let local = match sha256 {
            true => evidence::local(holder, image, digest).await?,
            false => Local::bare(digest.clone()),
        };
        let fingerprint = local.fingerprint(&keys);
        let key = digest.to_string();
        let mode = policy.signatures.mode;
        let reuse = |row: &Option<Row>| {
            let row = row.as_ref()?;
            let state = State::parse(&row.state)?;
            row.usable(&fingerprint, holder.kind(), mode)
                .then(|| Decision {
                    state,
                    detail: row.detail.clone(),
                    chain: local.chain(),
                })
        };
        let mut prior = read_row(db, holder.name(), &key).await?;
        if let Some(decision) = reuse(&prior) {
            return Ok(decision);
        }
        let flight = (holder.kind() == RepositoryType::Proxy)
            .then(|| self.flight(format!("verify|{}|{key}", holder.name())));
        let _held = match &flight {
            Some(flight) => Some(flight.lock().await),
            None => None,
        };
        if flight.is_some() {
            prior = read_row(db, holder.name(), &key).await?;
            if let Some(decision) = reuse(&prior) {
                return Ok(decision);
            }
        }
        let verdict = if !sha256 {
            evidence::verdict_for_unsupported_digest()
        } else if keys.is_empty() {
            evidence::verdict_without_keys()
        } else {
            let upstream = (holder.kind() == RepositoryType::Proxy).then_some(upstreams);
            let env = Env::new(holder, image, &keys, upstream);
            evidence::verify(&env, &local)
                .await
                .conclude(digest, local.truncated())
        };
        self.record(query, &fingerprint, prior.as_ref(), &verdict)
            .await;
        Ok(Decision {
            state: verdict.state,
            detail: verdict.detail,
            chain: local.chain(),
        })
    }

    async fn record(
        &self,
        query: &Query<'_>,
        fingerprint: &str,
        prior: Option<&Row>,
        verdict: &Verdict,
    ) {
        let (holder, digest) = (query.holder, query.digest);
        let written = holder
            .db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO oci_signature_check \
                 (repository, digest, fingerprint, keys_revision, state, signer, detail, checked_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, now()) \
                 ON CONFLICT (repository, digest) DO UPDATE SET \
                 fingerprint = EXCLUDED.fingerprint, keys_revision = EXCLUDED.keys_revision, \
                 state = EXCLUDED.state, signer = EXCLUDED.signer, detail = EXCLUDED.detail, \
                 checked_at = now()",
                vec![
                    holder.name().into(),
                    digest.to_string().into(),
                    fingerprint.into(),
                    query.snapshot.revision.into(),
                    verdict.state.as_str().into(),
                    verdict.signer.clone().into(),
                    verdict.detail.clone().into(),
                ],
            ))
            .await;
        if let Err(error) = written {
            tracing::error!(repository = holder.name(), %digest, %error, "the signature state of a manifest was not recorded");
            return;
        }
        let changed = prior.map(|row| row.state.as_str()) != Some(verdict.state.as_str());
        if changed {
            tracing::info!(repository = holder.name(), image = query.image, %digest, policy = %query.policy.name, state = verdict.state.as_str(), signer = verdict.signer.as_deref().unwrap_or_default(), "the signature state of a manifest changed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> Digest {
        Digest::sha256(b"manifest")
    }

    #[test]
    fn refusals_name_the_short_digest_and_the_cleaned_policy() {
        let digest = digest();
        let ContentError::Blocked(text) =
            refusal_of(State::Unsigned, "base\u{202e}\n", &digest, None)
        else {
            panic!("an unsigned image is blocked");
        };
        assert_eq!(
            text,
            format!(
                "signature: {} is not signed by a trusted key of policy base",
                short(&digest)
            )
        );
        assert_eq!(short(&digest).len(), "sha256:".len() + 12);
        let ContentError::Blocked(text) = refusal_of(
            State::Invalid,
            "base",
            &digest,
            Some("no trusted key verifies it"),
        ) else {
            panic!("an invalid signature is blocked");
        };
        assert!(text.ends_with("(no trusted key verifies it)"), "{text}");
        let unfinished = refusal_of(State::Error, "base", &digest, Some("the upstream is down"));
        assert!(
            matches!(&unfinished, ContentError::Unavailable(text) if text.contains("the upstream is down")),
            "{unfinished:?}"
        );
    }

    #[test]
    fn a_stored_state_is_reused_only_while_its_inputs_hold() {
        let held = |state: &str, age: f64| Row {
            fingerprint: "f".to_owned(),
            state: state.to_owned(),
            detail: Value::Null,
            age,
        };
        let (hosted, proxy) = (RepositoryType::Hosted, RepositoryType::Proxy);
        let (block, report) = (Mode::Block, Mode::Report);
        assert!(held("unsigned", 1e9).usable("f", hosted, block));
        assert!(!held("unsigned", 0.0).usable("g", hosted, block));
        assert!(!held("error", 0.0).usable("f", hosted, block));
        assert!(held("error", 1.0).usable("f", hosted, report));
        assert!(!held("error", 31.0).usable("f", hosted, report));
        assert!(held("signed", 1e9).usable("f", proxy, block));
        assert!(held("unsigned", 1.0).usable("f", proxy, block));
        assert!(!held("unsigned", 1e9).usable("f", proxy, block));
    }
}
