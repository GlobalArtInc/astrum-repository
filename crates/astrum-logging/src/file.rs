use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tracing_subscriber::fmt::MakeWriter;

pub const MAX_FILE_BYTES: u64 = 32 * 1024 * 1024;
pub const ROTATED_FILES: usize = 4;
pub const RETENTION: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const PREFIX: &str = "astrum-";
const SUFFIX: &str = ".log";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFile {
    pub name: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Debug)]
pub struct LogFiles {
    dir: PathBuf,
    current: String,
    max_bytes: u64,
    sink: Mutex<Sink>,
}

#[derive(Debug, Default)]
struct Sink {
    file: Option<File>,
    size: u64,
}

impl LogFiles {
    pub fn open(dir: &Path, node: &str) -> io::Result<Self> {
        Self::with_limit(dir, node, MAX_FILE_BYTES)
    }

    pub fn with_limit(dir: &Path, node: &str, max_bytes: u64) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let files = Self {
            dir: dir.to_owned(),
            current: file_name(node),
            max_bytes,
            sink: Mutex::new(Sink::default()),
        };
        files.prune(RETENTION);
        let mut sink = files.lock();
        files.reopen(&mut sink)?;
        drop(sink);
        Ok(files)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        let mut sink = self.lock();
        if sink.file.is_none() {
            self.reopen(&mut sink)?;
        }
        let Some(file) = sink.file.as_mut() else {
            return Ok(());
        };
        if let Err(error) = file.write_all(bytes) {
            sink.file = None;
            return Err(error);
        }
        sink.size += bytes.len() as u64;
        if sink.size >= self.max_bytes {
            self.rotate(&mut sink)?;
        }
        Ok(())
    }

    pub fn mark(&self, message: &str) -> io::Result<()> {
        let stars = "*".repeat(80);
        let time = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
        let message = message.replace(['\r', '\n'], " ");
        self.write(format!("{stars}\n{time} MARK - {message}\n{stars}\n").as_bytes())
    }

    pub fn list(&self) -> io::Result<Vec<LogFile>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !valid_file_name(&name) {
                continue;
            }
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                files.push(LogFile {
                    name,
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                });
            }
        }
        files.sort_by(|a, b| {
            (b.name == self.current)
                .cmp(&(a.name == self.current))
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(files)
    }

    pub fn path_of(&self, name: &str) -> Option<PathBuf> {
        valid_file_name(name).then(|| self.dir.join(name))
    }

    pub fn read(&self, name: &str, from: Option<u64>, limit: u64) -> io::Result<Chunk> {
        let path = self
            .path_of(name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such log file"))?;
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();
        let start = match from {
            Some(from) if from <= size && size - from <= limit => from,
            _ => line_start(&mut file, size.saturating_sub(limit))?,
        };
        file.seek(SeekFrom::Start(start))?;
        let mut text = Vec::with_capacity((size - start) as usize);
        Read::by_ref(&mut file)
            .take(size - start)
            .read_to_end(&mut text)?;
        Ok(Chunk {
            start,
            end: start + text.len() as u64,
            size,
            text,
        })
    }

    pub fn prune(&self, older_than: Duration) {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return;
        };
        let now = SystemTime::now();
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !valid_file_name(&name) || name == self.current {
                continue;
            }
            let old = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .is_some_and(|age| age > older_than);
            if old {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Sink> {
        self.sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn reopen(&self, sink: &mut Sink) -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(&self.current))?;
        sink.size = file.metadata()?.len();
        sink.file = Some(file);
        Ok(())
    }

    fn rotate(&self, sink: &mut Sink) -> io::Result<()> {
        sink.file = None;
        let rotated = |index: usize| self.dir.join(format!("{}.{index}", self.current));
        let _ = fs::remove_file(rotated(ROTATED_FILES));
        for index in (1..ROTATED_FILES).rev() {
            let from = rotated(index);
            if from.exists() {
                fs::rename(&from, rotated(index + 1))?;
            }
        }
        fs::rename(self.dir.join(&self.current), rotated(1))?;
        self.reopen(sink)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub text: Vec<u8>,
}

fn line_start(file: &mut File, start: u64) -> io::Result<u64> {
    if start == 0 {
        return Ok(0);
    }
    file.seek(SeekFrom::Start(start - 1))?;
    let mut buffer = [0u8; 4096];
    let read = file.read(&mut buffer)?;
    match buffer[..read].iter().position(|byte| *byte == b'\n') {
        Some(0) => Ok(start),
        Some(index) => Ok(start - 1 + index as u64 + 1),
        None => Ok(start),
    }
}

pub fn file_name(node: &str) -> String {
    let node: String = node
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => c,
            _ => '_',
        })
        .collect();
    let node = node.trim_matches('.');
    match node.is_empty() {
        true => format!("{PREFIX}node{SUFFIX}"),
        false => format!("{PREFIX}{node}{SUFFIX}"),
    }
}

