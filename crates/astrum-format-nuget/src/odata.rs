use std::cmp::Ordering;
use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};

use crate::hosted::Stored;
use crate::route::{Feed, odata_string, param};
use crate::urls::Urls;
use crate::version::{self, Version};
use crate::xml::{self, Element, escape};

pub const PAGE: usize = 100;
pub const MAX_TOP: usize = 1000;
const ATOM: &str = "http://www.w3.org/2005/Atom";
const DATA: &str = "http://schemas.microsoft.com/ado/2007/08/dataservices";
const META: &str = "http://schemas.microsoft.com/ado/2007/08/dataservices/metadata";
const SCHEME: &str = "http://schemas.microsoft.com/ado/2007/08/dataservices/scheme";
const ENTITY: &str = "NuGetGallery.OData.V2FeedPackage";

#[derive(Clone, Debug, PartialEq)]
pub enum Prop {
    Text(Option<String>),
    Bool(bool),
    Int32(i64),
    Int64(i64),
    Date(Option<String>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    pub props: Vec<(String, Prop)>,
}

fn edm_date(value: DateTime<FixedOffset>) -> String {
    value
        .with_timezone(&Utc)
        .naive_utc()
        .format("%Y-%m-%dT%H:%M:%S%.3f")
        .to_string()
}

impl Entry {
    pub fn get(&self, name: &str) -> Option<&Prop> {
        self.props
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }

