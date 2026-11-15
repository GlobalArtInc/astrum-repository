use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} exists already")]
    Duplicate(String),
    #[error("{0} does not exist")]
    NotFound(String),
    #[error("{0}")]
    Invalid(String),
    #[error("\"{name}\" holds a value this version cannot read: {reason}")]
    Stored { name: String, reason: String },
    #[error("the database refused a query: {0}")]
    Database(#[from] DbErr),
}
