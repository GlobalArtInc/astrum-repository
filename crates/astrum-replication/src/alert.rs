use std::time::Duration;

use astrum_notify::alerts::{
    self, REPLICATION_DEAD_ITEMS, REPLICATION_FAILING, REPLICATION_RECOVERED, ReplicationReport,
};
use sea_orm::{ConnectionTrait, DbErr, FromQueryResult, TransactionTrait};
use uuid::Uuid;

use crate::dispatch::Dispatcher;
use crate::queue::statement;

pub const SILENCE: Duration = Duration::from_secs(30 * 60);

const EVALUATE: &str = "SELECT r.id, r.name, r.enabled, r.alert_state, r.alert_channels, \
     r.last_error, m.name AS remote_name, m.last_error AS remote_error, \
     coalesce(m.suspended_until > now(), false) AS suspended, \
     count(i.id) FILTER (WHERE i.state = 'pending') AS pending, \
     count(i.id) FILTER (WHERE i.state = 'leased') AS leased, \
     count(i.id) FILTER (WHERE i.state = 'dead') AS dead, \
     extract(epoch FROM now() - r.last_success_at)::float8 AS since_success, \
     extract(epoch FROM now() - min(i.first_queued_at) \
         FILTER (WHERE i.state IN ('pending', 'leased')))::float8 AS oldest, \
     (SELECT d.last_error FROM replication_item d \
         WHERE d.replication_id = r.id AND d.state = 'dead' ORDER BY d.id DESC LIMIT 1) AS dead_error \
     FROM replication r \
     JOIN replication_remote m ON m.id = r.remote_id \
     LEFT JOIN replication_item i ON i.replication_id = r.id \
     GROUP BY r.id, m.id ORDER BY r.id";

const MOVE: &str = "UPDATE replication SET alert_state = $3 \
     WHERE id = $1 AND alert_state = $2 RETURNING id";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ok,
    Failing,
    DeadItems,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::Ok => "ok",
            State::Failing => "failing",
            State::DeadItems => "dead_items",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        [State::Ok, State::Failing, State::DeadItems]
            .into_iter()
            .find(|state| state.as_str() == text)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alerted {
    pub replication: Uuid,
    pub from: String,
    pub to: State,
    pub messages: usize,
}

#[derive(Debug, FromQueryResult)]
struct Row {
    id: Uuid,
    name: String,
    enabled: bool,
    alert_state: String,
    alert_channels: Vec<Uuid>,
    last_error: Option<String>,
    remote_name: String,
    remote_error: Option<String>,
    suspended: bool,
    pending: i64,
    leased: i64,
    dead: i64,
    since_success: Option<f64>,
    oldest: Option<f64>,
    dead_error: Option<String>,
}

impl Row {
    fn decide(&self) -> State {
        if !self.enabled {
            return State::Ok;
        }
        if self.dead > 0 {
            return State::DeadItems;
        }
        let silence = SILENCE.as_secs_f64();
        let waiting = self.pending + self.leased > 0;
        let stale = waiting
            && self.oldest.is_some_and(|age| age >= silence)
            && self.since_success.is_none_or(|age| age >= silence);
        match stale || self.suspended {
            true => State::Failing,
            false => State::Ok,
        }
    }

    fn error(&self, state: State) -> Option<String> {
        let first = match state {
            State::DeadItems => self.dead_error.as_ref(),
            _ if self.suspended => self.remote_error.as_ref(),
            _ => self.last_error.as_ref(),
        };
        first
            .or(self.last_error.as_ref())
            .or(self.remote_error.as_ref())
            .cloned()
    }

    fn report(&self, state: State) -> ReplicationReport {
        ReplicationReport {
            replication: self.name.clone(),
            remote: self.remote_name.clone(),
            pending: self.pending,
            leased: self.leased,
            dead: self.dead,
            suspended: self.suspended,
            last_error: self.error(state),
        }
    }
}

fn event_for(previous: Option<State>, wanted: State) -> Option<&'static str> {
    match (wanted, previous) {
        (State::Ok, Some(State::Failing | State::DeadItems)) => Some(REPLICATION_RECOVERED),
        (State::Ok, _) => None,
        (State::Failing, _) => Some(REPLICATION_FAILING),
        (State::DeadItems, _) => Some(REPLICATION_DEAD_ITEMS),
    }
}

