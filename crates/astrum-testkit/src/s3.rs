use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use s3s::auth::SimpleAuth;
use s3s::service::S3ServiceBuilder;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub const ACCESS_KEY: &str = "astrum-test";
pub const SECRET_KEY: &str = "astrum-test-secret";

pub struct S3Server {
    root: TempDir,
    address: SocketAddr,
    task: JoinHandle<()>,
}

impl S3Server {
    pub async fn start() -> Self {
        let root = tempfile::tempdir().expect("a temporary directory for the S3 server");
        let fs = s3s_fs::FileSystem::new(root.path()).expect("an S3 file system");
        let mut builder = S3ServiceBuilder::new(fs);
        builder.set_auth(SimpleAuth::from_single(ACCESS_KEY, SECRET_KEY));
        let service = builder.build();
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a local port for the S3 server");
        let address = listener.local_addr().expect("the bound address");
        let task = tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                let service = service.clone();
                tokio::spawn(async move {
                    let _ = Builder::new(TokioExecutor::new())
                        .serve_connection(TokioIo::new(socket), service)
                        .await;
                });
            }
        });
        Self {
            root,
            address,
            task,
        }
    }

    pub fn endpoint(&self) -> String {
        format!("http://{}", self.address)
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn objects(&self, bucket: &str) -> Vec<String> {
        let base = self.root.path().join(bucket);
        files_under(&base)
            .into_iter()
            .filter_map(|path| {
                path.strip_prefix(&base)
                    .ok()
                    .map(|key| key.to_string_lossy().into_owned())
            })
            .filter(|key| !key.starts_with('.'))
            .collect()
    }
}

impl Drop for S3Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}
