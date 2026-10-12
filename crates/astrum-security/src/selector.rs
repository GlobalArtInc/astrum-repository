use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use regex::{Regex, RegexBuilder};
use regex_syntax::ast::{self, Ast, LiteralKind, SpecialLiteralKind};

pub const CSEL: &str = "csel";
pub const MAX_LENGTH: usize = 8192;

const MAX_DEPTH: usize = 64;
const REGEX_SIZE_LIMIT: usize = 1 << 20;
const FLAGS: &str = "inline flags (only a leading (?i) is supported)";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Format,
    Path,
}

impl Field {
    pub fn name(self) -> &'static str {
        match self {
            Field::Format => "format",
            Field::Path => "path",
        }
    }

    fn parse(name: &str) -> Option<Self> {
        [Field::Format, Field::Path]
            .into_iter()
            .find(|field| field.name() == name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Equals,
    NotEquals,
    Matches,
    StartsWith,
}

impl Operator {
    pub fn symbol(self) -> &'static str {
        match self {
            Operator::Equals => "==",
            Operator::NotEquals => "!=",
            Operator::Matches => "=~",
            Operator::StartsWith => "=^",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comparison {
    Equals,
    NotEquals,
    StartsWith,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Operand {
    Field(Field),
    Text(String),
}

#[derive(Clone, Debug)]
struct Pattern {
    body: String,
    case_insensitive: bool,
    regex: Regex,
}

#[derive(Clone, Debug)]
enum Expr {
    All(Vec<Expr>),
    Any(Vec<Expr>),
    Compare {
        left: Operand,
        comparison: Comparison,
        right: Operand,
    },
    Match {
        left: Operand,
        pattern: Box<Pattern>,
    },
}

#[derive(Clone, Debug)]
pub struct Selector {
    expression: String,
    root: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Invalid CSEL: {message} at line {line} column {column}")]
pub struct SelectorError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Columns {
    format: &'static str,
    path: &'static str,
}

impl Columns {
    pub fn new(format: &'static str, path: &'static str) -> Self {
        for column in [format, path] {
            assert!(
                !column.is_empty()
                    && column
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.'),
                "\"{column}\" is not a plain column reference"
            );
        }
        Self { format, path }
    }

    fn get(&self, field: Field) -> &'static str {
        match field {
            Field::Format => self.format,
            Field::Path => self.path,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SqlFragment {
    pub sql: String,
    pub values: Vec<String>,
}

impl Selector {
    pub fn parse(expression: &str) -> Result<Self, SelectorError> {
        if expression.chars().count() > MAX_LENGTH {
            return Err(SelectorError {
                message: format!("the expression is longer than {MAX_LENGTH} characters"),
                line: 1,
                column: 1,
            });
        }
        let (tokens, end) = tokenize(expression)?;
        let mut parser = Parser {
            tokens,
            position: 0,
            end,
            depth: 0,
        };
        let root = parser.any()?;
        if let Some(extra) = parser.peek() {
            return Err(extra.error(format!("unexpected '{}'", extra.text)));
        }
        Ok(Self {
            expression: expression.to_owned(),
            root,
        })
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn matches(&self, format: &str, path: &str) -> bool {
        self.root.matches(format, path)
    }

    pub fn to_sql(&self, columns: &Columns, first_parameter: usize) -> SqlFragment {
        let mut builder = SqlBuilder {
            columns,
            next: first_parameter,
            fragment: SqlFragment::default(),
        };
        let sql = builder.expr(&self.root);
        builder.fragment.sql = sql;
        builder.fragment
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.expression)
    }
}

impl Expr {
    fn matches(&self, format: &str, path: &str) -> bool {
        let value = |operand: &Operand| -> String {
            match operand {
                Operand::Field(Field::Format) => format.to_owned(),
                Operand::Field(Field::Path) => path.to_owned(),
                Operand::Text(text) => text.clone(),
            }
        };
        match self {
            Expr::All(items) => items.iter().all(|item| item.matches(format, path)),
            Expr::Any(items) => items.iter().any(|item| item.matches(format, path)),
            Expr::Compare {
                left,
                comparison,
                right,
            } => {
                let (left, right) = (value(left), value(right));
                match comparison {
                    Comparison::Equals => left == right,
                    Comparison::NotEquals => left != right,
                    Comparison::StartsWith => left.starts_with(&right),
                }
            }
            Expr::Match { left, pattern } => pattern.regex.is_match(&value(left)),
        }
    }
}

struct SqlBuilder<'a> {
    columns: &'a Columns,
    next: usize,
    fragment: SqlFragment,
}

impl SqlBuilder<'_> {
    fn bind(&mut self, value: String) -> String {
        let placeholder = format!("${}", self.next);
        self.next += 1;
        self.fragment.values.push(value);
        placeholder
    }

    fn operand(&mut self, operand: &Operand) -> String {
        match operand {
            Operand::Field(field) => self.columns.get(*field).to_owned(),
            Operand::Text(text) => self.bind(text.clone()),
        }
    }

    fn expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::All(items) => self.join(items, " AND "),
            Expr::Any(items) => self.join(items, " OR "),
            Expr::Compare {
                left,
                comparison,
                right,
            } => {
                let left = self.operand(left);
                let right = self.operand(right);
                match comparison {
                    Comparison::Equals => format!("{left} = {right}"),
                    Comparison::NotEquals => format!("{left} <> {right}"),
                    Comparison::StartsWith => format!("starts_with({left}, {right})"),
                }
            }
            Expr::Match { left, pattern } => {
                let left = self.operand(left);
                let flags = if pattern.case_insensitive { "(?i)" } else { "" };
                let right = self.bind(format!("{flags}^(?:{})$", pattern.body));
                format!("{left} ~ {right}")
            }
        }
    }

    fn join(&mut self, items: &[Expr], separator: &str) -> String {
        let parts: Vec<String> = items.iter().map(|item| self.expr(item)).collect();
        format!("({})", parts.join(separator))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Open,
    Close,
    And,
    Or,
    Operator(Operator),
    Field(Field),
    Text(String),
}

#[derive(Clone, Debug)]
struct Spanned {
    token: Token,
    text: String,
    line: usize,
    column: usize,
}

impl Spanned {
    fn error(&self, message: String) -> SelectorError {
        SelectorError {
            message,
            line: self.line,
            column: self.column,
        }
    }
}

struct Cursor<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl Cursor<'_> {
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        let next = self.chars.next()?;
        if next == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(next)
    }
}

fn error_at(line: usize, column: usize, message: String) -> SelectorError {
    SelectorError {
        message,
        line,
        column,
    }
}

fn tokenize(input: &str) -> Result<(Vec<Spanned>, (usize, usize)), SelectorError> {
    let mut cursor = Cursor {
        chars: input.chars().peekable(),
        line: 1,
        column: 1,
    };
    let mut tokens = Vec::new();
    while let Some(c) = cursor.peek() {
        let (line, column) = (cursor.line, cursor.column);
        if c.is_whitespace() {
            cursor.next();
            continue;
        }
        let spanned = |token: Token, text: &str| Spanned {
            token,
            text: text.to_owned(),
            line,
            column,
        };
        let token = match c {
            '(' => {
                cursor.next();
                spanned(Token::Open, "(")
            }
            ')' => {
                cursor.next();
                spanned(Token::Close, ")")
            }
            '=' | '!' | '&' | '|' => {
                cursor.next();
                let second = cursor.peek();
                let (token, text) = match (c, second) {
                    ('=', Some('=')) => (Token::Operator(Operator::Equals), "=="),
                    ('=', Some('~')) => (Token::Operator(Operator::Matches), "=~"),
                    ('=', Some('^')) => (Token::Operator(Operator::StartsWith), "=^"),
                    ('!', Some('=')) => (Token::Operator(Operator::NotEquals), "!="),
                    ('&', Some('&')) => (Token::And, "&&"),
                    ('|', Some('|')) => (Token::Or, "||"),
                    ('!', _) => {
                        return Err(error_at(
                            line,
                            column,
                            "negation is not supported".to_owned(),
                        ));
                    }
                    _ => {
                        return Err(error_at(line, column, format!("unexpected '{c}'")));
                    }
                };
                cursor.next();
                spanned(token, text)
            }
            '"' | '\'' => {
                cursor.next();
                let text = string(&mut cursor, c, line, column)?;
                spanned(Token::Text(text.clone()), &text)
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut word = String::new();
                while let Some(c) = cursor.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '.' {
                        word.push(c);
                        cursor.next();
                    } else {
                        break;
                    }
                }
                let token = match word.as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    name => match Field::parse(name) {
                        Some(field) => Token::Field(field),
                        None => {
                            return Err(error_at(
                                line,
                                column,
                                format!("unknown identifier '{name}', use format or path"),
                            ));
                        }
                    },
                };
                spanned(token, &word)
            }
            c if c.is_ascii_digit() => {
                return Err(error_at(
                    line,
                    column,
                    "numbers are not supported, quote the value".to_owned(),
                ));
            }
            c => {
                return Err(error_at(line, column, format!("unexpected '{c}'")));
            }
        };
        tokens.push(token);
    }
    Ok((tokens, (cursor.line, cursor.column)))
}

