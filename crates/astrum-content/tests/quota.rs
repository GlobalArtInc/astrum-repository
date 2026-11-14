use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use astrum_blob::ContentStream;
use astrum_content::store::{self, PutAsset};
use astrum_content::transfer::{Conflict, Status};
use astrum_content::{ContentError, usage};
use astrum_repo::config::WritePolicy;
use bytes::Bytes;
use futures_util::StreamExt;

mod support;
use support::*;

fn chunks(count: usize, size: usize, polled: Arc<AtomicUsize>) -> ContentStream {
    futures_util::stream::iter(0..count)
        .map(move |_| {
            polled.fetch_add(1, Ordering::SeqCst);
            Ok(Bytes::from(vec![b'x'; size]))
        })
        .boxed()
}

fn blob_files(world: &World) -> usize {
    fn walk(dir: &Path) -> usize {
        std::fs::read_dir(dir).map_or(0, |entries| {
            entries
                .flatten()
                .map(|entry| match entry.path().is_dir() {
                    true => walk(&entry.path()),
                    false => 1,
                })
                .sum()
        })
    }
    walk(&world.dir.path().join("blobs/default/content"))
}

async fn settled(world: &World, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while blob_files(world) != expected && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(blob_files(world), expected);
}

async fn bytes_used(ctx: &astrum_content::Ctx) -> i64 {
    usage::current(ctx.db(), ctx.name())
        .await
        .unwrap()
        .unwrap()
        .bytes
}

#[tokio::test]
async fn an_oversized_body_is_refused_before_it_is_stored() {
    let world = world().await;
    let capped = capped_repository(&world, "capped", 100).await;

    let polled = Arc::new(AtomicUsize::new(0));
    let declared = PutAsset::hosted(&capped, "/declared.bin").declared_size(Some(500));
    let error = store::put(&capped, declared, chunks(10, 50, polled.clone()))
        .await
        .unwrap_err();
    assert!(
        matches!(error, ContentError::QuotaExceeded { .. }),
        "{error}"
    );
    assert_eq!(
        polled.load(Ordering::SeqCst),
        0,
        "a declared size is refused unread"
    );
    assert_eq!(blob_files(&world), 0);

    let counted = PutAsset::hosted(&capped, "/counted.bin");
    let error = store::put(&capped, counted, chunks(10, 50, polled.clone()))
        .await
        .unwrap_err();
    assert!(
        matches!(error, ContentError::QuotaExceeded { .. }),
        "{error}"
    );
    assert!(
        polled.load(Ordering::SeqCst) < 10,
        "the body stops at the limit"
    );
    assert_eq!(blob_files(&world), 0);
    assert_eq!(bytes_used(&capped).await, 0);
}

#[tokio::test]
async fn a_refused_record_discards_its_blob() {
    let world = world().await;
    let capped = capped_repository(&world, "capped", 100).await;
    store::put(
        &capped,
        PutAsset::hosted(&capped, "/a.bin"),
        chunks(1, 60, Arc::default()),
    )
    .await
    .unwrap();
    let written = store::write_blob(&capped, chunks(1, 60, Arc::default()))
        .await
        .unwrap();
    assert_eq!(blob_files(&world), 2);
    let error = store::adopt(&capped, PutAsset::hosted(&capped, "/b.bin"), &written)
        .await
        .unwrap_err();
    assert!(
        matches!(error, ContentError::QuotaExceeded { .. }),
        "{error}"
    );
    settled(&world, 1).await;
}

#[tokio::test]
async fn a_transfer_batch_fits_exactly_and_is_refused_one_byte_over() {
    let world = world().await;
    let source = repository(&world, "src", "default", WritePolicy::Allow).await;
    seed(&source, "/g/a.txt", b"alpha").await;
    seed(&source, "/g/b.txt", b"bravo!").await;
    let bundle = bundle_of(&source).await;
    let baseline = blob_files(&world);

    let tight = capped_repository(&world, "tight", 10).await;
    let error = run(&source, &tight, &bundle, Conflict::Fail, false)
        .await
        .unwrap_err();
    match error {
        ContentError::QuotaExceeded {
            limit_bytes,
            used_bytes,
            ..
        } => assert_eq!((limit_bytes, used_bytes), (10, 0)),
        other => panic!("{other}"),
    }
    assert_eq!(store::count(tight.db(), "tight").await.unwrap(), 0);
    assert_eq!(bytes_used(&tight).await, 0);
    settled(&world, baseline).await;
    assert_eq!(bytes_used(&source).await, 11, "the source is untouched");

    let exact = capped_repository(&world, "exact", 11).await;
    let outcome = run(&source, &exact, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(bytes_used(&exact).await, 11);
    let again = run(&source, &exact, &bundle, Conflict::Fail, false)
        .await
        .unwrap();
    assert_eq!(
        again.status,
        Status::Unchanged,
        "a no-op copy passes at the limit"
    );
}

#[tokio::test]
async fn the_write_policy_refusal_wins_over_the_quota_refusal() {
    let world = world().await;
    let capped = capped_repository(&world, "capped", 100).await;
    let racing = capped.clone();
    let body: ContentStream = futures_util::stream::once(async move {
        for (path, size) in [("/b.bin", 40), ("/a.bin", 60)] {
            store::put(
                &racing,
                PutAsset::hosted(&racing, path),
                chunks(1, size, Arc::default()),
            )
            .await
            .unwrap();
        }
        Ok(Bytes::from(vec![b'x'; 90]))
    })
    .boxed();
    let mut once = PutAsset::hosted(&capped, "/a.bin");
    once.write_policy = Some(WritePolicy::AllowOnce);
    let error = store::put(&capped, once, body).await.unwrap_err();
    assert!(matches!(error, ContentError::Conflict(_)), "{error}");
    assert_eq!(bytes_used(&capped).await, 100);
}
