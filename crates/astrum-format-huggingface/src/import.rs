use astrum_content::import::{Described, Imported, Plan};
use astrum_content::store::ComponentKey;
use astrum_repo::RepositoryType;
use serde_json::{Value, json};

use crate::paths::{RepoType, is_commit};
use crate::{FILE, ID, REVISION};

enum Stored {
    Revision,
    File {
        kind: RepoType,
        repo: String,
        commit: String,
        file: String,
    },
}

fn classify(path: &str, attributes: &Value) -> Option<Stored> {
    let mut segments = path.trim_start_matches('/').splitn(2, '/');
    let kind = RepoType::from_plural(segments.next()?)?;
    let rest = segments.next()?;
    if rest.ends_with(".json") && attributes.get(ID).is_none() {
        return Some(Stored::Revision);
    }
    let known = attributes.get(ID)?;
    let repo = known.get("repoId")?.as_str()?.to_owned();
    let commit = known.get("commit")?.as_str()?.to_owned();
    let file = known.get("file")?.as_str()?.to_owned();
    let expected = format!("/{}/{repo}/{commit}/{file}", kind.plural());
    (expected == path && is_commit(&commit)).then_some(Stored::File {
        kind,
        repo,
        commit,
        file,
    })
}

pub fn plan(repository: RepositoryType, path: &str, _kind: &str) -> Plan {
    match repository {
        RepositoryType::Proxy
            if RepoType::from_plural(
                path.trim_start_matches('/')
                    .split('/')
                    .next()
                    .unwrap_or_default(),
            )
            .is_some() =>
        {
            Plan::Import
        }
        _ => Plan::Skip("the path is not a cached revision or file".to_owned()),
    }
}

pub async fn describe(file: &Imported) -> Result<Option<Described>, String> {
    match classify(&file.path, &file.attributes) {
        Some(Stored::Revision) => Ok(Some(Described::source(
            file.put(file.path.clone())
                .kind(REVISION)
                .content_type("application/json"),
        ))),
        Some(Stored::File {
            kind,
            repo,
            commit,
            file: name,
        }) => {
            let known = &file.attributes[ID];
            let revisions = known
                .get("revisions")
                .filter(|value| value.is_array())
                .cloned()
                .unwrap_or_else(|| json!([]));
            let identity = json!({
                "repoId": repo,
                "repoType": kind.kind(),
                "commit": commit,
                "revisions": revisions,
            });
            let mut attributes = identity.clone();
            attributes["file"] = Value::String(name);
            attributes["lfs"] =
                Value::Bool(known.get("lfs").and_then(Value::as_bool).unwrap_or(false));
            if let Some(etag) = known.get("etag").and_then(Value::as_str) {
                attributes["etag"] = Value::String(etag.to_owned());
            }
            let put = file
                .put(file.path.clone())
                .kind(FILE)
                .component(ComponentKey {
                    namespace: kind.plural().to_owned(),
                    name: repo,
                    version: commit,
                    kind: kind.kind().to_owned(),
                    attributes: json!({ ID: identity }),
                })
                .attributes(json!({ ID: attributes }));
            Ok(Some(Described::source(put)))
        }
        None => Err("the path is not a cached revision or file".to_owned()),
    }
}
