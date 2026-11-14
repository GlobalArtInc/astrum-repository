pub use astrum_content::transfer::{Bundle, Outcome};
use serde::{Deserialize, Serialize};

pub const PROTOCOL: u32 = 1;
pub const MAX_MISSING: usize = 1000;
pub const BLOBS_MISSING: &str = "blobs_missing";

pub const CAPABILITIES: &str = "/service/rest/v1/replication/capabilities";
pub const MISSING: &str = "/service/rest/v1/replication/{repo}/blobs/missing";
pub const BLOB: &str = "/service/rest/v1/replication/{repo}/blobs/{sha256}";
pub const COMPONENTS: &str = "/service/rest/v1/replication/{repo}/components";
pub const INVENTORY_COMPARE: &str = "/service/rest/v1/replication/{repo}/inventory/compare";
pub const INVENTORY: &str = "/service/rest/v1/replication/{repo}/inventory";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub protocol: u32,
    pub instance_id: String,
    pub version: String,
    pub formats: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingRequest {
    pub sha256: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingReply {
    pub missing: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub fingerprint: String,
    pub origin: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryKey {
    pub key: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCompare {
    pub items: Vec<InventoryKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryEntry {
    pub key: String,
    pub fingerprint: String,
    #[serde(default)]
    pub origin: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryPage {
    pub items: Vec<InventoryEntry>,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<Vec<String>>,
}

pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn roundtrip<T>(value: &T, wire: serde_json::Value)
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        assert_eq!(serde_json::to_value(value).unwrap(), wire);
        assert_eq!(&serde_json::from_value::<T>(wire).unwrap(), value);
    }

    #[test]
    fn every_message_has_one_camel_case_wire_form() {
        roundtrip(
            &Capabilities {
                protocol: PROTOCOL,
                instance_id: "0b8e6f0a-0000-4000-8000-000000000001".to_owned(),
                version: "3.0.4".to_owned(),
                formats: vec!["raw".to_owned()],
            },
            json!({"protocol": 1, "instanceId": "0b8e6f0a-0000-4000-8000-000000000001", "version": "3.0.4", "formats": ["raw"]}),
        );
        roundtrip(
            &MissingRequest {
                sha256: vec!["a".repeat(64)],
            },
            json!({"sha256": ["a".repeat(64)]}),
        );
        roundtrip(
            &MissingReply {
                missing: Vec::new(),
            },
            json!({"missing": []}),
        );
        roundtrip(
            &Tombstone {
                namespace: String::new(),
                name: "app".to_owned(),
                version: "1".to_owned(),
                fingerprint: "f".repeat(64),
                origin: "o".to_owned(),
            },
            json!({"namespace": "", "name": "app", "version": "1", "fingerprint": "f".repeat(64), "origin": "o"}),
        );
        roundtrip(
            &InventoryCompare {
                items: vec![InventoryKey {
                    key: "app/1".to_owned(),
                    fingerprint: "f".to_owned(),
                }],
            },
            json!({"items": [{"key": "app/1", "fingerprint": "f"}]}),
        );
        roundtrip(
            &InventoryPage {
                items: vec![InventoryEntry {
                    key: "app/1".to_owned(),
                    fingerprint: "f".to_owned(),
                    origin: None,
                }],
                next: Some("app/1".to_owned()),
            },
            json!({"items": [{"key": "app/1", "fingerprint": "f", "origin": null}], "next": "app/1"}),
        );
    }

    #[test]
    fn an_inventory_entry_may_omit_its_origin_and_a_page_its_cursor() {
        let page: InventoryPage =
            serde_json::from_value(json!({"items": [{"key": "k", "fingerprint": "f"}]})).unwrap();
        assert_eq!(page.items[0].origin, None);
        assert_eq!(page.next, None);
    }

    #[test]
    fn the_error_body_carries_the_missing_list_only_when_there_is_one() {
        let plain = ErrorBody {
            code: "forbidden".to_owned(),
            message: "no".to_owned(),
            missing: None,
        };
        assert_eq!(
            serde_json::to_value(&plain).unwrap(),
            json!({"code": "forbidden", "message": "no"})
        );
        let missing: ErrorBody = serde_json::from_value(
            json!({"code": BLOBS_MISSING, "message": "m", "missing": ["x"]}),
        )
        .unwrap();
        assert_eq!(missing.missing, Some(vec!["x".to_owned()]));
        let empty: ErrorBody = serde_json::from_str("{}").unwrap();
        assert_eq!(empty, ErrorBody::default());
    }

    #[test]
    fn a_digest_is_sixty_four_lowercase_hex_digits() {
        assert!(valid_sha256(&"0123456789abcdef".repeat(4)));
        assert!(!valid_sha256(&"A".repeat(64)));
        assert!(!valid_sha256(&"a".repeat(63)));
        assert!(!valid_sha256(&"g".repeat(64)));
    }
}
