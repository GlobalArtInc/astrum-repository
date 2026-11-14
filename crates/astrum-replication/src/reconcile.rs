use astrum_content::Content;
use astrum_tasks::{Field, FieldType, Problem, Properties, Run, TaskError, TaskKind};
use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

use crate::dispatch::{Dispatcher, Pass};
use crate::error::{Error, clean};
use crate::queue;
use crate::scan::{Filters, Scan};
use crate::target::Target;
use crate::work::{Failure, Phase};

pub use crate::scan::{Limits, Tally};

pub const ID: &str = "replication.reconcile";
pub const PROPERTY: &str = "replicationId";
const DISABLED: &str = "the replication is disabled or gone";

#[derive(Debug)]
pub enum Ended {
    Stopped,
    Failed(String),
    Down(Error),
}

impl Ended {
    pub fn text(&self) -> String {
        match self {
            Ended::Stopped => "the run was stopped".to_owned(),
            Ended::Failed(text) => clean(text),
            Ended::Down(error) => clean(&error.to_string()),
        }
    }
}

impl From<Failure> for Ended {
    fn from(failure: Failure) -> Self {
        match failure {
            Failure::Down(error) => Ended::Down(error),
            other => Ended::Failed(other.text()),
        }
    }
}

impl From<DbErr> for Ended {
    fn from(error: DbErr) -> Self {
        Ended::Failed(clean(&error.to_string()))
    }
}

impl From<Error> for Ended {
    fn from(error: Error) -> Self {
        Ended::from(Failure::blame(Phase::Negotiate, error))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Complete,
    Capped,
    Skipped(String),
    Unreadable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub outcome: Outcome,
    pub tally: Tally,
}

impl Summary {
    pub fn message(&self) -> String {
        let tally = &self.tally;
        match &self.outcome {
            Outcome::Skipped(reason) => format!("skipped: {reason}"),
            Outcome::Unreadable(text) => text.clone(),
            outcome => {
                let tail = match (outcome, tally.deletes_capped) {
                    (Outcome::Capped, true) => {
                        "; the limit of deletions one run queues was reached, the next run continues"
                    }
                    (Outcome::Capped, false) => {
                        "; the limit of one run was reached, run the reconcile again"
                    }
                    _ => "",
                };
                let held = match tally.withheld {
                    0 => String::new(),
                    count => format!(
                        "; the remote lists no components, so {count} local components from it were not queued for deletion"
                    ),
                };
                let here = match tally.from_here {
                    0 => String::new(),
                    count => format!(", {count} from this server"),
                };
                format!(
                    "{} local components looked at ({} outside the filters, {} from the remote, \
                     {} without assets{here}), {} remote entries looked at, {} queued{held}{tail}",
                    tally.scanned,
                    tally.filtered,
                    tally.from_remote,
                    tally.empty,
                    tally.remote_scanned,
                    tally.enqueued,
                )
            }
        }
    }
}

async fn settle(dispatcher: &Dispatcher, target: &Target, ended: Ended) -> Ended {
    let Ended::Down(error) = &ended else {
        return ended;
    };
    match dispatcher.pause(target, error, &mut Pass::default()).await {
        Ok(_) => ended,
        Err(error) => Ended::from(error),
    }
}

pub async fn run(content: &Content, replication: Uuid) -> Result<Summary, Ended> {
    run_with(content, replication, &Limits::default(), &|| false).await
}

pub async fn run_with(
    content: &Content,
    replication: Uuid,
    limits: &Limits,
    stop: &(dyn Fn() -> bool + Send + Sync),
) -> Result<Summary, Ended> {
    let db = content.db();
    let Some(replication) = queue::replication(db, replication)
        .await?
        .filter(|found| found.enabled)
    else {
        return Ok(Summary {
            outcome: Outcome::Skipped(DISABLED.to_owned()),
            tally: Tally::default(),
        });
    };
    let remote = queue::remote(db, replication.remote_id)
        .await?
        .ok_or_else(|| Ended::Failed("the remote of the replication is gone".to_owned()))?;
    if let Some(seconds) = remote.suspended_for {
        let wait = chrono::Duration::milliseconds((seconds.max(0.0) * 1000.0) as i64);
        let until = (chrono::Utc::now() + wait).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        return Ok(Summary {
            outcome: Outcome::Skipped(format!("remote suspended until {until}")),
            tally: Tally::default(),
        });
    }
    let filters = Filters::new(&replication).map_err(Ended::Failed)?;
    let dispatcher = Dispatcher::new(content.clone());
    let target = dispatcher.target(remote).await;
    let client = target.client()?;
    let remote_instance = match dispatcher.remote_instance(&target, client).await {
        Ok(instance) => instance,
        Err(failure) => return Err(settle(&dispatcher, &target, failure.into()).await),
    };
    let own = dispatcher.instance().await?;
    let ctx = dispatcher.context(&replication.source_repository).await?;
    let format = content
        .format(ctx.format_id())
        .ok_or_else(|| Ended::Failed(format!("the format {} is not served", ctx.format_id())))?;
    let transfer = format.transfer().ok_or_else(|| {
        Ended::Failed(format!(
            "the format {} takes no part in replication",
            ctx.format_id()
        ))
    })?;
    let mut scan = Scan {
        db,
        client,
        replication: &replication,
        filters,
        remote: remote_instance,
        own,
        ctx: &ctx,
        transfer,
        limits,
        stop,
        tally: Tally::default(),
    };
    let scanned = async {
        match replication.pulls() {
            true => scan.pull().await?,
            false => {
                scan.local().await?;
                if replication.propagate_deletes && !scan.tally.capped {
                    scan.remote().await?;
                }
            }
        }
        Ok::<(), Ended>(())
    }
    .await;
    if let Err(ended) = scanned {
        return Err(settle(&dispatcher, &target, ended).await);
    }
    let tally = scan.tally;
    let outcome = match (&tally.first_unreadable, tally.capped) {
        (_, true) => Outcome::Capped,
        (Some(first), false) => Outcome::Unreadable(clean(&format!(
            "{} components could not be read, the first is {first}",
            tally.unreadable
        ))),
        (None, false) => {
            queue::reconciled(db, replication.id).await?;
            Outcome::Complete
        }
    };
    Ok(Summary { outcome, tally })
}

pub struct Reconcile;

#[async_trait]
impl TaskKind for Reconcile {
    fn id(&self) -> &'static str {
        ID
    }

