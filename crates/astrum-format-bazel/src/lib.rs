mod blobs;
pub mod documents;
mod hosted;
mod module_file;
pub mod names;
mod remote;

use astrum_content::format::UploadedFile;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Grant, Serve, UploadField,
    UploadForm, UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::documents::Document;
use crate::names::{Entry, REGISTRY_PATH, Route, VersionFile};

pub const ID: &str = "bazel";

const ALLOW_HOSTED: &str = "GET,HEAD,PUT,DELETE";
const ALLOW_PROXY: &str = "GET,HEAD,DELETE";
const ALLOW_GROUP: &str = "GET,HEAD";

#[derive(Default)]
pub struct Bazel;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Bazel module registry. Pass it to Bazel as <code>--registry={url}</code>, for example in <code>.bazelrc</code>; Bazel reads <code>{url}/modules/&lt;module&gt;/metadata.json</code> and the files of each version from there.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn generated(request: &ContentRequest, document: &Document) -> Response {
    let body = documents::render(document);
    let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..16]));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    let response = match (unchanged, &request.method) {
        (true, _) => builder.status(StatusCode::NOT_MODIFIED).body(Body::empty()),
        (false, &Method::HEAD) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, hosted::JSON_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, _) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, hosted::JSON_TYPE)
            .body(Body::from(body)),
    };
    response.expect("a JSON response is valid")
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => ALLOW_HOSTED,
            RepositoryType::Proxy => ALLOW_PROXY,
            RepositoryType::Group => ALLOW_GROUP,
        },
    }
}

fn unknown_path() -> ContentError {
    ContentError::BadRequest(
        "upload to /bazel_registry.json, /modules/<module>/metadata.json or /modules/<module>/<version>/ with MODULE.bazel, source.json, patches/<file> or overlay/<file>"
            .to_owned(),
    )
}

fn created(replaced: bool) -> Result<Response, ContentError> {
    Response::builder()
        .status(match replaced {
            true => StatusCode::OK,
            false => StatusCode::CREATED,
        })
        .body(Body::empty())
        .map_err(ContentError::internal)
}

async fn group_members(ctx: &Ctx, module: Option<&str>) -> Result<Vec<Ctx>, ContentError> {
    let mut members: Vec<Ctx> = group::members(ctx)
        .await?
        .into_iter()
        .map(|member| ctx.member(member))
        .collect();
    if let Some(module) = module {
        let mut owned = false;
        for member in members
            .iter()
            .filter(|member| member.kind() == RepositoryType::Hosted)
        {
            if hosted::has_module(member, module).await? {
                owned = true;
                break;
            }
        }
        if owned {
            members.retain(|member| member.kind() == RepositoryType::Hosted);
        }
    }
    Ok(members)
}

fn visible(ctx: &Ctx, module: &str, mut document: Document) -> Document {
    if matches!(ctx.grant(View::Read), Grant::Whole) {
        return document;
    }
    let readable = |version: &str| {
        VersionFile::new(module, version, Entry::Module)
            .is_some_and(|file| ctx.allowed(View::Read, &file.path()))
    };
    if let Some(Value::Array(versions)) = document.get_mut("versions") {
        versions.retain(|version| version.as_str().is_some_and(readable));
    }
    if let Some(Value::Object(yanked)) = document.get_mut("yanked_versions") {
        yanked.retain(|version, _| readable(version));
    }
    document
}

impl Bazel {
    async fn leaf_file(
        &self,
        member: &Ctx,
        file: &VersionFile,
    ) -> Result<Option<Asset>, ContentError> {
        match member.kind() {
            RepositoryType::Hosted => store::find(member.db(), member.name(), &file.path()).await,
            RepositoryType::Proxy => remote::file(member, file).await,
            RepositoryType::Group => Ok(None),
        }
    }

