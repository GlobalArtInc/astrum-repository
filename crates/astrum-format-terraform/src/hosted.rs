use std::collections::{BTreeMap, BTreeSet, HashSet};

use astrum_blob::{ContentStream, Written};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, UploadForm, View};
use astrum_repo::config::WritePolicy;
use axum::http::StatusCode;
use futures_util::TryStreamExt;
use serde_json::{Value, json};

use crate::paths::{self, FileKind, Module, Provider};
use crate::pgp::{self, PublicKey, SigningKey};
use crate::sums::Sums;
use crate::{
    DEFAULT_PROTOCOLS, Document, Download, ID, KIND_MODULE, KIND_PROVIDER, MODULE_ARCHIVE,
    Terraform, archive, versions,
};

const MAX_DOCUMENT: u64 = 4 * 1024 * 1024;
const MAX_LAYOUT: u64 = 512 * 1024 * 1024;
const LISTING: u64 = 100_000;

fn attributes(asset: &Asset) -> &Value {
    &asset.attributes[ID]
}

fn text_attribute<'a>(asset: &'a Asset, name: &str) -> Option<&'a str> {
    attributes(asset)[name].as_str()
}

fn generated(asset: &Asset) -> bool {
    attributes(asset)["generated"].as_bool().unwrap_or(false)
}

fn file_name(asset: &Asset) -> &str {
    asset.path.rsplit('/').next().unwrap_or_default()
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

pub async fn read(ctx: &Ctx, asset: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as u64 > limit {
        return Err(ContentError::BadRequest(format!(
            "{} is too large to read",
            asset.path
        )));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::not_found(asset.path.clone()));
    };
    collect(content.stream, limit).await
}

pub async fn collect(mut stream: ContentStream, limit: u64) -> Result<Vec<u8>, ContentError> {
    let mut body = Vec::new();
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if (body.len() + chunk.len()) as u64 > limit {
            return Err(ContentError::BadRequest("the file is too large".to_owned()));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_written(ctx: &Ctx, written: &Written, limit: u64) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let Some(content) = store
        .read_range(written.id, 0, written.size.min(limit))
        .await?
    else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob that was just written is missing",
        )));
    };
    collect(content.stream, limit).await
}

pub async fn under(ctx: &Ctx, prefix: &str, kinds: &[&str]) -> Result<Vec<Asset>, ContentError> {
    Ok(store::under(ctx.db(), ctx.name(), prefix, LISTING)
        .await?
        .into_iter()
        .filter(|asset| kinds.is_empty() || kinds.contains(&asset.kind.as_str()))
        .collect())
}

pub async fn in_directory(
    ctx: &Ctx,
    directory: &str,
    kinds: &[&str],
) -> Result<Vec<Asset>, ContentError> {
    let prefix = format!("{directory}/");
    Ok(under(ctx, &prefix, kinds)
        .await?
        .into_iter()
        .filter(|asset| !asset.path[prefix.len()..].contains('/'))
        .collect())
}

fn version_of(asset: &Asset, base: &str) -> Option<String> {
    if let Some(version) = text_attribute(asset, "version").filter(|v| paths::valid_version(v)) {
        return Some(version.to_owned());
    }
    let rest = asset.path.strip_prefix(base)?.trim_start_matches('/');
    let segment = rest.split('/').next()?;
    paths::valid_version(segment).then(|| segment.to_owned())
}

pub fn authorize(ctx: &Ctx, path: &str, exists: bool) -> Result<(), ContentError> {
    let view = if exists && write_policy(ctx) == WritePolicy::Allow {
        View::Edit
    } else {
        View::Add
    };
    ctx.require(view, path)
}

fn component(
    kind: &str,
    namespace: &str,
    name: &str,
    provider: &str,
    version: &str,
) -> ComponentKey {
    let component_name = match kind {
        KIND_MODULE => format!("{name}/{provider}"),
        _ => name.to_owned(),
    };
    ComponentKey {
        namespace: namespace.to_owned(),
        name: component_name,
        version: version.to_owned(),
        kind: kind.to_owned(),
        attributes: json!({ ID: {
            "kind": kind,
            "namespace": namespace,
            "name": name,
            "provider": provider,
        }}),
    }
}

pub fn module_component(module: &Module) -> ComponentKey {
    component(
        KIND_MODULE,
        &module.namespace,
        &module.name,
        &module.system,
        &module.version,
    )
}

pub fn provider_component(provider: &Provider) -> ComponentKey {
    component(
        KIND_PROVIDER,
        &provider.namespace,
        &provider.kind,
        &provider.kind,
        &provider.version,
    )
}

