mod support;

use std::sync::Arc;

use astrum_content::Format;
use astrum_content::inventory::LicenseFact;
use astrum_content::store;
use astrum_format_maven::Maven;
use axum::body::Body;
use axum::http::{Method, StatusCode};
use serde_json::json;
use support::{Auth, Fixture};

const REPOSITORY: &str = "licences";
const LICENCE: &str =
    "<licenses><license><name>Apache License, Version 2.0</name></license></licenses>";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture.state.content().register(Arc::new(Maven));
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/maven/hosted",
            Auth::Admin,
            Some(json!({
                "name": REPOSITORY,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": true, "writePolicy": "ALLOW"},
                "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT", "contentDisposition": "ATTACHMENT"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    fixture
}

fn pom_path(group: &str, artifact: &str, version: &str) -> String {
    format!(
        "/{}/{artifact}/{version}/{artifact}-{version}.pom",
        group.replace('.', "/")
    )
}

async fn upload(
    fixture: &Fixture,
    group: &str,
    artifact: &str,
    licences: &str,
    parent: Option<(&str, &str)>,
) {
    let parent = parent
        .map(|(artifact, version)| {
            format!(
                "<parent><groupId>{group}</groupId><artifactId>{artifact}</artifactId><version>{version}</version></parent>"
            )
        })
        .unwrap_or_default();
    let pom = format!(
        "<project><modelVersion>4.0.0</modelVersion>{parent}<groupId>{group}</groupId><artifactId>{artifact}</artifactId><version>1.0</version>{licences}</project>"
    );
    let path = format!(
        "/repository/{REPOSITORY}{}",
        pom_path(group, artifact, "1.0")
    );
    let reply = fixture
        .send_body(
            Method::PUT,
            &path,
            Auth::Admin,
            Body::from(pom),
            &[("content-type", "application/octet-stream")],
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn licence_of(fixture: &Fixture, group: &str, artifact: &str) -> Option<LicenseFact> {
    let db = fixture.state.db().conn();
    let asset = store::find(db, REPOSITORY, &pom_path(group, artifact, "1.0"))
        .await
        .unwrap()
        .unwrap();
    let component = store::component_by_id(db, asset.component_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    let content = fixture.state.content();
    let repository = content.repository(REPOSITORY).await.unwrap().unwrap();
    let ctx = content.context(repository, fixture.state.security().model(), None, "", "");
    Maven
        .inventory(&ctx, &component)
        .await
        .unwrap()
        .unwrap()
        .license
}

fn fact(declared: &str) -> Option<LicenseFact> {
    Some(LicenseFact {
        declared: declared.to_owned(),
        source: "maven.pom".to_owned(),
    })
}

async fn chain(fixture: &Fixture, group: &str, licensed_at: usize) {
    for level in 0..=licensed_at {
        let artifact = format!("p{level}");
        if level == licensed_at {
            upload(fixture, group, &artifact, LICENCE, None).await;
        } else {
            let parent = format!("p{}", level + 1);
            upload(fixture, group, &artifact, "", Some((&parent, "1.0"))).await;
        }
    }
}

#[tokio::test]
async fn a_pom_declares_its_licence() {
    let fixture = ready().await;
    upload(&fixture, "org.example", "lib", LICENCE, None).await;
    assert_eq!(
        licence_of(&fixture, "org.example", "lib").await,
        fact("Apache License, Version 2.0")
    );
}

#[tokio::test]
async fn two_licences_are_alternatives() {
    let fixture = ready().await;
    upload(
        &fixture,
        "org.example",
        "lib",
        "<licenses><license><name>MIT</name></license><license><url>https://www.apache.org/licenses/LICENSE-2.0</url></license></licenses>",
        None,
    )
    .await;
    assert_eq!(
        licence_of(&fixture, "org.example", "lib").await,
        fact("MIT OR https://www.apache.org/licenses/LICENSE-2.0")
    );
}

#[tokio::test]
async fn a_licence_is_inherited_from_a_grandparent() {
    let fixture = ready().await;
    upload(&fixture, "org.example", "lib", "", Some(("mid", "1.0"))).await;
    upload(&fixture, "org.example", "mid", "", Some(("top", "1.0"))).await;
    upload(&fixture, "org.example", "top", LICENCE, None).await;
    assert_eq!(
        licence_of(&fixture, "org.example", "lib").await,
        fact("Apache License, Version 2.0")
    );
}

#[tokio::test]
async fn parents_beyond_depth_five_are_not_read() {
    let fixture = ready().await;
    chain(&fixture, "org.depth5", 5).await;
    assert_eq!(
        licence_of(&fixture, "org.depth5", "p0").await,
        fact("Apache License, Version 2.0")
    );
    chain(&fixture, "org.depth6", 6).await;
    assert_eq!(licence_of(&fixture, "org.depth6", "p0").await, None);
}

#[tokio::test]
async fn a_parent_that_is_not_stored_gives_no_licence() {
    let fixture = ready().await;
    upload(&fixture, "org.example", "lib", "", Some(("missing", "1.0"))).await;
    assert_eq!(licence_of(&fixture, "org.example", "lib").await, None);
}

#[tokio::test]
async fn a_cycle_of_parents_gives_no_licence() {
    let fixture = ready().await;
    upload(&fixture, "org.example", "lib", "", Some(("mid", "1.0"))).await;
    upload(&fixture, "org.example", "mid", "", Some(("lib", "1.0"))).await;
    assert_eq!(licence_of(&fixture, "org.example", "lib").await, None);
}

#[tokio::test]
async fn a_pom_without_a_licence_gives_none() {
    let fixture = ready().await;
    upload(&fixture, "org.example", "lib", "", None).await;
    assert_eq!(licence_of(&fixture, "org.example", "lib").await, None);
}
