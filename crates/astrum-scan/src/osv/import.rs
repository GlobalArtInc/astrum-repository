use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::{Arc, Mutex};

use astrum_content::zip::{self, Source, ZipError};
use async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, RuntimeErr, Statement, TransactionTrait,
};
use serde_json::{Value, json};

use super::record::{self, Record};

pub const MAX_ENTRIES: usize = 1_000_000;
pub const MAX_ENTRY_BYTES: u64 = 1024 * 1024;
pub const MAX_COMPRESSED_ENTRY_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_INFLATED_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const BATCH: usize = 1000;
const BATCH_BYTES: usize = 8 * 1024 * 1024;
const MAX_READ: u64 = 64 * 1024 * 1024;
pub const MAX_UNFORCED_REMOVALS: usize = 100;
const MAX_UNFORCED_PERCENT: usize = 20;
const REJECTED_CLASSES: [&str; 3] = ["22", "23", "54"];
const LOCK: &str = "SELECT pg_advisory_xact_lock(hashtext('advisories.import'))";

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("the archive cannot be read: {0}")]
    Archive(String),
    #[error("the archive lists more than {MAX_ENTRIES} entries")]
    TooManyEntries,
    #[error("the archive inflates to more than {} bytes", MAX_INFLATED_BYTES)]
    TooMuchData,
    #[error("the archive holds no readable advisory")]
    Empty,
    #[error("the archive holds no advisory of the ecosystem {0}; nothing was removed")]
    OutOfScope(String),
    #[error(
        "the import would remove {removing} of the {held} advisories of the ecosystem, more than {MAX_UNFORCED_PERCENT} percent and more than {MAX_UNFORCED_REMOVALS} records; nothing was removed; repeat a manual import with force=true to allow it"
    )]
    Shrinks { removing: usize, held: usize },
    #[error("the import was stopped")]
    Cancelled,
    #[error("the database failed: {0}")]
    Database(#[from] DbErr),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub entries: u64,
    pub records: u64,
    pub changed: u64,
    pub unchanged: u64,
    pub skipped: u64,
    pub removed: u64,
    pub invalidated: u64,
}

struct FileSource {
    file: Arc<Mutex<File>>,
    size: u64,
}

impl FileSource {
    fn open(path: &Path) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            size,
        })
    }
}

#[async_trait]
impl Source for FileSource {
    fn size(&self) -> u64 {
        self.size
    }

    async fn read(&self, offset: u64, length: u64) -> Result<Vec<u8>, ZipError> {
        if length > MAX_READ {
            return Err(ZipError::Invalid("a read of the archive is too large"));
        }
        let available = self.size.saturating_sub(offset);
        let length = length.min(available) as usize;
        let file = self.file.clone();
        tokio::task::spawn_blocking(move || {
            let mut file = file.lock().expect("the archive lock is never poisoned");
            file.seek(SeekFrom::Start(offset))?;
            let mut buffer = vec![0u8; length];
            file.read_exact(&mut buffer)?;
            Ok(buffer)
        })
        .await
        .map_err(|error| ZipError::Io(std::io::Error::other(error)))?
    }
}

#[derive(Default)]
struct Batch {
    records: Vec<Record>,
    bytes: usize,
}

impl Batch {
    fn push(&mut self, record: Record, bytes: usize) {
        self.records.push(record);
        self.bytes += bytes;
    }

    fn full(&self) -> bool {
        self.records.len() >= BATCH || self.bytes >= BATCH_BYTES
    }
}

const SCOPE: &str = "(f.ecosystem = $1 OR left(f.ecosystem, length($1) + 1) = $1 || ':')";

fn in_scope(requested: &str, found: &str) -> bool {
    found == requested
        || found
            .strip_prefix(requested)
            .is_some_and(|rest| rest.starts_with(':'))
}

