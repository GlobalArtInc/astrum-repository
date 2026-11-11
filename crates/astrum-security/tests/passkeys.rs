#[path = "support/soft_passkey.rs"]
mod soft_passkey;

use astrum_security::two_factor::passkeys::{
    Passkey, PasskeyError, RelyingParty, RelyingPartyError, RelyingPartyPolicy,
    finish_authentication, finish_registration, start_authentication, start_registration,
};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use soft_passkey::SoftPasskey;
use uuid::Uuid;
use webauthn_rs::prelude::WebauthnError;

const RP_ID: &str = "astrum.example.com";
const ORIGIN: &str = "https://astrum.example.com";

fn relying_party() -> RelyingParty {
    RelyingParty::resolve(&RelyingPartyPolicy::default(), Some(ORIGIN)).unwrap()
}

fn soft(seed: u8) -> SoftPasskey {
    SoftPasskey::new(seed, RP_ID, ORIGIN)
}

fn register(rp: &RelyingParty, key: &SoftPasskey, counter: u32) -> Result<Passkey, PasskeyError> {
    let started = start_registration(rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    finish_registration(
        rp,
        &started.state,
        &key.register(&started.public_key, counter),
    )
}

fn authenticate(
    rp: &RelyingParty,
    key: &SoftPasskey,
    passkeys: &[Passkey],
    counter: u32,
) -> Result<astrum_security::two_factor::passkeys::Authenticated, PasskeyError> {
    let started = start_authentication(rp, passkeys).unwrap();
    finish_authentication(
        rp,
        &started.state,
        &key.assert(&started.public_key, counter),
        passkeys,
    )
}

fn stored_counter(passkey: &Passkey) -> u64 {
    serde_json::to_value(passkey).unwrap()["cred"]["counter"]
        .as_u64()
        .unwrap()
}

#[test]
fn a_passkey_registers_and_signs_in() {
    let rp = relying_party();
    let key = soft(7);
    let started = start_registration(&rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    assert!(started.state.is_object());
    assert_eq!(started.public_key["rp"]["id"], RP_ID);
    assert_eq!(
        started.public_key["authenticatorSelection"]["userVerification"],
        "required"
    );
    assert_eq!(started.public_key["attestation"], "none");
    let passkey =
        finish_registration(&rp, &started.state, &key.register(&started.public_key, 0)).unwrap();
    assert_eq!(passkey.cred_id().as_slice(), key.credential_id());

    let again = start_registration(
        &rp,
        Uuid::from_u128(1),
        "alice",
        "Alice",
        std::slice::from_ref(&passkey),
    )
    .unwrap();
    assert_eq!(
        again.public_key["excludeCredentials"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let challenge = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    assert_eq!(challenge.public_key["userVerification"], "required");
    assert_eq!(challenge.public_key["rpId"], RP_ID);
    let signed_in = finish_authentication(
        &rp,
        &challenge.state,
        &key.assert(&challenge.public_key, 1),
        std::slice::from_ref(&passkey),
    )
    .unwrap();
    assert!(signed_in.changed);
    assert_eq!(stored_counter(&signed_in.credential), 1);
}

#[test]
fn a_foreign_origin_is_refused_at_registration_and_sign_in() {
    let rp = relying_party();
    for origin in [
        "https://evil.example.net",
        "https://sub.astrum.example.com",
        "http://astrum.example.com",
        "https://astrum.example.com:8443",
    ] {
        let mut key = soft(7);
        key.origin = origin.to_owned();
        assert!(matches!(
            register(&rp, &key, 0),
            Err(PasskeyError::Refused(WebauthnError::InvalidRPOrigin))
        ));
    }
    let passkey = register(&rp, &soft(7), 0).unwrap();
    let mut key = soft(7);
    key.origin = "https://evil.example.net".to_owned();
    assert!(matches!(
        authenticate(&rp, &key, &[passkey], 1),
        Err(PasskeyError::Refused(WebauthnError::InvalidRPOrigin))
    ));
}

#[test]
fn a_wrong_rp_id_hash_is_refused_at_registration_and_sign_in() {
    let rp = relying_party();
    let mut key = soft(7);
    key.rp_id_hash = Some([9; 32]);
    assert!(matches!(
        register(&rp, &key, 0),
        Err(PasskeyError::Refused(WebauthnError::InvalidRPIDHash))
    ));
    let passkey = register(&rp, &soft(7), 0).unwrap();
    assert!(matches!(
        authenticate(&rp, &key, &[passkey], 1),
        Err(PasskeyError::Refused(WebauthnError::InvalidRPIDHash))
    ));
}

#[test]
fn user_verification_is_required_at_registration_and_sign_in() {
    let rp = relying_party();
    let mut key = soft(7);
    key.user_verified = false;
    assert!(matches!(
        register(&rp, &key, 0),
        Err(PasskeyError::Refused(WebauthnError::UserNotVerified))
    ));
    let passkey = register(&rp, &soft(7), 0).unwrap();
    assert!(matches!(
        authenticate(&rp, &key, std::slice::from_ref(&passkey), 1),
        Err(PasskeyError::Refused(WebauthnError::UserNotVerified))
    ));
    key.user_verified = true;
    key.user_present = false;
    assert!(matches!(
        authenticate(&rp, &key, &[passkey], 1),
        Err(PasskeyError::Refused(WebauthnError::UserNotPresent))
    ));
}

#[test]
fn a_counter_that_does_not_increase_is_a_regression() {
    let rp = relying_party();
    let key = soft(7);
    let passkey = register(&rp, &key, 0).unwrap();
    let five = authenticate(&rp, &key, std::slice::from_ref(&passkey), 5).unwrap();
    assert_eq!(stored_counter(&five.credential), 5);
    let stored = [five.credential.clone()];
    for counter in [5, 3, 0] {
        assert!(matches!(
            authenticate(&rp, &key, &stored, counter),
            Err(PasskeyError::CounterRegression)
        ));
    }
    let nine = authenticate(&rp, &key, &stored, 9).unwrap();
    assert!(nine.changed);

    let started = start_authentication(&rp, &[passkey]).unwrap();
    assert!(matches!(
        finish_authentication(
            &rp,
            &started.state,
            &key.assert(&started.public_key, 3),
            &[nine.credential],
        ),
        Err(PasskeyError::CounterRegression)
    ));
}

#[test]
fn a_counter_of_zero_is_accepted_repeatedly() {
    let rp = relying_party();
    let key = soft(7);
    let passkey = register(&rp, &key, 0).unwrap();
    let first = authenticate(&rp, &key, &[passkey], 0).unwrap();
    assert!(!first.changed);
    let second = authenticate(&rp, &key, &[first.credential], 0).unwrap();
    assert!(!second.changed);
    assert_eq!(stored_counter(&second.credential), 0);
}

#[test]
fn a_credential_that_is_not_registered_is_unknown() {
    let rp = relying_party();
    let key = soft(7);
    let passkey = register(&rp, &key, 0).unwrap();
    assert!(matches!(
        authenticate(&rp, &soft(8), std::slice::from_ref(&passkey), 1),
        Err(PasskeyError::UnknownCredential)
    ));
    let started = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    let other = register(&rp, &soft(8), 0).unwrap();
    assert!(matches!(
        finish_authentication(
            &rp,
            &started.state,
            &key.assert(&started.public_key, 1),
            &[other],
        ),
        Err(PasskeyError::UnknownCredential)
    ));
    assert!(matches!(
        start_authentication(&rp, &[]),
        Err(PasskeyError::NoCredentials)
    ));
}

type Accepted<'a> = (
    Option<&'a str>,
    &'a [&'a str],
    Option<&'a str>,
    &'a str,
    &'a [&'a str],
);
type Refused<'a> = (
    Option<&'a str>,
    &'a [&'a str],
    Option<&'a str>,
    RelyingPartyError,
);