impl Dispatcher {
    pub async fn evaluate_alerts(&self) -> Result<Vec<Alerted>, DbErr> {
        let rows = Row::find_by_statement(statement(EVALUATE, Vec::new()))
            .all(&self.db())
            .await?;
        let mut moved = Vec::new();
        for row in rows {
            let wanted = row.decide();
            if wanted.as_str() == row.alert_state {
                continue;
            }
            if let Some(alerted) = self.transition(&row, wanted).await? {
                moved.push(alerted);
            }
        }
        Ok(moved)
    }

    async fn transition(&self, row: &Row, wanted: State) -> Result<Option<Alerted>, DbErr> {
        let _permit = self
            .inner
            .gate
            .acquire()
            .await
            .map_err(|_| DbErr::Custom("the replication database gate is closed".to_owned()))?;
        let txn = self.content().db().begin().await?;
        let moved = txn
            .query_one(statement(
                MOVE,
                vec![
                    row.id.into(),
                    row.alert_state.clone().into(),
                    wanted.as_str().into(),
                ],
            ))
            .await?;
        if moved.is_none() {
            txn.rollback().await?;
            return Ok(None);
        }
        let event = event_for(State::parse(&row.alert_state), wanted).filter(|_| row.enabled);
        let mut messages = 0;
        if let Some(event) = event {
            let report = row.report(wanted);
            for id in &row.alert_channels {
                let Some(channel) = astrum_notify::store::find(&txn, *id).await? else {
                    continue;
                };
                if !channel.enabled {
                    continue;
                }
                let text = alerts::render_replication(&channel, event, &report);
                alerts::queue(&txn, &channel, event, text).await?;
                messages += 1;
            }
        }
        txn.commit().await?;
        Ok(Some(Alerted {
            replication: row.id,
            from: row.alert_state.clone(),
            to: wanted,
            messages,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> Row {
        Row {
            id: Uuid::nil(),
            name: "mirror".to_owned(),
            enabled: true,
            alert_state: "ok".to_owned(),
            alert_channels: Vec::new(),
            last_error: None,
            remote_name: "dr".to_owned(),
            remote_error: None,
            suspended: false,
            pending: 3,
            leased: 0,
            dead: 0,
            since_success: Some(3600.0),
            oldest: Some(3600.0),
            dead_error: None,
        }
    }

    #[test]
    fn the_state_follows_dead_items_then_silence_then_suspension() {
        assert_eq!(row().decide(), State::Failing);
        let young = Row {
            oldest: Some(60.0),
            ..row()
        };
        assert_eq!(young.decide(), State::Ok);
        let recent = Row {
            since_success: Some(60.0),
            ..row()
        };
        assert_eq!(recent.decide(), State::Ok);
        let never = Row {
            since_success: None,
            ..row()
        };
        assert_eq!(never.decide(), State::Failing);
        let idle = Row {
            pending: 0,
            oldest: None,
            ..row()
        };
        assert_eq!(idle.decide(), State::Ok);
        let paused = Row {
            suspended: true,
            ..idle
        };
        assert_eq!(paused.decide(), State::Failing);
        let dead = Row { dead: 1, ..row() };
        assert_eq!(dead.decide(), State::DeadItems);
        let off = Row {
            enabled: false,
            dead: 1,
            ..row()
        };
        assert_eq!(off.decide(), State::Ok);
    }

    #[test]
    fn only_a_move_into_a_problem_or_back_out_of_one_has_a_message() {
        assert_eq!(
            event_for(Some(State::Ok), State::Failing),
            Some(REPLICATION_FAILING)
        );
        assert_eq!(
            event_for(Some(State::Failing), State::DeadItems),
            Some(REPLICATION_DEAD_ITEMS)
        );
        assert_eq!(
            event_for(Some(State::DeadItems), State::Failing),
            Some(REPLICATION_FAILING)
        );
        assert_eq!(
            event_for(Some(State::DeadItems), State::Ok),
            Some(REPLICATION_RECOVERED)
        );
        assert_eq!(event_for(Some(State::Ok), State::Ok), None);
        assert_eq!(event_for(None, State::Ok), None);
    }
}
