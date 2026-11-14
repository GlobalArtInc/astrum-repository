mod support;

use std::net::IpAddr;

use astrum_db::entity::{access_token, customer, security_user};
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::tokens::MAX_REPOSITORIES;
use astrum_security::{AuthError, NewToken, Scope, TokenError};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use support::{customer_token, customer_world, issue};

#[tokio::test]
async fn a_customer_token_is_narrowed_and_answers_to_the_owner_or_the_customer() {
    let world = customer_world().await;
    let token = issue(&world, &["Raw-A", "raw-b", "raw-a"]).await;
    let subject = world
        .security
        .authenticate_token(Some("acme"), &token, None)
        .await
        .unwrap();
    assert!(subject.narrowed());
    assert_eq!(subject.scope, Scope::Read);
    assert_eq!(subject.user_id, ADMIN_USER);
    assert_eq!(
        subject.token.as_ref().unwrap().customer.as_deref(),
        Some("acme")
    );
    assert!(subject.is_permitted("astrum:repository-view:raw:raw-a:read"));
    assert!(subject.is_permitted("astrum:repository-view:raw:raw-b:browse"));
    assert!(!subject.is_permitted("astrum:repository-view:raw:raw-c:read"));
    assert!(!subject.is_permitted("astrum:repository-view:raw:raw-a:add"));
    assert!(!subject.is_permitted("astrum:users:read"));
    assert!(!subject.may_add_content());

    let by_owner = world
        .security
        .authenticate_token(Some(ADMIN_USER), &token, None)
        .await
        .unwrap();
    assert!(by_owner.narrowed());
    let anonymous_name = world.security.authenticate_token(None, &token, None).await;
    assert!(anonymous_name.unwrap().narrowed());
    for other in ["jane", "ACME", ""] {
        assert!(matches!(
            world
                .security
                .authenticate_token(Some(other), &token, None)
                .await,
            Err(AuthError::BadCredentials)
        ));
    }
    let stored = access_token::Entity::find()
        .one(world.db.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.repositories.unwrap(), vec!["raw-a", "raw-b"]);
    assert!(world.security.tokens(None).await.unwrap().is_empty());
}

