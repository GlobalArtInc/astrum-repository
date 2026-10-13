use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use axum::Router;
use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::{Datelike, Timelike, Utc};
use flate2::Compression;
use flate2::write::DeflateEncoder;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::routes::support::{health, sysinfo};
use crate::{ApiError, AppState, Caller, Json};

pub const READ: &str = "astrum:atlas:read";
pub const DOWNLOADS: &str = "downloads";
const DOWNLOAD_PATH: &str = "/service/rest/wonderland/download";
const FILE_LIMIT: u64 = 30 * 1024 * 1024;
const ZIP_LIMIT: u64 = 50 * 1024 * 1024;
const KEEP: Duration = Duration::from_secs(24 * 60 * 60);
const AUDIT_EVENTS: u64 = 10_000;
const TASK_LOG_DAYS: i32 = 7;
const TASK_LOG_LINES: u64 = 50_000;

static SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/service/rest/v1/support/supportzip", post(stream))
        .route("/service/rest/v1/support/supportzippath", post(to_path))
        .route(&format!("{DOWNLOAD_PATH}/{{file}}"), get(download))
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    pub system_information: bool,
    pub configuration: bool,
    pub security: bool,
    pub log: bool,
    pub task_log: bool,
    pub audit_log: bool,
    pub metrics: bool,
    pub archived_log: u32,
    pub limit_file_sizes: bool,
    pub limit_zip_size: bool,
    pub thread_dump: bool,
    pub jmx: bool,
    pub replication: bool,
}

#[derive(Serialize)]
struct Created {
    file: String,
    name: String,
    size: u64,
    truncated: bool,
}

async fn to_path(
    State(state): State<AppState>,
    caller: Caller,
    Json(options): Json<Options>,
) -> Result<axum::Json<Created>, ApiError> {
    caller.require(READ)?;
    let dir = state.data_dir().join(DOWNLOADS);
    let name = format!(
        "support-{}-{}.zip",
        Utc::now().format("%Y%m%d-%H%M%S"),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let path = dir.join(&name);
    let entries = gather(&state, &options).await;
    let (size, truncated) = {
        let path = path.clone();
        tokio::task::spawn_blocking(move || write_file(&dir, &path, entries, &options))
            .await
            .map_err(ApiError::internal)?
            .map_err(ApiError::internal)?
    };
    tracing::info!(file = %path.display(), size, truncated, "created a support ZIP");
    Ok(axum::Json(Created {
        file: path.display().to_string(),
        name,
        size,
        truncated,
    }))
}

async fn stream(
    State(state): State<AppState>,
    caller: Caller,
    Json(options): Json<Options>,
) -> Result<Response, ApiError> {
    caller.require(READ)?;
    let entries = gather(&state, &options).await;
    let bytes = tokio::task::spawn_blocking(move || {
        let mut zip = ZipWriter::new(Vec::new());
        add_all(&mut zip, entries, &options)?;
        zip.finish()
    })
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
    let name = format!("support-{}.zip", Utc::now().format("%Y%m%d-%H%M%S"));
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/zip".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{name}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn download(
    State(state): State<AppState>,
    caller: Caller,
    UrlPath(file): UrlPath<String>,
) -> Result<Response, ApiError> {
    caller.require(READ)?;
    let missing = || ApiError::not_found(format!("File {file} not found"));
    if !valid_download(&file) {
        return Err(missing());
    }
    let path = state.data_dir().join(DOWNLOADS).join(&file);
    let handle = match tokio::fs::File::open(&path).await {
        Ok(handle) => handle,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(missing()),
        Err(error) => return Err(ApiError::internal(error)),
    };
    let size = handle.metadata().await.map_err(ApiError::internal)?.len();
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/zip".to_owned()),
            (header::CONTENT_LENGTH, size.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{file}\""),
            ),
        ],
        Body::from_stream(tokio_util::io::ReaderStream::new(handle)),
    )
        .into_response())
}

fn valid_download(name: &str) -> bool {
    name.starts_with("support-")
        && name.ends_with(".zip")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        && !name.contains("..")
}

enum Source {
    Bytes(Vec<u8>),
    File(PathBuf),
}

struct Entry {
    name: String,
    source: Source,
}

fn json_entry(name: &str, value: &Value) -> Entry {
    Entry {
        name: name.to_owned(),
        source: Source::Bytes(serde_json::to_vec_pretty(value).unwrap_or_default()),
    }
}

async fn gather(state: &AppState, options: &Options) -> Vec<Entry> {
    let mut entries = Vec::new();
    if options.system_information {
        entries.push(json_entry(
            "info/sysinfo.json",
            &sysinfo::collect(state).await,
        ));
    }
    if options.configuration {
        entries.push(json_entry("info/configuration.json", &configuration(state)));
    }
    if options.security {
        entries.push(json_entry("info/security.json", &security(state)));
    }
    if options.metrics {
        let checks = health::run(state).await;
        entries.push(json_entry(
            "info/health.json",
            &serde_json::to_value(checks).unwrap_or_default(),
        ));
    }
    if options.log {
        entries.extend(log_files(state, options.archived_log));
    }
    if options.task_log {
        entries.push(Entry {
            name: "log/tasks.log".to_owned(),
            source: Source::Bytes(task_logs(state).await),
        });
    }
    if options.audit_log {
        entries.push(Entry {
            name: "log/audit.jsonl".to_owned(),
            source: Source::Bytes(audit_events(state).await),
        });
    }
    entries
}

