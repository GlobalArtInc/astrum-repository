use std::sync::Arc;

use astrum_api::AppState;
use astrum_content::Content;
use astrum_tasks::Tasks;
use axum::Router;

pub fn register(content: &Content) {
    content.register(Arc::new(astrum_format_alpine::Alpine::default()));
    content.register(Arc::new(astrum_format_ansible::Ansible::default()));
    content.register(Arc::new(astrum_format_apt::Apt::default()));
    content.register(Arc::new(astrum_format_cargo::Cargo::default()));
    content.register(Arc::new(astrum_format_chef::Chef::default()));
    content.register(Arc::new(astrum_format_cocoapods::CocoaPods));
    content.register(Arc::new(astrum_format_composer::Composer));
    content.register(Arc::new(astrum_format_conan::Conan));
    content.register(Arc::new(astrum_format_conda::Conda));
    content.register(Arc::new(astrum_format_docker::Docker::default()));
    content.register(Arc::new(astrum_format_gitlfs::GitLfs));
    content.register(Arc::new(astrum_format_go::Go));
    content.register(Arc::new(astrum_format_helm::Helm::default()));
    content.register(Arc::new(astrum_format_huggingface::HuggingFace::default()));
    content.register(Arc::new(astrum_format_maven::Maven));
    content.register(Arc::new(astrum_format_npm::Npm::default()));
    content.register(Arc::new(astrum_format_nuget::Nuget::default()));
    content.register(Arc::new(astrum_format_p2::P2::default()));
    content.register(Arc::new(astrum_format_pub::Pub));
    content.register(Arc::new(astrum_format_puppet::Puppet::default()));
    content.register(Arc::new(astrum_format_pypi::PyPi));
    content.register(Arc::new(astrum_format_r::R::default()));
    content.register(Arc::new(astrum_format_raw::Raw));
    content.register(Arc::new(astrum_format_swift::Swift));
    content.register(Arc::new(astrum_format_terraform::Terraform::default()));
    content.register(Arc::new(astrum_format_rubygems::Rubygems));
    content.register(Arc::new(astrum_format_yum::Yum::default()));
}

pub fn register_tasks(tasks: &Tasks) {
    tasks.register(Arc::new(
        astrum_format_alpine::task::RebuildMetadata::default(),
    ));
    tasks.register(Arc::new(astrum_format_apt::task::RebuildMetadata::default()));
    tasks.register(Arc::new(
        astrum_format_conda::task::RebuildMetadata::default(),
    ));
    tasks.register(Arc::new(astrum_format_docker::gc::Gc));
    tasks.add_expiry(Arc::new(astrum_format_docker::gc::UploadExpiry));
    tasks.register(Arc::new(astrum_format_r::task::RebuildMetadata::default()));
    tasks.register(Arc::new(astrum_format_yum::task::RebuildMetadata::default()));
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(astrum_format_conan::routes())
        .merge(astrum_format_docker::routes())
        .merge(astrum_format_npm::routes())
        .merge(astrum_format_terraform::routes())
}
