use std::collections::{BTreeMap, HashMap};

use quick_xml::events::{BytesStart, Event};

pub const HIDDEN_PROPERTIES: &[&str] = &["p2.mirrorsURL", "p2.mirrorsBaseURL", "p2.statsURI"];

const MAX_PROPERTIES: usize = 64;

fn invalid(error: impl std::fmt::Display) -> String {
    format!("the repository file is not valid XML: {error}")
}

fn attribute(element: &BytesStart<'_>, wanted: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attribute| attribute.key.as_ref() == wanted)
        .and_then(|attribute| {
            attribute
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.into_owned())
        })
}

fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            other => out.push(other),
        }
    }
    out
}

fn tag(element: &BytesStart<'_>, empty: bool, replace: (&str, &str)) -> String {
    let mut out = format!("<{}", element.name().as_ref());
    let mut replaced = false;
    for found in element.attributes().flatten() {
        let key = found.key.as_ref();
        let value = match key == replace.0 {
            true => {
                replaced = true;
                replace.1.to_owned()
            }
            false => found
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map(|value| value.into_owned())
                .unwrap_or_default(),
        };
        out.push_str(&format!(" {key}='{}'", escape(&value)));
    }
    if !replaced {
        out.push_str(&format!(" {}='{}'", replace.0, escape(replace.1)));
    }
    out.push_str(if empty { "/>" } else { ">" });
    out
}

fn whole_line(xml: &str, start: usize, end: usize) -> (usize, usize) {
    let line = xml[..start].rfind('\n').map(|at| at + 1).unwrap_or(0);
    if !xml[line..start].chars().all(char::is_whitespace) {
        return (start, end);
    }
    let rest = &xml[end..];
    let trailing = rest.len() - rest.trim_start_matches([' ', '\t']).len();
    let after = &rest[trailing..];
    if let Some(stripped) = after.strip_prefix("\r\n") {
        return (line, xml.len() - stripped.len());
    }
    if let Some(stripped) = after.strip_prefix('\n') {
        return (line, xml.len() - stripped.len());
    }
    (start, end)
}

struct Edit {
    start: usize,
    end: usize,
    text: String,
}

fn apply(xml: &str, mut edits: Vec<Edit>) -> Option<String> {
    if edits.is_empty() {
        return None;
    }
    edits.sort_by_key(|edit| edit.start);
    let mut out = String::with_capacity(xml.len());
    let mut at = 0;
    for edit in edits {
        if edit.start < at {
            continue;
        }
        out.push_str(&xml[at..edit.start]);
        out.push_str(&edit.text);
        at = edit.end;
    }
    out.push_str(&xml[at..]);
    Some(out)
}

