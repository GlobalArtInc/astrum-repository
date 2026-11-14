mod support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_content::transfer::{Bundle, BundleAsset, Outcome};
use astrum_outbound::Snapshot;
use astrum_replication::protocol::{BLOBS_MISSING, InventoryCompareReply, InventoryKey, Tombstone};
use astrum_replication::{Client, Error, Remote};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, GuardedResolver, Unverified};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use serde_json::json;
use support::{Forced, INSTANCE, TOKEN, build, connected, guard_off, outcome, remote, start};

fn digest(seed: &str) -> String {
    seed.repeat(64)
}

fn bundle() -> Bundle {
    Bundle {
        format: "raw".to_owned(),
        namespace: String::new(),
        name: "app".to_owned(),
        version: "1".to_owned(),
        assets: vec![BundleAsset {
            path: "a.txt".to_owned(),
            kind: "asset".to_owned(),
            content_type: "text/plain".to_owned(),
            size: 6,
            sha1: "1".repeat(40),
            sha256: digest("a"),
            sha512: "5".repeat(128),
            md5: "d".repeat(32),
            attributes: json!({}),
        }],
        meta: json!({}),
        origin: Some("origin-1".to_owned()),
    }
}

#[tokio::test]
async fn every_route_is_called_with_the_token_and_the_agreed_shape() {
    let (client, receiver, _) = connected().await;
    assert_eq!(client.max_concurrency(), 3);

    let checked = client.check().await.unwrap();
    assert_eq!(checked.instance_id, INSTANCE);
    assert_eq!(checked.formats, ["raw", "maven2"]);

    let wanted = vec![digest("a"), digest("b")];
    assert_eq!(client.missing("a b/c", &wanted).await.unwrap(), wanted);

    let parts = vec![Ok(Bytes::from("abc")), Ok(Bytes::from("def"))];
    client
        .put_blob("a b/c", &digest("a"), 6, stream::iter(parts))
        .await
        .unwrap();

    let committed = client.commit("a b/c", &bundle()).await.unwrap();
    assert_eq!(
        committed,
        serde_json::from_value::<Outcome>(outcome()).unwrap()
    );

    let tombstone = Tombstone {
        namespace: String::new(),
        name: "app".to_owned(),
        version: "1".to_owned(),
        fingerprint: digest("f"),
        origin: "origin-1".to_owned(),
    };
    client.tombstone("a b/c", &tombstone).await.unwrap();

    let items = vec![InventoryKey {
        key: "app/1".to_owned(),
        fingerprint: "f".to_owned(),
    }];
    let differing = client.inventory_compare("a b/c", &items).await.unwrap();
    assert_eq!(
        differing,
        InventoryCompareReply {
            differing: vec!["app/1".to_owned()]
        }
    );
    let page = client.inventory("a b/c", Some("app/0"), 50).await.unwrap();
    assert_eq!(page.next.as_deref(), Some("app/1"));

    let log = receiver.log();
    let repository = "/service/rest/v1/replication/a%20b%2Fc";
    let put = &log[2];
    assert_eq!(
        (put.method.as_str(), put.path.as_str()),
        (
            "PUT",
            format!("{repository}/blobs/{}", digest("a")).as_str()
        )
    );
    assert_eq!(put.content_length.as_deref(), Some("6"));
    assert!(!put.chunked);
    assert_eq!(put.body, b"abcdef");
    assert_eq!(
        serde_json::from_slice::<Bundle>(&log[3].body).unwrap(),
        bundle()
    );
    assert_eq!(log[3].path, format!("{repository}/components"));
    assert_eq!(log[4].method, "DELETE");
    assert_eq!(log[6].query.as_deref(), Some("after=app%2F0&limit=50"));
    for entry in &log {
        assert_eq!(
            entry.authorization.as_deref(),
            Some(format!("Bearer {TOKEN}").as_str())
        );
        let elsewhere = format!(
            "{} {:?} {}",
            entry.path,
            entry.query,
            String::from_utf8_lossy(&entry.body)
        );
        assert!(!elsewhere.contains(TOKEN));
    }
}

#[tokio::test]
async fn a_redirect_is_refused_and_never_followed() {
    let (target, hits) = start().await;
    let (client, receiver, _) = connected().await;
    receiver.force_with(Forced {
        status: 302,
        headers: vec![(
            "location",
            format!("http://{target}/service/rest/v1/replication/capabilities"),
        )],
        body: Vec::new(),
        echo_authorization: false,
    });
    let error = client.capabilities().await.unwrap_err();
    assert!(
        matches!(&error, Error::Refused(text) if text == "the remote redirected"),
        "{error:?}"
    );
    assert!(hits.log().is_empty());
    assert_eq!(receiver.log().len(), 1);
}