pub fn module_attributes(module: &Module, subdir: &str) -> Value {
    json!({ ID: {
        "kind": KIND_MODULE,
        "namespace": module.namespace,
        "name": module.name,
        "provider": module.system,
        "version": module.version,
        "subdir": subdir,
    }})
}

fn provider_attributes(provider: &Provider, extra: Value) -> Value {
    let mut values = json!({
        "kind": KIND_PROVIDER,
        "namespace": provider.namespace,
        "name": provider.kind,
        "provider": provider.kind,
        "version": provider.version,
    });
    if let (Some(values), Value::Object(extra)) = (values.as_object_mut(), extra) {
        values.extend(extra);
    }
    json!({ ID: values })
}

pub async fn module_versions(
    ctx: &Ctx,
    module: &Module,
) -> Result<Option<Vec<String>>, ContentError> {
    let base = module.base();
    let mut found: Vec<String> = under(ctx, &format!("{base}/"), &[MODULE_ARCHIVE])
        .await?
        .iter()
        .filter_map(|asset| version_of(asset, &base))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    versions::sort(&mut found);
    Ok(Some(found))
}

pub fn module_location(module: &Module, asset: &Asset) -> String {
    let location = paths::relative(&module.download_path(), &asset.path);
    match text_attribute(asset, "subdir").filter(|subdir| !subdir.is_empty()) {
        Some(subdir) => format!("{location}//{subdir}"),
        None => location,
    }
}

pub async fn module_archive(ctx: &Ctx, module: &Module) -> Result<Option<Asset>, ContentError> {
    Ok(in_directory(ctx, &module.directory(), &[MODULE_ARCHIVE])
        .await?
        .into_iter()
        .next())
}

pub async fn module_download(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
) -> Result<Download, ContentError> {
    match module_archive(ctx, module).await? {
        Some(asset) => {
            let tokens = terraform.tokens(ctx).await?;
            Ok(Download::Location(tokens.sign(
                ctx.name(),
                &module_location(module, &asset),
                &asset.path,
            )))
        }
        None => Ok(Download::Missing(
            StatusCode::NOT_FOUND,
            format!(
                "Module {} version {} not found",
                module.address(),
                module.version
            ),
        )),
    }
}

async fn head(ctx: &Ctx, written: &Written) -> Result<Vec<u8>, ContentError> {
    read_written(ctx, written, 8).await
}

fn discard(ctx: &Ctx, written: &Written) {
    store::discard(ctx.content.clone(), &written.store, written.id.uuid());
}

pub async fn put_module(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
    file: Option<&str>,
    body: ContentStream,
) -> Result<Asset, ContentError> {
    let written = store::write_blob(ctx, body).await?;
    match adopt_module(terraform, ctx, module, file, &written).await {
        Ok(asset) => Ok(asset),
        Err(error) => {
            discard(ctx, &written);
            Err(error)
        }
    }
}

async fn adopt_module(
    terraform: &Terraform,
    ctx: &Ctx,
    module: &Module,
    file: Option<&str>,
    written: &Written,
) -> Result<Asset, ContentError> {
    let sniffed = paths::sniff(&head(ctx, written).await?);
    let extension = file.and_then(paths::archive_extension).or(sniffed);
    let extension = match (extension, sniffed) {
        (Some(extension), Some(sniffed)) if extension == sniffed => extension,
        _ => {
            return Err(ContentError::BadRequest(
                "A module package must be a .zip or .tar.gz archive of the module directory"
                    .to_owned(),
            ));
        }
    };
    let path = module.archive_path(extension);
    let lock = terraform.lock(&format!("{}{}", ctx.name(), module.directory()));
    let _held = lock.lock().await;
    let existing = store::find(ctx.db(), ctx.name(), &path).await?;
    authorize(ctx, &path, existing.is_some())?;
    for other in in_directory(ctx, &module.directory(), &[MODULE_ARCHIVE]).await? {
        if other.path != path {
            if write_policy(ctx) != WritePolicy::Allow {
                return Err(ContentError::BadRequest(format!(
                    "Module {} version {} already exists as {}",
                    module.address(),
                    module.version,
                    other.path
                )));
            }
            store::delete(ctx, &other.path).await?;
        }
    }
    let data = read_written(ctx, written, MAX_LAYOUT)
        .await
        .unwrap_or_default();
    let extension_owned = extension.to_owned();
    let subdir = tokio::task::spawn_blocking(move || {
        archive::layout(&data, &extension_owned)
            .and_then(|layout| layout.single_top_directory())
            .unwrap_or_default()
    })
    .await
    .map_err(ContentError::internal)?;
    let put = PutAsset::hosted(ctx, path)
        .component(module_component(module))
        .kind(MODULE_ARCHIVE)
        .content_type(match extension {
            "zip" => "application/zip",
            _ => "application/gzip",
        })
        .attributes(module_attributes(module, &subdir));
    store::adopt(ctx, put, written).await
}

