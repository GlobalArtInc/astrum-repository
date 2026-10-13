use std::sync::Arc;

use astrum_api::AppState;
use astrum_content::Content;
use astrum_tasks::Tasks;
use axum::Router;

pub fn register(content: &Content) {
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    content.register(Arc::new(astrum_format_helm::Helm::default()));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_nuget::Nuget::default()));
    content.register(Arc::new(astrum_format_pypi::PyPi));
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_rubygems::Rubygems));
    content.register(Arc::new(astrum_format_yum::Yum::default()));
}

pub fn register_tasks(tasks: &Tasks) {
    tasks.register(Arc::new(astrum_format_docker::gc::Gc));
    tasks.add_expiry(Arc::new(astrum_format_docker::gc::UploadExpiry));
    tasks.register(Arc::new(astrum_format_yum::task::RebuildMetadata::default()));
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(astrum_format_docker::routes())
        .merge(astrum_format_npm::routes())
}
