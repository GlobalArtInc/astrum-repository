use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait};
use serde_json::Value;

use crate::Error;
use crate::entity::audit_event;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub domain: String,
    pub kind: String,
    pub context: String,
    pub initiator: String,
    pub attributes: Value,
}

impl Event {
    pub fn new(
        domain: impl Into<String>,
        kind: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            kind: kind.into(),
            context: context.into(),
            initiator: String::new(),
            attributes: Value::Object(Default::default()),
        }
    }

    pub fn by(mut self, initiator: impl Into<String>) -> Self {
        self.initiator = initiator.into();
        self
    }

    pub fn with(mut self, attributes: Value) -> Self {
        self.attributes = attributes;
        self
    }
}

pub async fn record(db: &impl ConnectionTrait, event: Event) -> Result<(), Error> {
    tracing::info!(
        target: "audit",
        domain = %event.domain,
        kind = %event.kind,
        context = %event.context,
        initiator = %event.initiator,
        "audit"
    );
    let row = audit_event::ActiveModel {
        id: ActiveValue::NotSet,
        at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        domain: ActiveValue::Set(event.domain),
        kind: ActiveValue::Set(event.kind),
        context: ActiveValue::Set(event.context),
        initiator: ActiveValue::Set(event.initiator),
        attributes: ActiveValue::Set(event.attributes),
    };
    audit_event::Entity::insert(row).exec(db).await?;
    Ok(())
}