pub fn protocols(value: &str) -> Result<Vec<String>, ContentError> {
    let protocols: Vec<String> = value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|protocol| !protocol.is_empty())
        .map(|protocol| match protocol.contains('.') {
            true => protocol.to_owned(),
            false => format!("{protocol}.0"),
        })
        .collect();
    for protocol in &protocols {
        let valid = protocol.split_once('.').is_some_and(|(major, minor)| {
            !major.is_empty()
                && !minor.is_empty()
                && major.bytes().all(|c| c.is_ascii_digit())
                && minor.bytes().all(|c| c.is_ascii_digit())
        });
        if !valid {
            return Err(ContentError::BadRequest(format!(
                "Invalid plugin protocol version: {protocol}"
            )));
        }
    }
    Ok(protocols)
}

fn manifest_protocols(manifest: &[u8]) -> Result<Vec<String>, ContentError> {
    let document: Value = serde_json::from_slice(manifest).map_err(|error| {
        ContentError::BadRequest(format!("The release manifest is not JSON: {error}"))
    })?;
    let listed: Vec<String> = document["metadata"]["protocol_versions"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|value| match value {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    if listed.is_empty() {
        return Err(ContentError::BadRequest(
            "The release manifest names no metadata.protocol_versions".to_owned(),
        ));
    }
    protocols(&listed.join(","))
}

fn manifest(protocols: &[String]) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "version": 1,
        "metadata": { "protocol_versions": protocols },
    }))
    .unwrap_or_default()
}

enum Prepared {
    Archive {
        name: String,
        os: String,
        arch: String,
        written: Written,
    },
    Document {
        name: String,
        kind: FileKind,
        bytes: Vec<u8>,
        extra: Value,
    },
}

impl Prepared {
    fn name(&self) -> &str {
        match self {
            Prepared::Archive { name, .. } | Prepared::Document { name, .. } => name,
        }
    }

    fn discard(&self, ctx: &Ctx) {
        if let Prepared::Archive { written, .. } = self {
            discard(ctx, written);
        }
    }
}

#[derive(Default)]
struct Release {
    packages: BTreeMap<String, String>,
    sums: Option<Vec<u8>>,
    signature: Option<Vec<u8>>,
    key: Option<PublicKey>,
}

fn kind_of(provider: &Provider, file: &str) -> Result<FileKind, ContentError> {
    provider
        .kind_of(file)
        .or_else(|| paths::is_key_upload(file).then_some(FileKind::Key))
        .ok_or_else(|| {
            ContentError::BadRequest(format!(
                "'{file}' is not a release file of {} {}; expected {}<os>_<arch>.zip, {}, {}, {} or a public key (.asc, .gpg)",
                provider.address(),
                provider.version,
                provider.prefix(),
                provider.shasums_file(),
                provider.signature_file(),
                provider.manifest_file()
            ))
        })
}

async fn prepare(
    ctx: &Ctx,
    provider: &Provider,
    file: &str,
    kind: FileKind,
    body: ContentStream,
) -> Result<Prepared, ContentError> {
    let name = provider.stored_name(&kind, file);
    match kind {
        FileKind::Archive { os, arch } => {
            let written = store::write_blob(ctx, body).await?;
            let sniffed = head(ctx, &written).await.map(|head| paths::sniff(&head));
            if !matches!(sniffed, Ok(Some("zip"))) {
                discard(ctx, &written);
                return Err(ContentError::BadRequest(format!(
                    "A provider package must be a zip archive: {file}"
                )));
            }
            if write_policy(ctx) == WritePolicy::AllowOnce
                && store::find(ctx.db(), ctx.name(), &provider.file_path(&name))
                    .await?
                    .is_some()
            {
                discard(ctx, &written);
                return Err(ContentError::Conflict(format!(
                    "{}{} cannot be updated as asset already exists and redeploy is not allowed",
                    ctx.name(),
                    provider.file_path(&name)
                )));
            }
            Ok(Prepared::Archive {
                name,
                os,
                arch,
                written,
            })
        }
        FileKind::Shasums => {
            let bytes = collect(body, MAX_DOCUMENT).await?;
            Sums::parse(&bytes).map_err(ContentError::BadRequest)?;
            Ok(Prepared::Document {
                name,
                kind,
                bytes,
                extra: json!({"generated": false}),
            })
        }
        FileKind::Signature => {
            let bytes = collect(body, MAX_DOCUMENT).await?;
            let bytes = pgp::binary_signature(&bytes).map_err(ContentError::BadRequest)?;
            Ok(Prepared::Document {
                name,
                kind,
                bytes,
                extra: json!({"generated": false}),
            })
        }
        FileKind::Key => {
            let bytes = collect(body, MAX_DOCUMENT).await?;
            let key = pgp::read_public_key(&bytes).map_err(ContentError::BadRequest)?;
            Ok(Prepared::Document {
                name,
                kind,
                bytes: key.armored.into_bytes(),
                extra: json!({"keyId": key.key_id}),
            })
        }
        FileKind::Manifest => {
            let bytes = collect(body, MAX_DOCUMENT).await?;
            let protocols = manifest_protocols(&bytes)?;
            Ok(Prepared::Document {
                name,
                kind,
                bytes,
                extra: json!({"protocols": protocols, "generated": false}),
            })
        }
    }
}

