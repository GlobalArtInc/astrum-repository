use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, UploadField, UploadForm,
    UploadSpec, View, proxy, serve,
};
use astrum_repo::RepositoryType;
use astrum_repo::config::ContentDisposition;
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};

pub const ID: &str = "raw";
const ALLOW: &str = "GET,HEAD,PUT,DELETE";

pub struct Raw;

fn component(path: &str) -> ComponentKey {
    let parent = match path.rfind('/') {
        Some(0) | None => "/".to_owned(),
        Some(index) => path[..index].to_owned(),
    };
    ComponentKey {
        namespace: parent,
        name: path.to_owned(),
        ..Default::default()
    }
}

fn disposition(ctx: &Ctx) -> &'static str {
    match ctx
        .repository
        .attributes
        .raw
        .as_ref()
        .and_then(|raw| raw.content_disposition)
    {
        Some(ContentDisposition::Inline) => "inline",
        _ => "attachment",
    }
}

fn forwarded_query(ctx: &Ctx, query: Option<&str>) -> Option<String> {
    let raw = ctx.repository.attributes.raw.as_ref()?;
    if !raw.forward_query_parameters.unwrap_or(false) {
        return None;
    }
    let excluded = raw.excluded_query_parameters.clone().unwrap_or_default();
    let kept: Vec<&str> = query?
        .split('&')
        .filter(|pair| {
            let name = pair.split('=').next().unwrap_or_default();
            !name.is_empty() && !excluded.iter().any(|excluded| excluded == name)
        })
        .collect();
    (!kept.is_empty()).then(|| kept.join("&"))
}

impl Raw {
    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Asset, ContentError> {
        let path = &request.path;
        let found = match (ctx.kind(), forwarded_query(ctx, request.query.as_deref())) {
            (RepositoryType::Proxy, Some(query)) => {
                proxy::fetch(ctx, path, Some(&query), Freshness::Content, || {
                    PutAsset::cached(path.clone()).component(component(path))
                })
                .await?
            }
            _ => self.get(ctx, path).await?,
        };
        found.ok_or_else(|| ContentError::not_found(path.clone()))
    }
}

#[async_trait]
impl Format for Raw {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.clone();
        match request.method {
            Method::GET | Method::HEAD => {
                if request.is_directory() {
                    return Err(ContentError::not_found("You can't browse this way"));
                }
                ctx.require(View::Read, &path)?;
                let asset = self.read(&ctx, &request).await?;
                serve(
                    &ctx,
                    &asset,
                    &request.method,
                    &request.headers,
                    Serve::download().disposition(disposition(&ctx)),
                )
                .await
            }
            Method::PUT if ctx.kind() == RepositoryType::Hosted => {
                if request.is_directory() {
                    return Err(ContentError::BadRequest("a file needs a name".to_owned()));
                }
                let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
                ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
                let put = PutAsset::hosted(&ctx, path.clone()).component(component(&path));
                store::put(&ctx, put, request.into_stream()).await?;
                Ok(StatusCode::CREATED.into_response())
            }
            Method::DELETE if ctx.kind() != RepositoryType::Group => {
                ctx.require(View::Delete, &path)?;
                match store::delete(&ctx, &path).await? {
                    true => Ok(StatusCode::NO_CONTENT.into_response()),
                    false => Err(ContentError::not_found(path)),
                }
            }
            method => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match ctx.kind() {
                    RepositoryType::Hosted => ALLOW,
                    RepositoryType::Proxy => "GET,HEAD,DELETE",
                    RepositoryType::Group => "GET,HEAD",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        astrum_content::fetch(self, ctx, path, Freshness::Content, || {
            PutAsset::cached(path).component(component(path))
        })
        .await
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: true,
            component_fields: vec![
                UploadField::text("directory", false)
                    .describe("Destination for uploaded files (e.g. /path/to/files/)")
                    .in_group("Component attributes"),
            ],
            asset_fields: vec![
                UploadField::text("filename", false),
                UploadField::file("asset"),
            ],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        let directory = form
            .field("raw.directory")
            .ok_or_else(|| ContentError::BadRequest("raw.directory is required".to_owned()))?;
        let directory = format!("/{}", directory.trim().trim_matches('/'));
        let mut planned = Vec::new();
        for (field, file) in &form.files {
            let Some(index) = field.strip_prefix("raw.asset") else {
                continue;
            };
            let filename = form
                .field(&format!("raw.asset{index}.filename"))
                .map(str::to_owned)
                .or_else(|| file.filename.clone())
                .ok_or_else(|| ContentError::BadRequest(format!("{field} needs a file name")))?;
            let joined = format!(
                "{}/{}",
                directory.trim_end_matches('/'),
                filename.trim_start_matches('/')
            );
            planned.push((astrum_content::normalize(&joined)?, file));
        }
        if planned.is_empty() {
            return Err(ContentError::BadRequest(
                "the upload holds no files".to_owned(),
            ));
        }
        for (path, _) in &planned {
            ctx.require(View::Add, path)?;
        }
        for (path, file) in planned {
            let put = PutAsset::hosted(&ctx, path.clone()).component(component(&path));
            store::put(&ctx, put, UploadForm::stream(file).await?).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_a_component_named_by_its_path() {
        let key = component("/a/b/hello.txt");
        assert_eq!(key.namespace, "/a/b");
        assert_eq!(key.name, "/a/b/hello.txt");
        assert_eq!(component("/top.txt").namespace, "/");
    }
}