    async fn file(
        &self,
        ctx: &Ctx,
        file: &VersionFile,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        if ctx.kind() != RepositoryType::Group {
            return Ok(self
                .leaf_file(ctx, file)
                .await?
                .map(|found| (ctx.clone(), found)));
        }
        for member in group_members(ctx, Some(&file.module)).await? {
            match self.leaf_file(&member, file).await {
                Ok(Some(found)) => return Ok(Some((member, found))),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                }
            }
        }
        Ok(None)
    }

    async fn stored_document(
        &self,
        ctx: &Ctx,
        route: &Route,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        let path = match route {
            Route::Registry => REGISTRY_PATH.to_owned(),
            Route::Metadata(module) => names::metadata_path(module),
            _ => return Ok(None),
        };
        let members = match ctx.kind() {
            RepositoryType::Group => {
                let module = match route {
                    Route::Metadata(module) => Some(module.as_str()),
                    _ => None,
                };
                group_members(ctx, module).await?
            }
            _ => vec![ctx.clone()],
        };
        for member in members {
            let found = match member.kind() {
                RepositoryType::Hosted => store::find(member.db(), member.name(), &path).await,
                RepositoryType::Proxy => match route {
                    Route::Registry => remote::registry(&member).await,
                    Route::Metadata(module) => remote::metadata(&member, module).await,
                    _ => Ok(None),
                },
                RepositoryType::Group => Ok(None),
            };
            match found {
                Ok(Some(found)) => return Ok(Some((member, found))),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(repository = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                }
            }
        }
        Ok(None)
    }

    async fn read_registry(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, REGISTRY_PATH)?;
        match ctx.kind() {
            RepositoryType::Hosted => {
                match store::find(ctx.db(), ctx.name(), REGISTRY_PATH).await? {
                    Some(found) => {
                        serve(
                            ctx,
                            &found,
                            &request.method,
                            &request.headers,
                            Serve::default(),
                        )
                        .await
                    }
                    None => Ok(generated(request, &documents::merge_registries(&[]))),
                }
            }
            RepositoryType::Proxy => {
                let found = remote::registry(ctx).await?.ok_or_else(|| {
                    ContentError::not_found("the remote has no bazel_registry.json")
                })?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await
            }
            RepositoryType::Group => {
                let mut documents = Vec::new();
                for member in group_members(ctx, None).await? {
                    let found = match member.kind() {
                        RepositoryType::Hosted => hosted::registry(&member).await.map(Some),
                        RepositoryType::Proxy => match remote::registry(&member).await {
                            Ok(Some(found)) => remote::parsed(&member, &found).await.map(Some),
                            Ok(None) => Ok(None),
                            Err(error) => Err(error),
                        },
                        RepositoryType::Group => Ok(None),
                    };
                    match found {
                        Ok(Some(document)) => documents.push(document),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no usable bazel_registry.json");
                        }
                    }
                }
                Ok(generated(request, &documents::merge_registries(&documents)))
            }
        }
    }

    async fn read_metadata(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        module: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &names::metadata_path(module))?;
        let missing = || ContentError::not_found(format!("the module {module} was not found"));
        match ctx.kind() {
            RepositoryType::Hosted => {
                let document = hosted::metadata(ctx, module).await?.ok_or_else(missing)?;
                Ok(generated(request, &visible(ctx, module, document)))
            }
            RepositoryType::Proxy => {
                let found = remote::metadata(ctx, module).await?.ok_or_else(missing)?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await
            }
            RepositoryType::Group => {
                let mut documents = Vec::new();
                for member in group_members(ctx, Some(module)).await? {
                    let found = match member.kind() {
                        RepositoryType::Hosted => hosted::metadata(&member, module).await,
                        RepositoryType::Proxy => match remote::metadata(&member, module).await {
                            Ok(Some(found)) => remote::parsed(&member, &found).await.map(Some),
                            Ok(None) => Ok(None),
                            Err(error) => Err(error),
                        },
                        RepositoryType::Group => Ok(None),
                    };
                    match found {
                        Ok(Some(document)) => documents.push(document),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no metadata for the module");
                        }
                    }
                }
                if documents.is_empty() {
                    return Err(missing());
                }
                let merged = documents::merge_metadata(documents);
                Ok(generated(request, &visible(ctx, module, merged)))
            }
        }
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        file: &VersionFile,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &file.path())?;
        let (owner, found) = self.file(ctx, file).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "{} of {} {} was not found",
                file.entry.relative(),
                file.module,
                file.version
            ))
        })?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn put(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(not_allowed(&request.method, ctx.kind()));
        }
        match names::route(&request.path) {
            Route::Registry => created(hosted::put_registry(ctx, request.into_stream()).await?),
            Route::Metadata(module) => {
                created(hosted::put_metadata(ctx, &module, request.into_stream()).await?)
            }
            Route::File(file) => {
                hosted::put_file(ctx, &file, request.into_stream()).await?;
                created(false)
            }
            Route::Root | Route::Other => Err(unknown_path()),
        }
    }

    async fn delete(&self, ctx: &Ctx, route: Route) -> Result<Response, ContentError> {
        if ctx.kind() == RepositoryType::Group {
            return Err(not_allowed(&Method::DELETE, ctx.kind()));
        }
        let path = match route {
            Route::Registry => REGISTRY_PATH.to_owned(),
            Route::Metadata(module) => names::metadata_path(&module),
            Route::File(file) => file.path(),
            Route::Root | Route::Other => return Err(unknown_path()),
        };
        ctx.require(View::Delete, &path)?;
        if !store::delete(ctx, &path).await? {
            return Err(ContentError::not_found(path));
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }
}

