mod support;

use astrum_security::ldap::TlsClient;
use astrum_testkit::ldap::{BASE, LdapTestServer, SYSTEM_DN, SYSTEM_PASSWORD};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};

const LDAP: &str = "/service/rest/v1/security/ldap";
const REALMS: &str = "/service/rest/v1/security/realms/active";
const USERS: &str = "/service/rest/v1/security/users";
const ROLES: &str = "/service/rest/v1/security/roles";
const SESSION: &str = "/service/rest/internal/session";
const STATE: &str = "/service/rest/internal/ui/state";
const TRUSTSTORE: &str = "/service/rest/v1/security/ssl/truststore";

fn server(port: u16) -> Value {
    json!({
        "name": "corp",
        "protocol": "ldap",
        "host": "127.0.0.1",
        "port": port,
        "searchBase": BASE,
        "authScheme": "SIMPLE",
        "authUsername": SYSTEM_DN,
        "authPassword": SYSTEM_PASSWORD,
        "connectionTimeoutSeconds": 5,
        "userBaseDn": "ou=people",
        "userSubtree": true,
        "userObjectClass": "inetOrgPerson",
        "userIdAttribute": "uid",
        "userRealNameAttribute": "cn",
        "userEmailAddressAttribute": "mail",
        "ldapGroupsAsRoles": true,
        "groupType": "dynamic",
        "userMemberOfAttribute": "memberOf"
    })
}

async fn setup() -> (Fixture, LdapTestServer) {
    let fixture = Fixture::new().await;
    let directory = LdapTestServer::start().await;
    fixture
        .state
        .security()
        .ldap()
        .set_tls(TlsClient::with_roots(vec![directory.ca()]));
    (fixture, directory)
}

async fn create(fixture: &Fixture, body: Value) -> Reply {
    fixture
        .call(Method::POST, LDAP, Auth::Admin, Some(body))
        .await
}

