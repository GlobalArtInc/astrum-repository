use crate::xml::{self, XmlError, line};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pom {
    pub group_id: Option<String>,
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    pub packaging: String,
    pub description: Option<String>,
}

pub fn parse(bytes: &[u8]) -> Result<Pom, XmlError> {
    let root = xml::parse(bytes)?;
    if root.name != "project" {
        return Err(XmlError::Syntax(format!(
            "the root element is {}, not project",
            root.name
        )));
    }
    let parent = root.child("parent");
    let inherited = |name: &str| {
        root.text_of(name)
            .or_else(|| parent.and_then(|parent| parent.text_of(name)))
    };
    Ok(Pom {
        group_id: inherited("groupId"),
        artifact_id: root.text_of("artifactId"),
        version: inherited("version"),
        packaging: root
            .text_of("packaging")
            .unwrap_or_else(|| "jar".to_owned()),
        description: root.text_of("description"),
    })
}

pub fn generate(
    group_id: &str,
    artifact_id: &str,
    version: &str,
    packaging: Option<&str>,
) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd\">\n",
    );
    line(&mut out, 1, "modelVersion", Some("4.0.0"));
    line(&mut out, 1, "groupId", Some(group_id));
    line(&mut out, 1, "artifactId", Some(artifact_id));
    line(&mut out, 1, "version", Some(version));
    line(&mut out, 1, "packaging", packaging);
    out.push_str("</project>\n");
    out
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Archetype {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub description: Option<String>,
}

pub fn parse_catalog(bytes: &[u8]) -> Result<Vec<Archetype>, XmlError> {
    let root = xml::parse(bytes)?;
    if root.name != "archetype-catalog" {
        return Err(XmlError::Syntax(format!(
            "the root element is {}, not archetype-catalog",
            root.name
        )));
    }
    Ok(root
        .child("archetypes")
        .map(|archetypes| {
            archetypes
                .children("archetype")
                .filter_map(|archetype| {
                    Some(Archetype {
                        group_id: archetype.text_of("groupId")?,
                        artifact_id: archetype.text_of("artifactId")?,
                        version: archetype.text_of("version")?,
                        description: archetype.text_of("description"),
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn write_catalog(archetypes: &[Archetype]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<archetype-catalog xmlns=\"http://maven.apache.org/plugins/maven-archetype-plugin/archetype-catalog/1.0.0\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:schemaLocation=\"http://maven.apache.org/plugins/maven-archetype-plugin/archetype-catalog/1.0.0 https://maven.apache.org/xsd/archetype-catalog-1.0.0.xsd\">\n",
    );
    if !archetypes.is_empty() {
        out.push_str("  <archetypes>\n");
        for archetype in archetypes {
            out.push_str("    <archetype>\n");
            line(&mut out, 3, "groupId", Some(&archetype.group_id));
            line(&mut out, 3, "artifactId", Some(&archetype.artifact_id));
            line(&mut out, 3, "version", Some(&archetype.version));
            line(&mut out, 3, "description", archetype.description.as_deref());
            out.push_str("    </archetype>\n");
        }
        out.push_str("  </archetypes>\n");
    }
    out.push_str("</archetype-catalog>\n");
    out
}

pub fn merge_catalogs(catalogs: impl IntoIterator<Item = Vec<Archetype>>) -> Vec<Archetype> {
    let mut merged: Vec<Archetype> = Vec::new();
    for archetype in catalogs.into_iter().flatten() {
        if !merged.iter().any(|known| {
            known.group_id == archetype.group_id
                && known.artifact_id == archetype.artifact_id
                && known.version == archetype.version
        }) {
            merged.push(archetype);
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poms_name_their_coordinates() {
        let pom = parse(
            br#"<project xmlns="http://maven.apache.org/POM/4.0.0">
  <parent><groupId>org.parent</groupId><artifactId>p</artifactId><version>7</version></parent>
  <artifactId>child</artifactId>
  <packaging>maven-archetype</packaging>
  <description>Makes things</description>
</project>"#,
        )
        .unwrap();
        assert_eq!(pom.group_id.as_deref(), Some("org.parent"));
        assert_eq!(pom.artifact_id.as_deref(), Some("child"));
        assert_eq!(pom.version.as_deref(), Some("7"));
        assert_eq!(pom.packaging, "maven-archetype");
        assert_eq!(parse(b"<project/>").unwrap().packaging, "jar");
        assert!(parse(b"<metadata/>").is_err());
    }

    #[test]
    fn generated_poms_read_back() {
        let generated = generate("com.acme", "tool", "2.1", Some("jar"));
        let pom = parse(generated.as_bytes()).unwrap();
        assert_eq!(
            (
                pom.group_id.as_deref(),
                pom.artifact_id.as_deref(),
                pom.version.as_deref()
            ),
            (Some("com.acme"), Some("tool"), Some("2.1"))
        );
        assert!(!generate("a", "b", "1", None).contains("packaging"));
    }

    #[test]
    fn catalogs_merge_and_round_trip() {
        let one = vec![Archetype {
            group_id: "a".to_owned(),
            artifact_id: "b".to_owned(),
            version: "1".to_owned(),
            description: Some("x & y".to_owned()),
        }];
        let written = write_catalog(&one);
        assert_eq!(parse_catalog(written.as_bytes()).unwrap(), one);
        assert_eq!(merge_catalogs([one.clone(), one.clone()]).len(), 1);
        assert!(
            parse_catalog(write_catalog(&[]).as_bytes())
                .unwrap()
                .is_empty()
        );
    }
}