fn rejected(error: &DbErr) -> bool {
    let (DbErr::Exec(RuntimeErr::SqlxError(error)) | DbErr::Query(RuntimeErr::SqlxError(error))) =
        error
    else {
        return false;
    };
    error
        .as_database_error()
        .and_then(|error| error.code())
        .is_some_and(|code| REJECTED_CLASSES.iter().any(|class| code.starts_with(class)))
}

pub async fn import_archive(
    db: &DatabaseConnection,
    ecosystem: &str,
    path: &Path,
    force: bool,
    cancelled: &(dyn Fn() -> bool + Send + Sync),
    stats: &mut Stats,
) -> Result<(), ImportError> {
    let source = FileSource::open(path).map_err(|error| ImportError::Archive(error.to_string()))?;
    let entries = zip::entries(&source)
        .await
        .map_err(|error| ImportError::Archive(error.to_string()))?;
    if entries.len() > MAX_ENTRIES {
        return Err(ImportError::TooManyEntries);
    }
    let mut offsets: HashSet<u64> = HashSet::new();
    if !entries.iter().all(|entry| offsets.insert(entry.offset())) {
        return Err(ImportError::Archive(
            "several entries of the archive share one local header".to_owned(),
        ));
    }
    drop(offsets);
    let mut seen: HashSet<String> = HashSet::new();
    let mut scoped: HashSet<String> = HashSet::new();
    let mut batch = Batch::default();
    let mut inflated = 0u64;
    for entry in entries {
        if cancelled() {
            return Err(ImportError::Cancelled);
        }
        if entry.is_directory() {
            continue;
        }
        stats.entries += 1;
        let Some(stem) = record::entry_stem(&entry.name) else {
            stats.skipped += 1;
            continue;
        };
        if !seen.insert(stem.to_owned()) {
            stats.skipped += 1;
            continue;
        }
        if entry.size > MAX_ENTRY_BYTES || entry.compressed > MAX_COMPRESSED_ENTRY_BYTES {
            stats.skipped += 1;
            continue;
        }
        let read = zip::read(&source, &entry, MAX_ENTRY_BYTES).await;
        inflated += match &read {
            Ok(bytes) => bytes.len() as u64,
            Err(_) => MAX_ENTRY_BYTES,
        };
        if inflated > MAX_INFLATED_BYTES {
            return Err(ImportError::TooMuchData);
        }
        let bytes = match read {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::debug!(%error, "an advisory entry cannot be read");
                stats.skipped += 1;
                continue;
            }
        };
        match record::parse(&bytes, stem) {
            Ok(record) => {
                stats.records += 1;
                if record
                    .affected
                    .iter()
                    .any(|package| in_scope(ecosystem, &package.ecosystem))
                {
                    scoped.insert(record.id.clone());
                }
                batch.push(record, bytes.len());
            }
            Err(reason) => {
                tracing::debug!(reason, "an advisory record is skipped");
                stats.skipped += 1;
            }
        }
        if batch.full() {
            for id in flush(db, &mut batch, stats).await? {
                scoped.remove(&id);
            }
        }
    }
    for id in flush(db, &mut batch, stats).await? {
        scoped.remove(&id);
    }
    if stats.records == 0 {
        return Err(ImportError::Empty);
    }
    if scoped.is_empty() {
        return Err(ImportError::OutOfScope(ecosystem.to_owned()));
    }
    remove_vanished(db, ecosystem, &seen, force, stats).await?;
    Ok(())
}

fn json_param(value: Value) -> sea_orm::Value {
    value.into()
}

