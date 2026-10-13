use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;

use crate::messages::{Links, coordinates};
use crate::notice::Notice;

pub const EVENT_HEADER: &str = "X-Astrum-Event";
pub const DELIVERY_HEADER: &str = "X-Astrum-Delivery";
pub const SIGNATURE_HEADER: &str = "X-Astrum-Signature";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub notice: Notice,
    pub url: Option<String>,
    pub docker_pull: Option<String>,
}

impl Envelope {
    pub fn new(notice: Notice, links: &Links) -> Self {
        Self {
            url: links.component(&notice),
            docker_pull: links.pull_command(&notice),
            notice,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub event: String,
    pub delivery_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub text: String,
    pub envelope: Option<Envelope>,
}

fn non_empty(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        Value::String(value.to_owned())
    }
}

pub fn body(message: &Message) -> Vec<u8> {
    let timestamp = message
        .timestamp
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut body = json!({
        "event": message.event,
        "deliveryId": message.delivery_id,
        "timestamp": timestamp,
        "repository": null,
        "component": null,
        "initiator": null,
        "url": null,
        "dockerPull": null,
        "text": message.text,
    });
    if let Some(envelope) = &message.envelope {
        let notice = &envelope.notice;
        body["repository"] = json!({
            "name": notice.repository,
            "format": non_empty(&notice.format),
            "type": non_empty(&notice.repository_type),
        });
        body["component"] = json!({
            "id": notice.component_id.map(|id| astrum_content::ids::encode(&notice.repository, id)),
            "group": non_empty(&notice.namespace),
            "name": notice.name,
            "version": non_empty(&notice.version),
            "format": non_empty(&notice.format),
            "coordinates": coordinates(notice),
        });
        body["initiator"] = notice
            .initiator
            .as_deref()
            .map(non_empty)
            .unwrap_or(Value::Null);
        body["url"] = json!(envelope.url);
        body["dockerPull"] = json!(envelope.docker_pull);
    }
    serde_json::to_vec(&body).expect("a JSON value always serializes")
}

pub fn signature(secret: &str, body: &[u8]) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC takes keys of any length");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notice::Action;

    #[test]
    fn the_body_is_stable_and_signed() {
        let notice = Notice {
            repository: "maven-releases".into(),
            repository_type: "hosted".into(),
            format: "maven2".into(),
            namespace: "org.example".into(),
            name: "demo".into(),
            version: "1.0.0".into(),
            action: Action::Published,
            initiator: Some("admin".into()),
            component_id: Some(42),
        };
        let links = Links {
            base_url: Some("https://repo.example.com".into()),
        };
        let message = Message {
            event: "component.published".into(),
            delivery_id: "0b4f7c9e-3d61-4a8e-9c1e-6f2d1a7b5e40".into(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2026-10-11T08:15:30.123Z")
                .unwrap()
                .to_utc(),
            text: "hello".into(),
            envelope: Some(Envelope::new(notice, &links)),
        };
        let body = body(&message);
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["timestamp"], "2026-10-11T08:15:30.123Z");
        assert_eq!(value["repository"]["type"], "hosted");
        assert_eq!(value["component"]["coordinates"], "org.example:demo:1.0.0");
        assert_eq!(
            value["component"]["id"],
            astrum_content::ids::encode("maven-releases", 42)
        );
        assert_eq!(
            value["url"],
            "https://repo.example.com/search/component/maven2%3Aorg.example%3Ademo?version=1.0.0"
        );
        assert_eq!(value["dockerPull"], Value::Null);
        assert_eq!(
            signature("key", b"The quick brown fox jumps over the lazy dog"),
            "sha256=f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }
}
