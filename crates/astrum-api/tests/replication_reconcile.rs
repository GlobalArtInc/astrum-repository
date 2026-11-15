mod accounts;
mod cluster;
mod support;

use std::sync::Arc;

use astrum_content::transfer::{BlobSource, Bundle, Outcome as Received, Terms, Transfer};
use astrum_content::{
    Asset, BrowseLayout, Component, ContentError, ContentRequest, Ctx, Format, SearchKey,
    UploadForm, UploadSpec,
};
use astrum_replication::dispatch::Dispatcher;
use astrum_replication::lifecycle;
use astrum_replication::reconcile::{self, Ended, Limits, Outcome, Summary};
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use cluster::{Fail, Node, REMOTES, REPLICATIONS};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::Auth;
use uuid::Uuid;

const TASKS: &str = "/service/rest/v1/tasks";

trait Reconciling {
    async fn reconcile(&self, replication: &str) -> Summary;
    async fn queued(&self) -> Vec<String>;
    async fn drain(&self);
}

impl Reconciling for Node {
    async fn reconcile(&self, replication: &str) -> Summary {
        let id = Uuid::parse_str(replication).unwrap();
        reconcile::run(self.fixture.state.content(), id)
            .await
            .unwrap()
    }

    async fn queued(&self) -> Vec<String> {
        let db = self.fixture.state.db().conn();
        let rows = db
            .query_all(Statement::from_string(
                DbBackend::Postgres,
                "SELECT name, reason FROM replication_item ORDER BY name",
            ))
            .await
            .unwrap();
        rows.iter()
            .map(|row| {
                let reason: String = row.try_get("", "reason").unwrap();
                assert_eq!(reason, "reconcile");
                row.try_get("", "name").unwrap()
            })
            .collect()
    }

    async fn drain(&self) {
        let dispatcher = Dispatcher::new(self.fixture.state.content().clone());
        for _ in 0..200 {
            if dispatcher.run_once().await.unwrap().leased == 0 {
                return;
            }
        }
        panic!("the queue did not drain");
    }
}

async fn start(repository: &str) -> Node {
    let node = Node::start(&[repository]).await;
    node.fixture
        .state
        .tasks()
        .register(Arc::new(reconcile::Reconcile));
    node
}

struct Pair {
    sender: Node,
    receiver: Node,
    replication: String,
    body: Value,
}

async fn nodes() -> (Node, Node, Value) {
    let sender = start("source").await;
    let receiver = start("mirror").await;
    let body = json!({
        "name": format!("remote-{}", Uuid::new_v4().simple()),
        "url": format!("http://{}", receiver.address),
        "token": receiver.token,
        "allowPrivateNetwork": true,
    });
    let made = sender
        .fixture
        .call(Method::POST, REMOTES, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    let remote = made.json()["id"].clone();
    (sender, receiver, remote)
}

fn replication_body(remote: &Value, extra: Value) -> Value {
    let mut body = json!({
        "name": format!("replication-{}", Uuid::new_v4().simple()),
        "sourceRepository": "source",
        "remoteId": remote,
        "remoteRepository": "mirror",
        "settleSeconds": 0,
    });
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

async fn create(sender: &Node, body: &Value) -> String {
    let made = sender
        .fixture
        .call(Method::POST, REPLICATIONS, Auth::Admin, Some(body.clone()))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    made.json()["id"].as_str().unwrap().to_owned()
}

async fn pair(extra: Value) -> Pair {
    let (sender, receiver, remote) = nodes().await;
    let body = replication_body(&remote, extra);
    let replication = create(&sender, &body).await;
    Pair {
        sender,
        receiver,
        replication,
        body,
    }
}

async fn seed(pair: &Pair, names: &[&str]) {
    for name in names {
        pair.sender.put("source", name, name.as_bytes()).await;
    }
    pair.sender.sql("DELETE FROM replication_item").await;
}

#[tokio::test]
async fn the_initial_sync_queues_every_component_and_a_synced_pair_queues_none() {
    let pair = pair(json!({})).await;
    for number in 0..1500 {
        pair.sender
            .put("source", &format!("/bulk/{number:04}"), b"x")
            .await;
    }
    pair.sender.sql("DELETE FROM replication_item").await;
    let first = pair.sender.reconcile(&pair.replication).await;
    assert_eq!(first.outcome, Outcome::Complete);
    assert_eq!((first.tally.scanned, first.tally.enqueued), (1500, 1500));
    assert_eq!(pair.receiver.force.compares(), 2);
    let reasons = "SELECT count(*) AS n FROM replication_item WHERE reason = 'reconcile'";
    assert_eq!(pair.sender.number(reasons).await, 1500);
    pair.sender.drain().await;
    let arrived = "SELECT count(*) AS n FROM component WHERE repository = 'mirror'";
    assert_eq!(pair.receiver.number(arrived).await, 1500);
    let second = pair.sender.reconcile(&pair.replication).await;
    assert_eq!(second.outcome, Outcome::Complete);
    assert_eq!(second.tally.enqueued, 0);
}

#[tokio::test]
async fn a_changed_copy_and_a_missing_component_are_queued() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b", "/c"]).await;
    pair.sender.reconcile(&pair.replication).await;
    pair.sender.drain().await;
    pair.receiver.put("mirror", "/a", b"changed").await;
    pair.receiver.delete("mirror", "/c").await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!(summary.tally.enqueued, 2);
    assert_eq!(pair.sender.queued().await, ["/a", "/c"]);
}

#[tokio::test]
async fn a_component_that_came_from_the_remote_is_not_queued() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b"]).await;
    let origin = pair.receiver.instance().await;
    pair.sender
        .sql(&format!(
            "UPDATE component SET attributes = jsonb_build_object('replication', \
             jsonb_build_object('origin', '{origin}')) WHERE name = '/a'"
        ))
        .await;
    pair.sender.sql("DELETE FROM replication_item").await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!((summary.tally.from_remote, summary.tally.enqueued), (1, 1));
    assert_eq!(pair.sender.queued().await, ["/b"]);
}

