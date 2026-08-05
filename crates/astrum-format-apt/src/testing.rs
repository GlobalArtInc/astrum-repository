pub fn signing_key() -> String {
    use pgp::composed::{ArmorOptions, KeyType, SecretKeyParamsBuilder};
    let mut params = SecretKeyParamsBuilder::default();
    params
        .key_type(KeyType::Ed25519Legacy)
        .can_certify(true)
        .can_sign(true)
        .primary_user_id("Astrum Test <test@example.org>".into());
    params
        .build()
        .expect("the test key parameters are valid")
        .generate(rand08::thread_rng())
        .expect("a test key can be generated")
        .to_armored_string(ArmorOptions::default())
        .expect("a test key can be armored")
}
