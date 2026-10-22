use std::collections::{BTreeMap, BTreeSet};

use astrum_content::{Ctx, View};

use crate::catalog::{self, Catalog};
use crate::connect::{Reply, Status};
use crate::model::Commit;
use crate::names::{self, Kind, OwnerKey};
use crate::proto::{self, list_commits_request::Order};
use crate::resolve::{Found, page};

pub async fn get(ctx: &Ctx, request: proto::GetCommitsRequest) -> Reply<proto::GetCommitsResponse> {
    if request.resource_refs.is_empty() {
        return Err(Status::invalid("at least one reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut commits = Vec::with_capacity(request.resource_refs.len());
    for reference in &request.resource_refs {
        let reference = names::resource_ref(Some(reference))?;
        commits.push(catalog.commit_for(&reference).await?.1.to_proto());
    }
    Ok(proto::GetCommitsResponse { commits })
}

pub async fn list(
    ctx: &Ctx,
    request: proto::ListCommitsRequest,
) -> Reply<proto::ListCommitsResponse> {
    let catalog = Catalog::new(ctx);
    let reference = names::resource_ref(request.resource_ref.as_ref())?;
    let mut commits = match catalog.resolve(&reference).await? {
        Found::Module(module) => catalog.commits(&module.owner, &module.name).await?,
        found => vec![catalog.commit_of(found).await?.1],
    };
    if !request.id_query.is_empty() {
        let query = request.id_query.to_lowercase();
        commits.retain(|commit| commit.id.contains(&query));
    }
    match Order::try_from(request.order).unwrap_or(Order::Unspecified) {
        Order::CreateTimeAsc => commits.sort_by_key(|commit| commit.created),
        Order::CreateTimeDesc | Order::Unspecified => {
            commits.sort_by_key(|commit| std::cmp::Reverse(commit.created))
        }
    }
    let (kept, next_page_token) = page(commits, request.page_size, &request.page_token)?;
    Ok(proto::ListCommitsResponse {
        next_page_token,
        commits: kept.iter().map(Commit::to_proto).collect(),
    })
}

pub async fn graph(ctx: &Ctx, request: proto::GetGraphRequest) -> Reply<proto::GetGraphResponse> {
    if request.resource_refs.is_empty() {
        return Err(Status::invalid("at least one reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut requested: Vec<Commit> = Vec::with_capacity(request.resource_refs.len());
    let mut modules = BTreeSet::new();
    for reference in &request.resource_refs {
        let reference = names::resource_ref(Some(reference))?;
        let (module, commit) = catalog.commit_for(&reference).await?;
        if !modules.insert(module.id.clone()) {
            return Err(Status::invalid(format!(
                "the module {}/{} is referenced twice",
                module.owner, module.name
            )));
        }
        requested.push(commit);
    }
    let mut nodes: BTreeMap<String, Commit> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for commit in requested {
        if !nodes.contains_key(&commit.id) {
            order.push(commit.id.clone());
            nodes.insert(commit.id.clone(), commit);
        }
    }
    let mut pending: Vec<String> = order.clone();
    while let Some(id) = pending.pop() {
        let wanted: Vec<String> = nodes[&id]
            .deps
            .iter()
            .map(|dependency| dependency.id.clone())
            .collect();
        for dependency in wanted {
            if nodes.contains_key(&dependency) {
                continue;
            }
            let commit = catalog.dependency(&dependency).await?;
            order.push(commit.id.clone());
            pending.push(commit.id.clone());
            nodes.insert(commit.id.clone(), commit);
        }
    }
    let mut edges = Vec::new();
    for id in &order {
        for target in &nodes[id].direct {
            edges.push(proto::graph::Edge {
                from_node: Some(proto::graph::Node {
                    commit_id: id.clone(),
                }),
                to_node: Some(proto::graph::Node {
                    commit_id: target.clone(),
                }),
            });
        }
    }
    Ok(proto::GetGraphResponse {
        graph: Some(proto::Graph {
            commits: order.iter().map(|id| nodes[id].to_proto()).collect(),
            edges,
            registry_commit_ids: Vec::new(),
        }),
    })
}

fn selected(path: &str, wanted: &[String]) -> bool {
    wanted.is_empty()
        || wanted.iter().any(|candidate| {
            path == candidate
                || path
                    .strip_prefix(candidate.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

pub async fn download(
    ctx: &Ctx,
    request: proto::DownloadRequest,
) -> Reply<proto::DownloadResponse> {
    if request.values.is_empty() {
        return Err(Status::invalid("at least one reference is required"));
    }
    if request.values.len() > 250 {
        return Err(Status::invalid(
            "at most 250 references can be downloaded at once",
        ));
    }
    let catalog = Catalog::new(ctx);
    let mut contents = Vec::with_capacity(request.values.len());
    for value in &request.values {
        let reference = names::resource_ref(value.resource_ref.as_ref())?;
        let (module, commit) = catalog.commit_for(&reference).await?;
        catalog::require(
            ctx,
            View::Read,
            &catalog::commit_path(&module.owner, &module.name, &commit.id),
        )?;
        let kinds: Vec<Kind> = value
            .file_types
            .iter()
            .map(|file_type| {
                names::kind_of(*file_type)
                    .ok_or_else(|| Status::invalid("the file type is not valid"))
            })
            .collect::<Reply<_>>()?;
        for path in &value.paths {
            names::valid_file_path(path)?;
        }
        let typed: Vec<&String> = commit
            .files
            .iter()
            .filter(|path| {
                kinds.is_empty() || names::classify(path).is_some_and(|kind| kinds.contains(&kind))
            })
            .collect();
        if !value.paths.is_empty() && !value.paths_allow_not_exist {
            for wanted in &value.paths {
                if !typed
                    .iter()
                    .any(|path| selected(path, std::slice::from_ref(wanted)))
                {
                    return Err(Status::not_found(format!(
                        "the path {wanted} has no files in the commit {}",
                        commit.id
                    )));
                }
            }
        }
        let mut files = Vec::new();
        for path in typed {
            if selected(path, &value.paths) {
                files.push(proto::File {
                    path: path.clone(),
                    content: catalog.file(&commit, path).await?,
                });
            }
        }
        contents.push(proto::download_response::Content {
            commit: Some(commit.to_proto()),
            files,
        });
    }
    Ok(proto::DownloadResponse { contents })
}

pub async fn resources(
    ctx: &Ctx,
    request: proto::GetResourcesRequest,
) -> Reply<proto::GetResourcesResponse> {
    if request.resource_refs.is_empty() {
        return Err(Status::invalid("at least one reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut resources = Vec::with_capacity(request.resource_refs.len());
    for reference in &request.resource_refs {
        let reference = names::resource_ref(Some(reference))?;
        let value = match catalog.resolve(&reference).await? {
            Found::Module(module) => proto::resource::Value::Module(module.to_proto()),
            Found::Label(_, label) => proto::resource::Value::Label(label.to_proto()),
            Found::Commit(_, commit) => proto::resource::Value::Commit(commit.to_proto()),
        };
        resources.push(proto::Resource { value: Some(value) });
    }
    Ok(proto::GetResourcesResponse { resources })
}

pub async fn owners(
    ctx: &Ctx,
    request: proto::GetOwnersRequest,
) -> Reply<proto::GetOwnersResponse> {
    if request.owner_refs.is_empty() {
        return Err(Status::invalid("at least one owner reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut owners = Vec::with_capacity(request.owner_refs.len());
    for reference in &request.owner_refs {
        let found =
            match names::owner_ref(reference)? {
                OwnerKey::Name(name) => {
                    catalog::require(ctx, View::Read, &catalog::owner_path(&name))?;
                    catalog.owner(&name).await?.ok_or_else(|| {
                        Status::not_found(format!("the owner {name} does not exist"))
                    })?
                }
                OwnerKey::Id(id) => {
                    let found = catalog.owner_by_id(&id).await?.ok_or_else(|| {
                        Status::not_found(format!("the owner {id} does not exist"))
                    })?;
                    catalog::require(ctx, View::Read, &catalog::owner_path(&found.1.name))?;
                    found
                }
            };
        owners.push(found.1.to_proto());
    }
    Ok(proto::GetOwnersResponse { owners })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_select_files_and_directories() {
        let none: Vec<String> = Vec::new();
        assert!(selected("a/b.proto", &none));
        assert!(selected("a/b.proto", &["a/b.proto".to_owned()]));
        assert!(selected("a/b.proto", &["a".to_owned()]));
        assert!(!selected("ab/c.proto", &["a".to_owned()]));
        assert!(!selected("a/b.proto", &["a/b".to_owned()]));
        assert!(selected("LICENSE", &["LICENSE".to_owned(), "x".to_owned()]));
    }
}