    pub fn text(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            Prop::Text(Some(value)) | Prop::Date(Some(value)) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn flag(&self, name: &str) -> bool {
        matches!(self.get(name), Some(Prop::Bool(true)))
    }

    pub fn id(&self) -> &str {
        self.text("Id").unwrap_or_default()
    }

    pub fn version(&self) -> &str {
        self.text("NormalizedVersion")
            .or_else(|| self.text("Version"))
            .unwrap_or_default()
    }

    pub fn parsed_version(&self) -> Option<Version> {
        Version::parse(self.version())
    }

    pub fn is_semver2(&self) -> bool {
        Version::parse(self.text("Version").unwrap_or_default())
            .or_else(|| self.parsed_version())
            .is_some_and(|v| v.is_semver2())
    }

    fn set(&mut self, name: &str, value: Prop) {
        match self
            .props
            .iter_mut()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
        {
            Some((_, slot)) => *slot = value,
            None => self.props.push((name.to_owned(), value)),
        }
    }

    pub fn from_stored(stored: &Stored, total_downloads: i64) -> Entry {
        let metadata = &stored.metadata;
        let text = |value: &Option<String>| Prop::Text(value.clone().filter(|v| !v.is_empty()));
        let hash = stored
            .package
            .as_ref()
            .and_then(|asset| hex::decode(&asset.sha512).ok())
            .map(|bytes| STANDARD.encode(bytes));
        let size = stored
            .package
            .as_ref()
            .map(|asset| asset.size)
            .unwrap_or_default();
        let tags = (!metadata.tags.is_empty()).then(|| metadata.tags.join(" "));
        let props = vec![
            ("Id", Prop::Text(Some(metadata.id.clone()))),
            ("Version", Prop::Text(Some(metadata.version.clone()))),
            (
                "NormalizedVersion",
                Prop::Text(Some(metadata.normalized_version.clone())),
            ),
            ("Authors", text(&metadata.authors)),
            ("Copyright", text(&metadata.copyright)),
            ("Created", Prop::Date(Some(edm_date(stored.published())))),
            ("Dependencies", Prop::Text(Some(metadata.v2_dependencies()))),
            ("Description", text(&metadata.description)),
            ("DownloadCount", Prop::Int32(total_downloads)),
            ("IconUrl", text(&metadata.icon_url)),
            ("IsLatestVersion", Prop::Bool(false)),
            ("IsAbsoluteLatestVersion", Prop::Bool(false)),
            ("IsPrerelease", Prop::Bool(metadata.is_prerelease)),
            ("Language", text(&metadata.language)),
            ("LastUpdated", Prop::Date(Some(edm_date(stored.updated())))),
            ("Published", Prop::Date(Some(edm_date(stored.published())))),
            ("PackageHash", Prop::Text(hash)),
            (
                "PackageHashAlgorithm",
                Prop::Text(Some("SHA512".to_owned())),
            ),
            ("PackageSize", Prop::Int64(size)),
            ("ProjectUrl", text(&metadata.project_url)),
            ("ReleaseNotes", text(&metadata.release_notes)),
            (
                "RequireLicenseAcceptance",
                Prop::Bool(metadata.require_license_acceptance),
            ),
            ("Summary", text(&metadata.summary)),
            ("Tags", Prop::Text(tags)),
            ("Title", text(&metadata.title)),
            ("VersionDownloadCount", Prop::Int32(stored.downloads())),
            ("MinClientVersion", text(&metadata.min_client_version)),
            ("LicenseUrl", text(&metadata.license_url)),
            ("Listed", Prop::Bool(true)),
        ];
        Entry {
            props: props
                .into_iter()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
        }
    }
}

pub fn from_stored(found: &[Stored]) -> Vec<Entry> {
    let mut totals: HashMap<String, i64> = HashMap::new();
    for stored in found {
        *totals.entry(stored.metadata.id.to_lowercase()).or_default() += stored.downloads();
    }
    found
        .iter()
        .map(|stored| {
            Entry::from_stored(
                stored,
                totals
                    .get(&stored.metadata.id.to_lowercase())
                    .copied()
                    .unwrap_or_default(),
            )
        })
        .collect()
}

pub fn mark_latest(entries: &mut [Entry]) {
    let mut latest: HashMap<String, (usize, Version)> = HashMap::new();
    let mut absolute: HashMap<String, (usize, Version)> = HashMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let Some(version) = entry.parsed_version() else {
            continue;
        };
        let id = entry.id().to_lowercase();
        let newer =
            |slot: Option<&(usize, Version)>| slot.is_none_or(|(_, known)| version > *known);
        if newer(absolute.get(&id)) {
            absolute.insert(id.clone(), (index, version.clone()));
        }
        if !version.is_prerelease() && newer(latest.get(&id)) {
            latest.insert(id, (index, version));
        }
    }
    for (index, entry) in entries.iter_mut().enumerate() {
        let id = entry.id().to_lowercase();
        let is = |map: &HashMap<String, (usize, Version)>| {
            map.get(&id).is_some_and(|(found, _)| *found == index)
        };
        let (is_latest, is_absolute) = (is(&latest), is(&absolute));
        entry.set("IsLatestVersion", Prop::Bool(is_latest));
        entry.set("IsAbsoluteLatestVersion", Prop::Bool(is_absolute));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Value),
    Property(String),
    Call(String, Vec<Expr>),
    Not(Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Str(String),
    Num(f64),
    Word(String),
    Open,
    Close,
    Comma,
}

fn tokens(text: &str) -> Option<Vec<Token>> {
    let mut found = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        match c {
            c if c.is_whitespace() => at += 1,
            '(' => {
                found.push(Token::Open);
                at += 1;
            }
            ')' => {
                found.push(Token::Close);
                at += 1;
            }
            ',' => {
                found.push(Token::Comma);
                at += 1;
            }
            '\'' => {
                let mut value = String::new();
                at += 1;
                loop {
                    let c = *chars.get(at)?;
                    at += 1;
                    if c == '\'' {
                        if chars.get(at) == Some(&'\'') {
                            value.push('\'');
                            at += 1;
                            continue;
                        }
                        break;
                    }
                    value.push(c);
                }
                found.push(Token::Str(value));
            }
            c if c.is_ascii_digit() || c == '-' => {
                let start = at;
                at += 1;
                while at < chars.len() && (chars[at].is_ascii_digit() || chars[at] == '.') {
                    at += 1;
                }
                let text: String = chars[start..at].iter().collect();
                found.push(Token::Num(text.parse().ok()?));
            }
            c if c.is_alphanumeric() || c == '_' || c == '$' => {
                let start = at;
                while at < chars.len()
                    && (chars[at].is_alphanumeric() || chars[at] == '_' || chars[at] == '.')
                {
                    at += 1;
                }
                found.push(Token::Word(chars[start..at].iter().collect()));
            }
            _ => return None,
        }
    }
    Some(found)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        self.at += 1;
        token
    }

    fn keyword(&self, word: &str) -> bool {
        matches!(self.peek(), Some(Token::Word(found)) if found.eq_ignore_ascii_case(word))
    }

