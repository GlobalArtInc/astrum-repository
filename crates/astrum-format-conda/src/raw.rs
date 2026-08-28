//! Reads the members of large JSON objects without building a tree, so repodata of public channels
//! (hundreds of megabytes) can be merged and trimmed with little memory: the values stay slices of
//! the input.

#[derive(Debug, thiserror::Error)]
#[error("the JSON document is not valid at byte {0}")]
pub struct RawError(pub usize);

struct Scanner<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Scanner<'a> {
    fn fail<T>(&self) -> Result<T, RawError> {
        Err(RawError(self.at))
    }

    fn space(&mut self) {
        while let Some(byte) = self.bytes.get(self.at) {
            if !matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
                break;
            }
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn expect(&mut self, wanted: u8) -> Result<(), RawError> {
        self.space();
        if self.peek() == Some(wanted) {
            self.at += 1;
            Ok(())
        } else {
            self.fail()
        }
    }

    fn string(&mut self) -> Result<&'a [u8], RawError> {
        let start = self.at;
        if self.peek() != Some(b'"') {
            return self.fail();
        }
        self.at += 1;
        loop {
            match self.peek() {
                None => return self.fail(),
                Some(b'"') => {
                    self.at += 1;
                    return Ok(&self.bytes[start..self.at]);
                }
                Some(b'\\') => self.at += 2,
                Some(byte) if byte < 0x20 => return self.fail(),
                Some(_) => self.at += 1,
            }
        }
    }

    fn value(&mut self) -> Result<&'a [u8], RawError> {
        self.space();
        let start = self.at;
        match self.peek() {
            Some(b'"') => {
                self.string()?;
            }
            Some(b'{') | Some(b'[') => {
                let mut depth = 0usize;
                loop {
                    match self.peek() {
                        None => return self.fail(),
                        Some(b'"') => {
                            self.string()?;
                            continue;
                        }
                        Some(b'{') | Some(b'[') => depth += 1,
                        Some(b'}') | Some(b']') => {
                            depth -= 1;
                            if depth == 0 {
                                self.at += 1;
                                break;
                            }
                        }
                        Some(_) => {}
                    }
                    self.at += 1;
                }
            }
            Some(byte) if byte == b'-' || byte.is_ascii_alphanumeric() => {
                while let Some(byte) = self.peek() {
                    if !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'+' | b'.')) {
                        break;
                    }
                    self.at += 1;
                }
            }
            _ => return self.fail(),
        }
        Ok(&self.bytes[start..self.at])
    }
}

/// The members of the JSON object `bytes` as (key, raw value) pairs, in document order.
pub fn members(bytes: &[u8]) -> Result<Vec<(String, &[u8])>, RawError> {
    let mut scanner = Scanner { bytes, at: 0 };
    scanner.expect(b'{')?;
    let mut found = Vec::new();
    scanner.space();
    if scanner.peek() == Some(b'}') {
        scanner.at += 1;
        return finish(scanner, found);
    }
    loop {
        scanner.space();
        let key_at = scanner.at;
        let key = scanner.string()?;
        let key: String = serde_json::from_slice(key).map_err(|_| RawError(key_at))?;
        scanner.expect(b':')?;
        let value = scanner.value()?;
        found.push((key, value));
        scanner.space();
        match scanner.peek() {
            Some(b',') => scanner.at += 1,
            Some(b'}') => {
                scanner.at += 1;
                return finish(scanner, found);
            }
            _ => return scanner.fail(),
        }
    }
}

fn finish<T>(mut scanner: Scanner<'_>, found: T) -> Result<T, RawError> {
    scanner.space();
    if scanner.at == scanner.bytes.len() {
        Ok(found)
    } else {
        scanner.fail()
    }
}

/// The raw value of member `key` of the JSON object `bytes`.
pub fn member<'a>(found: &[(String, &'a [u8])], key: &str) -> Option<&'a [u8]> {
    found
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| *value)
}

/// Checks that a raw value is valid JSON before it is copied into another document.
pub fn valid(value: &[u8]) -> bool {
    serde_json::from_slice::<serde::de::IgnoredAny>(value).is_ok()
}

pub fn quote(text: &str) -> String {
    serde_json::to_string(text).expect("a string always serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_keep_raw_values() {
        let document =
            br#" {"a": {"b": [1, "x\"}"]}, "cA": -1.5e3, "d": true, "e": null, "f": "}"} "#;
        let found = members(document).unwrap();
        let keys: Vec<&str> = found.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["a", "cA", "d", "e", "f"]);
        assert_eq!(member(&found, "a").unwrap(), br#"{"b": [1, "x\"}"]}"#);
        assert_eq!(member(&found, "cA").unwrap(), b"-1.5e3");
        assert_eq!(member(&found, "f").unwrap(), br#""}""#);
        assert!(valid(member(&found, "a").unwrap()));
        assert!(members(b"{}").unwrap().is_empty());
        assert!(members(br#"{"a": 1"#).is_err());
        assert!(members(br#"{"a": 1} x"#).is_err());
        assert!(members(b"[1]").is_err());
        assert_eq!(quote("a\"b"), r#""a\"b""#);
    }
}
