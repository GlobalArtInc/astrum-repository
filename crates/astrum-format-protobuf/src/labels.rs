use astrum_content::{Ctx, View};

use crate::catalog::{self, Catalog, MAX_HISTORY};
use crate::connect::{Reply, Status};
use crate::model::{self, Association, Commit, Label, LabelFile, Module};
use crate::names::{self, Ref};
use crate::proto::{
    self, CommitCheckStatus,
    list_label_history_request::Order as HistoryOrder,
    list_labels_request::{ArchiveFilter, Order},
};
use crate::resolve::{Found, page};

pub async fn point(
    catalog: &Catalog<'_>,
    module: &Module,
    name: &str,
    commit: &Commit,
    force: bool,
) -> Reply<Label> {
    names::valid_label(name)?;
    let path = catalog::label_path(&module.owner, &module.name, name);
    let now = model::now();
    match catalog.label(&module.owner, &module.name, name).await? {
        None => {
            catalog::require(catalog.ctx, View::Add, &path)?;
            let label = Label {
                id: model::new_id(),
                owner_id: module.owner_id.clone(),
                module_id: module.id.clone(),
                owner: module.owner.clone(),
                module: module.name.clone(),
                name: name.to_owned(),
                created: now,
                updated: now,
                archived: None,
                commit: commit.id.clone(),
            };
            let file = LabelFile {
                label: label.clone(),
                history: vec![Association {
                    commit: commit.id.clone(),
                    time: now,
                }],
            };
            catalog.save_label(&file).await?;
            Ok(label)
        }
        Some(existing) => {
            if existing.commit == commit.id && existing.archived.is_none() {
                return Ok(existing);
            }
            catalog::require(catalog.ctx, View::Edit, &path)?;
            if !force && existing.commit != commit.id {
                let current = catalog
                    .commit(&module.owner, &module.name, &existing.commit)
                    .await?;
                if current.is_some_and(|current| commit.created < current.created) {
                    return Err(Status::precondition(format!(
                        "the commit {} is older than the commit the label {name} points to",
                        commit.id
                    )));
                }
            }
            let mut file = catalog.history(&existing).await?;
            if file.label.commit != commit.id {
                file.label.commit = commit.id.clone();
                file.history.push(Association {
                    commit: commit.id.clone(),
                    time: now,
                });
                if file.history.len() > MAX_HISTORY {
                    let surplus = file.history.len() - MAX_HISTORY;
                    file.history.drain(..surplus);
                }
            }
            file.label.archived = None;
            file.label.updated = now;
            catalog.save_label(&file).await?;
            Ok(file.label)
        }
    }
}

async fn label_of(catalog: &Catalog<'_>, reference: &Ref) -> Reply<(Module, Label)> {
    match reference {
        Ref::Id(id) => {
            let label = catalog
                .label_by_id(id)
                .await?
                .ok_or_else(|| Status::not_found(format!("the label {id} does not exist")))?;
            let module = catalog.named_module(&label.owner, &label.module).await?;
            Ok((module, label))
        }
        Ref::Label {
            owner,
            module,
            label,
        } => {
            let module = catalog.named_module(owner, module).await?;
            let found = catalog
                .label(owner, &module.name, label)
                .await?
                .ok_or_else(|| {
                    Status::not_found(format!(
                        "the label {label} does not exist in {owner}/{}",
                        module.name
                    ))
                })?;
            Ok((module, found))
        }
        _ => Err(Status::invalid("a label reference is required")),
    }
}

