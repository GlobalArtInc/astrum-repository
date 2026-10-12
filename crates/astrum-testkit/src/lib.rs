pub mod s3;

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use astrum_db::{DatabaseConfig, DatabaseUrl};
use sea_orm::{ConnectionTrait, Database};

pub const ADMIN_URL: &str = "ASTRUM_TEST_DATABASE_URL";

static NEXT: AtomicU32 = AtomicU32::new(0);

pub struct TestDatabase {
    admin: DatabaseUrl,
    url: DatabaseUrl,
}

impl TestDatabase {
    pub async fn create() -> Self {
        let admin = admin_url();
        let name = unique_name();
        let conn = Database::connect(admin.expose())
            .await
            .unwrap_or_else(|error| panic!("cannot reach the test server {admin}: {error}"));
        conn.execute_unprepared(&format!("CREATE DATABASE \"{name}\""))
            .await
            .unwrap_or_else(|error| panic!("cannot create the test database {name}: {error}"));
        let _ = conn.close().await;
        Self {
            url: admin.with_database(&name),
            admin,
        }
    }

    pub fn name(&self) -> &str {
        self.url.database()
    }

    pub fn url(&self) -> &DatabaseUrl {
        &self.url
    }

    pub fn admin_url(&self) -> &DatabaseUrl {
        &self.admin
    }

    pub fn config(&self) -> DatabaseConfig {
        DatabaseConfig::new(self.url.clone())
    }

    pub async fn connect(&self) -> astrum_db::Database {
        astrum_db::Database::connect(&self.config())
            .await
            .expect("the test database accepts connections")
    }

    pub async fn migrated(&self) -> astrum_db::Database {
        let db = self.connect().await;
        db.migrate()
            .await
            .expect("migrations apply to an empty database");
        db
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let admin = self.admin.clone();
        let name = self.name().to_owned();
        let dropped = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            runtime.block_on(async {
                let conn = Database::connect(admin.expose()).await.ok()?;
                let result = conn
                    .execute_unprepared(&format!("DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"))
                    .await;
                let _ = conn.close().await;
                result.ok()
            })
        })
        .join();
        if !matches!(dropped, Ok(Some(_))) {
            eprintln!("the test database {} was left behind", self.name());
        }
    }
}

fn admin_url() -> DatabaseUrl {
    let value = std::env::var(ADMIN_URL).unwrap_or_else(|_| {
        panic!(
            "set {ADMIN_URL} to a PostgreSQL address whose user may create databases, \
             e.g. postgres://postgres@127.0.0.1:5432/postgres"
        )
    });
    DatabaseUrl::parse(&value).unwrap_or_else(|error| panic!("{ADMIN_URL}: {error}"))
}

fn unique_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos())
        .unwrap_or_default();
    format!(
        "astrum_test_{}_{}_{nanos}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
