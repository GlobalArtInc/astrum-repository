use ldap3_proto::proto::{LdapFilter, LdapMatchingRuleAssertion, LdapSubstringFilter};

const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("not a valid LDAP filter: {0}")]
pub struct FilterError(pub String);

pub fn parse(text: &str) -> Result<LdapFilter, FilterError> {
    let trimmed = text.trim();
    let wrapped;
    let source = match trimmed.starts_with('(') {
        true => trimmed,
        false => {
            wrapped = format!("({trimmed})");
            wrapped.as_str()
        }
    };
    let mut parser = Parser {
        bytes: source.as_bytes(),
        at: 0,
    };
    let filter = parser.filter(0)?;
    if parser.at != parser.bytes.len() {
        return Err(parser.fail("text follows the end of the filter"));
    }
    Ok(filter)
}

pub fn equality(attribute: &str, value: &str) -> LdapFilter {
    LdapFilter::Equality(attribute.to_owned(), value.to_owned())
}

pub fn present(attribute: &str) -> LdapFilter {
    LdapFilter::Present(attribute.to_owned())
}

pub fn starts_with(attribute: &str, prefix: &str) -> LdapFilter {
    LdapFilter::Substring(
        attribute.to_owned(),
        LdapSubstringFilter {
            initial: Some(prefix.to_owned()),
            any: Vec::new(),
            final_: None,
        },
    )
}

pub fn contains(attribute: &str, part: &str) -> LdapFilter {
    LdapFilter::Substring(
        attribute.to_owned(),
        LdapSubstringFilter {
            initial: None,
            any: vec![part.to_owned()],
            final_: None,
        },
    )
}

pub fn render(filter: &LdapFilter) -> String {
    match filter {
        LdapFilter::And(parts) => format!("(&{})", parts.iter().map(render).collect::<String>()),
        LdapFilter::Or(parts) => format!("(|{})", parts.iter().map(render).collect::<String>()),
        LdapFilter::Not(inner) => format!("(!{})", render(inner)),
        LdapFilter::Equality(attribute, value) => format!("({attribute}={})", escape(value)),
        LdapFilter::GreaterOrEqual(attribute, value) => {
            format!("({attribute}>={})", escape(value))
        }
        LdapFilter::LessOrEqual(attribute, value) => format!("({attribute}<={})", escape(value)),
        LdapFilter::Approx(attribute, value) => format!("({attribute}~={})", escape(value)),
        LdapFilter::Present(attribute) => format!("({attribute}=*)"),
        LdapFilter::Substring(attribute, parts) => {
            let mut text = format!("({attribute}=");
            if let Some(initial) = &parts.initial {
                text.push_str(&escape(initial));
            }
            text.push('*');
            for any in &parts.any {
                text.push_str(&escape(any));
                text.push('*');
            }
            if let Some(last) = &parts.final_ {
                text.push_str(&escape(last));
            }
            text.push(')');
            text
        }
        LdapFilter::Extensible(rule) => {
            let mut text = String::from("(");
            if let Some(kind) = &rule.type_ {
                text.push_str(kind);
            }
            if rule.dn_attributes {
                text.push_str(":dn");
            }
            if let Some(matching) = &rule.matching_rule {
                text.push(':');
                text.push_str(matching);
            }
            text.push_str(":=");
            text.push_str(&escape(&rule.match_value));
            text.push(')');
            text
        }
    }
}

