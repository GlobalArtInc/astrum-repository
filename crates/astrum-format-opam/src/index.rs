use std::io::{Seek, SeekFrom};

use astrum_content::{Asset, ContentError, Ctx};
use flate2::Compression;
use flate2::write::GzEncoder;
use futures_util::TryStreamExt;
use tokio::sync::mpsc;

use crate::names::REPO_TEXT;

pub const MAX_STORED_FILE: usize = 1024 * 1024;
pub const MAX_TOTAL: usize = 32 * 1024 * 1024;

const QUEUE: usize = 4;

struct Item {
    path: String,
    mtime: u64,
    data: Vec<u8>,
}

fn append(
    builder: &mut tar::Builder<GzEncoder<std::fs::File>>,
    item: &Item,
) -> std::io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_size(item.data.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(item.mtime);
    header.set_uid(0);
    header.set_gid(0);
    builder.append_data(&mut header, &item.path, item.data.as_slice())
}

async fn read_blob(ctx: &Ctx, found: &Asset, budget: &mut usize) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(format!(
            "the blob of {} is missing",
            found.path
        ))));
    };
    let mut stream = content.stream;
    let mut data = Vec::with_capacity(found.size.clamp(0, MAX_STORED_FILE as i64) as usize);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if data.len() + chunk.len() > MAX_STORED_FILE {
            return Err(ContentError::internal(std::io::Error::other(format!(
                "{} is larger than a package file may be",
                found.path
            ))));
        }
        if chunk.len() > *budget {
            return Err(ContentError::internal(std::io::Error::other(format!(
                "the package files are larger than the {MAX_TOTAL} bytes an index may read"
            ))));
        }
        *budget -= chunk.len();
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

fn relative(path: &str) -> &str {
    path.trim_start_matches('/')
}

pub async fn build(ctx: &Ctx, assets: &[Asset]) -> Result<std::fs::File, ContentError> {
    let (sender, mut receiver) = mpsc::channel::<Item>(QUEUE);
    let writer = tokio::task::spawn_blocking(move || -> std::io::Result<std::fs::File> {
        let file = tempfile::tempfile()?;
        let mut builder = tar::Builder::new(GzEncoder::new(file, Compression::default()));
        while let Some(item) = receiver.blocking_recv() {
            append(&mut builder, &item)?;
        }
        let mut file = builder.into_inner()?.finish()?;
        file.seek(SeekFrom::Start(0))?;
        Ok(file)
    });
    let produced = produce(ctx, assets, &sender).await;
    drop(sender);
    let written = writer.await.map_err(ContentError::internal)?;
    match (produced, written) {
        (Ok(()), Ok(file)) => Ok(file),
        (Ok(()), Err(error)) => Err(ContentError::internal(error)),
        (Err(error), Ok(_)) => Err(error),
        (Err(error), Err(written)) => Err(ContentError::internal(std::io::Error::other(format!(
            "{error}; the index writer failed: {written}"
        )))),
    }
}

async fn produce(
    ctx: &Ctx,
    assets: &[Asset],
    sender: &mpsc::Sender<Item>,
) -> Result<(), ContentError> {
    let mut budget = MAX_TOTAL;
    let stopped = || ContentError::internal(std::io::Error::other("the index writer stopped"));
    let first = assets
        .iter()
        .map(|found| found.blob_updated_at.timestamp().max(0) as u64)
        .max()
        .unwrap_or(0);
    sender
        .send(Item {
            path: "repo".to_owned(),
            mtime: first,
            data: REPO_TEXT.as_bytes().to_vec(),
        })
        .await
        .map_err(|_| stopped())?;
    for found in assets {
        let data = read_blob(ctx, found, &mut budget).await?;
        sender
            .send(Item {
                path: relative(&found.path).to_owned(),
                mtime: found.blob_updated_at.timestamp().max(0) as u64,
                data,
            })
            .await
            .map_err(|_| stopped())?;
    }
    Ok(())
}
