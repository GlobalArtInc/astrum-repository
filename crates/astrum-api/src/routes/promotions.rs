use std::sync::Arc;
use std::time::{Duration, Instant};

use astrum_content::store::{self, Component};
use astrum_content::transfer::{
    AssetOutcome, AssetStatus, Bundle, Conflict, LocalSource, MAX_COMPONENTS, Outcome, PROMOTED,
    Status, Terms,
};
use astrum_content::{ContentError, Ctx, Format, View, ids};
use astrum_db::audit::Event;
use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use astrum_security::Scope;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{ApiError, Validation};
use crate::images::{ImageError, ImageItem, ImageReport};
use crate::routes::internal::ui::UI;
use crate::{AppState, Caller, REST};

pub const MAX_ITEMS: usize = 100;
pub const REQUEST_BUDGET: Duration = Duration::from_secs(600);
pub const DOMAIN: &str = "repository.content";
const MAX_TEXT: usize = 1024;
const MAX_LISTED: usize = 500;
const IMAGES: &str = "docker";
const READ_MISSING: &str =
    "read access to every asset of the component in the source repository is missing";
const DELETE_MISSING: &str =
    "delete access to the component in the source repository is missing, which a move needs";
const ADD_MISSING: &str = "add access to the target repository is missing for this content";
const EDIT_MISSING: &str =
    "edit access to the target repository is missing, which replacing content needs";
const UNTAGGED_WARNING: &str = "an untagged image is removed by the next garbage collection";
const DIGEST_MOVE: &str =
    "a move needs a tag: deleting by digest would remove every tag of the image";
const GENERIC_FAILURE: &str = "the server failed; the log has the details";
const SOURCE_GONE: &str = "the source was kept: the component is already gone";
const SOURCE_CHANGED: &str =
    "the source was kept: the component changed in the source while the move ran; retry the move";
const SOURCE_FAILED: &str = "the source was kept: the server failed while deleting it";
const NOTHING_HELD: &str = "the source was kept: no asset was written to or found in the target, so nothing proves the content is held there";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{REST}/v1/promotions"), post(promote))
        .route(&format!("{UI}/promotion-targets"), get(targets))
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    source: String,
    target: String,
    items: Vec<Item>,
    #[serde(default, rename = "move")]
    relocate: bool,
    #[serde(default)]
    conflict: Conflict,
    #[serde(default)]
    dry_run: bool,
    #[serde(default = "yes")]
    referrers: bool,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Item {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_tags: Option<Vec<String>>,
}

enum Shape {
    Id(String),
    Coordinates {
        namespace: String,
        name: String,
        version: String,
    },
    Image {
        image: String,
        reference: String,
        by_digest: bool,
        target_tags: Vec<String>,
    },
}

fn text(value: &str) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        return Err("must not be empty");
    }
    free_text(value)
}

fn free_text(value: &str) -> Result<(), &'static str> {
    if value.len() > MAX_TEXT {
        return Err("is too long");
    }
    if value
        .chars()
        .any(|character| character.is_control() || is_format(character))
    {
        return Err("must not contain control or invisible formatting characters");
    }
    Ok(())
}

fn is_format(character: char) -> bool {
    matches!(
        u32::from(character),
        0x00AD
            | 0x0600..=0x0605
            | 0x061C
            | 0x06DD
            | 0x070F
            | 0x0890..=0x0891
            | 0x08E2
            | 0x180E
            | 0x200B..=0x200F
            | 0x202A..=0x202E
            | 0x2060..=0x2064
            | 0x2066..=0x206F
            | 0xFEFF
            | 0xFFF9..=0xFFFB
            | 0x110BD
            | 0x110CD
            | 0x13430..=0x1343F
            | 0x1BCA0..=0x1BCA3
            | 0x1D173..=0x1D17A
            | 0xE0001
            | 0xE0020..=0xE007F
    )
}