fn resolve(
    id: Option<&str>,
    origins: &[&str],
    base: Option<&str>,
) -> Result<RelyingParty, RelyingPartyError> {
    RelyingParty::resolve(
        &RelyingPartyPolicy {
            id: id.map(str::to_owned),
            origins: origins.iter().map(|origin| (*origin).to_owned()).collect(),
        },
        base,
    )
}

#[test]
fn the_relying_party_is_resolved_from_the_base_url_and_the_policy() {
    let accepted: Vec<Accepted> = vec![
        (
            None,
            &[],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[],
            Some("https://Astrum.Example.com:443/ui/?x=1#y"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[],
            Some("https://astrum.example.com:8443"),
            "astrum.example.com",
            &["https://astrum.example.com:8443/"],
        ),
        (
            Some("example.com"),
            &[],
            Some("https://astrum.example.com"),
            "example.com",
            &["https://astrum.example.com/"],
        ),
        (
            Some("Astrum.Example.com"),
            &[],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &["https://astrum.example.com/"],
        ),
        (
            None,
            &[
                "https://astrum.example.com",
                "https://alt.astrum.example.com",
            ],
            Some("https://astrum.example.com"),
            "astrum.example.com",
            &[
                "https://astrum.example.com/",
                "https://alt.astrum.example.com/",
            ],
        ),
        (
            Some("example.com"),
            &["https://example.com", "https://other.example.com:9443"],
            Some("https://astrum.example.com"),
            "example.com",
            &[
                "https://astrum.example.com/",
                "https://example.com/",
                "https://other.example.com:9443/",
            ],
        ),
        (
            None,
            &[],
            Some("http://localhost:8081"),
            "localhost",
            &["http://localhost:8081/"],
        ),
        (
            None,
            &["http://localhost:3000"],
            Some("http://localhost:8081"),
            "localhost",
            &["http://localhost:8081/", "http://localhost:3000/"],
        ),
        (
            Some("example.com"),
            &["https://example.com"],
            None,
            "example.com",
            &["https://example.com/"],
        ),
        (
            Some("localhost"),
            &["http://localhost:5173"],
            None,
            "localhost",
            &["http://localhost:5173/"],
        ),
    ];
    for (id, origins, base, expected_id, expected_origins) in accepted {
        let rp =
            resolve(id, origins, base).unwrap_or_else(|error| panic!("{base:?} {id:?}: {error}"));
        assert_eq!(rp.id(), expected_id, "{base:?} {id:?}");
        let got: Vec<&str> = rp.origins().iter().map(|origin| origin.as_str()).collect();
        assert_eq!(got, expected_origins, "{base:?} {id:?}");
    }

    let base = Some("https://astrum.example.com");
    let origin = |value: &str| RelyingPartyError::Origin {
        origin: value.to_owned(),
        reason: "",
    };
    let refused: Vec<Refused> = vec![
        (None, &[], None, RelyingPartyError::Unavailable),
        (
            None,
            &["https://astrum.example.com"],
            None,
            RelyingPartyError::Unavailable,
        ),
        (
            Some("example.com"),
            &[],
            None,
            RelyingPartyError::Unavailable,
        ),
        (Some("other.com"), &[], base, RelyingPartyError::IdNotSuffix),
        (
            Some("xample.com"),
            &[],
            base,
            RelyingPartyError::IdNotSuffix,
        ),
        (
            Some("sub.astrum.example.com"),
            &[],
            base,
            RelyingPartyError::IdNotSuffix,
        ),
        (Some("com"), &[], base, RelyingPartyError::Id("")),
        (
            Some("com"),
            &["https://example.com"],
            None,
            RelyingPartyError::Id(""),
        ),
        (Some("exa mple.com"), &[], base, RelyingPartyError::Id("")),
        (Some("127.0.0.1"), &[], None, RelyingPartyError::Id("")),
        (None, &["https://other.com"], base, origin("")),
        (None, &["https://badastrum.example.com"], base, origin("")),
        (None, &["http://astrum.example.com"], base, origin("")),
        (None, &["https://astrum.example.com/path"], base, origin("")),
        (None, &["https://user@astrum.example.com"], base, origin("")),
        (None, &["https://astrum.example.com?x=1"], base, origin("")),
        (None, &["not a url"], base, origin("")),
        (None, &[], Some("http://astrum.example.com"), origin("")),
        (
            None,
            &[],
            Some("https://127.0.0.1:8081"),
            RelyingPartyError::BaseUrl(""),
        ),
        (
            None,
            &[],
            Some("ftp://astrum.example.com"),
            RelyingPartyError::BaseUrl(""),
        ),
    ];
    for (id, origins, base, expected) in refused {
        let error = resolve(id, origins, base).expect_err(&format!("{base:?} {id:?} {origins:?}"));
        assert_eq!(
            std::mem::discriminant(&error),
            std::mem::discriminant(&expected),
            "{base:?} {id:?} {origins:?}: {error}"
        );
    }
    assert!(matches!(
        resolve(None, &[], Some("not a url")),
        Err(RelyingPartyError::BaseUrl(_))
    ));
    assert!(matches!(
        resolve(None, &[], Some("https://astrum.example.com.")),
        Err(RelyingPartyError::Id(_))
    ));
}

#[test]
fn stored_json_state_from_an_earlier_build_still_works() {
    let passkey: Value = serde_json::from_str(include_str!("data/passkeys/passkey.json")).unwrap();
    let registration: Value =
        serde_json::from_str(include_str!("data/passkeys/registration-state.json")).unwrap();
    let authentication: Value =
        serde_json::from_str(include_str!("data/passkeys/authentication-state.json")).unwrap();
    let stored: Passkey = serde_json::from_value(passkey.clone()).unwrap();
    assert_eq!(serde_json::to_value(&stored).unwrap(), passkey);
    assert_eq!(stored_counter(&stored), 5);

    let rp = relying_party();
    let key = soft(7);
    let registered = finish_registration(
        &rp,
        &registration,
        &key.register(&json!({"challenge": registration["rs"]["challenge"]}), 5),
    )
    .unwrap();
    assert_eq!(registered.cred_id(), stored.cred_id());

    let signed_in = finish_authentication(
        &rp,
        &authentication,
        &key.assert(&json!({"challenge": authentication["ast"]["challenge"]}), 6),
        std::slice::from_ref(&stored),
    )
    .unwrap();
    assert_eq!(stored_counter(&signed_in.credential), 6);
    let reread: Passkey =
        serde_json::from_value(serde_json::to_value(&signed_in.credential).unwrap()).unwrap();
    assert_eq!(stored_counter(&reread), 6);
}

fn decoded(value: &Value, path: [&str; 2]) -> Vec<u8> {
    URL_SAFE_NO_PAD
        .decode(value[path[0]][path[1]].as_str().unwrap())
        .unwrap()
}

fn with_field(value: &Value, field: &str, data: &[u8]) -> Value {
    let mut copy = value.clone();
    copy["response"][field] = Value::String(URL_SAFE_NO_PAD.encode(data));
    copy
}

#[test]
fn truncated_and_corrupted_client_data_is_an_error_and_never_a_panic() {
    let rp = relying_party();
    let key = soft(7);
    let started = start_registration(&rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    let response = key.register(&started.public_key, 0);
    for field in ["attestationObject", "clientDataJSON"] {
        let original = decoded(&response, ["response", field]);
        for length in 0..original.len() {
            let broken = with_field(&response, field, &original[..length]);
            assert!(
                finish_registration(&rp, &started.state, &broken).is_err(),
                "{field} cut to {length}"
            );
        }
        for index in 0..original.len() {
            let mut flipped = original.clone();
            flipped[index] ^= 0xff;
            let _ =
                finish_registration(&rp, &started.state, &with_field(&response, field, &flipped));
        }
    }

    let passkey = finish_registration(&rp, &started.state, &response).unwrap();
    let challenge = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    let assertion = key.assert(&challenge.public_key, 1);
    for field in ["authenticatorData", "clientDataJSON", "signature"] {
        let original = decoded(&assertion, ["response", field]);
        for length in 0..original.len() {
            let broken = with_field(&assertion, field, &original[..length]);
            assert!(
                finish_authentication(
                    &rp,
                    &challenge.state,
                    &broken,
                    std::slice::from_ref(&passkey)
                )
                .is_err(),
                "{field} cut to {length}"
            );
        }
        for index in 0..original.len() {
            let mut flipped = original.clone();
            flipped[index] ^= 0xff;
            assert!(
                finish_authentication(
                    &rp,
                    &challenge.state,
                    &with_field(&assertion, field, &flipped),
                    std::slice::from_ref(&passkey),
                )
                .is_err(),
                "{field} flipped at {index}"
            );
        }
    }

    for hostile in [
        Value::Null,
        json!([]),
        json!("x"),
        json!({"id": 1}),
        json!({"response": {}}),
    ] {
        assert!(matches!(
            finish_registration(&rp, &started.state, &hostile),
            Err(PasskeyError::Malformed)
        ));
        assert!(matches!(
            finish_authentication(
                &rp,
                &challenge.state,
                &hostile,
                std::slice::from_ref(&passkey)
            ),
            Err(PasskeyError::Malformed)
        ));
        assert!(matches!(
            finish_registration(&rp, &hostile, &response),
            Err(PasskeyError::State)
        ));
        assert!(matches!(
            finish_authentication(&rp, &hostile, &assertion, std::slice::from_ref(&passkey)),
            Err(PasskeyError::State)
        ));
    }
}
