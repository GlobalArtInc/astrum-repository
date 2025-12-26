#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("not a valid distinguished name: {0}")]
pub struct DnError(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

pub type Rdn = Vec<Attribute>;

pub fn parse(text: &str) -> Result<Vec<Rdn>, DnError> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    let mut rdns = Vec::new();
    loop {
        let mut rdn = Vec::new();
        loop {
            rdn.push(attribute(&chars, &mut at)?);
            skip_spaces(&chars, &mut at);
            match chars.get(at) {
                Some('+') => {
                    at += 1;
                }
                _ => break,
            }
        }
        rdns.push(rdn);
        match chars.get(at) {
            None => return Ok(rdns),
            Some(',' | ';') => at += 1,
            Some(other) => {
                return Err(DnError(format!(
                    "unexpected '{other}' at position {}",
                    at + 1
                )));
            }
        }
    }
}

pub fn is_valid(text: &str) -> bool {
    parse(text).is_ok_and(|rdns| !rdns.is_empty())
}

pub fn first_value(text: &str) -> Option<String> {
    parse(text)
        .ok()?
        .into_iter()
        .next()?
        .into_iter()
        .next()
        .map(|attribute| attribute.value)
}

pub fn escape_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let last = value.chars().count().saturating_sub(1);
    for (index, c) in value.chars().enumerate() {
        let edge_space = c == ' ' && (index == 0 || index == last);
        let leading_hash = c == '#' && index == 0;
        if matches!(c, ',' | '+' | '"' | '\\' | '<' | '>' | ';' | '=') || edge_space || leading_hash
        {
            out.push('\\');
            out.push(c);
        } else if c == '\0' {
            out.push_str("\\00");
        } else {
            out.push(c);
        }
    }
    out
}

fn skip_spaces(chars: &[char], at: &mut usize) {
    while chars.get(*at) == Some(&' ') {
        *at += 1;
    }
}

fn attribute(chars: &[char], at: &mut usize) -> Result<Attribute, DnError> {
    skip_spaces(chars, at);
    let start = *at;
    while let Some(c) = chars.get(*at) {
        if *c == '=' || *c == ' ' {
            break;
        }
        *at += 1;
    }
    let name: String = chars[start..*at].iter().collect();
    if !valid_type(&name) {
        return Err(DnError(format!(
            "'{name}' at position {} is not an attribute type",
            start + 1
        )));
    }
    skip_spaces(chars, at);
    if chars.get(*at) != Some(&'=') {
        return Err(DnError(format!("expected '=' at position {}", *at + 1)));
    }
    *at += 1;
    skip_spaces(chars, at);
    let value = match chars.get(*at) {
        Some('#') => hex_value(chars, at)?,
        Some('"') => quoted_value(chars, at)?,
        _ => plain_value(chars, at)?,
    };
    Ok(Attribute { name, value })
}

fn valid_type(name: &str) -> bool {
    let descriptor = name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let oid = name
        .strip_prefix("oid.")
        .or_else(|| name.strip_prefix("OID."))
        .unwrap_or(name);
    let numeric = !oid.is_empty()
        && oid
            .split('.')
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    descriptor || numeric
}

fn hex_value(chars: &[char], at: &mut usize) -> Result<String, DnError> {
    *at += 1;
    let start = *at;
    while chars.get(*at).is_some_and(char::is_ascii_hexdigit) {
        *at += 1;
    }
    let digits: String = chars[start..*at].iter().collect();
    if digits.is_empty() || !digits.len().is_multiple_of(2) {
        return Err(DnError(format!(
            "the hex value at position {} is not complete",
            start
        )));
    }
    skip_spaces(chars, at);
    Ok(format!("#{digits}"))
}

fn quoted_value(chars: &[char], at: &mut usize) -> Result<String, DnError> {
    let start = *at;
    *at += 1;
    let mut value = String::new();
    loop {
        match chars.get(*at) {
            None => {
                return Err(DnError(format!(
                    "the quote at position {} is not closed",
                    start + 1
                )));
            }
            Some('"') => {
                *at += 1;
                break;
            }
            Some('\\') => {
                let next = chars
                    .get(*at + 1)
                    .ok_or_else(|| DnError("the value ends with '\\'".to_owned()))?;
                value.push(*next);
                *at += 2;
            }
            Some(c) => {
                value.push(*c);
                *at += 1;
            }
        }
    }
    skip_spaces(chars, at);
    Ok(value)
}

