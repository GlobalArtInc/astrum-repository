mod accounts;
mod shop;
mod support;

use accounts::audit;
use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use shop::{CUSTOMERS, raw_repository, read_as, shop};
use support::{Auth, Fixture};

const TOKENS: &str = "/service/rest/internal/access-tokens";

async fn delete_repository(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::DELETE,
            &format!("/service/rest/v1/repositories/{name}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn tokens_of(fixture: &Fixture, customer: &str) -> Value {
    fixture
        .call(
            Method::GET,
            &format!("{CUSTOMERS}/{customer}/tokens"),
            Auth::Admin,
            None,
        )
        .await
        .json()
}

#[tokio::test]
async fn deleting_one_of_two_listed_repositories_keeps_the_token_for_the_other() {
    let fixture = shop().await;
    let (_, token) = fixture
        .customer_token("acme", &["files-b", "files-c"])
        .await;
    assert_eq!(
        read_as(&fixture, "acme", "files-c", &token).await,
        StatusCode::OK
    );
    delete_repository(&fixture, "files-c").await;

    let held = tokens_of(&fixture, "acme").await;
    assert_eq!(held.as_array().unwrap().len(), 1);
    assert_eq!(held[0]["repositories"], json!(["files-b"]));
    assert_eq!(
        read_as(&fixture, "acme", "files-b", &token).await,
        StatusCode::OK
    );
    let customers = audit(&fixture, "security.customer").await;
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0].0, "tokens-updated");
    assert_eq!(customers[0].1, "acme");
    assert_eq!(
        customers[0].3,
        json!({"repository": "files-c", "narrowed": 1, "deleted": 0})
    );
    assert!(audit(&fixture, "security.accesstoken").await.is_empty());
}

#[tokio::test]
async fn deleting_the_only_listed_repository_deletes_the_token() {
    let fixture = shop().await;
    let (id, token) = fixture.customer_token("acme", &["files-c"]).await;
    delete_repository(&fixture, "files-c").await;

    assert_eq!(
        read_as(&fixture, "acme", "files-b", &token).await,
        StatusCode::UNAUTHORIZED
    );
    assert!(
        tokens_of(&fixture, "acme")
            .await
            .as_array()
            .unwrap()
            .is_empty()
    );
    let kept = fixture
        .call(Method::GET, &format!("{CUSTOMERS}/acme"), Auth::Admin, None)
        .await;
    assert_eq!(kept.json()["tokens"], 0);

    let deleted = audit(&fixture, "security.accesstoken").await;
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].0, "deleted");
    assert_eq!(deleted[0].1, "release");
    assert_eq!(deleted[0].2, "admin");
    assert_eq!(deleted[0].3["id"], id.to_string());
    assert_eq!(deleted[0].3["customer"], "acme");
    assert_eq!(deleted[0].3["repositories"], json!(["files-c"]));
    assert_eq!(deleted[0].3["reason"], "repository-deleted");
    let customers = audit(&fixture, "security.customer").await;
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0].0, "tokens-updated");
    assert_eq!(
        customers[0].3,
        json!({"repository": "files-c", "narrowed": 0, "deleted": 1})
    );
    for event in deleted.iter().chain(&customers) {
        assert!(!event.3.to_string().contains(&token));
    }
}

#[tokio::test]
async fn a_repository_created_again_under_the_same_name_is_not_reached_by_old_tokens() {
    let fixture = shop().await;
    let (_, only) = fixture.customer_token("acme", &["files-c"]).await;
    let (_, both) = fixture
        .customer_token("acme", &["files-b", "files-c"])
        .await;
    delete_repository(&fixture, "files-c").await;
    raw_repository(&fixture, "files-c").await;

    assert_eq!(
        read_as(&fixture, "acme", "files-c", &only).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        read_as(&fixture, "acme", "files-c", &both).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        read_as(&fixture, "acme", "files-b", &both).await,
        StatusCode::OK
    );
    let admin = fixture
        .call(Method::GET, "/repository/files-c/a.txt", Auth::Admin, None)
        .await;
    assert_eq!(admin.status, StatusCode::OK);
}

#[tokio::test]
async fn personal_tokens_limited_to_repositories_follow_the_same_rule() {
    let fixture = shop().await;
    fixture
        .add_user("olivia", "correct horse", &["as-admin"])
        .await;
    let olivia = || Auth::Basic("olivia", "correct horse");
    let mut secrets = Vec::new();
    for listed in [json!(["files-b", "files-c"]), json!(["files-c"])] {
        let created = fixture
            .call(
                Method::POST,
                TOKENS,
                olivia(),
                Some(json!({"name": "ci", "scope": "READ", "expiresInDays": 5, "repositories": listed})),
            )
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
        secrets.push(created.json()["token"].as_str().unwrap().to_owned());
    }
    delete_repository(&fixture, "files-c").await;
    raw_repository(&fixture, "files-c").await;

    let mine = fixture.call(Method::GET, TOKENS, olivia(), None).await;
    assert_eq!(mine.json().as_array().unwrap().len(), 1);
    assert_eq!(mine.json()[0]["repositories"], json!(["files-b"]));
    assert_eq!(
        read_as(&fixture, "olivia", "files-b", &secrets[0]).await,
        StatusCode::OK
    );
    assert_eq!(
        read_as(&fixture, "olivia", "files-c", &secrets[0]).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        read_as(&fixture, "olivia", "files-c", &secrets[1]).await,
        StatusCode::UNAUTHORIZED
    );
    let deleted: Vec<_> = audit(&fixture, "security.accesstoken")
        .await
        .into_iter()
        .filter(|event| event.0 == "deleted")
        .collect();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].3["user"], "olivia");
    assert_eq!(deleted[0].3["reason"], "repository-deleted");
    assert_eq!(deleted[0].3["repositories"], json!(["files-c"]));
    assert!(deleted[0].3.get("customer").is_none());
    assert!(audit(&fixture, "security.customer").await.is_empty());
}
