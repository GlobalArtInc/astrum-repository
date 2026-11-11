mod accounts;
mod shop;
mod support;

use accounts::error_ids;
use axum::http::{Method, StatusCode};
use serde_json::json;
use shop::{CUSTOMERS, customer, issue, issue_body, read_as, shop};
use support::Auth;

const TOKENS: &str = "/service/rest/internal/access-tokens";

async fn read(fixture: &support::Fixture, repository: &str, token: &str) -> StatusCode {
    read_as(fixture, "acme", repository, token).await
}

#[tokio::test]
async fn revoking_takes_effect_on_the_next_request() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    let mut tokens = Vec::new();
    for _ in 0..3 {
        let reply = issue(
            &fixture,
            Auth::Admin,
            "acme",
            issue_body("admin", &["files-a"]),
        )
        .await;
        tokens.push((
            reply.json()["id"].as_str().unwrap().to_owned(),
            reply.json()["token"].as_str().unwrap().to_owned(),
        ));
    }
    for (_, token) in &tokens {
        assert_eq!(read(&fixture, "files-a", token).await, StatusCode::OK);
    }
    let one = fixture
        .call(
            Method::DELETE,
            &format!("{CUSTOMERS}/acme/tokens/{}", tokens[0].0),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(one.status, StatusCode::NO_CONTENT);
    assert_eq!(
        read(&fixture, "files-a", &tokens[0].1).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        read(&fixture, "files-a", &tokens[1].1).await,
        StatusCode::OK
    );
    let again = fixture
        .call(
            Method::DELETE,
            &format!("{CUSTOMERS}/acme/tokens/{}", tokens[0].0),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);

    let all = fixture
        .call(
            Method::POST,
            &format!("{CUSTOMERS}/acme/revoke"),
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(all.json(), json!({"revoked": 2}));
    for (_, token) in &tokens {
        assert_eq!(
            read(&fixture, "files-a", token).await,
            StatusCode::UNAUTHORIZED
        );
    }
    let kept = fixture
        .call(Method::GET, &format!("{CUSTOMERS}/acme"), Auth::Admin, None)
        .await;
    assert_eq!(kept.status, StatusCode::OK);
}

#[tokio::test]
async fn revoking_one_customer_leaves_the_tokens_of_another() {
    let fixture = shop().await;
    let (acme_id, acme_token) = fixture.customer_token("acme", &["files-a"]).await;
    let (globex_id, globex_token) = fixture.customer_token("globex", &["files-a"]).await;
    let globex_reads = || read_as(&fixture, "globex", "files-a", &globex_token);

    let foreign = fixture
        .call(
            Method::DELETE,
            &format!("{CUSTOMERS}/acme/tokens/{globex_id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(globex_reads().await, StatusCode::OK);

    let revoked = fixture
        .call(
            Method::POST,
            &format!("{CUSTOMERS}/acme/revoke"),
            Auth::Admin,
            Some(json!({})),
        )
        .await;
    assert_eq!(revoked.json(), json!({"revoked": 1}));
    assert_eq!(
        read(&fixture, "files-a", &acme_token).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(globex_reads().await, StatusCode::OK);

    let again = fixture
        .call(
            Method::DELETE,
            &format!("{CUSTOMERS}/acme/tokens/{acme_id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert_eq!(globex_reads().await, StatusCode::OK);
}

#[tokio::test]
async fn deleting_the_owner_deletes_the_customer_tokens_it_owns() {
    let fixture = shop().await;
    customer(&fixture, "acme").await;
    fixture
        .add_user("olivia", "correct horse", &["as-admin"])
        .await;
    let reply = issue(
        &fixture,
        Auth::Admin,
        "acme",
        issue_body("olivia", &["files-a"]),
    )
    .await;
    let token = reply.json()["token"].as_str().unwrap().to_owned();
    assert_eq!(read(&fixture, "files-a", &token).await, StatusCode::OK);
    let removed = fixture
        .call(
            Method::DELETE,
            "/service/rest/v1/security/users/olivia",
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.text());
    assert_eq!(
        read(&fixture, "files-a", &token).await,
        StatusCode::UNAUTHORIZED
    );
    let held = fixture
        .call(
            Method::GET,
            &format!("{CUSTOMERS}/acme/tokens"),
            Auth::Admin,
            None,
        )
        .await;
    assert!(held.json().as_array().unwrap().is_empty());
}

#[tokio::test]
async fn personal_tokens_may_be_limited_and_never_show_customer_tokens() {
    let fixture = shop().await;
    fixture
        .add_user("olivia", "correct horse", &["as-admin"])
        .await;
    let olivia = || Auth::Basic("olivia", "correct horse");
    let (customer_token_id, _) = fixture
        .customer_token_of("olivia", "acme", &["files-a"])
        .await;

    let listed = fixture.call(Method::GET, TOKENS, olivia(), None).await;
    assert!(listed.json().as_array().unwrap().is_empty());
    let removed = fixture
        .call(
            Method::DELETE,
            &format!("{TOKENS}/{customer_token_id}"),
            olivia(),
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NOT_FOUND, "{}", removed.text());

    let created = fixture
        .call(
            Method::POST,
            TOKENS,
            olivia(),
            Some(json!({"name": "ci", "scope": "READ", "expiresInDays": 5, "repositories": ["files-b"]})),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["repositories"], json!(["files-b"]));
    let secret = created.json()["token"].as_str().unwrap().to_owned();
    let mine = fixture.call(Method::GET, TOKENS, olivia(), None).await;
    assert_eq!(mine.json()[0]["repositories"], json!(["files-b"]));
    assert_eq!(
        shop::read_as(&fixture, "olivia", "files-b", &secret).await,
        StatusCode::OK
    );
    assert_eq!(
        shop::read_as(&fixture, "olivia", "files-a", &secret).await,
        StatusCode::FORBIDDEN
    );

    for (body, field) in [
        (
            json!({"name": "w", "scope": "WRITE", "expiresInDays": 5, "repositories": ["files-b"]}),
            "scope",
        ),
        (
            json!({"name": "x", "scope": "READ", "expiresInDays": 5, "repositories": ["nope"]}),
            "repositories",
        ),
        (
            json!({"name": "y", "scope": "READ", "expiresInDays": 5, "repositories": []}),
            "repositories",
        ),
    ] {
        let reply = fixture
            .call(Method::POST, TOKENS, olivia(), Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.text());
        assert_eq!(error_ids(&reply), [field]);
    }
}