pub fn valid_file_name(name: &str) -> bool {
    let base = match name.rsplit_once('.') {
        Some((base, index)) if !index.is_empty() && index.chars().all(|c| c.is_ascii_digit()) => {
            base
        }
        _ => name,
    };
    base.len() <= 255
        && base.starts_with(PREFIX)
        && base.ends_with(SUFFIX)
        && base.len() > PREFIX.len() + SUFFIX.len()
        && !base.contains("..")
        && base
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

#[derive(Clone, Debug)]
pub struct FileWriter(pub Arc<LogFiles>);

pub struct LineWriter<'a>(&'a LogFiles);

impl Write for LineWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for FileWriter {
    type Writer = LineWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        LineWriter(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_files_after_the_node() {
        assert_eq!(file_name("web-1"), "astrum-web-1.log");
        assert_eq!(file_name("a/b c"), "astrum-a_b_c.log");
        assert_eq!(file_name(""), "astrum-node.log");
        assert!(valid_file_name("astrum-web-1.log"));
        assert!(valid_file_name("astrum-web-1.log.3"));
        assert!(!valid_file_name("astrum-.log"));
        assert!(!valid_file_name("../astrum-x.log"));
        assert!(!valid_file_name("astrum-x..log"));
        assert!(!valid_file_name("audit.log"));
        assert!(!valid_file_name("astrum-x.log.x"));
    }

    #[test]
    fn appends_rotates_and_lists() {
        let dir = tempfile::tempdir().unwrap();
        let files = LogFiles::with_limit(dir.path(), "n1", 100).unwrap();
        files.write(&[b'a'; 60]).unwrap();
        files.write(&[b'b'; 60]).unwrap();
        files.write(b"after\n").unwrap();
        let listed = files.list().unwrap();
        let names: Vec<_> = listed.iter().map(|file| file.name.as_str()).collect();
        assert_eq!(names, ["astrum-n1.log", "astrum-n1.log.1"]);
        assert_eq!(listed[0].size, 6);
        assert_eq!(listed[1].size, 120);
        for _ in 0..10 {
            files.write(&[b'c'; 120]).unwrap();
        }
        let count = files.list().unwrap().len();
        assert_eq!(count, 1 + ROTATED_FILES);
    }

    #[test]
    fn reads_the_tail_from_a_line_start_and_follows() {
        let dir = tempfile::tempdir().unwrap();
        let files = LogFiles::open(dir.path(), "n1").unwrap();
        files.write(b"first line\nsecond line\nthird\n").unwrap();
        let tail = files.read("astrum-n1.log", None, 15).unwrap();
        assert_eq!(tail.text, b"third\n");
        assert_eq!(tail.end, tail.size);
        files.write(b"fourth\n").unwrap();
        let next = files.read("astrum-n1.log", Some(tail.end), 1000).unwrap();
        assert_eq!(next.text, b"fourth\n");
        let all = files.read("astrum-n1.log", None, 1000).unwrap();
        assert_eq!(all.start, 0);
        assert!(files.read("../etc/passwd", None, 10).is_err());
    }

    #[test]
    fn marks_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let files = LogFiles::open(dir.path(), "n1").unwrap();
        files.mark("before\nthe upgrade").unwrap();
        let text = fs::read_to_string(dir.path().join("astrum-n1.log")).unwrap();
        assert!(text.contains("MARK - before the upgrade\n"));
        assert_eq!(text.lines().count(), 3);
    }

    #[test]
    fn prunes_old_files_of_other_nodes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("astrum-gone.log"), b"x").unwrap();
        fs::write(dir.path().join("other.txt"), b"x").unwrap();
        let files = LogFiles::open(dir.path(), "n1").unwrap();
        files.prune(Duration::ZERO);
        let mut names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["astrum-n1.log", "other.txt"]);
    }
}