async fn flush(
    db: &DatabaseConnection,
    batch: &mut Batch,
    stats: &mut Stats,
) -> Result<Vec<String>, ImportError> {
    let records = std::mem::take(&mut batch.records);
    batch.bytes = 0;
    if records.is_empty() {
        return Ok(Vec::new());
    }
    match write(db, &records, stats).await {
        Ok(()) => Ok(Vec::new()),
        Err(error) if rejected(&error) => {
            tracing::warn!(%error, "the database refused a batch of advisories, retrying record by record");
            let mut failed = Vec::new();
            for record in &records {
                match write(db, std::slice::from_ref(record), stats).await {
                    Ok(()) => {}
                    Err(error) if rejected(&error) => {
                        tracing::warn!(%error, id = %record.id, "the database refused an advisory, it is skipped");
                        stats.records -= 1;
                        stats.skipped += 1;
                        failed.push(record.id.clone());
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            Ok(failed)
        }
        Err(error) => Err(error.into()),
    }
}

async fn write(
    db: &DatabaseConnection,
    records: &[Record],
    stats: &mut Stats,
) -> Result<(), DbErr> {
    let listed: Vec<Value> = records
        .iter()
        .map(|record| json!({"id": record.id, "modified": record.modified}))
        .collect();
    let unchanged: HashSet<String> = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT r.id FROM jsonb_to_recordset($1::jsonb) AS r(id text, modified text) \
             JOIN advisory a ON a.id = r.id AND a.modified_at = r.modified::timestamptz",
            [json_param(Value::Array(listed))],
        ))
        .await?
        .iter()
        .map(|row| row.try_get::<String>("", "id"))
        .collect::<Result<_, _>>()?;
    let changed: Vec<&Record> = records
        .iter()
        .filter(|record| !unchanged.contains(&record.id))
        .collect();
    if changed.is_empty() {
        stats.unchanged += unchanged.len() as u64;
        return Ok(());
    }
    let rows: Vec<Value> = changed
        .iter()
        .map(|record| {
            json!({
                "id": record.id,
                "aliases": record.aliases,
                "summary": record.summary,
                "severity": record.severity,
                "score": record.score,
                "vector": record.vector,
                "malicious": record.malicious,
                "published": record.published,
                "modified": record.modified,
                "withdrawn": record.withdrawn,
                "document": record.document,
            })
        })
        .collect();
    let affected: Vec<Value> = changed
        .iter()
        .flat_map(|record| {
            record.affected.iter().map(|package| {
                json!({
                    "advisory_id": record.id,
                    "ecosystem": package.ecosystem,
                    "name": package.name,
                    "ranges": package.ranges,
                    "versions": package.versions,
                })
            })
        })
        .collect();
    let ids: Vec<Value> = changed
        .iter()
        .map(|record| json!({"id": record.id}))
        .collect();
    let (rows, affected, ids) = (
        json_param(Value::Array(rows)),
        json_param(Value::Array(affected)),
        json_param(Value::Array(ids)),
    );
    let txn = db.begin().await?;
    txn.execute(Statement::from_string(DbBackend::Postgres, LOCK))
        .await?;
    let invalidated = txn
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "UPDATE sca_package SET matched_at = NULL WHERE id IN ( \
               SELECT p.id FROM sca_package p \
               WHERE p.matched_at IS NOT NULL AND (p.ecosystem, p.name) IN ( \
                  SELECT f.ecosystem, f.name FROM advisory_affected f \
                  WHERE f.advisory_id IN (SELECT i.id FROM jsonb_to_recordset($1::jsonb) AS i(id text)) \
                  UNION \
                  SELECT n.ecosystem, n.name FROM jsonb_to_recordset($2::jsonb) AS n(ecosystem text, name text)) \
               ORDER BY p.id FOR UPDATE OF p)",
            [ids.clone(), affected.clone()],
        ))
        .await?
        .rows_affected();
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO advisory (id, aliases, summary, severity, score, vector, malicious, \
                published_at, modified_at, withdrawn_at, document) \
         SELECT r.id, ARRAY(SELECT jsonb_array_elements_text(r.aliases)), r.summary, r.severity, \
                r.score, r.vector, r.malicious, r.published::timestamptz, r.modified::timestamptz, \
                r.withdrawn::timestamptz, r.document \
         FROM jsonb_to_recordset($1::jsonb) AS r(id text, aliases jsonb, summary text, severity text, \
                score numeric, vector text, malicious boolean, published text, modified text, \
                withdrawn text, document jsonb) \
         ON CONFLICT (id) DO UPDATE SET aliases = EXCLUDED.aliases, summary = EXCLUDED.summary, \
                severity = EXCLUDED.severity, score = EXCLUDED.score, vector = EXCLUDED.vector, \
                malicious = EXCLUDED.malicious, published_at = EXCLUDED.published_at, \
                modified_at = EXCLUDED.modified_at, withdrawn_at = EXCLUDED.withdrawn_at, \
                document = EXCLUDED.document",
        [rows],
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "DELETE FROM advisory_affected WHERE advisory_id IN \
         (SELECT i.id FROM jsonb_to_recordset($1::jsonb) AS i(id text))",
        [ids],
    ))
    .await?;
    txn.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO advisory_affected (advisory_id, ecosystem, name, ranges, versions) \
         SELECT a.advisory_id, a.ecosystem, a.name, a.ranges, \
                ARRAY(SELECT jsonb_array_elements_text(a.versions)) \
         FROM jsonb_to_recordset($1::jsonb) AS a(advisory_id text, ecosystem text, name text, \
                ranges jsonb, versions jsonb)",
        [affected],
    ))
    .await?;
    txn.commit().await?;
    stats.unchanged += unchanged.len() as u64;
    stats.changed += changed.len() as u64;
    stats.invalidated += invalidated;
    Ok(())
}