    fn name(&self) -> &'static str {
        "Replication - Reconcile a replication with its remote"
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field::new(PROPERTY, FieldType::Text, "Replication id")
                .help("The id of the replication whose components are compared with the remote")
                .required(),
        ]
    }

    fn system(&self) -> bool {
        true
    }

    async fn validate(
        &self,
        content: &Content,
        stored: Option<&Properties>,
        given: &Properties,
    ) -> Result<(), Problem> {
        let id = format!("properties.{PROPERTY}");
        let problem = |message: &str| Problem {
            id: id.clone(),
            message: message.to_owned(),
        };
        let named = given
            .get(PROPERTY)
            .and_then(|text| Uuid::parse_str(text).ok())
            .ok_or_else(|| problem("must be the id of a replication"))?;
        if stored.is_some_and(|stored| stored.get(PROPERTY) != given.get(PROPERTY)) {
            return Err(problem(
                "cannot change; the task belongs to one replication",
            ));
        }
        match queue::replication(content.db(), named).await {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(problem("there is no replication with this id")),
            Err(error) => Err(problem(&clean(&error.to_string()))),
        }
    }

    fn describe(&self, properties: &Properties) -> String {
        match properties.get(PROPERTY) {
            Some(id) => {
                format!("compares the replication {id} with its remote and queues what differs")
            }
            None => "compares a replication with its remote".to_owned(),
        }
    }

    async fn run(&self, run: &Run) -> Result<String, TaskError> {
        let id = run
            .property(PROPERTY)
            .and_then(|text| Uuid::parse_str(text).ok())
            .ok_or_else(|| TaskError::Failed("the task names no replication".to_owned()))?;
        let summary = run_with(run.content(), id, &Limits::default(), &|| run.cancelled())
            .await
            .map_err(|ended| match ended {
                Ended::Stopped => TaskError::Cancelled,
                other => TaskError::Failed(other.text()),
            })?;
        run.count(
            "enqueued",
            i64::try_from(summary.tally.enqueued).unwrap_or(i64::MAX),
        );
        let message = summary.message();
        match summary.outcome {
            Outcome::Unreadable(_) => Err(TaskError::Failed(message)),
            Outcome::Capped => {
                run.warn(message.clone());
                Ok(message)
            }
            _ => Ok(message),
        }
    }
}
