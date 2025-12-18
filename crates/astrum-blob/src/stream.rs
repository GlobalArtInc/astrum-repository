use std::io;

use bytes::BytesMut;
use futures_util::StreamExt;
use futures_util::stream;
use tokio::io::AsyncReadExt;

use crate::ContentStream;

const CHUNK: usize = 64 * 1024;

pub(crate) fn from_file(file: tokio::fs::File, length: u64) -> ContentStream {
    stream::try_unfold((file, length), |(mut file, left)| async move {
        if left == 0 {
            return Ok(None);
        }
        let want = left.min(CHUNK as u64) as usize;
        let mut buffer = BytesMut::zeroed(want);
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the blob ended before its recorded size",
            ));
        }
        buffer.truncate(read);
        Ok(Some((buffer.freeze(), (file, left - read as u64))))
    })
    .boxed()
}

pub(crate) fn holding<T: Send + 'static>(content: ContentStream, guard: T) -> ContentStream {
    stream::unfold(
        (content, Some(guard)),
        |(mut content, mut guard)| async move {
            match content.next().await {
                Some(item) => Some((item, (content, guard))),
                None => {
                    drop(guard.take());
                    None
                }
            }
        },
    )
    .boxed()
}
