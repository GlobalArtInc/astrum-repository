use std::collections::{BTreeSet, HashMap};

use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::archive::{self, Description, Packing};
use crate::dcf;
use crate::index::{self, Index, Kind};
use crate::paths::{self, PackageFile};
use crate::{ID, METADATA, PACKAGE};

const GENERATION: &str = "r-index-1";

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

fn escape_like(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub fn index_path(directory: &str, file: index::File) -> String {
    format!("{directory}/{}", file.name())
}

async fn packages_in(
    db: &impl ConnectionTrait,
    repository: &str,
    directory: &str,
) -> Result<Vec<Asset>, ContentError> {
    let prefix = escape_like(directory);
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(PACKAGE))
        .filter(Expr::col(asset::Column::Path).like(format!("{prefix}/%")))
        .filter(Expr::col(asset::Column::Path).not_like(format!("{prefix}/%/%")))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

fn fingerprint(kind: Kind, packages: &[Asset]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{GENERATION}\n{kind:?}\n"));
    for found in packages {
        hasher.update(found.path.as_bytes());
        hasher.update(b" ");
        hasher.update(found.sha256.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    found.attributes.get(ID)?.get("fingerprint")?.as_str()
}

fn kind_of(directory: &str) -> Kind {
    paths::hosted_kind(directory).unwrap_or_else(|| Kind::of_directory(directory))
}

pub async fn ensure_fresh(ctx: &Ctx, directory: &str) -> Result<(), ContentError> {
    let packages = packages_in(ctx.db(), ctx.name(), directory).await?;
    let current = store::find(
        ctx.db(),
        ctx.name(),
        &index_path(directory, index::File::Plain),
    )
    .await?;
    let stored = current.as_ref().and_then(stored_fingerprint);
    if packages.is_empty() && current.is_none() {
        return Ok(());
    }
    if stored == Some(fingerprint(kind_of(directory), &packages).as_str()) {
        return Ok(());
    }
    rebuild(ctx, directory, false).await
}

fn description_of(found: &Asset) -> Option<&Map<String, Value>> {
    found.attributes.get(ID)?.as_object()
}

async fn read_description(ctx: &Ctx, found: &Asset) -> Result<Map<String, Value>, ContentError> {
    if let Some(description) = description_of(found) {
        return Ok(description.clone());
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let size = found.size.max(0) as u64;
    let packing = archive::sniff_blob(&store, found.blob_id.into(), size)
        .await?
        .ok_or_else(|| {
            ContentError::BadRequest(format!("{} is not a package archive", found.path))
        })?;
    let described = archive::from_blob(store, found.blob_id.into(), size, packing).await?;
    let fields = fields_map(&described);
    let mut attributes = found.attributes.clone();
    if let Some(object) = attributes.as_object_mut() {
        object.insert(ID.to_owned(), Value::Object(fields.clone()));
    }
    store::set_attributes(ctx.db(), found.id, attributes).await?;
    Ok(fields)
}

fn latest(packages: &[Asset]) -> Vec<&Asset> {
    let mut chosen: HashMap<String, (&Asset, String)> = HashMap::new();
    for found in packages {
        let Some(file) = paths::package_file(&found.path) else {
            continue;
        };
        let better = match chosen.get(&file.name) {
            None => true,
            Some((current, version)) => match dcf::compare_versions(&file.version, version) {
                std::cmp::Ordering::Greater => true,
                std::cmp::Ordering::Equal => found.path > current.path,
                std::cmp::Ordering::Less => false,
            },
        };
        if better {
            chosen.insert(file.name.clone(), (found, file.version));
        }
    }
    let mut kept: Vec<&Asset> = chosen.into_values().map(|(found, _)| found).collect();
    kept.sort_by(|left, right| {
        let name = |found: &Asset| found.path.rsplit('/').next().unwrap_or_default().to_owned();
        name(left).cmp(&name(right))
    });
    kept
}

pub async fn rebuild(ctx: &Ctx, directory: &str, force: bool) -> Result<(), ContentError> {
    let kind = kind_of(directory);
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("r:{}:{directory}", ctx.name()).into()],
    ))
    .await?;
    let packages = packages_in(&txn, ctx.name(), directory).await?;
    let wanted = fingerprint(kind, &packages);
    let current = store::find(&txn, ctx.name(), &index_path(directory, index::File::Plain)).await?;
    if !force
        && current
            .as_ref()
            .and_then(stored_fingerprint)
            .is_some_and(|stored| stored == wanted)
    {
        txn.commit().await?;
        return Ok(());
    }
    if packages.is_empty() {
        for file in [index::File::Plain, index::File::Gz, index::File::Rds] {
            if let Some(found) = store::find(&txn, ctx.name(), &index_path(directory, file)).await?
            {
                store::delete_asset(&ctx.content, found).await?;
            }
        }
        txn.commit().await?;
        return Ok(());
    }
    let mut entries = Vec::new();
    for found in latest(&packages) {
        let description = match read_description(ctx, found).await {
            Ok(description) => description,
            Err(error) => {
                tracing::warn!(repository = ctx.name(), path = found.path, %error, "an R package is left out of PACKAGES");
                continue;
            }
        };
        entries.push(index::hosted_entry(kind, &description, &found.md5));
    }
    let files = Index { kind, entries }.files();
    for file in [index::File::Rds, index::File::Gz, index::File::Plain] {
        let attributes = match file {
            index::File::Plain => json!({ ID: { "fingerprint": wanted } }),
            _ => json!({}),
        };
        let mut put = PutAsset::hosted(ctx, index_path(directory, file))
            .kind(METADATA)
            .content_type(file.content_type())
            .attributes(attributes);
        put.check_content_type = false;
        put.write_policy = Some(WritePolicy::Allow);
        store::put(ctx, put, astrum_blob::content_of(files.get(file).to_vec())).await?;
    }
    txn.commit().await?;
    Ok(())
}

pub async fn rebuild_all(ctx: &Ctx) -> Result<usize, ContentError> {
    let rows: Vec<(String, String)> = asset::Entity::find()
        .select_only()
        .column(asset::Column::Path)
        .column(asset::Column::Kind)
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.is_in([PACKAGE, METADATA]))
        .into_tuple()
        .all(ctx.db())
        .await?;
    let directories: BTreeSet<String> = rows
        .into_iter()
        .filter(|(path, kind)| {
            kind == PACKAGE
                || path
                    .rsplit('/')
                    .next()
                    .and_then(index::File::parse)
                    .is_some()
        })
        .filter_map(|(path, _)| {
            path.rsplit_once('/')
                .map(|(directory, _)| directory.to_owned())
        })
        .collect();
    for directory in &directories {
        rebuild(ctx, directory, true).await?;
    }
    Ok(directories.len())
}

