use std::time::Duration;

use astrum_security::SecretBox;
use astrum_security::ldap::{
    Check, Directory, Draft, Ldap, LdapAuthError, LdapServer, Outcome, TlsClient, config,
};
use astrum_testkit::ldap::{BASE, LdapTestServer, SYSTEM_DN, SYSTEM_PASSWORD, SYSTEM_USER};

fn secrets() -> SecretBox {
    SecretBox::from_key([3; 32])
}

fn draft(port: u16) -> Draft {
    Draft {
        id: Some("one".to_owned()),
        name: Some("directory".to_owned()),
        protocol: Some("LDAP".to_owned()),
        host: Some("127.0.0.1".to_owned()),
        port: Some(i64::from(port)),
        search_base: Some(BASE.to_owned()),
        auth_scheme: Some("SIMPLE".to_owned()),
        auth_username: Some(SYSTEM_DN.to_owned()),
        connection_timeout_seconds: Some(5),
        user_base_dn: Some("ou=people".to_owned()),
        user_object_class: Some("inetOrgPerson".to_owned()),
        user_id_attribute: Some("uid".to_owned()),
        user_real_name_attribute: Some("cn".to_owned()),
        user_email_address_attribute: Some("mail".to_owned()),
        ldap_groups_as_roles: Some(true),
        group_type: Some("dynamic".to_owned()),
        user_member_of_attribute: Some("memberOf".to_owned()),
        ..Draft::default()
    }
}

fn server(draft: Draft) -> LdapServer {
    let mut server = config::check(&draft, Check::Everything).unwrap();
    server.auth_password = Some(secrets().seal(SYSTEM_PASSWORD));
    server
}

fn trusting(test: &LdapTestServer) -> TlsClient {
    TlsClient::with_roots(vec![test.ca()])
}

async fn verify(
    server: &LdapServer,
    password: Option<&str>,
    tls: &TlsClient,
) -> Result<(), String> {
    Directory {
        server,
        bind_password: password,
        tls,
    }
    .verify_connection()
    .await
    .map_err(|error| error.to_string())
}

#[tokio::test]
async fn connections_verify_over_every_transport() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    let plain = server(draft(test.port));
    verify(&plain, Some(SYSTEM_PASSWORD), &tls).await.unwrap();

    let ldaps = server(Draft {
        protocol: Some("LDAPS".to_owned()),
        host: Some("localhost".to_owned()),
        ..draft(test.ldaps_port)
    });
    verify(&ldaps, Some(SYSTEM_PASSWORD), &tls).await.unwrap();
    let untrusted = verify(
        &ldaps,
        Some(SYSTEM_PASSWORD),
        &TlsClient::with_roots(vec![]),
    )
    .await
    .unwrap_err();
    assert!(untrusted.contains("TLS handshake"), "{untrusted}");

    let start_tls = server(Draft {
        use_start_tls: Some(true),
        ..draft(test.port)
    });
    verify(&start_tls, Some(SYSTEM_PASSWORD), &tls)
        .await
        .unwrap();

    let anonymous = server(Draft {
        auth_scheme: Some("NONE".to_owned()),
        ..draft(test.port)
    });
    verify(&anonymous, None, &tls).await.unwrap();
}

#[tokio::test]
async fn connection_problems_are_explained() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    let plain = server(draft(test.port));
    let wrong = verify(&plain, Some("wrong"), &tls).await.unwrap_err();
    assert_eq!(
        wrong,
        "The LDAP server rejected the username or password of the system account"
    );
    let missing = verify(&plain, None, &tls).await.unwrap_err();
    assert!(missing.starts_with("No password is set"), "{missing}");
    let no_base = server(Draft {
        search_base: Some("dc=missing,dc=org".to_owned()),
        ..draft(test.port)
    });
    let refused = verify(&no_base, Some(SYSTEM_PASSWORD), &tls)
        .await
        .unwrap_err();
    assert_eq!(
        refused,
        "The search base 'dc=missing,dc=org' does not exist"
    );
    test.stop();
    tokio::time::sleep(Duration::from_millis(50)).await;
    let down = verify(&plain, Some(SYSTEM_PASSWORD), &tls)
        .await
        .unwrap_err();
    assert!(down.starts_with("could not connect to 127.0.0.1"), "{down}");
}