#[tokio::test]
async fn private_ranges_open_per_remote_and_loopback_never_does() {
    let strict = Guard::new(&Default::default());
    for host in [
        "10.0.0.1",
        "172.16.0.1",
        "192.168.1.1",
        "[fd00::1]",
        "100.64.0.1",
    ] {
        let mut private = remote(&format!("https://{host}:8443"));
        private.allow_private_network = false;
        assert!(
            matches!(build(&private, &strict), Err(Error::Refused(_))),
            "{host}"
        );
        private.allow_private_network = true;
        assert!(build(&private, &strict).is_ok(), "{host}");
    }
    for host in [
        "127.0.0.1",
        "[::1]",
        "169.254.1.1",
        "169.254.169.254",
        "[fe80::1]",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "[fd00:ec2::254]",
    ] {
        let refused = remote(&format!("https://{host}:8443"));
        assert!(
            matches!(build(&refused, &strict), Err(Error::Refused(_))),
            "{host}"
        );
    }
    assert!(build(&remote("https://8.8.8.8"), &strict).is_ok());
}

#[tokio::test]
async fn a_name_that_resolves_to_loopback_is_refused_when_connecting() {
    let (address, receiver) = start().await;
    let client = build(
        &remote(&format!("http://localhost:{}", address.port())),
        &Guard::new(&Default::default()),
    )
    .unwrap();
    let error = client.capabilities().await.unwrap_err();
    assert!(
        matches!(&error, Error::Refused(text) if text.contains("SSRF")),
        "{error:?}"
    );
    assert!(receiver.log().is_empty());
}

#[tokio::test]
async fn an_unusable_address_is_refused_at_construction() {
    let strict = guard_off();
    let mut plain = remote("http://example.com");
    plain.allow_private_network = false;
    for (case, url) in [
        ("http without private networks", plain.url.as_str()),
        ("user info", "https://user:secret@example.com"),
        ("a path", "https://example.com/base"),
        ("a query", "https://example.com/?a=b"),
        ("another scheme", "ftp://example.com"),
        ("no URL", "not a url"),
    ] {
        let mut candidate = plain.clone();
        candidate.url = url.to_owned();
        let error = build(&candidate, &strict).unwrap_err();
        assert!(matches!(error, Error::Refused(_)), "{case}: {error:?}");
    }
}

#[tokio::test]
async fn statuses_are_classified_for_the_dispatcher() {
    let (client, receiver, _) = connected().await;
    for status in [500, 502, 503] {
        receiver.force(status, "busy");
        let error = client.capabilities().await.unwrap_err();
        assert!(error.is_retryable(), "{status}: {error:?}");
    }
    receiver.force_with(Forced {
        status: 429,
        headers: vec![("retry-after", "42".to_owned())],
        body: Vec::new(),
        echo_authorization: false,
    });
    let throttled = client.capabilities().await.unwrap_err();
    assert!(throttled.is_retryable());
    assert_eq!(throttled.retry_after(), Some(Duration::from_secs(42)));
    for status in [400, 401, 403, 404, 409, 413, 422] {
        receiver.force(status, json!({"code": "nope", "message": "no"}).to_string());
        let error = client.capabilities().await.unwrap_err();
        assert!(!error.is_retryable(), "{status}: {error:?}");
        assert!(
            matches!(&error, Error::Permanent { status: seen, code, .. } if *seen == status && code == "nope")
        );
    }
    let wanted = digest("c");
    receiver.force(
        409,
        json!({"code": BLOBS_MISSING, "message": "m", "missing": [wanted]}).to_string(),
    );
    match client.commit("dst", &bundle()).await.unwrap_err() {
        Error::BlobsMissing(list) => assert_eq!(list, vec![digest("c")]),
        other => panic!("{other:?}"),
    }
    receiver.force(200, vec![b' '; 17 * 1024 * 1024]);
    assert!(matches!(
        client.capabilities().await.unwrap_err(),
        Error::Refused(_)
    ));
}

