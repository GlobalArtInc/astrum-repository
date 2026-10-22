use quick_xml::Reader;
use quick_xml::events::Event;

use crate::markup::{attribute, cdata, entity, escape_attribute, escape_text};
use crate::plugin::Plugin;

const MAX_DEPTH: usize = 64;
const MISCELLANEOUS: &str = "Miscellaneous";

pub struct Item<'a> {
    pub plugin: &'a Plugin,
    pub url: String,
    pub size: i64,
    pub downloads: i64,
    pub created_ms: i64,
    pub updated_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub version: String,
    pub category: String,
    pub xml: String,
}

fn since_and_until(plugin: &Plugin, minimum: bool) -> String {
    let mut element = String::from("<idea-version");
    if minimum {
        element.push_str(" min=\"n/a\" max=\"n/a\"");
    }
    if let Some(since) = &plugin.since_build {
        element.push_str(&format!(" since-build=\"{}\"", escape_attribute(since)));
    }
    if let Some(until) = &plugin.until_build {
        element.push_str(&format!(" until-build=\"{}\"", escape_attribute(until)));
    }
    element.push_str("/>");
    element
}

fn vendor_element(plugin: &Plugin) -> String {
    if plugin.vendor.name.is_empty()
        && plugin.vendor.email.is_empty()
        && plugin.vendor.url.is_empty()
    {
        return String::new();
    }
    format!(
        "<vendor email=\"{}\" url=\"{}\">{}</vendor>",
        escape_attribute(&plugin.vendor.email),
        escape_attribute(&plugin.vendor.url),
        escape_text(&plugin.vendor.name)
    )
}

fn depends_elements(plugin: &Plugin) -> String {
    plugin
        .depends
        .iter()
        .map(|depends| format!("<depends>{}</depends>", escape_text(depends)))
        .collect()
}

pub fn documented_entry(item: &Item<'_>) -> Entry {
    let plugin = item.plugin;
    let mut lines = vec![
        format!(
            "<plugin id=\"{}\" url=\"{}\" version=\"{}\">",
            escape_attribute(&plugin.id),
            escape_attribute(&item.url),
            escape_attribute(&plugin.version)
        ),
        format!("    {}", since_and_until(plugin, false)),
        format!("    <name>{}</name>", escape_text(&plugin.name)),
    ];
    let vendor = vendor_element(plugin);
    if !vendor.is_empty() {
        lines.push(format!("    {vendor}"));
    }
    lines.push(format!(
        "    <description>{}</description>",
        cdata(&plugin.description)
    ));
    lines.push(format!(
        "    <change-notes>{}</change-notes>",
        cdata(&plugin.change_notes)
    ));
    for depends in &plugin.depends {
        lines.push(format!("    <depends>{}</depends>", escape_text(depends)));
    }
    lines.push("  </plugin>".to_owned());
    Entry {
        id: plugin.id.clone(),
        version: plugin.version.clone(),
        category: String::new(),
        xml: lines.join("\n"),
    }
}

pub fn legacy_entry(item: &Item<'_>) -> Entry {
    let plugin = item.plugin;
    let category = match plugin.category.is_empty() {
        true => MISCELLANEOUS.to_owned(),
        false => plugin.category.clone(),
    };
    let xml = format!(
        "<idea-plugin downloads=\"{downloads}\" size=\"{size}\" date=\"{created}\" updatedDate=\"{updated}\" url=\"\"><name>{name}</name><id>{id}</id><description>{description}</description><version>{version}</version>{vendor}<change-notes>{notes}</change-notes>{range}{depends}<tags>{category}</tags></idea-plugin>",
        downloads = item.downloads.max(0),
        size = item.size.max(0),
        created = item.created_ms,
        updated = item.updated_ms,
        name = escape_text(&plugin.name),
        id = escape_text(&plugin.id),
        description = cdata(&plugin.description),
        version = escape_text(&plugin.version),
        vendor = vendor_element(plugin),
        notes = cdata(&plugin.change_notes),
        range = since_and_until(plugin, true),
        depends = depends_elements(plugin),
        category = escape_text(&category),
    );
    Entry {
        id: plugin.id.clone(),
        version: plugin.version.clone(),
        category,
        xml,
    }
}

