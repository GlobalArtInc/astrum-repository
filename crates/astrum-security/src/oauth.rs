use astrum_db::entity::{oauth_authorization, oauth_client, oauth_grant};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, QueryOrder,
    Statement,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::subject::{Scope, Subject};
use crate::tokens::hash;
use crate::{Error, Security, password, realm};

pub const ACCESS_PREFIX: &str = "astrum_at_";
pub const REFRESH_PREFIX: &str = "astrum_rt_";
const CLIENT_PREFIX: &str = "astrum-mcp-";
const TOKEN_LENGTH: usize = 48;
pub const ACCESS_TTL: Duration = Duration::hours(1);
pub const REFRESH_TTL: Duration = Duration::days(30);
pub const REQUEST_TTL: Duration = Duration::minutes(10);
pub const CODE_TTL: Duration = Duration::seconds(60);
const UNUSED_CLIENT_TTL: Duration = Duration::hours(24);
const TOUCH_EVERY: Duration = Duration::seconds(60);
pub const MAX_REDIRECT_URIS: usize = 10;
pub const MAX_CLIENT_NAME: usize = 100;
pub const READ_SCOPE: &str = "read";
pub const WRITE_SCOPE: &str = "write";

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error("the client is not known")]
    InvalidClient,
    #[error("{0}")]
    InvalidGrant(String),
    #[error("{0}")]
    InvalidRedirectUri(String),
    #[error("{0}")]
    InvalidClientMetadata(String),
    #[error("{0}")]
    InvalidScope(String),
    #[error(transparent)]
    Internal(#[from] Error),
}

impl OAuthError {
    pub fn code(&self) -> &'static str {
        match self {
            OAuthError::InvalidRequest(_) => "invalid_request",
            OAuthError::InvalidClient => "invalid_client",
            OAuthError::InvalidGrant(_) => "invalid_grant",
            OAuthError::InvalidRedirectUri(_) => "invalid_redirect_uri",
            OAuthError::InvalidClientMetadata(_) => "invalid_client_metadata",
            OAuthError::InvalidScope(_) => "invalid_scope",
            OAuthError::Internal(_) => "server_error",
        }
    }
}

