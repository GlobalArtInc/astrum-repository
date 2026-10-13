use std::sync::RwLock;
use std::time::{Duration, Instant};

use astrum_db::setting;
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;

const FRESH: Duration = Duration::from_secs(5);

tokio::task_local! {
    static CONFIGURED: Option<String>;
}

pub fn configured() -> Option<String> {
    CONFIGURED.try_with(Clone::clone).ok().flatten()
}

#[derive(Default)]
pub(crate) struct Cache {
    value: RwLock<Option<(Instant, Option<String>)>>,
}

impl AppState {
    pub async fn configured_base_url(&self) -> Option<String> {
        let cache = &self.inner.base_url;
        if let Some((at, value)) = cache.value.read().expect("never poisoned").clone()
            && at.elapsed() < FRESH
        {
            return value;
        }
        let value = match setting::get::<BaseUrl>(self.db().conn(), BASE_URL_KEY).await {
            Ok(Some(base)) if base.enabled && !base.url.trim().is_empty() => {
                Some(base.url.trim().trim_end_matches('/').to_owned())
            }
            Ok(_) => None,
            Err(error) => {
                tracing::warn!(%error, "the base URL could not be read");
                None
            }
        };
        *cache.value.write().expect("never poisoned") = Some((Instant::now(), value.clone()));
        value
    }

    pub fn forget_base_url(&self) {
        *self.inner.base_url.value.write().expect("never poisoned") = None;
    }
}

pub async fn scope(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let configured = state.configured_base_url().await;
    CONFIGURED.scope(configured, next.run(request)).await
}