fn string(
    cursor: &mut Cursor<'_>,
    quote: char,
    line: usize,
    column: usize,
) -> Result<String, SelectorError> {
    let unclosed = || SelectorError {
        message: "the string starting here is not closed".to_owned(),
        line,
        column,
    };
    let embedded = || SelectorError {
        message: "a string must not contain a quote (\" or ')".to_owned(),
        line,
        column,
    };
    let mut text = String::new();
    loop {
        match cursor.next().ok_or_else(unclosed)? {
            c if c == quote => return Ok(text),
            '"' | '\'' => return Err(embedded()),
            '\\' => match cursor.next().ok_or_else(unclosed)? {
                '\\' => text.push('\\'),
                '"' | '\'' => return Err(embedded()),
                other => {
                    text.push('\\');
                    text.push(other);
                }
            },
            c => text.push(c),
        }
    }
}

struct Parser {
    tokens: Vec<Spanned>,
    position: usize,
    end: (usize, usize),
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Spanned> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<Spanned> {
        let token = self.tokens.get(self.position).cloned();
        self.position += 1;
        token
    }

    fn ended(&self) -> SelectorError {
        SelectorError {
            message: "the expression ends too early".to_owned(),
            line: self.end.0,
            column: self.end.1,
        }
    }

    fn any(&mut self) -> Result<Expr, SelectorError> {
        let mut items = vec![self.all()?];
        while self.peek().is_some_and(|next| next.token == Token::Or) {
            self.advance();
            items.push(self.all()?);
        }
        Ok(match items.len() {
            1 => items.remove(0),
            _ => Expr::Any(items),
        })
    }