pub fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '*' | '(' | ')' | '\\' | '\0' => out.push_str(&format!("\\{:02x}", c as u32)),
            _ => out.push(c),
        }
    }
    out
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn fail(&self, message: &str) -> FilterError {
        FilterError(format!("{message} at position {}", self.at + 1))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn expect(&mut self, wanted: u8) -> Result<(), FilterError> {
        match self.peek() {
            Some(byte) if byte == wanted => {
                self.at += 1;
                Ok(())
            }
            _ => Err(self.fail(&format!("expected '{}'", wanted as char))),
        }
    }

    fn filter(&mut self, depth: usize) -> Result<LdapFilter, FilterError> {
        if depth > MAX_DEPTH {
            return Err(self.fail("the filter is nested too deeply"));
        }
        self.expect(b'(')?;
        let filter = match self.peek() {
            Some(b'&') => {
                self.at += 1;
                LdapFilter::And(self.list(depth)?)
            }
            Some(b'|') => {
                self.at += 1;
                LdapFilter::Or(self.list(depth)?)
            }
            Some(b'!') => {
                self.at += 1;
                LdapFilter::Not(Box::new(self.filter(depth + 1)?))
            }
            Some(_) => self.item()?,
            None => return Err(self.fail("the filter ends early")),
        };
        self.expect(b')')?;
        Ok(filter)
    }

    fn list(&mut self, depth: usize) -> Result<Vec<LdapFilter>, FilterError> {
        let mut filters = Vec::new();
        while self.peek() == Some(b'(') {
            filters.push(self.filter(depth + 1)?);
        }
        if filters.is_empty() {
            return Err(self.fail("expected at least one filter"));
        }
        Ok(filters)
    }

    fn item(&mut self) -> Result<LdapFilter, FilterError> {
        let start = self.at;
        while let Some(byte) = self.peek() {
            if matches!(byte, b'=' | b'~' | b'>' | b'<' | b'(' | b')') {
                break;
            }
            self.at += 1;
        }
        let description = std::str::from_utf8(&self.bytes[start..self.at])
            .map_err(|_| self.fail("the attribute is not text"))?
            .to_owned();
        let operator = match (self.peek(), self.bytes.get(self.at + 1).copied()) {
            (Some(b'~'), Some(b'=')) => "~=",
            (Some(b'>'), Some(b'=')) => ">=",
            (Some(b'<'), Some(b'=')) => "<=",
            (Some(b'='), _) => "=",
            _ => return Err(self.fail("expected an operator such as '='")),
        };
        self.at += operator.len();
        if description.contains(':') {
            if operator != "=" || !description.ends_with(':') {
                return Err(self.fail("an extensible match needs ':='"));
            }
            return self.extensible(&description[..description.len() - 1]);
        }
        if !valid_attribute(&description) {
            return Err(self.fail("the attribute name is not valid"));
        }
        let pieces = self.value_pieces()?;
        match operator {
            "~=" => Ok(LdapFilter::Approx(description, single(self, pieces)?)),
            ">=" => Ok(LdapFilter::GreaterOrEqual(
                description,
                single(self, pieces)?,
            )),
            "<=" => Ok(LdapFilter::LessOrEqual(description, single(self, pieces)?)),
            _ => Ok(equality_or_substring(description, pieces)),
        }
    }

    fn extensible(&mut self, left: &str) -> Result<LdapFilter, FilterError> {
        let mut parts = left.split(':');
        let kind = parts.next().unwrap_or_default();
        let mut dn_attributes = false;
        let mut matching_rule = None;
        for part in parts {
            if part.eq_ignore_ascii_case("dn") && !dn_attributes && matching_rule.is_none() {
                dn_attributes = true;
            } else if matching_rule.is_none() && !part.is_empty() {
                matching_rule = Some(part.to_owned());
            } else {
                return Err(self.fail("the extensible match is not valid"));
            }
        }
        if kind.is_empty() && matching_rule.is_none() {
            return Err(self.fail("an extensible match needs an attribute or a matching rule"));
        }
        if !kind.is_empty() && !valid_attribute(kind) {
            return Err(self.fail("the attribute name is not valid"));
        }
        let pieces = self.value_pieces()?;
        let value = single(self, pieces)?;
        Ok(LdapFilter::Extensible(LdapMatchingRuleAssertion {
            matching_rule,
            type_: (!kind.is_empty()).then(|| kind.to_owned()),
            match_value: value,
            dn_attributes,
        }))
    }

    fn value_pieces(&mut self) -> Result<Vec<Option<String>>, FilterError> {
        let mut pieces = Vec::new();
        let mut current: Vec<u8> = Vec::new();
        loop {
            match self.peek() {
                None => return Err(self.fail("the filter ends early")),
                Some(b')') => break,
                Some(b'(') => return Err(self.fail("'(' must be escaped as \\28")),
                Some(b'*') => {
                    pieces.push(Some(self.text(std::mem::take(&mut current))?));
                    pieces.push(None);
                    self.at += 1;
                }
                Some(b'\\') => {
                    let high = self.bytes.get(self.at + 1).copied();
                    let low = self.bytes.get(self.at + 2).copied();
                    let decoded = match (high.and_then(hex_value), low.and_then(hex_value)) {
                        (Some(high), Some(low)) => high * 16 + low,
                        _ => return Err(self.fail("'\\' must be followed by two hex digits")),
                    };
                    current.push(decoded);
                    self.at += 3;
                }
                Some(byte) => {
                    current.push(byte);
                    self.at += 1;
                }
            }
        }
        pieces.push(Some(self.text(current)?));
        Ok(pieces)
    }

    fn text(&self, bytes: Vec<u8>) -> Result<String, FilterError> {
        String::from_utf8(bytes).map_err(|_| self.fail("the value is not UTF-8 text"))
    }
}