#[tokio::test]
async fn remote_only_keys_are_queued_for_deletion_only_when_enabled_and_only_when_ours() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b", "/c"]).await;
    pair.sender.reconcile(&pair.replication).await;
    pair.sender.drain().await;
    pair.receiver
        .sql(&format!(
            "UPDATE component SET attributes = jsonb_build_object('replication', \
             jsonb_build_object('origin', '{}')) WHERE name = '/b'",
            Uuid::new_v4()
        ))
        .await;
    for name in ["/a", "/b", "/c"] {
        pair.sender.delete("source", name).await;
    }
    pair.sender.sql("DELETE FROM replication_item").await;
    let off = pair.sender.reconcile(&pair.replication).await;
    assert_eq!((off.tally.remote_scanned, off.tally.enqueued), (0, 0));
    pair.sender
        .sql("UPDATE replication SET propagate_deletes = true")
        .await;
    let on = pair.sender.reconcile(&pair.replication).await;
    assert_eq!((on.tally.remote_scanned, on.tally.enqueued), (3, 2));
    assert_eq!(pair.sender.queued().await, ["/a", "/c"]);
}

#[tokio::test]
async fn components_outside_the_filters_are_not_queued() {
    let pair = pair(json!({"nameFilter": "/keep/.*"})).await;
    seed(&pair, &["/keep/a", "/skip/b"]).await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!((summary.tally.filtered, summary.tally.enqueued), (1, 1));
    assert_eq!(pair.sender.queued().await, ["/keep/a"]);
    pair.sender.sql("DELETE FROM replication_item").await;
    pair.sender
        .sql("UPDATE replication SET version_filter = 'v.*'")
        .await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!((summary.tally.filtered, summary.tally.enqueued), (2, 0));
}