async fn remove_vanished(
    db: &DatabaseConnection,
    ecosystem: &str,
    seen: &HashSet<String>,
    force: bool,
    stats: &mut Stats,
) -> Result<(), ImportError> {
    let held = db
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!("SELECT DISTINCT f.advisory_id FROM advisory_affected f WHERE {SCOPE}"),
            [ecosystem.into()],
        ))
        .await?;
    let mut vanished = Vec::new();
    for row in &held {
        let id: String = row.try_get("", "advisory_id")?;
        if !seen.contains(&id) {
            vanished.push(id);
        }
    }
    if !force
        && vanished.len() > MAX_UNFORCED_REMOVALS
        && vanished.len() * 100 > held.len() * MAX_UNFORCED_PERCENT
    {
        return Err(ImportError::Shrinks {
            removing: vanished.len(),
            held: held.len(),
        });
    }
    for chunk in vanished.chunks(BATCH) {
        let ids: Vec<String> = chunk.to_vec();
        let txn = db.begin().await?;
        txn.execute(Statement::from_string(DbBackend::Postgres, LOCK))
            .await?;
        let invalidated = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                format!(
                    "UPDATE sca_package SET matched_at = NULL WHERE id IN ( \
                       SELECT p.id FROM sca_package p \
                       WHERE p.matched_at IS NOT NULL AND (p.ecosystem, p.name) IN ( \
                          SELECT f.ecosystem, f.name FROM advisory_affected f \
                          WHERE f.advisory_id = ANY($2) AND {SCOPE}) \
                       ORDER BY p.id FOR UPDATE OF p)"
                ),
                [ecosystem.into(), ids.clone().into()],
            ))
            .await?
            .rows_affected();
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            format!("DELETE FROM advisory_affected f WHERE f.advisory_id = ANY($2) AND {SCOPE}"),
            [ecosystem.into(), ids.clone().into()],
        ))
        .await?;
        let removed = txn
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM advisory a WHERE a.id = ANY($1) AND NOT EXISTS \
                 (SELECT 1 FROM advisory_affected f WHERE f.advisory_id = a.id)",
                [ids.into()],
            ))
            .await?
            .rows_affected();
        txn.commit().await?;
        stats.removed += removed;
        stats.invalidated += invalidated;
    }
    Ok(())
}
