use astrum_content::transfer::{Conflict, Outcome, Terms};
use astrum_content::{ContentError, Ctx};
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct ImageItem {
    pub image: String,
    pub reference: String,
    pub target_tags: Vec<String>,
    pub referrers: bool,
}

#[derive(Clone, Debug)]
pub struct ImageReport {
    pub outcome: Outcome,
    pub skipped_children: usize,
    pub untagged: bool,
    pub version: String,
    pub digest: String,
}

#[derive(Debug)]
pub enum ImageError {
    Plan(ContentError),
    Denied(String),
    Run(ContentError),
}

pub type Admit<'a> = &'a (dyn Fn(&[String]) -> Result<(), String> + Send + Sync);

pub type Approve<'a> = &'a (dyn Fn(&Outcome) -> Result<Conflict, String> + Send + Sync);

#[async_trait]
pub trait ImageTransfers: Send + Sync {
    async fn transfer(
        &self,
        source: &Ctx,
        target: &Ctx,
        item: &ImageItem,
        terms: Terms,
        admit: Admit<'_>,
        approve: Approve<'_>,
    ) -> Result<ImageReport, ImageError>;

    async fn remove_tag(
        &self,
        source: &Ctx,
        image: &str,
        tag: &str,
        digest: &str,
    ) -> Result<(), ContentError>;
}