#[tokio::test]
async fn a_failing_remote_aborts_without_moving_the_mark_and_a_complete_run_moves_it() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b", "/c"]).await;
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    pair.sender.reconcile(&pair.replication).await;
    assert_eq!(pair.sender.number(mark).await, 1);
    pair.sender
        .sql("UPDATE replication SET last_reconcile_at = NULL")
        .await;
    let id = Uuid::parse_str(&pair.replication).unwrap();
    let content = pair.sender.fixture.state.content();
    let suspended = "SELECT count(*) AS n FROM replication_remote WHERE suspended_until > now()";
    pair.receiver
        .force
        .fail(Fail::new(400, 1).on("/inventory/compare"));
    let Ended::Failed(text) = reconcile::run(content, id).await.unwrap_err() else {
        panic!("a refused compare is the run's own failure");
    };
    assert!(text.chars().count() <= 500 && !text.contains(&pair.receiver.token));
    assert_eq!(pair.sender.number(mark).await, 0);
    assert_eq!(pair.sender.number(suspended).await, 0);
    pair.receiver.force.fail(Fail::new(503, 1000));
    let ended = reconcile::run(content, id).await.unwrap_err();
    assert!(matches!(ended, Ended::Down(_)), "{ended:?}");
    assert!(!ended.text().contains(&pair.receiver.token));
    assert_eq!(pair.sender.number(mark).await, 0);
    assert_eq!(pair.sender.number(suspended).await, 1);
    let skipped = pair.sender.reconcile(&pair.replication).await;
    assert!(matches!(skipped.outcome, Outcome::Skipped(_)));
    pair.receiver.force.fail(Fail::new(503, 0));
    pair.sender
        .sql("UPDATE replication_remote SET suspended_until = NULL")
        .await;
    pair.sender.reconcile(&pair.replication).await;
    assert_eq!(pair.sender.number(mark).await, 1);
}

#[tokio::test]
async fn one_run_queues_at_most_its_limit_and_goes_on_past_what_is_queued() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b", "/c", "/d", "/e"]).await;
    let limits = Limits {
        enqueue: 3,
        ..Limits::default()
    };
    let id = Uuid::parse_str(&pair.replication).unwrap();
    let content = pair.sender.fixture.state.content();
    let first = reconcile::run_with(content, id, &limits, &|| false)
        .await
        .unwrap();
    assert_eq!((first.outcome, first.tally.enqueued), (Outcome::Capped, 3));
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(pair.sender.number(mark).await, 0);
    let second = reconcile::run_with(content, id, &limits, &|| false)
        .await
        .unwrap();
    assert_eq!(
        (second.outcome, second.tally.enqueued),
        (Outcome::Complete, 2)
    );
    assert_eq!(pair.sender.queued().await.len(), 5);
}

#[tokio::test]
async fn a_run_can_be_stopped_between_pages() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a"]).await;
    let id = Uuid::parse_str(&pair.replication).unwrap();
    let content = pair.sender.fixture.state.content();
    let ended = reconcile::run_with(content, id, &Limits::default(), &|| true).await;
    assert!(matches!(ended, Err(Ended::Stopped)));
}

#[tokio::test]
async fn the_task_of_a_replication_follows_its_life() {
    let pair = pair(json!({})).await;
    let sender = &pair.sender;
    let task = |what: &str| {
        format!(
            "SELECT {what} AS t FROM task WHERE type = 'replication.reconcile' \
             AND properties ->> 'replicationId' = '{}'",
            pair.replication
        )
    };
    assert_eq!(
        sender.text(&task("name")).await,
        Some(format!(
            "replication-reconcile-{}",
            pair.body["name"].as_str().unwrap()
        ))
    );
    let cron = sender
        .text(&task("frequency ->> 'cronExpression'"))
        .await
        .unwrap();
    assert!(cron.ends_with(" 0/6 * * ?"), "{cron}");
    let runs = "SELECT count(*) AS n FROM task_run WHERE state = 'waiting'";
    assert_eq!(sender.number(runs).await, 1);
    sender.fixture.state.tasks().run_pending().await.unwrap();
    let finished = "SELECT state AS t FROM task_run ORDER BY id DESC LIMIT 1";
    assert_eq!(sender.text(finished).await.as_deref(), Some("done"));
    let marked = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(sender.number(marked).await, 1);

    let path = format!("{REPLICATIONS}/{}", pair.replication);
    let mut body = pair.body.clone();
    body["reconcileMinutes"] = json!(30);
    let changed = sender
        .fixture
        .call(Method::PUT, &path, Auth::Admin, Some(body.clone()))
        .await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.text());
    let cron = sender
        .text(&task("frequency ->> 'cronExpression'"))
        .await
        .unwrap();
    assert!(cron.ends_with("/30 * * * ?"), "{cron}");
    assert_eq!(sender.number(runs).await, 0);

    body["enabled"] = json!(false);
    let off = sender
        .fixture
        .call(Method::PUT, &path, Auth::Admin, Some(body.clone()))
        .await;
    assert_eq!(off.status, StatusCode::OK, "{}", off.text());
    assert_eq!(sender.number(runs).await, 0);
    let summary = sender.reconcile(&pair.replication).await;
    assert_eq!(
        summary.outcome,
        Outcome::Skipped("the replication is disabled or gone".to_owned())
    );
    body["enabled"] = json!(true);
    let on = sender
        .fixture
        .call(Method::PUT, &path, Auth::Admin, Some(body))
        .await;
    assert_eq!(on.status, StatusCode::OK, "{}", on.text());
    assert_eq!(sender.number(runs).await, 1);

    let gone = sender
        .fixture
        .call(Method::DELETE, &path, Auth::Admin, None)
        .await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT, "{}", gone.text());
    let left = "SELECT count(*) AS n FROM task WHERE type = 'replication.reconcile'";
    assert_eq!(sender.number(left).await, 0);
}

