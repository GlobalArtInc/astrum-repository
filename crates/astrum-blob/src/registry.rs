use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use astrum_db::setting;
use astrum_security::SecretBox;
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use sea_orm::DatabaseConnection;

use crate::config::{self, FileSettings, Settings, StoreConfig};
use crate::s3::S3Security;
use crate::{
    BlobStore, Error, FileBlobStore, GroupBlobStore, Metrics, S3BlobStore, S3Keys, S3Settings,
};

pub const BLOBS_DIR: &str = "blobs";
pub const DEFAULT_STORE: &str = "default";
const PROVISIONED: &str = "blobstore.provisioned";
const MAX_GROUP_DEPTH: usize = 8;

pub struct BlobStores {
    db: DatabaseConnection,
    blobs_dir: PathBuf,
    secrets: SecretBox,
    open: Mutex<HashMap<String, Arc<dyn BlobStore>>>,
}

impl std::fmt::Debug for BlobStores {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobStores")
            .field("blobs_dir", &self.blobs_dir)
            .finish_non_exhaustive()
    }
}

impl BlobStores {
    pub fn new(db: DatabaseConnection, data_dir: &Path, secrets: SecretBox) -> Self {
        Self {
            db,
            blobs_dir: data_dir.join(BLOBS_DIR),
            secrets,
            open: Mutex::new(HashMap::new()),
        }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn blobs_dir(&self) -> &Path {
        &self.blobs_dir
    }

    pub fn resolve(&self, path: &str) -> PathBuf {
        normalize(&self.blobs_dir.join(path.trim()))
    }

    pub async fn configs(&self) -> Result<Vec<StoreConfig>, Error> {
        config::list(&self.db).await
    }

    pub async fn config(&self, name: &str) -> Result<Option<StoreConfig>, Error> {
        config::find(&self.db, name).await
    }

    pub async fn get(&self, name: &str) -> Result<Arc<dyn BlobStore>, Error> {
        self.get_at(name, 0).await
    }

    pub async fn open(&self, config: &StoreConfig) -> Result<Arc<dyn BlobStore>, Error> {
        self.open_at(config, 0).await
    }

    pub async fn open_s3(&self, name: &str, settings: &S3Settings) -> Result<S3BlobStore, Error> {
        let keys = self.keys(name, &settings.security)?;
        S3BlobStore::open(name, settings, keys, self.metrics(name)).await
    }

    pub fn metrics(&self, name: &str) -> Metrics {
        Metrics::new(self.db.clone(), name)
    }

    pub fn forget(&self) {
        self.cache().clear();
    }

    pub async fn overlapping(
        &self,
        path: &str,
        except: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        let wanted = self.resolve(path);
        Ok(self
            .configs()
            .await?
            .into_iter()
            .filter(|config| Some(config.name.as_str()) != except)
            .filter(|config| {
                config.file().is_some_and(|file| {
                    let other = self.resolve(&file.path);
                    other.starts_with(&wanted) || wanted.starts_with(&other)
                })
            })
            .map(|config| config.name)
            .collect())
    }

    fn cache(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<dyn BlobStore>>> {
        self.open
            .lock()
            .expect("the blob store cache lock is never poisoned")
    }

    fn get_at(&self, name: &str, depth: usize) -> BoxFuture<'_, Result<Arc<dyn BlobStore>, Error>> {
        let name = name.to_owned();
        async move {
            if let Some(store) = self.cache().get(&name) {
                return Ok(store.clone());
            }
            let config = self
                .config(&name)
                .await?
                .ok_or_else(|| Error::UnknownStore(name.clone()))?;
            let store = self.open_at(&config, depth).await?;
            Ok(self.cache().entry(name).or_insert(store).clone())
        }
        .boxed()
    }

    async fn open_at(
        &self,
        config: &StoreConfig,
        depth: usize,
    ) -> Result<Arc<dyn BlobStore>, Error> {
        let name = config.name.as_str();
        Ok(match &config.settings {
            Settings::File(FileSettings { path }) => {
                Arc::new(FileBlobStore::open(name, self.resolve(path), self.metrics(name)).await?)
            }
            Settings::S3(settings) => Arc::new(self.open_s3(name, settings).await?),
            Settings::Group(group) => {
                if depth >= MAX_GROUP_DEPTH {
                    return Err(Error::Config {
                        store: name.to_owned(),
                        message: "its groups nest too deep or contain themselves".to_owned(),
                    });
                }
                let mut members = Vec::with_capacity(group.members.len());
                for member in &group.members {
                    members.push(self.get_at(member, depth + 1).await?);
                }
                Arc::new(GroupBlobStore::new(name, members, group.fill_policy)?)
            }
        })
    }

    fn keys(&self, name: &str, security: &S3Security) -> Result<Option<S3Keys>, Error> {
        let present =
            |value: &Option<String>| value.clone().filter(|value| !value.trim().is_empty());
        let Some(access_key_id) = present(&security.access_key_id) else {
            return Ok(None);
        };
        let sealed = present(&security.secret_access_key).ok_or_else(|| Error::Config {
            store: name.to_owned(),
            message: "the access key has no secret".to_owned(),
        })?;
        Ok(Some(S3Keys {
            access_key_id,
            secret_access_key: self.secrets.open(&sealed)?,
            session_token: present(&security.session_token)
                .map(|token| self.secrets.open(&token))
                .transpose()?,
        }))
    }
}

pub async fn bootstrap(db: &DatabaseConnection, data_dir: &Path) -> Result<bool, Error> {
    if setting::get::<bool>(db, PROVISIONED).await?.is_some() {
        return Ok(false);
    }
    let mut created = false;
    if config::list(db).await?.is_empty() {
        FileBlobStore::open(
            DEFAULT_STORE,
            data_dir.join(BLOBS_DIR).join(DEFAULT_STORE),
            Metrics::new(db.clone(), DEFAULT_STORE),
        )
        .await?;
        let config = StoreConfig {
            name: DEFAULT_STORE.to_owned(),
            settings: Settings::File(FileSettings {
                path: DEFAULT_STORE.to_owned(),
            }),
            soft_quota: None,
        };
        created = match config::insert(db, &config).await {
            Ok(()) => true,
            Err(Error::Duplicate(_)) => false,
            Err(error) => return Err(error),
        };
        if created {
            tracing::info!(store = DEFAULT_STORE, "created the default blob store");
        }
    }
    setting::put(db, PROVISIONED, &true).await?;
    Ok(created)
}

fn normalize(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normal.pop();
            }
            other => normal.push(other),
        }
    }
    normal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_normalize_without_touching_the_disk() {
        assert_eq!(
            normalize(Path::new("/data/blobs/./a/../b/")),
            PathBuf::from("/data/blobs/b")
        );
        let base = Path::new("/data/blobs");
        for (path, expected) in [
            ("/abs/path", "/abs/path"),
            ("../outside", "/data/outside"),
            ("inside", "/data/blobs/inside"),
        ] {
            assert_eq!(normalize(&base.join(path)), PathBuf::from(expected));
        }
    }
}
