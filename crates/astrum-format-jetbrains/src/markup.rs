use quick_xml::events::BytesStart;

pub fn attribute(element: &BytesStart<'_>, wanted: &str) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|found| {
            found
                .key
                .local_name()
                .into_inner()
                .eq_ignore_ascii_case(wanted)
        })
        .and_then(|found| {
            found
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.trim().to_owned())
        })
}

pub fn entity(name: &str) -> Option<String> {
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

fn allowed(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r')
        || (c >= ' '
            && c != '\u{fffe}'
            && c != '\u{ffff}'
            && !('\u{fdd0}'..='\u{fdef}').contains(&c))
}

pub fn clean(text: &str) -> String {
    text.chars().filter(|c| allowed(*c)).collect()
}

pub fn escape_text(text: &str) -> String {
    clean(text)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn escape_attribute(text: &str) -> String {
    escape_text(text)
        .replace('"', "&quot;")
        .replace('\n', "&#10;")
        .replace('\r', "&#13;")
        .replace('\t', "&#9;")
}

pub fn cdata(text: &str) -> String {
    format!(
        "<![CDATA[{}]]>",
        clean(text).replace("]]>", "]]]]><![CDATA[>")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_attributes_are_escaped() {
        assert_eq!(escape_text("a<b>&c"), "a&lt;b&gt;&amp;c");
        assert_eq!(escape_attribute("a\"b\n"), "a&quot;b&#10;");
        assert_eq!(escape_text("a\u{0}b\u{8}c"), "abc");
    }

    #[test]
    fn cdata_survives_its_own_terminator() {
        assert_eq!(cdata("<p>x</p>"), "<![CDATA[<p>x</p>]]>");
        assert_eq!(cdata("a]]>b"), "<![CDATA[a]]]]><![CDATA[>b]]>");
    }

    #[test]
    fn entities_resolve() {
        assert_eq!(entity("amp").as_deref(), Some("&"));
        assert_eq!(entity("#65").as_deref(), Some("A"));
        assert_eq!(entity("#x41").as_deref(), Some("A"));
        assert_eq!(entity("nbsp"), None);
        assert_eq!(entity("#0"), None);
    }
}
