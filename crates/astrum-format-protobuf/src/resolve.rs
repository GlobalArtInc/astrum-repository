use astrum_content::View;

use crate::catalog::{self, COMMIT, Catalog, LABEL, MODULE};
use crate::connect::{Reply, Status};
use crate::model::{Commit, Label, Module};
use crate::names::{self, Ref};

pub const DEFAULT_PAGE: u32 = 10;
pub const MAX_PAGE: u32 = 250;

pub enum Found {
    Module(Module),
    Label(Module, Label),
    Commit(Module, Commit),
}

pub fn page<T>(items: Vec<T>, size: u32, token: &str) -> Reply<(Vec<T>, String)> {
    let size = match size {
        0 => DEFAULT_PAGE,
        size => size.min(MAX_PAGE),
    } as usize;
    let offset = match token {
        "" => 0,
        token => token
            .parse::<usize>()
            .map_err(|_| Status::invalid("the page token is not valid"))?,
    };
    let total = items.len();
    let kept: Vec<T> = items.into_iter().skip(offset).take(size).collect();
    let next = match offset + size < total {
        true => (offset + size).to_string(),
        false => String::new(),
    };
    Ok((kept, next))
}

impl Catalog<'_> {
    pub async fn named_module(&self, owner: &str, module: &str) -> Reply<Module> {
        catalog::require(self.ctx, View::Read, &catalog::module_path(owner, module))?;
        self.module(owner, module)
            .await?
            .map(|(_, module)| module)
            .ok_or_else(|| Status::not_found(format!("the module {owner}/{module} does not exist")))
    }

    pub async fn module_of(&self, reference: &Ref) -> Reply<Module> {
        match reference {
            Ref::Module { owner, module } => self.named_module(owner, module).await,
            Ref::Id(id) => {
                let module = self
                    .module_by_id(id)
                    .await?
                    .map(|(_, module)| module)
                    .ok_or_else(|| Status::not_found(format!("the module {id} does not exist")))?;
                catalog::require(
                    self.ctx,
                    View::Read,
                    &catalog::module_path(&module.owner, &module.name),
                )?;
                Ok(module)
            }
            _ => Err(Status::invalid("a module reference is required")),
        }
    }

    pub async fn resolve(&self, reference: &Ref) -> Reply<Found> {
        match reference {
            Ref::Module { owner, module } => {
                Ok(Found::Module(self.named_module(owner, module).await?))
            }
            Ref::Label {
                owner,
                module,
                label,
            } => {
                let module = self.named_module(owner, module).await?;
                let found = self
                    .label(owner, &module.name, label)
                    .await?
                    .ok_or_else(|| {
                        Status::not_found(format!(
                            "the label {label} does not exist in {owner}/{}",
                            module.name
                        ))
                    })?;
                Ok(Found::Label(module, found))
            }
            Ref::Named {
                owner,
                module,
                reference,
            } => {
                let module = self.named_module(owner, module).await?;
                if let Some(id) = names::commit_id(reference)
                    && let Some(commit) = self.commit(owner, &module.name, &id).await?
                {
                    return Ok(Found::Commit(module, commit));
                }
                match self.label(owner, &module.name, reference).await? {
                    Some(label) => Ok(Found::Label(module, label)),
                    None => Err(Status::not_found(format!(
                        "{reference} is neither a commit nor a label of {owner}/{}",
                        module.name
                    ))),
                }
            }
            Ref::Id(id) => {
                let missing = || Status::not_found(format!("nothing has the id {id}"));
                let found = self.by_id_any(id).await?.ok_or_else(missing)?;
                match found {
                    (MODULE, module) => {
                        let module: Module = catalog::meta(&module)?;
                        catalog::require(
                            self.ctx,
                            View::Read,
                            &catalog::module_path(&module.owner, &module.name),
                        )?;
                        Ok(Found::Module(module))
                    }
                    (LABEL, label) => {
                        let label: Label = catalog::meta(&label)?;
                        let module = self.named_module(&label.owner, &label.module).await?;
                        Ok(Found::Label(module, label))
                    }
                    (COMMIT, commit) => {
                        let commit: Commit = catalog::meta(&commit)?;
                        let module = self.named_module(&commit.owner, &commit.module).await?;
                        Ok(Found::Commit(module, commit))
                    }
                    _ => Err(missing()),
                }
            }
        }
    }

    async fn label_commit(&self, module: &Module, label: &Label) -> Reply<Commit> {
        self.commit(&module.owner, &module.name, &label.commit)
            .await?
            .ok_or_else(|| {
                Status::not_found(format!("the commit of the label {} is gone", label.name))
            })
    }

    pub async fn commit_of(&self, found: Found) -> Reply<(Module, Commit)> {
        match found {
            Found::Commit(module, commit) => Ok((module, commit)),
            Found::Label(module, label) => {
                let commit = self.label_commit(&module, &label).await?;
                Ok((module, commit))
            }
            Found::Module(module) => {
                let label = self
                    .label(&module.owner, &module.name, &module.default_label)
                    .await?
                    .ok_or_else(|| {
                        Status::not_found(format!(
                            "the module {}/{} has no commits on its default label {}",
                            module.owner, module.name, module.default_label
                        ))
                    })?;
                let commit = self.label_commit(&module, &label).await?;
                Ok((module, commit))
            }
        }
    }

    pub async fn commit_for(&self, reference: &Ref) -> Reply<(Module, Commit)> {
        let found = self.resolve(reference).await?;
        self.commit_of(found).await
    }

    pub async fn dependency(&self, id: &str) -> Reply<Commit> {
        self.commit_by_id(id)
            .await?
            .ok_or_else(|| Status::not_found(format!("the commit {id} does not exist")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_start_at_the_token_and_name_the_next_one() {
        let items: Vec<u32> = (0..25).collect();
        let (first, next) = page(items.clone(), 0, "").unwrap();
        assert_eq!(first, (0..10).collect::<Vec<_>>());
        assert_eq!(next, "10");
        let (second, next) = page(items.clone(), 10, &next).unwrap();
        assert_eq!(second, (10..20).collect::<Vec<_>>());
        assert_eq!(next, "20");
        let (last, next) = page(items.clone(), 10, &next).unwrap();
        assert_eq!(last, (20..25).collect::<Vec<_>>());
        assert_eq!(next, "");
        let (all, next) = page(items, 1000, "").unwrap();
        assert_eq!(all.len(), 25);
        assert_eq!(next, "");
    }

    #[test]
    fn a_page_token_must_be_a_number() {
        assert!(page(vec![1], 10, "abc").is_err());
        let (empty, next) = page(vec![1, 2], 10, "50").unwrap();
        assert!(empty.is_empty());
        assert_eq!(next, "");
    }
}