fn uploaded(prepared: &[Prepared]) -> Result<Release, ContentError> {
    let mut release = Release::default();
    for file in prepared {
        match file {
            Prepared::Archive { name, written, .. } => {
                release
                    .packages
                    .insert(name.clone(), written.digests.sha256.clone());
            }
            Prepared::Document {
                kind, bytes, extra, ..
            } => match kind {
                FileKind::Shasums => release.sums = Some(bytes.clone()),
                FileKind::Signature => release.signature = Some(bytes.clone()),
                FileKind::Key => {
                    release.key = Some(PublicKey {
                        key_id: extra["keyId"].as_str().unwrap_or_default().to_owned(),
                        armored: String::from_utf8_lossy(bytes).into_owned(),
                    })
                }
                _ => {}
            },
        }
    }
    Ok(release)
}

async fn file(ctx: &Ctx, provider: &Provider, name: &str) -> Result<Option<Asset>, ContentError> {
    store::find(ctx.db(), ctx.name(), &provider.file_path(name)).await
}

async fn stored(ctx: &Ctx, provider: &Provider) -> Result<Release, ContentError> {
    let mut release = Release::default();
    for archive in in_directory(ctx, &provider.directory(), &["PROVIDER_ARCHIVE"]).await? {
        release
            .packages
            .insert(file_name(&archive).to_owned(), archive.sha256.clone());
    }
    if let Some(sums) = file(ctx, provider, &provider.shasums_file())
        .await?
        .filter(|asset| !generated(asset))
    {
        release.sums = Some(read(ctx, &sums, MAX_DOCUMENT).await?);
    }
    if let Some(signature) = file(ctx, provider, &provider.signature_file())
        .await?
        .filter(|asset| !generated(asset))
    {
        release.signature = Some(read(ctx, &signature, MAX_DOCUMENT).await?);
    }
    if let Some(key) = file(ctx, provider, paths::KEY_FILE).await? {
        release.key = pgp::read_public_key(&read(ctx, &key, MAX_DOCUMENT).await?).ok();
    }
    Ok(release)
}

fn check(
    provider: &Provider,
    uploaded: &Release,
    stored: &Release,
    repository_key: Option<&PublicKey>,
) -> Result<(), ContentError> {
    let bad = |message: String| Err(ContentError::BadRequest(message));
    let sums_name = provider.shasums_file();
    let document = uploaded.sums.as_ref().or(stored.sums.as_ref());
    let sums = document
        .map(|document| Sums::parse(document))
        .transpose()
        .map_err(ContentError::BadRequest)?;
    if let Some(sums) = &sums {
        if uploaded.sums.is_some() {
            for (name, sha256) in &stored.packages {
                if !uploaded.packages.contains_key(name)
                    && sums.digest(name).is_some_and(|listed| listed != sha256)
                {
                    return bad(format!("{sums_name} does not match the stored {name}"));
                }
            }
        }
        for (name, sha256) in &uploaded.packages {
            match sums.digest(name) {
                None => {
                    return bad(format!(
                        "{sums_name} does not list {name}; upload the checksums of the release with its packages"
                    ));
                }
                Some(listed) if listed != sha256 => {
                    return bad(format!("{name} does not match its checksum in {sums_name}"));
                }
                _ => {}
            }
        }
    }
    let signature = uploaded.signature.as_ref().or(stored.signature.as_ref());
    let (Some(document), Some(signature)) = (document, signature) else {
        return Ok(());
    };
    if let Some(key) = &uploaded.key {
        if !pgp::verify(document, signature, &key.armored) {
            return bad(format!("Key {} did not sign {sums_name}", key.key_id));
        }
    } else if uploaded.signature.is_some() {
        let keys: Vec<&PublicKey> = stored.key.iter().chain(repository_key).collect();
        if !keys.is_empty()
            && !keys
                .iter()
                .any(|key| pgp::verify(document, signature, &key.armored))
        {
            return bad(format!(
                "{} is not a signature of {sums_name} by a known key",
                provider.signature_file()
            ));
        }
    }
    Ok(())
}

