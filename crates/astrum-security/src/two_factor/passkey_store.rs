use chrono::{DateTime, FixedOffset};
use sea_orm::{ConnectionTrait, DbErr, QueryResult, TransactionTrait, TryGetError};
use serde_json::{Value as Json, json};
use uuid::Uuid;

use super::passkeys::{self, Passkey, PasskeyError, RelyingParty};
use super::store::{
    CeremonyKind, ENROLL_LIFETIME, NewCeremony, RecoveryCodes, StoreError, ensure_factor_row,
    handle_hash, has_factor, insert_ceremony, issue_codes, lock_user_row, statement, store_codes,
};
use crate::Security;
use astrum_db::revision;

pub const MAX_PASSKEYS: i64 = 10;
pub const NAME_MAX_CHARS: usize = 64;

const COLUMNS: &str = "id, name, credential_id, rp_id, passkey, backup_eligible, backed_up, \
                       created_at, last_used_at";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasskeyRecord {
    pub id: Uuid,
    pub name: String,
    pub rp_id: String,
    pub backup_eligible: bool,
    pub backed_up: bool,
    pub created_at: DateTime<FixedOffset>,
    pub last_used_at: Option<DateTime<FixedOffset>>,
}

pub(super) struct Row {
    pub(super) record: PasskeyRecord,
    pub(super) credential_id: Vec<u8>,
    pub(super) passkey: Json,
}

impl Row {
    pub(super) fn credential(&self) -> Option<Passkey> {
        match serde_json::from_value(self.passkey.clone()) {
            Ok(passkey) => Some(passkey),
            Err(_) => {
                tracing::error!(passkey = %self.record.id, "a stored passkey cannot be read and is ignored");
                None
            }
        }
    }
}

pub struct BeginPasskey<'a> {
    pub user_id: &'a str,
    pub realm: &'a str,
    pub display_name: &'a str,
    pub name: &'a str,
    pub client_ip: &'a str,
}

pub struct PendingPasskey {
    pub ceremony: String,
    pub public_key: Json,
    pub expires_at: DateTime<FixedOffset>,
}

impl std::fmt::Debug for PendingPasskey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingPasskey(<redacted>)")
    }
}

#[derive(Debug)]
pub enum PasskeyEnrollment {
    Enrolled {
        passkey: PasskeyRecord,
        recovery_codes: Option<RecoveryCodes>,
    },
    Rejected,
    Expired,
    Duplicate,
    LimitReached,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasskeyRemoval {
    Removed,
    LastFactorRequired,
    NotFound,
}

pub fn passkey_name(name: &str) -> Result<String, StoreError> {
    let name = name.trim();
    let length = name.chars().count();
    if length == 0 || length > NAME_MAX_CHARS {
        return Err(StoreError::InvalidPasskeyName(
            "must be 1 to 64 characters long",
        ));
    }
    if name.chars().any(char::is_control) {
        return Err(StoreError::InvalidPasskeyName(
            "must not contain control characters",
        ));
    }
    Ok(name.to_owned())
}

fn record_of(row: &QueryResult) -> Result<PasskeyRecord, TryGetError> {
    Ok(PasskeyRecord {
        id: row.try_get("", "id")?,
        name: row.try_get("", "name")?,
        rp_id: row.try_get("", "rp_id")?,
        backup_eligible: row.try_get("", "backup_eligible")?,
        backed_up: row.try_get("", "backed_up")?,
        created_at: row.try_get("", "created_at")?,
        last_used_at: row.try_get("", "last_used_at")?,
    })
}

pub(super) async fn rows(
    conn: &impl ConnectionTrait,
    user_id: &str,
    rp_id: Option<&str>,
    lock: bool,
) -> Result<Vec<Row>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM user_passkey \
         WHERE user_id = $1 AND ($2::text IS NULL OR rp_id = $2::text) \
         ORDER BY created_at, id{}",
        if lock { " FOR UPDATE" } else { "" }
    );
    let found = conn
        .query_all(statement(
            &sql,
            [user_id.into(), rp_id.map(str::to_owned).into()],
        ))
        .await?;
    found
        .iter()
        .map(|row| {
            Ok(Row {
                record: record_of(row)?,
                credential_id: row.try_get("", "credential_id")?,
                passkey: row.try_get("", "passkey")?,
            })
        })
        .collect::<Result<_, TryGetError>>()
        .map_err(Into::into)
}

