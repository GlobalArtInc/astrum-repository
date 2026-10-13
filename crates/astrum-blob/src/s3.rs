use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_config::sts::AssumeRoleProvider;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::config::{
    Credentials, Region, RequestChecksumCalculation, ResponseChecksumValidation,
    SharedCredentialsProvider,
};
use aws_sdk_s3::error::{DisplayErrorContext, ProvideErrorMetadata, SdkError};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{
    BucketLocationConstraint, CompletedMultipartUpload, CompletedPart, CreateBucketConfiguration,
    ServerSideEncryption,
};
use aws_smithy_http_client::tls;
use bytes::{Bytes, BytesMut};
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::digest::{Digests, Hasher};
use crate::{
    BlobId, BlobStore, Content, ContentStream, Error, Metrics, StoreKind, StoredBlob, Totals,
    Written, clamp_range,
};

pub const DEFAULT_REGION: &str = "DEFAULT";
pub const FALLBACK_REGION: &str = "us-east-1";
pub const PART_SIZE: usize = 8 * 1024 * 1024;
pub const MIN_PART_SIZE: usize = 5 * 1024 * 1024;
pub const SIGNER_TYPES: [&str; 3] = ["DEFAULT", "S3SignerType", "AWSS3V4SignerType"];
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const SESSION_NAME: &str = "astrum-repository";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Settings {
    pub bucket: S3Bucket,
    #[serde(default)]
    pub security: S3Security,
    #[serde(default)]
    pub encryption: S3Encryption,
    #[serde(default)]
    pub connection: S3Connection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failover_buckets: Vec<S3FailoverBucket>,
    #[serde(default)]
    pub pre_signed_url_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Bucket {
    pub region: String,
    pub name: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i64>,
}

impl S3Bucket {
    pub fn active_region(&self) -> &str {
        let region = self.region.trim();
        if region.is_empty() || region.eq_ignore_ascii_case(DEFAULT_REGION) {
            FALLBACK_REGION
        } else {
            region
        }
    }

    pub fn key_prefix(&self) -> String {
        let prefix = self.prefix.trim_matches('/');
        if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}/")
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Security {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum S3EncryptionKind {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "s3ManagedEncryption")]
    S3Managed,
    #[serde(rename = "kmsManagedEncryption")]
    Kms,
}

impl S3EncryptionKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "" | "none" => Some(S3EncryptionKind::None),
            "s3ManagedEncryption" => Some(S3EncryptionKind::S3Managed),
            "kmsManagedEncryption" => Some(S3EncryptionKind::Kms),
            _ => None,
        }
    }

    pub fn as_api(self) -> Option<&'static str> {
        match self {
            S3EncryptionKind::None => None,
            S3EncryptionKind::S3Managed => Some("s3ManagedEncryption"),
            S3EncryptionKind::Kms => Some("kmsManagedEncryption"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Encryption {
    #[serde(default, rename = "type")]
    pub kind: S3EncryptionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Connection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_type: Option<String>,
    #[serde(default)]
    pub force_path_style: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connection_pool_size: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3FailoverBucket {
    pub region: String,
    pub bucket_name: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct S3Keys {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

impl fmt::Debug for S3Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Keys")
            .field("access_key_id", &self.access_key_id)
            .finish_non_exhaustive()
    }
}

pub struct S3BlobStore {
    name: String,
    client: Client,
    bucket: String,
    region: String,
    prefix: String,
    encryption: S3Encryption,
    metrics: Metrics,
    permits: Option<Arc<Semaphore>>,
    part_size: usize,
}

impl fmt::Debug for S3BlobStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3BlobStore")
            .field("name", &self.name)
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .finish_non_exhaustive()
    }
}

struct Multipart {
    upload_id: String,
    parts: Vec<CompletedPart>,
}

impl S3BlobStore {
    pub async fn open(
        name: impl Into<String>,
        settings: &S3Settings,
        keys: Option<S3Keys>,
        metrics: Metrics,
    ) -> Result<Self, Error> {
        let name = name.into();
        if settings.bucket.name.trim().is_empty() {
            return Err(Error::Config {
                store: name,
                message: "the bucket has no name".to_owned(),
            });
        }
        let pool = settings
            .connection
            .max_connection_pool_size
            .filter(|size| *size > 0);
        Ok(Self {
            client: client(settings, keys, pool).await,
            bucket: settings.bucket.name.clone(),
            region: settings.bucket.active_region().to_owned(),
            prefix: settings.bucket.key_prefix(),
            encryption: settings.encryption.clone(),
            metrics,
            permits: pool.map(|size| Arc::new(Semaphore::new(size as usize))),
            part_size: PART_SIZE,
            name,
        })
    }

    pub fn with_part_size(mut self, size: usize) -> Self {
        self.part_size = size.max(MIN_PART_SIZE);
        self
    }

    pub fn key_of(&self, id: BlobId) -> String {
        format!("{}content/{}", self.prefix, id.location())
    }

    pub async fn ensure_bucket(&self) -> Result<(), Error> {
        let _permit = self.permit().await;
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => return Ok(()),
            Err(error) if !not_found(&error) => return Err(failed("HeadBucket", error)),
            Err(_) => {}
        }
        let mut create = self.client.create_bucket().bucket(&self.bucket);
        if self.region != FALLBACK_REGION {
            create = create.create_bucket_configuration(
                CreateBucketConfiguration::builder()
                    .location_constraint(BucketLocationConstraint::from(self.region.as_str()))
                    .build(),
            );
        }
        create
            .send()
            .await
            .map_err(|error| failed("CreateBucket", error))?;
        tracing::info!(store = %self.name, bucket = %self.bucket, "created the bucket");
        Ok(())
    }

    async fn permit(&self) -> Option<OwnedSemaphorePermit> {
        match &self.permits {
            Some(permits) => permits.clone().acquire_owned().await.ok(),
            None => None,
        }
    }

    fn encryption(&self) -> (Option<ServerSideEncryption>, Option<String>) {
        match self.encryption.kind {
            S3EncryptionKind::None => (None, None),
            S3EncryptionKind::S3Managed => (Some(ServerSideEncryption::Aes256), None),
            S3EncryptionKind::Kms => (
                Some(ServerSideEncryption::AwsKms),
                self.encryption
                    .key_id
                    .clone()
                    .filter(|key| !key.trim().is_empty()),
            ),
        }
    }

    async fn write(
        &self,
        key: &str,
        mut content: ContentStream,
        upload: &mut Option<Multipart>,
    ) -> Result<(u64, Digests), Error> {
        let mut hasher = Hasher::default();
        let mut buffer = BytesMut::new();
        while let Some(chunk) = content.next().await {
            let chunk = chunk.map_err(Error::Content)?;
            hasher.update(&chunk);
            buffer.extend_from_slice(&chunk);
            while buffer.len() >= self.part_size {
                let part = buffer.split_to(self.part_size).freeze();
                if upload.is_none() {
                    *upload = Some(self.start(key).await?);
                }
                let started = upload.as_mut().expect("the upload was started above");
                self.send_part(key, started, part).await?;
            }
        }
        let rest = buffer.freeze();
        match upload {
            None => self.put_object(key, rest).await?,
            Some(started) => {
                if !rest.is_empty() {
                    self.send_part(key, started, rest).await?;
                }
                self.complete(key, started).await?;
            }
        }
        Ok(hasher.finish())
    }

    async fn put_object(&self, key: &str, body: Bytes) -> Result<(), Error> {
        let (encryption, kms_key) = self.encryption();
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_length(body.len() as i64)
            .body(ByteStream::from(body))
            .set_server_side_encryption(encryption)
            .set_ssekms_key_id(kms_key)
            .send()
            .await
            .map_err(|error| failed("PutObject", error))?;
        Ok(())
    }

    async fn start(&self, key: &str) -> Result<Multipart, Error> {
        let (encryption, kms_key) = self.encryption();
        let output = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .set_server_side_encryption(encryption)
            .set_ssekms_key_id(kms_key)
            .send()
            .await
            .map_err(|error| failed("CreateMultipartUpload", error))?;
        let upload_id = output.upload_id().ok_or_else(|| Error::S3 {
            operation: "CreateMultipartUpload",
            message: "the answer carried no upload id".to_owned(),
        })?;
        Ok(Multipart {
            upload_id: upload_id.to_owned(),
            parts: Vec::new(),
        })
    }

    async fn send_part(&self, key: &str, upload: &mut Multipart, body: Bytes) -> Result<(), Error> {
        let number = upload.parts.len() as i32 + 1;
        let output = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(&upload.upload_id)
            .part_number(number)
            .content_length(body.len() as i64)
            .body(ByteStream::from(body))
            .send()
            .await
            .map_err(|error| failed("UploadPart", error))?;
        upload.parts.push(
            CompletedPart::builder()
                .part_number(number)
                .set_e_tag(output.e_tag().map(str::to_owned))
                .build(),
        );
        Ok(())
    }

    async fn complete(&self, key: &str, upload: &mut Multipart) -> Result<(), Error> {
        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(&upload.upload_id)
            .multipart_upload(
                CompletedMultipartUpload::builder()
                    .set_parts(Some(std::mem::take(&mut upload.parts)))
                    .build(),
            )
            .send()
            .await
            .map_err(|error| failed("CompleteMultipartUpload", error))?;
        Ok(())
    }

    async fn abort(&self, key: &str, upload_id: &str) {
        if let Err(error) = self
            .client
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .send()
            .await
        {
            tracing::warn!(
                store = %self.name,
                key,
                error = %DisplayErrorContext(&error),
                "an unfinished upload was not aborted"
            );
        }
    }

    async fn get(
        &self,
        id: BlobId,
        range: Option<(u64, u64)>,
    ) -> Result<Option<(u64, ContentStream)>, Error> {
        let permit = self.permit().await;
        let mut request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(self.key_of(id));
        if let Some((offset, length)) = range {
            request = request.range(format!("bytes={}-{}", offset, offset + length - 1));
        }
        let output = match request.send().await {
            Ok(output) => output,
            Err(error) if not_found(&error) => return Ok(None),
            Err(error) => return Err(failed("GetObject", error)),
        };
        let length = output.content_length().unwrap_or(0).max(0) as u64;
        Ok(Some((
            length,
            crate::stream::holding(body(output.body), permit),
        )))
    }
}