fn fields_map(description: &Description) -> Map<String, Value> {
    let mut fields = Map::new();
    for (tag, value) in &description.record.fields {
        if !fields.contains_key(tag) {
            fields.insert(tag.clone(), Value::String(value.clone()));
        }
    }
    fields
}

pub fn check(
    file: &PackageFile,
    kind: Kind,
    packing: Option<Packing>,
    description: &Description,
) -> Result<(), String> {
    if !kind.extensions().contains(&file.extension) {
        return Err(format!(
            "{} holds {} files, not {}",
            file.directory,
            kind.extensions().join(", "),
            file.extension
        ));
    }
    let expected = Packing::of_extension(file.extension);
    if packing != expected {
        return Err(format!(
            "the file is not a {} archive",
            expected.map(Packing::describe).unwrap_or("package")
        ));
    }
    if let Some(package) = paths::archive_package(&file.directory)
        && package != file.name
    {
        return Err(format!(
            "{} keeps older versions of {package}, not of {}",
            file.directory, file.name
        ));
    }
    let record = &description.record;
    let package = record.get("Package").unwrap_or_default();
    let version = record.get("Version").unwrap_or_default();
    if package != file.name || version != file.version {
        return Err(format!(
            "the DESCRIPTION file names {package} {version}, but the file name says {} {}",
            file.name, file.version
        ));
    }
    if description.folder != file.name {
        return Err(format!(
            "the archive keeps its DESCRIPTION in {}/, not in {}/",
            description.folder, file.name
        ));
    }
    if matches!(kind, Kind::Windows | Kind::Mac) && record.get("Built").is_none() {
        return Err(
            "binary packages carry a Built field in DESCRIPTION; this looks like a source package"
                .to_owned(),
        );
    }
    Ok(())
}

pub async fn adopt(
    ctx: &Ctx,
    written: Written,
    file: PackageFile,
    packing: Option<Packing>,
    description: Description,
) -> Result<Asset, ContentError> {
    let refuse = |error: ContentError| {
        store::discard(ctx.content.clone(), &written.store, written.id.uuid());
        error
    };
    let Some(kind) = paths::hosted_kind(&file.directory) else {
        return Err(refuse(ContentError::BadRequest(paths::PLACES.to_owned())));
    };
    if let Err(message) = check(&file, kind, packing, &description) {
        return Err(refuse(ContentError::BadRequest(message)));
    }
    let path = file.path();
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    let allowed = if exists && write_policy(ctx) == WritePolicy::Allow {
        ctx.require(View::Edit, &path)
    } else {
        ctx.require(View::Add, &path)
    };
    if let Err(error) = allowed {
        return Err(refuse(error));
    }
    let put = PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            namespace: String::new(),
            name: file.name.clone(),
            version: file.version.clone(),
            kind: String::new(),
            attributes: json!({}),
        })
        .kind(PACKAGE)
        .content_type(paths::content_type(file.extension))
        .attributes(json!({ ID: fields_map(&description) }));
    let stored = store::adopt(ctx, put, &written).await?;
    rebuild(ctx, &file.directory, true).await?;
    Ok(stored)
}

pub async fn describe(
    ctx: &Ctx,
    written: &Written,
) -> Result<(Option<Packing>, Description), ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let packing = archive::sniff_blob(&store, written.id, written.size).await?;
    let Some(found) = packing else {
        return Err(ContentError::BadRequest(
            "the file is not an R package archive (.tar.gz, .tgz, .zip, .tar.bz2 or .tar.xz)"
                .to_owned(),
        ));
    };
    let description = archive::from_blob(store, written.id, written.size, found).await?;
    Ok((packing, description))
}