    fn or(&mut self, depth: usize) -> Option<Expr> {
        let mut left = self.and(depth)?;
        while self.keyword("or") {
            self.at += 1;
            let right = self.and(depth)?;
            left = Expr::Binary("or".to_owned(), Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn and(&mut self, depth: usize) -> Option<Expr> {
        let mut left = self.not(depth)?;
        while self.keyword("and") {
            self.at += 1;
            let right = self.not(depth)?;
            left = Expr::Binary("and".to_owned(), Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn not(&mut self, depth: usize) -> Option<Expr> {
        if self.keyword("not") {
            self.at += 1;
            return Some(Expr::Not(Box::new(self.not(depth + 1)?)));
        }
        self.comparison(depth)
    }

    fn comparison(&mut self, depth: usize) -> Option<Expr> {
        let left = self.primary(depth)?;
        for operator in ["eq", "ne", "gt", "ge", "lt", "le"] {
            if self.keyword(operator) {
                self.at += 1;
                let right = self.primary(depth)?;
                return Some(Expr::Binary(
                    operator.to_owned(),
                    Box::new(left),
                    Box::new(right),
                ));
            }
        }
        Some(left)
    }

    fn primary(&mut self, depth: usize) -> Option<Expr> {
        if depth > 32 {
            return None;
        }
        match self.next()? {
            Token::Open => {
                let inner = self.or(depth + 1)?;
                (self.next()? == Token::Close).then_some(inner)
            }
            Token::Str(value) => Some(Expr::Literal(Value::Str(value))),
            Token::Num(value) => Some(Expr::Literal(Value::Num(value))),
            Token::Word(word) => {
                let lower = word.to_ascii_lowercase();
                match lower.as_str() {
                    "true" => return Some(Expr::Literal(Value::Bool(true))),
                    "false" => return Some(Expr::Literal(Value::Bool(false))),
                    "null" => return Some(Expr::Literal(Value::Null)),
                    _ => {}
                }
                if self.peek() == Some(&Token::Open) {
                    self.at += 1;
                    let mut arguments = Vec::new();
                    if self.peek() == Some(&Token::Close) {
                        self.at += 1;
                    } else {
                        loop {
                            arguments.push(self.or(depth + 1)?);
                            match self.next()? {
                                Token::Comma => continue,
                                Token::Close => break,
                                _ => return None,
                            }
                        }
                    }
                    return Some(Expr::Call(lower, arguments));
                }
                Some(Expr::Property(word))
            }
            _ => None,
        }
    }
}

pub fn parse_expression(text: &str) -> Option<Expr> {
    let mut parser = Parser {
        tokens: tokens(text)?,
        at: 0,
    };
    let expression = parser.or(0)?;
    (parser.at == parser.tokens.len()).then_some(expression)
}

fn property(entry: &Entry, name: &str) -> Value {
    match entry.get(name) {
        Some(Prop::Text(Some(value)) | Prop::Date(Some(value))) => Value::Str(value.clone()),
        Some(Prop::Bool(value)) => Value::Bool(*value),
        Some(Prop::Int32(value) | Prop::Int64(value)) => Value::Num(*value as f64),
        _ => Value::Null,
    }
}

fn string(value: &Value) -> Option<String> {
    match value {
        Value::Str(text) => Some(text.clone()),
        Value::Num(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Null => None,
    }
}

fn compare(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Str(left), Value::Str(right)) => {
            match (Version::parse(left), Version::parse(right)) {
                (Some(left), Some(right)) => Some(left.cmp(&right)),
                _ => Some(left.to_lowercase().cmp(&right.to_lowercase())),
            }
        }
        (Value::Num(left), Value::Num(right)) => left.partial_cmp(right),
        (Value::Bool(left), Value::Bool(right)) => Some(left.cmp(right)),
        (Value::Null, Value::Null) => Some(Ordering::Equal),
        (Value::Null, _) => Some(Ordering::Less),
        (_, Value::Null) => Some(Ordering::Greater),
        _ => None,
    }
}

fn evaluate(expression: &Expr, entry: &Entry) -> Value {
    match expression {
        Expr::Literal(value) => value.clone(),
        Expr::Property(name) => property(entry, name),
        Expr::Not(inner) => match evaluate(inner, entry) {
            Value::Bool(value) => Value::Bool(!value),
            _ => Value::Null,
        },
        Expr::Binary(operator, left, right) => {
            let left = evaluate(left, entry);
            let right = evaluate(right, entry);
            match operator.as_str() {
                "and" => Value::Bool(left == Value::Bool(true) && right == Value::Bool(true)),
                "or" => Value::Bool(left == Value::Bool(true) || right == Value::Bool(true)),
                "eq" | "ne" => {
                    let equal = match (&left, &right) {
                        (Value::Str(a), Value::Str(b)) => {
                            a.eq_ignore_ascii_case(b) || version::same(a, b)
                        }
                        _ => compare(&left, &right) == Some(Ordering::Equal),
                    };
                    Value::Bool(equal == (operator == "eq"))
                }
                other => match compare(&left, &right) {
                    Some(ordering) => Value::Bool(match other {
                        "gt" => ordering == Ordering::Greater,
                        "ge" => ordering != Ordering::Less,
                        "lt" => ordering == Ordering::Less,
                        _ => ordering != Ordering::Greater,
                    }),
                    None => Value::Null,
                },
            }
        }
        Expr::Call(name, arguments) => {
            let values: Vec<Value> = arguments
                .iter()
                .map(|argument| evaluate(argument, entry))
                .collect();
            let text = |index: usize| values.get(index).and_then(string);
            match (name.as_str(), values.len()) {
                ("tolower", 1) => text(0)
                    .map(|value| Value::Str(value.to_lowercase()))
                    .unwrap_or(Value::Null),
                ("toupper", 1) => text(0)
                    .map(|value| Value::Str(value.to_uppercase()))
                    .unwrap_or(Value::Null),
                ("trim", 1) => text(0)
                    .map(|value| Value::Str(value.trim().to_owned()))
                    .unwrap_or(Value::Null),
                ("length", 1) => text(0)
                    .map(|value| Value::Num(value.chars().count() as f64))
                    .unwrap_or(Value::Null),
                ("substringof", 2) => match (text(0), text(1)) {
                    (Some(needle), Some(haystack)) => {
                        Value::Bool(haystack.to_lowercase().contains(&needle.to_lowercase()))
                    }
                    _ => Value::Bool(false),
                },
                ("startswith", 2) => match (text(0), text(1)) {
                    (Some(haystack), Some(prefix)) => {
                        Value::Bool(haystack.to_lowercase().starts_with(&prefix.to_lowercase()))
                    }
                    _ => Value::Bool(false),
                },
                ("endswith", 2) => match (text(0), text(1)) {
                    (Some(haystack), Some(suffix)) => {
                        Value::Bool(haystack.to_lowercase().ends_with(&suffix.to_lowercase()))
                    }
                    _ => Value::Bool(false),
                },
                ("indexof", 2) => match (text(0), text(1)) {
                    (Some(haystack), Some(needle)) => Value::Num(
                        haystack
                            .to_lowercase()
                            .find(&needle.to_lowercase())
                            .map(|at| at as f64)
                            .unwrap_or(-1.0),
                    ),
                    _ => Value::Null,
                },
                ("concat", 2) => Value::Str(format!(
                    "{}{}",
                    text(0).unwrap_or_default(),
                    text(1).unwrap_or_default()
                )),
                _ => Value::Null,
            }
        }
    }
}

pub fn keep(expression: &Expr, entry: &Entry) -> bool {
    evaluate(expression, entry) == Value::Bool(true)
}

pub fn order(entries: &mut [Entry], orderby: &str) {
    let mut keys: Vec<(Expr, bool)> = Vec::new();
    for part in orderby.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let lower = part.to_ascii_lowercase();
        let (text, descending) = if let Some(stripped) = lower.strip_suffix(" desc") {
            (&part[..stripped.len()], true)
        } else if let Some(stripped) = lower.strip_suffix(" asc") {
            (&part[..stripped.len()], false)
        } else {
            (part, false)
        };
        match parse_expression(text) {
            Some(expression) => keys.push((expression, descending)),
            None => return,
        }
    }
    if keys.is_empty() {
        return;
    }
    entries.sort_by(|left, right| {
        for (expression, descending) in &keys {
            let ordering = compare(&evaluate(expression, left), &evaluate(expression, right))
                .unwrap_or(Ordering::Equal);
            let ordering = if *descending {
                ordering.reverse()
            } else {
                ordering
            };
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        Ordering::Equal
    });
}

pub fn default_order(entries: &mut [Entry]) {
    entries.sort_by(|left, right| {
        left.id()
            .to_lowercase()
            .cmp(&right.id().to_lowercase())
            .then_with(|| version::compare_text(left.version(), right.version()))
    });
}

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub feed: Feed,
    pub count: bool,
    pub id: Option<String>,
    pub search_term: Option<String>,
    pub prerelease: bool,
    pub semver2: bool,
    pub filter: Option<Expr>,
    pub orderby: Option<String>,
    pub skip: usize,
    pub top: Option<usize>,
}

impl Request {
    pub fn parse(feed: Feed, count: bool, pairs: &[(String, String)]) -> Request {
        Request {
            feed,
            count,
            id: odata_string(pairs, "id"),
            search_term: odata_string(pairs, "searchTerm")
                .map(|term| term.trim().to_owned())
                .filter(|term| !term.is_empty()),
            prerelease: param(pairs, "includePrerelease")
                .is_some_and(|value| value.eq_ignore_ascii_case("true")),
            semver2: param(pairs, "semVerLevel").is_some_and(|level| level.trim().starts_with('2')),
            filter: param(pairs, "$filter").and_then(|filter| parse_expression(&filter)),
            orderby: param(pairs, "$orderby"),
            skip: param(pairs, "$skip")
                .and_then(|value| value.trim().parse().ok())
                .unwrap_or(0),
            top: param(pairs, "$top")
                .and_then(|value| value.trim().parse().ok())
                .map(|top: usize| top.min(MAX_TOP)),
        }
    }

    pub fn select(&self, mut entries: Vec<Entry>) -> Vec<Entry> {
        entries.retain(|entry| self.semver2 || !entry.is_semver2());
        if let Feed::FindPackagesById = self.feed {
            let id = self.id.clone().unwrap_or_default();
            entries.retain(|entry| entry.id().eq_ignore_ascii_case(&id));
        }
        mark_latest(&mut entries);
        if let Feed::Search = self.feed {
            let terms = self
                .search_term
                .as_deref()
                .map(crate::search::terms)
                .unwrap_or_default();
            let prerelease = self.prerelease;
            entries.retain(|entry| {
                let latest = match prerelease {
                    true => entry.flag("IsAbsoluteLatestVersion"),
                    false => entry.flag("IsLatestVersion"),
                };
                latest && matches_terms(entry, &terms)
            });
        }
        if let Some(filter) = &self.filter {
            entries.retain(|entry| keep(filter, entry));
        }
        default_order(&mut entries);
        if let Some(orderby) = &self.orderby {
            order(&mut entries, orderby);
        }
        entries
    }
}

fn matches_terms(entry: &Entry, terms: &[crate::search::Term]) -> bool {
    use crate::search::Term;
    let field = |name: &str, needle: &str| {
        entry
            .text(name)
            .is_some_and(|value| value.to_lowercase().contains(needle))
    };
    terms.iter().all(|term| match term {
        Term::Field(name, value) => match name.as_str() {
            "packageid" => entry.id().eq_ignore_ascii_case(value),
            "id" => field("Id", value),
            "version" => version::same(entry.version(), value),
            "tag" | "tags" => field("Tags", value),
            "author" | "authors" => field("Authors", value),
            "title" => field("Title", value),
            "description" => field("Description", value),
            "summary" => field("Summary", value),
            _ => false,
        },
        Term::Any(value) => ["Id", "Title", "Description", "Summary", "Tags", "Authors"]
            .iter()
            .any(|name| field(name, value)),
    })
}

pub struct Page<'a> {
    pub entries: &'a [Entry],
    pub total: usize,
    pub next: Option<String>,
}

pub fn page<'a>(entries: &'a [Entry], request: &Request) -> (&'a [Entry], bool) {
    let start = request.skip.min(entries.len());
    let size = request.top.unwrap_or(PAGE);
    let end = start.saturating_add(size).min(entries.len());
    let more = request.top.is_none() && end < entries.len();
    (&entries[start..end], more)
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn atom_date(edm: Option<&str>) -> String {
    edm.map(|value| match value.ends_with('Z') || value.contains('+') {
        true => value.to_owned(),
        false => format!("{value}Z"),
    })
    .unwrap_or_else(now)
}

fn key(entry: &Entry) -> String {
    format!(
        "Packages(Id='{}',Version='{}')",
        entry.id().replace('\'', "''"),
        entry
            .text("Version")
            .unwrap_or(entry.version())
            .replace('\'', "''")
    )
}

fn entry_xml(out: &mut String, entry: &Entry, urls: &Urls) {
    let root = urls.root();
    let key = escape(&key(entry));
    let version = entry.text("Version").unwrap_or(entry.version()).to_owned();
    out.push_str("<entry>");
    out.push_str(&format!("<id>{}{key}</id>", escape(&root)));
    out.push_str(&format!(
        r#"<category term="{ENTITY}" scheme="{SCHEME}" />"#
    ));
    out.push_str(&format!(
        r#"<link rel="edit" title="V2FeedPackage" href="{key}" /><link rel="self" title="V2FeedPackage" href="{key}" />"#
    ));
    out.push_str(&format!(
        r#"<title type="text">{}</title><summary type="text">{}</summary><updated>{}</updated><author><name>{}</name></author>"#,
        escape(entry.id()),
        escape(entry.text("Summary").unwrap_or_default()),
        escape(&atom_date(entry.text("LastUpdated"))),
        escape(entry.text("Authors").unwrap_or_default()),
    ));
    out.push_str(&format!(
        r#"<content type="application/zip" src="{}" />"#,
        escape(&urls.v2_download(entry.id(), &version))
    ));
    out.push_str("<m:properties>");
    for (name, value) in &entry.props {
        match value {
            Prop::Text(Some(text)) => {
                out.push_str(&format!("<d:{name}>{}</d:{name}>", escape(text)))
            }
            Prop::Text(None) => out.push_str(&format!(r#"<d:{name} m:null="true" />"#)),
            Prop::Bool(flag) => out.push_str(&format!(
                r#"<d:{name} m:type="Edm.Boolean">{flag}</d:{name}>"#
            )),
            Prop::Int32(number) => out.push_str(&format!(
                r#"<d:{name} m:type="Edm.Int32">{number}</d:{name}>"#
            )),
            Prop::Int64(number) => out.push_str(&format!(
                r#"<d:{name} m:type="Edm.Int64">{number}</d:{name}>"#
            )),
            Prop::Date(Some(date)) => out.push_str(&format!(
                r#"<d:{name} m:type="Edm.DateTime">{}</d:{name}>"#,
                escape(date)
            )),
            Prop::Date(None) => out.push_str(&format!(
                r#"<d:{name} m:type="Edm.DateTime" m:null="true" />"#
            )),
        }
    }
    out.push_str("</m:properties></entry>");
}

fn namespaces(urls: &Urls) -> String {
    format!(
        r#"xml:base="{}" xmlns="{ATOM}" xmlns:d="{DATA}" xmlns:m="{META}""#,
        escape(&urls.root())
    )
}

pub fn feed(name: &str, page: &Page<'_>, urls: &Urls) -> String {
    let mut out = String::from(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    out.push('\n');
    out.push_str(&format!("<feed {}>", namespaces(urls)));
    out.push_str(&format!("<m:count>{}</m:count>", page.total));
    out.push_str(&format!(
        r#"<id>{}{name}</id><title type="text">{name}</title><updated>{}</updated><link rel="self" title="{name}" href="{name}" />"#,
        escape(&urls.root()),
        now()
    ));
    for entry in page.entries {
        entry_xml(&mut out, entry, urls);
    }
    if let Some(next) = &page.next {
        out.push_str(&format!(r#"<link rel="next" href="{}" />"#, escape(next)));
    }
    out.push_str("</feed>");
    out
}

pub fn entry(entry: &Entry, urls: &Urls) -> String {
    let mut out = String::from(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    out.push('\n');
    let mut inner = String::new();
    entry_xml(&mut inner, entry, urls);
    out.push_str(&inner.replacen("<entry>", &format!("<entry {}>", namespaces(urls)), 1));
    out
}

pub fn service(urls: &Urls) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<service xml:base="{}" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:app="http://www.w3.org/2007/app" xmlns="http://www.w3.org/2007/app"><workspace><atom:title>Default</atom:title><collection href="Packages"><atom:title>Packages</atom:title></collection></workspace></service>"#,
        escape(&urls.root())
    )
}

pub fn metadata() -> String {
    let properties = [
        ("Id", "Edm.String", false),
        ("Version", "Edm.String", false),
        ("NormalizedVersion", "Edm.String", true),
        ("Authors", "Edm.String", true),
        ("Copyright", "Edm.String", true),
        ("Dependencies", "Edm.String", true),
        ("Description", "Edm.String", true),
        ("IconUrl", "Edm.String", true),
        ("Language", "Edm.String", true),
        ("PackageHash", "Edm.String", true),
        ("PackageHashAlgorithm", "Edm.String", true),
        ("ProjectUrl", "Edm.String", true),
        ("ReleaseNotes", "Edm.String", true),
        ("Summary", "Edm.String", true),
        ("Tags", "Edm.String", true),
        ("Title", "Edm.String", true),
        ("MinClientVersion", "Edm.String", true),
        ("LicenseUrl", "Edm.String", true),
        ("Created", "Edm.DateTime", false),
        ("LastUpdated", "Edm.DateTime", false),
        ("Published", "Edm.DateTime", false),
        ("IsLatestVersion", "Edm.Boolean", false),
        ("IsAbsoluteLatestVersion", "Edm.Boolean", false),
        ("IsPrerelease", "Edm.Boolean", false),
        ("RequireLicenseAcceptance", "Edm.Boolean", false),
        ("Listed", "Edm.Boolean", false),
        ("DownloadCount", "Edm.Int32", false),
        ("VersionDownloadCount", "Edm.Int32", false),
        ("PackageSize", "Edm.Int64", false),
    ];
    let mut out = String::from(
        r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?><edmx:Edmx Version="1.0" xmlns:edmx="http://schemas.microsoft.com/ado/2007/06/edmx"><edmx:DataServices xmlns:m="http://schemas.microsoft.com/ado/2007/08/dataservices/metadata" m:DataServiceVersion="2.0"><Schema Namespace="NuGetGallery.OData" xmlns="http://schemas.microsoft.com/ado/2006/04/edm"><EntityType Name="V2FeedPackage" m:HasStream="true"><Key><PropertyRef Name="Id" /><PropertyRef Name="Version" /></Key>"#,
    );
    for (name, kind, nullable) in properties {
        out.push_str(&format!(
            r#"<Property Name="{name}" Type="{kind}" Nullable="{nullable}" />"#
        ));
    }
    out.push_str(
        r#"</EntityType><EntityContainer Name="FeedContext_x0060_1" m:IsDefaultEntityContainer="true"><EntitySet Name="Packages" EntityType="NuGetGallery.OData.V2FeedPackage" /><FunctionImport Name="Search" ReturnType="Collection(NuGetGallery.OData.V2FeedPackage)" EntitySet="Packages" m:HttpMethod="GET"><Parameter Name="searchTerm" Type="Edm.String" Mode="In" /><Parameter Name="targetFramework" Type="Edm.String" Mode="In" /><Parameter Name="includePrerelease" Type="Edm.Boolean" Mode="In" /></FunctionImport><FunctionImport Name="FindPackagesById" ReturnType="Collection(NuGetGallery.OData.V2FeedPackage)" EntitySet="Packages" m:HttpMethod="GET"><Parameter Name="id" Type="Edm.String" Mode="In" /></FunctionImport></EntityContainer></Schema></edmx:DataServices></edmx:Edmx>"#,
    );
    out
}

fn prop_of(element: &Element) -> Prop {
    let null = element
        .attribute("null")
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let text = (!null).then(|| element.text.clone());
    match element.attribute("type").unwrap_or("Edm.String") {
        "Edm.Boolean" => {
            Prop::Bool(text.is_some_and(|text| text.trim().eq_ignore_ascii_case("true")))
        }
        "Edm.Int32" => Prop::Int32(text.and_then(|text| text.trim().parse().ok()).unwrap_or(0)),
        "Edm.Int64" => Prop::Int64(text.and_then(|text| text.trim().parse().ok()).unwrap_or(0)),
        "Edm.DateTime" | "Edm.DateTimeOffset" => {
            Prop::Date(text.map(|text| text.trim().to_owned()))
        }
        _ => Prop::Text(text),
    }
}

fn parse_entry(element: &Element) -> Option<Entry> {
    let properties = element.child("properties")?;
    let props: Vec<(String, Prop)> = properties
        .children
        .iter()
        .map(|child| (child.name.clone(), prop_of(child)))
        .collect();
    let entry = Entry { props };
    (!entry.id().is_empty()).then_some(entry)
}

#[derive(Debug, Default)]
pub struct Upstream {
    pub entries: Vec<Entry>,
    pub next: Option<String>,
}

pub fn parse_upstream(bytes: &[u8]) -> Option<Upstream> {
    let root = xml::parse(bytes).ok()?;
    match root.name.as_str() {
        "entry" => Some(Upstream {
            entries: parse_entry(&root).into_iter().collect(),
            next: None,
        }),
        "feed" => Some(Upstream {
            entries: root.children("entry").filter_map(parse_entry).collect(),
            next: root
                .children("link")
                .find(|link| link.attribute("rel") == Some("next"))
                .and_then(|link| link.attribute("href"))
                .map(str::to_owned),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::query_pairs;
    use crate::testing::stored;

    fn sample() -> Vec<Entry> {
        from_stored(&[
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>1.0.0</version><description>probe</description><tags>a b</tags></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>1.1.0-beta.1</version><description>probe</description></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>2.0.0</version><description>probe</description></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Astrum.Probe</id><version>3.0.0-rc</version><description>probe</description></metadata></package>",
            ),
            stored(
                "<package><metadata><id>Other</id><version>0.1.0</version><description>else</description></metadata></package>",
            ),
        ])
    }

    fn request(feed: Feed, query: &str) -> Request {
        Request::parse(feed, false, &query_pairs(Some(query)))
    }

    fn versions(entries: &[Entry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| format!("{} {}", entry.id(), entry.version()))
            .collect()
    }

    #[test]
    fn filters_parse_and_evaluate() {
        let entries = sample();
        let probe = &entries[0];
        for (filter, expected) in [
            ("IsPrerelease", false),
            ("not IsPrerelease", true),
            ("tolower(Id) eq 'astrum.probe'", true),
            ("Id eq 'ASTRUM.PROBE' and Version eq '1.0'", true),
            ("(Id eq 'x') or (Id eq 'Astrum.Probe')", true),
            ("substringof('probe', tolower(Id))", true),
            ("startswith(Id,'Astrum')", true),
            ("endswith(Id,'nope')", false),
            ("Version gt '0.9.9'", true),
            ("indexof(Id, 'probe') eq 7", true),
            ("DownloadCount ge 0", true),
            ("Tags eq null", false),
        ] {
            let parsed = parse_expression(filter).unwrap_or_else(|| panic!("{filter}"));
            assert_eq!(keep(&parsed, probe), expected, "{filter}");
        }
        for broken in ["(", "Id eq", "foo(bar", "'open", "a b"] {
            assert!(parse_expression(broken).is_none(), "{broken}");
        }
    }

    #[test]
    fn feeds_select_like_the_gallery() {
        let by_id = request(Feed::FindPackagesById, "id='astrum.probe'");
        assert_eq!(
            versions(&by_id.select(sample())),
            vec![
                "Astrum.Probe 1.0.0",
                "Astrum.Probe 2.0.0",
                "Astrum.Probe 3.0.0-rc"
            ]
        );
        let semver2 = request(
            Feed::FindPackagesById,
            "id='astrum.probe'&semVerLevel=2.0.0",
        );
        assert_eq!(semver2.select(sample()).len(), 4);
        let search = request(Feed::Search, "searchTerm='probe'&includePrerelease=false");
        assert_eq!(
            versions(&search.select(sample())),
            vec!["Astrum.Probe 2.0.0"]
        );
        let search = request(Feed::Search, "searchTerm=''&includePrerelease=true");
        assert_eq!(
            versions(&search.select(sample())),
            vec!["Astrum.Probe 3.0.0-rc", "Other 0.1.0"]
        );
        let latest = request(Feed::Packages, "$filter=IsLatestVersion&$orderby=Id desc");
        assert_eq!(
            versions(&latest.select(sample())),
            vec!["Other 0.1.0", "Astrum.Probe 2.0.0"]
        );
        let ordered = request(Feed::Packages, "$orderby=Version desc,Id");
        assert_eq!(
            versions(&ordered.select(sample()))[0],
            "Astrum.Probe 3.0.0-rc"
        );
        let ignored = request(Feed::Packages, "$filter=foo(bar");
        assert_eq!(ignored.select(sample()).len(), 4);
    }

    #[test]
    fn pages_end_with_a_next_link_only_when_the_server_cut_them() {
        let entries: Vec<Entry> = (0..250)
            .map(|n| {
                from_stored(&[stored(&format!(
                    "<package><metadata><id>P{n}</id><version>1.0.0</version></metadata></package>"
                ))])
                .remove(0)
            })
            .collect();
        let plain = request(Feed::Packages, "");
        let (first, more) = page(&entries, &plain);
        assert_eq!((first.len(), more), (PAGE, true));
        let last = request(Feed::Packages, "$skip=200");
        let (rest, more) = page(&entries, &last);
        assert_eq!((rest.len(), more), (50, false));
        let topped = request(Feed::Packages, "$top=10&$skip=5");
        let (ten, more) = page(&entries, &topped);
        assert_eq!((ten.len(), more, ten[0].id()), (10, false, "P5"));
    }

    #[test]
    fn rendered_feeds_read_back() {
        let urls = Urls::new("http://h/repository/n");
        let entries = request(Feed::Packages, "").select(sample());
        let text = feed(
            "Packages",
            &Page {
                entries: &entries,
                total: entries.len(),
                next: Some("http://h/repository/n/Packages?$skip=100".to_owned()),
            },
            &urls,
        );
        assert!(text.contains("<m:count>4</m:count>"));
        assert!(text.contains(r#"src="http://h/repository/n/package/Astrum.Probe/1.0.0""#));
        assert!(text.contains(r#"<d:Dependencies></d:Dependencies>"#));
        let parsed = parse_upstream(text.as_bytes()).unwrap();
        assert_eq!(parsed.entries.len(), 4);
        assert_eq!(parsed.entries[0], entries[0]);
        assert_eq!(
            parsed.next.as_deref(),
            Some("http://h/repository/n/Packages?$skip=100")
        );
        let single = entry(&entries[1], &urls);
        assert!(single.contains(r#"<entry xml:base="http://h/repository/n/""#));
        let parsed = parse_upstream(single.as_bytes()).unwrap();
        assert_eq!(parsed.entries[0].version(), "2.0.0");
        assert!(parsed.entries[0].flag("IsLatestVersion"));
        assert!(metadata().contains("FindPackagesById"));
        assert!(service(&urls).contains(r#"xml:base="http://h/repository/n/""#));
    }
}
