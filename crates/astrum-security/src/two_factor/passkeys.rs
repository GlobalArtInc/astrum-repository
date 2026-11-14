use serde::Deserialize;
use serde_json::Value;
use url::{Host, Url};
use uuid::Uuid;
use webauthn_rs::Webauthn;
pub use webauthn_rs::prelude::Passkey;
use webauthn_rs::prelude::{
    PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential,
    WebauthnBuilder, WebauthnError,
};

use super::RelyingPartyPolicy;

const RP_NAME: &str = "Astrum";
const LOCALHOST: &str = "localhost";
const MAX_LABEL: usize = 63;
const MAX_HOST: usize = 253;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RelyingPartyError {
    #[error("passkeys are unavailable: no base URL and no relying party id and origin are set")]
    Unavailable,
    #[error("the base URL is not usable for passkeys: {0}")]
    BaseUrl(&'static str),
    #[error("the relying party id is not valid: {0}")]
    Id(&'static str),
    #[error("the relying party id is not the host of the base URL or a parent domain of it")]
    IdNotSuffix,
    #[error("the origin {origin} is not valid for passkeys: {reason}")]
    Origin {
        origin: String,
        reason: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelyingParty {
    id: String,
    origins: Vec<Url>,
}

impl RelyingParty {
    pub fn resolve(
        policy_rp: &RelyingPartyPolicy,
        base_url: Option<&str>,
    ) -> Result<Self, RelyingPartyError> {
        let base = match base_url.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => Some(parse_base(value)?),
            None => None,
        };
        let base_host = base.as_ref().map(|(_, host)| host.as_str());
        let id = match policy_rp
            .id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(value) => {
                let id = normalise_id(value)?;
                if let Some(host) = base_host {
                    if !within(host, &id) {
                        return Err(RelyingPartyError::IdNotSuffix);
                    }
                    if host != id && !id.contains('.') {
                        return Err(RelyingPartyError::Id("a top level domain cannot be used"));
                    }
                } else if !id.contains('.') && id != LOCALHOST {
                    return Err(RelyingPartyError::Id("a top level domain cannot be used"));
                }
                id
            }
            None => match base_host {
                Some(host) => normalise_id(host)?,
                None => return Err(RelyingPartyError::Unavailable),
            },
        };
        let mut origins: Vec<Url> = Vec::new();
        let candidates = base
            .map(|(origin, _)| origin)
            .into_iter()
            .map(Ok)
            .chain(policy_rp.origins.iter().map(|value| parse_origin(value)));
        for candidate in candidates {
            let origin = candidate?;
            check_origin(&origin, &id)?;
            if !origins.contains(&origin) {
                origins.push(origin);
            }
        }
        if origins.is_empty() {
            return Err(RelyingPartyError::Unavailable);
        }
        Ok(Self { id, origins })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn origins(&self) -> &[Url] {
        &self.origins
    }

    fn webauthn(&self) -> Result<Webauthn, PasskeyError> {
        let (first, rest) = self
            .origins
            .split_first()
            .ok_or(PasskeyError::Configuration)?;
        let mut builder = WebauthnBuilder::new(&self.id, first)
            .map_err(|_| PasskeyError::Configuration)?
            .rp_name(RP_NAME);
        for origin in rest {
            builder = builder.append_allowed_origin(origin);
        }
        builder.build().map_err(|_| PasskeyError::Configuration)
    }
}

fn within(host: &str, id: &str) -> bool {
    host == id
        || host
            .strip_suffix(id)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

fn parse_base(value: &str) -> Result<(Url, String), RelyingPartyError> {
    let url = Url::parse(value).map_err(|_| RelyingPartyError::BaseUrl("it is not a URL"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(RelyingPartyError::BaseUrl("it is not an http or https URL"));
    }
    let host = match url.host() {
        Some(Host::Domain(domain)) => domain.to_owned(),
        Some(_) => return Err(RelyingPartyError::BaseUrl("its host is an IP address")),
        None => return Err(RelyingPartyError::BaseUrl("it has no host")),
    };
    let origin = Url::parse(&url.origin().ascii_serialization())
        .map_err(|_| RelyingPartyError::BaseUrl("it has no usable origin"))?;
    Ok((origin, host))
}

fn parse_origin(value: &str) -> Result<Url, RelyingPartyError> {
    let reject = |reason: &'static str| RelyingPartyError::Origin {
        origin: value.chars().take(120).collect(),
        reason,
    };
    let url = Url::parse(value.trim()).map_err(|_| reject("it is not a URL"))?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(reject("it carries credentials"));
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err(reject("it is not a bare origin"));
    }
    Ok(url)
}

fn check_origin(origin: &Url, id: &str) -> Result<(), RelyingPartyError> {
    let reject = |reason: &'static str| RelyingPartyError::Origin {
        origin: origin.as_str().chars().take(120).collect(),
        reason,
    };
    let Some(Host::Domain(host)) = origin.host() else {
        return Err(reject("its host is not a domain name"));
    };
    match origin.scheme() {
        "https" => {}
        "http" if host == LOCALHOST => {}
        _ => return Err(reject("it is neither https nor http://localhost")),
    }
    if !within(host, id) {
        return Err(reject(
            "its host is neither the relying party id nor a subdomain of it",
        ));
    }
    Ok(())
}

fn normalise_id(value: &str) -> Result<String, RelyingPartyError> {
    let id = value.to_ascii_lowercase();
    if id.is_empty() || id.len() > MAX_HOST {
        return Err(RelyingPartyError::Id("its length is not valid"));
    }
    let labels: Vec<&str> = id.split('.').collect();
    for label in &labels {
        let valid = !label.is_empty()
            && label.len() <= MAX_LABEL
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid {
            return Err(RelyingPartyError::Id(
                "it must be a plain ASCII domain name",
            ));
        }
    }
    if labels
        .last()
        .is_some_and(|label| label.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(RelyingPartyError::Id("an IP address cannot be used"));
    }
    Ok(id)
}

#[derive(Debug, thiserror::Error)]
pub enum PasskeyError {
    #[error("the relying party cannot be used to build a WebAuthn ceremony")]
    Configuration,
    #[error("there is no passkey to authenticate with")]
    NoCredentials,
    #[error("the stored ceremony state is not readable")]
    State,
    #[error("the client response is malformed")]
    Malformed,
    #[error("the response does not belong to a registered passkey")]
    UnknownCredential,
    #[error("the signature counter did not increase")]
    CounterRegression,
    #[error("the stored passkey is not readable")]
    StoredPasskey,
    #[error("the ceremony options cannot be encoded: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("the WebAuthn ceremony was refused: {0}")]
    Refused(WebauthnError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Challenge {
    pub public_key: Value,
    pub state: Value,
}

#[derive(Debug, Clone)]
pub struct Authenticated {
    pub credential: Passkey,
    pub changed: bool,
}

pub fn start_registration(
    rp: &RelyingParty,
    user_handle: Uuid,
    user_name: &str,
    display_name: &str,
    existing: &[Passkey],
) -> Result<Challenge, PasskeyError> {
    let exclude: Vec<_> = existing
        .iter()
        .map(|passkey| passkey.cred_id().clone())
        .collect();
    let (options, state) = rp
        .webauthn()?
        .start_passkey_registration(
            user_handle,
            user_name,
            display_name,
            (!exclude.is_empty()).then_some(exclude),
        )
        .map_err(PasskeyError::Refused)?;
    Ok(Challenge {
        public_key: serde_json::to_value(&options.public_key)?,
        state: serde_json::to_value(&state)?,
    })
}

pub fn finish_registration(
    rp: &RelyingParty,
    state: &Value,
    response: &Value,
) -> Result<Passkey, PasskeyError> {
    let state = PasskeyRegistration::deserialize(state).map_err(|_| PasskeyError::State)?;
    let response =
        RegisterPublicKeyCredential::deserialize(response).map_err(|_| PasskeyError::Malformed)?;
    rp.webauthn()?
        .finish_passkey_registration(&response, &state)
        .map_err(PasskeyError::Refused)
}

pub fn start_authentication(
    rp: &RelyingParty,
    credentials: &[Passkey],
) -> Result<Challenge, PasskeyError> {
    if credentials.is_empty() {
        return Err(PasskeyError::NoCredentials);
    }
    let (options, state) = rp
        .webauthn()?
        .start_passkey_authentication(credentials)
        .map_err(PasskeyError::Refused)?;
    Ok(Challenge {
        public_key: serde_json::to_value(&options.public_key)?,
        state: serde_json::to_value(&state)?,
    })
}

pub fn finish_authentication(
    rp: &RelyingParty,
    state: &Value,
    response: &Value,
    credentials: &[Passkey],
) -> Result<Authenticated, PasskeyError> {
    let state = PasskeyAuthentication::deserialize(state).map_err(|_| PasskeyError::State)?;
    let response =
        PublicKeyCredential::deserialize(response).map_err(|_| PasskeyError::Malformed)?;
    if cross_origin(&response) {
        return Err(PasskeyError::Refused(WebauthnError::CredentialCrossOrigin));
    }
    let result = rp
        .webauthn()?
        .finish_passkey_authentication(&response, &state)
        .map_err(|error| match error {
            WebauthnError::CredentialNotFound => PasskeyError::UnknownCredential,
            WebauthnError::CredentialPossibleCompromise => PasskeyError::CounterRegression,
            other => PasskeyError::Refused(other),
        })?;
    if !result.user_verified() {
        return Err(PasskeyError::Refused(WebauthnError::UserNotVerified));
    }
    let mut credential = credentials
        .iter()
        .find(|candidate| candidate.cred_id() == result.cred_id())
        .cloned()
        .ok_or(PasskeyError::UnknownCredential)?;
    let stored = stored_counter(&credential)?;
    let presented = result.counter();
    if (presented > 0 || stored > 0) && presented <= stored {
        return Err(PasskeyError::CounterRegression);
    }
    let changed = credential
        .update_credential(&result)
        .ok_or(PasskeyError::UnknownCredential)?;
    Ok(Authenticated {
        credential,
        changed,
    })
}

fn cross_origin(response: &PublicKeyCredential) -> bool {
    serde_json::from_slice::<Value>(response.response.client_data_json.as_ref())
        .ok()
        .and_then(|data| data.get("crossOrigin").and_then(Value::as_bool))
        .unwrap_or(false)
}

fn stored_counter(credential: &Passkey) -> Result<u32, PasskeyError> {
    serde_json::to_value(credential)
        .ok()
        .and_then(|value| value.pointer("/cred/counter").and_then(Value::as_u64))
        .and_then(|counter| u32::try_from(counter).ok())
        .ok_or(PasskeyError::StoredPasskey)
}