impl Item {
    fn shape(&self) -> Result<Shape, String> {
        let required = |value: &Option<String>| -> Result<(), String> {
            value
                .as_deref()
                .map_or(Ok(()), |value| text(value).map_err(str::to_owned))
        };
        let optional = |value: &Option<String>| -> Result<(), String> {
            value
                .as_deref()
                .map_or(Ok(()), |value| free_text(value).map_err(str::to_owned))
        };
        for value in [
            &self.component_id,
            &self.name,
            &self.image,
            &self.tag,
            &self.digest,
        ] {
            required(value)?;
        }
        for value in [&self.namespace, &self.version] {
            optional(value)?;
        }
        let tags = self.target_tags.as_deref().unwrap_or_default();
        if tags.len() > MAX_COMPONENTS {
            return Err(format!("targetTags takes at most {MAX_COMPONENTS} tags"));
        }
        for tag in tags {
            text(tag).map_err(|problem| format!("targetTags: {problem}"))?;
        }
        let coordinates = self.name.is_some() || self.version.is_some() || self.namespace.is_some();
        let image = self.image.is_some()
            || self.tag.is_some()
            || self.digest.is_some()
            || self.target_tags.is_some();
        match (self.component_id.as_ref(), coordinates, image) {
            (Some(id), false, false) => Ok(Shape::Id(id.clone())),
            (None, true, false) => match (&self.name, &self.version) {
                (Some(name), Some(version)) => Ok(Shape::Coordinates {
                    namespace: self.namespace.clone().unwrap_or_default(),
                    name: name.clone(),
                    version: version.clone(),
                }),
                _ => Err("name and version are both required".to_owned()),
            },
            (None, false, true) => {
                let image = self
                    .image
                    .clone()
                    .ok_or_else(|| "image is required".to_owned())?;
                let (reference, by_digest) = match (&self.tag, &self.digest) {
                    (Some(tag), None) => (tag.clone(), false),
                    (None, Some(digest)) => (digest.clone(), true),
                    _ => return Err("exactly one of tag and digest is required".to_owned()),
                };
                Ok(Shape::Image {
                    image,
                    reference,
                    by_digest,
                    target_tags: tags.to_vec(),
                })
            }
            (None, false, false) => Err(
                "an item names a component by componentId, by name and version, or an image by image and tag or digest"
                    .to_owned(),
            ),
            _ => Err("an item uses one form only: componentId, name and version, or image".to_owned()),
        }
    }
}

