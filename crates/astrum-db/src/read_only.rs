use sea_orm::ConnectionTrait;
use serde::{Deserialize, Serialize};

use crate::{Error, setting};

pub const KEY: &str = "read_only";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadOnly {
    pub frozen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub system_initiated: bool,
}

pub async fn state(db: &impl ConnectionTrait) -> Result<ReadOnly, Error> {
    Ok(setting::get(db, KEY).await?.unwrap_or_default())
}

pub async fn freeze(
    db: &impl ConnectionTrait,
    reason: impl Into<String>,
    system_initiated: bool,
) -> Result<(), Error> {
    let state = ReadOnly {
        frozen: true,
        reason: Some(reason.into()),
        system_initiated,
    };
    setting::put(db, KEY, &state).await
}

pub async fn release(db: &impl ConnectionTrait) -> Result<(), Error> {
    setting::delete(db, KEY).await.map(|_| ())
}
