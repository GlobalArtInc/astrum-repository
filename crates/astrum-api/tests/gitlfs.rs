mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use support::{Auth, Fixture, Reply};

const LFS: &str = "application/vnd.git-lfs+json";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_gitlfs::GitLfs));
    fixture
}

async fn create(fixture: &Fixture, name: &str, policy: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/gitlfs/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": policy}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn oid(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

async fn api(fixture: &Fixture, auth: Auth<'_>, method: Method, path: &str, body: Value) -> Reply {
    fixture
        .send_body(
            method,
            path,
            auth,
            Body::from(serde_json::to_vec(&body).unwrap()),
            &[("accept", LFS), ("content-type", LFS)],
        )
        .await
}

async fn put(fixture: &Fixture, auth: Auth<'_>, path: &str, data: &[u8]) -> Reply {
    fixture
        .send_body(
            Method::PUT,
            path,
            auth,
            Body::from(data.to_vec()),
            &[("content-type", "application/octet-stream")],
        )
        .await
}

async fn add_user_with(fixture: &Fixture, user: &str, privileges: &[&str]) {
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": format!("{user}-role"), "name": format!("{user}-role"), "description": "", "privileges": privileges, "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user(user, "Secret-123", &[&format!("{user}-role")])
        .await;
}

#[tokio::test]
async fn objects_move_through_the_batch_api_and_the_basic_transfer() {
    let fixture = ready().await;
    create(&fixture, "lfs", "ALLOW_ONCE").await;
    let data = b"a large file of a git repository\n".repeat(100);
    let id = oid(&data);
    let batch = "/repository/lfs/info/lfs/objects/batch";
    let upload = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        batch,
        json!({"operation": "upload", "transfers": ["basic"], "hash_algo": "sha256",
               "objects": [{"oid": id, "size": data.len()}, {"oid": "bad", "size": 1}, {"oid": id.replace('a', "b"), "size": -1}]}),
    )
    .await;
    assert_eq!(upload.status, StatusCode::OK, "{}", upload.text());
    assert_eq!(upload.headers[header::CONTENT_TYPE], LFS);
    let body = upload.json();
    assert_eq!(body["transfer"], "basic");
    assert_eq!(body["hash_algo"], "sha256");
    let href = format!("http://localhost/repository/lfs/info/lfs/objects/{id}");
    assert_eq!(body["objects"][0]["actions"]["upload"]["href"], href);
    assert_eq!(
        body["objects"][0]["actions"]["verify"]["href"],
        "http://localhost/repository/lfs/info/lfs/objects/verify"
    );
    assert_eq!(
        body["objects"][1]["error"],
        json!({"code": 422, "message": "Invalid object id"})
    );
    assert_eq!(
        body["objects"][2]["error"]["message"],
        "Invalid object size"
    );

    let wrong = put(
        &fixture,
        Auth::Admin,
        &format!("/repository/lfs/info/lfs/objects/{id}"),
        b"other",
    )
    .await;
    assert_eq!(wrong.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        wrong.json()["message"],
        format!("Object content hashes to {}, not to {id}", oid(b"other"))
    );
    let stored = put(
        &fixture,
        Auth::Admin,
        &format!("/repository/lfs/info/lfs/objects/{id}"),
        &data,
    )
    .await;
    assert_eq!(stored.status, StatusCode::OK, "{}", stored.text());
    let again = put(
        &fixture,
        Auth::Admin,
        &format!("/repository/lfs/info/lfs/objects/{id}"),
        &data,
    )
    .await;
    assert_eq!(again.status, StatusCode::OK);

    let verify = |size: usize| {
        api(
            &fixture,
            Auth::Admin,
            Method::POST,
            "/repository/lfs/info/lfs/objects/verify",
            json!({"oid": id, "size": size}),
        )
    };
    assert_eq!(
        verify(data.len()).await.json(),
        json!({"oid": id, "size": data.len()})
    );
    let short = verify(3).await;
    assert_eq!(short.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        short.json()["message"],
        format!("Object {id} has {} bytes, not 3", data.len())
    );

    let known = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        batch,
        json!({"operation": "upload", "objects": [{"oid": id, "size": data.len()}]}),
    )
    .await
    .json();
    assert_eq!(known["objects"][0], json!({"oid": id, "size": data.len()}));
    let missing = "0".repeat(64);
    let download = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        "/repository/lfs/objects/batch",
        json!({"operation": "download", "objects": [{"oid": id, "size": 1}, {"oid": missing, "size": 2}]}),
    )
    .await
    .json();
    assert_eq!(download["objects"][0]["size"], data.len());
    assert_eq!(
        download["objects"][0]["actions"]["download"]["href"],
        format!("http://localhost/repository/lfs/objects/{id}")
    );
    assert_eq!(
        download["objects"][1]["error"],
        json!({"code": 404, "message": "Object does not exist"})
    );

    for path in [
        format!("/repository/lfs/objects/{id}"),
        format!("/repository/lfs/info/lfs/objects/{id}"),
        format!("/repository/lfs/objects/{}/{}/{id}", &id[..2], &id[2..4]),
    ] {
        let reply = fixture.call(Method::GET, &path, Auth::Admin, None).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.body, data);
    }
    let absent = fixture
        .call(
            Method::GET,
            &format!("/repository/lfs/objects/{missing}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(absent.status, StatusCode::NOT_FOUND);
    assert_eq!(absent.text(), format!("Object {missing} does not exist"));
    let found = fixture
        .call(
            Method::GET,
            "/service/rest/v1/search?repository=lfs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(found["items"][0]["name"], id);
    assert_eq!(
        found["items"][0]["assets"][0]["path"],
        format!("/objects/{}/{}/{id}", &id[..2], &id[2..4])
    );

    let refused = fixture
        .send_body(
            Method::POST,
            batch,
            Auth::Admin,
            Body::from("{}"),
            &[("accept", "text/html")],
        )
        .await;
    assert_eq!(refused.status, StatusCode::NOT_ACCEPTABLE);
    let broken = fixture
        .send_body(
            Method::POST,
            batch,
            Auth::Admin,
            Body::from("{"),
            &[("accept", LFS)],
        )
        .await;
    assert_eq!(broken.status, StatusCode::BAD_REQUEST);
    let anonymous = api(
        &fixture,
        Auth::None,
        Method::POST,
        batch,
        json!({"operation": "download", "objects": []}),
    )
    .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert!(anonymous.headers.contains_key(header::WWW_AUTHENTICATE));
    let page = fixture
        .call(Method::GET, "/repository/lfs/", Auth::Admin, None)
        .await;
    assert!(page.text().contains("Git LFS repository"));
}

#[tokio::test]
async fn privileges_decide_which_objects_move_and_read_only_repositories_refuse_uploads() {
    let fixture = ready().await;
    create(&fixture, "lfs", "ALLOW").await;
    create(&fixture, "frozen", "DENY").await;
    add_user_with(
        &fixture,
        "reader",
        &[
            "as-repository-view-gitlfs-lfs-read",
            "as-repository-view-gitlfs-lfs-browse",
        ],
    )
    .await;
    let data = b"payload";
    let id = oid(data);
    let batch = "/repository/lfs/info/lfs/objects/batch";
    let denied = api(
        &fixture,
        Auth::Basic("reader", "Secret-123"),
        Method::POST,
        batch,
        json!({"operation": "upload", "objects": [{"oid": id, "size": 7}]}),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert_eq!(
        denied.json()["message"],
        "Not permitted to upload to this repository"
    );
    let put_denied = put(
        &fixture,
        Auth::Basic("reader", "Secret-123"),
        &format!("/repository/lfs/objects/{id}"),
        data,
    )
    .await;
    assert_eq!(put_denied.status, StatusCode::FORBIDDEN);
    assert_eq!(put_denied.headers[header::CONTENT_TYPE], LFS);
    assert_eq!(
        put_denied.json()["message"],
        "Not permitted in repository lfs"
    );
    let frozen = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        "/repository/frozen/info/lfs/objects/batch",
        json!({"operation": "upload", "objects": []}),
    )
    .await;
    assert_eq!(frozen.status, StatusCode::FORBIDDEN);
    assert_eq!(frozen.json()["message"], "Repository frozen is read-only");
    assert_eq!(
        put(
            &fixture,
            Auth::Admin,
            &format!("/repository/lfs/objects/{id}"),
            data
        )
        .await
        .status,
        StatusCode::OK
    );
    let read = api(
        &fixture,
        Auth::Basic("reader", "Secret-123"),
        Method::POST,
        batch,
        json!({"operation": "download", "objects": [{"oid": id, "size": 7}]}),
    )
    .await
    .json();
    assert!(read["objects"][0]["actions"]["download"]["href"].is_string());
}

#[tokio::test]
async fn locks_are_created_listed_verified_and_released() {
    let fixture = ready().await;
    create(&fixture, "lfs", "ALLOW").await;
    add_user_with(
        &fixture,
        "jane",
        &[
            "as-repository-view-gitlfs-lfs-read",
            "as-repository-view-gitlfs-lfs-add",
        ],
    )
    .await;
    let jane = || Auth::Basic("jane", "Secret-123");
    let locks = "/repository/lfs/info/lfs/locks";
    let created = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        locks,
        json!({"path": "assets/logo.psd", "ref": {"name": "refs/heads/main"}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let lock = created.json()["lock"].clone();
    assert_eq!(lock["path"], "assets/logo.psd");
    assert_eq!(lock["owner"]["name"], "admin");
    assert!(lock["locked_at"].as_str().unwrap().ends_with('Z'));
    let id = lock["id"].as_str().unwrap().to_owned();
    let taken = api(
        &fixture,
        jane(),
        Method::POST,
        locks,
        json!({"path": "assets/logo.psd"}),
    )
    .await;
    assert_eq!(taken.status, StatusCode::CONFLICT);
    assert_eq!(taken.json()["message"], "already created lock");
    assert_eq!(taken.json()["lock"]["id"], id.as_str());
    let other = api(
        &fixture,
        jane(),
        Method::POST,
        locks,
        json!({"path": "docs/manual.pdf"}),
    )
    .await;
    assert_eq!(other.status, StatusCode::CREATED);
    assert_eq!(
        api(&fixture, jane(), Method::POST, locks, json!({"path": " "}))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let all = fixture
        .call(Method::GET, locks, Auth::Admin, None)
        .await
        .json();
    assert_eq!(all["locks"].as_array().unwrap().len(), 2);
    let first = fixture
        .call(Method::GET, &format!("{locks}?limit=1"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(first["locks"].as_array().unwrap().len(), 1);
    let next = first["next_cursor"].as_str().unwrap();
    let second = fixture
        .call(
            Method::GET,
            &format!("{locks}?cursor={next}"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(second["locks"][0]["path"], "docs/manual.pdf");
    assert!(second.get("next_cursor").is_none());
    let by_path = fixture
        .call(
            Method::GET,
            "/repository/lfs/locks?path=assets/logo.psd",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(by_path["locks"][0]["id"], id.as_str());
    let by_id = fixture
        .call(
            Method::GET,
            &format!("{locks}?id={id}&path=other"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(by_id["locks"], json!([]));
    assert_eq!(
        fixture
            .call(Method::GET, &format!("{locks}?limit=0"), Auth::Admin, None)
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let verified = api(
        &fixture,
        jane(),
        Method::POST,
        &format!("{locks}/verify"),
        json!({"ref": {"name": "refs/heads/main"}}),
    )
    .await
    .json();
    assert_eq!(verified["ours"][0]["path"], "docs/manual.pdf");
    assert_eq!(verified["theirs"][0]["path"], "assets/logo.psd");

    let held = api(
        &fixture,
        jane(),
        Method::POST,
        &format!("{locks}/{id}/unlock"),
        json!({}),
    )
    .await;
    assert_eq!(held.status, StatusCode::FORBIDDEN);
    assert_eq!(
        held.json()["message"],
        "Lock on assets/logo.psd is held by admin"
    );
    let forced = api(
        &fixture,
        jane(),
        Method::POST,
        &format!("{locks}/{id}/unlock"),
        json!({"force": true}),
    )
    .await;
    assert_eq!(forced.status, StatusCode::FORBIDDEN);
    assert_eq!(
        forced.json()["message"],
        "Releasing the lock of another user needs the delete privilege"
    );
    let released = api(
        &fixture,
        Auth::Admin,
        Method::POST,
        &format!("{locks}/{id}/unlock"),
        json!({}),
    )
    .await;
    assert_eq!(released.status, StatusCode::OK);
    assert_eq!(released.json()["lock"]["id"], id.as_str());
    assert_eq!(
        api(
            &fixture,
            Auth::Admin,
            Method::POST,
            &format!("{locks}/{id}/unlock"),
            json!({})
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let deleted = fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/repositories/lfs",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    create(&fixture, "lfs", "ALLOW").await;
    assert_eq!(
        fixture
            .call(Method::GET, locks, Auth::Admin, None)
            .await
            .json()["locks"],
        json!([])
    );
}

#[tokio::test]
async fn objects_upload_through_the_components_api() {
    let fixture = ready().await;
    create(&fixture, "lfs", "ALLOW_ONCE").await;
    let spec = fixture
        .call(
            Method::GET,
            "/service/rest/v1/formats/gitlfs/upload-specs",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(spec["assetFields"][0]["name"], "asset");
    assert_eq!(spec["componentFields"], json!([]));
    let data = b"uploaded through the components API";
    let boundary = "lfsboundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"gitlfs.asset\"; filename=\"big.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    for _ in 0..2 {
        let reply = fixture
            .send_body(
                Method::POST,
                "/service/rest/v1/components?repository=lfs",
                Auth::Admin,
                Body::from(body.clone()),
                &[(
                    "content-type",
                    &format!("multipart/form-data; boundary={boundary}"),
                )],
            )
            .await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    }
    let id = oid(data);
    let reply = fixture
        .call(
            Method::GET,
            &format!("/repository/lfs/objects/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.body, data);
}
