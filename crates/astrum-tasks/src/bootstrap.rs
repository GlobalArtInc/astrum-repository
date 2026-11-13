use astrum_db::setting;
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;

use crate::kind::Properties;
use crate::schedule::Schedule;
use crate::store::{self, NewTask, StoreError};

pub const PROVISIONED: &str = "tasks.provisioned";
pub const USAGE_SNAPSHOT_PROVISIONED: &str = "tasks.provisioned.usageSnapshot";
pub const DOCKER_GC: &str = "repository.docker.gc";

fn properties(pairs: &[(&str, &str)]) -> Properties {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

pub fn builtin() -> Vec<NewTask> {
    let cron = |expression: &str| Schedule::cron(expression).expect("built-in schedules are valid");
    let task = |kind: &str, name: &str, schedule: Schedule, pairs: &[(&str, &str)]| NewTask {
        kind: kind.to_owned(),
        name: name.to_owned(),
        enabled: true,
        schedule,
        alert_email: None,
        notification_condition: crate::notification::FAILURE.to_owned(),
        properties: properties(pairs),
    };
    let soon = Utc::now() + Duration::minutes(5);
    vec![
        task(
            crate::cleanup::task::ID,
            "Cleanup service",
            cron("0 0 1 * * ?"),
            &[],
        ),
        task(
            DOCKER_GC,
            "Delete unused Docker manifests and layers",
            cron("0 30 1 * * ?"),
            &[("repositoryName", "*"), ("deployOffset", "24")],
        ),
        task(
            crate::kinds::compact::ID,
            "Compact blob stores",
            cron("0 0 2 * * ?"),
            &[("blobstoreName", "*"), ("blobsOlderThan", "1")],
        ),
        task(
            crate::kinds::expired::ID,
            "Delete expired sessions and uploads",
            Schedule::hourly(soon),
            &[],
        ),
    ]
}

fn usage_snapshot() -> NewTask {
    NewTask {
        kind: crate::kinds::usage_snapshot::ID.to_owned(),
        name: "Record repository usage".to_owned(),
        enabled: true,
        schedule: Schedule::cron("0 30 0 * * ?").expect("built-in schedules are valid"),
        alert_email: None,
        notification_condition: crate::notification::FAILURE.to_owned(),
        properties: Properties::new(),
    }
}

async fn create(db: &DatabaseConnection, tasks: Vec<NewTask>) -> Result<usize, astrum_db::Error> {
    let mut created = 0;
    for task in tasks {
        let name = task.name.clone();
        match store::create(db, task).await {
            Ok(_) => created += 1,
            Err(StoreError::Duplicate(_)) => {}
            Err(StoreError::Database(error)) => return Err(error.into()),
        }
        tracing::info!(task = %name, "created a built-in task");
    }
    Ok(created)
}

pub async fn bootstrap(db: &DatabaseConnection) -> Result<usize, astrum_db::Error> {
    let mut created = 0;
    if setting::get::<bool>(db, PROVISIONED).await?.is_none() {
        created += create(db, builtin()).await?;
        setting::put(db, PROVISIONED, &true).await?;
    }
    if setting::get::<bool>(db, USAGE_SNAPSHOT_PROVISIONED)
        .await?
        .is_none()
    {
        created += create(db, vec![usage_snapshot()]).await?;
        setting::put(db, USAGE_SNAPSHOT_PROVISIONED, &true).await?;
    }
    Ok(created)
}