struct Entry {
    item: Item,
    shape: Shape,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum Verdict {
    Copied,
    Unchanged,
    Skipped,
    Conflict,
    Forbidden,
    NotFound,
    Unsupported,
    Failed,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemResult {
    item: Item,
    status: Verdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    component_ids: Vec<String>,
    assets: Vec<AssetOutcome>,
    asset_count: usize,
    bytes_copied: u64,
    bytes_reused: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_kept: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skipped_children: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ItemResult {
    fn empty(item: Item, status: Verdict) -> Self {
        Self {
            item,
            status,
            component_id: None,
            component_ids: Vec::new(),
            assets: Vec::new(),
            asset_count: 0,
            bytes_copied: 0,
            bytes_reused: 0,
            source_kept: None,
            retryable: None,
            skipped_children: None,
            warning: None,
            error: None,
        }
    }

    fn refused(item: Item, refusal: Refusal) -> Self {
        let (status, message, retryable) = match refusal {
            Refusal::NotFound(message) => (Verdict::NotFound, message, None),
            Refusal::Forbidden(message) => (Verdict::Forbidden, message, None),
            Refusal::Unsupported(message) => (Verdict::Unsupported, message, None),
            Refusal::Failed { message, retryable } => {
                (Verdict::Failed, message, retryable.then_some(true))
            }
            Refusal::Conflict { message, retryable } => {
                (Verdict::Conflict, message, Some(retryable))
            }
        };
        let mut result = Self::empty(item, status);
        result.error = Some(message);
        result.retryable = retryable;
        result
    }
}

enum Refusal {
    NotFound(String),
    Forbidden(String),
    Unsupported(String),
    Failed { message: String, retryable: bool },
    Conflict { message: String, retryable: bool },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Plan,
    Run,
}

impl Refusal {
    fn from_content(error: ContentError, phase: Phase, forbidden: &str) -> Self {
        match error {
            ContentError::Unauthorized | ContentError::Forbidden => {
                Refusal::Forbidden(forbidden.to_owned())
            }
            ContentError::NotFound(message) => Refusal::NotFound(message),
            ContentError::BadRequest(message) => Refusal::Failed {
                message,
                retryable: false,
            },
            ContentError::Conflict(message) => Refusal::Conflict {
                message,
                retryable: phase == Phase::Run,
            },
            other => {
                tracing::error!(error = format!("{other:#}"), "a promotion item failed");
                Refusal::Failed {
                    message: GENERIC_FAILURE.to_owned(),
                    retryable: false,
                }
            }
        }
    }
}

fn content_failure(error: ContentError) -> ApiError {
    match error {
        ContentError::NotFound(message) => ApiError::NotFound(message),
        ContentError::BadRequest(message) => ApiError::BadRequest(message),
        ContentError::Unauthorized => ApiError::Unauthorized,
        ContentError::Forbidden => ApiError::forbidden(),
        other => ApiError::internal(other),
    }
}

fn sees(ctx: &Ctx) -> bool {
    [View::Browse, View::Read]
        .into_iter()
        .any(|view| ctx.grant(view).any())
}

fn reaches(ctx: &Ctx) -> bool {
    sees(ctx)
        || [View::Add, View::Edit, View::Delete]
            .into_iter()
            .any(|view| ctx.grant(view).any())
}

fn context(
    state: &AppState,
    caller: &Caller,
    repository: Arc<astrum_repo::config::Repository>,
) -> Ctx {
    state.content().context(
        repository,
        state.security().model(),
        caller.0.clone(),
        "",
        "",
    )
}

fn hosted_online(ctx: &Ctx, role: &str) -> Result<(), ApiError> {
    if ctx.kind() != RepositoryType::Hosted {
        return Err(ApiError::invalid(
            role,
            format!(
                "{} is a {} repository; promotion works between hosted repositories",
                ctx.name(),
                ctx.kind()
            ),
        ));
    }
    if !ctx.repository.online {
        return Err(ApiError::invalid(
            role,
            format!("{} is offline", ctx.name()),
        ));
    }
    Ok(())
}

async fn open(state: &AppState, caller: &Caller, name: &str) -> Result<Option<Ctx>, ApiError> {
    Ok(state
        .content()
        .repository(name)
        .await
        .map_err(content_failure)?
        .map(|repository| context(state, caller, repository)))
}

enum Removal<'a> {
    Component {
        component: &'a Component,
        bundle: &'a Bundle,
    },
    Tag {
        image: &'a str,
        tag: &'a str,
        digest: &'a str,
    },
}

struct Job {
    state: AppState,
    caller: Caller,
    source: Ctx,
    target: Ctx,
    format: Option<Arc<dyn Format>>,
    entries: Vec<Entry>,
    conflict: Conflict,
    relocate: bool,
    dry_run: bool,
    referrers: bool,
    budget: Duration,
}

async fn prepare(state: &AppState, caller: &Caller, request: Request) -> Result<Job, ApiError> {
    let mut validation = Validation::new();
    for (id, value) in [("source", &request.source), ("target", &request.target)] {
        if let Err(problem) = text(value) {
            validation.add(id, problem);
        }
    }
    validation.check(
        !request.items.is_empty(),
        "items",
        "must hold at least one item",
    );
    validation.check(
        request.items.len() <= MAX_ITEMS,
        "items",
        &format!("must hold at most {MAX_ITEMS} items"),
    );
    validation.finish()?;
    let mut entries = Vec::with_capacity(request.items.len());
    let mut shapes = Validation::new();
    for (index, item) in request.items.into_iter().enumerate() {
        match item.shape() {
            Ok(shape) => entries.push(Entry { item, shape }),
            Err(problem) => shapes.add(format!("items[{index}]"), problem),
        }
    }
    shapes.finish()?;
    let unknown = || ApiError::not_found("there is no such source repository");
    let source = open(state, caller, &request.source)
        .await?
        .filter(sees)
        .ok_or_else(unknown)?;
    hosted_online(&source, "source")?;
    let unavailable = || {
        ApiError::invalid(
            "target",
            "the target repository does not exist or is not available to you",
        )
    };
    let target = open(state, caller, &request.target)
        .await?
        .filter(reaches)
        .ok_or_else(unavailable)?;
    hosted_online(&target, "target")?;
    if source.name() == target.name() {
        return Err(ApiError::invalid(
            "target",
            "the source and the target must be different repositories",
        ));
    }
    if source.format_id() != target.format_id() {
        return Err(ApiError::invalid(
            "target",
            format!(
                "the source is a {} repository and the target a {} one",
                source.format_id(),
                target.format_id()
            ),
        ));
    }
    for (index, entry) in entries.iter().enumerate() {
        let image = matches!(entry.shape, Shape::Image { .. });
        if image != (source.format_id() == IMAGES) {
            return Err(ApiError::invalid(
                format!("items[{index}]"),
                match image {
                    true => "image items work in Docker repositories only",
                    false => "Docker repositories take image items: image and tag or digest",
                },
            ));
        }
    }
    let format = state.content().format(source.format_id());
    Ok(Job {
        state: state.clone(),
        caller: caller.clone(),
        source,
        target,
        format,
        entries,
        conflict: request.conflict,
        relocate: request.relocate,
        dry_run: request.dry_run,
        referrers: request.referrers,
        budget: state.promotion_budget().unwrap_or(REQUEST_BUDGET),
    })
}

async fn promote(
    State(state): State<AppState>,
    caller: Caller,
    body: Result<crate::Json<Request>, ApiError>,
) -> Result<Response, ApiError> {
    let subject = caller.signed_in()?;
    if subject.scope == Scope::Read {
        return Err(ApiError::Forbidden(
            "a token with the READ scope cannot promote".to_owned(),
        ));
    }
    let crate::Json(request) = body?;
    let job = prepare(&state, &caller, request).await?;
    let dry_run = job.dry_run;
    let client = crate::routes::mcp::client();
    let address = crate::client::current();
    let results = tokio::spawn(scoped(client, address, job.run()))
        .await
        .map_err(ApiError::internal)?;
    Ok(axum::Json(json!({"dryRun": dry_run, "results": results})).into_response())
}

async fn scoped<F>(
    client: Option<String>,
    address: Option<std::net::IpAddr>,
    future: F,
) -> F::Output
where
    F: Future,
{
    crate::client::with_address(address, async move {
        match client {
            Some(client) => crate::routes::mcp::scoped(client, future).await,
            None => future.await,
        }
    })
    .await
}

fn may_delete_source(outcome: &Outcome) -> bool {
    matches!(outcome.status, Status::Copied | Status::Unchanged) && !outcome.assets.is_empty()
}

fn conflict_text(outcome: &Outcome) -> String {
    let paths: Vec<&str> = outcome
        .assets
        .iter()
        .filter(|asset| asset.status == AssetStatus::Conflict)
        .map(|asset| asset.path.as_str())
        .collect();
    let shown: Vec<&str> = paths.iter().copied().take(5).collect();
    let more = paths.len() - shown.len();
    match more {
        0 => format!(
            "the target already has different content at {}",
            shown.join(", ")
        ),
        more => format!(
            "the target already has different content at {} and {more} more",
            shown.join(", ")
        ),
    }
}

impl Job {
    async fn run(self) -> Vec<ItemResult> {
        let deadline = Instant::now() + self.budget;
        let mut results = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            if Instant::now() >= deadline {
                let mut late = ItemResult::refused(
                    entry.item.clone(),
                    Refusal::Failed {
                        message: format!(
                            "timeout: the request time budget of {} seconds was used up before this item started",
                            self.budget.as_secs()
                        ),
                        retryable: true,
                    },
                );
                late.source_kept = (self.relocate && !self.dry_run).then_some(true);
                results.push(late);
                continue;
            }
            let result = match self.attempt(entry).await {
                Ok(result) => result,
                Err(refusal) => {
                    let mut refused = ItemResult::refused(entry.item.clone(), refusal);
                    refused.source_kept = (self.relocate && !self.dry_run).then_some(true);
                    refused
                }
            };
            results.push(result);
        }
        results
    }

    async fn attempt(&self, entry: &Entry) -> Result<ItemResult, Refusal> {
        if matches!(entry.shape, Shape::Image { .. }) {
            return self.image(entry).await;
        }
        let format = self.format.as_ref().ok_or_else(|| self.unsupported())?;
        let transfer = format.transfer().ok_or_else(|| self.unsupported())?;
        let component = self.locate(&entry.shape).await?;
        let bundle = transfer
            .bundle(&self.source, &component)
            .await
            .map_err(|error| Refusal::from_content(error, Phase::Plan, READ_MISSING))?;
        self.check_source(&bundle)?;
        self.check_target(&bundle)?;
        let blobs = LocalSource::new(self.source.clone());
        let planned = transfer
            .receive(
                &self.target,
                &bundle,
                &blobs,
                Terms::new(self.conflict, PROMOTED).dry_run(),
            )
            .await
            .map_err(|error| Refusal::from_content(error, Phase::Plan, ADD_MISSING))?;
        let effective = self
            .target_rights(&planned)
            .map_err(|message| Refusal::Forbidden(message.to_owned()))?;
        if self.dry_run {
            return Ok(self.report(&entry.item, &planned));
        }
        let outcome = match planned.status {
            Status::Copied => transfer
                .receive(
                    &self.target,
                    &bundle,
                    &blobs,
                    Terms::new(effective, PROMOTED),
                )
                .await
                .map_err(|error| Refusal::from_content(error, Phase::Run, ADD_MISSING))?,
            _ => planned,
        };
        let component_name = format!("{}:{}:{}", bundle.namespace, bundle.name, bundle.version);
        let removal = Removal::Component {
            component: &component,
            bundle: &bundle,
        };
        Ok(self
            .settle(&entry.item, &outcome, &component_name, removal)
            .await)
    }

    async fn settle(
        &self,
        item: &Item,
        outcome: &Outcome,
        component: &str,
        removal: Removal<'_>,
    ) -> ItemResult {
        let mut result = self.report(item, outcome);
        let mut removed = false;
        if self.relocate {
            if may_delete_source(outcome) {
                match self.withdraw(removal).await {
                    Ok(()) => {
                        removed = true;
                        result.source_kept = Some(false);
                    }
                    Err(reason) => {
                        result.source_kept = Some(true);
                        result.error = Some(reason);
                    }
                }
            } else {
                result.source_kept = Some(true);
                if matches!(outcome.status, Status::Copied | Status::Unchanged) {
                    result.error = Some(NOTHING_HELD.to_owned());
                }
            }
        }
        match (outcome.status, removed) {
            (_, true) => self.audit("moved", component, outcome).await,
            (Status::Copied, false) => self.audit("promoted", component, outcome).await,
            _ => {}
        }
        result
    }

    async fn withdraw(&self, removal: Removal<'_>) -> Result<(), String> {
        match removal {
            Removal::Component { component, bundle } => self.remove(component, bundle).await,
            Removal::Tag { image, tag, digest } => self.remove_tag(image, tag, digest).await,
        }
    }

    async fn remove_tag(&self, image: &str, tag: &str, digest: &str) -> Result<(), String> {
        let transfers = self
            .state
            .image_transfers()
            .ok_or_else(|| "the source was kept: image promotion is not available".to_owned())?;
        match transfers.remove_tag(&self.source, image, tag, digest).await {
            Ok(()) => Ok(()),
            Err(ContentError::NotFound(_)) => {
                Err("the source was kept: the tag is already gone".to_owned())
            }
            Err(ContentError::Conflict(_)) => {
                Err("the source was kept: the tag changed while the transfer ran".to_owned())
            }
            Err(ContentError::Forbidden | ContentError::Unauthorized) => {
                Err(format!("the source was kept: {DELETE_MISSING}"))
            }
            Err(error) => {
                tracing::error!(
                    error = format!("{error:#}"),
                    repository = self.source.name(),
                    "the source tag of a moved image was not deleted"
                );
                Err(SOURCE_FAILED.to_owned())
            }
        }
    }

    fn unsupported(&self) -> Refusal {
        Refusal::Unsupported(format!(
            "promotion is not supported for {} repositories",
            self.source.format_id()
        ))
    }

    async fn image(&self, entry: &Entry) -> Result<ItemResult, Refusal> {
        let Shape::Image {
            image,
            reference,
            by_digest,
            target_tags,
        } = &entry.shape
        else {
            return Err(self.unsupported());
        };
        let transfers = self
            .state
            .image_transfers()
            .ok_or_else(|| self.unsupported())?;
        if self.relocate && *by_digest {
            return Err(Refusal::Failed {
                message: DIGEST_MOVE.to_owned(),
                retryable: false,
            });
        }
        if !self.target.grant(View::Add).any() {
            return Err(Refusal::Forbidden(ADD_MISSING.to_owned()));
        }
        let requested = format!("/v2/{image}/manifests/{reference}");
        let delete_path = requested.clone();
        let admit = |paths: &[String]| -> Result<(), String> {
            match paths
                .iter()
                .all(|path| self.target.allowed(View::Add, path))
            {
                true => Ok(()),
                false => Err(ADD_MISSING.to_owned()),
            }
        };
        let approve = |planned: &Outcome| -> Result<Conflict, String> {
            if self.relocate && !self.source.allowed(View::Delete, &delete_path) {
                return Err(DELETE_MISSING.to_owned());
            }
            self.target_rights(planned).map_err(str::to_owned)
        };
        let wanted = ImageItem {
            image: image.clone(),
            reference: reference.clone(),
            target_tags: target_tags.clone(),
            referrers: self.referrers,
        };
        let terms = match self.dry_run {
            true => Terms::new(self.conflict, PROMOTED).dry_run(),
            false => Terms::new(self.conflict, PROMOTED),
        };
        let ImageReport {
            outcome,
            skipped_children,
            untagged,
            version,
            digest,
        } = transfers
            .transfer(&self.source, &self.target, &wanted, terms, &admit, &approve)
            .await
            .map_err(|error| match error {
                ImageError::Plan(ContentError::Forbidden | ContentError::Unauthorized)
                    if !self.source.allowed(View::Browse, &requested)
                        && !self.source.allowed(View::Read, &requested) =>
                {
                    Refusal::NotFound("there is no such image in the source repository".to_owned())
                }
                ImageError::Plan(error) => Refusal::from_content(error, Phase::Plan, READ_MISSING),
                ImageError::Denied(message) => Refusal::Forbidden(message),
                ImageError::Run(error) => Refusal::from_content(error, Phase::Run, ADD_MISSING),
            })?;
        let component_name = format!(":{image}:{version}");
        let mut result = match self.dry_run {
            true => self.report(&entry.item, &outcome),
            false => {
                let removal = Removal::Tag {
                    image,
                    tag: reference,
                    digest: &digest,
                };
                self.settle(&entry.item, &outcome, &component_name, removal)
                    .await
            }
        };
        if skipped_children > 0 {
            result.skipped_children = Some(skipped_children);
        }
        if untagged {
            result.warning = Some(UNTAGGED_WARNING.to_owned());
        }
        Ok(result)
    }

    async fn locate(&self, shape: &Shape) -> Result<Component, Refusal> {
        let missing =
            || Refusal::NotFound("there is no such component in the source repository".to_owned());
        let db = self.source.db();
        let found = match shape {
            Shape::Id(id) => {
                let (repository, number) = ids::decode(id).ok_or_else(missing)?;
                if repository != self.source.name() {
                    return Err(missing());
                }
                store::component_by_id(db, number)
                    .await
                    .map_err(|error| Refusal::from_content(error, Phase::Plan, READ_MISSING))?
            }
            Shape::Coordinates {
                namespace,
                name,
                version,
            } => component::Entity::find()
                .filter(component::Column::Repository.eq(self.source.name()))
                .filter(component::Column::Namespace.eq(namespace.as_str()))
                .filter(component::Column::Name.eq(name.as_str()))
                .filter(component::Column::Version.eq(version.as_str()))
                .one(db)
                .await
                .map_err(|error| {
                    Refusal::from_content(ContentError::from(error), Phase::Plan, READ_MISSING)
                })?,
            Shape::Image { .. } => None,
        };
        found
            .filter(|found| found.repository == self.source.name())
            .ok_or_else(missing)
    }

    fn check_source(&self, bundle: &Bundle) -> Result<(), Refusal> {
        let mut visible = false;
        let mut incomplete = false;
        for asset in &bundle.assets {
            if self.source.allowed(View::Read, &asset.path) {
                visible = true;
            } else {
                incomplete = true;
                if self.source.allowed(View::Browse, &asset.path) {
                    visible = true;
                }
            }
        }
        if !visible {
            return Err(Refusal::NotFound(
                "there is no such component in the source repository".to_owned(),
            ));
        }
        if incomplete {
            return Err(Refusal::Forbidden(READ_MISSING.to_owned()));
        }
        if self.relocate
            && bundle
                .assets
                .iter()
                .any(|asset| !self.source.allowed(View::Delete, &asset.path))
        {
            return Err(Refusal::Forbidden(DELETE_MISSING.to_owned()));
        }
        Ok(())
    }

    fn check_target(&self, bundle: &Bundle) -> Result<(), Refusal> {
        match bundle
            .assets
            .iter()
            .all(|asset| self.target.allowed(View::Add, &asset.path))
        {
            true => Ok(()),
            false => Err(Refusal::Forbidden(ADD_MISSING.to_owned())),
        }
    }

    fn target_rights(&self, planned: &Outcome) -> Result<Conflict, &'static str> {
        if !planned
            .assets
            .iter()
            .all(|asset| self.target.allowed(View::Add, &asset.path))
        {
            return Err(ADD_MISSING);
        }
        let replacing = planned
            .assets
            .iter()
            .any(|asset| asset.status == AssetStatus::Replaced);
        if !replacing {
            return Ok(Conflict::Fail);
        }
        match planned
            .assets
            .iter()
            .all(|asset| self.target.allowed(View::Edit, &asset.path))
        {
            true => Ok(Conflict::Replace),
            false => Err(EDIT_MISSING),
        }
    }

