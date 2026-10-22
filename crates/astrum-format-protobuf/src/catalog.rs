use std::collections::{BTreeMap, HashSet};

use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, Ctx, View};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use futures_util::TryStreamExt;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter,
    QueryOrder, Statement, TransactionTrait,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::connect::{Reply, Status};
use crate::model::{self, Commit, Label, LabelFile, Module, Owner};
use crate::names;

pub const OWNER: &str = "owner";
pub const MODULE: &str = "module";
pub const COMMIT: &str = "commit";
pub const LABEL: &str = "label";
pub const FILE: &str = "file";
pub const JSON: &str = "application/json";
pub const MAX_HISTORY: usize = 1000;
const MAX_METADATA: u64 = 16 * 1024 * 1024;

pub fn owner_path(owner: &str) -> String {
    format!("/{owner}/owner.json")
}

pub fn module_path(owner: &str, module: &str) -> String {
    format!("/{owner}/{module}/module.json")
}

pub fn module_dir(owner: &str, module: &str) -> String {
    format!("/{owner}/{module}/")
}

pub fn commits_dir(owner: &str, module: &str) -> String {
    format!("/{owner}/{module}/commits/")
}

pub fn commit_path(owner: &str, module: &str, id: &str) -> String {
    format!("/{owner}/{module}/commits/{id}/commit.json")
}

pub fn file_path(owner: &str, module: &str, id: &str, path: &str) -> String {
    format!("/{owner}/{module}/commits/{id}/{path}")
}

pub fn labels_dir(owner: &str, module: &str) -> String {
    format!("/{owner}/{module}/labels/")
}

pub fn label_path(owner: &str, module: &str, name: &str) -> String {
    format!(
        "/{owner}/{module}/labels/{}.json",
        names::encode_segment(name)
    )
}

pub fn require(ctx: &Ctx, view: View, path: &str) -> Reply<()> {
    ctx.require(view, path).map_err(Status::from)
}

pub fn writable(ctx: &Ctx) -> Reply<()> {
    let policy = ctx
        .repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy);
    match policy {
        Some(WritePolicy::Deny) => Err(Status::precondition(format!(
            "the repository {} is read-only",
            ctx.name()
        ))),
        _ => Ok(()),
    }
}

pub fn meta<T: DeserializeOwned>(found: &Asset) -> Reply<T> {
    found
        .attributes
        .get(model::KEY)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| Status::internal(format!("{} has unreadable metadata", found.path)))
}

fn escaped(prefix: &str) -> String {
    prefix
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub struct Catalog<'a> {
    pub ctx: &'a Ctx,
}

impl<'a> Catalog<'a> {
    pub fn new(ctx: &'a Ctx) -> Self {
        Self { ctx }
    }

    async fn at(&self, path: &str) -> Reply<Option<Asset>> {
        Ok(store::find(self.ctx.db(), self.ctx.name(), path).await?)
    }

    async fn under(&self, kind: &str, prefix: &str) -> Reply<Vec<Asset>> {
        Ok(asset::Entity::find()
            .filter(asset::Column::Repository.eq(self.ctx.name()))
            .filter(asset::Column::Kind.eq(kind))
            .filter(Expr::col(asset::Column::Path).like(format!("{}%", escaped(prefix))))
            .order_by_asc(asset::Column::Path)
            .all(self.ctx.db())
            .await?)
    }

