#[path = "support/soft_passkey.rs"]
mod soft_passkey;

use astrum_security::two_factor::RelyingPartyPolicy;
use astrum_security::two_factor::passkeys::{
    Passkey, PasskeyError, RelyingParty, finish_authentication, finish_registration,
    start_authentication, start_registration,
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

#[test]
fn a_cross_origin_ceremony_is_refused_at_registration_and_sign_in() {
    let rp = relying_party();
    let mut key = soft(7);
    key.cross_origin = true;
    assert!(matches!(
        register(&rp, &key, 0),
        Err(PasskeyError::Refused(WebauthnError::CredentialCrossOrigin))
    ));
    let passkey = register(&rp, &soft(7), 0).unwrap();
    assert!(matches!(
        authenticate(&rp, &key, &[passkey], 1),
        Err(PasskeyError::Refused(WebauthnError::CredentialCrossOrigin))
    ));
}

#[test]
fn a_response_to_another_challenge_is_refused() {
    let rp = relying_party();
    let key = soft(7);
    let first = start_registration(&rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    let second = start_registration(&rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    assert!(matches!(
        finish_registration(&rp, &first.state, &key.register(&second.public_key, 0)),
        Err(PasskeyError::Refused(WebauthnError::MismatchedChallenge))
    ));
    let passkey =
        finish_registration(&rp, &first.state, &key.register(&first.public_key, 0)).unwrap();
    let one = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    let two = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    assert!(matches!(
        finish_authentication(
            &rp,
            &one.state,
            &key.assert(&two.public_key, 1),
            std::slice::from_ref(&passkey),
        ),
        Err(PasskeyError::Refused(WebauthnError::MismatchedChallenge))
    ));
}

#[test]
fn a_state_of_the_other_ceremony_kind_is_refused() {
    let rp = relying_party();
    let key = soft(7);
    let registering = start_registration(&rp, Uuid::from_u128(1), "alice", "Alice", &[]).unwrap();
    let passkey = register(&rp, &key, 0).unwrap();
    let authenticating = start_authentication(&rp, std::slice::from_ref(&passkey)).unwrap();
    assert!(matches!(
        finish_authentication(
            &rp,
            &registering.state,
            &key.assert(&authenticating.public_key, 1),
            std::slice::from_ref(&passkey),
        ),
        Err(PasskeyError::State)
    ));
    assert!(matches!(
        finish_registration(
            &rp,
            &authenticating.state,
            &key.register(&registering.public_key, 0),
        ),
        Err(PasskeyError::State)
    ));
}

#[test]
fn a_credential_already_held_cannot_be_registered_again() {
    let rp = relying_party();
    let key = soft(7);
    let passkey = register(&rp, &key, 0).unwrap();
    let started = start_registration(
        &rp,
        Uuid::from_u128(1),
        "alice",
        "Alice",
        std::slice::from_ref(&passkey),
    )
    .unwrap();
    assert!(matches!(
        finish_registration(&rp, &started.state, &key.register(&started.public_key, 0)),
        Err(PasskeyError::Refused(_))
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
