use async_trait::async_trait;

use crate::store::Asset;
use crate::{ContentError, Ctx};

#[async_trait]
pub trait Gate: Send + Sync + 'static {
    async fn check(&self, ctx: &Ctx, asset: &Asset) -> Result<(), ContentError>;

    async fn check_digest(
        &self,
        ctx: &Ctx,
        repository: &str,
        digest: &str,
        reference: Option<&str>,
    ) -> Result<(), ContentError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Open;

#[async_trait]
impl Gate for Open {
    async fn check(&self, _ctx: &Ctx, _asset: &Asset) -> Result<(), ContentError> {
        Ok(())
    }

    async fn check_digest(
        &self,
        _ctx: &Ctx,
        _repository: &str,
        _digest: &str,
        _reference: Option<&str>,
    ) -> Result<(), ContentError> {
        Ok(())
    }
}
