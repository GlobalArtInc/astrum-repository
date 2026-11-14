#![allow(dead_code)]

use axum::http::Method;
use serde_json::{Value, json};

use crate::cluster::{Node, connect, replicate};
use crate::support::{self, Auth};

pub const REPLICATIONS: &str = "/service/rest/v1/replications";
pub const UNKNOWN: &str = "00000000-0000-0000-0000-000000000001";

pub struct Setup {
    pub sender: Node,
    pub _receiver: Node,
    pub id: String,
}

impl Setup {
    pub async fn new() -> Self {
        let sender = Node::start(&["source"]).await;
        let receiver = Node::start(&["mirror"]).await;
        let remote = connect(&sender, &receiver).await;
        let id = replicate(&sender, &remote, "source", "mirror", json!({})).await;
        Self {
            sender,
            _receiver: receiver,
            id,
        }
    }

    pub fn path(&self, tail: &str) -> String {
        format!("{REPLICATIONS}/{}{tail}", self.id)
    }

    pub async fn get(&self, tail: &str) -> support::Reply {
        self.call(Method::GET, tail, Auth::Admin, None).await
    }

    pub async fn post(&self, tail: &str, body: Value) -> support::Reply {
        self.call(Method::POST, tail, Auth::Admin, Some(body)).await
    }

    pub async fn call(
        &self,
        method: Method,
        tail: &str,
        auth: Auth<'_>,
        body: Option<Value>,
    ) -> support::Reply {
        self.sender
            .fixture
            .call(method, &self.path(tail), auth, body)
            .await
    }

    pub async fn item(&self, state: &str, name: &str, age_seconds: i64) -> i64 {
        let sql = format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, state, \
             reason, attempts, last_error, first_queued_at, leased_until) \
             VALUES ('{}', 'ns', '{name}', '1.0', '{state}', 'event', 4, 'broken {name}', \
             now() - interval '{age_seconds} seconds', \
             CASE WHEN '{state}' = 'leased' THEN now() + interval '10 minutes' END) RETURNING id AS n",
            self.id
        );
        let db = self.sender.fixture.state.db().conn();
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()
            .unwrap();
        row.try_get("", "n").unwrap()
    }

    pub async fn states(&self) -> Vec<(i64, String, i32, String)> {
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let sql = "SELECT id, state, attempts, reason FROM replication_item ORDER BY id";
        let rows = self
            .sender
            .fixture
            .state
            .db()
            .conn()
            .query_all(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
        rows.iter()
            .map(|row| {
                (
                    row.try_get("", "id").unwrap(),
                    row.try_get("", "state").unwrap(),
                    row.try_get("", "attempts").unwrap(),
                    row.try_get("", "reason").unwrap(),
                )
            })
            .collect()
    }
}
