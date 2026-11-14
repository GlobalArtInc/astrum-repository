use sea_orm::{DbBackend, FromQueryResult, Statement};
use uuid::Uuid;

use super::{Exposition, flag};
use crate::AppState;

const BACKLOG: &str = "SELECT r.id, r.name, \
     (SELECT count(*) FROM replication_item i \
         WHERE i.replication_id = r.id AND i.state = 'pending') AS pending, \
     (SELECT count(*) FROM replication_item i \
         WHERE i.replication_id = r.id AND i.state = 'dead') AS dead, \
     coalesce(extract(epoch FROM now() - \
         (SELECT min(i.first_queued_at) FROM replication_item i \
             WHERE i.replication_id = r.id AND i.state IN ('pending', 'leased'))), 0)::float8 AS lag, \
     coalesce(extract(epoch FROM r.last_success_at), 0)::float8 AS last_success \
     FROM replication r ORDER BY lower(r.name)";

const REMOTES: &str = "SELECT name, coalesce(suspended_until > now(), false) AS suspended \
     FROM replication_remote ORDER BY lower(name)";

#[derive(FromQueryResult)]
struct Backlog {
    id: Uuid,
    name: String,
    pending: i64,
    dead: i64,
    lag: f64,
    last_success: f64,
}

#[derive(FromQueryResult)]
struct Remote {
    name: String,
    suspended: bool,
}

pub(super) struct Measured {
    backlog: Vec<Backlog>,
    remotes: Vec<Remote>,
}

pub(super) async fn measure(state: &AppState) -> Option<Measured> {
    let db = state.db().conn();
    let backlog = Backlog::find_by_statement(Statement::from_string(DbBackend::Postgres, BACKLOG))
        .all(db)
        .await
        .ok()?;
    let remotes = Remote::find_by_statement(Statement::from_string(DbBackend::Postgres, REMOTES))
        .all(db)
        .await
        .ok()?;
    Some(Measured { backlog, remotes })
}

pub(super) fn render(out: &mut Exposition, state: &AppState, measured: &Measured) {
    out.family(
        "astrum_replication_pending",
        "gauge",
        "Components waiting to be sent by a replication.",
    );
    for row in &measured.backlog {
        out.sample(
            "astrum_replication_pending",
            &[("replication", &row.name)],
            row.pending,
        );
    }
    out.family(
        "astrum_replication_dead",
        "gauge",
        "Components a replication gave up on until an operator retries or discards them.",
    );
    for row in &measured.backlog {
        out.sample(
            "astrum_replication_dead",
            &[("replication", &row.name)],
            row.dead,
        );
    }
    out.family(
        "astrum_replication_lag_seconds",
        "gauge",
        "Age of the oldest component a replication has not delivered yet, 0 when none waits.",
    );
    for row in &measured.backlog {
        out.sample(
            "astrum_replication_lag_seconds",
            &[("replication", &row.name)],
            row.lag.max(0.0),
        );
    }
    out.family(
        "astrum_replication_last_success_timestamp_seconds",
        "gauge",
        "When a replication last delivered a component, 0 when it never did.",
    );
    for row in &measured.backlog {
        out.sample(
            "astrum_replication_last_success_timestamp_seconds",
            &[("replication", &row.name)],
            row.last_success,
        );
    }
    out.family(
        "astrum_replication_remote_suspended",
        "gauge",
        "1 while a replication remote is paused after failures, 0 otherwise.",
    );
    for remote in &measured.remotes {
        out.sample(
            "astrum_replication_remote_suspended",
            &[("remote", &remote.name)],
            flag(remote.suspended),
        );
    }
    out.family(
        "astrum_replication_transfers_total",
        "counter",
        "Transfer attempts of the replication dispatcher on this node, by result: ok, failed or dead.",
    );
    for row in &measured.backlog {
        let totals = state.metrics().replication().of(row.id);
        for (result, count) in [
            ("ok", totals.ok),
            ("failed", totals.failed),
            ("dead", totals.dead),
        ] {
            out.sample(
                "astrum_replication_transfers_total",
                &[("replication", &row.name), ("result", result)],
                count,
            );
        }
    }
    out.family(
        "astrum_replication_bytes_total",
        "counter",
        "Blob bytes this node transferred for a replication: uploaded to the remote for push, downloaded from it for pull.",
    );
    for row in &measured.backlog {
        out.sample(
            "astrum_replication_bytes_total",
            &[("replication", &row.name)],
            state.metrics().replication().of(row.id).bytes,
        );
    }
}
