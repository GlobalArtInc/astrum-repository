use std::collections::{BTreeMap, BTreeSet};

use astrum_content::{Ctx, View};

use crate::catalog::{self, Catalog};
use crate::connect::{Reply, Status};
use crate::digest::{self, Value};
use crate::labels;
use crate::model::{self, Commit, Dependency, Module};
use crate::names::{self, Kind, LabelKey};
use crate::proto;

struct Prepared {
    module: Module,
    files: BTreeMap<String, Vec<u8>>,
    source: String,
    labels: Vec<LabelKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Provider<'a> {
    Remote(&'a str),
    Local(usize),
}

#[derive(Default, Clone)]
struct Plan {
    remote: BTreeSet<String>,
    local: BTreeSet<usize>,
}

pub struct Computed {
    pub digest: Value,
    pub closure: BTreeMap<String, Value>,
    pub direct: Vec<String>,
}

fn dependency_value(dependency: &Dependency) -> Reply<Value> {
    digest::parse(&dependency.digest)
        .ok_or_else(|| Status::internal(format!("the digest of {} is unreadable", dependency.id)))
}

fn closure_of(commit: &Commit) -> Reply<BTreeMap<String, Value>> {
    let mut closure = BTreeMap::new();
    closure.insert(
        commit.id.clone(),
        digest::parse(&commit.digest).ok_or_else(|| {
            Status::internal(format!("the digest of {} is unreadable", commit.id))
        })?,
    );
    for dependency in &commit.deps {
        closure.insert(dependency.id.clone(), dependency_value(dependency)?);
    }
    Ok(closure)
}

fn plan_for(
    prepared: &[Prepared],
    remote: &BTreeMap<String, Commit>,
    dep_ids: &[String],
) -> Reply<Vec<Plan>> {
    if prepared.len() == 1 {
        return Ok(vec![Plan {
            remote: dep_ids.iter().cloned().collect(),
            local: BTreeSet::new(),
        }]);
    }
    let mut providers: BTreeMap<&str, Provider> = BTreeMap::new();
    for (id, commit) in remote {
        for path in &commit.files {
            if names::classify(path) == Some(Kind::Proto) {
                providers.entry(path).or_insert(Provider::Remote(id));
            }
        }
    }
    for (index, one) in prepared.iter().enumerate() {
        for path in one.files.keys() {
            if names::classify(path) == Some(Kind::Proto) {
                providers.insert(path, Provider::Local(index));
            }
        }
    }
    let mut plans = vec![Plan::default(); prepared.len()];
    for (index, one) in prepared.iter().enumerate() {
        for (path, content) in &one.files {
            if names::classify(path) != Some(Kind::Proto) {
                continue;
            }
            for import in names::imports(&String::from_utf8_lossy(content)) {
                match providers.get(import.as_str()) {
                    Some(Provider::Remote(id)) => {
                        plans[index].remote.insert((*id).to_owned());
                    }
                    Some(Provider::Local(other)) if *other != index => {
                        plans[index].local.insert(*other);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(plans)
}

fn order_of(plans: &[Plan]) -> Reply<Vec<usize>> {
    let mut done: Vec<usize> = Vec::with_capacity(plans.len());
    while done.len() < plans.len() {
        let before = done.len();
        for (index, plan) in plans.iter().enumerate() {
            if !done.contains(&index) && plan.local.iter().all(|other| done.contains(other)) {
                done.push(index);
            }
        }
        if done.len() == before {
            return Err(Status::invalid(
                "the modules of the upload import each other in a cycle",
            ));
        }
    }
    Ok(done)
}

pub fn compute(
    files: &BTreeMap<String, Vec<u8>>,
    direct: &[(String, BTreeMap<String, Value>)],
) -> Computed {
    let mut closure: BTreeMap<String, Value> = BTreeMap::new();
    for (_, reachable) in direct {
        for (id, value) in reachable {
            closure.insert(id.clone(), *value);
        }
    }
    let dependencies: Vec<Value> = closure.values().copied().collect();
    let digest = digest::module_digest(
        files
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_slice())),
        &dependencies,
    );
    let covered: BTreeSet<&String> = direct
        .iter()
        .flat_map(|(id, reachable)| reachable.keys().filter(move |other| *other != id))
        .collect();
    let mut kept: Vec<String> = direct
        .iter()
        .map(|(id, _)| id.clone())
        .filter(|id| !covered.contains(id))
        .collect();
    kept.sort();
    kept.dedup();
    Computed {
        digest,
        closure,
        direct: kept,
    }
}

pub async fn upload(ctx: &Ctx, request: proto::UploadRequest) -> Reply<proto::UploadResponse> {
    if request.contents.is_empty() {
        return Err(Status::invalid("at least one content is required"));
    }
    catalog::writable(ctx)?;
    let catalog = Catalog::new(ctx);

    let mut prepared = Vec::with_capacity(request.contents.len());
    let mut modules = BTreeSet::new();
    for content in request.contents {
        let reference = names::module_ref(content.module_ref.as_ref())?;
        let module = catalog.module_of(&reference).await?;
        if !modules.insert(module.id.clone()) {
            return Err(Status::invalid(format!(
                "the module {}/{} is uploaded twice in one request",
                module.owner, module.name
            )));
        }
        let files = names::module_files(content.files)?;
        let mut labels = Vec::new();
        for reference in &content.scoped_label_refs {
            let key = names::scoped_label_ref(reference)?;
            if !labels.contains(&key) {
                labels.push(key);
            }
        }
        if content.source_control_url.len() > 255 {
            return Err(Status::invalid(
                "a source control URL has at most 255 characters",
            ));
        }
        prepared.push(Prepared {
            module,
            files,
            source: content.source_control_url,
            labels,
        });
    }

    let mut dep_ids = Vec::new();
    for id in &request.dep_commit_ids {
        let id = names::commit_id(id)
            .ok_or_else(|| Status::invalid(format!("{id:?} is not a commit id")))?;
        if !dep_ids.contains(&id) {
            dep_ids.push(id);
        }
    }
    let mut remote: BTreeMap<String, Commit> = BTreeMap::new();
    for id in &dep_ids {
        let commit = catalog.dependency(id).await?;
        catalog.named_module(&commit.owner, &commit.module).await?;
        remote.insert(id.clone(), commit);
    }

    let plans = plan_for(&prepared, &remote, &dep_ids)?;
    let order = order_of(&plans)?;

    let mut computed: Vec<Option<(Computed, String)>> = (0..prepared.len()).map(|_| None).collect();
    for &index in &order {
        let mut direct: Vec<(String, BTreeMap<String, Value>)> = Vec::new();
        for id in &plans[index].remote {
            let commit = remote
                .get(id)
                .ok_or_else(|| Status::internal("a dependency was not loaded"))?;
            direct.push((id.clone(), closure_of(commit)?));
        }
        for other in &plans[index].local {
            let (done, id) = computed[*other]
                .as_ref()
                .ok_or_else(|| Status::internal("a local dependency was not computed"))?;
            let mut reachable = done.closure.clone();
            reachable.insert(id.clone(), done.digest);
            direct.push((id.clone(), reachable));
        }
        let result = compute(&prepared[index].files, &direct);
        let id = model::derived_commit_id(&prepared[index].module.id, &hex::encode(result.digest));
        computed[index] = Some((result, id));
    }

    let mut owners: Vec<&str> = prepared
        .iter()
        .map(|one| one.module.owner.as_str())
        .collect();
    owners.sort_unstable();
    owners.dedup();
    let mut locks = Vec::with_capacity(owners.len());
    for owner in owners {
        locks.push(catalog.lock(owner).await?);
    }

    let mut commits: Vec<Option<Commit>> = (0..prepared.len()).map(|_| None).collect();
    for &index in &order {
        let one = &prepared[index];
        let (result, id) = computed[index]
            .as_ref()
            .ok_or_else(|| Status::internal("a content was not computed"))?;
        let module = catalog
            .module(&one.module.owner, &one.module.name)
            .await?
            .map(|(_, module)| module)
            .ok_or_else(|| Status::not_found("the module was deleted"))?;
        catalog::require(
            ctx,
            View::Add,
            &catalog::commit_path(&module.owner, &module.name, id),
        )?;
        let commit = match catalog.commit(&module.owner, &module.name, id).await? {
            Some(existing) => existing,
            None => {
                let commit = Commit {
                    id: id.clone(),
                    owner_id: module.owner_id.clone(),
                    module_id: module.id.clone(),
                    owner: module.owner.clone(),
                    module: module.name.clone(),
                    created: model::now(),
                    digest: hex::encode(result.digest),
                    source_control_url: one.source.clone(),
                    deps: result
                        .closure
                        .iter()
                        .map(|(id, value)| Dependency {
                            id: id.clone(),
                            digest: hex::encode(value),
                        })
                        .collect(),
                    direct: result.direct.clone(),
                    files: one.files.keys().cloned().collect(),
                };
                catalog.save_commit(&commit, &one.files).await?;
                commit
            }
        };
        let mut names = Vec::new();
        for key in &one.labels {
            let name = match key {
                LabelKey::Name(name) => name.clone(),
                LabelKey::Id(label_id) => {
                    let label = catalog
                        .label_by_id(label_id)
                        .await?
                        .filter(|label| label.module_id == module.id)
                        .ok_or_else(|| {
                            Status::not_found(format!(
                                "the label {label_id} does not exist in {}/{}",
                                module.owner, module.name
                            ))
                        })?;
                    label.name
                }
            };
            if !names.contains(&name) {
                names.push(name);
            }
        }
        if names.is_empty() {
            names.push(module.default_label.clone());
        }
        for name in names {
            labels::point(&catalog, &module, &name, &commit, true).await?;
        }
        commits[index] = Some(commit);
    }
    drop(locks);
    Ok(proto::UploadResponse {
        commits: commits
            .into_iter()
            .flatten()
            .map(|commit| commit.to_proto())
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(proto: &str) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        files.insert("a/v1/a.proto".to_owned(), proto.as_bytes().to_vec());
        files
    }

    fn reachable(entries: &[(&str, &[u8])]) -> BTreeMap<String, Value> {
        entries
            .iter()
            .map(|(id, seed)| ((*id).to_owned(), digest::shake256(seed)))
            .collect()
    }

    fn prepared(imports: &[&str], paths: &[&str]) -> Prepared {
        let now = model::now();
        let mut files = BTreeMap::new();
        for path in paths {
            let source: String = imports
                .iter()
                .map(|import| format!("import \"{import}\";\n"))
                .collect();
            files.insert((*path).to_owned(), source.into_bytes());
        }
        Prepared {
            module: Module {
                id: model::new_id(),
                owner_id: model::new_id(),
                owner: "acme".to_owned(),
                name: "m".to_owned(),
                created: now,
                updated: now,
                visibility: model::Visibility::Public,
                state: model::State::Active,
                description: String::new(),
                url: String::new(),
                default_label: "main".to_owned(),
            },
            files,
            source: String::new(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn without_dependencies_the_digest_comes_from_the_files_alone() {
        let one = compute(&files("x"), &[]);
        assert_eq!(
            one.digest,
            digest::module_digest([("a/v1/a.proto", b"x".as_slice())], &[])
        );
        assert!(one.closure.is_empty() && one.direct.is_empty());
    }

    #[test]
    fn the_digest_covers_every_transitive_dependency() {
        let direct = vec![("b".to_owned(), reachable(&[("b", b"B"), ("c", b"C")]))];
        let one = compute(&files("x"), &direct);
        assert_eq!(one.closure.len(), 2);
        assert_eq!(one.direct, vec!["b".to_owned()]);
        let expected = digest::module_digest(
            [("a/v1/a.proto", b"x".as_slice())],
            &[digest::shake256(b"B"), digest::shake256(b"C")],
        );
        assert_eq!(one.digest, expected);
        assert_ne!(one.digest, compute(&files("x"), &[]).digest);
    }

    #[test]
    fn a_dependency_reachable_through_another_is_not_direct() {
        let direct = vec![
            ("b".to_owned(), reachable(&[("b", b"B"), ("c", b"C")])),
            ("c".to_owned(), reachable(&[("c", b"C")])),
        ];
        let one = compute(&files("x"), &direct);
        assert_eq!(one.direct, vec!["b".to_owned()]);
        assert_eq!(one.closure.len(), 2);
    }

    #[test]
    fn one_content_trusts_the_dependencies_of_the_client() {
        let one = vec![prepared(&[], &["a/v1/a.proto"])];
        let ids = vec!["x".repeat(32), "y".repeat(32)];
        let plans = plan_for(&one, &BTreeMap::new(), &ids).unwrap();
        assert_eq!(plans[0].remote.len(), 2);
        assert!(plans[0].local.is_empty());
    }

    #[test]
    fn several_contents_depend_on_each_other_through_imports() {
        let base = prepared(&[], &["base/v1/base.proto"]);
        let top = prepared(
            &["base/v1/base.proto", "google/protobuf/any.proto"],
            &["top/v1/top.proto"],
        );
        let plans = plan_for(&[top, base], &BTreeMap::new(), &[]).unwrap();
        assert_eq!(plans[0].local, BTreeSet::from([1]));
        assert!(plans[1].local.is_empty());
        assert_eq!(order_of(&plans).unwrap(), vec![1, 0]);
    }

    #[test]
    fn imports_that_cross_the_modules_in_a_circle_are_refused() {
        let one = prepared(&["two/v1/two.proto"], &["one/v1/one.proto"]);
        let two = prepared(&["one/v1/one.proto"], &["two/v1/two.proto"]);
        let plans = plan_for(&[one, two], &BTreeMap::new(), &[]).unwrap();
        assert!(order_of(&plans).is_err());
    }
}