enum Target {
    Metadata,
    File(Entry),
}

struct Planned<'a> {
    target: Target,
    file: &'a UploadedFile,
}

impl Planned<'_> {
    fn rank(&self) -> u8 {
        match &self.target {
            Target::Metadata => 0,
            Target::File(Entry::Patch(_) | Entry::Overlay(_)) => 1,
            Target::File(Entry::Module) => 2,
            Target::File(Entry::Source) => 3,
        }
    }
}

fn target_of(name: &str) -> Result<Target, ContentError> {
    let name = name.trim().trim_start_matches('/');
    if name == names::METADATA_FILE {
        return Ok(Target::Metadata);
    }
    if let Some(entry) = Entry::parse(name) {
        return Ok(Target::File(entry));
    }
    if !name.contains('/')
        && (name.ends_with(".patch") || name.ends_with(".diff"))
        && names::valid_relative(name)
    {
        return Ok(Target::File(Entry::Patch(name.to_owned())));
    }
    Err(ContentError::BadRequest(format!(
        "cannot tell where {name} goes; name it MODULE.bazel, source.json, metadata.json, patches/<file> or overlay/<file>"
    )))
}

#[async_trait]
impl Format for Bazel {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = names::route(&request.path);
        match (request.method.clone(), route) {
            (Method::GET | Method::HEAD, Route::Root) => directory_page(&ctx),
            (Method::GET | Method::HEAD, Route::Registry) => {
                self.read_registry(&ctx, &request).await
            }
            (Method::GET | Method::HEAD, Route::Metadata(module)) => {
                self.read_metadata(&ctx, &request, &module).await
            }
            (Method::GET | Method::HEAD, Route::File(file)) => {
                self.download(&ctx, &request, &file).await
            }
            (Method::GET | Method::HEAD, Route::Other) => {
                Err(ContentError::not_found(request.path.clone()))
            }
            (Method::PUT, _) => self.put(&ctx, request).await,
            (Method::DELETE, route) => self.delete(&ctx, route).await,
            (method, _) => Err(not_allowed(&method, ctx.kind())),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match names::route(path) {
            Route::File(file) => Ok(self.file(ctx, &file).await?.map(|(_, found)| found)),
            route @ (Route::Registry | Route::Metadata(_)) => Ok(self
                .stored_document(ctx, &route)
                .await?
                .map(|(_, found)| found)),
            Route::Root | Route::Other => Ok(None),
        }
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: None,
            generated: &[REGISTRY_PATH],
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: true,
            component_fields: vec![
                UploadField::text("module", false)
                    .describe("Name of the module, for example rules_cc")
                    .in_group("Module"),
                UploadField::text("version", true)
                    .describe("Version of the module, for example 0.0.9; every file but metadata.json belongs to a version")
                    .in_group("Module"),
            ],
            asset_fields: vec![
                UploadField::text("filename", true).describe(
                    "Where the file goes: MODULE.bazel, source.json, metadata.json, patches/<file> or overlay/<file>; a .patch or .diff file without a path goes to patches",
                ),
                UploadField::file("asset"),
            ],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is a {} repository; upload modules to a hosted Bazel repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        hosted::may_add(&ctx)?;
        let field = |name: &str| {
            form.field(&format!("{ID}.{name}"))
                .or_else(|| form.field(name))
                .map(str::trim)
        };
        let module = field("module")
            .ok_or_else(|| ContentError::BadRequest(format!("{ID}.module is required")))?;
        if !names::valid_module(module) {
            return Err(ContentError::BadRequest(format!(
                "{module} is not a module name; use lowercase letters, digits, '.', '-' and '_', starting with a letter"
            )));
        }
        let mut planned = Vec::new();
        for (name, file) in &form.files {
            let Some(index) = name
                .strip_prefix(&format!("{ID}.asset"))
                .or_else(|| name.strip_prefix("asset"))
            else {
                continue;
            };
            let filename = field(&format!("asset{index}.filename"))
                .map(str::to_owned)
                .or_else(|| file.filename.clone())
                .ok_or_else(|| ContentError::BadRequest(format!("{name} needs a file name")))?;
            planned.push(Planned {
                target: target_of(&filename)?,
                file,
            });
        }
        if planned.is_empty() {
            return Err(ContentError::BadRequest(
                "the upload holds no files".to_owned(),
            ));
        }
        planned.sort_by_key(Planned::rank);
        let version = field("version");
        let mut files = Vec::new();
        for item in &planned {
            if let Target::File(entry) = &item.target {
                let version = version.ok_or_else(|| {
                    ContentError::BadRequest(format!(
                        "{ID}.version is required for {}",
                        entry.relative()
                    ))
                })?;
                let file = VersionFile::new(module, version, entry.clone()).ok_or_else(|| {
                    ContentError::BadRequest(format!("{version} is not a valid module version"))
                })?;
                hosted::ensure_new(&ctx, &file).await?;
                files.push(file);
            }
        }
        let mut files = files.into_iter();
        for item in planned {
            let body = UploadForm::stream(item.file).await?;
            match item.target {
                Target::Metadata => {
                    hosted::put_metadata(&ctx, module, body).await?;
                }
                Target::File(_) => {
                    let file = files.next().expect("every file target was resolved");
                    hosted::put_file(&ctx, &file, body).await?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uploaded_files_find_their_place_by_name() {
        let entry = |name: &str| match target_of(name).unwrap() {
            Target::File(entry) => entry.relative(),
            Target::Metadata => "metadata".to_owned(),
        };
        assert_eq!(entry("MODULE.bazel"), "MODULE.bazel");
        assert_eq!(entry("/source.json"), "source.json");
        assert_eq!(entry("metadata.json"), "metadata");
        assert_eq!(entry("patches/fix.patch"), "patches/fix.patch");
        assert_eq!(entry("overlay/sub/BUILD.bazel"), "overlay/sub/BUILD.bazel");
        assert_eq!(
            entry("module_dot_bazel.patch"),
            "patches/module_dot_bazel.patch"
        );
        assert_eq!(entry("change.diff"), "patches/change.diff");
        for bad in ["BUILD.bazel", "notes.txt", "../x.patch", "a b.patch", ""] {
            assert!(target_of(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn uploads_go_in_the_order_that_keeps_a_version_whole() {
        let file = UploadedFile {
            filename: None,
            path: std::path::PathBuf::new(),
            size: 0,
        };
        let mut planned = [
            Planned {
                target: Target::File(Entry::Source),
                file: &file,
            },
            Planned {
                target: Target::File(Entry::Module),
                file: &file,
            },
            Planned {
                target: Target::File(Entry::Patch("p".to_owned())),
                file: &file,
            },
            Planned {
                target: Target::Metadata,
                file: &file,
            },
        ];
        planned.sort_by_key(Planned::rank);
        let ranks: Vec<u8> = planned.iter().map(Planned::rank).collect();
        assert_eq!(ranks, [0, 1, 2, 3]);
    }

    #[test]
    fn only_hosted_repositories_take_writes() {
        let allow = |kind| match not_allowed(&Method::PUT, kind) {
            ContentError::MethodNotAllowed { allow, .. } => allow,
            other => panic!("{other}"),
        };
        assert_eq!(allow(RepositoryType::Hosted), "GET,HEAD,PUT,DELETE");
        assert_eq!(allow(RepositoryType::Proxy), "GET,HEAD,DELETE");
        assert_eq!(allow(RepositoryType::Group), "GET,HEAD");
    }
}