#[async_trait]
impl BlobStore for S3BlobStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StoreKind {
        StoreKind::S3
    }

    async fn put(&self, content: ContentStream) -> Result<Written, Error> {
        let _permit = self.permit().await;
        let id = BlobId::new();
        let key = self.key_of(id);
        let mut upload = None;
        match self.write(&key, content, &mut upload).await {
            Ok((size, digests)) => {
                self.metrics.added(size).await;
                Ok(Written {
                    id,
                    store: self.name.clone(),
                    size,
                    digests,
                })
            }
            Err(error) => {
                if let Some(upload) = upload {
                    self.abort(&key, &upload.upload_id).await;
                }
                Err(error)
            }
        }
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        let Some((size, stream)) = self.get(id, None).await? else {
            return Ok(None);
        };
        Ok(Some(Content {
            size,
            offset: 0,
            length: size,
            stream,
        }))
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        let Some(size) = self.size(id).await? else {
            return Ok(None);
        };
        let length = clamp_range(size, offset, length)?;
        let stream = if length == 0 {
            stream::empty().boxed()
        } else {
            match self.get(id, Some((offset, length))).await? {
                Some((_, stream)) => stream,
                None => return Ok(None),
            }
        };
        Ok(Some(Content {
            size,
            offset,
            length,
            stream,
        }))
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        let _permit = self.permit().await;
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(self.key_of(id))
            .send()
            .await
        {
            Ok(output) => Ok(Some(output.content_length().unwrap_or(0).max(0) as u64)),
            Err(error) if not_found(&error) => Ok(None),
            Err(error) => Err(failed("HeadObject", error)),
        }
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        let Some(size) = self.size(id).await? else {
            return Ok(false);
        };
        let _permit = self.permit().await;
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(self.key_of(id))
            .send()
            .await
            .map_err(|error| failed("DeleteObject", error))?;
        self.metrics.removed(size).await;
        Ok(true)
    }

    async fn list(&self, after: Option<BlobId>, limit: usize) -> Result<Vec<StoredBlob>, Error> {
        let content = format!("{}content/", self.prefix);
        let mut start_after = after.map(|id| self.key_of(id));
        let mut found = Vec::new();
        loop {
            let _permit = self.permit().await;
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&content)
                .max_keys(limit.clamp(1, 1000) as i32);
            if let Some(key) = &start_after {
                request = request.start_after(key);
            }
            let output = request
                .send()
                .await
                .map_err(|error| failed("ListObjectsV2", error))?;
            let objects = output.contents();
            for object in objects {
                let Some(key) = object.key() else {
                    continue;
                };
                start_after = Some(key.to_owned());
                let Some(id) = key
                    .rsplit('/')
                    .next()
                    .and_then(|name| name.parse::<BlobId>().ok())
                else {
                    continue;
                };
                if key != self.key_of(id) {
                    continue;
                }
                let modified = object
                    .last_modified()
                    .and_then(|at| std::time::SystemTime::try_from(*at).ok())
                    .unwrap_or(std::time::SystemTime::now());
                found.push(StoredBlob {
                    id,
                    size: object.size().unwrap_or(0).max(0) as u64,
                    modified,
                });
                if found.len() >= limit {
                    return Ok(found);
                }
            }
            if objects.is_empty() || !output.is_truncated().unwrap_or(false) {
                return Ok(found);
            }
        }
    }

    async fn totals(&self) -> Result<Totals, Error> {
        self.metrics.totals().await
    }

    async fn available_space(&self) -> Result<Option<u64>, Error> {
        Ok(None)
    }

    async fn check(&self) -> Result<(), Error> {
        let _permit = self.permit().await;
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|error| failed("HeadBucket", error))?;
        Ok(())
    }
}