impl From<sea_orm::DbErr> for OAuthError {
    fn from(error: sea_orm::DbErr) -> Self {
        OAuthError::Internal(Error::from(error))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingAuthorization {
    pub id: String,
    pub client: Client,
    pub redirect_uri: String,
    pub requested_scope: Vec<String>,
    pub resource: Option<String>,
    pub expires_at: DateTime<FixedOffset>,
}

pub struct NewAuthorization<'a> {
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub code_challenge: &'a str,
    pub scope: Vec<String>,
    pub state: Option<&'a str>,
    pub resource: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantInfo {
    pub id: Uuid,
    pub client_id: String,
    pub client_name: String,
    pub user_id: String,
    pub scope: Scope,
    pub created_at: DateTime<FixedOffset>,
    pub last_used_at: Option<DateTime<FixedOffset>>,
    pub refresh_expires_at: DateTime<FixedOffset>,
}

#[derive(Clone, Debug)]
pub struct OAuthCaller {
    pub subject: Subject,
    pub client_name: String,
    pub resource: Option<String>,
}

pub fn looks_like_access_token(value: &str) -> bool {
    shaped(value, ACCESS_PREFIX)
}

pub fn looks_like_refresh_token(value: &str) -> bool {
    shaped(value, REFRESH_PREFIX)
}

fn shaped(value: &str, prefix: &str) -> bool {
    value.len() == prefix.len() + TOKEN_LENGTH
        && value.starts_with(prefix)
        && value[prefix.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
}

fn generate(prefix: &str) -> String {
    format!("{prefix}{}", password::random(TOKEN_LENGTH))
}

pub fn scope_of(granted: &str) -> Scope {
    match granted.split_whitespace().any(|scope| scope == WRITE_SCOPE) {
        true => Scope::Write,
        false => Scope::Read,
    }
}

pub fn scope_text(scope: Scope) -> String {
    match scope {
        Scope::Read => READ_SCOPE.to_owned(),
        Scope::Write => format!("{READ_SCOPE} {WRITE_SCOPE}"),
    }
}

pub fn parse_scope(text: Option<&str>) -> Result<Vec<String>, OAuthError> {
    let mut scopes = Vec::new();
    for scope in text.unwrap_or_default().split_whitespace() {
        match scope {
            READ_SCOPE | WRITE_SCOPE => {
                if !scopes.iter().any(|known| known == scope) {
                    scopes.push(scope.to_owned());
                }
            }
            other => {
                return Err(OAuthError::InvalidScope(format!(
                    "the scope {other} is not known; use read and write"
                )));
            }
        }
    }
    Ok(scopes)
}

pub fn valid_redirect_uri(uri: &str) -> Result<(), String> {
    let parsed = url::Url::parse(uri).map_err(|_| format!("{uri} is not an absolute URI"))?;
    if parsed.fragment().is_some() {
        return Err(format!("{uri} must not have a fragment"));
    }
    match parsed.scheme() {
        "https" if parsed.host().is_some() => Ok(()),
        "http" if is_loopback(&parsed) => Ok(()),
        "http" => Err(format!(
            "{uri} uses http on a host that is not this computer; use https"
        )),
        "javascript" | "data" | "file" | "vbscript" | "blob" | "about" | "ftp" | "ws" | "wss" => {
            Err(format!("{uri} uses a scheme that cannot take a redirect"))
        }
        scheme if scheme.len() >= 3 => Ok(()),
        _ => Err(format!("{uri} has no usable scheme")),
    }
}

fn is_loopback(url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        None => false,
    }
}

pub fn redirect_matches(registered: &str, presented: &str) -> bool {
    if registered == presented {
        return true;
    }
    let (Ok(registered), Ok(presented)) = (url::Url::parse(registered), url::Url::parse(presented))
    else {
        return false;
    };
    registered.scheme() == "http"
        && presented.scheme() == "http"
        && is_loopback(&registered)
        && registered.host() == presented.host()
        && registered.path() == presented.path()
        && registered.query() == presented.query()
}

pub fn pkce_matches(challenge: &str, verifier: &str) -> bool {
    let shaped = (43..=128).contains(&verifier.len())
        && verifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'));
    if !shaped {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    bool::from(computed.as_bytes().ct_eq(challenge.as_bytes()))
}

pub fn with_query(uri: &str, pairs: &[(&str, &str)]) -> String {
    match url::Url::parse(uri) {
        Ok(mut url) => {
            {
                let mut query = url.query_pairs_mut();
                for (key, value) in pairs {
                    query.append_pair(key, value);
                }
            }
            url.to_string()
        }
        Err(_) => uri.to_owned(),
    }
}

fn client_of(row: oauth_client::Model) -> Client {
    Client {
        redirect_uris: row
            .redirect_uris
            .as_array()
            .map(|uris| {
                uris.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        id: row.id,
        name: row.name,
        created_at: row.created_at,
    }
}

impl Security {
    pub async fn register_oauth_client(
        &self,
        name: Option<&str>,
        redirect_uris: &[String],
    ) -> Result<Client, OAuthError> {
        if redirect_uris.is_empty() {
            return Err(OAuthError::InvalidRedirectUri(
                "redirect_uris needs at least one URI".to_owned(),
            ));
        }
        if redirect_uris.len() > MAX_REDIRECT_URIS {
            return Err(OAuthError::InvalidRedirectUri(format!(
                "at most {MAX_REDIRECT_URIS} redirect URIs are accepted"
            )));
        }
        for uri in redirect_uris {
            valid_redirect_uri(uri).map_err(OAuthError::InvalidRedirectUri)?;
        }
        let name = name.map(str::trim).filter(|name| !name.is_empty());
        if name.is_some_and(|name| name.chars().count() > MAX_CLIENT_NAME) {
            return Err(OAuthError::InvalidClientMetadata(format!(
                "client_name may hold at most {MAX_CLIENT_NAME} characters"
            )));
        }
        self.sweep_oauth().await?;
        let now = Utc::now().fixed_offset();
        let row = oauth_client::ActiveModel {
            id: ActiveValue::Set(format!(
                "{CLIENT_PREFIX}{}",
                password::random(24).to_lowercase()
            )),
            name: ActiveValue::Set(name.unwrap_or("MCP client").to_owned()),
            redirect_uris: ActiveValue::Set(json!(redirect_uris)),
            created_at: ActiveValue::Set(now),
        };
        let stored = oauth_client::Entity::insert(row)
            .exec_with_returning(self.db())
            .await?;
        Ok(client_of(stored))
    }

    pub async fn oauth_client(&self, id: &str) -> Result<Option<Client>, Error> {
        Ok(oauth_client::Entity::find_by_id(id.to_owned())
            .one(self.db())
            .await?
            .map(client_of))
    }

    pub async fn begin_oauth_authorization(
        &self,
        request: NewAuthorization<'_>,
    ) -> Result<String, OAuthError> {
        let id = password::random(32);
        let now = Utc::now().fixed_offset();
        oauth_authorization::Entity::insert(oauth_authorization::ActiveModel {
            id: ActiveValue::Set(id.clone()),
            client_id: ActiveValue::Set(request.client_id.to_owned()),
            redirect_uri: ActiveValue::Set(request.redirect_uri.to_owned()),
            code_challenge: ActiveValue::Set(request.code_challenge.to_owned()),
            requested_scope: ActiveValue::Set(request.scope.join(" ")),
            state: ActiveValue::Set(request.state.map(str::to_owned)),
            resource: ActiveValue::Set(request.resource.map(str::to_owned)),
            user_id: ActiveValue::Set(None),
            realm: ActiveValue::Set(None),
            granted_scope: ActiveValue::Set(None),
            code_hash: ActiveValue::Set(None),
            created_at: ActiveValue::Set(now),
            expires_at: ActiveValue::Set(now + REQUEST_TTL),
        })
        .exec(self.db())
        .await?;
        Ok(id)
    }

    pub async fn pending_oauth_authorization(
        &self,
        id: &str,
    ) -> Result<Option<PendingAuthorization>, Error> {
        let Some(row) = oauth_authorization::Entity::find_by_id(id.to_owned())
            .one(self.db())
            .await?
        else {
            return Ok(None);
        };
        if row.user_id.is_some() || row.expires_at <= Utc::now().fixed_offset() {
            return Ok(None);
        }
        let Some(client) = self.oauth_client(&row.client_id).await? else {
            return Ok(None);
        };
        Ok(Some(PendingAuthorization {
            id: row.id,
            client,
            redirect_uri: row.redirect_uri,
            requested_scope: row
                .requested_scope
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
            resource: row.resource,
            expires_at: row.expires_at,
        }))
    }

    pub async fn approve_oauth_authorization(
        &self,
        id: &str,
        subject: &Subject,
        scope: Scope,
    ) -> Result<Option<String>, Error> {
        let code = generate("");
        let now = Utc::now().fixed_offset();
        let updated = oauth_authorization::Entity::update_many()
            .col_expr(
                oauth_authorization::Column::UserId,
                Expr::value(subject.user_id.clone()),
            )
            .col_expr(
                oauth_authorization::Column::Realm,
                Expr::value(subject.realm.clone()),
            )
            .col_expr(
                oauth_authorization::Column::GrantedScope,
                Expr::value(scope_text(scope)),
            )
            .col_expr(
                oauth_authorization::Column::CodeHash,
                Expr::value(hash(&code)),
            )
            .col_expr(
                oauth_authorization::Column::ExpiresAt,
                Expr::value(now + CODE_TTL),
            )
            .filter(oauth_authorization::Column::Id.eq(id))
            .filter(oauth_authorization::Column::UserId.is_null())
            .filter(oauth_authorization::Column::ExpiresAt.gt(now))
            .exec_with_returning(self.db())
            .await?;
        let Some(row) = updated.into_iter().next() else {
            return Ok(None);
        };
        let mut pairs = vec![("code", code.as_str())];
        if let Some(state) = row.state.as_deref() {
            pairs.push(("state", state));
        }
        Ok(Some(with_query(&row.redirect_uri, &pairs)))
    }

    pub async fn deny_oauth_authorization(&self, id: &str) -> Result<Option<String>, Error> {
        let Some(row) = oauth_authorization::Entity::find_by_id(id.to_owned())
            .one(self.db())
            .await?
        else {
            return Ok(None);
        };
        if row.user_id.is_some() {
            return Ok(None);
        }
        oauth_authorization::Entity::delete_by_id(id.to_owned())
            .exec(self.db())
            .await?;
        let mut pairs = vec![
            ("error", "access_denied"),
            ("error_description", "the user refused the access"),
        ];
        if let Some(state) = row.state.as_deref() {
            pairs.push(("state", state));
        }
        Ok(Some(with_query(&row.redirect_uri, &pairs)))
    }

    pub async fn exchange_oauth_code(
        &self,
        client_id: &str,
        code: &str,
        redirect_uri: &str,
        verifier: &str,
    ) -> Result<(TokenSet, GrantInfo), OAuthError> {
        let consumed = oauth_authorization::Entity::find()
            .filter(oauth_authorization::Column::CodeHash.eq(hash(code)))
            .one(self.db())
            .await?
            .ok_or_else(|| {
                OAuthError::InvalidGrant("the code is not known or was used".to_owned())
            })?;
        oauth_authorization::Entity::delete_by_id(consumed.id.clone())
            .exec(self.db())
            .await?;
        let now = Utc::now().fixed_offset();
        if consumed.expires_at <= now {
            return Err(OAuthError::InvalidGrant("the code expired".to_owned()));
        }
        if consumed.client_id != client_id {
            return Err(OAuthError::InvalidGrant(
                "the code was issued to another client".to_owned(),
            ));
        }
        if consumed.redirect_uri != redirect_uri {
            return Err(OAuthError::InvalidGrant(
                "redirect_uri differs from the one of the authorization request".to_owned(),
            ));
        }
        if !pkce_matches(&consumed.code_challenge, verifier) {
            return Err(OAuthError::InvalidGrant(
                "code_verifier does not match the code challenge".to_owned(),
            ));
        }
        let (Some(user_id), Some(user_realm), Some(scope)) =
            (consumed.user_id, consumed.realm, consumed.granted_scope)
        else {
            return Err(OAuthError::InvalidGrant(
                "the authorization was not approved".to_owned(),
            ));
        };
        let access = generate(ACCESS_PREFIX);
        let refresh = generate(REFRESH_PREFIX);
        let row = oauth_grant::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            client_id: ActiveValue::Set(client_id.to_owned()),
            user_id: ActiveValue::Set(user_id),
            realm: ActiveValue::Set(user_realm),
            scope: ActiveValue::Set(scope.clone()),
            resource: ActiveValue::Set(consumed.resource),
            access_hash: ActiveValue::Set(hash(&access)),
            access_expires_at: ActiveValue::Set(now + ACCESS_TTL),
            refresh_hash: ActiveValue::Set(hash(&refresh)),
            refresh_expires_at: ActiveValue::Set(now + REFRESH_TTL),
            created_at: ActiveValue::Set(now),
            last_used_at: ActiveValue::Set(None),
        };
        let stored = oauth_grant::Entity::insert(row)
            .exec_with_returning(self.db())
            .await?;
        let info = self.grant_info(stored).await?;
        Ok((
            TokenSet {
                access_token: access,
                refresh_token: refresh,
                expires_in: ACCESS_TTL.num_seconds(),
                scope,
            },
            info,
        ))
    }

    pub async fn refresh_oauth_tokens(
        &self,
        client_id: &str,
        refresh_token: &str,
    ) -> Result<TokenSet, OAuthError> {
        let invalid = || OAuthError::InvalidGrant("the refresh token is not valid".to_owned());
        if !looks_like_refresh_token(refresh_token) {
            return Err(invalid());
        }
        let row = oauth_grant::Entity::find()
            .filter(oauth_grant::Column::RefreshHash.eq(hash(refresh_token)))
            .one(self.db())
            .await?
            .ok_or_else(invalid)?;
        let now = Utc::now().fixed_offset();
        if row.client_id != client_id {
            return Err(invalid());
        }
        if row.refresh_expires_at <= now {
            oauth_grant::Entity::delete_by_id(row.id)
                .exec(self.db())
                .await?;
            return Err(OAuthError::InvalidGrant(
                "the refresh token expired; sign in again".to_owned(),
            ));
        }
        if self.grant_subject(&row.user_id, &row.realm).await.is_none() {
            oauth_grant::Entity::delete_by_id(row.id)
                .exec(self.db())
                .await?;
            return Err(OAuthError::InvalidGrant(
                "the user can no longer sign in".to_owned(),
            ));
        }
        let access = generate(ACCESS_PREFIX);
        let refresh = generate(REFRESH_PREFIX);
        let rotated = oauth_grant::Entity::update_many()
            .col_expr(oauth_grant::Column::AccessHash, Expr::value(hash(&access)))
            .col_expr(
                oauth_grant::Column::AccessExpiresAt,
                Expr::value(now + ACCESS_TTL),
            )
            .col_expr(
                oauth_grant::Column::RefreshHash,
                Expr::value(hash(&refresh)),
            )
            .col_expr(
                oauth_grant::Column::RefreshExpiresAt,
                Expr::value(now + REFRESH_TTL),
            )
            .filter(oauth_grant::Column::Id.eq(row.id))
            .filter(oauth_grant::Column::RefreshHash.eq(hash(refresh_token)))
            .exec(self.db())
            .await?;
        if rotated.rows_affected == 0 {
            return Err(invalid());
        }
        Ok(TokenSet {
            access_token: access,
            refresh_token: refresh,
            expires_in: ACCESS_TTL.num_seconds(),
            scope: row.scope,
        })
    }

    pub async fn revoke_oauth_token(&self, token: &str) -> Result<bool, Error> {
        let column = match (
            looks_like_access_token(token),
            looks_like_refresh_token(token),
        ) {
            (true, _) => oauth_grant::Column::AccessHash,
            (_, true) => oauth_grant::Column::RefreshHash,
            _ => return Ok(false),
        };
        let deleted = oauth_grant::Entity::delete_many()
            .filter(column.eq(hash(token)))
            .exec(self.db())
            .await?;
        Ok(deleted.rows_affected > 0)
    }

    pub async fn authenticate_oauth(&self, token: &str) -> Result<Option<OAuthCaller>, Error> {
        if !looks_like_access_token(token) {
            return Ok(None);
        }
        let Some(row) = oauth_grant::Entity::find()
            .filter(oauth_grant::Column::AccessHash.eq(hash(token)))
            .one(self.db())
            .await?
        else {
            return Ok(None);
        };
        let now = Utc::now().fixed_offset();
        if row.access_expires_at <= now {
            return Ok(None);
        }
        let Some(subject) = self.grant_subject(&row.user_id, &row.realm).await else {
            return Ok(None);
        };
        if row.last_used_at.is_none_or(|used| now - used > TOUCH_EVERY) {
            oauth_grant::Entity::update(oauth_grant::ActiveModel {
                id: ActiveValue::Unchanged(row.id),
                last_used_at: ActiveValue::Set(Some(now)),
                ..Default::default()
            })
            .exec(self.db())
            .await?;
        }
        let client_name = self
            .oauth_client(&row.client_id)
            .await?
            .map(|client| client.name)
            .unwrap_or_default();
        Ok(Some(OAuthCaller {
            subject: subject.with_scope(scope_of(&row.scope)),
            client_name,
            resource: row.resource,
        }))
    }

    pub async fn oauth_grants(&self, user_id: &str) -> Result<Vec<GrantInfo>, Error> {
        let rows = oauth_grant::Entity::find()
            .filter(oauth_grant::Column::UserId.eq(user_id))
            .order_by_asc(oauth_grant::Column::CreatedAt)
            .all(self.db())
            .await?;
        let mut grants = Vec::with_capacity(rows.len());
        for row in rows {
            grants.push(self.grant_info(row).await?);
        }
        Ok(grants)
    }

    pub async fn delete_oauth_grant(
        &self,
        id: Uuid,
        user_id: &str,
    ) -> Result<Option<GrantInfo>, Error> {
        let Some(row) = oauth_grant::Entity::find_by_id(id)
            .filter(oauth_grant::Column::UserId.eq(user_id))
            .one(self.db())
            .await?
        else {
            return Ok(None);
        };
        let info = self.grant_info(row).await?;
        oauth_grant::Entity::delete_by_id(id)
            .exec(self.db())
            .await?;
        Ok(Some(info))
    }

    pub async fn delete_oauth_grants_of(&self, user_id: &str) -> Result<u64, Error> {
        let deleted = oauth_grant::Entity::delete_many()
            .filter(oauth_grant::Column::UserId.eq(user_id))
            .exec(self.db())
            .await?;
        Ok(deleted.rows_affected)
    }

    async fn grant_info(&self, row: oauth_grant::Model) -> Result<GrantInfo, Error> {
        let client_name = self
            .oauth_client(&row.client_id)
            .await?
            .map(|client| client.name)
            .unwrap_or_default();
        Ok(GrantInfo {
            id: row.id,
            client_id: row.client_id,
            client_name,
            user_id: row.user_id,
            scope: scope_of(&row.scope),
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            refresh_expires_at: row.refresh_expires_at,
        })
    }

    async fn grant_subject(&self, user_id: &str, user_realm: &str) -> Option<Subject> {
        let model = self.model();
        match user_realm {
            realm::LDAP => match self.ldap_subject(user_id).await {
                Ok(subject) => subject,
                Err(error) => {
                    tracing::warn!(user = %user_id, %error, "LDAP cannot confirm an agent's access");
                    None
                }
            },
            realm::OIDC => model.oidc_subject(user_id),
            other => model.local_subject(user_id, other),
        }
    }

    async fn sweep_oauth(&self) -> Result<(), Error> {
        let now = Utc::now().fixed_offset();
        oauth_authorization::Entity::delete_many()
            .filter(oauth_authorization::Column::ExpiresAt.lte(now))
            .exec(self.db())
            .await?;
        oauth_grant::Entity::delete_many()
            .filter(oauth_grant::Column::RefreshExpiresAt.lte(now))
            .exec(self.db())
            .await?;
        self.db()
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM oauth_client c WHERE c.created_at < $1 \
                 AND NOT EXISTS (SELECT 1 FROM oauth_grant g WHERE g.client_id = c.id) \
                 AND NOT EXISTS (SELECT 1 FROM oauth_authorization a WHERE a.client_id = c.id)",
                [(now - UNUSED_CLIENT_TTL).into()],
            ))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_have_their_own_shapes() {
        let access = generate(ACCESS_PREFIX);
        let refresh = generate(REFRESH_PREFIX);
        assert!(looks_like_access_token(&access), "{access}");
        assert!(!looks_like_refresh_token(&access));
        assert!(looks_like_refresh_token(&refresh));
        assert!(!crate::tokens::looks_like_token(&access));
        assert!(!looks_like_access_token("astrum_at_short"));
    }

    #[test]
    fn pkce_follows_the_example_of_rfc_7636() {
        // RFC 7636 Appendix B
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(pkce_matches(challenge, verifier));
        assert!(!pkce_matches(challenge, &verifier.replace('d', "e")));
        assert!(!pkce_matches(challenge, "short"));
    }

    #[test]
    fn redirect_uris_must_be_safe_places() {
        for good in [
            "https://claude.ai/api/mcp/auth_callback",
            "http://127.0.0.1:33418/callback",
            "http://localhost/callback",
            "http://[::1]:9000/cb",
            "vscode://anthropic.claude-code/callback",
            "com.example.app:/oauth",
        ] {
            assert!(valid_redirect_uri(good).is_ok(), "{good}");
        }
        for bad in [
            "http://evil.example/cb",
            "javascript:alert(1)",
            "data:text/html,x",
            "https://x.example/cb#frag",
            "/relative",
            "x:y",
        ] {
            assert!(valid_redirect_uri(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn loopback_redirects_match_on_any_port() {
        assert!(redirect_matches(
            "http://127.0.0.1:3000/callback",
            "http://127.0.0.1:41234/callback"
        ));
        assert!(!redirect_matches(
            "http://127.0.0.1:3000/callback",
            "http://127.0.0.1:3000/other"
        ));
        assert!(!redirect_matches(
            "https://claude.ai/cb",
            "https://claude.ai/cb2"
        ));
        assert!(redirect_matches(
            "https://claude.ai/cb",
            "https://claude.ai/cb"
        ));
    }

    #[test]
    fn scopes_are_read_and_write() {
        assert_eq!(parse_scope(None).unwrap(), Vec::<String>::new());
        assert_eq!(
            parse_scope(Some("write read write")).unwrap(),
            ["write", "read"]
        );
        assert!(parse_scope(Some("admin")).is_err());
        assert_eq!(scope_of("read write"), Scope::Write);
        assert_eq!(scope_of("read"), Scope::Read);
        assert_eq!(scope_text(Scope::Write), "read write");
    }

    #[test]
    fn redirects_keep_their_own_query() {
        assert_eq!(
            with_query(
                "https://x.example/cb?a=1",
                &[("code", "c d"), ("state", "s")]
            ),
            "https://x.example/cb?a=1&code=c+d&state=s"
        );
    }
}
