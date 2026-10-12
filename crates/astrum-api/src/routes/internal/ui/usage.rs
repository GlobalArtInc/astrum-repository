use std::collections::{BTreeMap, HashMap};

use astrum_content::{DOWNLOAD_DAYS_KEPT, ids};
use astrum_repo::RepositoryType;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use chrono::{DateTime, Days, FixedOffset, NaiveDate, Utc};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::access::{Seen, Sql, survey};
use super::{UI, time};
use crate::{ApiError, AppState, Caller};

const DEFAULT_DAYS: i64 = 30;
const UNUSED_DAYS: i64 = 90;
const TOP: usize = 10;
const UNUSED_TOP: usize = 5;
const PATHS: usize = 50;

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/usage-dashboard"), get(usage))
}

#[derive(Deserialize)]
struct UsageQuery {
    days: Option<i64>,
    repository: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    days: i64,
    from: String,
    to: String,
    retention_days: i32,
    unused_days: i64,
    repository: Option<String>,
    repositories: Vec<RepositoryRef>,
    totals: Totals,
    downloads_per_day: Vec<Day>,
    top_components: Vec<UsageComponent>,
    top_components_all_time: Vec<UsageComponent>,
    top_repositories_by_downloads: Vec<RepositoryUsage>,
    top_repositories_by_storage: Vec<RepositoryUsage>,
    recent_components: Vec<UsageComponent>,
    unused: Unused,
}

