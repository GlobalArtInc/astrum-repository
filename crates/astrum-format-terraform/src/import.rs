use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::{Map, Value};

use crate::hosted::{module_attributes, module_component, provider_attributes, provider_component};
use crate::paths::{self, Endpoint};
use crate::{METADATA, MODULE_ARCHIVE};

const BASE_KEYS: &[&str] = &["kind", "namespace", "name", "provider", "version"];

pub fn plan(_repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match Endpoint::parse(path) {
        Endpoint::ModuleArchive(..) => Plan::Import,
        Endpoint::ProviderFile(..) => {
            Plan::Skip("Terraform providers cannot be transferred yet".to_owned())
        }
        _ => Plan::Skip("the path is not a module archive".to_owned()),
    }
}

fn extra(attributes: &Value) -> Value {
    let mut kept = Map::new();
    if let Some(values) = attributes.get(crate::ID).and_then(Value::as_object) {
        for (key, value) in values {
            if !BASE_KEYS.contains(&key.as_str()) {
                kept.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(kept)
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    match Endpoint::parse(&file.path) {
        Endpoint::ModuleArchive(module, name) => {
            let head = file.head(8).await.map_err(|error| error.to_string())?;
            let extension = match (paths::archive_extension(&name), paths::sniff(&head)) {
                (Some(extension), Some(sniffed)) if extension == sniffed => extension,
                _ => {
                    return Err(
                        "A module package must be a .zip or .tar.gz archive of the module directory"
                            .to_owned(),
                    );
                }
            };
            let subdir = file
                .attributes
                .pointer("/terraform/subdir")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let put = file
                .put(file.path.clone())
                .kind(MODULE_ARCHIVE)
                .component(module_component(&module))
                .content_type(match extension {
                    "zip" => "application/zip",
                    _ => "application/gzip",
                })
                .attributes(module_attributes(&module, subdir));
            Ok(Some(Described::source(put)))
        }
        Endpoint::ProviderFile(provider, _, kind) => {
            let mut extra = extra(&file.attributes);
            if let (paths::FileKind::Archive { os, arch }, Value::Object(values)) =
                (&kind, &mut extra)
            {
                values.insert("os".to_owned(), Value::String(os.clone()));
                values.insert("arch".to_owned(), Value::String(arch.clone()));
            }
            let put = file
                .put(file.path.clone())
                .kind(kind.asset_kind())
                .content_type(kind.content_type())
                .component(provider_component(&provider))
                .attributes(provider_attributes(&provider, extra));
            Ok(Some(Described::source(put)))
        }
        Endpoint::Discovery
        | Endpoint::ModuleVersions(_)
        | Endpoint::ProviderVersions(_)
        | Endpoint::ProviderDownload(..) => Ok(Some(Described::source(
            file.put(file.path.clone())
                .kind(METADATA)
                .content_type("application/json"),
        ))),
        _ => Ok(None),
    }
}