async fn call(sender: &Node, method: Method, path: &str, body: Option<Value>) -> support::Reply {
    sender.fixture.call(method, path, Auth::Admin, body).await
}

struct Fragile {
    raw: astrum_format_raw::Raw,
    transfer: FragileTransfer,
}

struct FragileTransfer;

fn raw_transfer() -> &'static dyn Transfer {
    static RAW: astrum_format_raw::Raw = astrum_format_raw::Raw;
    RAW.transfer().expect("raw takes part in replication")
}

#[async_trait]
impl Transfer for FragileTransfer {
    async fn bundle(&self, source: &Ctx, component: &Component) -> Result<Bundle, ContentError> {
        if component.name.contains("broken") {
            return Err(ContentError::internal(anyhow::anyhow!("unreadable")));
        }
        raw_transfer().bundle(source, component).await
    }

    async fn receive(
        &self,
        target: &Ctx,
        bundle: &Bundle,
        blobs: &dyn BlobSource,
        terms: Terms,
    ) -> Result<Received, ContentError> {
        raw_transfer().receive(target, bundle, blobs, terms).await
    }
}

#[async_trait]
impl Format for Fragile {
    fn id(&self) -> &'static str {
        self.raw.id()
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        self.raw.handle(ctx, request).await
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        self.raw.get(ctx, path).await
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        self.raw.upload_spec()
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        self.raw.upload(ctx, form).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        self.raw.search_keys()
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        self.raw.keyword_attributes()
    }

    fn browse_layout(&self) -> BrowseLayout {
        self.raw.browse_layout()
    }

    fn transfer(&self) -> Option<&dyn Transfer> {
        Some(&self.transfer)
    }
}

#[tokio::test]
async fn an_unreadable_component_fails_the_run_at_the_end_and_leaves_the_mark() {
    let pair = pair(json!({})).await;
    pair.sender
        .fixture
        .state
        .content()
        .register(Arc::new(Fragile {
            raw: astrum_format_raw::Raw,
            transfer: FragileTransfer,
        }));
    seed(&pair, &["/a", "/broken", "/c"]).await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert!(
        matches!(&summary.outcome, Outcome::Unreadable(text) if text.contains("broken")),
        "{:?}",
        summary.outcome
    );
    assert_eq!((summary.tally.unreadable, summary.tally.enqueued), (1, 2));
    assert_eq!(pair.sender.queued().await, ["/a", "/c"]);
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(pair.sender.number(mark).await, 0);
}

#[tokio::test]
async fn a_dead_item_stays_dead_and_the_run_still_completes() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b"]).await;
    pair.sender
        .sql(&format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, state, \
             reason, attempts, last_error) \
             VALUES ('{}', '/', '/a', '', 'dead', 'event', 12, 'refused')",
            pair.replication
        ))
        .await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!(summary.outcome, Outcome::Complete);
    assert_eq!(summary.tally.enqueued, 1);
    let dead = "SELECT state AS t FROM replication_item WHERE name = '/a'";
    assert_eq!(pair.sender.text(dead).await.as_deref(), Some("dead"));
    let attempts = "SELECT attempts::bigint AS n FROM replication_item WHERE name = '/a'";
    assert_eq!(pair.sender.number(attempts).await, 12);
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(pair.sender.number(mark).await, 1);
}

