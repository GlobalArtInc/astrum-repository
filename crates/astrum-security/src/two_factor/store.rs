use astrum_db::revision;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rand::RngCore;
use sea_orm::{
    ConnectionTrait, DatabaseTransaction, DbBackend, DbErr, QueryResult, SqlErr, Statement,
    TransactionTrait, TryGetError, Value,
};
use serde_json::Value as Json;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::{Requirement, qr};
use crate::secrets::SecretBox;
use crate::tokens::hex;
use crate::totp::{self, TotpSecret};
use crate::{Security, recovery_codes};

pub const CEREMONY_ATTEMPTS: i32 = 5;
pub const ENROLL_LIFETIME: Duration = Duration::minutes(10);
pub const LOCK_THRESHOLD: i32 = 10;
pub const LOCK_BASE_SECONDS: i64 = 15 * 60;
pub const LOCK_CAP_SECONDS: i64 = 24 * 60 * 60;
pub const ACCEPT_WINDOW: u8 = 1;
pub const HINT_WINDOW: u8 = 10;

const LOCK_LEVEL_CAP: i32 = 16;
const HANDLE_BYTES: usize = 32;
const HANDLE_MAX_LENGTH: usize = 128;
const CODE_MAX_LENGTH: usize = 64;
const DUMMY_SECRET: [u8; totp::SECRET_BYTES] = [0x5a; totp::SECRET_BYTES];

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Internal(#[from] crate::Error),
    #[error("the database refused a query: {0}")]
    Query(DbErr),
    #[error("the user does not exist")]
    UnknownUser,
    #[error("no secrets key is configured, so a TOTP secret cannot be sealed")]
    NoSecretsKey,
    #[error("the user already has a TOTP factor")]
    AlreadyEnrolled,
    #[error("the user has no second factor")]
    NotEnrolled,
    #[error("the issuer and the account must not be empty")]
    InvalidLabel,
    #[error("the requirement cannot be set to this value")]
    InvalidRequirement,
    #[error("the QR code cannot be drawn: {0}")]
    Qr(#[from] qr::QrError),
    #[error("a recovery code could not be hashed")]
    Hashing,
}

impl From<DbErr> for StoreError {
    fn from(error: DbErr) -> Self {
        match error.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => StoreError::UnknownUser,
            _ => StoreError::Query(error),
        }
    }
}

impl From<TryGetError> for StoreError {
    fn from(error: TryGetError) -> Self {
        StoreError::Query(DbErr::from(error))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CeremonyKind {
    SignIn,
    TotpEnroll,
    PasskeyEnroll,
}

impl CeremonyKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CeremonyKind::SignIn => "sign-in",
            CeremonyKind::TotpEnroll => "totp-enroll",
            CeremonyKind::PasskeyEnroll => "passkey-enroll",
        }
    }

    fn from_stored(value: &str) -> Option<Self> {
        match value {
            "sign-in" => Some(CeremonyKind::SignIn),
            "totp-enroll" => Some(CeremonyKind::TotpEnroll),
            "passkey-enroll" => Some(CeremonyKind::PasskeyEnroll),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct NewCeremony<'a> {
    pub kind: CeremonyKind,
    pub user_id: &'a str,
    pub realm: &'a str,
    pub state: Option<Json>,
    pub client_ip: &'a str,
    pub lifetime: Duration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ceremony {
    pub kind: CeremonyKind,
    pub user_id: String,
    pub realm: String,
    pub state: Option<Json>,
    pub attempts: i32,
    pub client_ip: String,
    pub expires_at: DateTime<FixedOffset>,
}

pub struct PendingTotp {
    pub ceremony: String,
    pub secret: String,
    pub otpauth_uri: String,
    pub qr_svg: String,
    pub expires_at: DateTime<FixedOffset>,
}

impl std::fmt::Debug for PendingTotp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingTotp(<redacted>)")
    }
}

#[derive(PartialEq, Eq)]
pub struct RecoveryCodes(Vec<String>);

impl RecoveryCodes {
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for RecoveryCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoveryCodes(<redacted>)")
    }
}

