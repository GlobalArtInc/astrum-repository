#[derive(Debug, Default, PartialEq, Eq)]
pub struct Declaration {
    pub name: Option<String>,
    pub version: Option<String>,
}

struct Scanner<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Scanner<'a> {
    fn peek(&self) -> Option<u8> {
        self.text.get(self.at).copied()
    }

    fn skip_blank(&mut self) {
        while let Some(byte) = self.peek() {
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => self.at += 1,
                b'#' => {
                    while self.peek().is_some_and(|byte| byte != b'\n') {
                        self.at += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Option<String> {
        let quote = self.peek().filter(|byte| matches!(byte, b'"' | b'\''))?;
        let triple = self.text[self.at..].starts_with(&[quote, quote, quote]);
        self.at += if triple { 3 } else { 1 };
        let mut value = Vec::new();
        loop {
            let byte = self.peek()?;
            if byte == b'\\' {
                value.push(*self.text.get(self.at + 1)?);
                self.at += 2;
                continue;
            }
            if byte == quote {
                if !triple {
                    self.at += 1;
                    break;
                }
                if self.text[self.at..].starts_with(&[quote, quote, quote]) {
                    self.at += 3;
                    break;
                }
            }
            if byte == b'\n' && !triple {
                return None;
            }
            value.push(byte);
            self.at += 1;
        }
        String::from_utf8(value).ok()
    }

    fn word(&mut self) -> &'a str {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.at += 1;
        }
        std::str::from_utf8(&self.text[start..self.at]).unwrap_or_default()
    }

    fn call(&mut self) -> Declaration {
        let mut declared = Declaration::default();
        let mut depth = 1usize;
        while depth > 0 {
            self.skip_blank();
            let Some(byte) = self.peek() else { break };
            match byte {
                b'(' | b'[' | b'{' => {
                    depth += 1;
                    self.at += 1;
                }
                b')' | b']' | b'}' => {
                    depth -= 1;
                    self.at += 1;
                }
                b'"' | b'\'' => {
                    if self.string().is_none() {
                        break;
                    }
                }
                byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                    let word = self.word();
                    if depth == 1 && matches!(word, "name" | "version") {
                        let mark = self.at;
                        self.skip_blank();
                        if self.peek() == Some(b'=') && self.text.get(self.at + 1) != Some(&b'=') {
                            self.at += 1;
                            self.skip_blank();
                            if let Some(value) = self.string() {
                                match word {
                                    "name" => declared.name = Some(value),
                                    _ => declared.version = Some(value),
                                }
                                continue;
                            }
                        }
                        self.at = mark;
                    }
                }
                _ => self.at += 1,
            }
        }
        declared
    }
}

pub fn declaration(text: &str) -> Option<Declaration> {
    let mut scanner = Scanner {
        text: text.as_bytes(),
        at: 0,
    };
    let mut depth = 0usize;
    loop {
        scanner.skip_blank();
        let byte = scanner.peek()?;
        match byte {
            b'(' | b'[' | b'{' => {
                depth += 1;
                scanner.at += 1;
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                scanner.at += 1;
            }
            b'"' | b'\'' => {
                scanner.string()?;
            }
            byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                let word = scanner.word();
                if depth == 0 && word == "module" {
                    scanner.skip_blank();
                    if scanner.peek() == Some(b'(') {
                        scanner.at += 1;
                        return Some(scanner.call());
                    }
                }
            }
            _ => scanner.at += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(text: &str) -> Declaration {
        declaration(text).unwrap_or_default()
    }

    #[test]
    fn the_module_call_names_the_module_and_its_version() {
        let text = "\nmodule(\n    name = \"bazel_skylib\",\n    # Keep in sync\n    version = \"1.7.1\",\n    compatibility_level = 1,\n)\n\nbazel_dep(name = \"platforms\", version = \"0.0.4\")\n";
        assert_eq!(
            declared(text),
            Declaration {
                name: Some("bazel_skylib".to_owned()),
                version: Some("1.7.1".to_owned()),
            }
        );
    }

    #[test]
    fn a_module_without_a_call_declares_nothing() {
        assert_eq!(
            declaration("bazel_dep(name = \"a\", version = \"1\")\n"),
            None
        );
        assert_eq!(declaration(""), None);
        assert_eq!(declared("module()"), Declaration::default());
    }

    #[test]
    fn comments_strings_and_other_calls_do_not_count() {
        let text = "# module(name = \"wrong\")\nx = \"module(name = 'wrong')\"\nbazel_dep(name = \"dep\", version = \"9\")\nmodule(name = 'right', version = '2')\n";
        assert_eq!(
            declared(text),
            Declaration {
                name: Some("right".to_owned()),
                version: Some("2".to_owned()),
            }
        );
    }

    #[test]
    fn only_literal_arguments_of_the_call_itself_count() {
        let text = "module(name = NAME, version = \"3\", repo_name = \"x\", extra = dict(name = \"inner\"))";
        assert_eq!(
            declared(text),
            Declaration {
                name: None,
                version: Some("3".to_owned()),
            }
        );
        assert_eq!(declared("module(name == \"a\")"), Declaration::default());
    }

    #[test]
    fn escapes_and_triple_quotes_are_read() {
        assert_eq!(
            declared("module(name = \"a\\\"b\")").name.as_deref(),
            Some("a\"b")
        );
        assert_eq!(
            declared("module(name = \"\"\"multi\nline\"\"\")")
                .name
                .as_deref(),
            Some("multi\nline")
        );
    }
}
