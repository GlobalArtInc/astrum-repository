use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sums {
    pub digests: BTreeMap<String, String>,
}

impl Sums {
    pub fn of(digests: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            digests: digests
                .into_iter()
                .map(|(file, digest)| (file, digest.to_ascii_lowercase()))
                .collect(),
        }
    }

    pub fn parse(document: &[u8]) -> Result<Self, String> {
        let text = String::from_utf8_lossy(document);
        let mut digests = BTreeMap::new();
        for (number, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let bad = || {
                format!(
                    "line {} of the checksums is not '<sha256>  <file name>'",
                    number + 1
                )
            };
            let line = line.trim_end();
            let (digest, rest) = line.split_once(char::is_whitespace).ok_or_else(bad)?;
            let file = rest.trim_start().trim_start_matches('*');
            if digest.len() != 64
                || !digest.bytes().all(|c| c.is_ascii_hexdigit())
                || file.is_empty()
                || file.contains(char::is_whitespace)
            {
                return Err(bad());
            }
            digests.insert(file.to_owned(), digest.to_ascii_lowercase());
        }
        if digests.is_empty() {
            return Err("the checksums document is empty".to_owned());
        }
        Ok(Self { digests })
    }

    pub fn digest(&self, file: &str) -> Option<&str> {
        self.digests.get(file).map(String::as_str)
    }

    pub fn render(&self) -> Vec<u8> {
        let mut document = String::new();
        for (file, digest) in &self.digests {
            document.push_str(&format!("{digest}  {file}\n"));
        }
        document.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_round_trip() {
        let a = "a".repeat(64);
        let sums = Sums::of([
            ("b.zip".to_owned(), "B".repeat(64)),
            ("a.zip".to_owned(), a.clone()),
        ]);
        let text = String::from_utf8(sums.render()).unwrap();
        assert_eq!(text, format!("{a}  a.zip\n{}  b.zip\n", "b".repeat(64)));
        assert_eq!(Sums::parse(text.as_bytes()).unwrap(), sums);
        assert_eq!(
            Sums::parse(format!("{a} *a.zip\r\n").as_bytes())
                .unwrap()
                .digest("a.zip"),
            Some(a.as_str())
        );
        assert!(Sums::parse(b"nonsense\n").is_err());
        assert!(Sums::parse(b"\n").is_err());
    }
}
