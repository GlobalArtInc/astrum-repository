use std::fmt::Write;

const MAX_DEPTH: usize = 64;

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Atom(String),
    Integer(i128),
    Float(f64),
    Binary(Vec<u8>),
    Chars(Vec<u32>),
    Tuple(Vec<Term>),
    List(Vec<Term>),
    Map(Vec<(Term, Term)>),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {message}")]
pub struct TermError {
    pub line: usize,
    pub message: String,
}

impl Term {
    pub fn text(&self) -> Option<String> {
        match self {
            Term::Binary(bytes) => String::from_utf8(bytes.clone()).ok(),
            Term::Chars(chars) => chars.iter().map(|c| char::from_u32(*c)).collect(),
            Term::List(items) if items.is_empty() => Some(String::new()),
            Term::List(items) => items
                .iter()
                .map(|item| match item {
                    Term::Integer(code) => u32::try_from(*code).ok().and_then(char::from_u32),
                    _ => None,
                })
                .collect(),
            Term::Atom(atom) => Some(atom.clone()),
            _ => None,
        }
    }

    pub fn items(&self) -> Option<Vec<Term>> {
        match self {
            Term::List(items) => Some(items.clone()),
            Term::Chars(chars) => Some(chars.iter().map(|c| Term::Integer(*c as i128)).collect()),
            Term::Map(pairs) => Some(
                pairs
                    .iter()
                    .map(|(key, value)| Term::Tuple(vec![key.clone(), value.clone()]))
                    .collect(),
            ),
            _ => None,
        }
    }

    pub fn pairs(&self) -> Option<Vec<(Term, Term)>> {
        match self {
            Term::Map(pairs) => Some(pairs.clone()),
            Term::List(items) => items
                .iter()
                .map(|item| match item {
                    Term::Tuple(parts) if parts.len() == 2 => {
                        Some((parts[0].clone(), parts[1].clone()))
                    }
                    _ => None,
                })
                .collect(),
            Term::Chars(chars) if chars.is_empty() => Some(Vec::new()),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<Term> {
        self.pairs()?
            .into_iter()
            .find(|(candidate, _)| candidate.text().as_deref() == Some(key))
            .map(|(_, value)| value)
    }

    pub fn boolean(&self) -> Option<bool> {
        match self {
            Term::Atom(atom) if atom == "true" => Some(true),
            Term::Atom(atom) if atom == "false" => Some(false),
            _ => None,
        }
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, Term::Atom(atom) if atom == "undefined" || atom == "nil")
    }
}

pub fn parse(text: &str) -> Result<Vec<Term>, TermError> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        at: 0,
        line: 1,
    };
    let mut terms = Vec::new();
    loop {
        parser.skip_space();
        if parser.done() {
            return Ok(terms);
        }
        terms.push(parser.term(0)?);
        parser.skip_space();
        parser.expect('.')?;
    }
}

struct Parser {
    chars: Vec<char>,
    at: usize,
    line: usize,
}