pub fn documented_document(entries: &[Entry]) -> String {
    let mut document = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plugins>\n");
    for entry in entries {
        document.push_str("  ");
        document.push_str(entry.xml.trim());
        document.push('\n');
    }
    document.push_str("</plugins>\n");
    document
}

pub fn legacy_document(entries: &[Entry]) -> String {
    let mut document = String::from("<?xml version='1.0' encoding='UTF-8'?>");
    if entries.is_empty() {
        document.push_str("<plugin-repository/>");
        return document;
    }
    document.push_str("<plugin-repository>");
    let mut categories: Vec<&str> = Vec::new();
    for entry in entries {
        if !categories.contains(&entry.category.as_str()) {
            categories.push(&entry.category);
        }
    }
    for category in categories {
        let name = match category.is_empty() {
            true => MISCELLANEOUS,
            false => category,
        };
        document.push_str(&format!("<category name=\"{}\">", escape_attribute(name)));
        for entry in entries.iter().filter(|entry| entry.category == category) {
            document.push_str(&entry.xml);
        }
        document.push_str("</category>");
    }
    document.push_str("</plugin-repository>");
    document
}

pub fn merge(lists: Vec<Vec<Entry>>) -> Vec<Entry> {
    let mut merged: Vec<Entry> = Vec::new();
    for entry in lists.into_iter().flatten() {
        if !merged.iter().any(|known| known.id == entry.id) {
            merged.push(entry);
        }
    }
    merged
}

struct Open {
    start: usize,
    depth: usize,
    id: String,
    version: String,
    legacy: bool,
    field: Option<&'static str>,
    text: String,
}

