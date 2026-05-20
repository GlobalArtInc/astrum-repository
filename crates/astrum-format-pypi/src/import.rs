use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::Value;

use crate::archive;
use crate::hosted::{METADATA_SUFFIX, Package};
use crate::index::File;
use crate::metadata::CoreMetadata;
use crate::name;
use crate::remote;

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    let page = path == "/simple/" || path.starts_with("/simple/");
    let companion = path.ends_with(METADATA_SUFFIX);
    match (repository, page) {
        (RepositoryType::Hosted, true) => Plan::Derived,
        (RepositoryType::Hosted, false) if companion => Plan::Derived,
        (_, true) => Plan::Import,
        (_, false) if remote::wanted(path).is_some() => Plan::Import,
        _ => Plan::Skip("the path is neither an index page nor a package file".to_owned()),
    }
}

fn form_fields(attributes: &Value) -> Vec<(String, String)> {
    let Some(pypi) = attributes.get("pypi").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    for (key, value) in pypi {
        match value {
            Value::String(text) => match key.as_str() {
                "classifiers" | "requires_dist" | "project_urls" | "platform"
                | "provides_extra" => {
                    for line in text.split('\n').filter(|line| !line.trim().is_empty()) {
                        fields.push((key.clone(), line.to_owned()));
                    }
                }
                _ => fields.push((key.clone(), text.clone())),
            },
            Value::Array(items) => {
                for item in items.iter().filter_map(Value::as_str) {
                    fields.push((key.clone(), item.to_owned()));
                }
            }
            _ => {}
        }
    }
    fields
}

async fn archived(file: &Imported, filename: &str) -> Option<String> {
    let source = archive::Blob {
        store: file.store.clone(),
        id: file.blob,
        size: file.size,
    };
    archive::metadata(&source, filename).await.ok().flatten()
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    if file.path.starts_with("/simple/") {
        return Ok(Some(Described::source(
            file.put(file.path.clone()).kind("index"),
        )));
    }
    let wanted =
        remote::wanted(&file.path).ok_or_else(|| "the path is not a package file".to_owned())?;
    let project = name::normalize(&wanted.project);
    if file.cached() {
        let listed = File {
            requires_python: file
                .attributes
                .pointer("/pypi/requires_python")
                .and_then(Value::as_str)
                .map(str::to_owned),
            ..Default::default()
        };
        let mut put = remote::describe(&file.path, &project, &wanted, &listed, None);
        if !wanted.metadata
            && let Some(text) = archived(file, &wanted.filename).await
        {
            let metadata = CoreMetadata::parse(&text);
            if let (Some(target), Value::Object(found)) = (
                put.attributes
                    .get_mut("pypi")
                    .and_then(Value::as_object_mut),
                metadata.asset_attributes()["pypi"].take(),
            ) {
                target.extend(found);
            }
            if let Some(component) = put.component.as_mut() {
                component.attributes = metadata.component_attributes();
            }
        }
        return Ok(Some(Described::source(put)));
    }
    if wanted.metadata {
        return Ok(None);
    }
    let fields = form_fields(&file.attributes);
    let mut metadata = CoreMetadata::from_form(
        fields
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    );
    let text = archived(file, &wanted.filename).await;
    if (metadata.name().is_none() || metadata.version().is_none())
        && let Some(text) = &text
    {
        metadata = CoreMetadata::parse(text);
    }
    let package = Package {
        metadata,
        filename: wanted.filename.clone(),
    };
    let (_, key) = package.path().map_err(|error| error.to_string())?;
    let put = file
        .put(file.path.clone())
        .component(key.clone())
        .kind("package")
        .content_type(name::content_type(&package.filename))
        .attributes(package.metadata.asset_attributes());
    let mut described = Described::source(put);
    if name::is_wheel(&package.filename)
        && let Some(text) = text
    {
        let companion = file
            .put(format!("{}{METADATA_SUFFIX}", file.path))
            .component(key)
            .kind("metadata")
            .content_type("text/plain");
        described = described.with(companion, text.into_bytes());
    }
    Ok(Some(described))
}
