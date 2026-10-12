use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use rand::RngCore;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct SaslError(pub String);

pub fn cram_md5(username: &str, password: &str, challenge: &[u8]) -> Vec<u8> {
    let mut mac =
        Hmac::<Md5>::new_from_slice(password.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(challenge);
    format!("{username} {}", hex(&mac.finalize().into_bytes())).into_bytes()
}

pub struct DigestMd5 {
    pub username: String,
    pub password: String,
    pub realm: Option<String>,
    pub digest_uri: String,
}

pub struct DigestResponse {
    pub response: Vec<u8>,
    pub expected_rspauth: String,
}

impl DigestMd5 {
    pub fn respond(&self, challenge: &[u8]) -> Result<DigestResponse, SaslError> {
        let mut cnonce = [0u8; 16];
        rand::rng().fill_bytes(&mut cnonce);
        self.respond_with(challenge, &hex(&cnonce))
    }

    pub fn respond_with(
        &self,
        challenge: &[u8],
        cnonce: &str,
    ) -> Result<DigestResponse, SaslError> {
        let text = std::str::from_utf8(challenge)
            .map_err(|_| SaslError("the DIGEST-MD5 challenge is not text".to_owned()))?;
        let directives = directives(text)?;
        let nonce = directives
            .get("nonce")
            .ok_or_else(|| SaslError("the DIGEST-MD5 challenge has no nonce".to_owned()))?;
        if let Some(qop) = directives.get("qop")
            && !qop.split(',').any(|offered| offered.trim() == "auth")
        {
            return Err(SaslError(
                "the server offers DIGEST-MD5 only with integrity or privacy protection".to_owned(),
            ));
        }
        if directives
            .get("algorithm")
            .is_some_and(|algorithm| algorithm != "md5-sess")
        {
            return Err(SaslError(
                "the DIGEST-MD5 challenge asks for an unknown algorithm".to_owned(),
            ));
        }
        let realm = self
            .realm
            .clone()
            .filter(|realm| !realm.is_empty())
            .or_else(|| directives.get("realm").cloned())
            .unwrap_or_default();
        let utf8 = directives
            .get("charset")
            .is_some_and(|charset| charset.eq_ignore_ascii_case("utf-8"));
        let digest_uri = &self.digest_uri;
        let nc = "00000001";
        let qop = "auth";
        let mut inner = Md5::new();
        inner.update(self.username.as_bytes());
        inner.update(b":");
        inner.update(realm.as_bytes());
        inner.update(b":");
        inner.update(self.password.as_bytes());
        let mut a1 = inner.finalize().to_vec();
        a1.extend_from_slice(format!(":{nonce}:{cnonce}").as_bytes());
        let ha1 = hex(&Md5::digest(&a1));
        let response = kd(
            &ha1,
            &format!(
                "{nonce}:{nc}:{cnonce}:{qop}:{}",
                hex(&Md5::digest(
                    format!("AUTHENTICATE:{digest_uri}").as_bytes()
                ))
            ),
        );
        let expected_rspauth = kd(
            &ha1,
            &format!(
                "{nonce}:{nc}:{cnonce}:{qop}:{}",
                hex(&Md5::digest(format!(":{digest_uri}").as_bytes()))
            ),
        );
        let mut reply = format!("username=\"{}\"", quote(&self.username));
        if !realm.is_empty() {
            reply.push_str(&format!(",realm=\"{}\"", quote(&realm)));
        }
        reply.push_str(&format!(
            ",nonce=\"{}\",cnonce=\"{cnonce}\",nc={nc},qop={qop},digest-uri=\"{digest_uri}\",response={response}",
            quote(nonce)
        ));
        if utf8 {
            reply.push_str(",charset=utf-8");
        }
        Ok(DigestResponse {
            response: reply.into_bytes(),
            expected_rspauth,
        })
    }
}

pub fn rspauth(final_challenge: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(final_challenge).ok()?;
    directives(text).ok()?.get("rspauth").cloned()
}

fn kd(key: &str, data: &str) -> String {
    hex(&Md5::digest(format!("{key}:{data}").as_bytes()))
}

fn quote(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn directives(text: &str) -> Result<BTreeMap<String, String>, SaslError> {
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    let mut found = BTreeMap::new();
    while at < chars.len() {
        while at < chars.len() && (chars[at] == ',' || chars[at].is_whitespace()) {
            at += 1;
        }
        if at >= chars.len() {
            break;
        }
        let start = at;
        while at < chars.len() && chars[at] != '=' {
            at += 1;
        }
        if at >= chars.len() {
            return Err(SaslError("a DIGEST-MD5 directive has no value".to_owned()));
        }
        let name: String = chars[start..at]
            .iter()
            .collect::<String>()
            .trim()
            .to_lowercase();
        at += 1;
        let mut value = String::new();
        if chars.get(at) == Some(&'"') {
            at += 1;
            loop {
                match chars.get(at) {
                    None => return Err(SaslError("a DIGEST-MD5 quote is not closed".to_owned())),
                    Some('"') => {
                        at += 1;
                        break;
                    }
                    Some('\\') => {
                        if let Some(next) = chars.get(at + 1) {
                            value.push(*next);
                        }
                        at += 2;
                    }
                    Some(c) => {
                        value.push(*c);
                        at += 1;
                    }
                }
            }
        } else {
            while at < chars.len() && chars[at] != ',' {
                value.push(chars[at]);
                at += 1;
            }
            value = value.trim().to_owned();
        }
        found.entry(name).or_insert(value);
    }
    Ok(found)
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cram_md5_follows_rfc_2195() {
        let response = cram_md5(
            "tim",
            "tanstaaftanstaaf",
            b"<1896.697170952@postoffice.reston.mci.net>",
        );
        assert_eq!(
            String::from_utf8(response).unwrap(),
            "tim b913a602c7eda7a495b4e6e7334d3890"
        );
    }

    #[test]
    fn digest_md5_follows_rfc_2831() {
        let client = DigestMd5 {
            username: "chris".to_owned(),
            password: "secret".to_owned(),
            realm: None,
            digest_uri: "imap/elwood.innosoft.com".to_owned(),
        };
        let challenge = b"realm=\"elwood.innosoft.com\",nonce=\"OA6MG9tEQGm2hh\",qop=\"auth\",algorithm=md5-sess,charset=utf-8";
        let reply = client.respond_with(challenge, "OA6MHXh6VqTrRk").unwrap();
        let text = String::from_utf8(reply.response).unwrap();
        let sent = directives(&text).unwrap();
        assert_eq!(sent["username"], "chris");
        assert_eq!(sent["realm"], "elwood.innosoft.com");
        assert_eq!(sent["digest-uri"], "imap/elwood.innosoft.com");
        assert_eq!(sent["nc"], "00000001");
        assert_eq!(sent["charset"], "utf-8");
        assert_eq!(sent["response"], "d388dad90d4bbd760a152321f2143af7");
        assert_eq!(reply.expected_rspauth, "ea40f60335c427b5527b84dbabcdfffd");
    }

    #[test]
    fn digest_md5_refuses_what_it_cannot_do() {
        let client = DigestMd5 {
            username: "u".to_owned(),
            password: "p".to_owned(),
            realm: Some("r".to_owned()),
            digest_uri: "ldap/h".to_owned(),
        };
        assert!(client.respond_with(b"realm=\"x\"", "c").is_err());
        assert!(
            client
                .respond_with(b"nonce=\"n\",qop=\"auth-conf\"", "c")
                .is_err()
        );
        let reply = client.respond_with(b"nonce=\"n\"", "c").unwrap();
        assert!(
            String::from_utf8(reply.response)
                .unwrap()
                .contains("realm=\"r\"")
        );
        assert_eq!(rspauth(b"rspauth=abc"), Some("abc".to_owned()));
    }
}