pub fn parse_entries(bytes: &[u8]) -> Result<Vec<Entry>, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "the list is not UTF-8".to_owned())?
        .trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(text);
    let mut stack: Vec<String> = Vec::new();
    let mut category = String::new();
    let mut entries = Vec::new();
    let mut open: Option<Open> = None;
    let broken = |error: quick_xml::Error| format!("the list is not valid XML: {error}");
    loop {
        let before = reader.buffer_position() as usize;
        match reader.read_event().map_err(broken)? {
            Event::Start(start) => {
                if stack.len() >= MAX_DEPTH {
                    return Err("the list nests too deeply".to_owned());
                }
                let name = start.local_name().into_inner().to_owned();
                match open.as_mut() {
                    None => match name.as_str() {
                        "category" => category = attribute(&start, "name").unwrap_or_default(),
                        "idea-plugin" => {
                            open = Some(Open {
                                start: before,
                                depth: stack.len(),
                                id: String::new(),
                                version: String::new(),
                                legacy: true,
                                field: None,
                                text: String::new(),
                            })
                        }
                        "plugin" if stack.len() == 1 => {
                            open = Some(Open {
                                start: before,
                                depth: stack.len(),
                                id: attribute(&start, "id").unwrap_or_default(),
                                version: attribute(&start, "version").unwrap_or_default(),
                                legacy: false,
                                field: None,
                                text: String::new(),
                            })
                        }
                        _ => {}
                    },
                    Some(current) => {
                        if current.legacy && stack.len() == current.depth + 1 {
                            current.field = match name.as_str() {
                                "id" => Some("id"),
                                "version" => Some("version"),
                                _ => None,
                            };
                            current.text.clear();
                        }
                    }
                }
                stack.push(name);
            }
            Event::Empty(empty) => {
                let name = empty.local_name().into_inner().to_owned();
                if open.is_none() && name == "plugin" && stack.len() == 1 {
                    let id = attribute(&empty, "id").unwrap_or_default();
                    if !id.is_empty() {
                        entries.push(Entry {
                            id,
                            version: attribute(&empty, "version").unwrap_or_default(),
                            category: String::new(),
                            xml: text[before..reader.buffer_position() as usize].to_owned(),
                        });
                    }
                }
            }
            Event::Text(content) => {
                if let Some(current) = open.as_mut().filter(|current| current.field.is_some()) {
                    current.text.push_str(&content.xml10_content());
                }
            }
            Event::CData(content) => {
                if let Some(current) = open.as_mut().filter(|current| current.field.is_some()) {
                    current.text.push_str(&content.into_inner());
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(current) = open.as_mut().filter(|current| current.field.is_some()) {
                    let name = reference.into_inner().into_owned();
                    current.text.push_str(&entity(&name).unwrap_or_default());
                }
            }
            Event::End(_) => {
                let closed = stack.pop().unwrap_or_default();
                if let Some(current) = open.as_mut() {
                    if stack.len() == current.depth {
                        let end = reader.buffer_position() as usize;
                        if let Some(finished) = open.take()
                            && !finished.id.is_empty()
                        {
                            entries.push(Entry {
                                id: finished.id,
                                version: finished.version,
                                category: category.clone(),
                                xml: text[finished.start..end].to_owned(),
                            });
                        }
                    } else if let Some(field) = current.field.take() {
                        let value = std::mem::take(&mut current.text).trim().to_owned();
                        match field {
                            "id" => current.id = value,
                            _ => current.version = value,
                        }
                    }
                } else if closed == "category" {
                    category.clear();
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Vendor;

    fn plugin(id: &str, version: &str) -> Plugin {
        Plugin {
            id: id.to_owned(),
            name: format!("{id} plugin"),
            version: version.to_owned(),
            vendor: Vendor {
                name: "Acme & Co".to_owned(),
                email: "dev@acme.test".to_owned(),
                url: "https://acme.test/?a=1&b=2".to_owned(),
            },
            since_build: Some("241.0".to_owned()),
            until_build: Some("243.*".to_owned()),
            description: "<p>Does things</p>".to_owned(),
            change_notes: "Fixed ]]> markers".to_owned(),
            depends: vec!["com.intellij.modules.platform".to_owned()],
            category: "Tools".to_owned(),
        }
    }

    fn item<'a>(plugin: &'a Plugin, url: &str) -> Item<'a> {
        Item {
            plugin,
            url: url.to_owned(),
            size: 10,
            downloads: 3,
            created_ms: 1000,
            updated_ms: 2000,
        }
    }

    #[test]
    fn the_documented_file_has_the_attributes_the_ide_reads() {
        let plugin = plugin("org.acme.tool", "1.2.3");
        let entry = documented_entry(&item(
            &plugin,
            "https://repo.test/repository/jb/org.acme.tool/1.2.3/org.acme.tool-1.2.3.zip",
        ));
        let document = documented_document(&[entry]);
        assert!(document.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plugins>\n"));
        assert!(document.contains(
            "<plugin id=\"org.acme.tool\" url=\"https://repo.test/repository/jb/org.acme.tool/1.2.3/org.acme.tool-1.2.3.zip\" version=\"1.2.3\">"
        ));
        assert!(document.contains("<idea-version since-build=\"241.0\" until-build=\"243.*\"/>"));
        assert!(document.contains("<description><![CDATA[<p>Does things</p>]]></description>"));
        assert!(
            document
                .contains("<change-notes><![CDATA[Fixed ]]]]><![CDATA[> markers]]></change-notes>")
        );
        assert!(document.contains("<vendor email=\"dev@acme.test\" url=\"https://acme.test/?a=1&amp;b=2\">Acme &amp; Co</vendor>"));
        assert!(document.contains("<depends>com.intellij.modules.platform</depends>"));
        let parsed = parse_entries(document.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "org.acme.tool");
        assert_eq!(parsed[0].version, "1.2.3");
        assert!(parsed[0].xml.starts_with("<plugin id="));
        assert!(parsed[0].xml.ends_with("</plugin>"));
    }

    #[test]
    fn the_legacy_list_groups_plugins_by_category() {
        let first = plugin("org.acme.a", "1.0");
        let mut other = plugin("org.acme.b", "2.0");
        other.category = String::new();
        let third = plugin("org.acme.c", "3.0");
        let entries = vec![
            legacy_entry(&item(&first, "")),
            legacy_entry(&item(&other, "")),
            legacy_entry(&item(&third, "")),
        ];
        let document = legacy_document(&entries);
        assert!(document.starts_with("<?xml version='1.0' encoding='UTF-8'?><plugin-repository>"));
        assert!(document.contains("<category name=\"Tools\">"));
        assert!(document.contains("<category name=\"Miscellaneous\">"));
        assert!(document.contains(
            "<idea-plugin downloads=\"3\" size=\"10\" date=\"1000\" updatedDate=\"2000\" url=\"\">"
        ));
        assert!(document.contains(
            "<idea-version min=\"n/a\" max=\"n/a\" since-build=\"241.0\" until-build=\"243.*\"/>"
        ));
        let tools = document.find("<category name=\"Tools\">").unwrap();
        let third_at = document.find("org.acme.c</id>").unwrap();
        let misc = document.find("<category name=\"Miscellaneous\">").unwrap();
        assert!(tools < third_at && third_at < misc);
        let parsed = parse_entries(document.as_bytes()).unwrap();
        let ids: Vec<&str> = parsed.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(ids, ["org.acme.a", "org.acme.c", "org.acme.b"]);
        assert_eq!(parsed[0].category, "Tools");
        assert_eq!(parsed[2].category, "Miscellaneous");
        assert_eq!(parsed[0].version, "1.0");
        assert_eq!(
            legacy_document(&[]),
            "<?xml version='1.0' encoding='UTF-8'?><plugin-repository/>"
        );
    }

    #[test]
    fn marketplace_style_lists_are_split_into_entries() {
        let upstream = "<?xml version='1.0' encoding='UTF-8'?><plugin-repository><ff>\"Keymap\"</ff><category name=\"Keymap\"><idea-plugin downloads=\"1\" size=\"2\" date=\"3\" updatedDate=\"4\" url=\"https://plugins.example/plugin/164\"><name>IdeaVim</name><id>IdeaVIM</id><description><![CDATA[<p>Vim &amp; more</p>]]></description><version>2.19.0</version><vendor email=\"a@b\" url=\"\">JetBrains</vendor><idea-version min=\"n/a\" max=\"n/a\" since-build=\"242.21829\"/><depends>com.intellij.modules.platform</depends></idea-plugin></category><category name=\"Theme\"><idea-plugin downloads=\"1\" size=\"2\" date=\"3\" updatedDate=\"4\" url=\"\"><name>Dark</name><id>org.dark</id><version>9.7.0</version></idea-plugin></category></plugin-repository>";
        let entries = parse_entries(upstream.as_bytes()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            (entries[0].id.as_str(), entries[0].version.as_str()),
            ("IdeaVIM", "2.19.0")
        );
        assert_eq!(entries[0].category, "Keymap");
        assert_eq!(entries[1].category, "Theme");
        assert!(entries[0].xml.starts_with("<idea-plugin downloads=\"1\""));
        assert!(entries[0].xml.ends_with("</idea-plugin>"));
        let rebuilt = legacy_document(&entries);
        assert_eq!(parse_entries(rebuilt.as_bytes()).unwrap(), entries);
    }

    #[test]
    fn groups_keep_the_first_entry_of_an_id() {
        let entry = |id: &str, version: &str| Entry {
            id: id.to_owned(),
            version: version.to_owned(),
            category: String::new(),
            xml: format!("<x>{id}-{version}</x>"),
        };
        let merged = merge(vec![
            vec![entry("a", "1"), entry("b", "1")],
            vec![entry("a", "2"), entry("c", "2")],
        ]);
        let shown: Vec<String> = merged
            .iter()
            .map(|entry| format!("{}-{}", entry.id, entry.version))
            .collect();
        assert_eq!(shown, ["a-1", "b-1", "c-2"]);
    }

    #[test]
    fn malformed_lists_are_reported() {
        assert!(parse_entries(b"<plugins><plugin id=\"a\"></plugins>").is_err());
        assert!(parse_entries(&[0xff, 0xfe]).is_err());
        assert!(parse_entries(b"<plugins/>").unwrap().is_empty());
    }
}
