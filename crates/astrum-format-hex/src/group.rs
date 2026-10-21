use astrum_content::store;
use astrum_content::{Asset, ContentError, Ctx, group};
use astrum_repo::RepositoryType;
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::proto::{Names, Package, Versions};
use crate::{Hex, ID, PUBLIC_KEY, REGISTRY, REGISTRY_TYPE, Resource, files, keys, proto, registry};

const GENERATION: &str = "1";
const MAX_KEY: u64 = 64 * 1024;

fn merge_key(signer: &str, resource: &Resource, answers: &[(Ctx, Asset)]) -> String {
    let mut digest = Sha256::new();
    digest.update(format!("{GENERATION}\n{signer}\n{}", resource.path()).as_bytes());
    for (member, found) in answers {
        digest.update(format!("\n{}={}", member.name(), found.sha256).as_bytes());
    }
    hex::encode(digest.finalize())
}

fn stored_key(found: &Asset) -> Option<&str> {
    files::attribute(found, "mergeKey").and_then(Value::as_str)
}

enum Payload {
    Names(Names),
    Versions(Versions),
    Package(Package),
}

fn decode(resource: &Resource, payload: &[u8]) -> Result<Payload, prost::DecodeError> {
    Ok(match resource {
        Resource::Names => Payload::Names(Names::decode(payload)?),
        Resource::Versions => Payload::Versions(Versions::decode(payload)?),
        Resource::Package(_) => Payload::Package(Package::decode(payload)?),
    })
}

fn merged(repository: &str, resource: &Resource, payloads: Vec<Payload>) -> Vec<u8> {
    match resource {
        Resource::Names => registry::merge_names(
            repository,
            payloads
                .into_iter()
                .filter_map(|payload| match payload {
                    Payload::Names(names) => Some(names),
                    _ => None,
                })
                .collect(),
        )
        .encode_to_vec(),
        Resource::Versions => registry::merge_versions(
            repository,
            payloads
                .into_iter()
                .filter_map(|payload| match payload {
                    Payload::Versions(versions) => Some(versions),
                    _ => None,
                })
                .collect(),
        )
        .encode_to_vec(),
        Resource::Package(name) => registry::merge_package(
            repository,
            name,
            payloads
                .into_iter()
                .filter_map(|payload| match payload {
                    Payload::Package(package) => Some(package),
                    _ => None,
                })
                .collect(),
        )
        .encode_to_vec(),
    }
}

impl Hex {
    async fn member_payload(&self, member: &Ctx, found: &Asset) -> Result<Vec<u8>, ContentError> {
        let bytes = files::bytes(member, found, proto::MAX_RESOURCE).await?;
        match member.kind() {
            RepositoryType::Proxy => {
                let key = crate::remote::get(member, PUBLIC_KEY)
                    .await?
                    .ok_or_else(|| {
                        ContentError::BadGateway(format!(
                            "{} serves no public key to check its registry with",
                            member.name()
                        ))
                    })?;
                let pem = String::from_utf8(files::bytes(member, &key, MAX_KEY).await?).map_err(
                    |_| ContentError::BadGateway("the public key is not PEM".to_owned()),
                )?;
                let public = keys::public_key_of(&pem).ok_or_else(|| {
                    ContentError::BadGateway("the public key is not an RSA public key".to_owned())
                })?;
                tokio::task::spawn_blocking(move || keys::verify(&public, &bytes))
                    .await
                    .map_err(ContentError::internal)?
                    .ok_or_else(|| {
                        ContentError::BadGateway(format!(
                            "the signature of {} from {} does not match its public key",
                            found.path,
                            member.name()
                        ))
                    })
            }
            _ => tokio::task::spawn_blocking(move || proto::open(&bytes))
                .await
                .map_err(ContentError::internal)?
                .map(|signed| signed.payload)
                .map_err(|error| ContentError::internal(std::io::Error::other(error.to_string()))),
        }
    }

    pub(crate) async fn merged_resource(
        &self,
        ctx: &Ctx,
        resource: &Resource,
    ) -> Result<Option<Asset>, ContentError> {
        let path = resource.path();
        let mut answers = Vec::new();
        for member in group::members(ctx).await? {
            let member = ctx.member(member);
            match Box::pin(self.resource(&member, resource)).await {
                Ok(Some(found)) => answers.push((member, found)),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no registry resource here");
                }
            }
        }
        if answers.is_empty() && matches!(resource, Resource::Package(_)) {
            return Ok(None);
        }
        let key = self.keys.get(ctx).await?;
        let wanted = merge_key(&key.fingerprint, resource, &answers);
        let cached =
            |found: Option<Asset>| found.filter(|found| stored_key(found) == Some(wanted.as_str()));
        if let Some(found) = cached(store::find(ctx.db(), ctx.name(), &path).await?) {
            return Ok(Some(found));
        }
        let lock = files::lock(ctx).await?;
        if let Some(found) = cached(store::find(&lock, ctx.name(), &path).await?) {
            lock.commit().await?;
            return Ok(Some(found));
        }
        let mut collected = Vec::with_capacity(answers.len());
        for (member, found) in &answers {
            match self.member_payload(member, found).await {
                Ok(payload) => collected.push((member.name().to_owned(), payload)),
                Err(error) => {
                    tracing::warn!(group = ctx.name(), member = member.name(), %error, "a member's registry resource is skipped");
                }
            }
        }
        if collected.is_empty() && matches!(resource, Resource::Package(_)) {
            lock.commit().await?;
            return Ok(None);
        }
        let repository = ctx.name().to_owned();
        let merging = resource.clone();
        let signer = key.clone();
        let (signed, members) = tokio::task::spawn_blocking(move || {
            let mut payloads = Vec::with_capacity(collected.len());
            let mut members = Vec::with_capacity(collected.len());
            for (member, payload) in collected {
                match decode(&merging, &payload) {
                    Ok(decoded) => {
                        payloads.push(decoded);
                        members.push(member);
                    }
                    Err(error) => {
                        tracing::warn!(group = %repository, member = %member, %error, "a member's registry resource cannot be read");
                    }
                }
            }
            (signer.sign(merged(&repository, &merging, payloads)), members)
        })
        .await
        .map_err(ContentError::internal)?;
        let attributes = json!({ ID: {
            "mergeKey": wanted,
            "members": members,
        }});
        let stored = store::put(
            ctx,
            files::metadata_put(ctx, &path, REGISTRY, REGISTRY_TYPE, attributes),
            astrum_blob::content_of(signed),
        )
        .await?;
        lock.commit().await?;
        Ok(Some(stored))
    }
}
