mod status;

use std::path::PathBuf;
use std::sync::Arc;

use astrum_db::Database;
use axum::Router;
use axum::http::StatusCode;

pub use status::{STATUS, STATUS_WRITABLE};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Inner>,
}

struct Inner {
    db: Database,
    data_dir: PathBuf,
}

impl AppState {
    pub fn new(db: Database, data_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner { db, data_dir }),
        }
    }

    pub fn db(&self) -> &Database {
        &self.inner.db
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.inner.data_dir
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(status::routes())
        .fallback(not_found)
        .with_state(state)
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
