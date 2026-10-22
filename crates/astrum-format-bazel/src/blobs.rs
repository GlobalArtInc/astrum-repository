use astrum_blob::ContentStream;
use astrum_content::{Asset, ContentError, Ctx};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};

pub async fn read_all(ctx: &Ctx, asset: &Asset, limit: usize) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as usize > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is larger than {limit} bytes",
            asset.path
        )));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::internal(std::io::Error::other(
            "a blob of the repository is missing",
        )));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

pub async fn collect(
    mut body: ContentStream,
    limit: usize,
    what: &str,
) -> Result<Vec<u8>, ContentError> {
    let mut collected = Vec::new();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(|error| {
            ContentError::BadRequest(format!("the request body cannot be read: {error}"))
        })?;
        if collected.len() + chunk.len() > limit {
            return Err(ContentError::BadRequest(format!(
                "{what} is larger than {limit} bytes"
            )));
        }
        collected.extend_from_slice(&chunk);
    }
    Ok(collected)
}

pub fn once(bytes: Vec<u8>) -> ContentStream {
    Box::pin(futures_util::stream::once(async move {
        Ok::<_, std::io::Error>(Bytes::from(bytes))
    }))
}