    async fn by_id(&self, kinds: &[&str], id: &str) -> Reply<Option<Asset>> {
        let kinds: Vec<String> = kinds.iter().map(|kind| format!("'{kind}'")).collect();
        let sql = format!(
            "SELECT * FROM asset WHERE format = 'protobuf' AND repository = $1 \
             AND kind IN ({}) AND attributes #>> '{{protobuf,id}}' = $2 LIMIT 1",
            kinds.join(", ")
        );
        Ok(asset::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                &sql,
                [self.ctx.name().into(), id.into()],
            ))
            .one(self.ctx.db())
            .await?)
    }

    pub async fn by_id_any(&self, id: &str) -> Reply<Option<(&'static str, Asset)>> {
        let kinds = [COMMIT, LABEL, MODULE];
        let found = self.by_id(&kinds, id).await?;
        Ok(found.and_then(|found| {
            let kind = kinds.into_iter().find(|kind| *kind == found.kind)?;
            Some((kind, found))
        }))
    }

    pub async fn owner(&self, name: &str) -> Reply<Option<(Asset, Owner)>> {
        match self.at(&owner_path(name)).await? {
            Some(found) => {
                let owner = meta(&found)?;
                Ok(Some((found, owner)))
            }
            None => Ok(None),
        }
    }

    pub async fn owner_by_id(&self, id: &str) -> Reply<Option<(Asset, Owner)>> {
        match self.by_id(&[OWNER], id).await? {
            Some(found) => {
                let owner = meta(&found)?;
                Ok(Some((found, owner)))
            }
            None => Ok(None),
        }
    }

    pub async fn module(&self, owner: &str, name: &str) -> Reply<Option<(Asset, Module)>> {
        match self.at(&module_path(owner, name)).await? {
            Some(found) => {
                let module = meta(&found)?;
                Ok(Some((found, module)))
            }
            None => Ok(None),
        }
    }

    pub async fn module_by_id(&self, id: &str) -> Reply<Option<(Asset, Module)>> {
        match self.by_id(&[MODULE], id).await? {
            Some(found) => {
                let module = meta(&found)?;
                Ok(Some((found, module)))
            }
            None => Ok(None),
        }
    }

    pub async fn modules(&self, owner: Option<&str>) -> Reply<Vec<Module>> {
        let prefix = owner.map(|owner| format!("/{owner}/")).unwrap_or_default();
        self.under(MODULE, &prefix)
            .await?
            .iter()
            .map(meta)
            .collect()
    }

    pub async fn commit(&self, owner: &str, module: &str, id: &str) -> Reply<Option<Commit>> {
        match self.at(&commit_path(owner, module, id)).await? {
            Some(found) => Ok(Some(meta(&found)?)),
            None => Ok(None),
        }
    }

    pub async fn commit_by_id(&self, id: &str) -> Reply<Option<Commit>> {
        match self.by_id(&[COMMIT], id).await? {
            Some(found) => Ok(Some(meta(&found)?)),
            None => Ok(None),
        }
    }

    pub async fn commits(&self, owner: &str, module: &str) -> Reply<Vec<Commit>> {
        self.under(COMMIT, &commits_dir(owner, module))
            .await?
            .iter()
            .map(meta)
            .collect()
    }

    pub async fn label(&self, owner: &str, module: &str, name: &str) -> Reply<Option<Label>> {
        match self.at(&label_path(owner, module, name)).await? {
            Some(found) => Ok(Some(meta(&found)?)),
            None => Ok(None),
        }
    }

    pub async fn label_by_id(&self, id: &str) -> Reply<Option<Label>> {
        match self.by_id(&[LABEL], id).await? {
            Some(found) => Ok(Some(meta(&found)?)),
            None => Ok(None),
        }
    }

    pub async fn labels(&self, owner: &str, module: &str) -> Reply<Vec<Label>> {
        self.under(LABEL, &labels_dir(owner, module))
            .await?
            .iter()
            .map(meta)
            .collect()
    }

    pub async fn history(&self, label: &Label) -> Reply<LabelFile> {
        let path = label_path(&label.owner, &label.module, &label.name);
        let found = self
            .at(&path)
            .await?
            .ok_or_else(|| Status::not_found(format!("the label {} is gone", label.name)))?;
        let body = self.read(&found).await?;
        serde_json::from_slice(&body).map_err(Status::internal)
    }

    pub async fn read(&self, found: &Asset) -> Reply<Vec<u8>> {
        let size = found.size.max(0) as u64;
        if size > MAX_METADATA {
            return Err(Status::internal(format!("{} is too large", found.path)));
        }
        let store = self
            .ctx
            .content
            .blob_stores()
            .get(&found.blob_store)
            .await
            .map_err(|error| Status::from(astrum_content::ContentError::from(error)))?;
        let Some(content) = store
            .read(found.blob_id.into())
            .await
            .map_err(|error| Status::from(astrum_content::ContentError::from(error)))?
        else {
            return Err(Status::internal(format!(
                "the blob of {} is missing",
                found.path
            )));
        };
        let mut body = Vec::with_capacity(size as usize);
        let mut stream = content.stream;
        while let Some(chunk) = stream.try_next().await.map_err(Status::internal)? {
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    pub async fn file(&self, commit: &Commit, path: &str) -> Reply<Vec<u8>> {
        let location = file_path(&commit.owner, &commit.module, &commit.id, path);
        let found = self
            .at(&location)
            .await?
            .ok_or_else(|| Status::internal(format!("{location} is missing")))?;
        let content = self.read(&found).await?;
        self.ctx.content.note_download(found.id);
        self.ctx
            .content
            .audit_download(&found, &self.ctx.uploader())
            .await;
        Ok(content)
    }

    async fn write(
        &self,
        path: &str,
        kind: &str,
        content_type: &str,
        attributes: Value,
        component: Option<ComponentKey>,
        body: Vec<u8>,
    ) -> Reply<Asset> {
        let mut put = PutAsset::hosted(self.ctx, path)
            .kind(kind)
            .content_type(content_type)
            .attributes(attributes);
        put.check_content_type = false;
        put.write_policy = Some(WritePolicy::Allow);
        if let Some(component) = component {
            put = put.component(component);
        }
        Ok(store::put(self.ctx, put, astrum_blob::content_of(body)).await?)
    }

    async fn write_json<T: Serialize>(
        &self,
        path: &str,
        kind: &str,
        meta: &T,
        body: &impl Serialize,
        component: Option<ComponentKey>,
    ) -> Reply<Asset> {
        let body = serde_json::to_vec(body).map_err(Status::internal)?;
        self.write(path, kind, JSON, model::attributes(meta), component, body)
            .await
    }

    pub async fn save_owner(&self, owner: &Owner) -> Reply<()> {
        self.write_json(&owner_path(&owner.name), OWNER, owner, owner, None)
            .await?;
        Ok(())
    }

    pub async fn save_module(&self, module: &Module) -> Reply<()> {
        self.write_json(
            &module_path(&module.owner, &module.name),
            MODULE,
            module,
            module,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn save_label(&self, file: &LabelFile) -> Reply<()> {
        let label = &file.label;
        self.write_json(
            &label_path(&label.owner, &label.module, &label.name),
            LABEL,
            label,
            file,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn save_commit(
        &self,
        commit: &Commit,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Reply<()> {
        let component = ComponentKey {
            namespace: commit.owner.clone(),
            name: commit.module.clone(),
            version: commit.id.clone(),
            kind: COMMIT.to_owned(),
            attributes: serde_json::json!({ model::KEY: { "digest": commit.digest } }),
        };
        for (path, content) in files {
            self.write(
                &file_path(&commit.owner, &commit.module, &commit.id, path),
                FILE,
                "text/plain",
                Value::Object(Default::default()),
                Some(component.clone()),
                content.clone(),
            )
            .await?;
        }
        self.write_json(
            &commit_path(&commit.owner, &commit.module, &commit.id),
            COMMIT,
            commit,
            commit,
            Some(component),
        )
        .await?;
        Ok(())
    }

    pub async fn dependent_of(&self, module: &Module) -> Reply<Option<String>> {
        let own: HashSet<String> = self
            .commits(&module.owner, &module.name)
            .await?
            .into_iter()
            .map(|commit| commit.id)
            .collect();
        if own.is_empty() {
            return Ok(None);
        }
        let prefix = module_dir(&module.owner, &module.name);
        let rows = asset::Entity::find()
            .filter(asset::Column::Repository.eq(self.ctx.name()))
            .filter(asset::Column::Kind.eq(COMMIT))
            .all(self.ctx.db())
            .await?;
        for row in rows {
            if row.path.starts_with(&prefix) {
                continue;
            }
            let commit: Commit = meta(&row)?;
            if commit
                .deps
                .iter()
                .any(|dependency| own.contains(&dependency.id))
            {
                return Ok(Some(format!("{}/{}", commit.owner, commit.module)));
            }
        }
        Ok(None)
    }

    pub async fn delete_module(&self, module: &Module) -> Reply<()> {
        let prefix = module_dir(&module.owner, &module.name);
        let rows = asset::Entity::find()
            .filter(asset::Column::Repository.eq(self.ctx.name()))
            .filter(Expr::col(asset::Column::Path).like(format!("{}%", escaped(&prefix))))
            .all(self.ctx.db())
            .await?;
        let components: HashSet<i64> = rows.iter().filter_map(|row| row.component_id).collect();
        for component in components {
            store::delete_component(&self.ctx.content, component).await?;
        }
        for row in rows {
            if row.component_id.is_none() {
                store::delete_asset(&self.ctx.content, row).await?;
            }
        }
        Ok(())
    }

    pub async fn lock(&self, owner: &str) -> Reply<DatabaseTransaction> {
        let txn = self.ctx.db().begin().await?;
        txn.execute(Statement::from_sql_and_values(
            txn.get_database_backend(),
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            [format!("protobuf:{}:{owner}", self.ctx.name()).into()],
        ))
        .await?;
        Ok(txn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_paths_nest_owner_module_and_commit() {
        assert_eq!(owner_path("acme"), "/acme/owner.json");
        assert_eq!(module_path("acme", "greeter"), "/acme/greeter/module.json");
        assert_eq!(module_dir("acme", "greeter"), "/acme/greeter/");
        assert_eq!(
            commit_path("acme", "greeter", "abc"),
            "/acme/greeter/commits/abc/commit.json"
        );
        assert_eq!(
            file_path("acme", "greeter", "abc", "acme/v1/greeter.proto"),
            "/acme/greeter/commits/abc/acme/v1/greeter.proto"
        );
        assert_eq!(
            label_path("acme", "greeter", "release/v1"),
            "/acme/greeter/labels/release%2Fv1.json"
        );
    }

    #[test]
    fn like_patterns_escape_their_wildcards() {
        assert_eq!(escaped("/a_b/c%d/"), "/a\\_b/c\\%d/");
    }
}
