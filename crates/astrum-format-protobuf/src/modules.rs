use std::collections::HashSet;

use astrum_content::{Ctx, View};

use crate::catalog::{self, Catalog};
use crate::connect::{Reply, Status};
use crate::model::{self, Module, Owner, State, Visibility};
use crate::names::{self, OwnerKey};
use crate::proto::{self, list_modules_request::Order};
use crate::resolve::page;

pub const DEFAULT_LABEL: &str = "main";
const MAX_DESCRIPTION: usize = 350;
const MAX_URL: usize = 255;

fn check_description(description: &str) -> Reply<()> {
    match description.chars().count() <= MAX_DESCRIPTION {
        true => Ok(()),
        false => Err(Status::invalid(format!(
            "a description has at most {MAX_DESCRIPTION} characters"
        ))),
    }
}

fn check_url(url: &str) -> Reply<()> {
    if url.len() > MAX_URL {
        return Err(Status::invalid(format!(
            "a URL has at most {MAX_URL} characters"
        )));
    }
    match url.is_empty() || url::Url::parse(url).is_ok() {
        true => Ok(()),
        false => Err(Status::invalid(format!("{url:?} is not a valid URL"))),
    }
}

pub async fn get(ctx: &Ctx, request: proto::GetModulesRequest) -> Reply<proto::GetModulesResponse> {
    if request.module_refs.is_empty() {
        return Err(Status::invalid("at least one module reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut modules = Vec::with_capacity(request.module_refs.len());
    for reference in &request.module_refs {
        let reference = names::module_ref(Some(reference))?;
        modules.push(catalog.module_of(&reference).await?.to_proto());
    }
    Ok(proto::GetModulesResponse { modules })
}

pub async fn list(
    ctx: &Ctx,
    request: proto::ListModulesRequest,
) -> Reply<proto::ListModulesResponse> {
    let catalog = Catalog::new(ctx);
    let mut modules = Vec::new();
    if request.owner_refs.is_empty() {
        modules = catalog.modules(None).await?;
    } else {
        let mut seen = HashSet::new();
        for reference in &request.owner_refs {
            let name = match names::owner_ref(reference)? {
                OwnerKey::Name(name) => name,
                OwnerKey::Id(id) => {
                    catalog
                        .owner_by_id(&id)
                        .await?
                        .ok_or_else(|| Status::not_found(format!("the owner {id} does not exist")))?
                        .1
                        .name
                }
            };
            if seen.insert(name.clone()) {
                modules.extend(catalog.modules(Some(&name)).await?);
            }
        }
    }
    modules.retain(|module| {
        ctx.allowed(
            View::Read,
            &catalog::module_path(&module.owner, &module.name),
        )
    });
    match Order::try_from(request.order).unwrap_or(Order::Unspecified) {
        Order::CreateTimeAsc => modules.sort_by_key(|module| module.created),
        Order::UpdateTimeAsc => modules.sort_by_key(|module| module.updated),
        Order::UpdateTimeDesc => modules.sort_by_key(|module| std::cmp::Reverse(module.updated)),
        Order::CreateTimeDesc | Order::Unspecified => {
            modules.sort_by_key(|module| std::cmp::Reverse(module.created))
        }
    }
    let (kept, next_page_token) = page(modules, request.page_size, &request.page_token)?;
    Ok(proto::ListModulesResponse {
        next_page_token,
        modules: kept.iter().map(Module::to_proto).collect(),
    })
}

pub async fn create(
    ctx: &Ctx,
    request: proto::CreateModulesRequest,
) -> Reply<proto::CreateModulesResponse> {
    if request.values.is_empty() {
        return Err(Status::invalid("at least one module is required"));
    }
    catalog::writable(ctx)?;
    let catalog = Catalog::new(ctx);
    let mut planned = Vec::with_capacity(request.values.len());
    let mut seen = HashSet::new();
    for value in &request.values {
        let owner = match value.owner_ref.as_ref().map(names::owner_ref).transpose()? {
            Some(OwnerKey::Name(name)) => name,
            Some(OwnerKey::Id(id)) => {
                catalog
                    .owner_by_id(&id)
                    .await?
                    .ok_or_else(|| Status::not_found(format!("the owner {id} does not exist")))?
                    .1
                    .name
            }
            None => return Err(Status::invalid("a module needs an owner")),
        };
        if !names::valid_owner(&owner) {
            return Err(Status::invalid(format!(
                "{owner:?} is not a valid owner name: use lowercase letters, digits and dashes, starting with a letter, up to {} characters",
                names::MAX_OWNER
            )));
        }
        if !names::valid_module(&value.name) {
            return Err(Status::invalid(format!(
                "{:?} is not a valid module name: use {} to {} lowercase letters, digits, dashes and underscores, starting with a letter",
                value.name,
                names::MIN_MODULE,
                names::MAX_MODULE
            )));
        }
        let default_label = match value.default_label_name.as_str() {
            "" => DEFAULT_LABEL.to_owned(),
            name => {
                names::valid_label(name)?;
                name.to_owned()
            }
        };
        check_description(&value.description)?;
        check_url(&value.url)?;
        if !seen.insert((owner.clone(), value.name.clone())) {
            return Err(Status::invalid(format!(
                "the module {owner}/{} is listed twice",
                value.name
            )));
        }
        catalog::require(ctx, View::Add, &catalog::module_path(&owner, &value.name))?;
        planned.push((owner, value, default_label));
    }
    let mut created = Vec::with_capacity(planned.len());
    for (owner, value, default_label) in planned {
        let _lock = catalog.lock(&owner).await?;
        if catalog.module(&owner, &value.name).await?.is_some() {
            return Err(Status::exists(format!(
                "the module {owner}/{} already exists",
                value.name
            )));
        }
        let now = model::now();
        let owner_record = match catalog.owner(&owner).await? {
            Some((_, record)) => record,
            None => {
                let record = Owner {
                    id: model::new_id(),
                    name: owner.clone(),
                    created: now,
                };
                catalog.save_owner(&record).await?;
                record
            }
        };
        let module = Module {
            id: model::new_id(),
            owner_id: owner_record.id,
            owner: owner.clone(),
            name: value.name.clone(),
            created: now,
            updated: now,
            visibility: Visibility::from_proto(value.visibility).unwrap_or(Visibility::Private),
            state: State::Active,
            description: value.description.clone(),
            url: value.url.clone(),
            default_label,
        };
        catalog.save_module(&module).await?;
        created.push(module.to_proto());
    }
    Ok(proto::CreateModulesResponse { modules: created })
}

pub async fn update(
    ctx: &Ctx,
    request: proto::UpdateModulesRequest,
) -> Reply<proto::UpdateModulesResponse> {
    if request.values.is_empty() {
        return Err(Status::invalid("at least one module is required"));
    }
    catalog::writable(ctx)?;
    let catalog = Catalog::new(ctx);
    let mut updated = Vec::with_capacity(request.values.len());
    for value in &request.values {
        let reference = names::module_ref(value.module_ref.as_ref())?;
        let found = catalog.module_of(&reference).await?;
        catalog::require(
            ctx,
            View::Edit,
            &catalog::module_path(&found.owner, &found.name),
        )?;
        let _lock = catalog.lock(&found.owner).await?;
        let mut module = catalog
            .module(&found.owner, &found.name)
            .await?
            .map(|(_, module)| module)
            .ok_or_else(|| Status::not_found("the module was deleted"))?;
        if let Some(visibility) = value.visibility {
            module.visibility = Visibility::from_proto(visibility)
                .ok_or_else(|| Status::invalid("the visibility is not valid"))?;
        }
        if let Some(state) = value.state {
            module.state = State::from_proto(state)
                .ok_or_else(|| Status::invalid("the state is not valid"))?;
        }
        if let Some(description) = &value.description {
            check_description(description)?;
            module.description = description.clone();
        }
        if let Some(url) = &value.url {
            check_url(url)?;
            module.url = url.clone();
        }
        if let Some(name) = &value.default_label_name {
            names::valid_label(name)?;
            let label = catalog
                .label(&module.owner, &module.name, name)
                .await?
                .ok_or_else(|| {
                    Status::not_found(format!(
                        "the label {name} does not exist in {}/{}",
                        module.owner, module.name
                    ))
                })?;
            if label.archived.is_some() {
                return Err(Status::precondition(format!(
                    "the label {name} is archived and cannot be the default"
                )));
            }
            module.default_label = name.clone();
        }
        module.updated = model::now();
        catalog.save_module(&module).await?;
        updated.push(module.to_proto());
    }
    Ok(proto::UpdateModulesResponse { modules: updated })
}

pub async fn delete(
    ctx: &Ctx,
    request: proto::DeleteModulesRequest,
) -> Reply<proto::DeleteModulesResponse> {
    if request.module_refs.is_empty() {
        return Err(Status::invalid("at least one module reference is required"));
    }
    let catalog = Catalog::new(ctx);
    let mut targets = Vec::with_capacity(request.module_refs.len());
    for reference in &request.module_refs {
        let reference = names::module_ref(Some(reference))?;
        let module = catalog.module_of(&reference).await?;
        catalog::require(
            ctx,
            View::Delete,
            &catalog::module_path(&module.owner, &module.name),
        )?;
        targets.push(module);
    }
    for module in &targets {
        if let Some(dependent) = catalog.dependent_of(module).await? {
            return Err(Status::precondition(format!(
                "the module {}/{} cannot be deleted: {dependent} depends on its commits",
                module.owner, module.name
            )));
        }
    }
    for module in targets {
        let _lock = catalog.lock(&module.owner).await?;
        catalog.delete_module(&module).await?;
    }
    Ok(proto::DeleteModulesResponse {})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptions_and_urls_are_bounded() {
        assert!(check_description(&"x".repeat(350)).is_ok());
        assert!(check_description(&"x".repeat(351)).is_err());
        assert!(check_url("").is_ok());
        assert!(check_url("https://example.org/acme").is_ok());
        assert!(check_url("not a url").is_err());
        assert!(check_url(&format!("https://example.org/{}", "a".repeat(250))).is_err());
    }
}