fn configuration(state: &AppState) -> Value {
    let entries: serde_json::Map<String, Value> = state
        .configuration()
        .iter()
        .map(|(key, value)| (key.clone(), json!(value)))
        .collect();
    json!({
        "configuration": entries,
        "logging": {
            "filter": state.logging().filter(),
            "overrides": state.logging().overrides(),
        },
    })
}

fn security(state: &AppState) -> Value {
    let model = state.security().model();
    json!({
        "realms": model.realms,
        "anonymous": {
            "enabled": model.anonymous.enabled,
            "userId": model.anonymous.user_id,
            "realmName": model.anonymous.realm_name,
        },
        "defaultRole": model.default_role,
        "sessionTimeoutMinutes": model.session_timeout_minutes,
        "users": model.users.len(),
        "roles": model.roles.len(),
        "privileges": model.privileges.len(),
        "contentSelectors": model.selectors.len(),
        "ldapServers": model.ldap_servers.iter().map(|server| &server.name).collect::<Vec<_>>(),
        "oidcConfigured": model.oidc.is_some(),
    })
}

fn log_files(state: &AppState, archived_days: u32) -> Vec<Entry> {
    let Some(files) = state.logging().files() else {
        return Vec::new();
    };
    let Ok(listed) = files.list() else {
        return Vec::new();
    };
    let since = SystemTime::now() - Duration::from_secs(u64::from(archived_days) * 24 * 60 * 60);
    listed
        .into_iter()
        .filter(|file| {
            !file
                .name
                .rsplit('.')
                .next()
                .is_some_and(|part| part.chars().all(|c| c.is_ascii_digit()))
                || (archived_days > 0 && file.modified.is_some_and(|modified| modified >= since))
        })
        .filter_map(|file| {
            files.path_of(&file.name).map(|path| Entry {
                name: format!("log/{}", file.name),
                source: Source::File(path),
            })
        })
        .collect()
}

#[derive(FromQueryResult)]
struct Line {
    line: String,
}

async fn lines(state: &AppState, sql: String) -> Vec<u8> {
    let rows = Line::find_by_statement(Statement::from_string(DbBackend::Postgres, sql))
        .all(state.db().conn())
        .await;
    match rows {
        Ok(rows) => {
            let mut out = Vec::new();
            for row in rows {
                out.extend_from_slice(row.line.as_bytes());
                out.push(b'\n');
            }
            out
        }
        Err(error) => format!("cannot read: {error}\n").into_bytes(),
    }
}

async fn audit_events(state: &AppState) -> Vec<u8> {
    lines(
        state,
        format!(
            "SELECT row_to_json(e)::text AS line FROM \
             (SELECT * FROM audit_event ORDER BY id DESC LIMIT {AUDIT_EVENTS}) e ORDER BY e.id"
        ),
    )
    .await
}

async fn task_logs(state: &AppState) -> Vec<u8> {
    lines(
        state,
        format!(
            "SELECT to_char(l.at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') || ' ' || \
             upper(l.level) || ' [' || t.name || ' / ' || t.type || ' run ' || r.id || '] ' || \
             l.message AS line \
             FROM task_log l JOIN task_run r ON r.id = l.run_id JOIN task t ON t.id = r.task_id \
             WHERE l.at > now() - interval '{TASK_LOG_DAYS} days' \
             ORDER BY l.at, l.run_id, l.line LIMIT {TASK_LOG_LINES}"
        ),
    )
    .await
}

fn write_file(
    dir: &Path,
    path: &Path,
    entries: Vec<Entry>,
    options: &Options,
) -> io::Result<(u64, bool)> {
    std::fs::create_dir_all(dir)?;
    prune(dir);
    let file = std::fs::File::create(path)?;
    let mut zip = ZipWriter::new(io::BufWriter::new(file));
    let truncated = add_all(&mut zip, entries, options)?;
    let mut out = zip.finish()?;
    out.flush()?;
    let size = std::fs::metadata(path)?.len();
    Ok((size, truncated))
}

fn prune(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let old = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age > KEEP);
        if old && name.to_str().is_some_and(valid_download) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn add_all<W: Write>(
    zip: &mut ZipWriter<W>,
    entries: Vec<Entry>,
    options: &Options,
) -> io::Result<bool> {
    let mut truncated = false;
    for entry in entries {
        let mut bytes = match entry.source {
            Source::Bytes(bytes) => bytes,
            Source::File(path) => {
                match read_tail(&path, options.limit_file_sizes.then_some(FILE_LIMIT)) {
                    Ok((bytes, cut)) => {
                        truncated |= cut;
                        bytes
                    }
                    Err(error) => format!("cannot read {}: {error}\n", path.display()).into_bytes(),
                }
            }
        };
        if options.limit_file_sizes && bytes.len() as u64 > FILE_LIMIT {
            bytes.drain(..bytes.len() - FILE_LIMIT as usize);
            truncated = true;
        }
        if options.limit_zip_size && zip.written() + bytes.len() as u64 / 4 > ZIP_LIMIT {
            truncated = true;
            continue;
        }
        zip.add(&entry.name, &bytes)?;
    }
    Ok(truncated)
}