#[tokio::test]
async fn a_remote_key_outside_the_filters_is_not_queued_for_deletion() {
    let pair = pair(json!({"propagateDeletes": true})).await;
    seed(&pair, &["/keep/a", "/skip/b"]).await;
    pair.sender.reconcile(&pair.replication).await;
    pair.sender.drain().await;
    for name in ["/keep/a", "/skip/b"] {
        pair.sender.delete("source", name).await;
    }
    pair.sender.sql("DELETE FROM replication_item").await;
    pair.sender
        .sql("UPDATE replication SET name_filter = '/keep/.*'")
        .await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert_eq!(
        (summary.tally.remote_scanned, summary.tally.enqueued),
        (2, 1)
    );
    assert_eq!(pair.sender.queued().await, ["/keep/a"]);
}

#[tokio::test]
async fn a_task_that_already_has_the_wanted_name_makes_the_new_one_carry_the_id() {
    let (sender, _receiver, remote) = nodes().await;
    let body = replication_body(&remote, json!({}));
    let wanted = format!("replication-reconcile-{}", body["name"].as_str().unwrap());
    sender
        .sql(&format!(
            "INSERT INTO task (id, type, name) VALUES (gen_random_uuid(), \
             'blobstore.metrics.reconcile', '{wanted}')"
        ))
        .await;
    let id = create(&sender, &body).await;
    let own = format!(
        "SELECT name AS t FROM task WHERE type = 'replication.reconcile' \
         AND properties ->> 'replicationId' = '{id}'"
    );
    assert_eq!(sender.text(&own).await, Some(format!("{wanted}-{id}")));
}

#[tokio::test]
async fn the_reconcile_task_is_a_system_task_that_an_operator_can_only_run_stop_and_delete() {
    let pair = pair(json!({})).await;
    let sender = &pair.sender;
    let kind = "replication.reconcile";
    let listed = call(
        sender,
        Method::GET,
        "/service/rest/v1/tasks/templates",
        None,
    )
    .await;
    assert!(
        listed
            .json()
            .as_array()
            .unwrap()
            .iter()
            .all(|template| template["type"] != kind)
    );
    let made = call(
        sender,
        Method::POST,
        TASKS,
        Some(json!({
            "type": kind, "name": "mine",
            "frequency": {"schedule": "manual"},
            "properties": {"replicationId": pair.replication},
        })),
    )
    .await;
    assert_eq!(made.status, StatusCode::BAD_REQUEST, "{}", made.text());
    let id = sender
        .text(&format!(
            "SELECT id::text AS t FROM task WHERE type = '{kind}'"
        ))
        .await
        .unwrap();
    let path = format!("{TASKS}/{id}");
    let other = call(
        sender,
        Method::PUT,
        &path,
        Some(json!({"properties": {"replicationId": Uuid::new_v4()}})),
    )
    .await;
    assert_eq!(other.status, StatusCode::BAD_REQUEST, "{}", other.text());
    let same = call(
        sender,
        Method::PUT,
        &path,
        Some(json!({"enabled": false, "properties": {"replicationId": pair.replication}})),
    )
    .await;
    assert_eq!(same.status, StatusCode::OK, "{}", same.text());
    let gone = call(sender, Method::DELETE, &path, None).await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT, "{}", gone.text());
    let count = format!("SELECT count(*) AS n FROM task WHERE type = '{kind}'");
    let path = format!("{REPLICATIONS}/{}", pair.replication);
    let updated = call(sender, Method::PUT, &path, Some(pair.body.clone())).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    assert_eq!(sender.number(&count).await, 0);
    let db = sender.fixture.state.db().conn();
    assert_eq!(lifecycle::backfill(db).await.unwrap(), 1);
    assert_eq!(sender.number(&count).await, 1);
}

#[tokio::test]
async fn the_backfill_gives_a_replication_without_a_task_one_and_then_nothing() {
    let (sender, _receiver, remote) = nodes().await;
    let id = Uuid::new_v4();
    sender
        .sql(&format!(
            "INSERT INTO replication (id, name, source_repository, remote_id, remote_repository, \
             reconcile_minutes) VALUES ('{id}', 'legacy', 'source', '{}', 'mirror', 45)",
            remote.as_str().unwrap()
        ))
        .await;
    let db = sender.fixture.state.db().conn();
    assert_eq!(lifecycle::backfill(db).await.unwrap(), 1);
    assert_eq!(lifecycle::backfill(db).await.unwrap(), 0);
    let own = format!(
        "SELECT frequency ->> 'cronExpression' AS t FROM task WHERE type = 'replication.reconcile' \
         AND properties ->> 'replicationId' = '{id}'"
    );
    let cron = sender.text(&own).await.unwrap();
    assert!(cron.ends_with("/30 * * * ?"), "{cron}");
    let stored = "SELECT reconcile_minutes::bigint AS n FROM replication WHERE name = 'legacy'";
    assert_eq!(sender.number(stored).await, 45);
}