pub async fn get(ctx: &Ctx, request: proto::GetLabelsRequest) -> Reply<proto::GetLabelsResponse> {
    if request.label_refs.is_empty() {
        return Err(Status::invalid("at least one label reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut labels = Vec::with_capacity(request.label_refs.len());
    for reference in &request.label_refs {
        let reference = names::label_ref(Some(reference))?;
        labels.push(label_of(&catalog, &reference).await?.1.to_proto());
    }
    Ok(proto::GetLabelsResponse { labels })
}

pub async fn list(
    ctx: &Ctx,
    request: proto::ListLabelsRequest,
) -> Reply<proto::ListLabelsResponse> {
    let catalog = Catalog::new(ctx);
    let reference = names::resource_ref(request.resource_ref.as_ref())?;
    let mut labels = match catalog.resolve(&reference).await? {
        Found::Module(module) => catalog.labels(&module.owner, &module.name).await?,
        Found::Label(_, label) => vec![label],
        Found::Commit(module, commit) => catalog
            .labels(&module.owner, &module.name)
            .await?
            .into_iter()
            .filter(|label| label.commit == commit.id)
            .collect(),
    };
    for status in &request.commit_check_statuses {
        if matches!(
            CommitCheckStatus::try_from(*status),
            Ok(CommitCheckStatus::Pending | CommitCheckStatus::Rejected)
        ) {
            return Err(Status::invalid(
                "labels never point to pending or rejected commits",
            ));
        }
    }
    if !request.commit_check_statuses.is_empty()
        && !request.commit_check_statuses.iter().any(|status| {
            matches!(
                CommitCheckStatus::try_from(*status),
                Ok(CommitCheckStatus::Disabled)
            )
        })
    {
        labels.clear();
    }
    match ArchiveFilter::try_from(request.archive_filter).unwrap_or(ArchiveFilter::Unspecified) {
        ArchiveFilter::ArchivedOnly => labels.retain(|label| label.archived.is_some()),
        ArchiveFilter::All => {}
        ArchiveFilter::UnarchivedOnly | ArchiveFilter::Unspecified => {
            labels.retain(|label| label.archived.is_none())
        }
    }
    if !request.name_query.is_empty() {
        let query = request.name_query.to_lowercase();
        labels.retain(|label| label.name.to_lowercase().contains(&query));
    }
    match Order::try_from(request.order).unwrap_or(Order::Unspecified) {
        Order::CreateTimeAsc => labels.sort_by_key(|label| label.created),
        Order::UpdateTimeAsc => labels.sort_by_key(|label| label.updated),
        Order::UpdateTimeDesc => labels.sort_by_key(|label| std::cmp::Reverse(label.updated)),
        Order::CreateTimeDesc | Order::Unspecified => {
            labels.sort_by_key(|label| std::cmp::Reverse(label.created))
        }
    }
    let (kept, next_page_token) = page(labels, request.page_size, &request.page_token)?;
    Ok(proto::ListLabelsResponse {
        next_page_token,
        labels: kept.iter().map(Label::to_proto).collect(),
    })
}

pub async fn history(
    ctx: &Ctx,
    request: proto::ListLabelHistoryRequest,
) -> Reply<proto::ListLabelHistoryResponse> {
    let catalog = Catalog::new(ctx);
    let reference = names::label_ref(request.label_ref.as_ref())?;
    let (module, label) = label_of(&catalog, &reference).await?;
    let file = catalog.history(&label).await?;
    let mut entries = file.history;
    if request.only_commits_with_changed_digests {
        let mut previous: Option<String> = None;
        let mut kept = Vec::with_capacity(entries.len());
        for entry in entries {
            let digest = catalog
                .commit(&module.owner, &module.name, &entry.commit)
                .await?
                .map(|commit| commit.digest);
            if digest.is_some() && digest != previous {
                previous = digest;
                kept.push(entry);
            }
        }
        entries = kept;
    }
    if HistoryOrder::try_from(request.order).unwrap_or(HistoryOrder::Unspecified)
        != HistoryOrder::Asc
    {
        entries.reverse();
    }
    if !request.start_commit_id.is_empty() {
        let start = names::commit_id(&request.start_commit_id)
            .ok_or_else(|| Status::invalid("the start commit id is not valid"))?;
        let position = entries
            .iter()
            .position(|entry| entry.commit == start)
            .ok_or_else(|| {
                Status::not_found(format!(
                    "the commit {start} is not in the history of the label {}",
                    label.name
                ))
            })?;
        entries.drain(..position);
    }
    let (kept, next_page_token) = page(entries, request.page_size, &request.page_token)?;
    let mut values = Vec::with_capacity(kept.len());
    for entry in kept {
        let Some(commit) = catalog
            .commit(&module.owner, &module.name, &entry.commit)
            .await?
        else {
            continue;
        };
        values.push(proto::list_label_history_response::Value {
            commit: Some(commit.to_proto()),
            commit_check_state: Some(proto::CommitCheckState {
                status: CommitCheckStatus::Disabled as i32,
                update_time: Some(model::timestamp(entry.time)),
            }),
        });
    }
    Ok(proto::ListLabelHistoryResponse {
        next_page_token,
        values,
    })
}

pub async fn create_or_update(
    ctx: &Ctx,
    request: proto::CreateOrUpdateLabelsRequest,
) -> Reply<proto::CreateOrUpdateLabelsResponse> {
    if request.values.is_empty() {
        return Err(Status::invalid("at least one label is required"));
    }
    catalog::writable(ctx)?;
    let catalog = Catalog::new(ctx);
    let mut labels = Vec::with_capacity(request.values.len());
    for value in &request.values {
        let reference = names::label_ref(value.label_ref.as_ref())?;
        let commit_id = names::commit_id(&value.commit_id)
            .ok_or_else(|| Status::invalid("the commit id is not valid"))?;
        let (module, name) = match &reference {
            Ref::Label {
                owner,
                module,
                label,
            } => (catalog.named_module(owner, module).await?, label.clone()),
            Ref::Id(id) => {
                let label = catalog
                    .label_by_id(id)
                    .await?
                    .ok_or_else(|| Status::not_found(format!("the label {id} does not exist")))?;
                (
                    catalog.named_module(&label.owner, &label.module).await?,
                    label.name,
                )
            }
            _ => return Err(Status::invalid("a label reference is required")),
        };
        let commit = catalog
            .commit(&module.owner, &module.name, &commit_id)
            .await?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "the commit {commit_id} does not exist in {}/{}",
                    module.owner, module.name
                ))
            })?;
        let _lock = catalog.lock(&module.owner).await?;
        labels.push(
            point(&catalog, &module, &name, &commit, false)
                .await?
                .to_proto(),
        );
    }
    Ok(proto::CreateOrUpdateLabelsResponse { labels })
}