    fn report(&self, item: &Item, outcome: &Outcome) -> ItemResult {
        let status = match outcome.status {
            Status::Copied => Verdict::Copied,
            Status::Unchanged => Verdict::Unchanged,
            Status::Skipped => Verdict::Skipped,
            Status::Conflict => Verdict::Conflict,
        };
        let mut result = ItemResult::empty(item.clone(), status);
        let target = self.target.name();
        result.component_id = outcome.component_id.map(|id| ids::encode(target, id));
        result.component_ids = outcome
            .component_ids
            .iter()
            .map(|id| ids::encode(target, *id))
            .collect();
        result.assets = outcome.assets.iter().take(MAX_LISTED).cloned().collect();
        result.asset_count = outcome.assets.len();
        result.bytes_copied = outcome.bytes_copied;
        result.bytes_reused = outcome.bytes_reused;
        if outcome.status == Status::Conflict {
            result.retryable = Some(false);
            result.error = Some(conflict_text(outcome));
        }
        result
    }

    async fn remove(&self, component: &Component, bundle: &Bundle) -> Result<(), String> {
        let expected: Vec<(String, String)> = bundle
            .assets
            .iter()
            .map(|asset| (asset.path.clone(), asset.sha256.clone()))
            .collect();
        match store::delete_component_matching(self.state.content(), component.id, &expected).await
        {
            Ok(true) => Ok(()),
            Ok(false) => Err(SOURCE_GONE.to_owned()),
            Err(ContentError::Conflict(_)) => Err(SOURCE_CHANGED.to_owned()),
            Err(error) => {
                tracing::error!(
                    error = format!("{error:#}"),
                    repository = self.source.name(),
                    "the source of a moved component was not deleted"
                );
                Err(SOURCE_FAILED.to_owned())
            }
        }
    }

