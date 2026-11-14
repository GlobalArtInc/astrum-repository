use astrum_db::entity::content_policy;
use astrum_policy::Error;
use astrum_policy::exception::{self, Exception, Rule};
use astrum_policy::key::{self, TrustedKey};
use astrum_policy::policy::{
    self, LicenseList, LicenseRule, Malicious, Mode, Policy, Severity, SignatureRule,
    UnknownLicense, Unscanned, VulnerabilityRule,
};
use astrum_testkit::TestDatabase;
use chrono::{SubsecRound, TimeDelta, Utc};
use sea_orm::EntityTrait;
use sea_orm::sea_query::Expr;
use uuid::Uuid;

fn strict(name: &str) -> Policy {
    Policy {
        name: name.to_owned(),
        notes: "Blocks critical findings".to_owned(),
        vulnerabilities: VulnerabilityRule {
            mode: Mode::Block,
            severity: Some(Severity::Critical),
            malicious: Malicious::Threshold,
            unscanned: Unscanned::Block,
            notify: Some(Severity::High),
        },
        licenses: LicenseRule {
            mode: Mode::Report,
            list: LicenseList::Deny,
            ids: vec!["GPL-3.0-only".to_owned()],
            unknown: UnknownLicense::Block,
        },
        signatures: SignatureRule {
            mode: Mode::Off,
            keys: vec!["release".to_owned()],
        },
        revision: 1,
    }
}

#[tokio::test]
async fn content_policies_round_trip_and_bump_their_revision_on_update() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &strict("strict")).await.unwrap();
    policy::create(conn, &strict("audit")).await.unwrap();
    assert_eq!(
        policy::find(conn, "strict").await.unwrap(),
        Some(strict("strict"))
    );
    let names: Vec<String> = policy::list(conn)
        .await
        .unwrap()
        .into_iter()
        .map(|policy| policy.name)
        .collect();
    assert_eq!(names, ["audit", "strict"]);
    let duplicate = policy::create(conn, &strict("STRICT")).await;
    assert!(
        matches!(duplicate, Err(Error::Duplicate(_))),
        "{duplicate:?}"
    );

    let mut changed = strict("strict");
    changed.notes = "Blocks everything".to_owned();
    changed.licenses.mode = Mode::Block;
    policy::update(conn, &changed).await.unwrap();
    let found = policy::find(conn, "strict").await.unwrap().unwrap();
    assert_eq!(found.notes, "Blocks everything");
    assert_eq!(found.licenses.mode, Mode::Block);
    assert_eq!(found.revision, 2);
    let missing = policy::update(conn, &strict("nobody")).await;
    assert!(matches!(missing, Err(Error::NotFound(_))), "{missing:?}");

    assert!(policy::delete(conn, "audit").await.unwrap());
    assert!(!policy::delete(conn, "audit").await.unwrap());
    assert_eq!(policy::find(conn, "audit").await.unwrap(), None);
}

#[tokio::test]
async fn policy_exceptions_round_trip_with_and_without_a_policy() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &strict("strict")).await.unwrap();
    let now = Utc::now().trunc_subsecs(6);
    let for_policy = Exception {
        id: Uuid::new_v4(),
        policy: Some("strict".to_owned()),
        rule: Rule::License,
        advisory: None,
        license: Some("GPL-3.0-only".to_owned()),
        format: Some("npm".to_owned()),
        namespace: None,
        name: Some("left-pad".to_owned()),
        version: None,
        digest: None,
        reason: "Build tool only".to_owned(),
        expires_at: Some(now + TimeDelta::days(30)),
        created_by: "alice".to_owned(),
        created_at: now,
    };
    let for_all = Exception {
        id: Uuid::new_v4(),
        policy: None,
        rule: Rule::Vulnerability,
        advisory: Some("GHSA-xxxx-xxxx-xxxx".to_owned()),
        license: None,
        format: None,
        namespace: None,
        name: None,
        version: None,
        digest: None,
        reason: "Not reachable".to_owned(),
        expires_at: None,
        created_by: "bob".to_owned(),
        created_at: now + TimeDelta::seconds(1),
    };
    exception::create(conn, &for_policy).await.unwrap();
    exception::create(conn, &for_all).await.unwrap();
    assert_eq!(
        exception::find(conn, for_policy.id).await.unwrap(),
        Some(for_policy.clone())
    );
    assert_eq!(
        exception::list(conn).await.unwrap(),
        [for_policy.clone(), for_all.clone()]
    );
    let duplicate = exception::create(conn, &for_all).await;
    assert!(
        matches!(duplicate, Err(Error::Duplicate(_))),
        "{duplicate:?}"
    );

    assert!(exception::delete(conn, for_policy.id).await.unwrap());
    assert!(!exception::delete(conn, for_policy.id).await.unwrap());
    assert_eq!(exception::find(conn, for_policy.id).await.unwrap(), None);
}

