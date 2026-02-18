mod support;

use astrum_blob::config::{self, GroupSettings, Settings, StoreConfig};
use astrum_blob::{ContentStream, FillPolicy, S3Bucket, S3Connection, S3Security, S3Settings};
use astrum_db::entity::{audit_event, blob_store, repository};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use astrum_testkit::s3::{ACCESS_KEY, S3Server, SECRET_KEY};
use axum::http::{Method, StatusCode, header};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const BLOBSTORES: &str = "/service/rest/v1/blobstores";

fn ids(reply: &support::Reply) -> Vec<String> {
    reply
        .json()
        .as_array()
        .unwrap_or_else(|| panic!("an array of errors: {}", reply.text()))
        .iter()
        .map(|error| error["id"].as_str().unwrap().to_owned())
        .collect()
}

fn content(bytes: &'static [u8]) -> ContentStream {
    astrum_blob::content_of(bytes)
}

async fn allow_loopback(fixture: &Fixture) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

async fn with_default(fixture: &Fixture) {
    assert!(
        astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
            .await
            .unwrap()
    );
}

async fn create_file(fixture: &Fixture, body: Value) -> support::Reply {
    fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/file"),
            Auth::Admin,
            Some(body),
        )
        .await
}

async fn audit_kinds(fixture: &Fixture, name: &str) -> Vec<String> {
    audit_event::Entity::find()
        .filter(audit_event::Column::Domain.eq("blobstore"))
        .filter(audit_event::Column::Context.eq(name))
        .order_by_asc(audit_event::Column::Id)
        .all(fixture.state.db().conn())
        .await
        .unwrap()
        .into_iter()
        .map(|event| {
            assert_eq!(event.initiator, "admin");
            event.kind
        })
        .collect()
}

async fn add_repository(fixture: &Fixture, name: &str, store: &str) {
    let now = chrono::Utc::now().fixed_offset();
    repository::Entity::insert(repository::ActiveModel {
        name: ActiveValue::Set(name.to_owned()),
        format: ActiveValue::Set("raw".to_owned()),
        kind: ActiveValue::Set("hosted".to_owned()),
        online: ActiveValue::Set(true),
        blob_store: ActiveValue::Set(Some(store.to_owned())),
        attributes: ActiveValue::Set(json!({})),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
    })
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
}