#[tokio::test]
async fn a_connection_that_cannot_be_made_is_retryable() {
    let (address, _) = start().await;
    let client = build(&remote(&format!("http://{address}")), &guard_off()).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let closed = listener.local_addr().unwrap();
    drop(listener);
    let lost = build(&remote(&format!("http://{closed}")), &guard_off()).unwrap();
    assert!(client.capabilities().await.is_ok());
    assert!(lost.capabilities().await.unwrap_err().is_retryable());
}

#[tokio::test]
async fn a_protocol_or_an_instance_the_client_cannot_use_is_refused() {
    let (client, receiver, _) = connected().await;
    receiver.force(
        200,
        json!({"protocol": 2, "instanceId": INSTANCE, "version": "9", "formats": []}).to_string(),
    );
    assert!(
        matches!(client.check().await.unwrap_err(), Error::Refused(text) if text == "the remote speaks protocol 2")
    );
    receiver.force(
        200,
        json!({"protocol": 1, "instanceId": "x", "version": "9", "formats": []}).to_string(),
    );
    assert!(matches!(
        client.check().await.unwrap_err(),
        Error::Refused(_)
    ));
    receiver.force(200, "<html>");
    assert!(matches!(
        client.capabilities().await.unwrap_err(),
        Error::Refused(_)
    ));
    receiver.force(200, "");
    client
        .put_blob("dst", &digest("a"), 0, stream::empty())
        .await
        .unwrap();
    receiver.force(204, "");
    assert!(matches!(
        client.put_blob("dst", &digest("a"), 0, stream::empty()).await.unwrap_err(),
        Error::Refused(text) if text.contains("201")
    ));
}

#[tokio::test]
async fn the_token_is_in_no_error_and_no_debug_output() {
    let (client, receiver, _) = connected().await;
    receiver.force_with(Forced {
        status: 401,
        headers: Vec::new(),
        body: Vec::new(),
        echo_authorization: true,
    });
    let echoed = client.capabilities().await.unwrap_err();
    let shown = format!(
        "{echoed} {echoed:?} {client:?} {:?}",
        remote("https://example.com")
    );
    assert!(!shown.contains(TOKEN), "{shown}");
    assert!(shown.contains("[redacted]"));
    let bad = Client::new(
        &remote("https://example.com"),
        "line\nbreak",
        &Default::default(),
        &astrum_security::SecretBox::from_key([1; 32]),
        &guard_off(),
    )
    .unwrap_err();
    assert!(!format!("{bad} {bad:?}").contains("break"));
}

#[tokio::test]
async fn the_digests_asked_about_are_bounded_and_checked() {
    let (client, receiver, _) = connected().await;
    let too_many: Vec<String> = (0..1001).map(|_| digest("a")).collect();
    assert!(matches!(
        client.missing("dst", &too_many).await.unwrap_err(),
        Error::Refused(_)
    ));
    assert!(matches!(
        client
            .missing("dst", &["XYZ".to_owned()])
            .await
            .unwrap_err(),
        Error::Refused(_)
    ));
    assert!(matches!(
        client.missing("..", &[]).await.unwrap_err(),
        Error::Refused(_)
    ));
    let in_range: Vec<String> = (0..1000).map(|_| digest("a")).collect();
    assert_eq!(client.missing("dst", &in_range).await.unwrap().len(), 1000);
    assert_eq!(receiver.log().len(), 1);
}

#[tokio::test]
async fn an_upload_takes_as_long_as_the_rate_allows() {
    let (address, _receiver) = start().await;
    let mut limited: Remote = remote(&format!("http://{address}"));
    limited.max_bytes_per_second = Some(100_000);
    let client = build(&limited, &guard_off()).unwrap();
    let chunks: Vec<std::io::Result<Bytes>> = (0..10)
        .map(|_| Ok(Bytes::from(vec![1u8; 10_000])))
        .collect();
    let started = Instant::now();
    client
        .put_blob("dst", &digest("a"), 100_000, stream::iter(chunks))
        .await
        .unwrap();
    let taken = started.elapsed().as_secs_f64();
    assert!((0.9..=1.5).contains(&taken), "{taken}");
}