pub fn signing_key(ctx: &Ctx) -> Option<SigningKey> {
    let signing = ctx.repository.attributes.terraform_signing.as_ref()?;
    let open = |sealed: &Option<String>| {
        sealed
            .as_deref()
            .and_then(|sealed| ctx.content.secrets().open(sealed).ok())
    };
    let keypair = open(&signing.keypair)?;
    let passphrase = open(&signing.passphrase);
    match pgp::read_signing_key(&keypair, passphrase.as_deref()) {
        Ok(key) => Some(key),
        Err(error) => {
            tracing::warn!(repository = ctx.name(), %error, "the signing key of the repository cannot be used");
            None
        }
    }
}

async fn store_bytes(
    ctx: &Ctx,
    provider: &Provider,
    name: &str,
    kind: &FileKind,
    bytes: Vec<u8>,
    extra: Value,
) -> Result<Asset, ContentError> {
    let mut put = PutAsset::hosted(ctx, provider.file_path(name))
        .component(provider_component(provider))
        .kind(kind.asset_kind())
        .content_type(kind.content_type())
        .attributes(provider_attributes(provider, extra));
    put.check_content_type = false;
    if put.write_policy != Some(WritePolicy::Deny) {
        put.write_policy = Some(WritePolicy::Allow);
    }
    store::put(ctx, put, astrum_blob::content_of(bytes)).await
}

pub async fn put_provider_files(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
    files: Vec<(String, ContentStream)>,
    protocols: Option<Vec<String>>,
) -> Result<Vec<Asset>, ContentError> {
    let mut targets = HashSet::new();
    let mut kinds = Vec::new();
    let names: Vec<String> = files.iter().map(|(file, _)| file.clone()).collect();
    for file in &names {
        let kind = kind_of(provider, file)?;
        let target = provider.stored_name(&kind, file);
        if !targets.insert(target.clone()) {
            return Err(ContentError::BadRequest(format!(
                "The upload holds more than one {target}"
            )));
        }
        let path = provider.file_path(&target);
        let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
        authorize(ctx, &path, exists)?;
        kinds.push(kind);
    }
    let lock = terraform.lock(&format!("{}{}", ctx.name(), provider.directory()));
    let _held = lock.lock().await;
    let mut prepared = Vec::new();
    for ((file, body), kind) in files.into_iter().zip(kinds) {
        match prepare(ctx, provider, &file, kind, body).await {
            Ok(done) => prepared.push(done),
            Err(error) => {
                prepared.iter().for_each(|done| done.discard(ctx));
                return Err(error);
            }
        }
    }
    let repository_key = signing_key(ctx).map(|key| key.public);
    let checked = async {
        let uploaded = uploaded(&prepared)?;
        let stored = stored(ctx, provider).await?;
        check(provider, &uploaded, &stored, repository_key.as_ref())
    }
    .await;
    if let Err(error) = checked {
        prepared.iter().for_each(|done| done.discard(ctx));
        return Err(error);
    }
    let mut saved = Vec::new();
    let mut manifest_uploaded = false;
    for done in prepared {
        let name = done.name().to_owned();
        let asset = match done {
            Prepared::Archive {
                os, arch, written, ..
            } => {
                let mut put = PutAsset::hosted(ctx, provider.file_path(&name))
                    .component(provider_component(provider))
                    .kind("PROVIDER_ARCHIVE")
                    .content_type("application/zip")
                    .attributes(provider_attributes(
                        provider,
                        json!({"os": os, "arch": arch}),
                    ));
                put.check_content_type = false;
                store::adopt(ctx, put, &written).await?
            }
            Prepared::Document {
                kind, bytes, extra, ..
            } => {
                manifest_uploaded |= kind == FileKind::Manifest;
                store_bytes(ctx, provider, &name, &kind, bytes, extra).await?
            }
        };
        saved.push(asset);
    }
    if let Some(protocols) = protocols.filter(|protocols| !protocols.is_empty())
        && !manifest_uploaded
        && file(ctx, provider, &provider.manifest_file())
            .await?
            .is_none_or(|asset| generated(&asset))
    {
        store_bytes(
            ctx,
            provider,
            &provider.manifest_file(),
            &FileKind::Manifest,
            manifest(&protocols),
            json!({"protocols": protocols, "generated": true}),
        )
        .await?;
    }
    refresh_locked(ctx, provider).await?;
    Ok(saved)
}

