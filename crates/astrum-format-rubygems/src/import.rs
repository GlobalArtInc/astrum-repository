use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::json;

use crate::{GEM, GEM_TYPE, ID, MAX_INDEX, Route, component, gem, route};

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    let found = route(path);
    match (repository, &found) {
        (RepositoryType::Hosted, Route::Gem(_)) => Plan::Import,
        (RepositoryType::Hosted, _) if found.cached().is_some() => Plan::Derived,
        (_, _) if found.cached().is_some() => Plan::Import,
        _ => Plan::Skip("the path is not a gem, a specification or an index".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    let found = route(&file.path);
    let (kind, _, content_type) = found
        .cached()
        .ok_or_else(|| "the path is not a gem, a specification or an index".to_owned())?;
    if kind != GEM {
        return Ok(Some(Described::source(
            file.put(file.path.clone())
                .kind(kind)
                .content_type(content_type),
        )));
    }
    let bytes = file
        .bytes(MAX_INDEX)
        .await
        .map_err(|error| error.to_string())?;
    let spec = tokio::task::spawn_blocking(move || gem::read_gem(bytes.as_slice()))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| format!("the gem cannot be read: {error:?}"))?;
    let put = file
        .put(file.path.clone())
        .component(component(&spec))
        .kind(GEM)
        .content_type(GEM_TYPE)
        .attributes(json!({ ID: spec.attributes() }));
    Ok(Some(Described::source(put)))
}
