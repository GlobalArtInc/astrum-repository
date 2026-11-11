mod support;

use astrum_db::entity::{access_token, customer};
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::tokens::{MAX_PER_CUSTOMER, MAX_PER_USER};
use astrum_security::{NewToken, Scope, TokenError};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use support::{customer_token, issue, world};

#[tokio::test]
async fn the_database_refuses_customer_tokens_that_are_not_narrowed_reads() {
    let world = world().await;
    let insert = |scope: &str, customer: &str, repositories: &str, expires: &str| {
        format!(
            "INSERT INTO access_token (id, user_id, name, token_hash, token_hint, scope, customer, repositories, expires_at) \
             VALUES (gen_random_uuid(), 'admin', 'x', md5(random()::text), 'astrum_xxxx', '{scope}', {customer}, {repositories}, {expires})"
        )
    };
    let future = "now() + interval '1 day'";
    let shape = "access_token_customer_shape";
    let reads = "access_token_narrowed_reads";
    let cases = [
        (
            "WRITE with repositories",
            insert("WRITE", "NULL", "ARRAY['a']", future),
            reads,
        ),
        (
            "WRITE customer",
            insert("WRITE", "'acme'", "ARRAY['a']", future),
            shape,
        ),
        (
            "customer without repositories",
            insert("READ", "'acme'", "NULL", future),
            shape,
        ),
        (
            "customer with empty list",
            insert("READ", "'acme'", "ARRAY[]::text[]", future),
            shape,
        ),
        (
            "customer without expiry",
            insert("READ", "'acme'", "ARRAY['a']", "NULL"),
            shape,
        ),
        (
            "empty list on a personal token",
            insert("READ", "NULL", "ARRAY[]::text[]", future),
            reads,
        ),
    ];
    for (name, sql, constraint) in cases {
        let error = world
            .db
            .conn()
            .execute_unprepared(&sql)
            .await
            .expect_err(name)
            .to_string();
        assert!(error.contains(constraint), "{name}: {error}");
    }
    let allowed = [
        insert("READ", "'acme'", "ARRAY['a']", future),
        insert("READ", "NULL", "ARRAY['a']", future),
        insert("WRITE", "NULL", "NULL", "NULL"),
    ];
    for sql in allowed {
        world.db.conn().execute_unprepared(&sql).await.unwrap();
    }
}

#[tokio::test]
async fn a_customer_holds_twenty_tokens_and_they_do_not_count_against_the_owner() {
    let world = world().await;
    for _ in 0..MAX_PER_CUSTOMER {
        issue(&world, &["raw-a"]).await;
    }
    assert!(matches!(
        world
            .security
            .create_token(ADMIN_USER, customer_token(&["raw-a"]))
            .await,
        Err(TokenError::TooManyForCustomer)
    ));
    for n in 0..MAX_PER_USER {
        world
            .security
            .create_token(
                ADMIN_USER,
                NewToken::personal(&format!("t{n}"), Scope::Write, None),
            )
            .await
            .unwrap();
    }
    assert!(matches!(
        world
            .security
            .create_token(
                ADMIN_USER,
                NewToken::personal("one too many", Scope::Write, None)
            )
            .await,
        Err(TokenError::TooMany)
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_creations_cannot_pass_the_customer_limit() {
    let world = world().await;
    for _ in 1..MAX_PER_CUSTOMER {
        issue(&world, &["raw-a"]).await;
    }
    let results = futures_util::future::join_all((0..8).map(|_| {
        world
            .security
            .create_token(ADMIN_USER, customer_token(&["raw-a"]))
    }))
    .await;
    let created = results.iter().filter(|result| result.is_ok()).count();
    assert_eq!(created, 1, "{results:?}");
    let held = access_token::Entity::find()
        .filter(access_token::Column::Customer.eq("acme"))
        .count(world.db.conn())
        .await
        .unwrap();
    assert_eq!(held, MAX_PER_CUSTOMER);
}

#[tokio::test]
async fn customer_tokens_are_managed_through_their_customer_only() {
    let world = world().await;
    customer::Entity::insert(customer::ActiveModel {
        id: ActiveValue::Set("globex".to_owned()),
        name: ActiveValue::Set("Globex".to_owned()),
        reference: ActiveValue::Set(String::new()),
        note: ActiveValue::Set(String::new()),
        disabled: ActiveValue::Set(false),
        created_by: ActiveValue::Set("admin".to_owned()),
        created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    })
    .exec(world.db.conn())
    .await
    .unwrap();
    let (first, _) = world
        .security
        .create_token(ADMIN_USER, customer_token(&["raw-a"]))
        .await
        .unwrap();
    issue(&world, &["raw-a"]).await;
    let globex = NewToken {
        customer: Some("globex".to_owned()),
        ..customer_token(&["raw-a"])
    };
    let (other, _) = world
        .security
        .create_token(ADMIN_USER, globex)
        .await
        .unwrap();

    assert!(world.security.token(first.id).await.unwrap().is_none());
    assert!(!world.security.delete_token(first.id).await.unwrap());
    assert_eq!(
        world.security.customer_tokens("acme").await.unwrap().len(),
        2
    );
    assert!(
        !world
            .security
            .revoke_customer_token("globex", first.id)
            .await
            .unwrap()
    );
    assert!(
        !world
            .security
            .revoke_customer_token("acme", other.id)
            .await
            .unwrap()
    );
    assert_eq!(
        access_token::Entity::find()
            .count(world.db.conn())
            .await
            .unwrap(),
        3
    );
    assert!(
        world
            .security
            .revoke_customer_token("acme", first.id)
            .await
            .unwrap()
    );
    assert_eq!(
        world.security.revoke_customer_tokens("acme").await.unwrap(),
        1
    );
    assert_eq!(
        world
            .security
            .customer_tokens("globex")
            .await
            .unwrap()
            .len(),
        1
    );
}