pub async fn refresh_checksums(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
) -> Result<(), ContentError> {
    let lock = terraform.lock(&format!("{}{}", ctx.name(), provider.directory()));
    let _held = lock.lock().await;
    refresh_locked(ctx, provider).await
}

async fn refresh_locked(ctx: &Ctx, provider: &Provider) -> Result<(), ContentError> {
    let archives = in_directory(ctx, &provider.directory(), &["PROVIDER_ARCHIVE"]).await?;
    let sums = file(ctx, provider, &provider.shasums_file()).await?;
    let signature = file(ctx, provider, &provider.signature_file()).await?;
    if archives.is_empty() {
        for asset in [sums, signature].into_iter().flatten() {
            if generated(&asset) {
                store::delete(ctx, &asset.path).await?;
            }
        }
        return Ok(());
    }
    let mut changed = false;
    let document = match &sums {
        Some(asset) if !generated(asset) => read(ctx, asset, MAX_DOCUMENT).await?,
        _ => {
            let document = Sums::of(
                archives
                    .iter()
                    .map(|archive| (file_name(archive).to_owned(), archive.sha256.clone())),
            )
            .render();
            let same = match &sums {
                Some(asset) => read(ctx, asset, MAX_DOCUMENT).await? == document,
                None => false,
            };
            if !same {
                store_bytes(
                    ctx,
                    provider,
                    &provider.shasums_file(),
                    &FileKind::Shasums,
                    document.clone(),
                    json!({"generated": true}),
                )
                .await?;
                changed = true;
            }
            document
        }
    };
    if signature.as_ref().is_some_and(|asset| !generated(asset)) {
        return Ok(());
    }
    let Some(key) = signing_key(ctx) else {
        if let Some(asset) = signature {
            store::delete(ctx, &asset.path).await?;
        }
        return Ok(());
    };
    let signed_by = signature
        .as_ref()
        .and_then(|asset| text_attribute(asset, "keyId"))
        .map(str::to_owned);
    if signature.is_none() || changed || signed_by.as_deref() != Some(key.public.key_id.as_str()) {
        let signed = key
            .sign(&document)
            .map_err(|error| ContentError::internal(anyhow::anyhow!(error)))?;
        store_bytes(
            ctx,
            provider,
            &provider.signature_file(),
            &FileKind::Signature,
            signed,
            json!({"generated": true, "keyId": key.public.key_id}),
        )
        .await?;
    }
    Ok(())
}