    async fn audit(&self, kind: &str, component: &str, outcome: &Outcome) {
        let event = Event::new(
            DOMAIN,
            kind,
            format!("{} -> {}", self.source.name(), self.target.name()),
        )
        .with(json!({
            "source": self.source.name(),
            "target": self.target.name(),
            "component": component,
            "assets": outcome.assets.len(),
            "bytes": outcome.bytes_copied,
            "dryRun": false,
            "via": "rest",
        }));
        self.state.audit(&self.caller, event).await;
    }
}

#[derive(Deserialize)]
struct TargetsQuery {
    #[serde(default)]
    source: Option<String>,
}

async fn targets(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<TargetsQuery>,
) -> Result<Response, ApiError> {
    caller.signed_in()?;
    let name = query.source.as_deref().unwrap_or_default();
    if let Err(problem) = text(name) {
        return Err(ApiError::invalid("source", problem));
    }
    let unknown = || ApiError::not_found("there is no such source repository");
    let source = open(&state, &caller, name)
        .await?
        .filter(sees)
        .ok_or_else(unknown)?;
    let transferable = match source.format_id() == IMAGES {
        true => state.image_transfers().is_some(),
        false => state
            .content()
            .format(source.format_id())
            .is_some_and(|format| format.transfer().is_some()),
    };
    let supported =
        source.kind() == RepositoryType::Hosted && source.repository.online && transferable;
    if !supported {
        return Ok(axum::Json(Vec::<serde_json::Value>::new()).into_response());
    }
    let mut found: Vec<(String, serde_json::Value)> = Vec::new();
    let repositories = astrum_repo::store::list(state.db().conn())
        .await
        .map_err(ApiError::internal)?;
    for repository in repositories {
        if repository.name == source.name()
            || repository.kind != RepositoryType::Hosted
            || !repository.online
            || repository.format.id != source.format_id()
        {
            continue;
        }
        let name = repository.name.clone();
        let ctx = context(&state, &caller, Arc::new(repository));
        if ctx.grant(View::Add).any() {
            found.push((
                name.clone(),
                json!({"name": name, "format": ctx.format_id(), "type": "hosted"}),
            ));
        }
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));
    let listed: Vec<serde_json::Value> = found.into_iter().map(|(_, value)| value).collect();
    Ok(axum::Json(listed).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str, version: &str) -> Item {
        Item {
            name: Some(name.to_owned()),
            version: Some(version.to_owned()),
            ..Item::default()
        }
    }

    #[test]
    fn a_content_conflict_is_retryable_only_after_the_plan_passed() {
        for (phase, expected) in [(Phase::Plan, false), (Phase::Run, true)] {
            let refusal = Refusal::from_content(
                ContentError::Conflict("changed while the transfer ran".to_owned()),
                phase,
                ADD_MISSING,
            );
            assert!(matches!(
                refusal,
                Refusal::Conflict { retryable, .. } if retryable == expected
            ));
        }
    }

    #[test]
    fn rights_errors_name_the_missing_right_and_nothing_else() {
        for error in [ContentError::Forbidden, ContentError::Unauthorized] {
            let refusal = Refusal::from_content(error, Phase::Plan, READ_MISSING);
            assert!(matches!(refusal, Refusal::Forbidden(message) if message == READ_MISSING));
        }
    }

    #[test]
    fn an_item_takes_exactly_one_form() {
        assert!(matches!(
            named("a", "").shape(),
            Ok(Shape::Coordinates { .. })
        ));
        assert!(named("a", "").shape().is_ok());
        let both = Item {
            component_id: Some("x".to_owned()),
            ..named("a", "1")
        };
        assert!(both.shape().is_err());
        let image = Item {
            image: Some("app".to_owned()),
            tag: Some("1".to_owned()),
            digest: Some("sha256:00".to_owned()),
            ..Item::default()
        };
        assert!(image.shape().is_err());
        assert!(Item::default().shape().is_err());
    }

    fn outcome(status: Status, assets: usize) -> Outcome {
        Outcome {
            status,
            component_id: None,
            component_ids: Vec::new(),
            assets: (0..assets)
                .map(|index| AssetOutcome {
                    path: format!("/a{index}"),
                    status: AssetStatus::Unchanged,
                    size: 1,
                })
                .collect(),
            bytes_copied: 0,
            bytes_reused: 0,
        }
    }

    #[test]
    fn the_source_goes_only_when_the_target_is_proven_to_hold_something() {
        for (status, assets, expected) in [
            (Status::Copied, 1, true),
            (Status::Unchanged, 1, true),
            (Status::Unchanged, 0, false),
            (Status::Copied, 0, false),
            (Status::Skipped, 1, false),
            (Status::Conflict, 1, false),
        ] {
            assert_eq!(
                may_delete_source(&outcome(status, assets)),
                expected,
                "{status:?} with {assets} assets"
            );
        }
    }

    #[test]
    fn names_with_invisible_formatting_characters_are_refused() {
        for bad in [
            "a\u{202e}b",
            "a\u{200b}",
            "\u{feff}a",
            "a\u{2066}",
            "a\u{1}",
        ] {
            assert!(free_text(bad).is_err(), "{bad:?}");
        }
        for good in [
            "app",
            "caf\u{e9}",
            "\u{65e5}\u{672c}\u{8a9e}",
            "1.0-rc.1+build",
        ] {
            assert!(free_text(good).is_ok(), "{good:?}");
        }
    }
}
