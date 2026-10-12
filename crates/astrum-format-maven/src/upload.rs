use astrum_content::format::UploadedFile;
use astrum_content::store::{self, PutAsset};
use astrum_content::{ContentError, Ctx, UploadField, UploadForm, UploadSpec, View};
use astrum_repo::RepositoryType;
use astrum_repo::config::VersionPolicy;

use crate::path::{self, Coordinates, Kind};
use crate::{DOCUMENT_LIMIT, ID, describe, hosted, pom, version, version_policy};

const COORDINATES: &str = "Component coordinates";

pub fn spec() -> UploadSpec {
    let coordinate = |name: &'static str, optional: bool| {
        UploadField::text(name, optional).in_group(COORDINATES)
    };
    UploadSpec {
        format: ID,
        multiple_upload: true,
        component_fields: vec![
            coordinate("groupId", false),
            coordinate("artifactId", false),
            coordinate("version", false),
            UploadField {
                kind: "BOOLEAN",
                ..coordinate("generate-pom", true)
            },
            coordinate("packaging", true),
        ],
        asset_fields: vec![
            UploadField::text("classifier", true),
            UploadField::text("extension", false),
            UploadField::file("asset"),
        ],
    }
}

struct Planned<'a> {
    index: String,
    file: Option<&'a UploadedFile>,
    classifier: Option<String>,
    extension: String,
    bytes: Option<Vec<u8>>,
}

fn bad(message: impl Into<String>) -> ContentError {
    ContentError::BadRequest(message.into())
}

fn field(form: &UploadForm, name: &str) -> Option<String> {
    form.field(&format!("{ID}.{name}"))
        .map(|value| value.trim().to_owned())
}

fn valid_part(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '/' | '\\' | ':'))
}

async fn read_file(file: &UploadedFile) -> Result<Vec<u8>, ContentError> {
    if file.size as usize > DOCUMENT_LIMIT {
        return Err(bad("the POM of the upload is too large"));
    }
    tokio::fs::read(&file.path)
        .await
        .map_err(ContentError::internal)
}

