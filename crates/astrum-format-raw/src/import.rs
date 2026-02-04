use astrum_content::import::{Described, Imported, Plan};
use astrum_repo::RepositoryType;

pub fn plan(_repository: RepositoryType, _path: &str, _kind: &str) -> Plan {
    Plan::Import
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    Ok(Some(Described::source(
        file.put(file.path.clone())
            .component(crate::component(&file.path)),
    )))
}
