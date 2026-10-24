use astrum_security::api_keys::NUGET;
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::model::REALMS_KEY;
use astrum_security::{AuthError, SecretBox, Security, bootstrap};
use astrum_testkit::TestDatabase;

#[tokio::test]
async fn api_keys_are_kept_revealed_reset_and_checked_against_the_realm() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let data = tempfile::tempdir().unwrap();
    bootstrap(db.conn(), data.path()).await.unwrap();
    let security = Security::load(db.conn().clone()).await.unwrap();
    let secrets = SecretBox::from_key([7; 32]);

    assert_eq!(
        security.api_key_info(NUGET, ADMIN_USER).await.unwrap(),
        None
    );
    let first = security.api_key(&secrets, NUGET, ADMIN_USER).await.unwrap();
    assert!(first.created);
    assert_eq!(first.key.len(), 36);
    let again = security.api_key(&secrets, NUGET, ADMIN_USER).await.unwrap();
    assert!(!again.created);
    assert_eq!(again.key, first.key);
    assert!(
        security
            .api_key_info(NUGET, ADMIN_USER)
            .await
            .unwrap()
            .is_some()
    );

    assert!(matches!(
        security.authenticate_api_key(NUGET, &first.key).await,
        Err(AuthError::BadCredentials)
    ));
    astrum_db::setting::put(db.conn(), REALMS_KEY, &["AstrumAuthenticatingRealm", NUGET])
        .await
        .unwrap();
    security.changed().await.unwrap();
    let subject = security
        .authenticate_api_key(NUGET, &first.key)
        .await
        .unwrap();
    assert_eq!(subject.user_id, ADMIN_USER);
    assert_eq!(subject.realm, NUGET);
    let used = security.api_key_info(NUGET, ADMIN_USER).await.unwrap();
    assert!(used.unwrap().last_used_at.is_some());
    assert!(matches!(
        security.authenticate_api_key(NUGET, "wrong").await,
        Err(AuthError::BadCredentials)
    ));
    assert!(matches!(
        security
            .authenticate_api_key("RubyGemsApiKey", &first.key)
            .await,
        Err(AuthError::BadCredentials)
    ));

    assert!(security.delete_api_key(NUGET, ADMIN_USER).await.unwrap());
    assert!(!security.delete_api_key(NUGET, ADMIN_USER).await.unwrap());
    let reset = security.api_key(&secrets, NUGET, ADMIN_USER).await.unwrap();
    assert!(reset.created);
    assert_ne!(reset.key, first.key);
    assert!(
        security
            .authenticate_api_key(NUGET, &first.key)
            .await
            .is_err()
    );
    assert!(
        security
            .authenticate_api_key(NUGET, &reset.key)
            .await
            .is_ok()
    );

    assert_eq!(security.delete_api_keys_of(ADMIN_USER).await.unwrap(), 1);
    assert!(
        security
            .authenticate_api_key(NUGET, &reset.key)
            .await
            .is_err()
    );
}
