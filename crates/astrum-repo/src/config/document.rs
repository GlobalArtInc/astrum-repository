use serde_json::{Map, Value, json};

use crate::config::{Attributes, Presence, Repository, Section};

impl Repository {
    pub fn url(&self, base_url: &str) -> String {
        format!(
            "{}/repository/{}",
            base_url.trim_end_matches('/'),
            self.name
        )
    }

    pub fn document(&self, base_url: &str) -> Value {
        let spec = self.spec();
        let attributes = self.attributes.redacted();
        let mut document = Map::new();
        document.insert("name".to_owned(), json!(self.name));
        document.insert("format".to_owned(), json!(self.format.id));
        document.insert("type".to_owned(), json!(self.kind.as_str()));
        document.insert("url".to_owned(), json!(self.url(base_url)));
        document.insert("online".to_owned(), json!(self.online));
        for section in Section::ALL {
            let Some(presence) = spec.presence(*section) else {
                continue;
            };
            match attributes.value(*section) {
                Some(value) => {
                    document.insert(section.key().to_owned(), value);
                }
                None if presence == Presence::Omitted => {}
                None => {
                    document.insert(section.key().to_owned(), Value::Null);
                }
            }
        }
        if spec.has_latest_policy()
            && let Some(Value::Object(storage)) = document.get_mut("storage")
        {
            storage.entry("latestPolicy").or_insert(Value::Null);
        }
        if spec.has_preemptive_authentication()
            && let Some(Value::Object(authentication)) = document
                .get_mut("httpClient")
                .and_then(|client| client.get_mut("authentication"))
        {
            authentication.entry("preemptive").or_insert(Value::Null);
        }
        Value::Object(document)
    }

    pub fn summary(&self, base_url: &str) -> Value {
        let mut attributes = Map::new();
        if let Some(remote_url) = self
            .attributes
            .proxy
            .as_ref()
            .and_then(|proxy| proxy.remote_url.as_ref())
        {
            attributes.insert("proxy".to_owned(), json!({ "remoteUrl": remote_url }));
        }
        json!({
            "name": self.name,
            "format": self.format.id,
            "type": self.kind.as_str(),
            "url": self.url(base_url),
            "online": self.online,
            "attributes": attributes,
        })
    }

    pub fn stored_attributes(&self) -> Value {
        let mut attributes = self.attributes.clone();
        if let Some(storage) = &mut attributes.storage {
            storage.blob_store_name = None;
        }
        serde_json::to_value(attributes).unwrap_or_else(|_| json!({}))
    }

    pub fn from_stored(
        name: String,
        format: &str,
        kind: &str,
        online: bool,
        blob_store: Option<String>,
        attributes: Value,
    ) -> Result<Self, String> {
        let format = crate::format(format).ok_or_else(|| format!("unknown format \"{format}\""))?;
        let kind = crate::RepositoryType::parse(kind)
            .ok_or_else(|| format!("unknown repository type \"{kind}\""))?;
        let mut attributes: Attributes =
            serde_json::from_value(attributes).map_err(|error| error.to_string())?;
        attributes
            .storage
            .get_or_insert_with(Default::default)
            .blob_store_name = blob_store;
        Ok(Self {
            name,
            format,
            kind,
            online,
            attributes,
        })
    }
}