#[derive(Serialize)]
struct RepositoryRef {
    name: String,
    format: String,
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Totals {
    repositories: usize,
    hosted: usize,
    proxy: usize,
    group: usize,
    components: i64,
    assets: i64,
    storage_bytes: i64,
    proxy_storage_bytes: i64,
    downloads: i64,
    downloads_today: i64,
    downloads_in_period: i64,
    components_downloaded_in_period: i64,
}

#[derive(Serialize)]
struct Day {
    day: String,
    downloads: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UsageComponent {
    id: String,
    repository: String,
    format: String,
    group: Option<String>,
    name: String,
    version: Option<String>,
    downloads: i64,
    created: String,
    created_by: Option<String>,
    browse_path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoryUsage {
    name: String,
    format: String,
    #[serde(rename = "type")]
    kind: &'static str,
    downloads: i64,
    storage_bytes: i64,
    components: i64,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Unused {
    components: i64,
    bytes: i64,
    never_downloaded_components: i64,
    never_downloaded_bytes: i64,
    repositories: Vec<UnusedRepository>,
}

#[derive(Serialize)]
struct UnusedRepository {
    name: String,
    format: String,
    components: i64,
    bytes: i64,
}

#[derive(Debug, FromQueryResult)]
struct ContentRow {
    repository: String,
    components: i64,
    assets: i64,
    bytes: i64,
    downloads: i64,
    unused_components: i64,
    unused_bytes: i64,
    never_components: i64,
    never_bytes: i64,
}

#[derive(Debug, FromQueryResult)]
struct DayRow {
    repository: Option<String>,
    day: Option<String>,
    downloads: Option<i64>,
    components: i64,
    total: i32,
}

#[derive(Debug, FromQueryResult)]
struct ComponentRow {
    id: i64,
    repository: String,
    format: String,
    namespace: String,
    name: String,
    version: String,
    created_at: DateTime<FixedOffset>,
    downloads: i64,
}

#[derive(Debug, FromQueryResult)]
struct DetailRow {
    id: i64,
    created_by: Option<String>,
    paths: Vec<String>,
}

async fn usage(
    State(state): State<AppState>,
    caller: Caller,
    Query(query): Query<UsageQuery>,
) -> Result<axum::Json<Usage>, ApiError> {
    let survey = survey(&state, &caller).await?;
    if let Err(error) = state.content().flush_downloads().await {
        tracing::warn!(%error, "pending downloads were left out of the usage figures");
    }
    let days = query
        .days
        .unwrap_or(DEFAULT_DAYS)
        .clamp(1, i64::from(DOWNLOAD_DAYS_KEPT));
    let wanted = query
        .repository
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty());
    let mut visible: Vec<&Seen> = survey
        .seen
        .iter()
        .filter(|seen| seen.reach.visible())
        .collect();
    visible.sort_by(|left, right| left.repository.name.cmp(&right.repository.name));
    let selected: Vec<&Seen> = match &wanted {
        Some(name) => visible
            .iter()
            .copied()
            .filter(|seen| &seen.repository.name == name)
            .collect(),
        None => visible.clone(),
    };
    if wanted.is_some() && selected.is_empty() {
        return Err(ApiError::not_found("Repository not found"));
    }

    let today = Utc::now().date_naive();
    let from = today
        .checked_sub_days(Days::new(u64::try_from(days - 1).unwrap_or_default()))
        .unwrap_or(today);
    let figures = Figures::read(&state, &selected, from, today).await?;
    Ok(axum::Json(figures.into_usage(
        days, from, today, wanted, &visible, &selected,
    )))
}

#[derive(Default)]
struct Figures {
    content: Vec<ContentRow>,
    days: Vec<DayRow>,
    top: Vec<ComponentRow>,
    top_all_time: Vec<ComponentRow>,
    recent: Vec<ComponentRow>,
    details: HashMap<i64, DetailRow>,
}

const COMPONENT_COLUMNS: &str =
    "c.id, c.repository, c.format, c.namespace, c.name, c.version, c.created_at";

impl Figures {
    async fn read(
        state: &AppState,
        selected: &[&Seen],
        from: NaiveDate,
        today: NaiveDate,
    ) -> Result<Self, ApiError> {
        let db = state.db().conn();
        let mut figures = Figures::default();
        let mut sql = Sql::default();
        let Some(condition) = sql.visible(selected.iter().copied()) else {
            return Ok(figures);
        };
        let statement = format!(
            "SELECT repository, \
               count(component_id)::bigint AS components, \
               coalesce(sum(assets), 0)::bigint AS assets, \
               coalesce(sum(bytes), 0)::bigint AS bytes, \
               coalesce(sum(downloads), 0)::bigint AS downloads, \
               count(component_id) FILTER (WHERE last_used < now() - interval '{UNUSED_DAYS} days')::bigint AS unused_components, \
               coalesce(sum(bytes) FILTER (WHERE component_id IS NOT NULL AND last_used < now() - interval '{UNUSED_DAYS} days'), 0)::bigint AS unused_bytes, \
               count(component_id) FILTER (WHERE last_downloaded IS NULL)::bigint AS never_components, \
               coalesce(sum(bytes) FILTER (WHERE component_id IS NOT NULL AND last_downloaded IS NULL), 0)::bigint AS never_bytes \
             FROM (SELECT a.repository, a.component_id, count(*) AS assets, sum(a.size) AS bytes, \
                     sum(a.download_count) AS downloads, max(a.last_downloaded_at) AS last_downloaded, \
                     max(coalesce(a.last_downloaded_at, a.created_at)) AS last_used \
                   FROM asset a WHERE {condition} GROUP BY a.repository, a.component_id) AS per_component \
             GROUP BY repository"
        );
        figures.content = ContentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;

        let mut sql = Sql::default();
        let condition = sql.visible(selected.iter().copied()).unwrap_or_default();
        let first = sql.bind(from.to_string());
        let last = sql.bind(today.to_string());
        let statement = format!(
            "SELECT a.repository, d.day::text AS day, sum(d.downloads)::bigint AS downloads, \
               count(DISTINCT a.component_id)::bigint AS components, \
               GROUPING(a.repository, d.day) AS total \
             FROM asset_download_day d JOIN asset a ON a.id = d.asset_id \
             WHERE d.day BETWEEN {first}::date AND {last}::date AND {condition} \
             GROUP BY GROUPING SETS ((a.repository, d.day), ())"
        );
        figures.days = DayRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;

        let mut sql = Sql::default();
        let condition = sql.visible(selected.iter().copied()).unwrap_or_default();
        let first = sql.bind(from.to_string());
        let statement = format!(
            "SELECT {COMPONENT_COLUMNS}, sum(d.downloads)::bigint AS downloads \
             FROM asset_download_day d JOIN asset a ON a.id = d.asset_id \
             JOIN component c ON c.id = a.component_id \
             WHERE d.day >= {first}::date AND {condition} \
             GROUP BY c.id ORDER BY downloads DESC, c.id LIMIT {TOP}"
        );
        figures.top = ComponentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;

        let mut sql = Sql::default();
        let condition = sql.visible(selected.iter().copied()).unwrap_or_default();
        let statement = format!(
            "SELECT {COMPONENT_COLUMNS}, sum(a.download_count)::bigint AS downloads \
             FROM asset a JOIN component c ON c.id = a.component_id \
             WHERE {condition} GROUP BY c.id HAVING sum(a.download_count) > 0 \
             ORDER BY downloads DESC, c.id LIMIT {TOP}"
        );
        figures.top_all_time = ComponentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;

        let mut sql = Sql::default();
        let condition = sql.visible(selected.iter().copied()).unwrap_or_default();
        let names = sql.bind(
            selected
                .iter()
                .map(|seen| seen.repository.name.clone())
                .collect::<Vec<_>>(),
        );
        let statement = format!(
            "SELECT {COMPONENT_COLUMNS}, 0::bigint AS downloads FROM component c \
             WHERE c.repository = ANY({names}) \
               AND EXISTS (SELECT 1 FROM asset a WHERE a.component_id = c.id AND {condition}) \
             ORDER BY c.id DESC LIMIT {TOP}"
        );
        figures.recent = ComponentRow::find_by_statement(sql.statement(&statement))
            .all(db)
            .await?;

        let wanted: Vec<i64> = figures
            .top
            .iter()
            .chain(&figures.top_all_time)
            .chain(&figures.recent)
            .map(|row| row.id)
            .collect();
        if !wanted.is_empty() {
            let mut sql = Sql::default();
            let condition = sql.visible(selected.iter().copied()).unwrap_or_default();
            let ids = sql.bind(wanted);
            let statement = format!(
                "SELECT a.component_id AS id, min(a.uploader) FILTER (WHERE a.uploader <> '') AS created_by, \
                   (array_agg(a.path ORDER BY a.path))[1:{PATHS}] AS paths \
                 FROM asset a WHERE a.component_id = ANY({ids}) AND {condition} \
                 GROUP BY a.component_id"
            );
            figures.details = DetailRow::find_by_statement(sql.statement(&statement))
                .all(db)
                .await?
                .into_iter()
                .map(|row| (row.id, row))
                .collect();
        }
        Ok(figures)
    }

    fn into_usage(
        self,
        days: i64,
        from: NaiveDate,
        today: NaiveDate,
        repository: Option<String>,
        visible: &[&Seen],
        selected: &[&Seen],
    ) -> Usage {
        let kinds: HashMap<&str, (&str, RepositoryType)> = selected
            .iter()
            .map(|seen| {
                (
                    seen.repository.name.as_str(),
                    (seen.repository.format.id, seen.repository.kind),
                )
            })
            .collect();
        let mut totals = Totals {
            repositories: selected.len(),
            ..Totals::default()
        };
        for seen in selected {
            match seen.repository.kind {
                RepositoryType::Hosted => totals.hosted += 1,
                RepositoryType::Proxy => totals.proxy += 1,
                RepositoryType::Group => totals.group += 1,
            }
        }

        let mut per_day: BTreeMap<String, i64> = BTreeMap::new();
        let mut per_repository: HashMap<String, i64> = HashMap::new();
        for row in &self.days {
            if row.total != 0 {
                totals.components_downloaded_in_period = row.components;
                continue;
            }
            let (Some(repository), Some(day)) = (&row.repository, &row.day) else {
                continue;
            };
            let downloads = row.downloads.unwrap_or_default();
            *per_day.entry(day.clone()).or_default() += downloads;
            *per_repository.entry(repository.clone()).or_default() += downloads;
        }
        let mut downloads_per_day = Vec::new();
        for day in from.iter_days().take_while(|day| *day <= today) {
            let key = day.to_string();
            downloads_per_day.push(Day {
                downloads: per_day.get(&key).copied().unwrap_or_default(),
                day: key,
            });
        }
        totals.downloads_in_period = downloads_per_day.iter().map(|day| day.downloads).sum();
        totals.downloads_today = downloads_per_day
            .last()
            .map(|day| day.downloads)
            .unwrap_or_default();

        let mut unused = Unused::default();
        let mut usage: Vec<RepositoryUsage> = Vec::new();
        for row in &self.content {
            totals.components += row.components;
            totals.assets += row.assets;
            totals.storage_bytes += row.bytes;
            totals.downloads += row.downloads;
            unused.components += row.unused_components;
            unused.bytes += row.unused_bytes;
            unused.never_downloaded_components += row.never_components;
            unused.never_downloaded_bytes += row.never_bytes;
            let Some((format, kind)) = kinds.get(row.repository.as_str()) else {
                continue;
            };
            if *kind == RepositoryType::Proxy {
                totals.proxy_storage_bytes += row.bytes;
            }
            if row.unused_components > 0 {
                unused.repositories.push(UnusedRepository {
                    name: row.repository.clone(),
                    format: (*format).to_owned(),
                    components: row.unused_components,
                    bytes: row.unused_bytes,
                });
            }
        }
        unused.repositories.sort_by(|left, right| {
            right
                .bytes
                .cmp(&left.bytes)
                .then(right.components.cmp(&left.components))
                .then(left.name.cmp(&right.name))
        });
        unused.repositories.truncate(UNUSED_TOP);

        let content: HashMap<&str, &ContentRow> = self
            .content
            .iter()
            .map(|row| (row.repository.as_str(), row))
            .collect();
        for seen in selected {
            let name = seen.repository.name.as_str();
            let stored = content.get(name);
            usage.push(RepositoryUsage {
                name: name.to_owned(),
                format: seen.repository.format.id.to_owned(),
                kind: seen.repository.kind.as_str(),
                downloads: per_repository.get(name).copied().unwrap_or_default(),
                storage_bytes: stored.map(|row| row.bytes).unwrap_or_default(),
                components: stored.map(|row| row.components).unwrap_or_default(),
            });
        }
        let ranked = |key: fn(&RepositoryUsage) -> i64| {
            let mut ranked: Vec<&RepositoryUsage> =
                usage.iter().filter(|entry| key(entry) > 0).collect();
            ranked
                .sort_by(|left, right| key(right).cmp(&key(left)).then(left.name.cmp(&right.name)));
            ranked.into_iter().take(TOP).cloned().collect::<Vec<_>>()
        };
        let top_repositories_by_downloads = ranked(|entry| entry.downloads);
        let top_repositories_by_storage = ranked(|entry| entry.storage_bytes);

        let mut recent = self.recent;
        recent.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then(right.id.cmp(&left.id))
        });
        let details = &self.details;
        let listed = |rows: Vec<ComponentRow>| -> Vec<UsageComponent> {
            rows.into_iter()
                .map(|row| {
                    let detail = details.get(&row.id);
                    component(row, detail)
                })
                .collect()
        };

        Usage {
            days,
            from: from.to_string(),
            to: today.to_string(),
            retention_days: DOWNLOAD_DAYS_KEPT,
            unused_days: UNUSED_DAYS,
            repository,
            repositories: visible
                .iter()
                .map(|seen| RepositoryRef {
                    name: seen.repository.name.clone(),
                    format: seen.repository.format.id.to_owned(),
                    kind: seen.repository.kind.as_str(),
                })
                .collect(),
            totals,
            downloads_per_day,
            top_components: listed(self.top),
            top_components_all_time: listed(self.top_all_time),
            top_repositories_by_downloads,
            top_repositories_by_storage,
            recent_components: listed(recent),
            unused,
        }
    }
}

fn component(row: ComponentRow, detail: Option<&DetailRow>) -> UsageComponent {
    UsageComponent {
        id: ids::encode(&row.repository, row.id),
        group: Some(row.namespace).filter(|value| !value.is_empty()),
        version: Some(row.version).filter(|value| !value.is_empty()),
        downloads: row.downloads,
        created: time(row.created_at),
        created_by: detail.and_then(|detail| detail.created_by.clone()),
        browse_path: detail.and_then(|detail| browse_path(&detail.paths)),
        repository: row.repository,
        format: row.format,
        name: row.name,
    }
}

fn parent(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(folder, _)| folder)
        .unwrap_or("")
}

