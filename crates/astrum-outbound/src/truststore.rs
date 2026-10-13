use astrum_db::entity::ssl_certificate;
use rustls::pki_types::CertificateDer;
use sea_orm::{ActiveValue, ConnectionTrait, DbErr, EntityTrait, QueryOrder, SqlErr};

use crate::certificate::{self, Certificate, CertificateError};

#[derive(Debug, thiserror::Error)]
pub enum TrustStoreError {
    #[error(transparent)]
    Certificate(#[from] CertificateError),
    #[error("the certificate {0} is already in the truststore")]
    Duplicate(String),
    #[error("the database refused a query: {0}")]
    Database(#[from] DbErr),
}

pub struct Stored {
    pub der: Vec<u8>,
    pub certificate: Certificate,
}

pub async fn list(db: &impl ConnectionTrait) -> Result<Vec<Stored>, DbErr> {
    let rows = ssl_certificate::Entity::find()
        .order_by_asc(ssl_certificate::Column::CreatedAt)
        .all(db)
        .await?;
    let mut stored = Vec::new();
    for row in rows {
        let parsed = certificate::der_from_pem(&row.pem)
            .and_then(|der| certificate::describe(&der).map(|described| (der, described)));
        match parsed {
            Ok((der, certificate)) => stored.push(Stored { der, certificate }),
            Err(error) => {
                tracing::warn!(id = row.id, %error, "a stored certificate cannot be read; it is skipped")
            }
        }
    }
    Ok(stored)
}

pub async fn roots(db: &impl ConnectionTrait) -> Result<Vec<CertificateDer<'static>>, DbErr> {
    Ok(list(db)
        .await?
        .into_iter()
        .map(|stored| CertificateDer::from(stored.der))
        .collect())
}

pub async fn add(db: &impl ConnectionTrait, pem: &str) -> Result<Certificate, TrustStoreError> {
    let der = certificate::der_from_pem(pem)?;
    let described = certificate::describe(&der)?;
    if ssl_certificate::Entity::find_by_id(described.id.clone())
        .one(db)
        .await?
        .is_some()
    {
        return Err(TrustStoreError::Duplicate(described.id));
    }
    let inserted = ssl_certificate::Entity::insert(ssl_certificate::ActiveModel {
        id: ActiveValue::Set(described.id.clone()),
        pem: ActiveValue::Set(described.pem.clone()),
        created_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
    })
    .exec(db)
    .await;
    match inserted {
        Ok(_) => Ok(described),
        Err(error) if matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => {
            Err(TrustStoreError::Duplicate(described.id))
        }
        Err(error) => Err(error.into()),
    }
}

pub async fn remove(db: &impl ConnectionTrait, id: &str) -> Result<Option<Certificate>, DbErr> {
    let Some(row) = ssl_certificate::Entity::find_by_id(normalize_id(id))
        .one(db)
        .await?
    else {
        return Ok(None);
    };
    ssl_certificate::Entity::delete_by_id(row.id.clone())
        .exec(db)
        .await?;
    Ok(certificate::der_from_pem(&row.pem)
        .and_then(|der| certificate::describe(&der))
        .ok())
}

pub fn normalize_id(id: &str) -> String {
    let hex: String = id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    hex.as_bytes()
        .chunks(2)
        .map(|pair| String::from_utf8_lossy(pair).into_owned())
        .collect::<Vec<_>>()
        .join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_accepted_with_or_without_colons() {
        assert_eq!(normalize_id("c83e08"), "C8:3E:08");
        assert_eq!(normalize_id("C8:3E:08"), "C8:3E:08");
    }
}
