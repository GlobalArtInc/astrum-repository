use astrum_blob::config::{Settings, StoreConfig};
use astrum_blob::s3::SIGNER_TYPES;
use astrum_blob::{
    S3Bucket, S3Connection, S3Encryption, S3EncryptionKind, S3FailoverBucket, S3Security,
    S3Settings, SoftQuota, StoreKind,
};
use astrum_db::audit::Event;
use astrum_security::SecretBox;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    CREATE, DOMAIN, QuotaInput, READ, UPDATE, check_name, check_quota, ensure_unique, find, insert,
    internal, refuse_type_change, save,
};
use crate::{ApiError, AppState, Caller, Json, Validation};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct S3Request {
    name: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    soft_quota: Option<QuotaInput>,
    bucket_configuration: Option<BucketConfigurationInput>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketConfigurationInput {
    bucket: Option<BucketInput>,
    #[serde(alias = "security")]
    bucket_security: Option<SecurityInput>,
    encryption: Option<EncryptionInput>,
    #[serde(alias = "advancedConnection")]
    advanced_bucket_connection: Option<ConnectionInput>,
    failover_buckets: Option<Vec<FailoverInput>>,
    pre_signed_url_enabled: Option<bool>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BucketInput {
    region: Option<String>,
    name: Option<String>,
    prefix: Option<String>,
    expiration: Option<i64>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SecurityInput {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    role: Option<String>,
    session_token: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptionInput {
    encryption_type: Option<String>,
    encryption_key: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionInput {
    endpoint: Option<String>,
    signer_type: Option<String>,
    force_path_style: Option<bool>,
    max_connection_pool_size: Option<i64>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FailoverInput {
    region: Option<String>,
    bucket_name: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct S3Model {
    name: String,
    soft_quota: Option<SoftQuota>,
    bucket_configuration: BucketConfigurationModel,
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BucketConfigurationModel {
    bucket: BucketModel,
    bucket_security: Option<SecurityModel>,
    encryption: Option<EncryptionModel>,
    advanced_bucket_connection: Option<ConnectionModel>,
    failover_buckets: Vec<FailoverModel>,
    pre_signed_url_enabled: bool,
    active_region: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BucketModel {
    region: String,
    name: String,
    prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SecurityModel {
    access_key_id: Option<String>,
    role: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptionModel {
    encryption_type: &'static str,
    encryption_key: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionModel {
    endpoint: Option<String>,
    signer_type: Option<String>,
    force_path_style: bool,
    max_connection_pool_size: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FailoverModel {
    region: String,
    bucket_name: String,
}

pub(super) async fn create(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<S3Request>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let mut validation = Validation::new();
    let name = check_name(&mut validation, body.name.as_deref());
    check_kind(&mut validation, body.kind.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let settings = settings(
        &mut validation,
        body.bucket_configuration,
        None,
        state.secrets(),
    );
    validation.finish()?;
    ensure_unique(&state, &name).await?;
    check_endpoint(&state, &settings, None).await?;
    let config = StoreConfig {
        name,
        settings: Settings::S3(Box::new(settings)),
        soft_quota,
    };
    connect(&state, &config).await?;
    insert(&state, &config).await?;
    state.blob_stores().forget();
    state.audit(&caller, audit_event("created", &config)).await;
    Ok(StatusCode::CREATED)
}

pub(super) async fn read(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<axum::Json<S3Model>, ApiError> {
    caller.require(READ)?;
    let config = find(&state, &name).await?;
    let Some(settings) = config.s3() else {
        return Err(ApiError::BadRequest(format!(
            "{name} is not an S3 blob store"
        )));
    };
    Ok(axum::Json(model(&config.name, config.soft_quota, settings)))
}

pub(super) async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
    body: Result<Json<S3Request>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(UPDATE)?;
    let Json(body) = body?;
    let current = find(&state, &name).await?;
    refuse_type_change(&current, StoreKind::S3)?;
    let mut validation = Validation::new();
    if let Some(other) = body.name.as_deref().map(str::trim)
        && !other.is_empty()
        && other != current.name
    {
        validation.add("PARAMETER name", "the name of a blob store cannot change");
    }
    check_kind(&mut validation, body.kind.as_deref());
    let soft_quota = check_quota(&mut validation, body.soft_quota);
    let settings = settings(
        &mut validation,
        body.bucket_configuration,
        current.s3(),
        state.secrets(),
    );
    validation.finish()?;

    let old = current.s3().cloned().unwrap_or_default();
    let moved = old.bucket.name != settings.bucket.name
        || old.bucket.key_prefix() != settings.bucket.key_prefix();
    if moved {
        let totals = state
            .blob_stores()
            .metrics(&current.name)
            .totals()
            .await
            .map_err(internal)?;
        if totals.blob_count > 0 {
            return Err(ApiError::invalid(
                "PARAMETER bucket",
                "The bucket and prefix of a blob store that holds blobs cannot change",
            ));
        }
    }
    check_endpoint(&state, &settings, current.s3()).await?;
    let config = StoreConfig {
        name: current.name.clone(),
        settings: Settings::S3(Box::new(settings)),
        soft_quota,
    };
    connect(&state, &config).await?;
    save(&state, &config).await?;
    state.audit(&caller, audit_event("updated", &config)).await;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn test_connection(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<Json<S3Request>, ApiError>,
) -> Result<StatusCode, ApiError> {
    caller.require(CREATE)?;
    let Json(body) = body?;
    let named = body
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned);
    let stored = match &named {
        Some(name) => state
            .blob_stores()
            .config(name)
            .await
            .map_err(internal)?
            .filter(|config| config.kind() == StoreKind::S3),
        None => None,
    };
    if stored.is_some() {
        caller.require(UPDATE)?;
    }
    let mut validation = Validation::new();
    check_kind(&mut validation, body.kind.as_deref());
    let settings = settings(
        &mut validation,
        body.bucket_configuration,
        stored.as_ref().and_then(StoreConfig::s3),
        state.secrets(),
    );
    validation.finish()?;
    check_endpoint(&state, &settings, None).await?;
    let name = named.unwrap_or_else(|| "test-connection".to_owned());
    let outcome = match state.blob_stores().open_s3(&name, &settings).await {
        Ok(store) => {
            match tokio::time::timeout(super::CHECK_TIMEOUT, astrum_blob::BlobStore::check(&store))
                .await
            {
                Ok(Ok(())) => Ok(()),
                Ok(Err(error)) => Err(error.to_string()),
                Err(_) => Err("the bucket did not answer in time".to_owned()),
            }
        }
        Err(error) => Err(error.to_string()),
    };
    match outcome {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(reason) => {
            tracing::warn!(store = %name, %reason, "the S3 connection test failed");
            Err(ApiError::BadRequest(CONNECTION_FAILED.to_owned()))
        }
    }
}

pub(super) const CONNECTION_FAILED: &str =
    "Connection failed, check the logs for more information.";

fn check_kind(validation: &mut Validation, kind: Option<&str>) {
    if let Some(kind) = kind
        && !kind.trim().is_empty()
        && !kind.trim().eq_ignore_ascii_case(StoreKind::S3.as_str())
    {
        validation.add(
            "PARAMETER type",
            "must be S3; the type of a blob store cannot change",
        );
    }
}

fn settings(
    validation: &mut Validation,
    input: Option<BucketConfigurationInput>,
    current: Option<&S3Settings>,
    secrets: &SecretBox,
) -> S3Settings {
    let Some(input) = input else {
        validation.add("PARAMETER bucketConfiguration", "must not be empty");
        return S3Settings::default();
    };
    let present = input.bucket.is_some();
    if !present {
        validation.add("PARAMETER bucket", "must not be empty");
    }
    let bucket = input.bucket.unwrap_or_default();
    let region = text(bucket.region);
    let name = text(bucket.name);
    if present && region.is_empty() {
        validation.add("PARAMETER region", "must not be empty");
    }
    if present && name.is_empty() {
        validation.add("PARAMETER name", "must not be empty");
    } else if present && !valid_bucket(&name) {
        validation.add(
            "bucketConfiguration.bucket.name",
            "must be 3 to 63 lower-case letters, digits, '.' or '-', starting and ending with a letter or digit",
        );
    }
    if bucket.expiration.is_some_and(|days| days < -1) {
        validation.add(
            "bucketConfiguration.bucket.expiration",
            "must be -1 (never) or a number of days",
        );
    }
    let mut settings = S3Settings {
        bucket: S3Bucket {
            region,
            name,
            prefix: text(bucket.prefix).trim_matches('/').to_owned(),
            expiration: bucket.expiration,
        },
        security: security(
            validation,
            input.bucket_security.unwrap_or_default(),
            current.map(|current| &current.security),
            secrets,
        ),
        encryption: encryption(validation, input.encryption.unwrap_or_default()),
        connection: connection(
            validation,
            input.advanced_bucket_connection.unwrap_or_default(),
        ),
        failover_buckets: Vec::new(),
        pre_signed_url_enabled: input.pre_signed_url_enabled.unwrap_or(false),
    };
    for (index, failover) in input
        .failover_buckets
        .unwrap_or_default()
        .into_iter()
        .enumerate()
    {
        let region = text(failover.region);
        let bucket_name = text(failover.bucket_name);
        if region.is_empty() || bucket_name.is_empty() {
            validation.add(
                format!("bucketConfiguration.failoverBuckets[{index}]"),
                "needs a region and a bucket name",
            );
        }
        settings.failover_buckets.push(S3FailoverBucket {
            region,
            bucket_name,
        });
    }
    settings
}

fn security(
    validation: &mut Validation,
    input: SecurityInput,
    current: Option<&S3Security>,
    secrets: &SecretBox,
) -> S3Security {
    let access_key_id = optional(input.access_key_id);
    let role = optional(input.role);
    let kept = |pick: fn(&S3Security) -> &Option<String>| {
        current
            .filter(|current| current.access_key_id == access_key_id)
            .and_then(|current| pick(current).clone())
    };
    let secret_access_key = optional(input.secret_access_key)
        .map(|secret| secrets.seal(&secret))
        .or_else(|| kept(|current| &current.secret_access_key));
    let session_token = optional(input.session_token)
        .map(|token| secrets.seal(&token))
        .or_else(|| kept(|current| &current.session_token));
    match (&access_key_id, &secret_access_key, &session_token) {
        (Some(_), None, _) => validation.add(
            "bucketConfiguration.bucketSecurity.secretAccessKey",
            "must not be empty when an access key id is set",
        ),
        (None, Some(_), _) | (None, _, Some(_)) => validation.add(
            "bucketConfiguration.bucketSecurity.accessKeyId",
            "must not be empty when a secret access key or session token is set",
        ),
        _ => {}
    }
    S3Security {
        access_key_id,
        secret_access_key,
        role,
        session_token,
    }
}

fn encryption(validation: &mut Validation, input: EncryptionInput) -> S3Encryption {
    let kind = text(input.encryption_type);
    let Some(kind) = S3EncryptionKind::parse(&kind) else {
        validation.add(
            "bucketConfiguration.encryption.encryptionType",
            "must be s3ManagedEncryption or kmsManagedEncryption",
        );
        return S3Encryption::default();
    };
    S3Encryption {
        kind,
        key_id: optional(input.encryption_key).filter(|_| kind == S3EncryptionKind::Kms),
    }
}

fn connection(validation: &mut Validation, input: ConnectionInput) -> S3Connection {
    let endpoint = optional(input.endpoint);
    if let Some(endpoint) = &endpoint
        && !(endpoint.starts_with("http://") || endpoint.starts_with("https://"))
    {
        validation.add(
            "bucketConfiguration.advancedBucketConnection.endpoint",
            "must be an http:// or https:// address",
        );
    }
    let signer_type = optional(input.signer_type);
    if let Some(signer) = &signer_type
        && !SIGNER_TYPES.contains(&signer.as_str())
    {
        validation.add(
            "bucketConfiguration.advancedBucketConnection.signerType",
            format!("must be one of {}", SIGNER_TYPES.join(", ")),
        );
    }
    let max_connection_pool_size = match input.max_connection_pool_size {
        Some(size) if size > i64::from(u32::MAX) => {
            validation.add(
                "bucketConfiguration.advancedBucketConnection.maxConnectionPoolSize",
                "is too large",
            );
            None
        }
        Some(size) if size > 0 => Some(size as u32),
        _ => None,
    };
    S3Connection {
        endpoint,
        signer_type,
        force_path_style: input.force_path_style.unwrap_or(false),
        max_connection_pool_size,
    }
}

async fn check_endpoint(
    state: &AppState,
    settings: &S3Settings,
    stored: Option<&S3Settings>,
) -> Result<(), ApiError> {
    let endpoint = |settings: &S3Settings| settings.connection.endpoint.clone();
    let Some(address) = endpoint(settings) else {
        return Ok(());
    };
    if stored.and_then(endpoint).as_deref() == Some(address.as_str()) {
        return Ok(());
    }
    match crate::routes::security::refused_destination(state, &address).await? {
        Some(message) => Err(ApiError::invalid(
            "bucketConfiguration.advancedBucketConnection.endpoint",
            message,
        )),
        None => Ok(()),
    }
}

async fn connect(state: &AppState, config: &StoreConfig) -> Result<(), ApiError> {
    let Some(settings) = config.s3() else {
        return Ok(());
    };
    let store = state
        .blob_stores()
        .open_s3(&config.name, settings)
        .await
        .map_err(|error| {
            ApiError::BadRequest(format!("The S3 settings cannot be used: {error}"))
        })?;
    store
        .ensure_bucket()
        .await
        .map_err(|error| ApiError::BadRequest(format!("The bucket cannot be used: {error}")))
}

fn model(name: &str, soft_quota: Option<SoftQuota>, settings: &S3Settings) -> S3Model {
    let security = &settings.security;
    let connection = &settings.connection;
    S3Model {
        name: name.to_owned(),
        soft_quota,
        bucket_configuration: BucketConfigurationModel {
            active_region: settings.bucket.active_region().to_owned(),
            bucket: BucketModel {
                region: settings.bucket.region.clone(),
                name: settings.bucket.name.clone(),
                prefix: settings.bucket.prefix.clone(),
                expiration: settings.bucket.expiration,
            },
            bucket_security: (security.access_key_id.is_some() || security.role.is_some()).then(
                || SecurityModel {
                    access_key_id: security.access_key_id.clone(),
                    role: security.role.clone(),
                },
            ),
            encryption: settings
                .encryption
                .kind
                .as_api()
                .map(|encryption_type| EncryptionModel {
                    encryption_type,
                    encryption_key: settings.encryption.key_id.clone(),
                }),
            advanced_bucket_connection: (connection != &S3Connection::default()).then(|| {
                ConnectionModel {
                    endpoint: connection.endpoint.clone(),
                    signer_type: connection.signer_type.clone(),
                    force_path_style: connection.force_path_style,
                    max_connection_pool_size: connection.max_connection_pool_size,
                }
            }),
            failover_buckets: settings
                .failover_buckets
                .iter()
                .map(|failover| FailoverModel {
                    region: failover.region.clone(),
                    bucket_name: failover.bucket_name.clone(),
                })
                .collect(),
            pre_signed_url_enabled: settings.pre_signed_url_enabled,
        },
        kind: StoreKind::S3.as_str(),
    }
}

fn audit_event(kind: &str, config: &StoreConfig) -> Event {
    let bucket = config
        .s3()
        .map(|settings| settings.bucket.name.clone())
        .unwrap_or_default();
    Event::new(DOMAIN, kind, &config.name).with(json!({"type": "S3", "bucket": bucket}))
}

fn valid_bucket(name: &str) -> bool {
    let bytes = name.as_bytes();
    (3..=63).contains(&bytes.len())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        && bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
}

fn text(value: Option<String>) -> String {
    value.unwrap_or_default().trim().to_owned()
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_names_follow_the_s3_rules() {
        for name in ["abc", "my.bucket-1", "0-0"] {
            assert!(valid_bucket(name), "{name}");
        }
        for name in [
            "ab",
            "Upper",
            "under_score",
            "-start",
            "end-",
            &"a".repeat(64),
        ] {
            assert!(!valid_bucket(name), "{name}");
        }
    }

    #[test]
    fn a_secret_is_kept_only_for_the_same_access_key() {
        let secrets = SecretBox::from_key([9; 32]);
        let current = S3Security {
            access_key_id: Some("AKID".to_owned()),
            secret_access_key: Some(secrets.seal("old")),
            role: None,
            session_token: None,
        };
        let mut validation = Validation::new();
        let kept = security(
            &mut validation,
            SecurityInput {
                access_key_id: Some("AKID".to_owned()),
                ..Default::default()
            },
            Some(&current),
            &secrets,
        );
        assert!(validation.is_empty());
        assert_eq!(kept.secret_access_key, current.secret_access_key);

        let replaced = security(
            &mut validation,
            SecurityInput {
                access_key_id: Some("AKID".to_owned()),
                secret_access_key: Some("new".to_owned()),
                ..Default::default()
            },
            Some(&current),
            &secrets,
        );
        assert_eq!(
            secrets
                .open(replaced.secret_access_key.as_deref().unwrap())
                .unwrap(),
            "new"
        );

        security(
            &mut validation,
            SecurityInput {
                access_key_id: Some("OTHER".to_owned()),
                ..Default::default()
            },
            Some(&current),
            &secrets,
        );
        assert!(!validation.is_empty(), "another key needs its own secret");
    }
}