fn read_tail(path: &Path, limit: Option<u64>) -> io::Result<(Vec<u8>, bool)> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    let start = match limit {
        Some(limit) if size > limit => size - limit,
        _ => 0,
    };
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::with_capacity((size - start) as usize);
    file.read_to_end(&mut bytes)?;
    Ok((bytes, start > 0))
}

pub struct ZipWriter<W: Write> {
    out: W,
    offset: u64,
    central: Vec<u8>,
    count: u16,
    time: u16,
    date: u16,
}

impl<W: Write> ZipWriter<W> {
    pub fn new(out: W) -> Self {
        let now = Utc::now();
        let time = ((now.hour() << 11) | (now.minute() << 5) | (now.second() / 2)) as u16;
        let year = now.year().clamp(1980, 2107) as u32 - 1980;
        let date = ((year << 9) | (now.month() << 5) | now.day()) as u16;
        Self {
            out,
            offset: 0,
            central: Vec::new(),
            count: 0,
            time,
            date,
        }
    }

    pub fn written(&self) -> u64 {
        self.offset
    }

    pub fn add(&mut self, name: &str, bytes: &[u8]) -> io::Result<()> {
        let too_big = || io::Error::other("a ZIP entry over 4 GiB needs ZIP64");
        let mut crc = flate2::Crc::new();
        crc.update(bytes);
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes)?;
        let compressed = encoder.finish()?;
        let size = u32::try_from(bytes.len()).map_err(|_| too_big())?;
        let packed = u32::try_from(compressed.len()).map_err(|_| too_big())?;
        let offset = u32::try_from(self.offset).map_err(|_| too_big())?;
        let name = name.as_bytes();
        let mut local = Vec::with_capacity(30 + name.len());
        local.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        self.common(&mut local, crc.sum(), packed, size, name.len() as u16);
        local.extend_from_slice(&0u16.to_le_bytes());
        local.extend_from_slice(name);
        self.out.write_all(&local)?;
        self.out.write_all(&compressed)?;
        let mut central = Vec::with_capacity(46 + name.len());
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&(3u16 << 8 | 20).to_le_bytes());
        self.common(&mut central, crc.sum(), packed, size, name.len() as u16);
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&(0o100_644u32 << 16).to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
        self.central.extend_from_slice(&central);
        self.offset += (local.len() + compressed.len()) as u64;
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| io::Error::other("too many ZIP entries"))?;
        Ok(())
    }

    fn common(&self, out: &mut Vec<u8>, crc: u32, packed: u32, size: u32, name: u16) {
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0x0800u16.to_le_bytes());
        out.extend_from_slice(&8u16.to_le_bytes());
        out.extend_from_slice(&self.time.to_le_bytes());
        out.extend_from_slice(&self.date.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&packed.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&name.to_le_bytes());
    }

    pub fn finish(mut self) -> io::Result<W> {
        let start =
            u32::try_from(self.offset).map_err(|_| io::Error::other("the ZIP needs ZIP64"))?;
        self.out.write_all(&self.central)?;
        let mut end = Vec::with_capacity(22);
        end.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        end.extend_from_slice(&self.count.to_le_bytes());
        end.extend_from_slice(&self.count.to_le_bytes());
        end.extend_from_slice(&(self.central.len() as u32).to_le_bytes());
        end.extend_from_slice(&start.to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        self.out.write_all(&end)?;
        Ok(self.out)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn writes_entries_a_reader_can_inflate() {
        let mut zip = ZipWriter::new(Vec::new());
        zip.add("info/a.json", b"{\"a\":1}").unwrap();
        zip.add("log/b.log", &b"line\n".repeat(1000)).unwrap();
        let bytes = zip.finish().unwrap();
        let end = &bytes[bytes.len() - 22..];
        assert_eq!(&end[..4], &0x0605_4b50u32.to_le_bytes());
        assert_eq!(u16::from_le_bytes([end[10], end[11]]), 2);
        let central = u32::from_le_bytes([end[16], end[17], end[18], end[19]]) as usize;
        assert_eq!(&bytes[central..central + 4], &0x0201_4b50u32.to_le_bytes());
        assert_eq!(&bytes[..4], &0x0403_4b50u32.to_le_bytes());
        let packed = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]) as usize;
        let name = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
        let data = &bytes[30 + name..30 + name + packed];
        let mut text = String::new();
        flate2::read::DeflateDecoder::new(data)
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "{\"a\":1}");
    }

    #[test]
    fn accepts_only_support_zip_names_for_download() {
        assert!(valid_download("support-20261013-175727-1.zip"));
        assert!(!valid_download("../support-1.zip"));
        assert!(!valid_download("support-1.zip/../../x"));
        assert!(!valid_download("secrets.key"));
    }
}
