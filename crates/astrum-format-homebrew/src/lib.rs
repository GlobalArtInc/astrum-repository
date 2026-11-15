mod names;
mod remote;

use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;

use crate::names::Item;

pub const ID: &str = "homebrew";

const ALLOW_READ: &str = "GET,HEAD";

pub struct Homebrew {
    registry: String,
    flights: remote::Flights,
}

impl Default for Homebrew {
    fn default() -> Self {
        Self::with_registry(remote::REGISTRY)
    }
}

impl Homebrew {
    pub fn with_registry(registry: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            flights: remote::Flights::default(),
        }
    }

    async fn asset(
        &self,
        ctx: &Ctx,
        path: &str,
        item: &Item,
    ) -> Result<Option<Asset>, ContentError> {
        remote::fetch(ctx, &self.flights, &self.registry, path, item).await
    }
}

fn is_read(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD)
}

#[async_trait]
impl Format for Homebrew {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return Err(ContentError::not_found(
                "Homebrew repositories are proxies only",
            ));
        }
        if !is_read(&request.method) {
            return Err(ContentError::MethodNotAllowed {
                method: request.method.to_string(),
                allow: ALLOW_READ,
            });
        }
        let item = names::parse(&request.path)
            .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
        ctx.require(View::Read, &request.path)?;
        let found = self
            .asset(&ctx, &request.path, &item)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("{} was not found", request.path)))?;
        let options = match item {
            Item::Blob { .. } => Serve::download(),
            Item::Api { .. } | Item::Manifest { .. } => Serve::default(),
        };
        serve(&ctx, &found, &request.method, &request.headers, options).await
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return Ok(None);
        }
        match names::parse(path) {
            Some(item) => self.asset(ctx, path, &item).await,
            None => Ok(None),
        }
    }
}