    fn all(&mut self) -> Result<Expr, SelectorError> {
        let mut items = vec![self.primary()?];
        while self.peek().is_some_and(|next| next.token == Token::And) {
            self.advance();
            items.push(self.primary()?);
        }
        Ok(match items.len() {
            1 => items.remove(0),
            _ => Expr::All(items),
        })
    }

    fn primary(&mut self) -> Result<Expr, SelectorError> {
        let next = self.peek().cloned().ok_or_else(|| self.ended())?;
        if next.token != Token::Open {
            return self.comparison();
        }
        self.advance();
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(next.error("the parentheses are nested too deeply".to_owned()));
        }
        let inner = self.any()?;
        match self.advance() {
            Some(close) if close.token == Token::Close => {}
            Some(other) => return Err(other.error(format!("unexpected '{}'", other.text))),
            None => return Err(self.ended()),
        }
        self.depth -= 1;
        Ok(inner)
    }

    fn operand(&mut self) -> Result<(Operand, Spanned), SelectorError> {
        let next = self.advance().ok_or_else(|| self.ended())?;
        match &next.token {
            Token::Field(field) => Ok((Operand::Field(*field), next)),
            Token::Text(text) => Ok((Operand::Text(text.clone()), next)),
            _ => Err(next.error(format!("unexpected '{}'", next.text))),
        }
    }

    fn comparison(&mut self) -> Result<Expr, SelectorError> {
        let (left, first) = self.operand()?;
        let operator = match self.advance() {
            Some(Spanned {
                token: Token::Operator(operator),
                ..
            }) => operator,
            Some(other) => return Err(other.error(format!("unexpected '{}'", other.text))),
            None => {
                return Err(first.error(format!(
                    "'{}' needs a comparison: ==, !=, =~ or =^",
                    first.text
                )));
            }
        };
        let (right, second) = self.operand()?;
        let comparison = match operator {
            Operator::Equals => Comparison::Equals,
            Operator::NotEquals => Comparison::NotEquals,
            Operator::StartsWith | Operator::Matches => {
                let Operand::Text(text) = &right else {
                    return Err(second.error(format!(
                        "the right side of {} must be a quoted string",
                        operator.symbol()
                    )));
                };
                if operator == Operator::Matches {
                    let pattern = compile(text).map_err(|message| second.error(message))?;
                    return Ok(Expr::Match {
                        left,
                        pattern: Box::new(pattern),
                    });
                }
                Comparison::StartsWith
            }
        };
        Ok(Expr::Compare {
            left,
            comparison,
            right,
        })
    }
}

