pub mod audit;
mod config;
pub mod entity;
pub mod read_only;
pub mod setting;
pub mod system;

use std::time::Duration;

use sea_orm::sqlx::pool::PoolConnection;
use sea_orm::sqlx::{self, Executor, Postgres};
use sea_orm::{
    ConnectOptions, ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement,
    TransactionTrait,
};

pub use config::{
    ConfigError, DEFAULT_MAX_CONNECTIONS, DEFAULT_SCHEMA, DatabaseConfig, DatabaseUrl,
    MIN_CONNECTIONS, check_pool, check_schema,
};
pub use migrator;

const TIMEOUT: Duration = Duration::from_secs(10);
const SCHEMA_LOCK: i64 = i64::from_be_bytes(*b"astrschm");
const MIGRATION_LOCK: i64 = i64::from_be_bytes(*b"astrmigr");
const LEDGER_LOCK: i64 = i64::from_be_bytes(*b"astrledg");

pub const LEDGER: &str = "migration";

pub static MIGRATIONS: migrator::Migrations = migrator::embed!("$CARGO_MANIFEST_DIR/migrations")
    .table(LEDGER)
    .lock_key(LEDGER_LOCK);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not connect to {database}: {source}")]
    Connect {
        database: String,
        #[source]
        source: DbErr,
    },
    #[error("could not prepare the schema \"{schema}\": {source}")]
    Schema {
        schema: String,
        #[source]
        source: DbErr,
    },
    #[error("could not take the migration lock: {0}")]
    Lock(#[source] sqlx::Error),
    #[error("migrations failed: {0}")]
    Migration(#[from] migrator::Error),
    #[error("the database refused a query: {0}")]
    Query(#[from] DbErr),
    #[error("the setting \"{key}\" holds a value this version cannot read or write: {source}")]
    Setting {
        key: String,
        #[source]
        source: serde_json::Error,
    },
}

#[derive(Clone, Debug)]
pub struct Database {
    conn: DatabaseConnection,
    schema: String,
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, Error> {
        let mut options = ConnectOptions::new(config.url.expose());
        options
            .max_connections(config.max_connections)
            .min_connections(0)
            .connect_timeout(TIMEOUT)
            .acquire_timeout(TIMEOUT)
            .set_schema_search_path(config.schema.clone())
            .sqlx_logging(false);
        let conn = sea_orm::Database::connect(options)
            .await
            .map_err(|source| Error::Connect {
                database: config.url.to_string(),
                source,
            })?;
        prepare_schema(&conn, &config.schema).await?;
        tracing::info!(database = %config.url, schema = %config.schema, "database connected");
        Ok(Self {
            conn,
            schema: config.schema.clone(),
        })
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub async fn migrate(&self) -> Result<Vec<String>, Error> {
        let lock = SessionLock::take(&self.conn, MIGRATION_LOCK).await?;
        let applied = MIGRATIONS.apply(&self.conn).await;
        lock.release().await;
        Ok(applied?)
    }

    pub async fn ping(&self) -> Result<(), Error> {
        Ok(self.conn.ping().await?)
    }

    pub async fn accepts_writes(&self) -> Result<bool, Error> {
        let row = self
            .conn
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                "SELECT pg_is_in_recovery() OR current_setting('transaction_read_only') = 'on' AS read_only",
            ))
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("read_only".to_owned()))?;
        let read_only: bool = row.try_get("", "read_only")?;
        Ok(!read_only)
    }

    pub async fn close(self) -> Result<(), Error> {
        Ok(self.conn.close().await?)
    }
}

struct SessionLock {
    connection: PoolConnection<Postgres>,
    key: i64,
}

impl SessionLock {
    async fn take(db: &DatabaseConnection, key: i64) -> Result<Self, Error> {
        let mut connection = db
            .get_postgres_connection_pool()
            .acquire()
            .await
            .map_err(Error::Lock)?;
        connection.close_on_drop();
        connection
            .execute(format!("SELECT pg_advisory_lock({key})").as_str())
            .await
            .map_err(Error::Lock)?;
        Ok(Self { connection, key })
    }

    async fn release(mut self) {
        let release = format!("SELECT pg_advisory_unlock({})", self.key);
        let _ = self.connection.execute(release.as_str()).await;
    }
}

async fn prepare_schema(conn: &DatabaseConnection, schema: &str) -> Result<(), Error> {
    let failed = |source| Error::Schema {
        schema: schema.to_owned(),
        source,
    };
    check_schema(schema).map_err(|error| failed(DbErr::Custom(error.to_string())))?;
    let present = conn
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT 1 AS present FROM pg_namespace WHERE nspname = $1",
            [schema.into()],
        ))
        .await
        .map_err(failed)?;
    if present.is_some() {
        return Ok(());
    }
    let txn = conn.begin().await.map_err(failed)?;
    txn.execute_unprepared(&format!("SELECT pg_advisory_xact_lock({SCHEMA_LOCK})"))
        .await
        .map_err(failed)?;
    txn.execute_unprepared(&format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\""))
        .await
        .map_err(failed)?;
    txn.commit().await.map_err(failed)?;
    tracing::info!(schema, "schema created");
    Ok(())
}
