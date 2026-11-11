use astrum_db::entity::share_link;
use astrum_security::{password, tokens};
use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, QueryFilter, QueryOrder,
    Statement,
};
use uuid::Uuid;

use crate::{ContentError, normalize};

pub const SECRET_PREFIX: &str = "astrum_share_";
const SECRET_LENGTH: usize = 45;
pub const KEPT_AFTER_EXPIRY: Duration = Duration::days(30);

pub type Link = share_link::Model;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Portion {
    pub offset: u64,
    pub length: u64,
    pub size: u64,
}

impl Portion {
    pub fn whole(size: u64) -> Self {
        Self {
            offset: 0,
            length: size,
            size,
        }
    }

    fn starts_download(&self) -> bool {
        self.offset == 0
    }

    fn bytes(&self) -> i64 {
        i64::try_from(self.length).unwrap_or(i64::MAX)
    }

    fn total(&self) -> i64 {
        i64::try_from(self.size).unwrap_or(i64::MAX)
    }
}

#[derive(Clone, Debug)]
pub struct NewLink {
    pub repository: String,
    pub path: String,
    pub prefix: bool,
    pub note: String,
    pub created_by: String,
    pub expires_at: DateTime<Utc>,
    pub max_downloads: Option<i32>,
}

pub fn looks_like_secret(value: &str) -> bool {
    value.len() == SECRET_PREFIX.len() + SECRET_LENGTH
        && value.starts_with(SECRET_PREFIX)
        && value[SECRET_PREFIX.len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
}

pub fn hash(secret: &str) -> String {
    tokens::hash(secret)
}

fn generate() -> String {
    format!("{SECRET_PREFIX}{}", password::random(SECRET_LENGTH))
}

pub fn permits(link: &Link, portion: Portion, now: DateTime<Utc>) -> bool {
    if link.expires_at.with_timezone(&Utc) <= now {
        return false;
    }
    link.max_downloads.is_none_or(|maximum| {
        i128::from(link.bytes_served) + i128::from(portion.length)
            <= i128::from(maximum) * i128::from(portion.size)
    })
}

pub async fn create(
    db: &impl ConnectionTrait,
    new: NewLink,
) -> Result<(Link, String), ContentError> {
    let path = normalize(&new.path)?;
    if new.prefix != path.ends_with('/') {
        return Err(ContentError::BadRequest(if new.prefix {
            "the path of a prefix link must end with a slash".to_owned()
        } else {
            "the path of a single asset link must not end with a slash".to_owned()
        }));
    }
    if new.prefix && new.max_downloads.is_some() {
        return Err(ContentError::BadRequest(
            "a prefix link cannot have a download limit".to_owned(),
        ));
    }
    if new.max_downloads.is_some_and(|maximum| maximum < 1) {
        return Err(ContentError::BadRequest(
            "the download limit must be at least 1".to_owned(),
        ));
    }
    let now = Utc::now();
    if new.expires_at <= now {
        return Err(ContentError::BadRequest(
            "the link must expire in the future".to_owned(),
        ));
    }
    let secret = generate();
    let link = share_link::Entity::insert(share_link::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        secret_hash: ActiveValue::Set(hash(&secret)),
        repository: ActiveValue::Set(new.repository),
        path: ActiveValue::Set(path),
        prefix: ActiveValue::Set(new.prefix),
        note: ActiveValue::Set(new.note),
        created_by: ActiveValue::Set(new.created_by),
        created_at: ActiveValue::Set(now.fixed_offset()),
        expires_at: ActiveValue::Set(new.expires_at.fixed_offset()),
        max_downloads: ActiveValue::Set(new.max_downloads),
        downloads: ActiveValue::Set(0),
        bytes_served: ActiveValue::Set(0),
        last_used_at: ActiveValue::Set(None),
        last_used_ip: ActiveValue::Set(None),
    })
    .exec_with_returning(db)
    .await?;
    Ok((link, secret))
}

pub async fn by_secret(
    db: &impl ConnectionTrait,
    secret: &str,
) -> Result<Option<Link>, ContentError> {
    if !looks_like_secret(secret) {
        return Ok(None);
    }
    Ok(share_link::Entity::find()
        .filter(share_link::Column::SecretHash.eq(hash(secret)))
        .one(db)
        .await?)
}

pub async fn consume(
    db: &impl ConnectionTrait,
    id: Uuid,
    address: &str,
    portion: Portion,
) -> Result<bool, ContentError> {
    let address = Some(address).filter(|address| !address.is_empty());
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE share_link SET \
             downloads = downloads + CASE WHEN $3::boolean THEN 1 ELSE 0 END, \
             bytes_served = bytes_served + $2::bigint, \
             last_used_at = now(), last_used_ip = $5 \
             WHERE id = $1 AND expires_at > now() \
             AND (max_downloads IS NULL \
                  OR bytes_served::numeric + $2::bigint::numeric <= max_downloads::numeric * $4::bigint::numeric) \
             RETURNING id",
            [
                id.into(),
                portion.bytes().into(),
                portion.starts_download().into(),
                portion.total().into(),
                address.map(str::to_owned).into(),
            ],
        ))
        .await?;
    Ok(row.is_some())
}