#[tokio::test]
async fn a_bundle_is_asked_for_by_its_key_and_a_blob_comes_down_at_the_rate_allowed() {
    let (address, receiver) = start().await;
    let mut limited: Remote = remote(&format!("http://{address}"));
    limited.max_bytes_per_second = Some(100_000);
    let client = build(&limited, &guard_off()).unwrap();
    let fetched = client.bundle("a b/c", "ns", "app", "1").await.unwrap();
    assert_eq!(
        (fetched.name.as_str(), fetched.origin.as_deref()),
        ("app", Some("origin-1"))
    );
    let started = Instant::now();
    let download = client.blob("a b/c", &digest("a")).await.unwrap();
    assert_eq!(download.length, Some(100_000));
    let (body, failures) = download.into_parts();
    let chunks: Vec<std::io::Result<Bytes>> = body.collect().await;
    let total: usize = chunks
        .iter()
        .map(|chunk| chunk.as_ref().unwrap().len())
        .sum();
    assert_eq!(total, 100_000);
    assert!(failures.take().is_none());
    let taken = started.elapsed().as_secs_f64();
    assert!((0.9..=1.5).contains(&taken), "{taken}");
    let log = receiver.log();
    let repository = "/service/rest/v1/replication/a%20b%2Fc";
    assert_eq!(log[0].path, format!("{repository}/bundle"));
    assert_eq!(
        log[0].query.as_deref(),
        Some("namespace=ns&name=app&version=1")
    );
    assert_eq!(log[1].path, format!("{repository}/blobs/{}", digest("a")));
    for entry in &log {
        assert_eq!(
            entry.authorization.as_deref(),
            Some(format!("Bearer {TOKEN}").as_str())
        );
    }
}

#[tokio::test]
async fn a_pull_is_classified_like_a_push_and_a_blob_answer_must_be_a_plain_200() {
    let (client, receiver, _) = connected().await;
    receiver.force(503, "busy");
    assert!(
        client
            .bundle("dst", "", "app", "1")
            .await
            .unwrap_err()
            .is_retryable()
    );
    assert!(
        client
            .blob("dst", &digest("a"))
            .await
            .err()
            .unwrap()
            .is_retryable()
    );
    receiver.force(
        404,
        json!({"code": "component_not_found", "message": "gone"}).to_string(),
    );
    assert!(matches!(
        client.bundle("dst", "", "app", "1").await.unwrap_err(),
        Error::Permanent { status: 404, code, .. } if code == "component_not_found"
    ));
    assert!(matches!(
        client.blob("dst", &digest("a")).await.err().unwrap(),
        Error::Permanent { status: 404, .. }
    ));
    receiver.force(206, "partial");
    assert!(matches!(
        client.blob("dst", &digest("a")).await.err().unwrap(),
        Error::Refused(_)
    ));
    assert!(matches!(
        client.blob("dst", "not-a-digest").await.err().unwrap(),
        Error::Refused(_)
    ));
}

#[tokio::test]
async fn clear_text_never_reaches_a_public_address() {
    let public = remote("http://203.0.113.7:8081");
    let error = build(&public, &guard_off()).unwrap_err();
    assert!(
        matches!(&error, Error::Refused(text) if text.contains("private network")),
        "{error:?}"
    );
    assert!(build(&remote("http://10.0.0.5:8081"), &guard_off()).is_ok());
    assert!(build(&remote("https://203.0.113.7"), &guard_off()).is_ok());

    let guard = Arc::new(guard_off());
    let plain = GuardedResolver::with_private_ranges(guard.clone());
    let clear = GuardedResolver::with_private_ranges(guard).for_cleartext();
    assert!(plain.verify("203.0.113.7", 80).await.is_ok());
    assert!(matches!(
        clear.verify("203.0.113.7", 80).await,
        Err(Unverified::Blocked(_))
    ));
    assert!(clear.verify("10.1.2.3", 80).await.is_ok());
}

#[tokio::test]
async fn a_remote_behind_the_proxy_is_checked_here_before_the_proxy_is_asked() {
    let through_proxy = Snapshot {
        http: serde_json::from_value(
            json!({"httpProxy": {"enabled": true, "host": "127.0.0.1", "port": "9"}}),
        )
        .unwrap(),
        ..Default::default()
    };
    let secrets = SecretBox::from_key([7; 32]);
    let build_with = |guard: &Guard| {
        Client::new(
            &remote("http://localhost:8081"),
            TOKEN,
            &through_proxy,
            &secrets,
            guard,
        )
        .unwrap()
    };
    let refused = build_with(&Guard::new(&Default::default()))
        .capabilities()
        .await
        .unwrap_err();
    assert!(
        matches!(&refused, Error::Refused(text) if text.contains("SSRF")),
        "{refused:?}"
    );
    let allowed = build_with(&guard_off()).capabilities().await.unwrap_err();
    assert!(allowed.is_retryable(), "{allowed:?}");
}