#[tokio::test]
async fn expired_revoked_and_orphaned_customer_tokens_are_refused() {
    let world = customer_world().await;
    let refused = |token: String| {
        let security = &world.security;
        async move {
            matches!(
                security
                    .authenticate_token(Some("acme"), &token, None)
                    .await,
                Err(AuthError::BadCredentials)
            )
        }
    };

    let expired = issue(&world, &["raw-a"]).await;
    access_token::Entity::update_many()
        .col_expr(
            access_token::Column::ExpiresAt,
            sea_orm::sea_query::Expr::value(
                chrono::Utc::now().fixed_offset() - chrono::Duration::days(1),
            ),
        )
        .exec(world.db.conn())
        .await
        .unwrap();
    assert!(refused(expired).await);

    let (info, revoked) = world
        .security
        .create_token(ADMIN_USER, customer_token(&["raw-a"]))
        .await
        .unwrap();
    assert!(
        world
            .security
            .authenticate_token(Some("acme"), &revoked, None)
            .await
            .is_ok()
    );
    assert!(
        world
            .security
            .revoke_customer_token("acme", info.id)
            .await
            .unwrap()
    );
    assert!(refused(revoked).await);

    let disabled_owner = issue(&world, &["raw-a"]).await;
    security_user::Entity::update_many()
        .col_expr(
            security_user::Column::Status,
            sea_orm::sea_query::Expr::value("disabled"),
        )
        .filter(security_user::Column::Id.eq(ADMIN_USER))
        .exec(world.db.conn())
        .await
        .unwrap();
    world.security.changed().await.unwrap();
    assert!(refused(disabled_owner).await);
    security_user::Entity::update_many()
        .col_expr(
            security_user::Column::Status,
            sea_orm::sea_query::Expr::value("active"),
        )
        .filter(security_user::Column::Id.eq(ADMIN_USER))
        .exec(world.db.conn())
        .await
        .unwrap();
    world.security.changed().await.unwrap();

    let orphaned = issue(&world, &["raw-a"]).await;
    customer::Entity::delete_by_id("acme")
        .exec(world.db.conn())
        .await
        .unwrap();
    assert!(refused(orphaned).await);
    assert_eq!(
        access_token::Entity::find()
            .count(world.db.conn())
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn the_address_is_stored_once_per_touch_interval() {
    let world = customer_world().await;
    let token = issue(&world, &["raw-a"]).await;
    let first: IpAddr = "198.51.100.7".parse().unwrap();
    let second: IpAddr = "198.51.100.8".parse().unwrap();
    let stored = || async {
        access_token::Entity::find()
            .one(world.db.conn())
            .await
            .unwrap()
            .unwrap()
            .last_used_ip
    };
    assert_eq!(stored().await, None);
    world
        .security
        .authenticate_token(None, &token, Some(first))
        .await
        .unwrap();
    assert_eq!(stored().await.as_deref(), Some("198.51.100.7"));
    world
        .security
        .authenticate_token(None, &token, Some(second))
        .await
        .unwrap();
    assert_eq!(stored().await.as_deref(), Some("198.51.100.7"));
    access_token::Entity::update_many()
        .col_expr(
            access_token::Column::LastUsedAt,
            sea_orm::sea_query::Expr::value(
                chrono::Utc::now().fixed_offset() - chrono::Duration::minutes(5),
            ),
        )
        .exec(world.db.conn())
        .await
        .unwrap();
    world
        .security
        .authenticate_token(None, &token, None)
        .await
        .unwrap();
    assert_eq!(stored().await.as_deref(), Some("198.51.100.7"));
    access_token::Entity::update_many()
        .col_expr(
            access_token::Column::LastUsedAt,
            sea_orm::sea_query::Expr::value(
                chrono::Utc::now().fixed_offset() - chrono::Duration::minutes(5),
            ),
        )
        .exec(world.db.conn())
        .await
        .unwrap();
    world
        .security
        .authenticate_token(None, &token, Some(second))
        .await
        .unwrap();
    assert_eq!(stored().await.as_deref(), Some("198.51.100.8"));
}

#[tokio::test]
async fn a_disabled_customer_token_is_refused_and_kept() {
    let world = customer_world().await;
    let token = issue(&world, &["raw-a"]).await;
    let works = || async {
        world
            .security
            .authenticate_token(Some("acme"), &token, None)
            .await
            .is_ok()
    };
    assert!(works().await);
    for (disabled, expected) in [(true, false), (false, true)] {
        customer::Entity::update_many()
            .col_expr(
                customer::Column::Disabled,
                sea_orm::sea_query::Expr::value(disabled),
            )
            .exec(world.db.conn())
            .await
            .unwrap();
        assert_eq!(works().await, expected, "disabled = {disabled}");
        assert_eq!(
            access_token::Entity::find()
                .count(world.db.conn())
                .await
                .unwrap(),
            1
        );
    }
}

#[tokio::test]
async fn token_requests_are_validated() {
    let world = customer_world().await;
    let create = |new: NewToken| world.security.create_token(ADMIN_USER, new);
    let write = NewToken {
        scope: Scope::Write,
        ..customer_token(&["raw-a"])
    };
    assert!(matches!(create(write).await, Err(TokenError::Scope)));
    let personal_write = NewToken {
        customer: None,
        scope: Scope::Write,
        ..customer_token(&["raw-a"])
    };
    assert!(matches!(
        create(personal_write).await,
        Err(TokenError::Scope)
    ));
    for bad in ["*", "a:b", "a,b", "", " a", "a*", "_a", "a/b"] {
        assert!(
            matches!(
                create(customer_token(&[bad])).await,
                Err(TokenError::Repositories(_))
            ),
            "{bad:?}"
        );
    }
    assert!(matches!(
        create(customer_token(&[])).await,
        Err(TokenError::Repositories(_))
    ));
    let many: Vec<String> = (0..=MAX_REPOSITORIES)
        .map(|n| format!("repo-{n}"))
        .collect();
    let many: Vec<&str> = many.iter().map(String::as_str).collect();
    assert!(matches!(
        create(customer_token(&many)).await,
        Err(TokenError::Repositories(_))
    ));
    let no_expiry = NewToken {
        days: None,
        ..customer_token(&["raw-a"])
    };
    assert!(matches!(
        create(no_expiry).await,
        Err(TokenError::CustomerExpiry)
    ));
    let no_list = NewToken {
        repositories: None,
        ..customer_token(&["raw-a"])
    };
    assert!(matches!(
        create(no_list).await,
        Err(TokenError::Repositories(_))
    ));
    let unknown = NewToken {
        customer: Some("nobody".to_owned()),
        ..customer_token(&["raw-a"])
    };
    assert!(matches!(
        create(unknown).await,
        Err(TokenError::UnknownCustomer)
    ));
    assert_eq!(
        access_token::Entity::find()
            .count(world.db.conn())
            .await
            .unwrap(),
        0
    );
}