pub async fn refund(
    db: &impl ConnectionTrait,
    id: Uuid,
    portion: Portion,
) -> Result<(), ContentError> {
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "UPDATE share_link SET \
         downloads = GREATEST(downloads - CASE WHEN $3::boolean THEN 1 ELSE 0 END, 0), \
         bytes_served = GREATEST(bytes_served - $2::bigint, 0) \
         WHERE id = $1",
        [
            id.into(),
            portion.bytes().into(),
            portion.starts_download().into(),
        ],
    ))
    .await?;
    Ok(())
}

pub async fn list(
    db: &impl ConnectionTrait,
    creator: Option<&str>,
    repository: Option<&str>,
) -> Result<Vec<Link>, ContentError> {
    let mut query = share_link::Entity::find();
    if let Some(creator) = creator {
        query = query.filter(share_link::Column::CreatedBy.eq(creator));
    }
    if let Some(repository) = repository {
        query = query.filter(share_link::Column::Repository.eq(repository));
    }
    Ok(query
        .order_by_desc(share_link::Column::CreatedAt)
        .order_by_asc(share_link::Column::Id)
        .all(db)
        .await?)
}

pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<bool, ContentError> {
    let result = share_link::Entity::delete_by_id(id).exec(db).await?;
    Ok(result.rows_affected > 0)
}

pub async fn delete_of_user(db: &impl ConnectionTrait, user: &str) -> Result<u64, ContentError> {
    let result = share_link::Entity::delete_many()
        .filter(share_link::Column::CreatedBy.eq(user))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn prune(db: &impl ConnectionTrait) -> Result<u64, ContentError> {
    let before = (Utc::now() - KEPT_AFTER_EXPIRY).fixed_offset();
    let result = share_link::Entity::delete_many()
        .filter(share_link::Column::ExpiresAt.lt(before))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(expires_in: i64, downloads: i32, bytes_served: i64, maximum: Option<i32>) -> Link {
        let now = Utc::now();
        Link {
            id: Uuid::nil(),
            secret_hash: String::new(),
            repository: "files".to_owned(),
            path: "/a".to_owned(),
            prefix: false,
            note: String::new(),
            created_by: "admin".to_owned(),
            created_at: now.fixed_offset(),
            expires_at: (now + Duration::seconds(expires_in)).fixed_offset(),
            max_downloads: maximum,
            downloads,
            bytes_served,
            last_used_at: None,
            last_used_ip: None,
        }
    }

    #[test]
    fn secrets_have_a_fixed_shape_and_their_hash_is_hex_sha256() {
        let first = generate();
        let second = generate();
        assert_ne!(first, second);
        assert!(looks_like_secret(&first), "{first}");
        assert!(!looks_like_secret(&first[..first.len() - 1]));
        assert!(!looks_like_secret(&format!("{first}x")));
        assert!(!looks_like_secret(
            &first.replace("astrum_share_", "astrum_")
        ));
        assert!(!looks_like_secret(&format!(
            "{}/",
            &first[..first.len() - 1]
        )));
        assert!(!tokens::looks_like_token(&first));
        assert_eq!(hash(&first).len(), 64);
        assert!(hash(&first).chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(hash(&first), first);
    }

    #[test]
    fn a_link_admits_by_expiry_and_byte_budget() {
        let now = Utc::now();
        let part = |offset, length| Portion {
            offset,
            length,
            size: 10,
        };
        assert!(permits(&link(60, 0, 0, None), Portion::whole(10), now));
        assert!(permits(&link(60, 9, 900, None), Portion::whole(10), now));
        assert!(!permits(&link(-1, 0, 0, None), Portion::whole(10), now));
        assert!(permits(&link(60, 0, 0, Some(1)), Portion::whole(10), now));
        assert!(!permits(&link(60, 1, 10, Some(1)), Portion::whole(10), now));
        assert!(permits(&link(60, 1, 1, Some(1)), part(1, 9), now));
        assert!(!permits(&link(60, 1, 10, Some(1)), part(1, 9), now));
        assert!(!permits(&link(60, 1, 2, Some(1)), part(1, 9), now));
        assert!(permits(&link(60, 1, 10, Some(2)), Portion::whole(10), now));
        assert!(!permits(&link(-1, 1, 1, Some(1)), part(1, 9), now));
    }
}