#[tokio::test]
async fn servers_are_created_read_changed_and_deleted() {
    let (fixture, directory) = setup().await;
    let realms = fixture
        .call(Method::GET, REALMS, Auth::Admin, None)
        .await
        .json();
    assert!(!realms.as_array().unwrap().contains(&json!("LdapRealm")));

    let reply = create(&fixture, server(directory.port)).await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    let realms = fixture
        .call(Method::GET, REALMS, Auth::Admin, None)
        .await
        .json();
    assert!(
        realms.as_array().unwrap().contains(&json!("LdapRealm")),
        "the first server turns the realm on: {realms}"
    );

    let list = fixture
        .call(Method::GET, LDAP, Auth::Admin, None)
        .await
        .json();
    assert_eq!(list.as_array().unwrap().len(), 1);
    let stored = &list[0];
    assert_eq!(stored["order"], 1);
    assert_eq!(stored["name"], "corp");
    assert_eq!(stored["protocol"], "LDAP");
    assert_eq!(stored["authScheme"], "SIMPLE");
    assert_eq!(stored["groupType"], "DYNAMIC");
    assert_eq!(stored["port"], directory.port);
    assert_eq!(stored["connectionRetryDelaySeconds"], 300);
    assert!(stored.get("authPassword").is_none());
    assert!(stored.get("authRealm").is_none(), "nulls are left out");
    assert!(
        !fixture.state.security().model().ldap_servers[0]
            .auth_password
            .as_deref()
            .unwrap()
            .contains(SYSTEM_PASSWORD)
    );

    let duplicate = create(&fixture, server(directory.port)).await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);
    assert_eq!(duplicate.json()[0]["id"], "name");

    let read = fixture
        .call(Method::GET, &format!("{LDAP}/CORP"), Auth::Admin, None)
        .await;
    assert_eq!(read.status, StatusCode::OK);
    let id = read.json()["id"].as_str().unwrap().to_owned();

    let mut renamed = server(directory.port);
    renamed["id"] = json!(id);
    renamed["name"] = json!("directory");
    renamed["authPassword"] = Value::Null;
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{LDAP}/corp"),
            Auth::Admin,
            Some(renamed.clone()),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let model = fixture.state.security().model();
    assert_eq!(model.ldap_servers[0].name, "directory");
    assert!(
        model.ldap_servers[0].auth_password.is_some(),
        "the password is kept"
    );

    let mut moved = renamed.clone();
    moved["host"] = json!("localhost");
    let reply = fixture
        .call(
            Method::PUT,
            &format!("{LDAP}/directory"),
            Auth::Admin,
            Some(moved),
        )
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()[0]["id"], "authPassword");

    let missing = fixture
        .call(Method::GET, &format!("{LDAP}/nope"), Auth::Admin, None)
        .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing.json()["message"],
        "LDAP server 'nope' does not exist"
    );

    let mut second = server(directory.port);
    second["name"] = json!("backup");
    assert_eq!(create(&fixture, second).await.status, StatusCode::CREATED);
    let reply = fixture
        .call(
            Method::POST,
            &format!("{LDAP}/change-order"),
            Auth::Admin,
            Some(json!(["backup"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let names: Vec<Value> = fixture
        .call(Method::GET, LDAP, Auth::Admin, None)
        .await
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|server| json!([server["order"], server["name"]]))
        .collect();
    assert_eq!(names, [json!([1, "backup"]), json!([2, "directory"])]);
    let unknown = fixture
        .call(
            Method::POST,
            &format!("{LDAP}/change-order"),
            Auth::Admin,
            Some(json!(["ghost"])),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown.json()[0]["id"], "order");

    let deleted = fixture
        .call(Method::DELETE, &format!("{LDAP}/backup"), Auth::Admin, None)
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(fixture.state.security().model().ldap_servers.len(), 1);

    let templates = fixture
        .call(Method::GET, &format!("{LDAP}/templates"), Auth::Admin, None)
        .await
        .json();
    assert_eq!(templates[0]["name"], "Active Directory");
    assert_eq!(templates[1]["groupType"], "static");

    let cache = fixture
        .call(Method::DELETE, &format!("{LDAP}/cache"), Auth::Admin, None)
        .await;
    assert_eq!(cache.status, StatusCode::NO_CONTENT);

    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=security.ldap",
            Auth::Admin,
            None,
        )
        .await
        .text();
    assert!(
        audit.contains("\"created\"") && audit.contains("\"deleted\""),
        "{audit}"
    );
    assert!(!audit.contains(SYSTEM_PASSWORD));
}

#[tokio::test]
async fn drafts_are_validated_with_field_names() {
    let (fixture, directory) = setup().await;
    let mut broken = server(directory.port);
    broken["host"] = json!("not a host");
    broken["userObjectClass"] = Value::Null;
    broken["authPassword"] = Value::Null;
    let reply = create(&fixture, broken).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let fields: Vec<String> = reply
        .json()
        .as_array()
        .unwrap()
        .iter()
        .map(|error| error["id"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        fields.contains(&"host".to_owned()) && fields.contains(&"userObjectClass".to_owned()),
        "{fields:?}"
    );
    let mut no_password = server(directory.port);
    no_password["authPassword"] = Value::Null;
    let reply = create(&fixture, no_password).await;
    assert_eq!(reply.json()[0]["id"], "authPassword");
    assert_eq!(reply.json()[0]["message"], "Password is required");
}

#[tokio::test]
async fn drafts_are_verified_against_the_directory() {
    let (fixture, directory) = setup().await;
    let verify = |path: &'static str, body: Value| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::POST,
                    &format!("{LDAP}/{path}"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    };
    let reply = verify("verify-connection", server(directory.port)).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    let mut wrong = server(directory.port);
    wrong["authPassword"] = json!("wrong");
    let reply = verify("verify-connection", wrong).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reply.json()[0]["message"],
        "The LDAP server rejected the username or password of the system account"
    );

    let users = verify("verify-user-mapping", server(directory.port)).await;
    assert_eq!(users.status, StatusCode::OK, "{}", users.text());
    let alice = users
        .json()
        .as_array()
        .unwrap()
        .iter()
        .find(|user| user["username"] == "alice")
        .cloned()
        .unwrap();
    assert_eq!(
        alice,
        json!({"username": "alice", "realName": "Alice Smith", "email": "alice@example.com",
               "membership": ["admins", "developers"]})
    );

    let login = |user: &str, password: &str| {
        json!({
            "base64Username": STANDARD.encode(user),
            "base64Password": STANDARD.encode(password),
            "ldapServer": server(directory.port),
        })
    };
    assert_eq!(
        verify("verify-login", login("bob", "bob-secret"))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    let refused = verify("verify-login", login("bob", "nope")).await;
    assert_eq!(
        refused.json()[0]["message"],
        "The password for user 'bob' is incorrect"
    );
    let unknown = verify("verify-login", login("zed", "x")).await;
    assert_eq!(unknown.json()[0]["message"], "User 'zed' was not found");
    let garbled = verify(
        "verify-login",
        json!({"base64Username": "%%%", "ldapServer": server(directory.port)}),
    )
    .await;
    assert_eq!(garbled.json()[0]["id"], "base64Username");

    assert_eq!(
        create(&fixture, server(directory.port)).await.status,
        StatusCode::CREATED
    );
    let mut stored_password = server(directory.port);
    stored_password["authPassword"] = Value::Null;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{LDAP}/verify-connection?existingServerName=corp"),
            Auth::Admin,
            Some(stored_password.clone()),
        )
        .await;
    assert_eq!(
        reply.status,
        StatusCode::NO_CONTENT,
        "the stored password is used"
    );
    let reply = verify("verify-connection", stored_password).await;
    assert_eq!(reply.json()[0]["id"], "authPassword");

    fixture.add_user("viewer", "viewer-password", &[]).await;
    let forbidden = fixture
        .call(
            Method::POST,
            &format!("{LDAP}/verify-connection"),
            Auth::Basic("viewer", "viewer-password"),
            Some(server(directory.port)),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
}

async fn create_role(fixture: &Fixture, body: Value) {
    let reply = fixture
        .call(Method::POST, ROLES, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

#[tokio::test]
async fn directory_users_sign_in_with_their_groups_as_roles() {
    let (fixture, directory) = setup().await;
    assert_eq!(
        create(&fixture, server(directory.port)).await.status,
        StatusCode::CREATED
    );
    create_role(
        &fixture,
        json!({"id": "developers", "name": "Developers", "source": "LDAP",
               "privileges": ["as-users-read", "as-roles-read"]}),
    )
    .await;
    let roles = fixture
        .call(Method::GET, ROLES, Auth::Admin, None)
        .await
        .json();
    let mapped = roles
        .as_array()
        .unwrap()
        .iter()
        .find(|role| role["id"] == "developers")
        .unwrap();
    assert_eq!(mapped["source"], "LDAP");

    let groups = fixture
        .call(
            Method::GET,
            &format!("{ROLES}?source=LDAP"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    let ids: Vec<&str> = groups
        .as_array()
        .unwrap()
        .iter()
        .map(|role| role["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["admins", "developers"]);
    assert_eq!(groups[0]["readOnly"], true);
    let found = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/roles?source=LDAP&search=dev",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(found.as_array().unwrap().len(), 1);
    assert_eq!(found[0]["id"], "developers");
    assert_eq!(
        fixture
            .call(
                Method::GET,
                &format!("{ROLES}/admins?source=LDAP"),
                Auth::Admin,
                None
            )
            .await
            .status,
        StatusCode::OK
    );
    let sources = fixture
        .call(
            Method::GET,
            "/service/rest/internal/ui/roles/sources",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(sources[0]["id"], "LDAP");
    assert_eq!(sources[1]["id"], "OAuth2");

    let bob = fixture
        .call(Method::GET, USERS, Auth::Basic("bob", "bob-secret"), None)
        .await;
    assert_eq!(
        bob.status,
        StatusCode::OK,
        "bob reads users through his group"
    );
    let wrong = fixture
        .call(Method::GET, USERS, Auth::Basic("bob", "nope"), None)
        .await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
    let carol = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("carol", "carol-secret"),
            None,
        )
        .await;
    assert_eq!(carol.status, StatusCode::FORBIDDEN, "carol has no group");

    let users = fixture
        .call(
            Method::GET,
            &format!("{USERS}?source=LDAP&userId=c"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(users.as_array().unwrap().len(), 1);
    assert_eq!(users[0]["userId"], "carol");
    assert_eq!(users[0]["firstName"], "Carol");
    assert_eq!(users[0]["lastName"], "White");
    assert_eq!(users[0]["readOnly"], true);
    assert_eq!(users[0]["source"], "LDAP");

    let reply = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/carol"),
            Auth::Admin,
            Some(
                json!({"userId": "carol", "source": "LDAP", "status": "active",
                        "roles": ["as-admin"]}),
            ),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    fixture.state.security().ldap().clear();
    let carol = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("carol", "carol-secret"),
            None,
        )
        .await;
    assert_eq!(carol.status, StatusCode::OK, "the mapped role applies");
    let alice = fixture
        .call(
            Method::GET,
            &format!("{USERS}?source=LDAP&userId=alice"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(alice[0]["externalRoles"], json!(["admins", "developers"]));
    assert_eq!(alice[0]["roles"], json!([]));

    let ghost = fixture
        .call(
            Method::PUT,
            &format!("{USERS}/ghost"),
            Auth::Admin,
            Some(json!({"userId": "ghost", "source": "LDAP", "status": "active", "roles": []})),
        )
        .await;
    assert_eq!(ghost.status, StatusCode::NOT_FOUND);
    let unmapped = fixture
        .call(
            Method::DELETE,
            &format!("{USERS}/carol?realm=LdapRealm"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(unmapped.status, StatusCode::NO_CONTENT);
    assert!(
        fixture
            .state
            .security()
            .model()
            .roles_of("carol", "LDAP")
            .is_empty()
    );

    drop(directory);
}

fn session_cookies(reply: &Reply) -> String {
    reply
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| {
            value
                .to_str()
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[tokio::test]
async fn directory_users_get_a_session_like_local_users() {
    let (fixture, directory) = setup().await;
    assert_eq!(
        create(&fixture, server(directory.port)).await.status,
        StatusCode::CREATED
    );
    create_role(
        &fixture,
        json!({"id": "admins", "name": "Admins", "source": "LDAP", "privileges": ["as-all"]}),
    )
    .await;
    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "alice", "password": "alice-secret"})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(reply.json()["user"]["source"], "LDAP");
    assert_eq!(reply.json()["user"]["realm"], "LdapRealm");
    assert_eq!(reply.json()["user"]["firstName"], "Alice");
    let cookies = session_cookies(&reply);
    let state = fixture
        .send(
            Method::GET,
            STATE,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await
        .json();
    assert_eq!(state["user"]["userId"], "alice");
    assert_eq!(state["user"]["source"], "LDAP");
    assert_eq!(state["user"]["name"], "Alice Smith");
    assert_eq!(state["user"]["email"], "alice@example.com");
    assert!(
        state["permissions"]
            .as_array()
            .unwrap()
            .contains(&json!("astrum:*"))
    );

    directory.remove(&format!("uid=alice,ou=people,{BASE}"));
    fixture.state.security().ldap().clear();
    let gone = fixture
        .send(
            Method::GET,
            STATE,
            Auth::None,
            None,
            &[("cookie", &cookies)],
        )
        .await
        .json();
    assert_eq!(
        gone["user"],
        Value::Null,
        "a user removed from LDAP loses the session"
    );
}

#[tokio::test]
async fn the_local_realm_comes_first_and_ldap_only_while_active() {
    let (fixture, directory) = setup().await;
    assert_eq!(
        create(&fixture, server(directory.port)).await.status,
        StatusCode::CREATED
    );
    fixture
        .add_user("alice", "local-password", &["as-admin"])
        .await;
    let local = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("alice", "local-password"),
            None,
        )
        .await;
    assert_eq!(local.status, StatusCode::OK);
    let directory_password = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("alice", "alice-secret"),
            None,
        )
        .await;
    assert_eq!(
        directory_password.status,
        StatusCode::FORBIDDEN,
        "the LDAP alice signs in but has no roles"
    );
    let reply = fixture
        .call(
            Method::PUT,
            REALMS,
            Auth::Admin,
            Some(json!(["AstrumAuthenticatingRealm", "AccessTokenRealm"])),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let off = fixture
        .call(Method::GET, USERS, Auth::Basic("bob", "bob-secret"), None)
        .await;
    assert_eq!(off.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_default_role_reaches_every_signed_in_user() {
    let (fixture, _directory) = setup().await;
    fixture.add_user("plain", "plain-password", &[]).await;
    let before = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("plain", "plain-password"),
            None,
        )
        .await;
    assert_eq!(before.status, StatusCode::FORBIDDEN);
    create_role(
        &fixture,
        json!({"id": "readers", "name": "Readers", "privileges": ["as-users-read"]}),
    )
    .await;
    let path = "/service/rest/internal/ui/default-role";
    let unknown = fixture
        .call(
            Method::PUT,
            path,
            Auth::Admin,
            Some(json!({"role": "nope"})),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown.json()[0]["id"], "role");
    let set = fixture
        .call(
            Method::PUT,
            path,
            Auth::Admin,
            Some(json!({"role": "readers"})),
        )
        .await;
    assert_eq!(set.status, StatusCode::NO_CONTENT, "{}", set.text());
    assert_eq!(
        fixture
            .call(Method::GET, path, Auth::Admin, None)
            .await
            .json(),
        json!({"role": "readers", "enabled": true, "missing": false})
    );
    let realms = fixture
        .call(Method::GET, REALMS, Auth::Admin, None)
        .await
        .json();
    assert!(realms.as_array().unwrap().contains(&json!("DefaultRole")));
    let after = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("plain", "plain-password"),
            None,
        )
        .await;
    assert_eq!(after.status, StatusCode::OK);

    let paused = fixture
        .call(
            Method::PUT,
            path,
            Auth::Admin,
            Some(json!({"role": "readers", "enabled": false})),
        )
        .await;
    assert_eq!(paused.status, StatusCode::NO_CONTENT, "{}", paused.text());
    assert_eq!(
        fixture
            .call(Method::GET, path, Auth::Admin, None)
            .await
            .json(),
        json!({"role": "readers", "enabled": false, "missing": false})
    );
    let without = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("plain", "plain-password"),
            None,
        )
        .await;
    assert_eq!(without.status, StatusCode::FORBIDDEN);
    fixture
        .call(
            Method::PUT,
            path,
            Auth::Admin,
            Some(json!({"role": "readers", "enabled": true})),
        )
        .await;

    fixture
        .call(
            Method::PUT,
            "/service/rest/v1/security/anonymous",
            Auth::Admin,
            Some(json!({"enabled": true, "userId": "anonymous", "realmName": "AstrumAuthorizingRealm"})),
        )
        .await;
    let anonymous = fixture.call(Method::GET, USERS, Auth::None, None).await;
    assert_eq!(
        anonymous.status,
        StatusCode::UNAUTHORIZED,
        "anonymous callers get no default role"
    );

    let reply = fixture
        .call(
            Method::POST,
            SESSION,
            Auth::None,
            Some(json!({"username": "plain", "password": "plain-password"})),
        )
        .await;
    let state = fixture
        .send(
            Method::GET,
            STATE,
            Auth::None,
            None,
            &[("cookie", &session_cookies(&reply))],
        )
        .await
        .json();
    assert_eq!(
        state["defaultRole"],
        json!({"id": "readers", "name": "Readers"})
    );

    let removed = fixture.call(Method::DELETE, path, Auth::Admin, None).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let gone = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("plain", "plain-password"),
            None,
        )
        .await;
    assert_eq!(gone.status, StatusCode::FORBIDDEN);
    assert_eq!(
        fixture
            .call(Method::GET, path, Auth::Admin, None)
            .await
            .json(),
        json!({"role": null, "enabled": false, "missing": false})
    );
}

#[tokio::test]
async fn a_directory_is_trusted_through_the_truststore_when_asked() {
    let fixture = Fixture::new().await;
    let directory = LdapTestServer::start().await;
    let secure = |use_trust_store: bool| {
        let mut body = server(directory.ldaps_port);
        body["protocol"] = json!("ldaps");
        body["host"] = json!("localhost");
        body["useTrustStore"] = json!(use_trust_store);
        body
    };
    let verify = |body: Value| {
        let fixture = &fixture;
        async move {
            fixture
                .call(
                    Method::POST,
                    &format!("{LDAP}/verify-connection"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    };
    let untrusted = verify(secure(true)).await;
    assert_eq!(untrusted.status, StatusCode::BAD_REQUEST);
    assert!(
        untrusted.json()[0]["message"]
            .as_str()
            .unwrap()
            .contains("TLS handshake"),
        "{}",
        untrusted.text()
    );

    let added = fixture
        .send_body(
            Method::POST,
            TRUSTSTORE,
            Auth::Admin,
            Body::from(directory.certificates.ca_pem.clone()),
            &[("content-type", "text/plain")],
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.text());
    let id = added.json()["id"].as_str().unwrap().to_owned();
    assert_eq!(
        verify(secure(false)).await.status,
        StatusCode::BAD_REQUEST,
        "without the option only the system certificates count"
    );
    let trusted = verify(secure(true)).await;
    assert_eq!(trusted.status, StatusCode::NO_CONTENT, "{}", trusted.text());
    let mut start_tls = server(directory.port);
    start_tls["host"] = json!("localhost");
    start_tls["useStartTls"] = json!(true);
    start_tls["useTrustStore"] = json!(true);
    let reply = verify(start_tls).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());

    assert_eq!(
        create(&fixture, secure(true)).await.status,
        StatusCode::CREATED
    );
    create_role(
        &fixture,
        json!({"id": "admins", "name": "Admins", "source": "LDAP", "privileges": ["as-all"]}),
    )
    .await;
    let alice = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("alice", "alice-secret"),
            None,
        )
        .await;
    assert_eq!(alice.status, StatusCode::OK, "{}", alice.text());

    let removed = fixture
        .call(
            Method::DELETE,
            &format!("{TRUSTSTORE}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    let refused = fixture
        .call(
            Method::GET,
            USERS,
            Auth::Basic("alice", "alice-secret"),
            None,
        )
        .await;
    assert_eq!(
        refused.status,
        StatusCode::UNAUTHORIZED,
        "taking the certificate out ends the cached sign-in at once"
    );
}
