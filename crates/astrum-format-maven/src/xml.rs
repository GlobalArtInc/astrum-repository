use quick_xml::Reader;
use quick_xml::events::Event;

const MAX_DEPTH: usize = 64;

#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    #[error("the document is not UTF-8")]
    Encoding,
    #[error("the document is not well-formed: {0}")]
    Syntax(String),
    #[error("the document has no root element")]
    Empty,
    #[error("the document nests too deeply")]
    TooDeep,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub text: String,
    pub children: Vec<Element>,
}

impl Element {
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    pub fn children<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> + 'a {
        self.children.iter().filter(move |child| child.name == name)
    }

    pub fn value(&self) -> Option<String> {
        let trimmed = self.text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    }

    pub fn text_of(&self, name: &str) -> Option<String> {
        self.child(name).and_then(Element::value)
    }
}

fn reference(name: &str) -> Option<String> {
    let resolved = match name {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "apos" => '\'',
        "quot" => '"',
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix('x') {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code).filter(|c| *c != '\0')?
        }
    };
    Some(resolved.to_string())
}

pub fn parse(bytes: &[u8]) -> Result<Element, XmlError> {
    let text = std::str::from_utf8(bytes).map_err(|_| XmlError::Encoding)?;
    let text = text.trim_start_matches('\u{feff}');
    let mut reader = Reader::from_str(text);
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    let syntax = |error: quick_xml::Error| XmlError::Syntax(error.to_string());
    loop {
        match reader.read_event().map_err(syntax)? {
            Event::Start(start) => {
                if root.is_some() {
                    return Err(XmlError::Syntax(
                        "content follows the root element".to_owned(),
                    ));
                }
                if stack.len() >= MAX_DEPTH {
                    return Err(XmlError::TooDeep);
                }
                stack.push(Element {
                    name: start.local_name().into_inner().to_owned(),
                    ..Default::default()
                });
            }
            Event::Empty(empty) => {
                let element = Element {
                    name: empty.local_name().into_inner().to_owned(),
                    ..Default::default()
                };
                match stack.last_mut() {
                    Some(parent) => parent.children.push(element),
                    None if root.is_none() => root = Some(element),
                    None => {
                        return Err(XmlError::Syntax(
                            "content follows the root element".to_owned(),
                        ));
                    }
                }
            }
            Event::End(_) => {
                let element = stack
                    .pop()
                    .ok_or_else(|| XmlError::Syntax("an end tag has no start".to_owned()))?;
                match stack.last_mut() {
                    Some(parent) => parent.children.push(element),
                    None => root = Some(element),
                }
            }
            Event::Text(content) => match stack.last_mut() {
                Some(current) => current.text.push_str(&content.xml10_content()),
                None if content.xml10_content().trim().is_empty() => {}
                None => return Err(XmlError::Syntax("text outside the root element".to_owned())),
            },
            Event::CData(content) => {
                if let Some(current) = stack.last_mut() {
                    current.text.push_str(&content.into_inner());
                }
            }
            Event::GeneralRef(entity) => {
                let name = entity.into_inner();
                let resolved = reference(&name)
                    .ok_or_else(|| XmlError::Syntax(format!("the entity &{name}; is not known")))?;
                if let Some(current) = stack.last_mut() {
                    current.text.push_str(&resolved);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err(XmlError::Syntax("an element is not closed".to_owned()));
    }
    root.ok_or(XmlError::Empty)
}

pub fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            other => escaped.push(other),
        }
    }
    escaped
}

pub fn line(out: &mut String, depth: usize, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        out.push_str(&"  ".repeat(depth));
        out.push_str(&format!("<{name}>{}</{name}>\n", escape(value)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_become_trees() {
        let root = parse(
            b"<?xml version=\"1.0\"?>\n<a xmlns=\"urn:x\"><b>one &amp; two</b><c/><b> &#x41;<![CDATA[<z>]]> </b></a>",
        )
        .unwrap();
        assert_eq!(root.name, "a");
        assert_eq!(root.text_of("b").as_deref(), Some("one & two"));
        assert_eq!(
            root.children("b")
                .nth(1)
                .and_then(Element::value)
                .as_deref(),
            Some("A<z>")
        );
        assert!(root.child("c").is_some());
    }

    #[test]
    fn broken_documents_are_refused() {
        for bad in [
            &b"garbage <not xml"[..],
            b"<a><b></a>",
            b"<a>",
            b"",
            b"<a/><b/>",
            b"<a>&nope;</a>",
            b"\xff\xfe<a/>",
        ] {
            assert!(parse(bad).is_err(), "{}", String::from_utf8_lossy(bad));
        }
    }

    #[test]
    fn text_is_escaped() {
        assert_eq!(escape("a<b>&\"c\""), "a&lt;b&gt;&amp;&quot;c&quot;");
        let mut out = String::new();
        line(&mut out, 2, "x", Some("1&2"));
        line(&mut out, 2, "y", None);
        assert_eq!(out, "    <x>1&amp;2</x>\n");
    }
}
