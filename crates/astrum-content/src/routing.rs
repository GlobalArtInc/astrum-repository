use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;
use astrum_repo::routing::{self, Matcher};
use sea_orm::DatabaseConnection;

use crate::ContentError;

const REFRESH: Duration = Duration::from_secs(10);

pub const BLOCKED: &str = "Routing rules block the requested item from this repository";

type Matchers = Arc<HashMap<String, Matcher>>;

#[derive(Default)]
pub(crate) struct RoutingCache {
    rules: Mutex<Option<(Instant, Matchers)>>,
}

impl RoutingCache {
    async fn matchers(&self, db: &DatabaseConnection) -> Result<Matchers, ContentError> {
        if let Some((at, matchers)) = self
            .rules
            .lock()
            .expect("the routing cache lock is never poisoned")
            .clone()
            && at.elapsed() < REFRESH
        {
            return Ok(matchers);
        }
        let matchers = Arc::new(
            routing::matchers(db)
                .await
                .map_err(ContentError::internal)?,
        );
        *self
            .rules
            .lock()
            .expect("the routing cache lock is never poisoned") =
            Some((Instant::now(), matchers.clone()));
        Ok(matchers)
    }

    pub(crate) fn forget(&self) {
        *self
            .rules
            .lock()
            .expect("the routing cache lock is never poisoned") = None;
    }

    pub(crate) async fn allows(
        &self,
        db: &DatabaseConnection,
        repository: &Repository,
        path: &str,
    ) -> Result<bool, ContentError> {
        if repository.kind == RepositoryType::Hosted {
            return Ok(true);
        }
        let Some(rule) = repository.attributes.routing_rule_name.as_deref() else {
            return Ok(true);
        };
        let matchers = self.matchers(db).await?;
        Ok(matchers
            .get(rule)
            .is_none_or(|matcher| matcher.allows(path)))
    }
}
