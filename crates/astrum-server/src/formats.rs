use std::sync::Arc;

use astrum_api::AppState;
use astrum_content::Content;
use axum::Router;

pub fn register(content: &Content) {
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    content.register(Arc::new(astrum_format_helm::Helm::default()));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_pypi::PyPi));
    content.register(Arc::new(astrum_format_raw::Raw));
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(astrum_format_docker::routes())
        .merge(astrum_format_npm::routes())
}
