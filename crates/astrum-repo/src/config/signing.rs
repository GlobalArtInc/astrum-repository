use serde::{Deserialize, Serialize};

use crate::config::problems::Problems;
use crate::config::{Check, Context};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keypair: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
}

impl Signing {
    pub fn redact(&mut self) {
        self.keypair = None;
        self.passphrase = None;
    }
}

impl Check for Signing {
    fn check(&mut self, context: &Context<'_>, stored: Option<&Self>, problems: &mut Problems) {
        if let Some(key) = self.keypair.as_deref().filter(|key| !key.trim().is_empty()) {
            match context.spec.pem_signing_keys() {
                true if !is_pem_private_key(key) => {
                    problems.add("keypair", "must be a PEM-encoded private key")
                }
                false if !is_armored_pgp_key(key) => {
                    problems.add("keypair", "must be an ASCII-armored OpenPGP key")
                }
                true if context.spec.plain_signing_keys() && key.contains("ENCRYPTED") => problems
                    .add(
                        "keypair",
                        "must be a PEM-encoded private key without encryption",
                    ),
                _ => {}
            }
        }
        if context.spec.plain_signing_keys()
            && self
                .passphrase
                .as_deref()
                .is_some_and(|passphrase| !passphrase.is_empty())
        {
            problems.add(
                "passphrase",
                "is not used; give the private key without encryption",
            );
        }
        self.keypair = context.secret(
            self.keypair.take(),
            stored.and_then(|old| old.keypair.as_ref()),
        );
        self.passphrase = context.secret(
            self.passphrase.take(),
            stored.and_then(|old| old.passphrase.as_ref()),
        );
    }
}

pub fn is_armored_pgp_key(text: &str) -> bool {
    let text = text.trim();
    text.starts_with("-----BEGIN PGP ") && text.contains("-----END PGP ")
}

pub fn is_pem_private_key(text: &str) -> bool {
    let text = text.trim();
    text.starts_with("-----BEGIN ")
        && text
            .lines()
            .next()
            .is_some_and(|line| line.ends_with("PRIVATE KEY-----"))
        && text.contains("-----END ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::Spec;

    const PGP: &str =
        "-----BEGIN PGP PRIVATE KEY BLOCK-----\n\nlQ\n-----END PGP PRIVATE KEY BLOCK-----\n";
    const PEM: &str = "-----BEGIN PRIVATE KEY-----\nMII\n-----END PRIVATE KEY-----\n";

    fn check(format: &str, given: serde_json::Value, stored: Option<&Signing>) -> (Signing, usize) {
        let mut signing: Signing = serde_json::from_value(given).unwrap();
        let seal = |plain: &str| format!("sealed:{plain}");
        let context = Context::new(Spec::from_api(format, "hosted").unwrap(), &seal);
        let mut problems = Problems::default();
        signing.check(&context, stored, &mut problems);
        (signing, problems.finish().err().unwrap_or_default().len())
    }

    #[test]
    fn keys_are_recognised_by_their_armour() {
        assert!(is_armored_pgp_key(PGP));
        assert!(!is_armored_pgp_key(PEM));
        assert!(is_pem_private_key(PEM));
        assert!(is_pem_private_key(
            "-----BEGIN RSA PRIVATE KEY-----\nx\n-----END RSA PRIVATE KEY-----"
        ));
        assert!(!is_pem_private_key(PGP));
        assert!(!is_pem_private_key("garbage"));
        assert_eq!(check("yum", json!({"keypair": "garbage"}), None).1, 1);
        assert_eq!(check("alpine", json!({"keypair": PGP}), None).1, 1);
        assert_eq!(check("alpine", json!({"keypair": PEM}), None).1, 0);
        assert_eq!(check("hex", json!({"keypair": PEM}), None).1, 0);
        assert_eq!(check("hex", json!({"keypair": PGP}), None).1, 1);
        let encrypted =
            "-----BEGIN ENCRYPTED PRIVATE KEY-----\nMII\n-----END ENCRYPTED PRIVATE KEY-----\n";
        assert_eq!(check("hex", json!({"keypair": encrypted}), None).1, 1);
        assert_eq!(check("alpine", json!({"keypair": encrypted}), None).1, 0);
        assert_eq!(
            check("hex", json!({"keypair": PEM, "passphrase": "p"}), None).1,
            1
        );
        assert_eq!(check("pacman", json!({"keypair": PEM}), None).1, 1);
        assert_eq!(check("pacman", json!({"keypair": PGP}), None).1, 0);
    }

    #[test]
    fn signing_secrets_are_sealed_and_kept_when_left_out() {
        let (first, problems) = check("yum", json!({"keypair": PGP, "passphrase": "p"}), None);
        assert_eq!(problems, 0);
        assert_eq!(first.keypair, Some(format!("sealed:{PGP}")));
        assert_eq!(first.passphrase.as_deref(), Some("sealed:p"));
        let (kept, _) = check("yum", json!({}), Some(&first));
        assert_eq!(kept, first);
        let (changed, _) = check("terraform", json!({"passphrase": "q"}), Some(&first));
        assert_eq!(changed.keypair, first.keypair);
        assert_eq!(changed.passphrase.as_deref(), Some("sealed:q"));
        let (mut nothing, _) = check("yum", json!({"keypair": ""}), None);
        assert_eq!(nothing, Signing::default());
        nothing.keypair = Some("x".to_owned());
        nothing.redact();
        assert_eq!(serde_json::to_value(nothing).unwrap(), json!({}));
    }
}