async fn archive_all(ctx: &Ctx, references: &[proto::LabelRef], archive: bool) -> Reply<()> {
    if references.is_empty() {
        return Err(Status::invalid("at least one label reference is required"));
    }
    catalog::writable(ctx)?;
    let catalog = Catalog::new(ctx);
    let mut targets = Vec::with_capacity(references.len());
    for reference in references {
        let reference = names::label_ref(Some(reference))?;
        let (module, label) = label_of(&catalog, &reference).await?;
        catalog::require(
            ctx,
            View::Edit,
            &catalog::label_path(&module.owner, &module.name, &label.name),
        )?;
        if archive && label.name == module.default_label {
            return Err(Status::precondition(format!(
                "the label {} is the default label of {}/{} and cannot be archived",
                label.name, module.owner, module.name
            )));
        }
        targets.push((module, label));
    }
    for (module, label) in targets {
        let _lock = catalog.lock(&module.owner).await?;
        let mut file = catalog.history(&label).await?;
        let now = model::now();
        file.label.archived = match archive {
            true => file.label.archived.or(Some(now)),
            false => None,
        };
        file.label.updated = now;
        catalog.save_label(&file).await?;
    }
    Ok(())
}

pub async fn archive(
    ctx: &Ctx,
    request: proto::ArchiveLabelsRequest,
) -> Reply<proto::ArchiveLabelsResponse> {
    archive_all(ctx, &request.label_refs, true).await?;
    Ok(proto::ArchiveLabelsResponse {})
}

pub async fn unarchive(
    ctx: &Ctx,
    request: proto::UnarchiveLabelsRequest,
) -> Reply<proto::UnarchiveLabelsResponse> {
    archive_all(ctx, &request.label_refs, false).await?;
    Ok(proto::UnarchiveLabelsResponse {})
}