impl Drop for RecoveryCodes {
    fn drop(&mut self) {
        for code in &mut self.0 {
            zeroize::Zeroize::zeroize(code);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Confirmation {
    Enrolled {
        recovery_codes: Option<RecoveryCodes>,
        offset_steps: i64,
    },
    ClockDrift {
        seconds_off: i64,
    },
    Rejected,
    Expired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verification {
    Accepted,
    Rejected,
    Locked { until: DateTime<FixedOffset> },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Detail {
    pub fresh_lock: bool,
    pub unreadable_secret: bool,
    pub offset_steps: Option<i8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifyReport {
    pub result: Verification,
    pub detail: Detail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryOutcome {
    Accepted { left: u32 },
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveOutcome {
    Removed,
    LastFactorRequired,
    NotEnrolled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResetOutcome {
    pub sessions_closed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Overview {
    pub totp: bool,
    pub passkeys: u32,
    pub recovery_codes_left: u32,
    pub requirement: Requirement,
    pub enroll_by: Option<DateTime<FixedOffset>>,
    pub locked_until: Option<DateTime<FixedOffset>>,
}

impl Overview {
    pub fn enrolled(&self) -> bool {
        self.totp || self.passkeys > 0
    }
}

fn statement(sql: &str, values: impl IntoIterator<Item = Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

fn count(row: &QueryResult, column: &str) -> Result<u32, TryGetError> {
    let value: i64 = row.try_get("", column)?;
    Ok(u32::try_from(value).unwrap_or(u32::MAX))
}

fn handle_hash(handle: &str) -> String {
    hex(&Sha256::digest(handle.as_bytes()))
}

fn unix(now: DateTime<Utc>) -> u64 {
    u64::try_from(now.timestamp()).unwrap_or(0)
}

fn squeezed(code: &str) -> String {
    if code.len() > CODE_MAX_LENGTH {
        return String::new();
    }
    code.split_whitespace().collect()
}

fn open_secret(secrets: Option<&SecretBox>, sealed: &str) -> Option<TotpSecret> {
    let plain = Zeroizing::new(secrets?.open(sealed).ok()?);
    TotpSecret::from_base32(&plain).ok()
}

fn ceremony_from_row(row: &QueryResult) -> Result<Option<Ceremony>, TryGetError> {
    let kind: String = row.try_get("", "kind")?;
    let Some(kind) = CeremonyKind::from_stored(&kind) else {
        return Ok(None);
    };
    Ok(Some(Ceremony {
        kind,
        user_id: row.try_get("", "user_id")?,
        realm: row.try_get("", "realm")?,
        state: row.try_get("", "state")?,
        attempts: row.try_get("", "attempts")?,
        client_ip: row.try_get("", "client_ip")?,
        expires_at: row.try_get("", "expires_at")?,
    }))
}

const EXPIRED_CEREMONIES: &str = "DELETE FROM second_factor_ceremony WHERE handle_hash IN \
     (SELECT handle_hash FROM second_factor_ceremony WHERE expires_at <= now() FOR UPDATE SKIP LOCKED)";
const CEREMONY_COLUMNS: &str = "kind, user_id, realm, state, attempts, client_ip, expires_at";

async fn insert_ceremony(
    conn: &impl ConnectionTrait,
    new: NewCeremony<'_>,
) -> Result<String, StoreError> {
    conn.execute(statement(EXPIRED_CEREMONIES, [])).await?;
    let mut bytes = [0u8; HANDLE_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let handle = hex(&bytes);
    conn.execute(statement(
        "INSERT INTO second_factor_ceremony (handle_hash, kind, user_id, realm, state, client_ip, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now() + make_interval(secs => $7::double precision))",
        [
            handle_hash(&handle).into(),
            new.kind.as_str().into(),
            new.user_id.into(),
            new.realm.into(),
            new.state.into(),
            new.client_ip.into(),
            (new.lifetime.num_milliseconds() as f64 / 1000.0).into(),
        ],
    ))
    .await?;
    Ok(handle)
}

async fn has_factor(conn: &impl ConnectionTrait, user_id: &str) -> Result<bool, StoreError> {
    let row = conn
        .query_one(statement(
            "SELECT EXISTS (SELECT 1 FROM user_second_factor WHERE user_id = $1 \
                 AND (totp_secret IS NOT NULL OR totp_confirmed_at IS NOT NULL)) \
             OR EXISTS (SELECT 1 FROM user_passkey WHERE user_id = $1) AS present",
            [user_id.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("factor check".to_owned()))?;
    Ok(row.try_get("", "present")?)
}

async fn webauthn_context(
    conn: &impl ConnectionTrait,
    user_id: &str,
) -> Result<Option<String>, StoreError> {
    let row = conn
        .query_one(statement(
            "SELECT webauthn_handle::text AS context FROM user_second_factor WHERE user_id = $1",
            [user_id.into()],
        ))
        .await?;
    row.map(|row| row.try_get("", "context"))
        .transpose()
        .map_err(Into::into)
}

pub async fn ensure_factor_row(
    conn: &impl ConnectionTrait,
    user_id: &str,
) -> Result<String, StoreError> {
    conn.execute(statement(
        "INSERT INTO user_second_factor (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
        [user_id.into()],
    ))
    .await?;
    webauthn_context(conn, user_id)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()).into())
}

struct LockedRow {
    totp: bool,
    passkeys: bool,
    requirement: Requirement,
}

async fn lock_user_row(
    txn: &DatabaseTransaction,
    user_id: &str,
    create: bool,
) -> Result<Option<LockedRow>, StoreError> {
    if create {
        txn.execute(statement(
            "INSERT INTO user_second_factor (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
            [user_id.into()],
        ))
        .await?;
    }
    let row = txn
        .query_one(statement(
            "SELECT requirement, (totp_secret IS NOT NULL OR totp_confirmed_at IS NOT NULL) AS totp, \
                    EXISTS (SELECT 1 FROM user_passkey WHERE user_id = $1) AS passkeys \
             FROM user_second_factor WHERE user_id = $1 FOR UPDATE",
            [user_id.into()],
        ))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(LockedRow {
        totp: row.try_get("", "totp")?,
        passkeys: row.try_get("", "passkeys")?,
        requirement: Requirement::from_stored(&row.try_get::<String>("", "requirement")?),
    }))
}

async fn issue_codes(context: String) -> Result<(RecoveryCodes, Vec<String>), StoreError> {
    let issued = tokio::task::spawn_blocking(move || {
        let codes = recovery_codes::generate_set();
        let hashes: Vec<String> = codes
            .iter()
            .filter_map(|code| recovery_codes::hash(&context, code))
            .collect();
        (codes, hashes)
    })
    .await
    .map_err(crate::Error::from)?;
    let (codes, hashes) = issued;
    if hashes.len() != codes.len() {
        return Err(StoreError::Hashing);
    }
    Ok((RecoveryCodes(codes), hashes))
}

async fn store_codes(
    conn: &impl ConnectionTrait,
    user_id: &str,
    hashes: &[String],
) -> Result<(), StoreError> {
    conn.execute(statement(
        "DELETE FROM user_recovery_code WHERE user_id = $1",
        [user_id.into()],
    ))
    .await?;
    if hashes.is_empty() {
        return Ok(());
    }
    let mut values: Vec<Value> = vec![user_id.into()];
    let mut rows = Vec::with_capacity(hashes.len());
    for (index, hash) in hashes.iter().enumerate() {
        values.push(hash.as_str().into());
        rows.push(format!("($1, ${})", index + 2));
    }
    let sql = format!(
        "INSERT INTO user_recovery_code (user_id, code_hash) VALUES {}",
        rows.join(", ")
    );
    conn.execute(statement(&sql, values)).await?;
    Ok(())
}

impl Security {
    pub async fn create_ceremony(&self, new: NewCeremony<'_>) -> Result<String, StoreError> {
        let txn = self.db().begin().await?;
        let handle = insert_ceremony(&txn, new).await?;
        txn.commit().await?;
        Ok(handle)
    }

    pub async fn replace_ceremony(&self, new: NewCeremony<'_>) -> Result<String, StoreError> {
        let user_id = new.user_id;
        let kind = new.kind;
        let txn = self.db().begin().await?;
        lock_user_row(&txn, user_id, true).await?;
        txn.execute(statement(
            "DELETE FROM second_factor_ceremony WHERE user_id = $1 AND kind = $2",
            [user_id.into(), kind.as_str().into()],
        ))
        .await?;
        let handle = insert_ceremony(&txn, new).await?;
        txn.commit().await?;
        Ok(handle)
    }

    pub async fn take_ceremony(
        &self,
        handle: &str,
        kind: CeremonyKind,
    ) -> Result<Option<Ceremony>, StoreError> {
        if handle.is_empty() || handle.len() > HANDLE_MAX_LENGTH {
            return Ok(None);
        }
        let row = self
            .db()
            .query_one(statement(
                &format!(
                    "DELETE FROM second_factor_ceremony \
                     WHERE handle_hash = $1 AND kind = $2 AND expires_at > now() \
                     RETURNING {CEREMONY_COLUMNS}"
                ),
                [handle_hash(handle).into(), kind.as_str().into()],
            ))
            .await?;
        Ok(row
            .map(|row| ceremony_from_row(&row))
            .transpose()?
            .flatten())
    }

    pub async fn attempt_ceremony(
        &self,
        handle: &str,
        kind: CeremonyKind,
        user_id: Option<&str>,
    ) -> Result<Option<Ceremony>, StoreError> {
        if handle.is_empty() || handle.len() > HANDLE_MAX_LENGTH {
            return Ok(None);
        }
        let row = self
            .db()
            .query_one(statement(
                &format!(
                    "UPDATE second_factor_ceremony SET attempts = attempts + 1 \
                     WHERE handle_hash = $1 AND kind = $2 AND expires_at > now() AND attempts < $4 \
                       AND ($3::text IS NULL OR user_id = $3::text) \
                     RETURNING {CEREMONY_COLUMNS}"
                ),
                [
                    handle_hash(handle).into(),
                    kind.as_str().into(),
                    user_id.map(str::to_owned).into(),
                    CEREMONY_ATTEMPTS.into(),
                ],
            ))
            .await?;
        Ok(row
            .map(|row| ceremony_from_row(&row))
            .transpose()?
            .flatten())
    }

    pub async fn expire_ceremonies(&self) -> Result<u64, StoreError> {
        let deleted = self.db().execute(statement(EXPIRED_CEREMONIES, [])).await?;
        Ok(deleted.rows_affected())
    }

    pub async fn begin_totp(
        &self,
        user_id: &str,
        realm: &str,
        issuer: &str,
        account: &str,
        client_ip: &str,
    ) -> Result<PendingTotp, StoreError> {
        if issuer.trim().is_empty() || account.trim().is_empty() {
            return Err(StoreError::InvalidLabel);
        }
        let secrets = self.secrets().ok_or(StoreError::NoSecretsKey)?;
        let secret = TotpSecret::generate();
        let uri = totp::otpauth_uri(issuer, account, &secret);
        let qr_svg = qr::svg(&uri)?;
        let text = Zeroizing::new(secret.to_base32());
        let sealed = secrets.seal(&text);
        let txn = self.db().begin().await?;
        let locked = lock_user_row(&txn, user_id, true)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()))?;
        if locked.totp {
            return Err(StoreError::AlreadyEnrolled);
        }
        txn.execute(statement(
            "DELETE FROM second_factor_ceremony WHERE user_id = $1 AND kind = $2",
            [user_id.into(), CeremonyKind::TotpEnroll.as_str().into()],
        ))
        .await?;
        let ceremony = insert_ceremony(
            &txn,
            NewCeremony {
                kind: CeremonyKind::TotpEnroll,
                user_id,
                realm,
                state: Some(Json::String(sealed)),
                client_ip,
                lifetime: ENROLL_LIFETIME,
            },
        )
        .await?;
        let expires = txn
            .query_one(statement(
                "SELECT expires_at FROM second_factor_ceremony WHERE handle_hash = $1",
                [handle_hash(&ceremony).into()],
            ))
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("ceremony".to_owned()))?;
        let expires_at: DateTime<FixedOffset> = expires.try_get("", "expires_at")?;
        txn.commit().await?;
        Ok(PendingTotp {
            ceremony,
            secret: text.to_string(),
            otpauth_uri: uri,
            qr_svg,
            expires_at,
        })
    }

    pub async fn confirm_totp(
        &self,
        user_id: &str,
        ceremony: &str,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<Confirmation, StoreError> {
        let Some(found) = self
            .attempt_ceremony(ceremony, CeremonyKind::TotpEnroll, Some(user_id))
            .await?
        else {
            return Ok(Confirmation::Expired);
        };
        let sealed = found.state.as_ref().and_then(Json::as_str);
        let secret = sealed.and_then(|sealed| open_secret(self.secrets(), sealed));
        let (Some(sealed), Some(secret)) = (sealed, secret) else {
            return Ok(Confirmation::Rejected);
        };
        let now_seconds = unix(now);
        let code = squeezed(code);
        let accepted = totp::matching_step(&secret, &code, now_seconds, ACCEPT_WINDOW);
        let nearby = totp::matching_step(&secret, &code, now_seconds, HINT_WINDOW);
        let current = totp::step_at(now_seconds);
        let Some(step) = accepted else {
            return Ok(match nearby {
                Some(step) => Confirmation::ClockDrift {
                    seconds_off: (i64::try_from(step).unwrap_or(i64::MAX)
                        - i64::try_from(current).unwrap_or(i64::MAX))
                        * i64::try_from(totp::PERIOD_SECONDS).unwrap_or(i64::MAX),
                },
                None => Confirmation::Rejected,
            });
        };
        let context = ensure_factor_row(self.db(), user_id).await?;
        let issued = if has_factor(self.db(), user_id).await? {
            None
        } else {
            Some(issue_codes(context).await?)
        };
        let txn = self.db().begin().await?;
        let locked = lock_user_row(&txn, user_id, true)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()))?;
        let taken = txn
            .execute(statement(
                "DELETE FROM second_factor_ceremony \
                 WHERE handle_hash = $1 AND kind = $2 AND user_id = $3 AND expires_at > now()",
                [
                    handle_hash(ceremony).into(),
                    CeremonyKind::TotpEnroll.as_str().into(),
                    user_id.into(),
                ],
            ))
            .await?;
        if taken.rows_affected() != 1 {
            return Ok(Confirmation::Expired);
        }
        if locked.totp {
            return Err(StoreError::AlreadyEnrolled);
        }
        txn.execute(statement(
            "UPDATE user_second_factor SET totp_secret = $2, totp_confirmed_at = now(), \
                 totp_last_step = $3, failed_codes = 0, locked_until = NULL, \
                 lock_level = 0, updated_at = now() \
             WHERE user_id = $1",
            [
                user_id.into(),
                sealed.into(),
                i64::try_from(step).unwrap_or(i64::MAX).into(),
            ],
        ))
        .await?;
        let recovery_codes = match issued {
            Some((codes, hashes)) if !locked.passkeys => {
                store_codes(&txn, user_id, &hashes).await?;
                Some(codes)
            }
            _ => None,
        };
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        self.follow(revision).await;
        Ok(Confirmation::Enrolled {
            recovery_codes,
            offset_steps: i64::try_from(step).unwrap_or(i64::MAX)
                - i64::try_from(current).unwrap_or(i64::MAX),
        })
    }

    pub async fn remove_totp(&self, user_id: &str) -> Result<RemoveOutcome, StoreError> {
        let txn = self.db().begin().await?;
        let Some(locked) = lock_user_row(&txn, user_id, false).await? else {
            return Ok(RemoveOutcome::NotEnrolled);
        };
        if !locked.totp {
            return Ok(RemoveOutcome::NotEnrolled);
        }
        if !locked.passkeys && self.effectively_required(locked.requirement, user_id) {
            return Ok(RemoveOutcome::LastFactorRequired);
        }
        txn.execute(statement(
            "UPDATE user_second_factor SET totp_secret = NULL, totp_confirmed_at = NULL, \
                 totp_last_step = 0, failed_codes = 0, locked_until = NULL, \
                 lock_level = 0, updated_at = now() \
             WHERE user_id = $1",
            [user_id.into()],
        ))
        .await?;
        if !locked.passkeys {
            store_codes(&txn, user_id, &[]).await?;
        }
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        self.follow(revision).await;
        Ok(RemoveOutcome::Removed)
    }

    pub async fn verify_totp(
        &self,
        user_id: &str,
        code: &str,
        now: DateTime<Utc>,
    ) -> Result<VerifyReport, StoreError> {
        let row = self
            .db()
            .query_one(statement(
                "SELECT totp_secret, \
                        (totp_secret IS NOT NULL AND locked_until > now()) AS locked, locked_until \
                 FROM user_second_factor WHERE user_id = $1",
                [user_id.into()],
            ))
            .await?;
        let (sealed, locked_until, locked) = match &row {
            Some(row) => (
                row.try_get::<Option<String>>("", "totp_secret")?,
                row.try_get::<Option<DateTime<FixedOffset>>>("", "locked_until")?,
                row.try_get::<Option<bool>>("", "locked")?.unwrap_or(false),
            ),
            None => (None, None, false),
        };
        let opened = sealed
            .as_deref()
            .and_then(|sealed| open_secret(self.secrets(), sealed));
        let unreadable_secret = sealed.is_some() && opened.is_none();
        if unreadable_secret {
            tracing::error!(user = %user_id, "a stored TOTP secret cannot be opened; the code is refused");
        }
        let real = opened.is_some();
        let secret = opened.unwrap_or_else(|| TotpSecret::from_bytes(DUMMY_SECRET));
        let now_seconds = unix(now);
        let matched = totp::matching_step(&secret, &squeezed(code), now_seconds, ACCEPT_WINDOW);
        let mut detail = Detail {
            unreadable_secret,
            ..Detail::default()
        };
        if let (true, Some(until)) = (locked, locked_until) {
            return Ok(VerifyReport {
                result: Verification::Locked { until },
                detail,
            });
        }
        if let (true, Some(step), Some(sealed)) = (real, matched, sealed.as_deref())
            && self.apply_step(user_id, sealed, step).await?
        {
            let offset = i64::try_from(step).unwrap_or(i64::MAX)
                - i64::try_from(totp::step_at(now_seconds)).unwrap_or(i64::MAX);
            detail.offset_steps = i8::try_from(offset).ok();
            return Ok(VerifyReport {
                result: Verification::Accepted,
                detail,
            });
        }
        let (result, fresh_lock) = self.record_failed_code(user_id).await?;
        detail.fresh_lock = fresh_lock;
        Ok(VerifyReport { result, detail })
    }

    async fn apply_step(&self, user_id: &str, sealed: &str, step: u64) -> Result<bool, StoreError> {
        let accepted = self
            .db()
            .execute(statement(
                "UPDATE user_second_factor SET totp_last_step = $2, failed_codes = 0, \
                     locked_until = NULL, lock_level = 0, updated_at = now() \
                 WHERE user_id = $1 AND totp_secret = $3 AND totp_confirmed_at IS NOT NULL \
                   AND totp_last_step < $2 AND (locked_until IS NULL OR locked_until <= now())",
                [
                    user_id.into(),
                    i64::try_from(step).unwrap_or(i64::MAX).into(),
                    sealed.into(),
                ],
            ))
            .await?;
        Ok(accepted.rows_affected() == 1)
    }

    async fn record_failed_code(&self, user_id: &str) -> Result<(Verification, bool), StoreError> {
        let row = self
            .db()
            .query_one(statement(
                "UPDATE user_second_factor SET \
                     failed_codes = CASE WHEN failed_codes + 1 >= $2 THEN 0 ELSE failed_codes + 1 END, \
                     locked_until = CASE WHEN failed_codes + 1 >= $2 \
                         THEN now() + make_interval(secs => LEAST($3::double precision \
                             * power(2::double precision, lock_level::double precision), $4::double precision)) \
                         ELSE locked_until END, \
                     lock_level = CASE WHEN failed_codes + 1 >= $2 \
                         THEN LEAST(lock_level + 1, $5) ELSE lock_level END, \
                     updated_at = now() \
                 WHERE user_id = $1 AND totp_secret IS NOT NULL \
                   AND (locked_until IS NULL OR locked_until <= now()) \
                 RETURNING (locked_until > now()) AS locked, locked_until, \
                           (failed_codes = 0) AS streak_ended",
                [
                    user_id.into(),
                    LOCK_THRESHOLD.into(),
                    (LOCK_BASE_SECONDS as f64).into(),
                    (LOCK_CAP_SECONDS as f64).into(),
                    LOCK_LEVEL_CAP.into(),
                ],
            ))
            .await?;
        let Some(row) = row else {
            return Ok((Verification::Rejected, false));
        };
        let locked = row.try_get::<Option<bool>>("", "locked")?.unwrap_or(false);
        let until = row.try_get::<Option<DateTime<FixedOffset>>>("", "locked_until")?;
        let fresh = row.try_get::<bool>("", "streak_ended")?;
        Ok(match (locked, until) {
            (true, Some(until)) => (Verification::Locked { until }, fresh),
            _ => (Verification::Rejected, false),
        })
    }

    pub async fn consume_recovery_code(
        &self,
        user_id: &str,
        input: &str,
    ) -> Result<RecoveryOutcome, StoreError> {
        if input.len() > CODE_MAX_LENGTH {
            return Ok(RecoveryOutcome::Rejected);
        }
        let context = webauthn_context(self.db(), user_id)
            .await?
            .unwrap_or_default();
        let input = input.to_owned();
        let hash = tokio::task::spawn_blocking(move || recovery_codes::hash(&context, &input))
            .await
            .map_err(crate::Error::from)?;
        let Some(hash) = hash else {
            return Ok(RecoveryOutcome::Rejected);
        };
        let txn = self.db().begin().await?;
        if lock_user_row(&txn, user_id, false).await?.is_none() {
            return Ok(RecoveryOutcome::Rejected);
        }
        let row = txn
            .query_one(statement(
                "WITH used AS (UPDATE user_recovery_code SET used_at = now() \
                     WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL RETURNING 1) \
                 SELECT (SELECT count(*) FROM used) AS taken, \
                        (SELECT count(*) FROM user_recovery_code \
                            WHERE user_id = $1 AND used_at IS NULL) AS remaining",
                [user_id.into(), hash.into()],
            ))
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("recovery code count".to_owned()))?;
        let taken = count(&row, "taken")?;
        if taken == 0 {
            return Ok(RecoveryOutcome::Rejected);
        }
        let remaining = count(&row, "remaining")?;
        txn.execute(statement(
            "UPDATE user_second_factor SET failed_codes = 0, \
                 locked_until = NULL, lock_level = 0, updated_at = now() WHERE user_id = $1",
            [user_id.into()],
        ))
        .await?;
        txn.commit().await?;
        Ok(RecoveryOutcome::Accepted {
            left: remaining.saturating_sub(taken),
        })
    }

    pub async fn regenerate_recovery_codes(
        &self,
        user_id: &str,
    ) -> Result<RecoveryCodes, StoreError> {
        if !has_factor(self.db(), user_id).await? {
            return Err(StoreError::NotEnrolled);
        }
        let context = ensure_factor_row(self.db(), user_id).await?;
        let (codes, hashes) = issue_codes(context).await?;
        let txn = self.db().begin().await?;
        let locked = lock_user_row(&txn, user_id, true)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()))?;
        if !locked.totp && !locked.passkeys {
            return Err(StoreError::NotEnrolled);
        }
        store_codes(&txn, user_id, &hashes).await?;
        txn.commit().await?;
        Ok(codes)
    }

    pub async fn set_requirement(
        &self,
        user_id: &str,
        requirement: Requirement,
    ) -> Result<(), StoreError> {
        if requirement == Requirement::Unreadable {
            return Err(StoreError::InvalidRequirement);
        }
        let txn = self.db().begin().await?;
        txn.execute(statement(
            "INSERT INTO user_second_factor (user_id, requirement) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO UPDATE SET requirement = EXCLUDED.requirement, updated_at = now()",
            [user_id.into(), requirement.as_str().into()],
        ))
        .await?;
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        self.follow(revision).await;
        Ok(())
    }

    pub async fn reset(&self, user_id: &str) -> Result<ResetOutcome, StoreError> {
        let txn = self.db().begin().await?;
        let locked = lock_user_row(&txn, user_id, true)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()))?;
        let required = self.effectively_required(locked.requirement, user_id);
        for table in [
            "user_passkey",
            "user_recovery_code",
            "second_factor_ceremony",
        ] {
            txn.execute(statement(
                &format!("DELETE FROM {table} WHERE user_id = $1"),
                [user_id.into()],
            ))
            .await?;
        }
        txn.execute(statement(
            "UPDATE user_second_factor SET totp_secret = NULL, totp_confirmed_at = NULL, \
                 totp_last_step = 0, failed_codes = 0, locked_until = NULL, lock_level = 0, \
                 enroll_by = CASE WHEN $2::boolean THEN now() ELSE enroll_by END, \
                 updated_at = now() \
             WHERE user_id = $1",
            [user_id.into(), required.into()],
        ))
        .await?;
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        let closed = self.close_sessions_of(user_id).await;
        self.follow(revision).await;
        Ok(ResetOutcome {
            sessions_closed: closed?,
        })
    }

    async fn follow(&self, revision: i64) {
        if let Err(error) = self.catch_up(revision).await {
            tracing::error!(%error, "the security model could not be reloaded after a second factor change; it is retried on the next sync");
        }
    }

    pub async fn summary(&self, user_id: &str) -> Result<Overview, StoreError> {
        let row = self
            .db()
            .query_one(statement(
                "SELECT f.requirement, f.enroll_by, \
                        COALESCE(f.totp_secret IS NOT NULL OR f.totp_confirmed_at IS NOT NULL, false) AS totp, \
                        CASE WHEN f.locked_until > now() THEN f.locked_until END AS locked_until, \
                        (SELECT count(*) FROM user_passkey WHERE user_id = $1) AS passkeys, \
                        (SELECT count(*) FROM user_recovery_code \
                            WHERE user_id = $1 AND used_at IS NULL) AS codes \
                 FROM (SELECT $1::text AS user_id) u \
                 LEFT JOIN user_second_factor f ON f.user_id = u.user_id",
                [user_id.into()],
            ))
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("factor summary".to_owned()))?;
        let requirement = row
            .try_get::<Option<String>>("", "requirement")?
            .map_or(Requirement::Policy, |stored| {
                Requirement::from_stored(&stored)
            });
        Ok(Overview {
            totp: row.try_get("", "totp")?,
            passkeys: count(&row, "passkeys")?,
            recovery_codes_left: count(&row, "codes")?,
            requirement,
            enroll_by: row.try_get("", "enroll_by")?,
            locked_until: row.try_get("", "locked_until")?,
        })
    }

    fn effectively_required(&self, requirement: Requirement, user_id: &str) -> bool {
        self.model().requires_second_factor(requirement, user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::totp::HMAC_CALLS;
    use astrum_testkit::TestDatabase;

    const USERS: [&str; 5] = ["good", "broken", "passkey", "ghost", "swapped"];

    async fn security() -> (TestDatabase, Security) {
        let test = TestDatabase::create().await;
        let db = test.migrated().await;
        for user in USERS {
            db.conn()
                .execute_unprepared(&format!(
                    "INSERT INTO security_user (id, first_name, last_name, email, status) \
                     VALUES ('{user}', '{user}', 'Tester', '{user}@example.org', 'active')"
                ))
                .await
                .unwrap();
        }
        let security = Security::load(db.conn().clone())
            .await
            .unwrap()
            .with_secrets(SecretBox::from_key([3u8; 32]));
        (test, security)
    }

    fn sealed(security: &Security, secret: &TotpSecret) -> String {
        security.secrets().unwrap().seal(&secret.to_base32())
    }

    async fn run(security: &Security, sql: &str) {
        security.db().execute_unprepared(sql).await.unwrap();
    }

    fn calls() -> u32 {
        HMAC_CALLS.with(|calls| calls.replace(0))
    }

    #[tokio::test]
    async fn every_kind_of_refusal_costs_the_same_hmac_work_as_an_acceptance() {
        let (_test, security) = security().await;
        let secret = TotpSecret::generate();
        let good = sealed(&security, &secret);
        run(
            &security,
            &format!(
                "INSERT INTO user_second_factor (user_id, totp_secret, totp_confirmed_at) VALUES \
                 ('good', '{good}', now()), ('broken', 'garbage', now())"
            ),
        )
        .await;
        run(
            &security,
            "INSERT INTO user_second_factor (user_id) VALUES ('passkey')",
        )
        .await;
        let now = Utc::now();
        let step = totp::step_at(unix(now));
        let right = totp::code_at(&secret, step);
        let wrong = if right == "000000" {
            "000001"
        } else {
            "000000"
        };
        for (user, code) in [
            ("ghost", wrong),
            ("passkey", wrong),
            ("broken", right.as_str()),
            ("good", wrong),
            ("good", right.as_str()),
        ] {
            calls();
            let report = security.verify_totp(user, code, now).await.unwrap();
            assert_eq!(calls(), 3, "{user} {code}");
            assert_eq!(
                report.result == Verification::Accepted,
                user == "good" && code == right,
                "{user}"
            );
        }
    }

    #[tokio::test]
    async fn a_step_is_refused_when_the_secret_was_replaced_after_it_was_read() {
        let (_test, security) = security().await;
        let old = sealed(&security, &TotpSecret::generate());
        let new = sealed(&security, &TotpSecret::generate());
        run(
            &security,
            &format!(
                "INSERT INTO user_second_factor (user_id, totp_secret, totp_confirmed_at) \
                 VALUES ('swapped', '{new}', now())"
            ),
        )
        .await;
        assert!(!security.apply_step("swapped", &old, 100).await.unwrap());
        assert!(security.apply_step("swapped", &new, 100).await.unwrap());
        assert!(!security.apply_step("swapped", &new, 100).await.unwrap());
        assert!(security.apply_step("swapped", &new, 101).await.unwrap());
    }
}