async fn passkey_count(conn: &impl ConnectionTrait, user_id: &str) -> Result<i64, StoreError> {
    let row = conn
        .query_one(statement(
            "SELECT count(*) AS found FROM user_passkey WHERE user_id = $1",
            [user_id.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound("passkey count".to_owned()))?;
    Ok(row.try_get("", "found")?)
}

impl Security {
    pub async fn list_passkeys(&self, user_id: &str) -> Result<Vec<PasskeyRecord>, StoreError> {
        Ok(rows(self.db(), user_id, None, false)
            .await?
            .into_iter()
            .map(|row| row.record)
            .collect())
    }

    pub async fn begin_passkey(
        &self,
        rp: &RelyingParty,
        request: BeginPasskey<'_>,
    ) -> Result<PendingPasskey, StoreError> {
        let name = passkey_name(request.name)?;
        let user_id = request.user_id;
        let context = ensure_factor_row(self.db(), user_id).await?;
        let handle = Uuid::parse_str(&context)
            .map_err(|_| DbErr::Custom("the WebAuthn user handle is not a UUID".to_owned()))?;
        let txn = self.db().begin().await?;
        lock_user_row(&txn, user_id, true)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("second factor row".to_owned()))?;
        let held = rows(&txn, user_id, None, false).await?;
        if i64::try_from(held.len()).unwrap_or(i64::MAX) >= MAX_PASSKEYS {
            return Err(StoreError::TooManyPasskeys);
        }
        let credentials: Vec<Passkey> = held.iter().filter_map(Row::credential).collect();
        let challenge =
            passkeys::start_registration(rp, handle, user_id, request.display_name, &credentials)?;
        txn.execute(statement(
            "DELETE FROM second_factor_ceremony WHERE user_id = $1 AND kind = $2",
            [user_id.into(), CeremonyKind::PasskeyEnroll.as_str().into()],
        ))
        .await?;
        let ceremony = insert_ceremony(
            &txn,
            NewCeremony {
                kind: CeremonyKind::PasskeyEnroll,
                user_id,
                realm: request.realm,
                state: Some(json!({
                    "name": name,
                    "rp": rp.id(),
                    "registration": challenge.state,
                })),
                client_ip: request.client_ip,
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
        Ok(PendingPasskey {
            ceremony,
            public_key: challenge.public_key,
            expires_at,
        })
    }

    pub async fn finish_passkey(
        &self,
        rp: &RelyingParty,
        user_id: &str,
        ceremony: &str,
        credential: &Json,
    ) -> Result<PasskeyEnrollment, StoreError> {
        let Some(state) = self
            .take_bound_ceremony(ceremony, CeremonyKind::PasskeyEnroll, user_id, None)
            .await?
        else {
            return Ok(PasskeyEnrollment::Expired);
        };
        let name = state.get("name").and_then(Json::as_str);
        let pinned = state.get("rp").and_then(Json::as_str);
        let registration = state.get("registration");
        let (Some(name), Some(pinned), Some(registration)) = (name, pinned, registration) else {
            return Ok(PasskeyEnrollment::Rejected);
        };
        if pinned != rp.id() {
            return Ok(PasskeyEnrollment::Rejected);
        }
        let passkey = match passkeys::finish_registration(rp, registration, credential) {
            Ok(passkey) => passkey,
            Err(error) => {
                tracing::info!(user = %user_id, reason = error.kind(), "a passkey registration was refused");
                return Ok(PasskeyEnrollment::Rejected);
            }
        };
        let facts = passkeys::facts(&passkey)?;
        let stored = serde_json::to_value(&passkey).map_err(PasskeyError::from)?;
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
        if passkey_count(&txn, user_id).await? >= MAX_PASSKEYS {
            return Ok(PasskeyEnrollment::LimitReached);
        }
        let id = Uuid::new_v4();
        let inserted = txn
            .query_one(statement(
                "INSERT INTO user_passkey \
                     (id, user_id, name, credential_id, rp_id, passkey, backup_eligible, backed_up) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (credential_id) DO NOTHING RETURNING created_at",
                [
                    id.into(),
                    user_id.into(),
                    name.into(),
                    passkeys::credential_id(&passkey).into(),
                    rp.id().into(),
                    stored.into(),
                    facts.backup_eligible.into(),
                    facts.backed_up.into(),
                ],
            ))
            .await?;
        let Some(inserted) = inserted else {
            return Ok(PasskeyEnrollment::Duplicate);
        };
        let recovery_codes = match issued {
            Some((codes, hashes)) if !locked.totp && !locked.passkeys => {
                store_codes(&txn, user_id, &hashes).await?;
                Some(codes)
            }
            _ => None,
        };
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        self.follow(revision).await;
        Ok(PasskeyEnrollment::Enrolled {
            passkey: PasskeyRecord {
                id,
                name: name.to_owned(),
                rp_id: rp.id().to_owned(),
                backup_eligible: facts.backup_eligible,
                backed_up: facts.backed_up,
                created_at: inserted.try_get("", "created_at")?,
                last_used_at: None,
            },
            recovery_codes,
        })
    }

    pub async fn rename_passkey(
        &self,
        user_id: &str,
        id: Uuid,
        name: &str,
    ) -> Result<bool, StoreError> {
        let name = passkey_name(name)?;
        let renamed = self
            .db()
            .execute(statement(
                "UPDATE user_passkey SET name = $3 WHERE id = $1 AND user_id = $2",
                [id.into(), user_id.into(), name.into()],
            ))
            .await?;
        Ok(renamed.rows_affected() == 1)
    }

    pub async fn remove_passkey(
        &self,
        user_id: &str,
        id: Uuid,
    ) -> Result<PasskeyRemoval, StoreError> {
        let txn = self.db().begin().await?;
        let Some(locked) = lock_user_row(&txn, user_id, false).await? else {
            return Ok(PasskeyRemoval::NotFound);
        };
        let held = rows(&txn, user_id, None, true).await?;
        if !held.iter().any(|row| row.record.id == id) {
            return Ok(PasskeyRemoval::NotFound);
        }
        let last_factor = !locked.totp && held.len() == 1;
        if last_factor && self.effectively_required(locked.requirement, user_id) {
            return Ok(PasskeyRemoval::LastFactorRequired);
        }
        txn.execute(statement(
            "DELETE FROM user_passkey WHERE id = $1 AND user_id = $2",
            [id.into(), user_id.into()],
        ))
        .await?;
        if last_factor {
            store_codes(&txn, user_id, &[]).await?;
        }
        let revision = revision::bump(&txn, revision::SECURITY).await?;
        txn.commit().await?;
        self.follow(revision).await;
        Ok(PasskeyRemoval::Removed)
    }
}