impl Parser {
    fn done(&self) -> bool {
        self.at >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at + offset).copied()
    }

    fn next(&mut self) -> Option<char> {
        let next = self.peek()?;
        self.at += 1;
        if next == '\n' {
            self.line += 1;
        }
        Some(next)
    }

    fn fail<T>(&self, message: impl Into<String>) -> Result<T, TermError> {
        Err(TermError {
            line: self.line,
            message: message.into(),
        })
    }

    fn skip_space(&mut self) {
        while let Some(next) = self.peek() {
            match next {
                '%' => {
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.next();
                    }
                }
                c if c.is_whitespace() => {
                    self.next();
                }
                _ => return,
            }
        }
    }

    fn expect(&mut self, wanted: char) -> Result<(), TermError> {
        self.skip_space();
        match self.next() {
            Some(found) if found == wanted => Ok(()),
            Some(found) => self.fail(format!("expected '{wanted}' but found '{found}'")),
            None => self.fail(format!("expected '{wanted}' but the text ends")),
        }
    }

    fn term(&mut self, depth: usize) -> Result<Term, TermError> {
        if depth > MAX_DEPTH {
            return self.fail("the terms are nested too deeply");
        }
        self.skip_space();
        match (self.peek(), self.peek_at(1)) {
            (Some('{'), _) => {
                self.next();
                Ok(Term::Tuple(self.sequence('}', depth)?))
            }
            (Some('['), _) => {
                self.next();
                self.list(depth)
            }
            (Some('<'), Some('<')) => {
                self.at += 2;
                self.binary()
            }
            (Some('#'), Some('{')) => {
                self.at += 2;
                self.map(depth)
            }
            (Some('"'), _) => Ok(Term::Chars(self.strings()?)),
            (Some('\''), _) => {
                self.next();
                let chars = self.quoted('\'')?;
                Ok(Term::Atom(
                    chars.iter().filter_map(|c| char::from_u32(*c)).collect(),
                ))
            }
            (Some(c), _) if c.is_ascii_lowercase() => {
                let mut atom = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '@' {
                        atom.push(c);
                        self.next();
                    } else {
                        break;
                    }
                }
                Ok(Term::Atom(atom))
            }
            (Some(c), _) if c.is_ascii_digit() || c == '-' || c == '+' || c == '$' => self.number(),
            (Some(c), _) => self.fail(format!("unexpected '{c}'")),
            (None, _) => self.fail("the text ends inside a term"),
        }
    }

    fn sequence(&mut self, close: char, depth: usize) -> Result<Vec<Term>, TermError> {
        let mut items = Vec::new();
        self.skip_space();
        if self.peek() == Some(close) {
            self.next();
            return Ok(items);
        }
        loop {
            items.push(self.term(depth + 1)?);
            self.skip_space();
            match self.next() {
                Some(',') => continue,
                Some(c) if c == close => return Ok(items),
                Some(c) => return self.fail(format!("expected ',' or '{close}' but found '{c}'")),
                None => return self.fail(format!("expected '{close}' but the text ends")),
            }
        }
    }

    fn list(&mut self, depth: usize) -> Result<Term, TermError> {
        let mut items = Vec::new();
        self.skip_space();
        if self.peek() == Some(']') {
            self.next();
            return Ok(Term::List(items));
        }
        loop {
            items.push(self.term(depth + 1)?);
            self.skip_space();
            match self.next() {
                Some(',') => continue,
                Some(']') => return Ok(Term::List(items)),
                Some('|') => {
                    let tail = self.term(depth + 1)?;
                    self.expect(']')?;
                    match tail {
                        Term::List(rest) => items.extend(rest),
                        Term::Chars(rest) => {
                            items.extend(rest.into_iter().map(|c| Term::Integer(c as i128)))
                        }
                        _ => return self.fail("an improper list is not supported"),
                    }
                    return Ok(Term::List(items));
                }
                Some(c) => return self.fail(format!("expected ',' or ']' but found '{c}'")),
                None => return self.fail("expected ']' but the text ends"),
            }
        }
    }

    fn map(&mut self, depth: usize) -> Result<Term, TermError> {
        let mut pairs = Vec::new();
        self.skip_space();
        if self.peek() == Some('}') {
            self.next();
            return Ok(Term::Map(pairs));
        }
        loop {
            let key = self.term(depth + 1)?;
            self.skip_space();
            if self.peek() != Some('=') || self.peek_at(1) != Some('>') {
                return self.fail("expected '=>' in a map");
            }
            self.at += 2;
            let value = self.term(depth + 1)?;
            pairs.push((key, value));
            self.skip_space();
            match self.next() {
                Some(',') => continue,
                Some('}') => return Ok(Term::Map(pairs)),
                Some(c) => return self.fail(format!("expected ',' or '}}' but found '{c}'")),
                None => return self.fail("expected '}' but the text ends"),
            }
        }
    }

    fn binary(&mut self) -> Result<Term, TermError> {
        let mut bytes = Vec::new();
        self.skip_space();
        if self.peek() == Some('>') && self.peek_at(1) == Some('>') {
            self.at += 2;
            return Ok(Term::Binary(bytes));
        }
        loop {
            self.skip_space();
            let segment = match self.peek() {
                Some('"') => Segment::Chars(self.strings()?),
                _ => match self.number()? {
                    Term::Integer(value) => Segment::Integer(value),
                    _ => return self.fail("a binary segment must be a string or an integer"),
                },
            };
            self.skip_space();
            let mut utf8 = false;
            if self.peek() == Some('/') {
                self.next();
                let mut kind = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '-' {
                        kind.push(c);
                        self.next();
                    } else {
                        break;
                    }
                }
                match kind.as_str() {
                    "utf8" => utf8 = true,
                    "binary" | "bytes" => {}
                    other => return self.fail(format!("the binary type {other} is not supported")),
                }
            }
            match segment {
                Segment::Chars(chars) if utf8 => {
                    for code in chars {
                        let Some(c) = char::from_u32(code) else {
                            return self.fail("a binary holds an invalid character");
                        };
                        let mut buffer = [0; 4];
                        bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                    }
                }
                Segment::Chars(chars) => bytes.extend(chars.into_iter().map(|code| code as u8)),
                Segment::Integer(value) if utf8 => {
                    let Some(c) = u32::try_from(value).ok().and_then(char::from_u32) else {
                        return self.fail("a binary holds an invalid character");
                    };
                    let mut buffer = [0; 4];
                    bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                }
                Segment::Integer(value) => bytes.push(value as u8),
            }
            self.skip_space();
            match (self.next(), self.peek()) {
                (Some(','), _) => continue,
                (Some('>'), Some('>')) => {
                    self.next();
                    return Ok(Term::Binary(bytes));
                }
                _ => return self.fail("expected ',' or '>>' in a binary"),
            }
        }
    }

    fn strings(&mut self) -> Result<Vec<u32>, TermError> {
        let mut chars = Vec::new();
        loop {
            self.skip_space();
            if self.peek() != Some('"') {
                return Ok(chars);
            }
            self.next();
            chars.extend(self.quoted('"')?);
        }
    }

    fn quoted(&mut self, close: char) -> Result<Vec<u32>, TermError> {
        let mut chars = Vec::new();
        loop {
            match self.next() {
                None => return self.fail("the text ends inside a string"),
                Some(c) if c == close => return Ok(chars),
                Some('\\') => chars.push(self.escape()?),
                Some(c) => chars.push(c as u32),
            }
        }
    }

    fn escape(&mut self) -> Result<u32, TermError> {
        let Some(c) = self.next() else {
            return self.fail("the text ends inside an escape");
        };
        Ok(match c {
            'b' => 8,
            'd' => 127,
            'e' => 27,
            'f' => 12,
            'n' => 10,
            'r' => 13,
            's' => 32,
            't' => 9,
            'v' => 11,
            '0'..='7' => {
                let mut value = c.to_digit(8).unwrap_or_default();
                for _ in 0..2 {
                    match self.peek().and_then(|next| next.to_digit(8)) {
                        Some(digit) => {
                            value = value * 8 + digit;
                            self.next();
                        }
                        None => break,
                    }
                }
                value
            }
            'x' => {
                let mut digits = String::new();
                if self.peek() == Some('{') {
                    self.next();
                    loop {
                        match self.next() {
                            Some('}') => break,
                            Some(digit) if digit.is_ascii_hexdigit() => digits.push(digit),
                            _ => return self.fail("a \\x{...} escape is not closed"),
                        }
                    }
                } else {
                    for _ in 0..2 {
                        match self.next() {
                            Some(digit) if digit.is_ascii_hexdigit() => digits.push(digit),
                            _ => return self.fail("a \\x escape needs two hex digits"),
                        }
                    }
                }
                match u32::from_str_radix(&digits, 16) {
                    Ok(value) => value,
                    Err(_) => return self.fail("a \\x escape is not a number"),
                }
            }
            '^' => match self.next() {
                Some(control) => (control as u32) & 31,
                None => return self.fail("the text ends inside an escape"),
            },
            other => other as u32,
        })
    }

    fn number(&mut self) -> Result<Term, TermError> {
        let mut text = String::new();
        if let Some(sign @ ('-' | '+')) = self.peek() {
            text.push(sign);
            self.next();
        }
        if self.peek() == Some('$') {
            self.next();
            let code = match self.next() {
                Some('\\') => self.escape()?,
                Some(c) => c as u32,
                None => return self.fail("the text ends after '$'"),
            };
            let value = code as i128;
            return Ok(Term::Integer(if text == "-" { -value } else { value }));
        }
        while let Some(c) = self.peek() {
            let part = c.is_ascii_alphanumeric()
                || c == '_'
                || c == '#'
                || (c == '.' && self.peek_at(1).is_some_and(|next| next.is_ascii_digit()))
                || ((c == '-' || c == '+') && text.ends_with(['e', 'E']) && text.contains('.'));
            if !part {
                break;
            }
            text.push(c);
            self.next();
        }
        let plain = text.replace('_', "");
        if let Some((base, digits)) = plain.split_once('#') {
            let (negative, base) = match base.strip_prefix('-') {
                Some(base) => (true, base),
                None => (false, base.trim_start_matches('+')),
            };
            let parsed = base
                .parse::<u32>()
                .ok()
                .filter(|base| (2..=36).contains(base))
                .and_then(|base| i128::from_str_radix(digits, base).ok());
            return match parsed {
                Some(value) => Ok(Term::Integer(if negative { -value } else { value })),
                None => self.fail(format!("{text} is not a number")),
            };
        }
        if plain.contains('.') {
            return match plain.parse::<f64>() {
                Ok(value) => Ok(Term::Float(value)),
                Err(_) => self.fail(format!("{text} is not a number")),
            };
        }
        match plain.parse::<i128>() {
            Ok(value) => Ok(Term::Integer(value)),
            Err(_) => self.fail(format!("{text} is not a number")),
        }
    }
}