fn browse_path(paths: &[String]) -> Option<String> {
    let first = paths.first()?;
    let folder = parent(first).trim_start_matches('/');
    if paths.len() > 1
        && !folder.is_empty()
        && paths.iter().all(|path| parent(path) == parent(first))
    {
        return Some(format!("{folder}/"));
    }
    Some(first.trim_start_matches('/').to_owned())
}

#[cfg(test)]
mod tests {
    use super::browse_path;

    fn paths(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn a_single_file_is_its_own_node() {
        assert_eq!(
            browse_path(&paths(&["/docs/guide.pdf"])),
            Some("docs/guide.pdf".to_owned())
        );
        assert_eq!(browse_path(&[]), None);
    }

    #[test]
    fn files_sharing_a_folder_point_at_the_folder() {
        assert_eq!(
            browse_path(&paths(&[
                "/org/acme/lib/1.0/lib-1.0.jar",
                "/org/acme/lib/1.0/lib-1.0.pom"
            ])),
            Some("org/acme/lib/1.0/".to_owned())
        );
        assert_eq!(
            browse_path(&paths(&["/a/x.txt", "/b/y.txt"])),
            Some("a/x.txt".to_owned())
        );
        assert_eq!(
            browse_path(&paths(&["/x.txt", "/y.txt"])),
            Some("x.txt".to_owned())
        );
    }
}