pub async fn upload(ctx: &Ctx, form: UploadForm) -> Result<(), ContentError> {
    match ctx.kind() {
        RepositoryType::Hosted => {}
        other => {
            return Err(bad(format!(
                "Uploading components to a '{}' type repository is unsupported, must be 'hosted'",
                other.as_str()
            )));
        }
    }
    let policy = version_policy(ctx);
    if policy == Some(VersionPolicy::Snapshot) {
        return Err(bad(
            "Upload to snapshot repositories not supported, use the maven client.",
        ));
    }
    let mut planned = Vec::new();
    let mut problems = Vec::new();
    for (name, file) in &form.files {
        let Some(index) = name.strip_prefix(&format!("{ID}.asset")) else {
            continue;
        };
        if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        match field(&form, &format!("asset{index}.extension")) {
            Some(extension) => planned.push(Planned {
                index: index.to_owned(),
                file: Some(file),
                classifier: field(&form, &format!("asset{index}.classifier")),
                extension,
                bytes: None,
            }),
            None => problems.push(format!(
                "Missing required asset field 'Extension' on '{index}'"
            )),
        }
    }
    if !problems.is_empty() {
        return Err(bad(problems.join("; ")));
    }
    if planned.is_empty() {
        return Err(bad("The upload holds no assets"));
    }
    let mut seen: Vec<&Planned> = Vec::new();
    for asset in &planned {
        if let Some(first) = seen.iter().find(|known| {
            known.classifier == asset.classifier && known.extension == asset.extension
        }) {
            return Err(bad(format!(
                "The assets {} and {} have identical coordinates",
                first.index, asset.index
            )));
        }
        seen.push(asset);
    }
    let pom_position = planned
        .iter()
        .position(|asset| asset.extension == "pom" && asset.classifier.is_none());
    let from_pom = match pom_position {
        Some(position) => {
            let file = planned[position]
                .file
                .expect("planned files come from the form");
            let bytes = read_file(file).await?;
            let parsed = pom::parse(&bytes)
                .map_err(|error| bad(format!("The POM of the upload cannot be read: {error}")))?;
            planned[position].bytes = Some(bytes);
            Some(parsed)
        }
        None => None,
    };
    let group_id = from_pom
        .as_ref()
        .and_then(|pom| pom.group_id.clone())
        .or_else(|| field(&form, "groupId"));
    let artifact_id = from_pom
        .as_ref()
        .and_then(|pom| pom.artifact_id.clone())
        .or_else(|| field(&form, "artifactId"));
    let version = from_pom
        .as_ref()
        .and_then(|pom| pom.version.clone())
        .or_else(|| field(&form, "version"));
    let (Some(group_id), Some(artifact_id), Some(version)) =
        (group_id.clone(), artifact_id.clone(), version.clone())
    else {
        let missing: Vec<String> = [
            (group_id, "Group ID"),
            (artifact_id, "Artifact ID"),
            (version, "Version"),
        ]
        .into_iter()
        .filter(|(value, _)| value.is_none())
        .map(|(_, label)| format!("Missing required component field '{label}'"))
        .collect();
        return Err(bad(missing.join("; ")));
    };
    let generate =
        field(&form, "generate-pom").is_some_and(|value| value.eq_ignore_ascii_case("true"));
    if pom_position.is_none() && generate {
        let packaging = field(&form, "packaging");
        planned.push(Planned {
            index: "pom".to_owned(),
            file: None,
            classifier: None,
            extension: "pom".to_owned(),
            bytes: Some(
                pom::generate(&group_id, &artifact_id, &version, packaging.as_deref()).into_bytes(),
            ),
        });
    }
    let snapshot = version::is_snapshot(&version);
    let mut paths = Vec::new();
    for asset in &planned {
        let coordinates = Coordinates {
            group_id: group_id.clone(),
            artifact_id: artifact_id.clone(),
            version: version.clone(),
            base_version: version::base_version(&version),
            classifier: asset.classifier.clone(),
            extension: asset.extension.clone(),
        };
        let parts = [&group_id, &artifact_id, &version, &asset.extension];
        let path = coordinates.path();
        let classifier_ok = asset.classifier.as_deref().is_none_or(valid_part);
        let parsed = path::parse(&path);
        let matches = matches!(&parsed.kind, Kind::Artifact(found) | Kind::Signature(found) if *found == coordinates);
        if !parts.iter().all(|part| valid_part(part))
            || group_id.split('.').any(str::is_empty)
            || !classifier_ok
            || parsed.hash.is_some()
            || !matches
        {
            return Err(bad(format!(
                "The coordinates do not make a valid Maven path: {}",
                path.trim_start_matches('/')
            )));
        }
        if snapshot && policy == Some(VersionPolicy::Release) {
            return Err(bad(format!(
                "Version policy mismatch, cannot upload SNAPSHOT content to RELEASE repositories for file '{}'",
                path.trim_start_matches('/')
            )));
        }
        let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
        ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
        paths.push(parsed);
    }
    for (asset, parsed) in planned.iter().zip(&paths) {
        let put = describe(parsed, PutAsset::hosted(ctx, parsed.path.clone()));
        let (put, body) = match (&asset.bytes, asset.file) {
            (Some(bytes), _) => {
                let put = match &parsed.kind {
                    Kind::Artifact(coordinates) if hosted::is_pom(coordinates) => {
                        hosted::with_pom(put, bytes)
                    }
                    _ => put,
                };
                (put, astrum_blob::content_of(bytes.clone()))
            }
            (None, Some(file)) => (put, UploadForm::stream(file).await?),
            (None, None) => continue,
        };
        store::put(ctx, put, body).await?;
    }
    Ok(())
}
