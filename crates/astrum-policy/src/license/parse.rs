use std::fmt;

use super::spdx;

pub const MAX_LENGTH: usize = 1024;
pub const MAX_DEPTH: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Id(String),
    With(String, String),
    And(Vec<Expression>),
    Or(Vec<Expression>),
}

impl Expression {
    pub fn ids(&self) -> Vec<&str> {
        let mut ids = Vec::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids<'a>(&'a self, ids: &mut Vec<&'a str>) {
        match self {
            Self::Id(id) | Self::With(id, _) => ids.push(id),
            Self::And(terms) | Self::Or(terms) => {
                for term in terms {
                    term.collect_ids(ids);
                }
            }
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => f.write_str(id),
            Self::With(id, exception) => write!(f, "{id} WITH {exception}"),
            Self::And(terms) => {
                let parts: Vec<String> = terms
                    .iter()
                    .map(|term| match term {
                        Self::Or(_) => format!("({term})"),
                        _ => term.to_string(),
                    })
                    .collect();
                f.write_str(&parts.join(" AND "))
            }
            Self::Or(terms) => {
                let parts: Vec<String> = terms.iter().map(ToString::to_string).collect();
                f.write_str(&parts.join(" OR "))
            }
        }
    }
}

pub struct Parsed {
    pub expression: Expression,
    pub mapped: bool,
}

pub fn parse(text: &str) -> Option<Parsed> {
    if text.len() > MAX_LENGTH {
        return None;
    }
    let mut parser = Parser {
        tokens: tokenize(text),
        position: 0,
        depth: 0,
        mapped: false,
    };
    let expression = parser.or()?;
    (parser.position == parser.tokens.len()).then_some(Parsed {
        expression,
        mapped: parser.mapped,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    Open,
    Close,
    Slash,
    And,
    Or,
    With,
    Word(&'a str),
}

fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut word_start: Option<usize> = None;
    for (index, ch) in text.char_indices() {
        let symbol = match ch {
            '(' => Some(Token::Open),
            ')' => Some(Token::Close),
            '/' => Some(Token::Slash),
            _ if ch.is_whitespace() => None,
            _ => {
                word_start.get_or_insert(index);
                continue;
            }
        };
        if let Some(start) = word_start.take() {
            tokens.push(word(&text[start..index]));
        }
        tokens.extend(symbol);
    }
    if let Some(start) = word_start {
        tokens.push(word(&text[start..]));
    }
    tokens
}

fn word(text: &str) -> Token<'_> {
    if text.eq_ignore_ascii_case("AND") {
        Token::And
    } else if text.eq_ignore_ascii_case("OR") {
        Token::Or
    } else if text.eq_ignore_ascii_case("WITH") {
        Token::With
    } else {
        Token::Word(text)
    }
}

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    position: usize,
    depth: usize,
    mapped: bool,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Token<'a>> {
        self.tokens.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn or(&mut self) -> Option<Expression> {
        let mut terms = vec![self.and()?];
        while let Some(operator @ (Token::Or | Token::Slash)) = self.peek() {
            self.mapped |= operator == Token::Slash;
            self.advance();
            terms.push(self.and()?);
        }
        Some(group(terms, Expression::Or))
    }

    fn and(&mut self) -> Option<Expression> {
        let mut terms = vec![self.with()?];
        while self.peek() == Some(Token::And) {
            self.advance();
            terms.push(self.with()?);
        }
        Some(group(terms, Expression::And))
    }

    fn with(&mut self) -> Option<Expression> {
        if self.peek() == Some(Token::Open) {
            self.advance();
            self.depth += 1;
            if self.depth > MAX_DEPTH {
                return None;
            }
            let inner = self.or()?;
            if self.peek() != Some(Token::Close) {
                return None;
            }
            self.advance();
            self.depth -= 1;
            return Some(inner);
        }
        let Some(Token::Word(word)) = self.peek() else {
            return None;
        };
        self.advance();
        let id = self.license(word)?;
        if self.peek() != Some(Token::With) {
            return Some(Expression::Id(id));
        }
        self.advance();
        let Some(Token::Word(exception)) = self.peek() else {
            return None;
        };
        self.advance();
        is_id(exception).then(|| Expression::With(id, exception.to_owned()))
    }

    fn license(&mut self, word: &str) -> Option<String> {
        if let Some(entry) = spdx::lookup(word) {
            return Some(entry.id.clone());
        }
        if is_reference(word) {
            return Some(word.to_owned());
        }
        let (core, plus) = match word.strip_suffix('+') {
            Some(core) => (core, true),
            None => (word, false),
        };
        if !is_id(core) {
            return None;
        }
        if let Some(later) = plus.then(|| or_later(core)).flatten() {
            self.mapped = true;
            return Some(later);
        }
        Some(word.to_owned())
    }
}

fn group(mut terms: Vec<Expression>, wrap: fn(Vec<Expression>) -> Expression) -> Expression {
    if terms.len() == 1 {
        terms.swap_remove(0)
    } else {
        wrap(terms)
    }
}

fn or_later(core: &str) -> Option<String> {
    let base = spdx::lookup(core).filter(|entry| !entry.deprecated)?;
    let later = spdx::lookup(&format!("{}-or-later", base.id))?;
    (!later.deprecated).then(|| later.id.clone())
}

fn is_id(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-')
}

fn is_reference(word: &str) -> bool {
    match word.split_once(':') {
        Some((document, license)) => {
            strip_prefix_ci(document, "DocumentRef-").is_some_and(is_id)
                && strip_prefix_ci(license, "LicenseRef-").is_some_and(is_id)
        }
        None => strip_prefix_ci(word, "LicenseRef-").is_some_and(is_id),
    }
}

fn strip_prefix_ci<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    if head.eq_ignore_ascii_case(prefix) {
        text.get(prefix.len()..)
    } else {
        None
    }
}
