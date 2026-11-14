use std::time::Duration;

use futures_util::StreamExt;

use super::*;
use crate::layers::fixtures::*;

fn reader_of(bytes: Vec<u8>, deadline: Instant) -> BlobReader {
    let stream: ContentStream =
        futures_util::stream::iter(vec![Ok::<Bytes, io::Error>(Bytes::from(bytes))]).boxed();
    BlobReader::new(stream, deadline)
}

async fn cut(bytes: Vec<u8>, limits: Limits, deadline: Instant) -> Result<LayerFacts, Failure> {
    let permit = SLOT.clone().acquire_owned().await.unwrap();
    assert_eq!(SLOT.available_permits(), 0);
    let reader = reader_of(bytes, deadline);
    let guard = Guard::new(Arc::new(AtomicBool::new(false)), deadline);
    let task =
        tokio::task::spawn_blocking(move || extract_blocking(permit, reader, &limits, guard));
    let outcome = task.await.unwrap();
    assert_eq!(SLOT.available_permits(), 1);
    outcome
}

#[tokio::test]
async fn the_process_semaphore_is_free_again_when_the_work_is_cut_or_finishes() {
    let mut hostile = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    hostile.extend(std::iter::repeat_n([0u8, 0, 0, 0xff, 0xff], 4000).flatten());
    let limits = Limits {
        stall_input: 1024,
        stall_output: 16,
        ..Limits::DEFAULT
    };
    let soon = Instant::now() + Duration::from_secs(60);
    let facts = cut(hostile.clone(), limits, soon).await.unwrap();
    assert!(facts.incomplete);

    let past = Instant::now() - Duration::from_secs(1);
    let failure = cut(hostile, Limits::DEFAULT, past).await;
    assert!(matches!(failure, Err(Failure::Deadline)), "{failure:?}");

    let facts = cut(alpine_layer(), Limits::DEFAULT, soon).await.unwrap();
    assert!(!facts.incomplete);
}