fn compile(source: &str) -> Result<Pattern, String> {
    let (case_insensitive, body) = match source.strip_prefix("(?i)") {
        Some(rest) => (true, rest),
        None => (false, source),
    };
    let invalid = |reason: &dyn fmt::Display| {
        format!("the regular expression '{source}' is not valid: {reason}")
    };
    let tree = ast::parse::Parser::new()
        .parse(body)
        .map_err(|error| invalid(error.kind()))?;
    ast::visit(&tree, Portable).map_err(|what| {
        format!(
            "the regular expression '{source}' uses {what}, which content selectors do not support"
        )
    })?;
    let regex = RegexBuilder::new(&format!("^(?:{body})$"))
        .case_insensitive(case_insensitive)
        .size_limit(REGEX_SIZE_LIMIT)
        .build()
        .map_err(|error| match error {
            regex::Error::CompiledTooBig(_) => invalid(&"it is too large"),
            _ => invalid(&"it cannot be compiled"),
        })?;
    Ok(Pattern {
        body: body.to_owned(),
        case_insensitive,
        regex,
    })
}

struct Portable;

impl ast::Visitor for Portable {
    type Output = ();
    type Err = &'static str;

    fn finish(self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn visit_pre(&mut self, tree: &Ast) -> Result<(), Self::Err> {
        match tree {
            Ast::Flags(_) => Err(FLAGS),
            Ast::Group(group) => match &group.kind {
                ast::GroupKind::CaptureName { .. } => Err("named groups"),
                ast::GroupKind::NonCapturing(flags) if !flags.items.is_empty() => Err(FLAGS),
                _ => Ok(()),
            },
            Ast::Assertion(assertion) => match assertion.kind {
                ast::AssertionKind::StartLine | ast::AssertionKind::EndLine => Ok(()),
                _ => Err("word boundaries, \\A or \\z (use ^ and $)"),
            },
            Ast::ClassUnicode(_) => Err("Unicode classes (\\p, \\P)"),
            Ast::Literal(literal) => portable_literal(literal),
            _ => Ok(()),
        }
    }

    fn visit_class_set_item_pre(&mut self, item: &ast::ClassSetItem) -> Result<(), Self::Err> {
        match item {
            ast::ClassSetItem::Unicode(_) => Err("Unicode classes (\\p, \\P)"),
            ast::ClassSetItem::Perl(perl) if perl.negated => {
                Err("negated classes such as \\D inside brackets")
            }
            ast::ClassSetItem::Literal(literal) => portable_literal(literal),
            _ => Ok(()),
        }
    }

    fn visit_class_set_binary_op_pre(
        &mut self,
        _operation: &ast::ClassSetBinaryOp,
    ) -> Result<(), Self::Err> {
        Err("class operations such as && inside brackets")
    }
}

fn portable_literal(literal: &ast::Literal) -> Result<(), &'static str> {
    match literal.kind {
        LiteralKind::HexFixed(_) | LiteralKind::HexBrace(_) => Err("hexadecimal escapes"),
        LiteralKind::Octal => Err("octal escapes"),
        LiteralKind::Special(SpecialLiteralKind::Space) => Err("escaped spaces"),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(expression: &str) -> Selector {
        Selector::parse(expression).unwrap_or_else(|error| panic!("{expression}: {error}"))
    }

    fn refused(expression: &str) -> SelectorError {
        match Selector::parse(expression) {
            Ok(_) => panic!("{expression} was accepted"),
            Err(error) => error,
        }
    }

    #[test]
    fn comparisons() {
        let raw = parse("format == \"raw\"");
        assert!(raw.matches("raw", "/a"));
        assert!(!raw.matches("npm", "/a"));
        let not_raw = parse("format != 'raw'");
        assert!(!not_raw.matches("raw", "/a"));
        assert!(not_raw.matches("npm", "/a"));
        let prefix = parse("path =^ \"/org/astrum/\"");
        assert!(prefix.matches("maven2", "/org/astrum/core/1.0/core-1.0.jar"));
        assert!(!prefix.matches("maven2", "/org/other/core.jar"));
        let swapped = parse("\"raw\" == format");
        assert!(swapped.matches("raw", "/"));
        let fields = parse("format == path");
        assert!(fields.matches("x", "x"));
        assert!(!fields.matches("x", "y"));
        assert!(parse("format == \"\"").matches("", "/"));
    }

    #[test]
    fn regular_expressions_match_the_whole_value() {
        let selector = parse("path =~ \"/org/.*\\.jar\"");
        assert!(selector.matches("maven2", "/org/astrum/a.jar"));
        assert!(!selector.matches("maven2", "/org/astrum/a.jar.sha1"));
        assert!(!selector.matches("maven2", "/x/org/a.jar"));
        let alternation = parse("path =~ \"a|b\"");
        assert!(alternation.matches("raw", "a"));
        assert!(!alternation.matches("raw", "ab"));
        let digits = parse("path =~ '\\d+'");
        assert!(digits.matches("raw", "123"));
        let escaped = parse("path =~ \"\\\\d+\"");
        assert!(escaped.matches("raw", "42"));
        let insensitive = parse("path =~ \"(?i)/ORG/.*\"");
        assert!(insensitive.matches("raw", "/org/a"));
        assert!(parse("path =~ \"[[:alpha:]]+\"").matches("raw", "abc"));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        let selector = parse("format == \"a\" || format == \"b\" && path == \"/x\"");
        assert!(selector.matches("a", "/y"));
        assert!(selector.matches("b", "/x"));
        assert!(!selector.matches("b", "/y"));
        let grouped = parse("(format == \"a\" or format == \"b\") and path == \"/x\"");
        assert!(!grouped.matches("a", "/y"));
        assert!(grouped.matches("a", "/x"));
        let nested = parse("((format == \"a\"))");
        assert!(nested.matches("a", "/"));
        let lines = parse("format == \"raw\"\n&& path =^ \"/a\"");
        assert!(lines.matches("raw", "/a/b"));
        assert!(parse("format == \"raw\" and(path == \"/x\")").matches("raw", "/x"));
    }

    #[test]
    fn errors_name_the_place() {
        let error = refused("format == \"raw\" AND path =^ \"/a\"");
        assert_eq!((error.line, error.column), (1, 17));
        assert!(error.message.contains("unknown identifier 'AND'"));
        let error = refused("format == \"raw\" &&");
        assert_eq!(error.message, "the expression ends too early");
        assert_eq!(error.column, 19);
        let error = refused("format === \"raw\"");
        assert_eq!(
            (error.column, error.message.as_str()),
            (10, "unexpected '='")
        );
        let error = refused("format == \"raw\")");
        assert_eq!(
            (error.column, error.message.as_str()),
            (16, "unexpected ')'")
        );
        let error = refused("format == \"raw\"\n  || name == \"x\"");
        assert_eq!((error.line, error.column), (2, 6));
        assert_eq!(
            error.to_string(),
            "Invalid CSEL: unknown identifier 'name', use format or path at line 2 column 6"
        );
    }

    #[test]
    fn unsupported_forms_are_refused() {
        for expression in [
            "",
            "   ",
            "format",
            "format == raw",
            "Format == \"raw\"",
            "coordinate.groupId == \"x\"",
            "!(format == \"raw\")",
            "not format == \"raw\"",
            "format == 1",
            "format == \"raw\" & path == \"/x\"",
            "format == \"raw\" ; path == \"/x\"",
            "format == \"raw\" path == \"/x\"",
            "(format == \"raw\"",
            "path >= \"a\"",
            "format == `raw`",
            "format == \"a\\\"b\"",
            "format == 'a\\'b'",
            "format == 'a\"b'",
            "path =~ \"a\\\"",
            "path == \"unclosed",
            "path =~ format",
            "path =^ format",
            "format == \"raw\" andpath == \"x\"",
        ] {
            refused(expression);
        }
        let deep = format!("{}format == \"a\"{}", "(".repeat(100), ")".repeat(100));
        assert!(refused(&deep).message.contains("nested too deeply"));
        let long = format!("path == \"{}\"", "a".repeat(MAX_LENGTH));
        assert!(refused(&long).message.contains("longer than"));
    }

    #[test]
    fn regular_expressions_must_compile_and_port_to_sql() {
        for (pattern, reason) in [
            ("[a", "not valid"),
            ("a(b", "not valid"),
            ("a)", "not valid"),
            ("*a", "not valid"),
            ("(?<n>a)", "named groups"),
            ("a(?i)b", "inline flags"),
            ("(?s:a.)", "inline flags"),
            ("\\bword\\b", "word boundaries"),
            ("\\Aa\\z", "word boundaries"),
            ("\\p{L}+", "Unicode classes"),
            ("[\\p{L}]", "Unicode classes"),
            ("[\\D]", "negated classes"),
            ("[a-z&&[^b]]", "class operations"),
            ("\\x41", "hexadecimal"),
        ] {
            let error = refused(&format!("path =~ \"{pattern}\""));
            assert!(error.message.contains(reason), "{pattern}: {error}");
            assert_eq!(error.column, 9, "{pattern}");
        }
    }

    #[test]
    fn sql_binds_every_literal() {
        let columns = Columns::new("a.format", "a.path");
        let selector = parse(
            "format == \"raw\" && (path =^ \"/a); DROP TABLE x; --\" || path =~ \"(?i)/b/.*\") && \"x\" != format",
        );
        let fragment = selector.to_sql(&columns, 3);
        assert_eq!(
            fragment.sql,
            "(a.format = $3 AND (starts_with(a.path, $4) OR a.path ~ $5) AND $6 <> a.format)"
        );
        assert_eq!(
            fragment.values,
            ["raw", "/a); DROP TABLE x; --", "(?i)^(?:/b/.*)$", "x"]
        );
        let single = parse("format == path").to_sql(&columns, 1);
        assert_eq!(single.sql, "a.format = a.path");
        assert!(single.values.is_empty());
    }

    #[test]
    #[should_panic(expected = "not a plain column reference")]
    fn columns_are_plain_identifiers() {
        Columns::new("format; DROP TABLE x", "path");
    }
}
