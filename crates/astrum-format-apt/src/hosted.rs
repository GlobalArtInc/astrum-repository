use astrum_blob::Written;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, View};
use astrum_db::entity::asset;
use astrum_repo::config::WritePolicy;
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Statement, TransactionTrait,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::deb::{self, Control};
use crate::metadata::{self, Package};
use crate::signing::{self, Keys};
use crate::{DEB, DEB_TYPE, ID, METADATA, distribution};

const MAX_CONTROL_PREFIX: u64 = 32 * 1024 * 1024;
const GENERATION: &str = "1\n";

fn release_path(distribution: &str) -> String {
    format!("/dists/{distribution}/Release")
}

async fn prefix(ctx: &Ctx, written: &Written) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let length = written.size.min(MAX_CONTROL_PREFIX);
    let Some(content) = store.read_range(written.id, 0, length).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    let mut body = Vec::with_capacity(length as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn refuse(ctx: &Ctx, written: &Written, error: ContentError) -> ContentError {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
    error
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub async fn adopt(ctx: &Ctx, keys: &Keys, written: Written) -> Result<Asset, ContentError> {
    let data = prefix(ctx, &written).await?;
    let control = match tokio::task::spawn_blocking(move || deb::read_control(&data))
        .await
        .map_err(ContentError::internal)?
    {
        Ok(control) => control,
        Err(error) => {
            return Err(refuse(
                ctx,
                &written,
                ContentError::BadRequest(error.to_string()),
            ));
        }
    };
    let path = deb::pool_path(&control);
    let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
    let allowed = if exists && write_policy(ctx) == WritePolicy::Allow {
        ctx.require(View::Edit, &path)
    } else {
        ctx.require(View::Add, &path)
    };
    if let Err(error) = allowed {
        return Err(refuse(ctx, &written, error));
    }
    let fields: Vec<Value> = control
        .fields
        .iter()
        .map(|(key, value)| json!([key, value]))
        .collect();
    let put = PutAsset::hosted(ctx, path)
        .component(ComponentKey {
            namespace: control.architecture().to_owned(),
            name: control.package().to_owned(),
            version: control.version().to_owned(),
            kind: String::new(),
            attributes: json!({}),
        })
        .kind(DEB)
        .content_type(DEB_TYPE)
        .attributes(json!({ ID: { "control": fields } }));
    let stored = store::adopt(ctx, put, &written).await?;
    rebuild(ctx, keys, true).await?;
    Ok(stored)
}

fn control_of(found: &Asset) -> Option<Control> {
    let fields = found.attributes.get(ID)?.get("control")?.as_array()?;
    Some(Control {
        fields: fields
            .iter()
            .filter_map(|pair| {
                let pair = pair.as_array()?;
                Some((
                    pair.first()?.as_str()?.to_owned(),
                    pair.get(1)?.as_str()?.to_owned(),
                ))
            })
            .collect(),
    })
}

async fn debs(db: &impl ConnectionTrait, repository: &str) -> Result<Vec<Asset>, ContentError> {
    Ok(asset::Entity::find()
        .filter(asset::Column::Repository.eq(repository))
        .filter(asset::Column::Kind.eq(DEB))
        .order_by_asc(asset::Column::Path)
        .all(db)
        .await?)
}

fn fingerprint(distribution: &str, debs: &[Asset], signer: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(GENERATION.as_bytes());
    hasher.update(distribution.as_bytes());
    hasher.update(b"\n");
    hasher.update(signer.as_bytes());
    for found in debs {
        hasher.update(b"\n");
        hasher.update(found.path.as_bytes());
        hasher.update(b" ");
        hasher.update(found.sha256.as_bytes());
    }
    hex::encode(hasher.finalize())
}

fn stored_fingerprint(found: &Asset) -> Option<&str> {
    found.attributes.get(ID)?.get("fingerprint")?.as_str()
}

pub async fn ensure_fresh(ctx: &Ctx, keys: &Keys) -> Result<(), ContentError> {
    let distribution = distribution(ctx);
    let key = keys.get(ctx)?;
    let wanted = fingerprint(
        &distribution,
        &debs(ctx.db(), ctx.name()).await?,
        signing::signer(key.as_deref()),
    );
    let current = store::find(ctx.db(), ctx.name(), &release_path(&distribution)).await?;
    if current
        .as_ref()
        .and_then(stored_fingerprint)
        .is_some_and(|stored| stored == wanted)
    {
        return Ok(());
    }
    rebuild(ctx, keys, false).await
}

pub async fn rebuild(ctx: &Ctx, keys: &Keys, force: bool) -> Result<(), ContentError> {
    let distribution = distribution(ctx);
    let key = keys.get(ctx)?;
    let txn = ctx.db().begin().await?;
    txn.execute(Statement::from_sql_and_values(
        txn.get_database_backend(),
        "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
        [format!("apt:{}", ctx.name()).into()],
    ))
    .await?;
    let found = debs(&txn, ctx.name()).await?;
    let wanted = fingerprint(&distribution, &found, signing::signer(key.as_deref()));
    if !force
        && store::find(&txn, ctx.name(), &release_path(&distribution))
            .await?
            .as_ref()
            .and_then(stored_fingerprint)
            .is_some_and(|stored| stored == wanted)
    {
        txn.commit().await?;
        return Ok(());
    }
    let packages: Vec<Package> = found
        .iter()
        .filter_map(|found| {
            Some(Package {
                control: control_of(found)?,
                filename: found.path.clone(),
                size: found.size,
                md5: found.md5.clone(),
                sha1: found.sha1.clone(),
                sha256: found.sha256.clone(),
            })
        })
        .collect();
    let (architectures, files) = metadata::architecture_files(&distribution, &packages);
    let release = metadata::release(&distribution, &architectures, &files, Utc::now());
    let mut generated: Vec<(String, &'static str, Vec<u8>, Value)> = files
        .into_iter()
        .map(|file| (file.path, file.content_type, file.bytes, json!({})))
        .collect();
    let attributes = json!({ ID: { "fingerprint": wanted } });
    if let Some(key) = &key {
        let inline = key
            .clearsign(&release)
            .map_err(|error| ContentError::internal(std::io::Error::other(error)))?;
        let detached = key
            .sign(release.as_bytes())
            .map_err(|error| ContentError::internal(std::io::Error::other(error)))?;
        generated.push((
            format!("/dists/{distribution}/InRelease"),
            "text/plain",
            inline.into_bytes(),
            json!({}),
        ));
        generated.push((
            format!("/dists/{distribution}/Release.gpg"),
            "application/pgp-signature",
            detached,
            json!({}),
        ));
    }
    generated.push((
        release_path(&distribution),
        "text/plain",
        release.into_bytes(),
        attributes,
    ));
    let keep: Vec<String> = generated.iter().map(|(path, ..)| path.clone()).collect();
    for (path, content_type, bytes, attributes) in generated {
        let mut put = PutAsset::hosted(ctx, path)
            .kind(METADATA)
            .content_type(content_type)
            .attributes(attributes);
        put.check_content_type = false;
        put.write_policy = Some(WritePolicy::Allow);
        store::put(ctx, put, astrum_blob::content_of(bytes)).await?;
    }
    let stale = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(METADATA))
        .filter(Expr::col(asset::Column::Path).is_not_in(keep))
        .all(&txn)
        .await?;
    txn.commit().await?;
    for old in stale {
        store::delete_asset(&ctx.content, old).await?;
    }
    Ok(())
}
