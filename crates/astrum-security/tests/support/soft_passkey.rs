use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const FLAG_USER_PRESENT: u8 = 0x01;
const FLAG_USER_VERIFIED: u8 = 0x04;
const FLAG_BACKUP_ELIGIBLE: u8 = 0x08;
const FLAG_BACKUP_STATE: u8 = 0x10;
const FLAG_ATTESTED: u8 = 0x40;

pub struct SoftPasskey {
    key: SigningKey,
    credential_id: Vec<u8>,
    pub rp_id: String,
    pub origin: String,
    pub rp_id_hash: Option<[u8; 32]>,
    pub user_present: bool,
    pub user_verified: bool,
    pub backup_eligible: bool,
    pub backup_state: bool,
    pub user_handle: Option<Vec<u8>>,
}

impl SoftPasskey {
    pub fn new(seed: u8, rp_id: &str, origin: &str) -> Self {
        Self {
            key: SigningKey::from_slice(&[seed; 32]).expect("a repeated non-zero byte is a key"),
            credential_id: vec![seed; 32],
            rp_id: rp_id.to_owned(),
            origin: origin.to_owned(),
            rp_id_hash: None,
            user_present: true,
            user_verified: true,
            backup_eligible: false,
            backup_state: false,
            user_handle: None,
        }
    }

    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }

    pub fn register(&self, public_key: &Value, counter: u32) -> Value {
        let challenge = challenge_of(public_key);
        let mut auth_data = self.authenticator_data(counter, true);
        auth_data.extend([0u8; 16]);
        auth_data.extend((self.credential_id.len() as u16).to_be_bytes());
        auth_data.extend(&self.credential_id);
        auth_data.extend(self.cose_key());
        let mut attestation = vec![0xa3];
        text(&mut attestation, "fmt");
        text(&mut attestation, "none");
        text(&mut attestation, "attStmt");
        attestation.push(0xa0);
        text(&mut attestation, "authData");
        bytes(&mut attestation, &auth_data);
        json!({
            "id": b64(&self.credential_id),
            "rawId": b64(&self.credential_id),
            "type": "public-key",
            "response": {
                "attestationObject": b64(&attestation),
                "clientDataJSON": b64(&self.client_data("webauthn.create", &challenge)),
            },
        })
    }

    pub fn assert(&self, public_key: &Value, counter: u32) -> Value {
        let challenge = challenge_of(public_key);
        let auth_data = self.authenticator_data(counter, false);
        let client_data = self.client_data("webauthn.get", &challenge);
        let mut signed = auth_data.clone();
        signed.extend(Sha256::digest(&client_data));
        let signature: Signature = self.key.sign(&signed);
        json!({
            "id": b64(&self.credential_id),
            "rawId": b64(&self.credential_id),
            "type": "public-key",
            "response": {
                "authenticatorData": b64(&auth_data),
                "clientDataJSON": b64(&client_data),
                "signature": b64(signature.to_der().as_bytes()),
                "userHandle": self.user_handle.as_deref().map(b64),
            },
        })
    }

    fn client_data(&self, kind: &str, challenge: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "type": kind,
            "challenge": challenge,
            "origin": self.origin,
            "crossOrigin": false,
        }))
        .expect("client data is serialisable")
    }

    fn authenticator_data(&self, counter: u32, attested: bool) -> Vec<u8> {
        let mut flags = 0;
        for (set, flag) in [
            (self.user_present, FLAG_USER_PRESENT),
            (self.user_verified, FLAG_USER_VERIFIED),
            (self.backup_eligible, FLAG_BACKUP_ELIGIBLE),
            (self.backup_state, FLAG_BACKUP_STATE),
            (attested, FLAG_ATTESTED),
        ] {
            if set {
                flags |= flag;
            }
        }
        let hash = self
            .rp_id_hash
            .unwrap_or_else(|| Sha256::digest(self.rp_id.as_bytes()).into());
        let mut data = hash.to_vec();
        data.push(flags);
        data.extend(counter.to_be_bytes());
        data
    }

    fn cose_key(&self) -> Vec<u8> {
        let point = self.key.verifying_key().to_encoded_point(false);
        let (x, y) = (
            point.x().expect("an uncompressed point has x"),
            point.y().expect("an uncompressed point has y"),
        );
        let mut key = vec![0xa5, 0x01, 0x02, 0x03, 0x26, 0x20, 0x01, 0x21];
        bytes(&mut key, x);
        key.push(0x22);
        bytes(&mut key, y);
        key
    }
}

fn challenge_of(public_key: &Value) -> String {
    public_key["challenge"]
        .as_str()
        .expect("the options carry a challenge")
        .to_owned()
}

fn b64(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

fn head(out: &mut Vec<u8>, major: u8, length: usize) {
    let major = major << 5;
    match length {
        0..=23 => out.push(major | length as u8),
        24..=255 => out.extend([major | 24, length as u8]),
        _ => {
            out.push(major | 25);
            out.extend((length as u16).to_be_bytes());
        }
    }
}

fn bytes(out: &mut Vec<u8>, data: &[u8]) {
    head(out, 2, data.len());
    out.extend(data);
}

fn text(out: &mut Vec<u8>, value: &str) {
    head(out, 3, value.len());
    out.extend(value.as_bytes());
}
