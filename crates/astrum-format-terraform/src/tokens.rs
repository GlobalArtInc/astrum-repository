use std::sync::Mutex;

use astrum_content::ContentError;
use astrum_db::setting;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ConnectionTrait, EntityTrait};
use sha2::Sha256;

use crate::paths;

pub const EXPIRES: &str = "expires";
pub const SIGNATURE: &str = "signature";
pub const LIFETIME_SECONDS: i64 = 15 * 60;
const SECRET_KEY: &str = "terraform.downloadSecret";

#[derive(Default)]
pub struct Secret {
    loaded: Mutex<Option<Vec<u8>>>,
}

impl Secret {
    pub async fn tokens(&self, db: &impl ConnectionTrait) -> Result<Tokens, ContentError> {
        if let Some(secret) = self
            .loaded
            .lock()
            .expect("the secret lock is never poisoned")
            .clone()
        {
            return Ok(Tokens { secret });
        }
        let fresh: [u8; 32] = rand::random();
        setting::Entity::insert(setting::ActiveModel {
            key: ActiveValue::Set(SECRET_KEY.to_owned()),
            value: ActiveValue::Set(serde_json::Value::String(STANDARD.encode(fresh))),
            updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        })
        .on_conflict(
            OnConflict::column(setting::Column::Key)
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(db)
        .await?;
        let stored: String = setting::get(db, SECRET_KEY).await?.ok_or_else(|| {
            ContentError::internal(anyhow::anyhow!("the download secret is missing"))
        })?;
        let secret = STANDARD
            .decode(stored)
            .map_err(|error| ContentError::internal(anyhow::anyhow!("{error}")))?;
        *self
            .loaded
            .lock()
            .expect("the secret lock is never poisoned") = Some(secret.clone());
        Ok(Tokens { secret })
    }
}

pub struct Tokens {
    secret: Vec<u8>,
}

impl Tokens {
    fn mac(&self, repository: &str, path: &str, expires: i64) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.secret).expect("HMAC takes keys of any size");
        mac.update(format!("{repository}\n{path}\n{expires}").as_bytes());
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }

    pub fn sign(&self, repository: &str, relative: &str, target: &str) -> String {
        let expires = chrono::Utc::now().timestamp() + LIFETIME_SECONDS;
        let separator = if relative.contains('?') { '&' } else { '?' };
        format!(
            "{relative}{separator}{EXPIRES}={expires}&{SIGNATURE}={}",
            self.mac(repository, target, expires)
        )
    }

    pub fn link(&self, repository: &str, from: &str, to: &str) -> String {
        self.sign(repository, &paths::relative(from, to), to)
    }

    pub fn resign(&self, repository: &str, url: &str, document: &str) -> String {
        let Some((base, query)) = url.split_once('?') else {
            return url.to_owned();
        };
        if url.contains("://") || !query.contains(&format!("{SIGNATURE}=")) {
            return url.to_owned();
        }
        let target = match base.get(1..).and_then(|rest| rest.find("//")) {
            Some(at) => &base[..at + 1],
            None => base,
        };
        match paths::resolve(document, target) {
            Some(path) => self.sign(repository, base, &path),
            None => url.to_owned(),
        }
    }

    pub fn valid(&self, repository: &str, path: &str, query: Option<&str>) -> bool {
        let Some(query) = query else {
            return false;
        };
        let mut expires = None;
        let mut signature = None;
        for (name, value) in url::form_urlencoded::parse(query.as_bytes()) {
            match name.as_ref() {
                EXPIRES => expires = value.parse::<i64>().ok(),
                SIGNATURE => signature = Some(value.into_owned()),
                _ => {}
            }
        }
        let (Some(expires), Some(signature)) = (expires, signature) else {
            return false;
        };
        if expires < chrono::Utc::now().timestamp() {
            return false;
        }
        let expected = self.mac(repository, path, expires);
        subtle::ConstantTimeEq::ct_eq(expected.as_bytes(), signature.as_bytes()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        Tokens {
            secret: b"0123456789abcdef".to_vec(),
        }
    }

    fn query(url: &str) -> Option<&str> {
        url.split_once('?').map(|(_, query)| query)
    }

    #[test]
    fn links_are_good_for_their_path_and_repository_only() {
        let tokens = tokens();
        let link = tokens.link(
            "hosted",
            "/v1/providers/a/t/1.0.0/download/linux/amd64",
            "/v1/providers/a/t/1.0.0/f.zip",
        );
        assert!(link.starts_with("../../f.zip?expires="));
        assert!(tokens.valid("hosted", "/v1/providers/a/t/1.0.0/f.zip", query(&link)));
        assert!(!tokens.valid("other", "/v1/providers/a/t/1.0.0/f.zip", query(&link)));
        assert!(!tokens.valid("hosted", "/v1/providers/a/t/1.0.0/g.zip", query(&link)));
        assert!(!tokens.valid("hosted", "/x", None));
        let stale = format!("expires=1&signature={}", tokens.mac("hosted", "/x", 1));
        assert!(!tokens.valid("hosted", "/x", Some(&stale)));
    }

    #[test]
    fn groups_sign_their_members_links_again() {
        let tokens = tokens();
        let document = "/v1/modules/a/b/c/1.0.0/download";
        let member = tokens.sign(
            "member",
            "./b-c-1.0.0.tar.gz",
            "/v1/modules/a/b/c/1.0.0/b-c-1.0.0.tar.gz",
        );
        let resigned = tokens.resign("group", &member, document);
        assert!(resigned.starts_with("./b-c-1.0.0.tar.gz?expires="));
        assert!(tokens.valid(
            "group",
            "/v1/modules/a/b/c/1.0.0/b-c-1.0.0.tar.gz",
            query(&resigned)
        ));
        let with_subdir = tokens.resign(
            "group",
            "./b-c-1.0.0.tar.gz//modules/x?expires=1&signature=a",
            document,
        );
        assert!(
            with_subdir.starts_with("./b-c-1.0.0.tar.gz//modules/x?expires="),
            "{with_subdir}"
        );
        assert_eq!(
            tokens.resign("group", "https://x/y.zip?signature=1", document),
            "https://x/y.zip?signature=1"
        );
        assert_eq!(
            tokens.resign("group", "./plain.zip", document),
            "./plain.zip"
        );
    }
}