#[tokio::test]
async fn the_default_store_answers_like_the_reference() {
    let fixture = Fixture::new().await;
    with_default(&fixture).await;

    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/file/default"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json(), json!({"softQuota": null, "path": "default"}));

    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/default/quota-status"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        json!({
            "isViolation": false,
            "message": "Blob store default has no quota",
            "blobStoreName": "default"
        })
    );

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let list = reply.json();
    assert_eq!(list.as_array().unwrap().len(), 1);
    let default = &list[0];
    assert_eq!(default["name"], "default");
    assert_eq!(default["type"], "File");
    assert_eq!(default["unavailable"], false);
    assert_eq!(default["blobCount"], 0);
    assert_eq!(default["totalSizeInBytes"], 0);
    assert_eq!(default["softQuota"], Value::Null);
    assert!(default["availableSpaceInBytes"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn every_endpoint_needs_the_matching_permission() {
    let fixture = Fixture::new().await;
    with_default(&fixture).await;
    fixture.add_user("nobody", "nobody-password", &[]).await;
    fixture
        .add_user("reader", "reader-password", &["as-blobstores-read-role"])
        .await;
    let now = chrono::Utc::now().fixed_offset();
    astrum_db::entity::security_role::Entity::insert(
        astrum_db::entity::security_role::ActiveModel {
            id: ActiveValue::Set("as-blobstores-read-role".to_owned()),
            source: ActiveValue::Set("default".to_owned()),
            name: ActiveValue::Set("as-blobstores-read-role".to_owned()),
            description: ActiveValue::Set(String::new()),
            privileges: ActiveValue::Set(json!(["as-blobstores-read"])),
            roles: ActiveValue::Set(json!([])),
            read_only: ActiveValue::Set(false),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        },
    )
    .exec(fixture.state.db().conn())
    .await
    .unwrap();
    fixture.state.security().reload().await.unwrap();

    let file = json!({"name": "files", "path": "files"});
    let s3 = json!({"name": "objects", "bucketConfiguration": {"bucket": {"region": "DEFAULT", "name": "bucket"}}});
    let calls: Vec<(Method, String, Option<Value>, bool)> = vec![
        (Method::GET, BLOBSTORES.to_owned(), None, true),
        (
            Method::GET,
            format!("{BLOBSTORES}/file/default"),
            None,
            true,
        ),
        (Method::GET, format!("{BLOBSTORES}/s3/default"), None, true),
        (
            Method::GET,
            format!("{BLOBSTORES}/default/quota-status"),
            None,
            true,
        ),
        (
            Method::POST,
            format!("{BLOBSTORES}/file"),
            Some(file.clone()),
            false,
        ),
        (
            Method::PUT,
            format!("{BLOBSTORES}/file/default"),
            Some(file),
            false,
        ),
        (
            Method::POST,
            format!("{BLOBSTORES}/s3"),
            Some(s3.clone()),
            false,
        ),
        (
            Method::PUT,
            format!("{BLOBSTORES}/s3/default"),
            Some(s3.clone()),
            false,
        ),
        (
            Method::POST,
            format!("{BLOBSTORES}/test-connection"),
            Some(s3),
            false,
        ),
        (Method::DELETE, format!("{BLOBSTORES}/default"), None, false),
    ];
    for (method, path, body, readable) in calls {
        let reply = fixture
            .call(method.clone(), &path, Auth::None, body.clone())
            .await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        assert!(reply.headers.contains_key(header::WWW_AUTHENTICATE));

        let reply = fixture
            .call(
                method.clone(),
                &path,
                Auth::Basic("nobody", "nobody-password"),
                body.clone(),
            )
            .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");

        let reply = fixture
            .call(
                method.clone(),
                &path,
                Auth::Basic("reader", "reader-password"),
                body,
            )
            .await;
        if readable {
            assert_ne!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
            assert_ne!(reply.status, StatusCode::UNAUTHORIZED, "{method} {path}");
        } else {
            assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        }
    }
    assert!(
        config::find(fixture.state.db().conn(), "files")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        config::find(fixture.state.db().conn(), "default")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn a_file_store_goes_through_its_whole_life() {
    let fixture = Fixture::new().await;
    let reply = create_file(
        &fixture,
        json!({"name": "files", "path": "files", "softQuota": {"type": "spaceUsedQuota", "limit": 1000}}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert!(fixture.data.path().join("blobs/files/content").is_dir());

    let path = format!("{BLOBSTORES}/file/files");
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(
        reply.json(),
        json!({"softQuota": {"type": "spaceUsedQuota", "limit": 1000}, "path": "files"})
    );

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    let listed = &reply.json()[0];
    assert_eq!(listed["name"], "files");
    assert_eq!(
        listed["softQuota"],
        json!({"type": "spaceUsedQuota", "limit": 1000})
    );

    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"path": "files"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(
        reply.json(),
        json!({"softQuota": null, "path": "files"}),
        "a PUT without a quota removes it"
    );

    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"path": "moved"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert!(fixture.data.path().join("blobs/moved/content").is_dir());

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/files"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    assert!(!fixture.data.path().join("blobs/moved").exists());
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json()["id"], "*");
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/files"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    assert_eq!(
        audit_kinds(&fixture, "files").await,
        ["created", "updated", "updated", "deleted"]
    );
}

#[tokio::test]
async fn file_requests_are_validated() {
    let fixture = Fixture::new().await;
    with_default(&fixture).await;

    let reply = create_file(&fixture, json!({})).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["PARAMETER name", "PARAMETER path"]);

    let reply = create_file(&fixture, json!({"name": "files"})).await;
    assert_eq!(ids(&reply), ["PARAMETER path"]);
    let reply = create_file(&fixture, json!({"name": "files", "path": "   "})).await;
    assert_eq!(ids(&reply), ["PARAMETER path"]);

    for name in ["bad name", "_files", ".files", "a/b"] {
        let reply = create_file(&fixture, json!({"name": name, "path": "x"})).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{name}");
        assert_eq!(ids(&reply), ["PARAMETER name"], "{name}");
    }

    for quota in [
        json!({"type": "bogus", "limit": 10}),
        json!({"type": "spaceUsedQuota"}),
        json!({"type": "spaceUsedQuota", "limit": 0}),
        json!({"type": "spaceRemainingQuota", "limit": -1}),
    ] {
        let reply = create_file(
            &fixture,
            json!({"name": "files", "path": "files", "softQuota": quota}),
        )
        .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{quota}");
        assert_eq!(ids(&reply), ["*"], "{quota}");
    }

    let reply = create_file(&fixture, json!({"name": "DEFAULT", "path": "other"})).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["name"]);

    for path in ["default", "default/inner", ".", "./default/../default"] {
        let reply = create_file(&fixture, json!({"name": "inner", "path": path})).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(ids(&reply), ["*"], "{path}");
        assert!(reply.text().contains("default"), "{}", reply.text());
    }

    let reply = fixture
        .send(
            Method::POST,
            &format!("{BLOBSTORES}/file"),
            Auth::Admin,
            None,
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/file/nope"),
            Auth::Admin,
            Some(json!({"path": "nope"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/file/default"),
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(ids(&reply), ["PARAMETER path"]);

    let reply = create_file(
        &fixture,
        json!({"name": "loose", "path": "loose", "softQuota": {"limit": 5}}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/file/loose"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json()["softQuota"],
        Value::Null,
        "no type means no quota"
    );

    assert_eq!(
        blob_store::Entity::find()
            .all(fixture.state.db().conn())
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn names_are_matched_exactly_in_paths() {
    let fixture = Fixture::new().await;
    with_default(&fixture).await;
    for path in [
        format!("{BLOBSTORES}/file/DEFAULT"),
        format!("{BLOBSTORES}/DEFAULT/quota-status"),
        format!("{BLOBSTORES}/nope/quota-status"),
    ] {
        let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
    }
}

#[tokio::test]
async fn a_store_in_use_cannot_be_deleted() {
    let fixture = Fixture::new().await;
    with_default(&fixture).await;
    add_repository(&fixture, "raw-hosted", "default").await;

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/default"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["*"]);
    assert!(reply.text().contains("is in use and cannot be deleted"));
    assert!(fixture.data.path().join("blobs/default").is_dir());

    assert_eq!(
        create_file(&fixture, json!({"name": "member", "path": "member"}))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    config::insert(
        fixture.state.db().conn(),
        &StoreConfig {
            name: "group".to_owned(),
            settings: Settings::Group(GroupSettings {
                members: vec!["member".to_owned()],
                fill_policy: FillPolicy::RoundRobin,
            }),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/member"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    let group = reply
        .json()
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "group")
        .cloned()
        .unwrap();
    assert_eq!(group["type"], "Group");
    assert_eq!(group["unavailable"], false);

    repository::Entity::delete_by_id("raw-hosted")
        .exec(fixture.state.db().conn())
        .await
        .unwrap();
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/default"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

#[tokio::test]
async fn a_store_with_blobs_keeps_its_path_and_reports_its_quota() {
    let fixture = Fixture::new().await;
    let reply = create_file(
        &fixture,
        json!({"name": "small", "path": "small", "softQuota": {"type": "spaceUsedQuota", "limit": 5}}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let store = fixture.state.blob_stores().get("small").await.unwrap();
    store.put(content(b"ten bytes!")).await.unwrap();

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    assert_eq!(reply.json()[0]["blobCount"], 1);
    assert_eq!(reply.json()[0]["totalSizeInBytes"], 10);

    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/small/quota-status"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(
        reply.json(),
        json!({
            "isViolation": true,
            "message": "Blob store small uses 10 B of its 5 B limit",
            "blobStoreName": "small"
        })
    );

    let path = format!("{BLOBSTORES}/file/small");
    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"path": "elsewhere"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["PARAMETER path"]);
    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(json!({"path": "./small", "softQuota": {"type": "spaceUsedQuota", "limit": 100}})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/small/quota-status"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.json()["isViolation"], false);

    let reply = fixture
        .call(
            Method::DELETE,
            &format!("{BLOBSTORES}/small"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(
        fixture.data.path().join("blobs/small/content").is_dir(),
        "blobs of a deleted store are left on disk"
    );
}

fn s3_body(server: &S3Server, name: &str, bucket: &str) -> Value {
    json!({
        "name": name,
        "type": "S3",
        "softQuota": {"type": "spaceUsedQuota", "limit": 1000000},
        "bucketConfiguration": {
            "bucket": {"region": "DEFAULT", "name": bucket, "prefix": "/astrum/", "expiration": 3},
            "bucketSecurity": {"accessKeyId": ACCESS_KEY, "secretAccessKey": SECRET_KEY, "sessionToken": ""},
            "encryption": {"encryptionType": "s3ManagedEncryption"},
            "advancedBucketConnection": {
                "endpoint": server.endpoint(),
                "signerType": "DEFAULT",
                "forcePathStyle": true,
                "maxConnectionPoolSize": 8
            },
            "failoverBuckets": [{"region": "eu-west-1", "bucketName": "spare"}],
            "preSignedUrlEnabled": false
        }
    })
}

#[tokio::test]
async fn an_s3_store_keeps_its_secrets_to_itself() {
    let server = S3Server::start().await;
    let fixture = Fixture::new().await;
    allow_loopback(&fixture).await;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/s3"),
            Auth::Admin,
            Some(s3_body(&server, "objects", "astrum-bucket")),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    assert!(
        server.root().join("astrum-bucket").is_dir(),
        "the bucket was created"
    );

    let row = blob_store::Entity::find_by_id("objects")
        .one(fixture.state.db().conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.kind, "S3");
    let stored = row.attributes.to_string();
    assert!(!stored.contains(SECRET_KEY), "{stored}");
    assert!(stored.contains("astrum:v1:"), "{stored}");

    let path = format!("{BLOBSTORES}/s3/objects");
    let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(!reply.text().contains(SECRET_KEY));
    assert!(!reply.text().contains("secretAccessKey"));
    assert!(!reply.text().contains("astrum:v1:"));
    assert_eq!(
        reply.json(),
        json!({
            "name": "objects",
            "softQuota": {"type": "spaceUsedQuota", "limit": 1000000},
            "bucketConfiguration": {
                "bucket": {"region": "DEFAULT", "name": "astrum-bucket", "prefix": "astrum", "expiration": 3},
                "bucketSecurity": {"accessKeyId": ACCESS_KEY, "role": null},
                "encryption": {"encryptionType": "s3ManagedEncryption", "encryptionKey": null},
                "advancedBucketConnection": {
                    "endpoint": server.endpoint(),
                    "signerType": "DEFAULT",
                    "forcePathStyle": true,
                    "maxConnectionPoolSize": 8
                },
                "failoverBuckets": [{"region": "eu-west-1", "bucketName": "spare"}],
                "preSignedUrlEnabled": false,
                "activeRegion": "us-east-1"
            },
            "type": "S3"
        })
    );

    let store = fixture.state.blob_stores().get("objects").await.unwrap();
    let written = store.put(content(b"in the bucket")).await.unwrap();
    assert_eq!(
        server.objects("astrum-bucket"),
        [format!("astrum/content/{}", written.id.location())]
    );

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    let listed = &reply.json()[0];
    assert_eq!(listed["type"], "S3");
    assert_eq!(listed["unavailable"], false);
    assert_eq!(listed["blobCount"], 1);
    assert_eq!(listed["availableSpaceInBytes"], Value::Null);

    let mut update = s3_body(&server, "objects", "astrum-bucket");
    update["bucketConfiguration"]["bucketSecurity"] = json!({"accessKeyId": ACCESS_KEY});
    update["softQuota"] = json!(null);
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(update))
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let store = fixture.state.blob_stores().get("objects").await.unwrap();
    store.check().await.unwrap();
    assert!(
        store.exists(written.id).await.unwrap(),
        "the stored secret was kept"
    );

    let mut wrong = s3_body(&server, "objects", "astrum-bucket");
    wrong["bucketConfiguration"]["bucketSecurity"]["secretAccessKey"] = json!("not-the-secret");
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(wrong))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert!(!reply.text().contains("not-the-secret"));
    fixture
        .state
        .blob_stores()
        .get("objects")
        .await
        .unwrap()
        .check()
        .await
        .unwrap();

    let mut moved = s3_body(&server, "objects", "other-bucket");
    moved["bucketConfiguration"]["security"] = moved["bucketConfiguration"]
        .as_object_mut()
        .unwrap()
        .remove("bucketSecurity")
        .unwrap();
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(moved))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["PARAMETER bucket"]);

    let reply = fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/s3"),
            Auth::Admin,
            Some(s3_body(&server, "OBJECTS", "another-bucket")),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&reply), ["name"]);

    assert_eq!(
        audit_kinds(&fixture, "objects").await,
        ["created", "updated"]
    );
    let audited = audit_event::Entity::find()
        .all(fixture.state.db().conn())
        .await
        .unwrap();
    for event in audited {
        assert!(!event.attributes.to_string().contains(SECRET_KEY));
    }
}

#[tokio::test]
async fn s3_requests_are_validated() {
    let fixture = Fixture::new().await;
    allow_loopback(&fixture).await;
    let post = |body: Value| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::POST,
                    &format!("{BLOBSTORES}/s3"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    };

    let reply = post(json!({})).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        ids(&reply),
        ["PARAMETER name", "PARAMETER bucketConfiguration"]
    );
    let reply = post(json!({"name": "objects", "bucketConfiguration": {}})).await;
    assert_eq!(ids(&reply), ["PARAMETER bucket"]);
    let reply = post(json!({"name": "objects", "bucketConfiguration": {"bucket": {}}})).await;
    assert_eq!(ids(&reply), ["PARAMETER region", "PARAMETER name"]);

    let bucket = json!({"region": "DEFAULT", "name": "bucket"});
    for (configuration, id) in [
        (
            json!({"bucket": {"region": "DEFAULT", "name": "Bad_Bucket"}}),
            "bucketConfiguration.bucket.name",
        ),
        (
            json!({"bucket": bucket, "bucketSecurity": {"accessKeyId": "AKID"}}),
            "bucketConfiguration.bucketSecurity.secretAccessKey",
        ),
        (
            json!({"bucket": bucket, "bucketSecurity": {"secretAccessKey": "secret"}}),
            "bucketConfiguration.bucketSecurity.accessKeyId",
        ),
        (
            json!({"bucket": bucket, "encryption": {"encryptionType": "rot13"}}),
            "bucketConfiguration.encryption.encryptionType",
        ),
        (
            json!({"bucket": bucket, "advancedBucketConnection": {"endpoint": "ftp://host"}}),
            "bucketConfiguration.advancedBucketConnection.endpoint",
        ),
        (
            json!({"bucket": bucket, "advancedBucketConnection": {"signerType": "v1"}}),
            "bucketConfiguration.advancedBucketConnection.signerType",
        ),
        (
            json!({"bucket": bucket, "failoverBuckets": [{"region": "eu-west-1"}]}),
            "bucketConfiguration.failoverBuckets[0]",
        ),
    ] {
        let reply = post(json!({"name": "objects", "bucketConfiguration": configuration})).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{id}");
        assert_eq!(ids(&reply), [id]);
        assert!(!reply.text().contains("secret\""), "{}", reply.text());
    }

    let reply =
        post(json!({"name": "objects", "type": "File", "bucketConfiguration": {"bucket": bucket}}))
            .await;
    assert_eq!(ids(&reply), ["PARAMETER type"]);
    let reply = post(json!({
        "name": "objects",
        "softQuota": {"type": "spaceRemainingQuota"},
        "bucketConfiguration": {"bucket": bucket}
    }))
    .await;
    assert_eq!(ids(&reply), ["*"]);

    let reply = post(json!({
        "name": "objects",
        "bucketConfiguration": {
            "bucket": bucket,
            "bucketSecurity": {"accessKeyId": ACCESS_KEY, "secretAccessKey": SECRET_KEY},
            "advancedBucketConnection": {"endpoint": "http://127.0.0.1:1", "forcePathStyle": true}
        }
    }))
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert_eq!(ids(&reply), ["*"]);
    assert!(!reply.text().contains(SECRET_KEY));
    assert!(
        blob_store::Entity::find()
            .all(fixture.state.db().conn())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn s3_endpoints_on_internal_addresses_are_refused_unless_allowed() {
    let server = S3Server::start().await;
    let fixture = Fixture::new().await;
    let create = |body: Value| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::POST,
                    &format!("{BLOBSTORES}/s3"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    };
    let reply = create(s3_body(&server, "objects", "guarded")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!([{
            "id": "bucketConfiguration.advancedBucketConnection.endpoint",
            "message": "Host resolves to private/local IP address(es): 127.0.0.1 (loopback address). To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection."
        }])
    );
    assert!(
        !server.root().join("guarded").exists(),
        "a refused endpoint is never contacted"
    );

    allow_loopback(&fixture).await;
    let reply = create(s3_body(&server, "objects", "guarded")).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection::default(),
    )
    .await
    .unwrap();
    let path = format!("{BLOBSTORES}/s3/objects");
    let reply = fixture
        .call(
            Method::PUT,
            &path,
            Auth::Admin,
            Some(s3_body(&server, "objects", "guarded")),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::NO_CONTENT,
        "an unchanged endpoint is not judged again: {}",
        reply.text()
    );
    let mut moved = s3_body(&server, "objects", "guarded");
    moved["bucketConfiguration"]["advancedBucketConnection"]["endpoint"] =
        json!("http://10.255.0.1:9000");
    let reply = fixture
        .call(Method::PUT, &path, Auth::Admin, Some(moved))
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert!(
        reply
            .text()
            .contains("10.255.0.1 (private network address)"),
        "{}",
        reply.text()
    );
}

#[tokio::test]
async fn s3_settings_can_be_tested_before_they_are_saved() {
    let server = S3Server::start().await;
    let fixture = Fixture::new().await;
    let test = |body: Value| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::POST,
                    &format!("{BLOBSTORES}/test-connection"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    };
    let reply = test(s3_body(&server, "objects", "tested")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        ids(&reply),
        ["bucketConfiguration.advancedBucketConnection.endpoint"]
    );

    allow_loopback(&fixture).await;
    let reply = test(s3_body(&server, "objects", "tested")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
    assert_eq!(
        reply.json(),
        json!([{"id": "*", "message": "Connection failed, check the logs for more information."}]),
        "the bucket does not exist yet"
    );
    assert!(
        !server.root().join("tested").exists(),
        "a test never creates the bucket"
    );
    let reply = test(json!({"name": "objects", "bucketConfiguration": {}})).await;
    assert_eq!(ids(&reply), ["PARAMETER bucket"]);

    let reply = fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/s3"),
            Auth::Admin,
            Some(s3_body(&server, "objects", "tested")),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let reply = test(s3_body(&server, "fresh", "tested")).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());

    let mut without_secret = s3_body(&server, "objects", "tested");
    without_secret["bucketConfiguration"]["bucketSecurity"]["secretAccessKey"] = json!("");
    let reply = test(without_secret.clone()).await;
    assert_eq!(
        reply.status,
        StatusCode::NO_CONTENT,
        "the stored secret of the named store is used: {}",
        reply.text()
    );
    without_secret["name"] = json!("fresh");
    let reply = test(without_secret).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);

    let mut wrong = s3_body(&server, "fresh", "tested");
    wrong["bucketConfiguration"]["bucketSecurity"]["secretAccessKey"] = json!("not-the-secret");
    let reply = test(wrong).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reply.json()[0]["message"],
        "Connection failed, check the logs for more information."
    );
    assert!(!reply.text().contains("not-the-secret"));
}

#[tokio::test]
async fn the_type_of_a_store_cannot_change() {
    let server = S3Server::start().await;
    let fixture = Fixture::new().await;
    allow_loopback(&fixture).await;
    with_default(&fixture).await;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{BLOBSTORES}/s3"),
            Auth::Admin,
            Some(s3_body(&server, "objects", "typed")),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());

    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/s3/default"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("not an S3 blob store"));
    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/file/objects"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    let reply = fixture
        .call(
            Method::GET,
            &format!("{BLOBSTORES}/s3/nope"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/s3/default"),
            Auth::Admin,
            Some(s3_body(&server, "default", "typed")),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("cannot change"));
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/file/objects"),
            Auth::Admin,
            Some(json!({"path": "objects"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.text().contains("cannot change"));

    let mut renamed = s3_body(&server, "renamed", "typed");
    renamed["type"] = json!("S3");
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{BLOBSTORES}/s3/objects"),
            Auth::Admin,
            Some(renamed),
        )
        .await;
    assert_eq!(ids(&reply), ["PARAMETER name"]);

    let config = config::find(fixture.state.db().conn(), "default")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.file().unwrap().path, "default");
}

#[tokio::test]
async fn an_unreachable_store_is_listed_as_unavailable() {
    let fixture = Fixture::new().await;
    let sealed = fixture.state.secrets().seal(SECRET_KEY);
    config::insert(
        fixture.state.db().conn(),
        &StoreConfig {
            name: "gone".to_owned(),
            settings: Settings::S3(Box::new(S3Settings {
                bucket: S3Bucket {
                    region: "DEFAULT".to_owned(),
                    name: "gone".to_owned(),
                    ..Default::default()
                },
                security: S3Security {
                    access_key_id: Some(ACCESS_KEY.to_owned()),
                    secret_access_key: Some(sealed),
                    ..Default::default()
                },
                connection: S3Connection {
                    endpoint: Some("http://127.0.0.1:1".to_owned()),
                    force_path_style: true,
                    ..Default::default()
                },
                ..Default::default()
            })),
            soft_quota: None,
        },
    )
    .await
    .unwrap();
    let reply = create_file(&fixture, json!({"name": "unplugged", "path": "unplugged"})).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let root = fixture.data.path().join("blobs/unplugged");
    std::fs::remove_dir_all(&root).unwrap();

    let reply = fixture
        .call(Method::GET, BLOBSTORES, Auth::Admin, None)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let list = reply.json();
    assert_eq!(list[0]["name"], "gone");
    assert_eq!(list[0]["unavailable"], true);
    assert_eq!(list[1]["name"], "unplugged");
    assert_eq!(list[1]["unavailable"], true);
    assert!(
        !root.exists(),
        "a vanished directory is not silently recreated"
    );
}
