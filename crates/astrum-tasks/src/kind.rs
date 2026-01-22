use std::collections::BTreeMap;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::run::Run;

pub type Properties = BTreeMap<String, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Number,
    Checkbox,
    Repository,
    BlobStore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub id: &'static str,
    pub kind: FieldType,
    pub label: &'static str,
    pub help: &'static str,
    pub required: bool,
    pub initial: Option<&'static str>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub formats: &'static [&'static str],
    pub types: &'static [&'static str],
    pub all: bool,
    pub groups: bool,
}

impl Field {
    pub const fn new(id: &'static str, kind: FieldType, label: &'static str) -> Self {
        Self {
            id,
            kind,
            label,
            help: "",
            required: false,
            initial: None,
            min: None,
            max: None,
            formats: &[],
            types: &[],
            all: false,
            groups: false,
        }
    }

    pub const fn help(mut self, help: &'static str) -> Self {
        self.help = help;
        self
    }

    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub const fn initial(mut self, value: &'static str) -> Self {
        self.initial = Some(value);
        self
    }

    pub const fn min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }

    pub const fn max(mut self, value: i64) -> Self {
        self.max = Some(value);
        self
    }

    pub const fn formats(mut self, formats: &'static [&'static str]) -> Self {
        self.formats = formats;
        self
    }

    pub const fn types(mut self, types: &'static [&'static str]) -> Self {
        self.types = types;
        self
    }

    pub const fn with_all(mut self) -> Self {
        self.all = true;
        self
    }

    pub const fn with_groups(mut self) -> Self {
        self.groups = true;
        self
    }

    pub fn template(&self) -> Value {
        let (kind, store) = match self.kind {
            FieldType::Text => ("string", None),
            FieldType::Number => ("number", None),
            FieldType::Checkbox => ("checkbox", None),
            FieldType::Repository => ("combobox", Some("repositories")),
            FieldType::BlobStore => ("combobox", Some("blobstores")),
        };
        let filters = match self.kind {
            FieldType::Repository if !self.formats.is_empty() || !self.types.is_empty() => {
                let mut filters = serde_json::Map::new();
                if !self.formats.is_empty() {
                    filters.insert("format".to_owned(), self.formats.join(",").into());
                }
                if !self.types.is_empty() {
                    filters.insert("type".to_owned(), self.types.join(",").into());
                }
                Value::Object(filters)
            }
            _ => Value::Null,
        };
        json!({
            "id": self.id,
            "type": kind,
            "label": self.label,
            "helpText": match self.help.is_empty() {
                true => Value::Null,
                false => self.help.into(),
            },
            "required": self.required,
            "disabled": false,
            "readOnly": false,
            "regexValidation": null,
            "initialValue": self.initial,
            "minValue": self.min.map(|value| value.to_string()),
            "maxValue": self.max.map(|value| value.to_string()),
            "storeApi": store,
            "storeFilters": filters,
            "idMapping": null,
            "nameMapping": null,
            "allowAutocomplete": store.map(|_| true),
            "attributes": {
                "allowAll": self.all,
                "groups": self.groups,
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("the task was stopped")]
    Cancelled,
    #[error("{0}")]
    Failed(String),
}

impl TaskError {
    pub fn failed(error: impl std::fmt::Display) -> Self {
        TaskError::Failed(error.to_string())
    }
}

impl From<sea_orm::DbErr> for TaskError {
    fn from(error: sea_orm::DbErr) -> Self {
        TaskError::failed(error)
    }
}

impl From<astrum_content::ContentError> for TaskError {
    fn from(error: astrum_content::ContentError) -> Self {
        TaskError::failed(error)
    }
}

impl From<astrum_blob::Error> for TaskError {
    fn from(error: astrum_blob::Error) -> Self {
        TaskError::failed(error)
    }
}

impl From<astrum_repo::Error> for TaskError {
    fn from(error: astrum_repo::Error) -> Self {
        TaskError::failed(error)
    }
}

impl From<anyhow::Error> for TaskError {
    fn from(error: anyhow::Error) -> Self {
        TaskError::Failed(format!("{error:#}"))
    }
}

#[async_trait]
pub trait TaskKind: Send + Sync + 'static {
    fn id(&self) -> &'static str;

    fn name(&self) -> &'static str;

    fn fields(&self) -> Vec<Field> {
        Vec::new()
    }

    fn singleton(&self) -> bool {
        false
    }

    fn describe(&self, properties: &Properties) -> String;

    async fn run(&self, run: &Run) -> Result<String, TaskError>;
}

#[async_trait]
pub trait Expiry: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    async fn expire(&self, content: &astrum_content::Content) -> Result<u64, TaskError>;
}