/// Drops the repository properties that send clients elsewhere (mirrors, statistics) and lets
/// `child` rewrite the location of each child of a composite repository. Answers `None` when
/// nothing changed, so the original bytes can be kept.
pub fn rewrite(
    xml: &str,
    mut child: impl FnMut(&str) -> Option<String>,
) -> Result<Option<String>, String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut edits = Vec::new();
    let mut properties: Option<(usize, usize, BytesStart<'static>)> = None;
    let mut kept = 0usize;
    let mut removed = 0usize;
    let mut hidden_start: Option<usize> = None;
    loop {
        let before = reader.buffer_position() as usize;
        let event = reader.read_event().map_err(invalid)?;
        let after = reader.buffer_position() as usize;
        let parents: Vec<&str> = stack.iter().map(String::as_str).collect();
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let empty = matches!(event, Event::Empty(_));
                let name = element.name().as_ref().to_owned();
                match (parents.as_slice(), name.as_str()) {
                    (["repository"], "properties") if !empty => {
                        properties = Some((before, after, element.to_owned()));
                        kept = 0;
                        removed = 0;
                    }
                    (["repository", "properties"], "property") => {
                        let hidden = attribute(element, "name")
                            .is_some_and(|name| HIDDEN_PROPERTIES.contains(&name.as_str()));
                        match (hidden, empty) {
                            (true, true) => {
                                let (start, end) = whole_line(xml, before, after);
                                edits.push(Edit {
                                    start,
                                    end,
                                    text: String::new(),
                                });
                                removed += 1;
                            }
                            (true, false) => hidden_start = Some(before),
                            (false, _) => kept += 1,
                        }
                    }
                    (["repository", "children"], "child") => {
                        if let Some(location) = attribute(element, "location")
                            && let Some(written) = child(&location)
                            && written != location
                        {
                            edits.push(Edit {
                                start: before,
                                end: after,
                                text: tag(element, empty, ("location", &written)),
                            });
                        }
                    }
                    _ => {}
                }
                if !empty {
                    stack.push(name);
                }
            }
            Event::End(ref element) => {
                let name = element.name().as_ref().to_owned();
                if name == "property"
                    && parents.as_slice() == ["repository", "properties", "property"]
                    && let Some(start) = hidden_start.take()
                {
                    let (start, end) = whole_line(xml, start, after);
                    edits.push(Edit {
                        start,
                        end,
                        text: String::new(),
                    });
                    removed += 1;
                }
                if name == "properties"
                    && parents.as_slice() == ["repository", "properties"]
                    && let Some((start, end, element)) = properties.take()
                    && removed > 0
                {
                    edits.push(Edit {
                        start,
                        end,
                        text: tag(&element, false, ("size", &kept.to_string())),
                    });
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(apply(xml, edits))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Descriptor {
    pub classifier: String,
    pub id: String,
    pub version: String,
    pub properties: BTreeMap<String, String>,
}

impl Descriptor {
    pub fn key(&self) -> String {
        format!("{}/{}_{}", self.classifier, self.id, self.version)
    }
}

/// The artifact descriptors of a simple artifact repository, by classifier and file stem.
#[derive(Debug, Default)]
pub struct Descriptors {
    by_key: HashMap<String, Descriptor>,
}

impl Descriptors {
    pub fn parse(xml: &str) -> Result<Self, String> {
        let mut reader = quick_xml::Reader::from_str(xml);
        let mut stack: Vec<String> = Vec::new();
        let mut found = Descriptors::default();
        let mut current: Option<Descriptor> = None;
        loop {
            let event = reader.read_event().map_err(invalid)?;
            match event {
                Event::Start(ref element) | Event::Empty(ref element) => {
                    let empty = matches!(event, Event::Empty(_));
                    let name = element.name().as_ref().to_owned();
                    let parents: Vec<&str> = stack.iter().map(String::as_str).collect();
                    match (parents.as_slice(), name.as_str()) {
                        (["repository", "artifacts"], "artifact") => {
                            let descriptor = Descriptor {
                                classifier: attribute(element, "classifier").unwrap_or_default(),
                                id: attribute(element, "id").unwrap_or_default(),
                                version: attribute(element, "version").unwrap_or_default(),
                                properties: BTreeMap::new(),
                            };
                            match empty {
                                true => found.add(descriptor),
                                false => current = Some(descriptor),
                            }
                        }
                        (["repository", "artifacts", "artifact", "properties"], "property") => {
                            if let Some(descriptor) = current.as_mut()
                                && descriptor.properties.len() < MAX_PROPERTIES
                                && let (Some(key), Some(value)) =
                                    (attribute(element, "name"), attribute(element, "value"))
                            {
                                descriptor.properties.insert(key, value);
                            }
                        }
                        _ => {}
                    }
                    if !empty {
                        stack.push(name);
                    }
                }
                Event::End(ref element) => {
                    if element.name().as_ref() == "artifact"
                        && stack.len() == 3
                        && let Some(descriptor) = current.take()
                    {
                        found.add(descriptor);
                    }
                    stack.pop();
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(found)
    }

    fn add(&mut self, descriptor: Descriptor) {
        if descriptor.id.is_empty() || descriptor.version.is_empty() {
            return;
        }
        let packed = descriptor
            .properties
            .get("format")
            .is_some_and(|format| format == "packed");
        let key = descriptor.key();
        match self.by_key.get(&key) {
            Some(_) if packed => {}
            _ => {
                self.by_key.insert(key, descriptor);
            }
        }
    }

    pub fn find(&self, classifier: &str, stem: &str) -> Option<&Descriptor> {
        self.by_key.get(&format!("{classifier}/{stem}"))
    }

    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

/// The `label` of the root `feature` element of a `feature.xml`.
pub fn feature_label(xml: &str) -> Option<String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event().ok()? {
            Event::Start(ref element) | Event::Empty(ref element) => {
                return match element.name().as_ref() {
                    "feature" => attribute(element, "label"),
                    _ => None,
                };
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACTS: &str = "<?xml version='1.0' encoding='UTF-8'?>
<?artifactRepository version='1.1.0'?>
<repository name='Example' type='org.eclipse.equinox.p2.artifact.repository.simpleRepository' version='1'>
  <properties size='5'>
    <property name='p2.timestamp' value='1700000000000'/>
    <property name='p2.compressed' value='true'/>
    <property name='p2.mirrorsURL' value='https://www.example.org/mirrors?file=/x&amp;format=xml'/>
    <property name='p2.statsURI' value='https://stats.example.org/x'/>
    <property name='pgp.publicKeys' value='line one&#10;line two'/>
  </properties>
  <mappings size='1'>
    <rule filter='(&amp; (classifier=osgi.bundle))' output='${repoUrl}/plugins/${id}_${version}.jar'/>
  </mappings>
  <artifacts size='3'>
    <artifact classifier='osgi.bundle' id='org.example.core' version='1.2.0.v2024'>
      <properties size='2'>
        <property name='artifact.size' value='1234'/>
        <property name='download.checksum.sha-256' value='abc'/>
      </properties>
    </artifact>
    <artifact classifier='osgi.bundle' id='org.example.core' version='1.2.0.v2024'>
      <properties size='1'>
        <property name='format' value='packed'/>
      </properties>
    </artifact>
    <artifact classifier='org.eclipse.update.feature' id='org.example.feature' version='1.2.0'/>
  </artifacts>
</repository>
";

    #[test]
    fn mirrors_and_statistics_are_dropped_and_the_rest_is_kept() {
        let rewritten = rewrite(ARTIFACTS, |_| None).unwrap().unwrap();
        assert!(!rewritten.contains("p2.mirrorsURL"));
        assert!(!rewritten.contains("p2.statsURI"));
        assert!(rewritten.contains("<properties size='3'>"));
        assert!(
            rewritten.contains("<property name='pgp.publicKeys' value='line one&#10;line two'/>")
        );
        let expected = ARTIFACTS
            .replace("<properties size='5'>", "<properties size='3'>")
            .replace(
                "    <property name='p2.mirrorsURL' value='https://www.example.org/mirrors?file=/x&amp;format=xml'/>\n",
                "",
            )
            .replace("    <property name='p2.statsURI' value='https://stats.example.org/x'/>\n", "");
        assert_eq!(rewritten, expected);
        let again = rewrite(&rewritten, |_| None).unwrap();
        assert_eq!(again, None);
    }

    #[test]
    fn composite_children_get_new_locations() {
        let composite = "<?xml version='1.0' encoding='UTF-8'?>
<?compositeArtifactRepository version='1.0.0'?>
<repository name='All' type='org.eclipse.equinox.internal.p2.artifact.repository.CompositeArtifactRepository' version='1.0.0'>
  <properties size='1'>
    <property name='p2.timestamp' value='1'/>
  </properties>
  <children size='3'>
    <child location='first'/>
    <child location='https://other.example.org/p2/x'/>
    <child location='../up' ></child>
  </children>
</repository>
";
        let mut seen = Vec::new();
        let rewritten = rewrite(composite, |location| {
            seen.push(location.to_owned());
            match location {
                "first" => Some("first".to_owned()),
                "https://other.example.org/p2/x" => {
                    Some(".remote/https/other.example.org/p2/x".to_owned())
                }
                _ => Some(".remote/https/h/up".to_owned()),
            }
        })
        .unwrap()
        .unwrap();
        assert_eq!(seen, ["first", "https://other.example.org/p2/x", "../up"]);
        assert!(rewritten.contains("<child location='first'/>"));
        assert!(rewritten.contains("<child location='.remote/https/other.example.org/p2/x'/>"));
        assert!(rewritten.contains("<child location='.remote/https/h/up'></child>"));
        assert!(rewritten.contains("<properties size='1'>"));
    }

    #[test]
    fn broken_files_are_refused() {
        assert!(rewrite("<repository><properties></repository>", |_| None).is_err());
    }

    #[test]
    fn descriptors_are_found_by_classifier_and_stem() {
        let descriptors = Descriptors::parse(ARTIFACTS).unwrap();
        assert_eq!(descriptors.len(), 2);
        let core = descriptors
            .find("osgi.bundle", "org.example.core_1.2.0.v2024")
            .unwrap();
        assert_eq!(core.id, "org.example.core");
        assert_eq!(core.properties["artifact.size"], "1234");
        assert!(!core.properties.contains_key("format"));
        let feature = descriptors
            .find("org.eclipse.update.feature", "org.example.feature_1.2.0")
            .unwrap();
        assert_eq!(feature.version, "1.2.0");
        assert!(
            descriptors
                .find("osgi.bundle", "org.example.feature_1.2.0")
                .is_none()
        );
    }

    #[test]
    fn feature_labels_are_read() {
        assert_eq!(
            feature_label(
                "<?xml version='1.0'?>\n<!-- x -->\n<feature id='a' label='%featureName' version='1'>\n</feature>"
            ),
            Some("%featureName".to_owned())
        );
        assert_eq!(feature_label("<plugin id='a'/>"), None);
    }
}