async fn client(settings: &S3Settings, keys: Option<S3Keys>, pool: Option<u32>) -> Client {
    let mut http = aws_smithy_http_client::Builder::new();
    if let Some(size) = pool {
        http = http.pool_max_idle_per_host(size as usize);
    }
    let http = http
        .tls_provider(tls::Provider::Rustls(
            tls::rustls_provider::CryptoMode::Ring,
        ))
        .build_https();
    let region = Region::new(settings.bucket.active_region().to_owned());
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .http_client(http)
        .region(region.clone())
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .build(),
        );
    if let Some(keys) = keys {
        loader = loader.credentials_provider(Credentials::new(
            keys.access_key_id,
            keys.secret_access_key,
            keys.session_token,
            None,
            "astrum-blob-store",
        ));
    }
    let mut sdk = loader.load().await;
    if let Some(role) = settings
        .security
        .role
        .as_deref()
        .map(str::trim)
        .filter(|role| !role.is_empty())
    {
        let provider = AssumeRoleProvider::builder(role)
            .session_name(SESSION_NAME)
            .region(region)
            .configure(&sdk)
            .build()
            .await;
        sdk = sdk
            .into_builder()
            .credentials_provider(SharedCredentialsProvider::new(provider))
            .build();
    }
    let mut config = aws_sdk_s3::config::Builder::from(&sdk)
        .force_path_style(settings.connection.force_path_style)
        .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
        .response_checksum_validation(ResponseChecksumValidation::WhenRequired);
    if let Some(endpoint) = settings
        .connection
        .endpoint
        .as_deref()
        .map(str::trim)
        .filter(|endpoint| !endpoint.is_empty())
    {
        config = config.endpoint_url(endpoint);
    }
    Client::from_conf(config.build())
}