enum Segment {
    Chars(Vec<u32>),
    Integer(i128),
}

pub fn print(term: &Term) -> String {
    let mut out = String::new();
    write_term(&mut out, term);
    out
}

fn write_term(out: &mut String, term: &Term) {
    match term {
        Term::Atom(atom) => write_atom(out, atom),
        Term::Integer(value) => {
            let _ = write!(out, "{value}");
        }
        Term::Float(value) => {
            let _ = write!(out, "{value:?}");
        }
        Term::Binary(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) if text.is_ascii() => {
                out.push_str("<<\"");
                write_chars(out, text.chars());
                out.push_str("\">>");
            }
            Ok(text) => {
                out.push_str("<<\"");
                write_chars(out, text.chars());
                out.push_str("\"/utf8>>");
            }
            Err(_) => {
                out.push_str("<<");
                let bytes: Vec<String> = bytes.iter().map(u8::to_string).collect();
                out.push_str(&bytes.join(","));
                out.push_str(">>");
            }
        },
        Term::Chars(chars) => {
            out.push('"');
            write_chars(out, chars.iter().filter_map(|c| char::from_u32(*c)));
            out.push('"');
        }
        Term::Tuple(items) => {
            out.push('{');
            write_items(out, items);
            out.push('}');
        }
        Term::List(items) => {
            out.push('[');
            write_items(out, items);
            out.push(']');
        }
        Term::Map(pairs) => {
            out.push_str("#{");
            for (index, (key, value)) in pairs.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_term(out, key);
                out.push_str(" => ");
                write_term(out, value);
            }
            out.push('}');
        }
    }
}

