use astrum_db::setting;
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;

use crate::kind::Properties;
use crate::schedule::Schedule;
use crate::store::{self, NewTask, StoreError};

pub const PROVISIONED: &str = "tasks.provisioned";
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
        notification_condition: "FAILURE".to_owned(),
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

pub async fn bootstrap(db: &DatabaseConnection) -> Result<usize, astrum_db::Error> {
    if setting::get::<bool>(db, PROVISIONED).await?.is_some() {
        return Ok(0);
    }
    let mut created = 0;
    for task in builtin() {
        let name = task.name.clone();
        match store::create(db, task).await {
            Ok(_) => created += 1,
            Err(StoreError::Duplicate(_)) => {}
            Err(StoreError::Database(error)) => return Err(error.into()),
        }
        tracing::info!(task = %name, "created a built-in task");
    }
    setting::put(db, PROVISIONED, &true).await?;
    Ok(created)
}
