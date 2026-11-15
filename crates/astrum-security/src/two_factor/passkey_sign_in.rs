use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration;
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde_json::{Value as Json, json};
use uuid::Uuid;

use super::passkey_store::{Row, rows};
use super::passkeys::{self, Passkey, PasskeyError, RelyingParty};
use super::store::{
    CEREMONY_ATTEMPTS, Ceremony, CeremonyKind, HANDLE_MAX_LENGTH, NewCeremony, StoreError,
    ceremony_from_row, handle_hash, insert_ceremony, statement,
};
use crate::Security;

pub struct PendingSignIn {
    pub ceremony: String,
    pub public_key: Json,
}

impl std::fmt::Debug for PendingSignIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingSignIn(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignInOutcome {
    Accepted { passkey: Uuid },
    CounterRegression { passkey: Option<Uuid> },
    Rejected,
}

fn credential_of(response: &Json) -> Option<Vec<u8>> {
    let raw = response.get("rawId")?.as_str()?;
    URL_SAFE_NO_PAD.decode(raw).ok()
}

impl Security {
    pub async fn peek_ceremony(
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
                "SELECT kind, user_id, realm, state, attempts, client_ip, expires_at \
                 FROM second_factor_ceremony \
                 WHERE handle_hash = $1 AND kind = $2 AND expires_at > now() AND attempts < $3",
                [
                    handle_hash(handle).into(),
                    kind.as_str().into(),
                    CEREMONY_ATTEMPTS.into(),
                ],
            ))
            .await?;
        Ok(row
            .map(|row| ceremony_from_row(&row))
            .transpose()?
            .flatten())
    }

    pub async fn begin_passkey_sign_in(
        &self,
        rp: &RelyingParty,
        user_id: &str,
        realm: &str,
        session: &str,
        client_ip: &str,
        lifetime: Duration,
    ) -> Result<Option<PendingSignIn>, StoreError> {
        let credentials: Vec<Passkey> = rows(self.db(), user_id, Some(rp.id()), false)
            .await?
            .iter()
            .filter_map(Row::credential)
            .collect();
        if credentials.is_empty() {
            return Ok(None);
        }
        let challenge = passkeys::start_authentication(rp, &credentials)?;
        let session_hash = handle_hash(session);
        let txn = self.db().begin().await?;
        txn.execute(statement(
            "DELETE FROM second_factor_ceremony \
             WHERE user_id = $1 AND kind = $2 AND state ->> 'session' = $3",
            [
                user_id.into(),
                CeremonyKind::SignIn.as_str().into(),
                session_hash.clone().into(),
            ],
        ))
        .await?;
        let ceremony = insert_ceremony(
            &txn,
            NewCeremony {
                kind: CeremonyKind::SignIn,
                user_id,
                realm,
                state: Some(json!({"session": session_hash, "passkey": challenge.state})),
                client_ip,
                lifetime,
            },
        )
        .await?;
        txn.commit().await?;
        Ok(Some(PendingSignIn {
            ceremony,
            public_key: challenge.public_key,
        }))
    }

    pub async fn finish_passkey_sign_in(
        &self,
        rp: &RelyingParty,
        user_id: &str,
        ceremony: &str,
        session: &str,
        credential: &Json,
    ) -> Result<SignInOutcome, StoreError> {
        let session_hash = handle_hash(session);
        let Some(state) = self
            .take_bound_ceremony(ceremony, CeremonyKind::SignIn, user_id, Some(&session_hash))
            .await?
        else {
            return Ok(SignInOutcome::Rejected);
        };
        let Some(challenge) = state.get("passkey") else {
            return Ok(SignInOutcome::Rejected);
        };
        let txn = self.db().begin().await?;
        let held = rows(&txn, user_id, Some(rp.id()), true).await?;
        let credentials: Vec<Passkey> = held.iter().filter_map(Row::credential).collect();
        match passkeys::finish_authentication(rp, challenge, credential, &credentials) {
            Ok(authenticated) => {
                let wanted = passkeys::credential_id(&authenticated.credential);
                let Some(row) = held.iter().find(|row| row.credential_id == wanted) else {
                    return Ok(SignInOutcome::Rejected);
                };
                let facts = passkeys::facts(&authenticated.credential)?;
                let stored =
                    serde_json::to_value(&authenticated.credential).map_err(PasskeyError::from)?;
                txn.execute(statement(
                    "UPDATE user_passkey SET passkey = $2, backup_eligible = $3, backed_up = $4, \
                         last_used_at = now() WHERE id = $1",
                    [
                        row.record.id.into(),
                        stored.into(),
                        facts.backup_eligible.into(),
                        facts.backed_up.into(),
                    ],
                ))
                .await?;
                txn.commit().await?;
                Ok(SignInOutcome::Accepted {
                    passkey: row.record.id,
                })
            }
            Err(PasskeyError::CounterRegression) => {
                let presented = credential_of(credential);
                let passkey = presented.and_then(|wanted| {
                    held.iter()
                        .find(|row| row.credential_id == wanted)
                        .map(|row| row.record.id)
                });
                Ok(SignInOutcome::CounterRegression { passkey })
            }
            Err(error) => {
                tracing::warn!(user = %user_id, reason = error.kind(), detail = %error.detail(), "a passkey sign-in was refused");
                Ok(SignInOutcome::Rejected)
            }
        }
    }

    pub(super) async fn take_bound_ceremony(
        &self,
        handle: &str,
        kind: CeremonyKind,
        user_id: &str,
        session_hash: Option<&str>,
    ) -> Result<Option<Json>, StoreError> {
        if handle.is_empty() || handle.len() > HANDLE_MAX_LENGTH {
            return Ok(None);
        }
        let row = self
            .db()
            .query_one(statement(
                "DELETE FROM second_factor_ceremony \
                 WHERE handle_hash = $1 AND kind = $2 AND user_id = $3 AND expires_at > now() \
                   AND ($4::text IS NULL OR state ->> 'session' = $4::text) \
                 RETURNING state",
                [
                    handle_hash(handle).into(),
                    kind.as_str().into(),
                    user_id.into(),
                    session_hash.map(str::to_owned).into(),
                ],
            ))
            .await?;
        Ok(row
            .map(|row| row.try_get::<Option<Json>>("", "state"))
            .transpose()?
            .flatten())
    }
}