fn write_items(out: &mut String, items: &[Term]) {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_term(out, item);
    }
}

fn write_atom(out: &mut String, atom: &str) {
    let plain = atom.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && atom
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '@');
    if plain {
        out.push_str(atom);
        return;
    }
    out.push('\'');
    for c in atom.chars() {
        match c {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
    out.push('\'');
}

fn write_chars(out: &mut String, chars: impl Iterator<Item = char>) {
    for c in chars {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 32 || c as u32 == 127 => {
                let _ = write!(out, "\\{:03o}", c as u32);
            }
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin(text: &str) -> Term {
        Term::Binary(text.as_bytes().to_vec())
    }

    #[test]
    fn metadata_terms_parse_like_the_files_clients_write() {
        let text = "{<<\"name\">>,<<\"hello\">>}.\n{<<\"version\">>,<<\"1.0.0\">>}.\n\
                    {<<\"description\">>,<<\"Grüße \\\"quoted\\\"\\n\"/utf8>>}.\n\
                    {<<\"licenses\">>,[<<\"MIT\">>,<<\"Apache-2.0\">>]}.\n\
                    {<<\"requirements\">>,\n  [[{<<\"app\">>,<<\"jason\">>},\n    {<<\"name\">>,<<\"jason\">>},\n    {<<\"optional\">>,false},\n    {<<\"repository\">>,<<\"hexpm\">>},\n    {<<\"requirement\">>,<<\"~> 1.0\">>}]]}.\n\
                    % a comment\n{<<\"extra\">>,#{<<\"n\">> => -12, 'quoted atom' => 1.5e3, x => 16#ff, c => $a}}.\n\
                    {<<\"files\">>,[<<\"lib\">>, <<\"lib/a\" \"b.ex\">>, <<>>, <<1,2>>, \"chars\" \"more\", []]}.\n";
        let terms = parse(text).unwrap();
        assert_eq!(terms.len(), 7);
        assert_eq!(terms[0], Term::Tuple(vec![bin("name"), bin("hello")]));
        let description = &terms[2];
        assert_eq!(
            description,
            &Term::Tuple(vec![bin("description"), bin("Grüße \"quoted\"\n")])
        );
        let requirement = match &terms[4] {
            Term::Tuple(parts) => parts[1].items().unwrap()[0].clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(requirement.get("app").unwrap().text().unwrap(), "jason");
        assert_eq!(requirement.get("optional").unwrap().boolean(), Some(false));
        let extra = match &terms[5] {
            Term::Tuple(parts) => parts[1].clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(extra.get("n"), Some(Term::Integer(-12)));
        assert_eq!(extra.get("quoted atom"), Some(Term::Float(1500.0)));
        assert_eq!(extra.get("x"), Some(Term::Integer(255)));
        assert_eq!(extra.get("c"), Some(Term::Integer(97)));
        let files = match &terms[6] {
            Term::Tuple(parts) => parts[1].items().unwrap(),
            other => panic!("{other:?}"),
        };
        assert_eq!(files[1], bin("lib/ab.ex"));
        assert_eq!(files[2], bin(""));
        assert_eq!(files[3], Term::Binary(vec![1, 2]));
        assert_eq!(files[4].text().unwrap(), "charsmore");
        assert_eq!(files[5].text().unwrap(), "");
    }

    #[test]
    fn broken_terms_name_their_line() {
        for (text, line) in [
            ("{<<\"name\">>,<<\"a\">>}", 1),
            ("{<<\"name\">>,\n<<\"a\">>.", 2),
            ("{a, \"open}.", 1),
            ("[1|2].", 1),
            ("{<<\"a\"/float>>}.", 1),
            ("{a, ?MACRO}.", 1),
        ] {
            let error = parse(text).unwrap_err();
            assert_eq!(error.line, line, "{text}: {error}");
        }
        let deep = format!("{}{}.", "[".repeat(100), "]".repeat(100));
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn printed_terms_parse_back() {
        let term = Term::Tuple(vec![
            bin("links"),
            Term::List(vec![Term::Tuple(vec![
                bin("Source"),
                bin("https://example.org/ä\"\\"),
            ])]),
            Term::Atom("true".to_owned()),
            Term::Atom("Odd atom".to_owned()),
            Term::Integer(-7),
            Term::Binary(vec![255, 0]),
            Term::Map(vec![(bin("k"), Term::List(vec![]))]),
            Term::Chars(vec!['h' as u32, 'i' as u32]),
        ]);
        let printed = print(&term);
        assert!(printed.contains("\"/utf8>>"), "{printed}");
        assert_eq!(parse(&format!("{printed}.")).unwrap(), vec![term]);
    }
}