fn plain_value(chars: &[char], at: &mut usize) -> Result<String, DnError> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut trailing_spaces = 0usize;
    loop {
        match chars.get(*at) {
            None | Some(',' | ';' | '+') => break,
            Some('"' | '<' | '>' | '=') => {
                return Err(DnError(format!(
                    "'{}' at position {} must be escaped",
                    chars[*at],
                    *at + 1
                )));
            }
            Some('\\') => {
                let next = chars.get(*at + 1).copied();
                let after = chars.get(*at + 2).copied();
                match (next, after) {
                    (Some(high), Some(low))
                        if high.is_ascii_hexdigit() && low.is_ascii_hexdigit() =>
                    {
                        let byte = u8::from_str_radix(&format!("{high}{low}"), 16)
                            .map_err(|_| DnError("a hex escape is not valid".to_owned()))?;
                        bytes.push(byte);
                        *at += 3;
                    }
                    (
                        Some(
                            special @ (' ' | '"' | '#' | '+' | ',' | ';' | '<' | '=' | '>' | '\\'),
                        ),
                        _,
                    ) => {
                        let mut buffer = [0u8; 4];
                        bytes.extend_from_slice(special.encode_utf8(&mut buffer).as_bytes());
                        *at += 2;
                    }
                    _ => {
                        return Err(DnError(format!(
                            "the escape at position {} is not valid",
                            *at + 1
                        )));
                    }
                }
                trailing_spaces = 0;
            }
            Some(c) => {
                trailing_spaces = match c {
                    ' ' => trailing_spaces + 1,
                    _ => 0,
                };
                let mut buffer = [0u8; 4];
                bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                *at += 1;
            }
        }
    }
    bytes.truncate(bytes.len() - trailing_spaces);
    String::from_utf8(bytes).map_err(|_| DnError("a value is not UTF-8 text".to_owned()))
}

pub fn same(left: &str, right: &str) -> bool {
    match (parse(left), parse(right)) {
        (Ok(left), Ok(right)) => {
            left.len() == right.len()
                && left.iter().zip(&right).all(|(a, b)| {
                    a.len() == b.len()
                        && a.iter().zip(b).all(|(a, b)| {
                            a.name.eq_ignore_ascii_case(&b.name)
                                && a.value.to_lowercase() == b.value.to_lowercase()
                        })
                })
        }
        _ => left.eq_ignore_ascii_case(right),
    }
}

pub fn is_under(entry: &str, base: &str) -> bool {
    let (Ok(entry), Ok(base)) = (parse(entry), parse(base)) else {
        return false;
    };
    entry.len() >= base.len()
        && entry[entry.len() - base.len()..]
            .iter()
            .zip(&base)
            .all(|(a, b)| {
                a.len() == b.len()
                    && a.iter().zip(b).all(|(a, b)| {
                        a.name.eq_ignore_ascii_case(&b.name)
                            && a.value.to_lowercase() == b.value.to_lowercase()
                    })
            })
}

pub fn depth(text: &str) -> usize {
    parse(text).map(|rdns| rdns.len()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_names_parse() {
        let rdns = parse("cn=developers,ou=groups,dc=example,dc=com").unwrap();
        assert_eq!(rdns.len(), 4);
        assert_eq!(rdns[0][0].name, "cn");
        assert_eq!(rdns[0][0].value, "developers");
        assert!(is_valid("dc=example, dc=com"));
        assert!(is_valid("uid=a+cn=b,dc=x"));
        assert!(is_valid("2.5.4.3=x,dc=y"));
        assert_eq!(
            first_value("CN=Smith\\, John,OU=People"),
            Some("Smith, John".to_owned())
        );
        assert_eq!(first_value("cn=caf\\c3\\a9,dc=x"), Some("café".to_owned()));
        assert_eq!(first_value("cn=\"a,b\",dc=x"), Some("a,b".to_owned()));
        assert_eq!(
            first_value("cn=trailing   ,dc=x"),
            Some("trailing".to_owned())
        );
    }

    #[test]
    fn broken_names_are_refused() {
        for broken in [
            "",
            "dc",
            "=x",
            "dc=x,",
            "dc=x,,dc=y",
            "dc=a\\",
            "dc=a=b",
            "1dc=x",
            "cn=\"open",
            "cn=#abc",
        ] {
            assert!(!is_valid(broken), "{broken}");
        }
        assert_eq!(first_value("plain text"), None);
    }

    #[test]
    fn names_compare_ignoring_case_and_spacing() {
        assert!(same(
            "CN=Admins, OU=Groups,DC=example",
            "cn=admins,ou=groups,dc=Example"
        ));
        assert!(!same("cn=a,dc=x", "cn=b,dc=x"));
        assert!(is_under(
            "uid=a,ou=people,dc=example,dc=com",
            "DC=example,dc=com"
        ));
        assert!(is_under("dc=example,dc=com", "dc=example,dc=com"));
        assert!(!is_under("dc=com", "dc=example,dc=com"));
        assert_eq!(depth("uid=a,ou=people,dc=x"), 3);
    }

    #[test]
    fn values_escape_round_trip() {
        for value in ["Smith, John", " lead", "#hash", "a+b=c", "plain", "trail "] {
            let dn = format!("cn={},dc=x", escape_value(value));
            assert_eq!(first_value(&dn).as_deref(), Some(value), "{dn}");
        }
    }
}
