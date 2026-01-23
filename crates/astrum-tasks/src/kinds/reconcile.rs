use async_trait::async_trait;

use super::compact::physical;
use super::{ALL, size};
use crate::kind::{Field, FieldType, Properties, TaskError, TaskKind};
use crate::run::Run;

pub const ID: &str = "blobstore.metrics.reconcile";
const PAGE: usize = 1000;

pub struct Reconcile;

#[async_trait]
impl TaskKind for Reconcile {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Repair - Recalculate blob store storage"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new("blobstoreName", FieldType::BlobStore, "Blob store")
                .help("The blob store whose blob count and size are counted again")
                .required()
                .with_all(),
        ]
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get("blobstoreName").map(String::as_str) {
            Some(ALL) | None => "counts the blobs of every blob store again".to_owned(),
            Some(name) => format!("counts the blobs of the blob store {name} again"),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let wanted = run.property("blobstoreName").unwrap_or(ALL).to_owned();
        let configs = run.content().blob_stores().configs().await?;
        let stores = physical(&configs, &wanted)?;
        let mut counted = 0usize;
        for name in stores.keys() {
            run.check()?;
            let store = run.content().blob_stores().get(name).await?;
            let before = store.totals().await?;
            let mut totals = astrum_blob::Totals::default();
            let mut after = None;
            loop {
                run.check()?;
                let page = store.list(after, PAGE).await?;
                let Some(last) = page.last() else {
                    break;
                };
                after = Some(last.id);
                totals.blob_count += page.len() as u64;
                totals.total_bytes += page.iter().map(|blob| blob.size).sum::<u64>();
            }
            run.content()
                .blob_stores()
                .metrics(name)
                .reset(totals)
                .await?;
            counted += 1;
            run.info(format!(
                "{name}: {} blobs, {} (recorded before: {} blobs, {})",
                totals.blob_count,
                size(totals.total_bytes as i64),
                before.blob_count,
                size(before.total_bytes as i64)
            ));
        }
        Ok(format!("counted the blobs of {counted} blob stores again"))
    }
}