#[tokio::test]
async fn trusted_keys_round_trip_and_are_unique_by_name_and_fingerprint() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let release = TrustedKey {
        name: "release".to_owned(),
        algorithm: "ed25519".to_owned(),
        public_key: "-----BEGIN PUBLIC KEY-----".to_owned(),
        fingerprint: "SHA256:release".to_owned(),
        notes: "Release signing".to_owned(),
        created_at: Utc::now().trunc_subsecs(6),
    };
    key::create(conn, &release).await.unwrap();
    assert_eq!(
        key::find(conn, "release").await.unwrap(),
        Some(release.clone())
    );
    assert_eq!(key::list(conn).await.unwrap(), vec![release.clone()]);
    let same_fingerprint = TrustedKey {
        name: "other".to_owned(),
        ..release.clone()
    };
    let refused = key::create(conn, &same_fingerprint).await;
    assert!(matches!(refused, Err(Error::Duplicate(_))), "{refused:?}");

    assert!(key::delete(conn, "release").await.unwrap());
    assert!(!key::delete(conn, "release").await.unwrap());
    assert_eq!(key::find(conn, "release").await.unwrap(), None);
}

fn advised(policy: Option<&str>) -> Exception {
    Exception {
        id: Uuid::new_v4(),
        policy: policy.map(str::to_owned),
        rule: Rule::Vulnerability,
        advisory: Some("GHSA-xxxx-xxxx-xxxx".to_owned()),
        license: None,
        format: None,
        namespace: None,
        name: None,
        version: None,
        digest: None,
        reason: "Not reachable".to_owned(),
        expires_at: None,
        created_by: "bob".to_owned(),
        created_at: Utc::now().trunc_subsecs(6),
    }
}

#[tokio::test]
async fn trusted_key_names_are_unique_ignoring_case() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let release = TrustedKey {
        name: "release".to_owned(),
        algorithm: "ed25519".to_owned(),
        public_key: "-----BEGIN PUBLIC KEY-----".to_owned(),
        fingerprint: "SHA256:release".to_owned(),
        notes: String::new(),
        created_at: Utc::now().trunc_subsecs(6),
    };
    key::create(conn, &release).await.unwrap();
    let other_case = TrustedKey {
        name: "RELEASE".to_owned(),
        fingerprint: "SHA256:other".to_owned(),
        ..release.clone()
    };
    let refused = key::create(conn, &other_case).await;
    assert!(matches!(refused, Err(Error::Duplicate(_))), "{refused:?}");
    assert_eq!(key::list(conn).await.unwrap(), vec![release]);
}

#[tokio::test]
async fn a_private_key_is_refused_as_a_trusted_key() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    let private = TrustedKey {
        name: "signer".to_owned(),
        algorithm: "ed25519".to_owned(),
        public_key: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEI\n-----END PRIVATE KEY-----"
            .to_owned(),
        fingerprint: "SHA256:signer".to_owned(),
        notes: String::new(),
        created_at: Utc::now().trunc_subsecs(6),
    };
    let refused = key::create(conn, &private).await;
    assert!(matches!(refused, Err(Error::Invalid(_))), "{refused:?}");
    assert!(key::list(conn).await.unwrap().is_empty());
}

#[tokio::test]
async fn exceptions_are_checked_before_they_are_stored() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &strict("strict")).await.unwrap();
    let base = advised(None);
    let no_reason = Exception {
        reason: "  ".to_owned(),
        ..base.clone()
    };
    let refused = exception::create(conn, &no_reason).await;
    assert!(matches!(refused, Err(Error::Invalid(_))), "{refused:?}");
    let empty_advisory = Exception {
        advisory: Some(String::new()),
        ..base.clone()
    };
    let refused = exception::create(conn, &empty_advisory).await;
    assert!(matches!(refused, Err(Error::Invalid(_))), "{refused:?}");
    let no_target = Exception {
        advisory: None,
        ..base.clone()
    };
    let refused = exception::create(conn, &no_target).await;
    assert!(matches!(refused, Err(Error::Invalid(_))), "{refused:?}");
    let unknown = Exception {
        policy: Some("lenient".to_owned()),
        ..base.clone()
    };
    let refused = exception::create(conn, &unknown).await;
    assert!(matches!(refused, Err(Error::NotFound(_))), "{refused:?}");
    assert!(exception::list(conn).await.unwrap().is_empty());
}

#[tokio::test]
async fn deleting_a_policy_takes_its_exceptions_but_keeps_the_global_ones() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &strict("strict")).await.unwrap();
    let scoped = advised(Some("strict"));
    let global = advised(None);
    exception::create(conn, &scoped).await.unwrap();
    exception::create(conn, &global).await.unwrap();
    assert!(policy::delete(conn, "strict").await.unwrap());
    assert_eq!(exception::list(conn).await.unwrap(), vec![global]);
}

#[tokio::test]
async fn a_row_with_fields_of_a_newer_version_is_still_readable() {
    let test = TestDatabase::create().await;
    let db = test.migrated().await;
    let conn = db.conn();
    policy::create(conn, &strict("strict")).await.unwrap();
    content_policy::Entity::update_many()
        .col_expr(
            content_policy::Column::Vulnerabilities,
            Expr::value(serde_json::json!({"mode": "block", "cooldown": {"days": 3}})),
        )
        .exec(conn)
        .await
        .unwrap();
    let found = policy::find(conn, "strict").await.unwrap().unwrap();
    assert_eq!(found.vulnerabilities.mode, Mode::Block);
    assert_eq!(policy::list(conn).await.unwrap().len(), 1);
}