#[tokio::test]
async fn sasl_mechanisms_bind_the_system_account() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    for scheme in ["CRAM-MD5", "DIGEST-MD5"] {
        let sasl = server(Draft {
            auth_scheme: Some(scheme.to_owned()),
            auth_username: Some(SYSTEM_USER.to_owned()),
            ..draft(test.port)
        });
        verify(&sasl, Some(SYSTEM_PASSWORD), &tls)
            .await
            .unwrap_or_else(|error| panic!("{scheme}: {error}"));
        let wrong = verify(&sasl, Some("wrong"), &tls).await.unwrap_err();
        assert!(wrong.contains("rejected"), "{scheme}: {wrong}");
    }
}

#[tokio::test]
async fn users_authenticate_and_bring_their_groups() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    let dynamic = server(draft(test.port));
    let directory = Directory {
        server: &dynamic,
        bind_password: Some(SYSTEM_PASSWORD),
        tls: &tls,
    };
    let Outcome::Authenticated(alice) = directory
        .authenticate("alice", "alice-secret")
        .await
        .unwrap()
    else {
        panic!("alice signs in");
    };
    assert_eq!(alice.username, "alice");
    assert_eq!(alice.real_name.as_deref(), Some("Alice Smith"));
    assert_eq!(alice.email.as_deref(), Some("alice@example.com"));
    assert_eq!(alice.groups, ["admins", "developers"]);
    assert_eq!(
        directory.authenticate("alice", "nope").await.unwrap(),
        Outcome::IncorrectPassword
    );
    assert_eq!(
        directory.authenticate("alice", "").await.unwrap(),
        Outcome::IncorrectPassword
    );
    assert_eq!(
        directory.authenticate("nobody", "x").await.unwrap(),
        Outcome::UserNotFound
    );
    assert_eq!(
        directory.authenticate("dave", "dave-secret").await.unwrap(),
        Outcome::UserNotFound
    );

    let subtree = server(Draft {
        user_subtree: Some(true),
        user_password_attribute: Some("userPassword".to_owned()),
        ..draft(test.port)
    });
    let directory = Directory {
        server: &subtree,
        bind_password: Some(SYSTEM_PASSWORD),
        tls: &tls,
    };
    assert!(matches!(
        directory.authenticate("dave", "dave-secret").await.unwrap(),
        Outcome::Authenticated(_)
    ));
    let Outcome::Authenticated(bob) = directory.authenticate("bob", "bob-secret").await.unwrap()
    else {
        panic!("bob signs in through his {{SSHA}} password attribute");
    };
    assert_eq!(bob.groups, ["developers"]);
    assert_eq!(
        directory.authenticate("bob", "wrong").await.unwrap(),
        Outcome::IncorrectPassword
    );
}

#[tokio::test]
async fn static_groups_list_their_members() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    let static_groups = server(Draft {
        group_type: Some("static".to_owned()),
        group_base_dn: Some("ou=groups".to_owned()),
        group_object_class: Some("groupOfUniqueNames".to_owned()),
        group_id_attribute: Some("cn".to_owned()),
        group_member_attribute: Some("uniqueMember".to_owned()),
        group_member_format: Some("${dn}".to_owned()),
        user_member_of_attribute: None,
        ..draft(test.port)
    });
    let directory = Directory {
        server: &static_groups,
        bind_password: Some(SYSTEM_PASSWORD),
        tls: &tls,
    };
    let alice = directory.find_user("ALICE").await.unwrap().unwrap();
    assert_eq!(alice.groups, ["admins", "developers"]);
    let carol = directory.find_user("carol").await.unwrap().unwrap();
    assert!(carol.groups.is_empty());
    assert_eq!(
        directory.search_groups(None, 100).await.unwrap(),
        ["developers", "admins"]
    );
    assert_eq!(
        directory.search_groups(Some("dev"), 100).await.unwrap(),
        ["developers"]
    );
    assert!(directory.group_exists("Admins").await.unwrap());
    assert!(!directory.group_exists("nobody").await.unwrap());

    let users = directory.search_users(Some("a"), 20).await.unwrap();
    assert_eq!(
        users
            .iter()
            .map(|user| user.username.as_str())
            .collect::<Vec<_>>(),
        ["alice"]
    );
    let everyone = directory.search_users(None, 2).await.unwrap();
    assert_eq!(everyone.len(), 2, "the size limit holds");
}

#[tokio::test]
async fn dynamic_groups_come_from_the_directory_tree() {
    let test = LdapTestServer::start().await;
    let tls = trusting(&test);
    let dynamic = server(draft(test.port));
    let directory = Directory {
        server: &dynamic,
        bind_password: Some(SYSTEM_PASSWORD),
        tls: &tls,
    };
    assert_eq!(
        directory.search_groups(None, 100).await.unwrap(),
        ["developers", "admins"]
    );
    assert!(directory.group_exists("developers").await.unwrap());
}

