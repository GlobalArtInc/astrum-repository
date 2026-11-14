use async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, ExecResult, QueryResult, Statement,
};
use tokio::sync::{Semaphore, SemaphorePermit};

pub(crate) struct Gated<'a> {
    db: &'a DatabaseConnection,
    gate: &'a Semaphore,
}

impl<'a> Gated<'a> {
    pub(crate) fn new(db: &'a DatabaseConnection, gate: &'a Semaphore) -> Self {
        Self { db, gate }
    }

    async fn permit(&self) -> Result<SemaphorePermit<'a>, DbErr> {
        self.gate
            .acquire()
            .await
            .map_err(|_| DbErr::Custom("the replication database gate is closed".to_owned()))
    }
}

#[async_trait]
impl ConnectionTrait for Gated<'_> {
    fn get_database_backend(&self) -> DbBackend {
        self.db.get_database_backend()
    }

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        let _permit = self.permit().await?;
        self.db.execute(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        let _permit = self.permit().await?;
        self.db.execute_unprepared(sql).await
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        let _permit = self.permit().await?;
        self.db.query_one(stmt).await
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        let _permit = self.permit().await?;
        self.db.query_all(stmt).await
    }
}
