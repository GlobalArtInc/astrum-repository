use std::io;

use astrum_blob::ContentStream;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bytes::Bytes;
use futures_util::TryStreamExt;

/// Decodes a part sent with `Content-Transfer-Encoding: base64`, as `ansible-galaxy` sends the
/// archive (Python's MIME classes encode application parts that way), without holding it whole.
pub fn decode_base64(stream: ContentStream) -> ContentStream {
    Box::pin(futures_util::stream::try_unfold(
        (stream, Vec::new(), false),
        |(mut stream, mut pending, finished): (ContentStream, Vec<u8>, bool)| async move {
            if finished {
                return Ok(None);
            }
            loop {
                match stream.try_next().await? {
                    Some(chunk) => {
                        pending.extend(chunk.iter().copied().filter(|b| !b.is_ascii_whitespace()));
                        let usable = pending.len() / 4 * 4;
                        if usable == 0 {
                            continue;
                        }
                        let decoded = STANDARD
                            .decode(&pending[..usable])
                            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                        pending.drain(..usable);
                        return Ok(Some((Bytes::from(decoded), (stream, pending, false))));
                    }
                    None if pending.is_empty() => return Ok(None),
                    None => {
                        let decoded = STANDARD
                            .decode(&pending)
                            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                        return Ok(Some((Bytes::from(decoded), (stream, Vec::new(), true))));
                    }
                }
            }
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn base64_parts_decode_across_chunks_and_lines() {
        let data: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
        let encoded = STANDARD.encode(&data);
        let wrapped: String = encoded
            .as_bytes()
            .chunks(76)
            .map(|line| format!("{}\r\n", std::str::from_utf8(line).unwrap()))
            .collect();
        let chunks: Vec<io::Result<Bytes>> = wrapped
            .as_bytes()
            .chunks(7)
            .map(|chunk| Ok(Bytes::copy_from_slice(chunk)))
            .collect();
        let stream: ContentStream = futures_util::stream::iter(chunks).boxed();
        let decoded: Vec<Bytes> = decode_base64(stream).try_collect().await.unwrap();
        assert_eq!(decoded.concat(), data);
        let broken: ContentStream =
            futures_util::stream::iter(vec![Ok(Bytes::from_static(b"@@@@"))]).boxed();
        assert!(decode_base64(broken).try_collect::<Vec<_>>().await.is_err());
    }
}