fn single(parser: &Parser<'_>, pieces: Vec<Option<String>>) -> Result<String, FilterError> {
    match pieces.as_slice() {
        [Some(value)] => Ok(value.clone()),
        _ => Err(parser.fail("'*' must be escaped as \\2a here")),
    }
}

fn equality_or_substring(attribute: String, pieces: Vec<Option<String>>) -> LdapFilter {
    if pieces.len() == 1 {
        let value = pieces.into_iter().flatten().next().unwrap_or_default();
        return LdapFilter::Equality(attribute, value);
    }
    let texts: Vec<String> = pieces.into_iter().flatten().collect();
    if texts.len() == 2 && texts.iter().all(String::is_empty) {
        return LdapFilter::Present(attribute);
    }
    let last = texts.len() - 1;
    let initial = Some(texts[0].clone()).filter(|text| !text.is_empty());
    let final_ = Some(texts[last].clone()).filter(|text| !text.is_empty());
    let any = texts[1..last]
        .iter()
        .filter(|text| !text.is_empty())
        .cloned()
        .collect();
    LdapFilter::Substring(
        attribute,
        LdapSubstringFilter {
            initial,
            any,
            final_,
        },
    )
}

fn valid_attribute(name: &str) -> bool {
    let (base, _options) = name.split_once(';').unwrap_or((name, ""));
    let descriptor = base
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && base.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let oid = !base.is_empty()
        && base
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    descriptor || oid
}

fn hex_value(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|value| value as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_filters_parse_with_or_without_parentheses() {
        assert_eq!(parse("uid=alice").unwrap(), equality("uid", "alice"));
        assert_eq!(parse(" (uid=alice) ").unwrap(), equality("uid", "alice"));
        assert_eq!(parse("(mail=*)").unwrap(), present("mail"));
        assert_eq!(
            parse("(memberOf=cn=astrum-users,ou=groups,dc=example,dc=com)").unwrap(),
            equality("memberOf", "cn=astrum-users,ou=groups,dc=example,dc=com")
        );
    }

    #[test]
    fn composite_filters_nest() {
        let filter = parse("(&(objectClass=person)(|(uid=a*)(!(cn=x))))").unwrap();
        assert_eq!(
            filter,
            LdapFilter::And(vec![
                equality("objectClass", "person"),
                LdapFilter::Or(vec![
                    starts_with("uid", "a"),
                    LdapFilter::Not(Box::new(equality("cn", "x"))),
                ]),
            ])
        );
    }

    #[test]
    fn escapes_and_substrings_decode() {
        assert_eq!(
            parse("(cn=a\\2ab\\28c\\29)").unwrap(),
            equality("cn", "a*b(c)")
        );
        assert_eq!(
            parse("(cn=*mid*dle*end)").unwrap(),
            LdapFilter::Substring(
                "cn".to_owned(),
                LdapSubstringFilter {
                    initial: None,
                    any: vec!["mid".to_owned(), "dle".to_owned()],
                    final_: Some("end".to_owned()),
                }
            )
        );
        assert_eq!(
            parse("(uidNumber>=1000)").unwrap(),
            LdapFilter::GreaterOrEqual("uidNumber".to_owned(), "1000".to_owned())
        );
    }

    #[test]
    fn extensible_matches_parse() {
        assert_eq!(
            parse("(userAccountControl:1.2.840.113556.1.4.803:=2)").unwrap(),
            LdapFilter::Extensible(LdapMatchingRuleAssertion {
                matching_rule: Some("1.2.840.113556.1.4.803".to_owned()),
                type_: Some("userAccountControl".to_owned()),
                match_value: "2".to_owned(),
                dn_attributes: false,
            })
        );
        assert!(parse("(cn:dn:=x)").is_ok());
    }

    #[test]
    fn broken_filters_are_refused() {
        for broken in [
            "",
            "(",
            "(uid=alice",
            "(uid=alice))",
            "(&)",
            "(uid=a(b)",
            "(=x)",
            "(1uid=x)",
            "(uid>=a*)",
            "(cn=\\2)",
            "uid",
        ] {
            assert!(parse(broken).is_err(), "{broken}");
        }
    }

    #[test]
    fn rendering_escapes_what_needs_it() {
        let filter = LdapFilter::And(vec![
            equality("uid", "a*b(c)\\"),
            starts_with("cn", "jo"),
            present("mail"),
        ]);
        let text = render(&filter);
        assert_eq!(text, "(&(uid=a\\2ab\\28c\\29\\5c)(cn=jo*)(mail=*))");
        assert_eq!(parse(&text).unwrap(), filter);
        assert_eq!(escape("plain é"), "plain é");
    }
}
