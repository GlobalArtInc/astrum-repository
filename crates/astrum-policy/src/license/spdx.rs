use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub id: String,
    pub deprecated: bool,
    #[serde(default)]
    pub equivalent: String,
}

struct Table {
    entries: Vec<Entry>,
    index: HashMap<String, usize>,
}

static TABLE: OnceLock<Table> = OnceLock::new();

fn table() -> &'static Table {
    TABLE.get_or_init(|| {
        let entries: Vec<Entry> =
            serde_json::from_str(include_str!("spdx.json")).unwrap_or_default();
        let index = entries
            .iter()
            .enumerate()
            .map(|(position, entry)| (entry.id.to_ascii_lowercase(), position))
            .collect();
        Table { entries, index }
    })
}

pub fn lookup(id: &str) -> Option<&'static Entry> {
    let table = table();
    table
        .index
        .get(&id.to_ascii_lowercase())
        .and_then(|position| table.entries.get(*position))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::{State, normalise};

    #[test]
    fn every_listed_id_normalises_to_spdx_under_its_own_spelling() {
        let entries = &table().entries;
        assert!(!entries.is_empty());
        for entry in entries {
            let normalised = normalise(&entry.id);
            assert_eq!(normalised.state, State::Spdx, "{}", entry.id);
            assert_eq!(normalised.text, entry.id);
        }
    }

    #[test]
    fn every_equivalent_is_a_listed_id_that_is_not_deprecated() {
        for entry in table()
            .entries
            .iter()
            .filter(|entry| entry.deprecated && !entry.equivalent.is_empty())
        {
            let modern = lookup(&entry.equivalent).expect("the equivalent is listed");
            assert!(
                !modern.deprecated,
                "{} points at {}",
                entry.id, entry.equivalent
            );
        }
    }
}
