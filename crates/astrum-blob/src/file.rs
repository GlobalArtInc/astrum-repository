use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use futures_util::StreamExt;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::digest::Hasher;
use crate::{
    BlobId, BlobStore, Content, ContentStream, Error, Metrics, StoreKind, Totals, Written,
    clamp_range, stream,
};

pub const CONTENT_DIR: &str = "content";
pub const TEMP_DIR: &str = "tmp";
pub const STALE_TEMP: Duration = Duration::from_secs(3600);
const TEMP_SUFFIX: &str = ".part";

#[derive(Debug)]
pub struct FileBlobStore {
    name: String,
    root: PathBuf,
    metrics: Metrics,
}

impl FileBlobStore {
    pub async fn open(
        name: impl Into<String>,
        root: impl Into<PathBuf>,
        metrics: Metrics,
    ) -> Result<Self, Error> {
        let store = Self {
            name: name.into(),
            root: root.into(),
            metrics,
        };
        for dir in [store.content_dir(), store.temp_dir()] {
            tokio::fs::create_dir_all(&dir)
                .await
                .map_err(Error::io(&dir))?;
        }
        store.sweep_temp(STALE_TEMP).await;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_of(&self, id: BlobId) -> PathBuf {
        self.content_dir().join(id.location())
    }

    fn content_dir(&self) -> PathBuf {
        self.root.join(CONTENT_DIR)
    }

    fn temp_dir(&self) -> PathBuf {
        self.root.join(TEMP_DIR)
    }

    pub async fn sweep_temp(&self, older_than: Duration) -> usize {
        let dir = self.temp_dir();
        tokio::task::spawn_blocking(move || sweep(&dir, older_than))
            .await
            .unwrap_or_default()
    }

    pub async fn remove_if_empty(&self) -> Result<bool, Error> {
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || remove_empty_tree(&root))
            .await
            .map_err(|error| Error::Io {
                path: self.root.clone(),
                source: io::Error::other(error),
            })?
            .map_err(Error::io(&self.root))
    }

    async fn write_temp(
        &self,
        temp: &Path,
        mut content: ContentStream,
    ) -> Result<(u64, crate::Digests), Error> {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(temp)
            .await
            .map_err(Error::io(temp))?;
        let mut hasher = Hasher::default();
        while let Some(chunk) = content.next().await {
            let chunk = chunk.map_err(Error::Content)?;
            hasher.update(&chunk);
            file.write_all(&chunk).await.map_err(Error::io(temp))?;
        }
        file.flush().await.map_err(Error::io(temp))?;
        file.sync_all().await.map_err(Error::io(temp))?;
        Ok(hasher.finish())
    }

    async fn open_blob(&self, id: BlobId) -> Result<Option<(tokio::fs::File, u64)>, Error> {
        let path = self.path_of(id);
        let file = match tokio::fs::File::open(&path).await {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(Error::Io {
                    path,
                    source: error,
                });
            }
        };
        let size = file.metadata().await.map_err(Error::io(&path))?.len();
        Ok(Some((file, size)))
    }
}