pub async fn provider_versions(ctx: &Ctx, provider: &Provider) -> Result<Document, ContentError> {
    let base = provider.base();
    let mut platforms: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    let mut protocols: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for asset in under(
        ctx,
        &format!("{base}/"),
        &["PROVIDER_ARCHIVE", "PROVIDER_MANIFEST"],
    )
    .await?
    {
        let Some(version) = version_of(&asset, &base) else {
            continue;
        };
        if asset.kind == "PROVIDER_ARCHIVE" {
            if let (Some(os), Some(arch)) =
                (text_attribute(&asset, "os"), text_attribute(&asset, "arch"))
            {
                platforms
                    .entry(version)
                    .or_default()
                    .insert((os.to_owned(), arch.to_owned()));
            }
        } else {
            protocols.insert(version, protocols_of(&asset));
        }
    }
    if platforms.is_empty() {
        return Ok(Document::missing(format!(
            "Provider {} not found",
            provider.address()
        )));
    }
    let mut ordered: Vec<String> = platforms.keys().cloned().collect();
    versions::sort(&mut ordered);
    let entries: Vec<Value> = ordered
        .iter()
        .map(|version| {
            json!({
                "version": version,
                "protocols": protocols.get(version).cloned().unwrap_or_else(default_protocols),
                "platforms": platforms[version].iter().map(|(os, arch)| json!({"os": os, "arch": arch})).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(Document::Found(json!({ "versions": entries })))
}

fn default_protocols() -> Vec<String> {
    DEFAULT_PROTOCOLS.iter().map(|p| (*p).to_owned()).collect()
}

fn protocols_of(asset: &Asset) -> Vec<String> {
    attributes(asset)["protocols"]
        .as_array()
        .filter(|list| !list.is_empty())
        .map(|list| {
            list.iter()
                .filter_map(|value| value.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_else(default_protocols)
}

pub async fn provider_package(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
    os: &str,
    arch: &str,
) -> Result<Document, ContentError> {
    let missing = |message: String| Ok(Document::missing(message));
    let archive_name = provider.archive_file(os, arch);
    if file(ctx, provider, &archive_name).await?.is_none() {
        return missing(format!(
            "{} {} has no package for {os}_{arch}",
            provider.address(),
            provider.version
        ));
    }
    let Some(sums_asset) = file(ctx, provider, &provider.shasums_file()).await? else {
        return missing(format!(
            "No checksums for {} {}",
            provider.address(),
            provider.version
        ));
    };
    let sums = Sums::parse(&read(ctx, &sums_asset, MAX_DOCUMENT).await?)
        .map_err(ContentError::BadRequest)?;
    let Some(shasum) = sums.digest(&archive_name).map(str::to_owned) else {
        return missing(format!(
            "{} does not list {archive_name}",
            provider.shasums_file()
        ));
    };
    let Some(signature) = file(ctx, provider, &provider.signature_file()).await? else {
        return missing(format!(
            "{} {} is not signed: upload {} with its public key, or give the repository a signing key",
            provider.address(),
            provider.version,
            provider.signature_file()
        ));
    };
    let mut keys: Vec<PublicKey> = Vec::new();
    if !generated(&signature)
        && let Some(uploaded) = file(ctx, provider, paths::KEY_FILE).await?
        && let Ok(key) = pgp::read_public_key(&read(ctx, &uploaded, MAX_DOCUMENT).await?)
    {
        keys.push(key);
    }
    if let Some(key) = signing_key(ctx)
        && !keys.iter().any(|known| known.key_id == key.public.key_id)
    {
        keys.push(key.public);
    }
    if keys.is_empty() {
        return missing(format!(
            "No public key verifies the signature of {} {}; upload the key that signed {}",
            provider.address(),
            provider.version,
            provider.shasums_file()
        ));
    }
    let protocols = match file(ctx, provider, &provider.manifest_file()).await? {
        Some(manifest) => protocols_of(&manifest),
        None => default_protocols(),
    };
    let from = provider.download_path(os, arch);
    let tokens = terraform.tokens(ctx).await?;
    let link = |name: &str| tokens.link(ctx.name(), &from, &provider.file_path(name));
    Ok(Document::Found(json!({
        "protocols": protocols,
        "os": os,
        "arch": arch,
        "filename": archive_name,
        "download_url": link(&archive_name),
        "shasums_url": link(&provider.shasums_file()),
        "shasums_signature_url": link(&provider.signature_file()),
        "shasum": shasum,
        "signing_keys": {
            "gpg_public_keys": keys.iter().map(|key| json!({
                "key_id": key.key_id,
                "ascii_armor": key.armored,
                "trust_signature": "",
                "source": ctx.name(),
                "source_url": "",
            })).collect::<Vec<_>>(),
        },
    })))
}

pub async fn mirror_index(ctx: &Ctx, provider: &Provider) -> Result<Document, ContentError> {
    Ok(match provider_versions(ctx, provider).await? {
        Document::Found(document) => {
            let listed: Vec<String> = document["versions"]
                .as_array()
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| entry["version"].as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            Document::Found(crate::mirror_index_document(&listed))
        }
        missing => missing,
    })
}

pub async fn mirror_version(
    terraform: &Terraform,
    ctx: &Ctx,
    provider: &Provider,
    document: &str,
) -> Result<Document, ContentError> {
    let archives = in_directory(ctx, &provider.directory(), &["PROVIDER_ARCHIVE"]).await?;
    if archives.is_empty() {
        return Ok(Document::missing(format!(
            "Provider {} version {} not found",
            provider.address(),
            provider.version
        )));
    }
    let tokens = terraform.tokens(ctx).await?;
    let mut entries = serde_json::Map::new();
    for archive in archives {
        let (Some(os), Some(arch)) = (
            text_attribute(&archive, "os"),
            text_attribute(&archive, "arch"),
        ) else {
            continue;
        };
        entries.insert(
            format!("{os}_{arch}"),
            json!({
                "url": tokens.link(ctx.name(), document, &archive.path),
                "hashes": [format!("zh:{}", archive.sha256)],
            }),
        );
    }
    Ok(Document::Found(json!({ "archives": entries })))
}

fn field<'a>(form: &'a UploadForm, name: &str) -> Option<&'a str> {
    form.field(&format!("{ID}.{name}")).map(str::trim)
}

fn required<'a>(form: &'a UploadForm, name: &str) -> Result<&'a str, ContentError> {
    let value = field(form, name).ok_or_else(|| {
        ContentError::BadRequest(format!("Missing required component field '{name}'"))
    })?;
    if !paths::valid_name(value) {
        return Err(ContentError::BadRequest(format!(
            "Invalid {name} '{value}': use letters, digits, '-' and '_'"
        )));
    }
    Ok(value)
}

fn required_version(form: &UploadForm) -> Result<String, ContentError> {
    let value = field(form, "version").ok_or_else(|| {
        ContentError::BadRequest("Missing required component field 'version'".to_owned())
    })?;
    let value = match value.strip_prefix('v') {
        Some(rest) if paths::valid_version(rest) => rest,
        _ => value,
    };
    if !paths::valid_version(value) {
        return Err(ContentError::BadRequest(format!(
            "Version '{value}' is not a semantic version"
        )));
    }
    Ok(value.to_owned())
}

fn base_name(file: &str) -> String {
    file.replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned()
}

pub async fn upload(
    terraform: &Terraform,
    ctx: &Ctx,
    form: UploadForm,
) -> Result<(), ContentError> {
    let module = match field(&form, "uploadType").map(str::to_ascii_lowercase) {
        Some(kind) if kind == KIND_MODULE => true,
        Some(kind) if kind == KIND_PROVIDER => false,
        Some(_) => {
            return Err(ContentError::BadRequest(
                "Upload type must be module or provider".to_owned(),
            ));
        }
        None => {
            let module = field(&form, "system").is_some() || field(&form, "name").is_some();
            let provider = field(&form, "type").is_some();
            if module == provider {
                return Err(ContentError::BadRequest(
                    "Give name and system for a module, or type for a provider".to_owned(),
                ));
            }
            module
        }
    };
    if form.files.is_empty() {
        return Err(ContentError::BadRequest(
            "No assets found in upload".to_owned(),
        ));
    }
    let version = required_version(&form)?;
    if module {
        if form.files.len() != 1 {
            return Err(ContentError::BadRequest(
                "A module upload takes exactly one package".to_owned(),
            ));
        }
        let coordinates = Module::new(
            required(&form, "namespace")?,
            required(&form, "name")?,
            required(&form, "system")?,
            &version,
        )
        .ok_or_else(|| ContentError::BadRequest("Invalid module coordinates".to_owned()))?;
        let file = form.files.values().next().expect("one file is there");
        let name = file.filename.as_deref().map(base_name);
        put_module(
            terraform,
            ctx,
            &coordinates,
            name.as_deref(),
            UploadForm::stream(file).await?,
        )
        .await?;
        return Ok(());
    }
    let provider = Provider::new(
        required(&form, "namespace")?,
        required(&form, "type")?,
        &version,
    )
    .ok_or_else(|| ContentError::BadRequest("Invalid provider coordinates".to_owned()))?;
    let protocols = field(&form, "protocols").map(protocols).transpose()?;
    let mut files = Vec::new();
    let mut names = HashSet::new();
    for file in form.files.values() {
        let name = file
            .filename
            .as_deref()
            .map(base_name)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                ContentError::BadRequest(
                    "Provider files are recognized by their names; a file has no name".to_owned(),
                )
            })?;
        if !names.insert(name.clone()) {
            return Err(ContentError::BadRequest(format!(
                "The upload holds {name} more than once"
            )));
        }
        files.push((name, UploadForm::stream(file).await?));
    }
    put_provider_files(terraform, ctx, &provider, files, protocols).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocols_are_normalised_and_checked() {
        assert_eq!(protocols("5, 6.0").unwrap(), ["5.0", "6.0"]);
        assert!(protocols("six").is_err());
        assert_eq!(
            manifest_protocols(br#"{"version":1,"metadata":{"protocol_versions":["6.0"]}}"#)
                .unwrap(),
            ["6.0"]
        );
        assert!(manifest_protocols(b"{}").is_err());
    }

    #[test]
    fn releases_must_fit_together() {
        let provider = Provider::new("acme", "demo", "1.0.0").unwrap();
        let name = provider.archive_file("linux", "amd64");
        let digest = "a".repeat(64);
        let sums = Sums::of([(name.clone(), digest.clone())]).render();
        let uploaded = Release {
            packages: BTreeMap::from([(name.clone(), digest.clone())]),
            sums: Some(sums.clone()),
            ..Default::default()
        };
        assert!(check(&provider, &uploaded, &Release::default(), None).is_ok());
        let wrong = Release {
            packages: BTreeMap::from([(name.clone(), "b".repeat(64))]),
            sums: Some(sums.clone()),
            ..Default::default()
        };
        assert!(check(&provider, &wrong, &Release::default(), None).is_err());
        let unlisted = Release {
            packages: BTreeMap::from([("other.zip".to_owned(), digest.clone())]),
            ..Default::default()
        };
        let stored = Release {
            sums: Some(sums),
            ..Default::default()
        };
        assert!(check(&provider, &unlisted, &stored, None).is_err());
    }
}
