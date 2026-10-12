use crate::hosted::Stored;
use crate::metadata::parse_nuspec;

pub fn stored(nuspec: &str) -> Stored {
    let metadata = parse_nuspec(nuspec.as_bytes()).unwrap();
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-12T19:32:38.698+00:00").unwrap();
    Stored {
        component: astrum_content::Component {
            id: 1,
            repository: "r".to_owned(),
            format: "nuget".to_owned(),
            namespace: String::new(),
            name: metadata.id.clone(),
            version: metadata.normalized_version.clone(),
            kind: "package".to_owned(),
            attributes: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        },
        metadata,
        package: None,
        symbols: None,
    }
}
