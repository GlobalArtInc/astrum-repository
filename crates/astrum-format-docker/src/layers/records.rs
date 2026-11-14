pub const FIELD_LIMIT: usize = 1024;

pub struct Budget {
    left: usize,
}

impl Budget {
    pub fn new(total: usize) -> Self {
        Budget { left: total }
    }

    pub fn take(&mut self, bytes: usize) -> bool {
        match self.left.checked_sub(bytes) {
            Some(left) => {
                self.left = left;
                true
            }
            None => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truncation {
    Count,
    Budget,
}

pub struct Parsed<T> {
    pub packages: Vec<T>,
    pub malformed: usize,
    pub truncated: Option<Truncation>,
}

impl<T> Default for Parsed<T> {
    fn default() -> Self {
        Parsed {
            packages: Vec::new(),
            malformed: 0,
            truncated: None,
        }
    }
}

pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
}

pub fn field(bytes: &[u8]) -> Option<String> {
    let trimmed = bytes.trim_ascii();
    (trimmed.len() <= FIELD_LIMIT).then(|| String::from_utf8_lossy(trimmed).into_owned())
}

pub fn assign(slot: &mut String, too_long: &mut bool, raw: &[u8]) {
    match field(raw) {
        Some(value) => *slot = value,
        None => *too_long = true,
    }
}

pub fn token(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

pub fn optional(text: String) -> Result<Option<String>, ()> {
    match text.is_empty() {
        true => Ok(None),
        false if token(&text) => Ok(Some(text)),
        false => Err(()),
    }
}
