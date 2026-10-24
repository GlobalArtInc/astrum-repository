use astrum_db::setting;
use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use astrum_mail::Message;
use chrono::{DateTime, Utc};
use sea_orm::ConnectionTrait;
use uuid::Uuid;

use crate::Tasks;
use crate::scheduler::describe;
use crate::store::{self, RunState, Task, TaskRun};

pub const FAILURE: &str = "FAILURE";
pub const SUCCESS_FAILURE: &str = "SUCCESS_FAILURE";
pub const CONDITIONS: [&str; 2] = [FAILURE, SUCCESS_FAILURE];

pub fn wanted(condition: &str, state: RunState) -> bool {
    match state {
        RunState::Failed => true,
        RunState::Done | RunState::Cancelled => condition == SUCCESS_FAILURE,
        RunState::Waiting | RunState::Running => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub to: String,
    pub condition: String,
    pub task_id: Uuid,
    pub task_name: String,
    pub task_type: String,
    pub type_name: String,
    pub state: RunState,
    pub trigger: String,
    pub initiator: String,
    pub node: String,
    pub message: String,
    pub started: Option<DateTime<Utc>>,
    pub finished: Option<DateTime<Utc>>,
}

fn time(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

impl Report {
    pub fn of(task: &Task, run: &TaskRun, type_name: &str) -> Option<Self> {
        let to = task
            .alert_email
            .as_deref()
            .map(str::trim)
            .filter(|to| !to.is_empty())?;
        let state = RunState::parse(&run.state)?;
        if !wanted(&task.notification_condition, state) {
            return None;
        }
        Some(Self {
            to: to.to_owned(),
            condition: task.notification_condition.clone(),
            task_id: task.id,
            task_name: task.name.clone(),
            task_type: task.kind.clone(),
            type_name: type_name.to_owned(),
            state,
            trigger: run.trigger.clone(),
            initiator: run.initiator.clone(),
            node: run.node.clone(),
            message: run.message.clone(),
            started: run.started_at.map(|at| at.with_timezone(&Utc)),
            finished: run.finished_at.map(|at| at.with_timezone(&Utc)),
        })
    }

    pub fn subject(&self) -> String {
        let outcome = match self.state {
            RunState::Done => "Task finished",
            RunState::Cancelled => "Task canceled",
            _ => "Task failed",
        };
        format!("{outcome}: {}", self.task_name)
    }

    pub fn text(&self, base_url: Option<&str>) -> String {
        let (verb, result, detail) = match self.state {
            RunState::Done => ("finished", "OK", "Summary"),
            RunState::Cancelled => ("was stopped", "Canceled", "Summary"),
            _ => ("failed", "Failed", "Error"),
        };
        let mut lines = vec![
            ("Task", self.task_name.clone()),
            (
                "Type",
                match self.type_name == self.task_type {
                    true => self.task_type.clone(),
                    false => format!("{} ({})", self.type_name, self.task_type),
                },
            ),
            ("Result", result.to_owned()),
        ];
        if let Some(started) = self.started {
            lines.push(("Started", time(started)));
        }
        if let Some(finished) = self.finished {
            let finished = match self.started {
                Some(started) => {
                    format!("{}, after {}", time(finished), describe(finished - started))
                }
                None => time(finished),
            };
            lines.push(("Finished", finished));
        }
        let by = match (self.trigger.as_str(), self.initiator.trim()) {
            ("schedule", _) | (_, "") => "the schedule".to_owned(),
            (_, initiator) => initiator.to_owned(),
        };
        lines.push(("Started by", by));
        if !self.node.trim().is_empty() {
            lines.push(("Server", self.node.clone()));
        }
        if !self.message.trim().is_empty() {
            lines.push((detail, self.message.clone()));
        }
        let mut text = format!("The task \"{}\" {verb}.\n\n", self.task_name);
        for (label, value) in lines {
            text.push_str(&format!("{label}: {value}\n"));
        }
        if let Some(base) = base_url
            .map(|base| base.trim().trim_end_matches('/'))
            .filter(|base| !base.is_empty())
        {
            text.push_str(&format!(
                "\nOpen the task: {base}/admin/system/tasks/{}\n",
                self.task_id
            ));
        }
        let when = match self.condition.as_str() {
            SUCCESS_FAILURE => "after every run",
            _ => "when a run fails",
        };
        text.push_str(&format!(
            "\nThis message was sent by Astrum Repository because the task notifies {} {when}.\n",
            self.to
        ));
        text
    }

    pub fn message(&self, base_url: Option<&str>) -> Message {
        Message {
            to: vec![self.to.clone()],
            subject: self.subject(),
            text: self.text(base_url),
        }
    }
}

async fn base_url(db: &impl ConnectionTrait) -> Option<String> {
    match setting::get::<BaseUrl>(db, BASE_URL_KEY).await {
        Ok(Some(base)) if base.enabled && !base.url.trim().is_empty() => {
            Some(base.url.trim().trim_end_matches('/').to_owned())
        }
        Ok(_) => None,
        Err(error) => {
            tracing::warn!(%error, "the base URL could not be read");
            None
        }
    }
}

impl Tasks {
    pub(crate) fn notify(&self, run: TaskRun) {
        let tasks = self.clone();
        tokio::spawn(async move { tasks.send_notification(run).await });
    }

    async fn send_notification(&self, run: TaskRun) {
        let task = match store::find(self.db(), run.task_id).await {
            Ok(Some(task)) => task,
            Ok(None) => return,
            Err(error) => {
                tracing::warn!(%error, run = run.id, "the notification of a task run was not sent");
                return;
            }
        };
        let type_name = self
            .kind(&task.kind)
            .map_or_else(|| task.kind.clone(), |kind| kind.name().to_owned());
        let Some(report) = Report::of(&task, &run, &type_name) else {
            return;
        };
        let message = report.message(base_url(self.db()).await.as_deref());
        match self.mailer().send(&message).await {
            Ok(()) => {
                tracing::info!(task = %task.name, run = run.id, "the notification of a task run was sent")
            }
            Err(error) => tracing::warn!(
                %error,
                task = %task.name,
                run = run.id,
                "the notification of a task run was not sent"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    const ID: &str = "0b6a3f5e-8f0e-4a43-9d36-5c1f0e3a7b21";

    fn task(email: Option<&str>, condition: &str) -> Task {
        let at = Utc
            .with_ymd_and_hms(2026, 10, 1, 0, 0, 0)
            .unwrap()
            .fixed_offset();
        Task {
            id: Uuid::parse_str(ID).unwrap(),
            kind: "blobstore.compact".to_owned(),
            name: "Nightly compaction".to_owned(),
            enabled: true,
            frequency: json!({"schedule": "manual"}),
            alert_email: email.map(str::to_owned),
            notification_condition: condition.to_owned(),
            properties: json!({}),
            next_run_at: None,
            created_at: at,
            updated_at: at,
        }
    }

    fn run(state: RunState, trigger: &str, initiator: &str, message: &str) -> TaskRun {
        let started = Utc.with_ymd_and_hms(2026, 10, 24, 1, 0, 0).unwrap();
        TaskRun {
            id: 7,
            task_id: Uuid::parse_str(ID).unwrap(),
            state: state.as_str().to_owned(),
            trigger: trigger.to_owned(),
            initiator: initiator.to_owned(),
            node: "astrum-1:42".to_owned(),
            message: message.to_owned(),
            counts: json!({}),
            cancel_requested: false,
            queued_at: started.fixed_offset(),
            started_at: Some(started.fixed_offset()),
            finished_at: Some((started + chrono::Duration::milliseconds(5300)).fixed_offset()),
        }
    }

    #[test]
    fn each_condition_picks_its_outcomes() {
        let matrix = [
            (FAILURE, RunState::Failed, true),
            (FAILURE, RunState::Done, false),
            (FAILURE, RunState::Cancelled, false),
            (SUCCESS_FAILURE, RunState::Failed, true),
            (SUCCESS_FAILURE, RunState::Done, true),
            (SUCCESS_FAILURE, RunState::Cancelled, true),
        ];
        for (condition, state, expected) in matrix {
            assert_eq!(wanted(condition, state), expected, "{condition} {state:?}");
            let report = Report::of(
                &task(Some("ops@example.org"), condition),
                &run(state, "schedule", "*TASK", "done"),
                "Admin - Compact blob store",
            );
            assert_eq!(report.is_some(), expected, "{condition} {state:?}");
        }
        for condition in CONDITIONS {
            for state in [RunState::Waiting, RunState::Running] {
                assert!(!wanted(condition, state), "{condition} {state:?}");
            }
        }
        assert!(wanted("UNKNOWN", RunState::Failed));
        assert!(!wanted("UNKNOWN", RunState::Done));
    }

    #[test]
    fn a_task_without_an_address_sends_nothing() {
        for email in [None, Some(""), Some("  ")] {
            let report = Report::of(
                &task(email, SUCCESS_FAILURE),
                &run(RunState::Failed, "schedule", "*TASK", "broken"),
                "Admin - Compact blob store",
            );
            assert_eq!(report, None, "{email:?}");
        }
    }

    #[test]
    fn a_failure_says_what_failed_where_and_links_to_the_task() {
        let report = Report::of(
            &task(Some(" ops@example.org "), FAILURE),
            &run(
                RunState::Failed,
                "schedule",
                "*TASK",
                "the blob store \"gone\" does not exist",
            ),
            "Admin - Compact blob store",
        )
        .unwrap();
        let message = report.message(Some("https://repo.example.com/"));
        assert_eq!(message.to, ["ops@example.org"]);
        assert_eq!(message.subject, "Task failed: Nightly compaction");
        assert_eq!(
            message.text,
            "The task \"Nightly compaction\" failed.\n\
             \n\
             Task: Nightly compaction\n\
             Type: Admin - Compact blob store (blobstore.compact)\n\
             Result: Failed\n\
             Started: 2026-10-24 01:00:00 UTC\n\
             Finished: 2026-10-24 01:00:05 UTC, after 5.3 s\n\
             Started by: the schedule\n\
             Server: astrum-1:42\n\
             Error: the blob store \"gone\" does not exist\n\
             \n\
             Open the task: https://repo.example.com/admin/system/tasks/0b6a3f5e-8f0e-4a43-9d36-5c1f0e3a7b21\n\
             \n\
             This message was sent by Astrum Repository because the task notifies ops@example.org when a run fails.\n"
        );
    }

    #[test]
    fn a_manual_success_names_who_started_it_and_has_no_link_without_a_base_url() {
        let report = Report::of(
            &task(Some("ops@example.org"), SUCCESS_FAILURE),
            &run(RunState::Done, "manual", "admin", "compacted 3 blobs"),
            "blobstore.compact",
        )
        .unwrap();
        let message = report.message(None);
        assert_eq!(message.subject, "Task finished: Nightly compaction");
        assert!(
            message
                .text
                .starts_with("The task \"Nightly compaction\" finished.\n\n")
        );
        assert!(message.text.contains("Type: blobstore.compact\n"));
        assert!(message.text.contains("Result: OK\n"));
        assert!(message.text.contains("Started by: admin\n"));
        assert!(message.text.contains("Summary: compacted 3 blobs\n"));
        assert!(!message.text.contains("Error:"));
        assert!(!message.text.contains("Open the task"));
        assert!(
            message
                .text
                .ends_with("notifies ops@example.org after every run.\n")
        );
        assert_eq!(report.message(Some("  ")).text, message.text);
    }

    #[test]
    fn a_stopped_run_is_reported_as_canceled() {
        let mut stopped = run(
            RunState::Cancelled,
            "manual",
            "jane",
            "stopped before it finished",
        );
        stopped.node = String::new();
        stopped.started_at = None;
        let report = Report::of(
            &task(Some("ops@example.org"), SUCCESS_FAILURE),
            &stopped,
            "Admin - Compact blob store",
        )
        .unwrap();
        let message = report.message(None);
        assert_eq!(message.subject, "Task canceled: Nightly compaction");
        assert!(
            message
                .text
                .contains("The task \"Nightly compaction\" was stopped.")
        );
        assert!(message.text.contains("Result: Canceled\n"));
        assert!(message.text.contains("Finished: 2026-10-24 01:00:05 UTC\n"));
        assert!(!message.text.contains("Started:"));
        assert!(!message.text.contains("Server:"));
    }
}
