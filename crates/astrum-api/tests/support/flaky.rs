use astrum_content::import::{Described, Imported};
use astrum_content::transfer::{GenericTransfer, Transfer};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format};
use astrum_format_raw::{Raw, import};
use async_trait::async_trait;
use axum::response::Response;
use futures_util::future::BoxFuture;

pub const REFRESH_FAILURE: &str = "the index is locked";

pub struct FlakyRaw {
    transfer: GenericTransfer,
}

impl FlakyRaw {
    pub fn new() -> Self {
        let transfer = GenericTransfer {
            plan: import::plan,
            describe: describe_imported,
            after: None,
            admit: None,
            replaceable: None,
            check_content_type: true,
            carried: &[],
            bundle_skip: None,
        }
        .after_fn(refresh);
        Self { transfer }
    }
}

fn describe_imported(file: &Imported) -> BoxFuture<'_, Result<Option<Described>, String>> {
    Box::pin(import::describe(file))
}

fn refresh(_: &Ctx) -> BoxFuture<'_, Result<(), ContentError>> {
    Box::pin(async { Err(ContentError::Unavailable(REFRESH_FAILURE.to_owned())) })
}

#[async_trait]
impl Format for FlakyRaw {
    fn id(&self) -> &'static str {
        "raw"
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        Raw.handle(ctx, request).await
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        Raw.get(ctx, path).await
    }

    fn transfer(&self) -> Option<&dyn Transfer> {
        Some(&self.transfer)
    }
}