#[tokio::test]
async fn the_manager_caches_and_falls_over() {
    let first = LdapTestServer::start().await;
    let second = LdapTestServer::start().await;
    let ldap = Ldap::new();
    ldap.set_tls(trusting(&first));
    let secrets = secrets();
    let mut one = server(draft(first.port));
    one.max_incidents_count = 2;
    one.connection_retry_delay_seconds = 60;
    let two = server(Draft {
        id: Some("two".to_owned()),
        name: Some("backup".to_owned()),
        ..draft(second.port)
    });
    let servers = vec![one, two];

    let alice = ldap
        .authenticate(&servers, Some(&secrets), "alice", "alice-secret")
        .await
        .unwrap();
    assert_eq!(alice.groups, ["admins", "developers"]);
    let binds = first.binds();
    ldap.authenticate(&servers, Some(&secrets), "alice", "alice-secret")
        .await
        .unwrap();
    assert_eq!(
        first.binds(),
        binds,
        "a repeated sign-in is served from the cache"
    );
    assert_eq!(
        ldap.authenticate(&servers, Some(&secrets), "alice", "other")
            .await
            .unwrap_err(),
        LdapAuthError::IncorrectPassword
    );
    assert_eq!(
        ldap.authenticate(&servers, Some(&secrets), "ghost", "x")
            .await
            .unwrap_err(),
        LdapAuthError::NotFound
    );
    let searches = first.searches();
    assert!(
        ldap.user(&servers, Some(&secrets), "alice")
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(first.searches(), searches, "the user lookup is cached too");

    first.stop();
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        ldap.user(&servers, Some(&secrets), "alice")
            .await
            .unwrap()
            .is_some(),
        "a cached user survives an outage"
    );
    ldap.clear();
    let from_backup = ldap
        .authenticate(&servers, Some(&secrets), "bob", "bob-secret")
        .await
        .unwrap();
    assert_eq!(from_backup.username, "bob");
    let backup_binds = second.binds();
    ldap.clear();
    ldap.authenticate(&servers, Some(&secrets), "carol", "carol-secret")
        .await
        .unwrap();
    assert!(second.binds() > backup_binds);
    ldap.authenticate(&servers, Some(&secrets), "carol", "carol-secret")
        .await
        .unwrap();
    ldap.clear();
    second.stop();
    tokio::time::sleep(Duration::from_millis(50)).await;
    let failed = ldap
        .authenticate(&servers, Some(&secrets), "carol", "carol-secret")
        .await
        .unwrap_err();
    assert!(
        matches!(failed, LdapAuthError::Unavailable(_)),
        "{failed:?}"
    );
    let failed = ldap
        .user(&servers[..1], Some(&secrets), "dave")
        .await
        .unwrap_err();
    assert!(failed.0.starts_with("could not connect"), "{failed}");
    let skipped = ldap
        .user(&servers[..1], Some(&secrets), "dave")
        .await
        .unwrap_err();
    assert_eq!(
        skipped.0,
        "LDAP server directory is skipped after repeated failures"
    );

    let users = ldap.search_users(&servers, Some(&secrets), None, 10).await;
    assert!(
        users.is_empty(),
        "unreachable servers are left out of searches"
    );
}

#[tokio::test]
async fn a_short_cache_asks_again() {
    let test = LdapTestServer::start().await;
    let ldap = Ldap::new();
    ldap.set_tls(trusting(&test));
    ldap.set_cache_ttl(Duration::ZERO);
    let secrets = secrets();
    let servers = vec![server(draft(test.port))];
    ldap.authenticate(&servers, Some(&secrets), "alice", "alice-secret")
        .await
        .unwrap();
    let binds = test.binds();
    ldap.authenticate(&servers, Some(&secrets), "alice", "alice-secret")
        .await
        .unwrap();
    assert!(test.binds() > binds);
    test.set_attribute(
        &format!("uid=alice,ou=people,{BASE}"),
        "memberOf",
        &["cn=developers,ou=groups,dc=example,dc=com"],
    );
    let alice = ldap
        .user(&servers, Some(&secrets), "alice")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(alice.groups, ["developers"]);
    let groups = ldap
        .search_groups(&servers, Some(&secrets), Some("adm"), 10)
        .await;
    assert_eq!(groups, ["admins"]);
}
