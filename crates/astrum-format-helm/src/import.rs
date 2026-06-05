use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::json;

use crate::provenance::{self, MAX_PROVENANCE};
use crate::{
    CHART, CHART_TYPE, INDEX, INDEX_KIND, INDEX_TYPE, PROVENANCE, PROVENANCE_SUFFIX,
    PROVENANCE_TYPE, chart, hosted,
};

const MAX_ARCHIVE: u64 = 256 * 1024 * 1024;

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match (repository, path) {
        (RepositoryType::Hosted, INDEX) => Plan::Derived,
        (_, INDEX) => Plan::Import,
        _ if path.ends_with(".tgz") || path.ends_with(PROVENANCE_SUFFIX) => Plan::Import,
        _ => Plan::Skip("the path is neither the index, a chart nor a provenance file".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    if file.path == INDEX {
        return Ok(Some(Described::source(
            file.put(INDEX).kind(INDEX_KIND).content_type(INDEX_TYPE),
        )));
    }
    if file.path.ends_with(PROVENANCE_SUFFIX) {
        let bytes = file
            .bytes(MAX_PROVENANCE as u64)
            .await
            .map_err(|error| error.to_string())?;
        let text = String::from_utf8(bytes)
            .map_err(|_| "the provenance file is not UTF-8 text".to_owned())?;
        let chart = provenance::chart(&text).map_err(|error| error.to_string())?;
        let put = file
            .put(file.path.clone())
            .component(hosted::key(&chart))
            .kind(PROVENANCE)
            .content_type(PROVENANCE_TYPE)
            .attributes(json!({}));
        return Ok(Some(Described::source(put)));
    }
    let bytes = file
        .bytes(MAX_ARCHIVE)
        .await
        .map_err(|error| error.to_string())?;
    let chart = tokio::task::spawn_blocking(move || chart::read(&bytes))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let put = file
        .put(file.path.clone())
        .component(hosted::key(&chart))
        .kind(CHART)
        .content_type(CHART_TYPE)
        .attributes(chart.attributes());
    Ok(Some(Described::source(put)))
}
