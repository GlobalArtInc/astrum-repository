pub const FIELD_LIMIT: usize = 1024;

pub struct Parsed<T> {
    pub packages: Vec<T>,
    pub malformed: usize,
    pub truncated: bool,
}

impl<T> Default for Parsed<T> {
    fn default() -> Self {
        Parsed {
            packages: Vec::new(),
            malformed: 0,
            truncated: false,
        }
    }
}

pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
}

pub fn field(bytes: &[u8]) -> String {
    let trimmed = bytes.trim_ascii();
    let kept = &trimmed[..trimmed.len().min(FIELD_LIMIT)];
    String::from_utf8_lossy(kept).into_owned()
}

pub fn token(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(|ch| ch.is_control() || ch.is_whitespace())
}

pub fn optional(text: String) -> Option<String> {
    token(&text).then_some(text)
}
