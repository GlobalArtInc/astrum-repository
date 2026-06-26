use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;
use serde_json::json;

use crate::hosted::component;
use crate::{OTHER, RPM, RPM_TYPE, paths, remote, rpm};

const FIRST_READ: u64 = 64 * 1024;

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match repository {
        RepositoryType::Hosted if paths::in_repodata(path) => Plan::Derived,
        _ => Plan::Import,
    }
}

async fn package(file: &Imported) -> Result<rpm::Package, String> {
    let mut prefix = file
        .head(FIRST_READ.min(file.size))
        .await
        .map_err(|error| error.to_string())?;
    loop {
        match rpm::needed(&prefix).map_err(|error| error.to_string())? {
            None => break,
            Some(wanted) if wanted as u64 > file.size || prefix.len() as u64 >= file.size => {
                return Err("the RPM package is truncated".to_owned());
            }
            Some(wanted) => {
                let offset = prefix.len() as u64;
                let more = file
                    .range(offset, wanted as u64 - offset)
                    .await
                    .map_err(|error| error.to_string())?;
                if more.is_empty() {
                    return Err("the RPM package is truncated".to_owned());
                }
                prefix.extend_from_slice(&more);
            }
        }
    }
    tokio::task::spawn_blocking(move || rpm::parse(&prefix))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    if file.cached() {
        let (kind, _) = remote::classify(&file.path);
        let mut put = remote::describe(&file.path, kind);
        put.verified = true;
        if kind == RPM
            && let Ok(package) = package(file).await
        {
            put.attributes = json!({"yum": package.attributes()});
        }
        return Ok(Some(Described::source(put)));
    }
    if !paths::is_rpm(&file.path) {
        return Ok(Some(Described::source(
            file.put(file.path.clone()).kind(OTHER),
        )));
    }
    let package = package(file).await?;
    let put = file
        .put(file.path.clone())
        .component(component(&package))
        .kind(RPM)
        .content_type(RPM_TYPE)
        .attributes(json!({"yum": package.attributes()}));
    Ok(Some(Described::source(put)))
}