#[async_trait]
impl BlobStore for FileBlobStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> StoreKind {
        StoreKind::File
    }

    async fn put(&self, content: ContentStream) -> Result<Written, Error> {
        let id = BlobId::new();
        let temp = self.temp_dir().join(format!("{id}{TEMP_SUFFIX}"));
        let guard = TempFile(Some(temp.clone()));
        let (size, digests) = self.write_temp(&temp, content).await?;
        let target = self.path_of(id);
        let parent = target
            .parent()
            .expect("a blob path always has a parent")
            .to_owned();
        tokio::fs::create_dir_all(&parent)
            .await
            .map_err(Error::io(&parent))?;
        tokio::fs::rename(&temp, &target)
            .await
            .map_err(Error::io(&target))?;
        guard.keep();
        if let Err(error) = sync_dir(&parent).await {
            tracing::warn!(dir = %parent.display(), %error, "a directory could not be synced");
        }
        self.metrics.added(size).await;
        Ok(Written {
            id,
            store: self.name.clone(),
            size,
            digests,
        })
    }

    async fn read(&self, id: BlobId) -> Result<Option<Content>, Error> {
        let Some((file, size)) = self.open_blob(id).await? else {
            return Ok(None);
        };
        Ok(Some(Content {
            size,
            offset: 0,
            length: size,
            stream: stream::from_file(file, size),
        }))
    }

    async fn read_range(
        &self,
        id: BlobId,
        offset: u64,
        length: u64,
    ) -> Result<Option<Content>, Error> {
        let Some((mut file, size)) = self.open_blob(id).await? else {
            return Ok(None);
        };
        let length = clamp_range(size, offset, length)?;
        file.seek(io::SeekFrom::Start(offset))
            .await
            .map_err(Error::io(self.path_of(id)))?;
        Ok(Some(Content {
            size,
            offset,
            length,
            stream: stream::from_file(file, length),
        }))
    }

    async fn size(&self, id: BlobId) -> Result<Option<u64>, Error> {
        let path = self.path_of(id);
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => Ok(Some(metadata.len())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(Error::Io {
                path,
                source: error,
            }),
        }
    }

    async fn delete(&self, id: BlobId) -> Result<bool, Error> {
        let Some(size) = self.size(id).await? else {
            return Ok(false);
        };
        let path = self.path_of(id);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {
                self.metrics.removed(size).await;
                Ok(true)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(Error::Io {
                path,
                source: error,
            }),
        }
    }

    async fn totals(&self) -> Result<Totals, Error> {
        self.metrics.totals().await
    }

    async fn available_space(&self) -> Result<Option<u64>, Error> {
        let root = self.root.clone();
        let stats = tokio::task::spawn_blocking(move || rustix::fs::statvfs(&root))
            .await
            .map_err(|error| Error::Io {
                path: self.root.clone(),
                source: io::Error::other(error),
            })?
            .map_err(|errno| Error::Io {
                path: self.root.clone(),
                source: errno.into(),
            })?;
        Ok(Some(stats.f_bavail.saturating_mul(stats.f_frsize)))
    }

    async fn check(&self) -> Result<(), Error> {
        let probe = self
            .temp_dir()
            .join(format!("check-{}{TEMP_SUFFIX}", BlobId::new()));
        tokio::fs::write(&probe, b"astrum")
            .await
            .map_err(Error::io(&probe))?;
        tokio::fs::remove_file(&probe)
            .await
            .map_err(Error::io(&probe))?;
        let content = self.content_dir();
        let metadata = tokio::fs::metadata(&content)
            .await
            .map_err(Error::io(&content))?;
        if !metadata.is_dir() {
            return Err(Error::Io {
                path: content,
                source: io::Error::new(io::ErrorKind::NotADirectory, "not a directory"),
            });
        }
        Ok(())
    }
}

struct TempFile(Option<PathBuf>);

impl TempFile {
    fn keep(mut self) {
        self.0 = None;
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

async fn sync_dir(dir: &Path) -> io::Result<()> {
    let dir = dir.to_owned();
    tokio::task::spawn_blocking(move || std::fs::File::open(dir)?.sync_all())
        .await
        .map_err(io::Error::other)?
}

fn sweep(dir: &Path, older_than: Duration) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let stale = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| {
                now.duration_since(modified)
                    .is_ok_and(|age| age >= older_than)
            });
        let ours = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(TEMP_SUFFIX));
        if stale && ours && std::fs::remove_file(&path).is_ok() {
            tracing::info!(path = %path.display(), "removed an unfinished blob write");
            removed += 1;
        }
    }
    removed
}

fn remove_empty_tree(root: &Path) -> io::Result<bool> {
    fn prune(dir: &Path) -> io::Result<bool> {
        let mut empty = true;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                empty &= prune(&entry.path())?;
            } else {
                empty = false;
            }
        }
        if empty {
            std::fs::remove_dir(dir)?;
        }
        Ok(empty)
    }
    match prune(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(true),
        other => other,
    }
}