fn body(body: ByteStream) -> ContentStream {
    stream::try_unfold(body, |mut body| async move {
        match body.next().await {
            Some(Ok(bytes)) => Ok(Some((bytes, body))),
            Some(Err(error)) => Err(std::io::Error::other(error)),
            None => Ok(None),
        }
    })
    .boxed()
}

fn not_found<E: ProvideErrorMetadata>(error: &SdkError<E>) -> bool {
    match error {
        SdkError::ServiceError(context) => {
            context.raw().status().as_u16() == 404
                || matches!(
                    context.err().code(),
                    Some("NoSuchKey" | "NotFound" | "NoSuchBucket")
                )
        }
        _ => false,
    }
}

fn failed<E>(operation: &'static str, error: SdkError<E>) -> Error
where
    E: std::error::Error + 'static,
{
    Error::S3 {
        operation,
        message: DisplayErrorContext(&error).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_region_falls_back_to_us_east_1() {
        let mut bucket = S3Bucket {
            region: "DEFAULT".to_owned(),
            name: "b".to_owned(),
            ..Default::default()
        };
        assert_eq!(bucket.active_region(), "us-east-1");
        bucket.region = String::new();
        assert_eq!(bucket.active_region(), "us-east-1");
        bucket.region = "eu-central-1".to_owned();
        assert_eq!(bucket.active_region(), "eu-central-1");
    }

    #[test]
    fn prefixes_become_one_folder() {
        let mut bucket = S3Bucket::default();
        assert_eq!(bucket.key_prefix(), "");
        bucket.prefix = "/team/blobs/".to_owned();
        assert_eq!(bucket.key_prefix(), "team/blobs/");
    }

    #[test]
    fn keys_are_never_printed() {
        let keys = S3Keys {
            access_key_id: "AKIDEXAMPLE".to_owned(),
            secret_access_key: "very-secret".to_owned(),
            session_token: Some("token-secret".to_owned()),
        };
        let printed = format!("{keys:?}");
        assert!(printed.contains("AKIDEXAMPLE"));
        assert!(!printed.contains("secret"), "{printed}");
    }

    #[test]
    fn encryption_names_follow_the_api() {
        for (name, kind) in [
            ("s3ManagedEncryption", S3EncryptionKind::S3Managed),
            ("kmsManagedEncryption", S3EncryptionKind::Kms),
        ] {
            assert_eq!(S3EncryptionKind::parse(name), Some(kind));
            assert_eq!(kind.as_api(), Some(name));
        }
        assert_eq!(S3EncryptionKind::parse(""), Some(S3EncryptionKind::None));
        assert_eq!(
            S3EncryptionKind::parse("none"),
            Some(S3EncryptionKind::None)
        );
        assert_eq!(S3EncryptionKind::parse("aes"), None);
    }
}
