use astrum_db::audit::Event;
use astrum_security::two_factor::AUDIT_DOMAIN;
use serde_json::json;

use crate::{AppState, Caller};

pub(crate) fn current_id(state: &AppState) -> Option<String> {
    state
        .security()
        .model()
        .relying_party()
        .ok()
        .map(|relying_party| relying_party.id().to_owned())
}

pub(crate) async fn follow(state: &AppState, revision: i64) {
    for attempt in 1..=2 {
        match state.security().catch_up(revision).await {
            Ok(_) => return,
            Err(error) => tracing::error!(
                %error,
                attempt,
                "the security model could not be reloaded after a settings change; the revision is stored and the next sync retries it"
            ),
        }
    }
}

pub(crate) async fn report_change(state: &AppState, caller: &Caller, before: Option<String>) {
    let after = current_id(state);
    if after == before {
        return;
    }
    let stranded = match state.security().stranded_passkeys(after.as_deref()).await {
        Ok(stranded) => stranded,
        Err(error) => {
            tracing::error!(%error, "the passkeys stranded by a relying party change could not be counted");
            return;
        }
    };
    if stranded.passkeys == 0 {
        return;
    }
    tracing::warn!(
        passkeys = stranded.passkeys,
        users = stranded.users,
        "the relying party changed and these passkeys cannot be used until it is restored"
    );
    state
        .audit(
            caller,
            Event::new(AUDIT_DOMAIN, "passkey.stranded", "relyingParty")
                .with(json!({"passkeys": stranded.passkeys, "users": stranded.users})),
        )
        .await;
}
