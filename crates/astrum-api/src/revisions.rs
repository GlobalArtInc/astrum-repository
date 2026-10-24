use std::time::Duration;

use astrum_db::revision;
use tokio::task::JoinHandle;

use crate::AppState;

pub const INTERVAL: Duration = Duration::from_secs(2);

pub async fn catch_up(state: &AppState) -> Result<(), astrum_security::Error> {
    let revisions = revision::all(state.db().conn()).await?;
    if state
        .security()
        .catch_up(revisions.of(revision::SECURITY))
        .await?
    {
        tracing::debug!("loaded users and security settings changed on another server");
    }
    if state
        .blob_stores()
        .catch_up(revisions.of(revision::BLOB_STORES))
    {
        tracing::debug!("reopening blob stores changed on another server");
    }
    Ok(())
}

pub fn spawn(state: AppState, every: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut failing = false;
        loop {
            tokio::time::sleep(every).await;
            match catch_up(&state).await {
                Ok(()) if failing => {
                    tracing::info!("changes made on other servers are followed again");
                    failing = false;
                }
                Ok(()) => {}
                Err(error) if failing => {
                    tracing::debug!(%error, "changes made on other servers could not be checked");
                }
                Err(error) => {
                    tracing::warn!(%error, "changes made on other servers could not be checked; trying again");
                    failing = true;
                }
            }
        }
    })
}