#[tokio::test]
async fn an_update_racing_a_delete_never_fails_and_leaves_no_task() {
    let (sender, _receiver, remote) = nodes().await;
    for round in 0..20 {
        let body = replication_body(
            &remote,
            json!({"remoteRepository": format!("mirror-{round}")}),
        );
        let id = create(&sender, &body).await;
        let path = format!("{REPLICATIONS}/{id}");
        let mut changed = body.clone();
        changed["reconcileMinutes"] = json!(30);
        let (update, delete) = tokio::join!(
            sender
                .fixture
                .call(Method::PUT, &path, Auth::Admin, Some(changed)),
            sender
                .fixture
                .call(Method::DELETE, &path, Auth::Admin, None),
        );
        assert!(
            [StatusCode::OK, StatusCode::NOT_FOUND].contains(&update.status),
            "{}",
            update.text()
        );
        assert_eq!(delete.status, StatusCode::NO_CONTENT, "{}", delete.text());
    }
    let tasks = "SELECT count(*) AS n FROM task WHERE type = 'replication.reconcile'";
    assert_eq!(sender.number(tasks).await, 0);
    assert_eq!(
        sender.number("SELECT count(*) AS n FROM replication").await,
        0
    );
}

#[tokio::test]
async fn a_suspended_remote_skips_the_run_and_leaves_the_mark() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a"]).await;
    pair.sender
        .sql("UPDATE replication_remote SET suspended_until = now() + interval '1 hour'")
        .await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert!(
        matches!(&summary.outcome, Outcome::Skipped(text) if text.starts_with("remote suspended until ")),
        "{:?}",
        summary.outcome
    );
    assert!(pair.sender.queued().await.is_empty());
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(pair.sender.number(mark).await, 0);
}

#[tokio::test]
async fn a_replication_of_a_format_that_cannot_be_replicated_is_skipped_and_its_items_die_once() {
    let pair = pair(json!({})).await;
    seed(&pair, &["/a", "/b"]).await;
    pair.sender
        .sql(&format!(
            "INSERT INTO replication_item (replication_id, namespace, name, version, reason) \
             VALUES ('{}', '/', '/a', '', 'reconcile')",
            pair.replication
        ))
        .await;
    pair.sender
        .fixture
        .state
        .content()
        .register(Arc::new(astrum_format_docker::Docker::default()));
    pair.sender
        .sql("UPDATE repository SET format = 'docker' WHERE name = 'source'")
        .await;
    let summary = pair.sender.reconcile(&pair.replication).await;
    assert!(
        matches!(&summary.outcome, Outcome::Skipped(text) if text.contains("Docker images cannot be replicated")),
        "{:?}",
        summary.outcome
    );
    assert!(summary.message().starts_with("skipped: Docker images"));
    assert_eq!(pair.sender.queued().await, ["/a"]);
    let mark = "SELECT count(*) AS n FROM replication WHERE last_reconcile_at IS NOT NULL";
    assert_eq!(pair.sender.number(mark).await, 0);

    pair.sender.drain().await;
    let dead = "SELECT state AS t FROM replication_item WHERE name = '/a'";
    assert_eq!(pair.sender.text(dead).await.as_deref(), Some("dead"));
    let reason = "SELECT last_error AS t FROM replication_item WHERE name = '/a'";
    assert!(
        pair.sender
            .text(reason)
            .await
            .unwrap()
            .contains("Docker images cannot be replicated")
    );
    let attempts = "SELECT attempts::bigint AS n FROM replication_item WHERE name = '/a'";
    assert_eq!(pair.sender.number(attempts).await, 1);
    assert_eq!(pair.sender.pass().await.leased, 0);
    pair.sender.reconcile(&pair.replication).await;
    assert_eq!(pair.sender.text(dead).await.as_deref(), Some("dead"));
    assert_eq!(pair.sender.number(attempts).await, 1);
}
