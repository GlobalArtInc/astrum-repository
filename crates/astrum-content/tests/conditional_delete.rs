use astrum_blob::content_of;
use astrum_content::ContentError;
use astrum_content::store::{self, PutAsset};
use astrum_repo::config::WritePolicy;

mod support;
use support::*;

#[tokio::test]
async fn a_component_goes_only_when_its_assets_are_exactly_the_expected_ones() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"alpha").await;
    seed(&dev, "/g/b.txt", b"bravo").await;
    let (a, b) = (row(&dev, "/g/a.txt").await, row(&dev, "/g/b.txt").await);
    let component = a.component_id.unwrap();
    let both = vec![
        (a.path.clone(), a.sha256.to_uppercase()),
        (b.path.clone(), b.sha256.clone()),
    ];
    let wrong = vec![
        (a.path.clone(), a.sha256.clone()),
        (b.path.clone(), a.sha256.clone()),
    ];
    let mut extra = both.clone();
    extra.push(("/g/c.txt".to_owned(), a.sha256.clone()));
    for (label, expected) in [
        ("fewer", both[..1].to_vec()),
        ("another digest", wrong),
        ("more", extra),
    ] {
        let refused =
            store::delete_component_matching(&dev.content, component, &expected, None).await;
        assert!(
            matches!(refused, Err(ContentError::Conflict(_))),
            "{label}: {refused:?}"
        );
        assert_eq!(
            store::assets_of(dev.db(), component).await.unwrap().len(),
            2
        );
    }
    assert!(
        store::delete_component_matching(&dev.content, component, &both, None)
            .await
            .unwrap()
    );
    assert!(
        store::assets_of(dev.db(), component)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        !store::delete_component_matching(&dev.content, component, &both, None)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn an_asset_goes_only_while_it_holds_the_content_that_was_seen() {
    let world = world().await;
    let dev = repository(&world, "dev", "default", WritePolicy::Allow).await;
    seed(&dev, "/g/a.txt", b"first").await;
    let seen = row(&dev, "/g/a.txt").await;
    store::put(
        &dev,
        PutAsset::hosted(&dev, "/g/a.txt").component(key()),
        content_of(&b"second"[..]),
    )
    .await
    .unwrap();
    let refused = store::delete_asset_if(&dev.content, &seen).await;
    assert!(
        matches!(refused, Err(ContentError::Conflict(_))),
        "{refused:?}"
    );
    let current = row(&dev, "/g/a.txt").await;
    assert_ne!(current.sha256, seen.sha256);
    store::delete_asset_if(&dev.content, &current)
        .await
        .unwrap();
    assert!(
        store::find(dev.db(), "dev", "/g/a.txt")
            .await
            .unwrap()
            .is_none()
    );
    let gone = store::delete_asset_if(&dev.content, &current).await;
    assert!(matches!(gone, Err(ContentError::NotFound(_))), "{gone:?}");
}
