pub const COMMON_NS: &str = "http://linux.duke.edu/metadata/common";
pub const RPM_NS: &str = "http://linux.duke.edu/metadata/rpm";
pub const FILELISTS_NS: &str = "http://linux.duke.edu/metadata/filelists";
pub const OTHER_NS: &str = "http://linux.duke.edu/metadata/other";
pub const REPO_NS: &str = "http://linux.duke.edu/metadata/repo";
pub const DECLARATION: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";

fn valid(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r')
        || ('\u{20}'..='\u{D7FF}').contains(&c)
        || ('\u{E000}'..='\u{FFFD}').contains(&c)
        || c >= '\u{10000}'
}

pub fn text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars().filter(|c| valid(*c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

pub fn attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars().filter(|c| valid(*c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' => out.push_str("&#9;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_are_escaped_and_cleaned() {
        assert_eq!(text("a < b & c\u{1}"), "a &lt; b &amp; c");
        assert_eq!(attribute("say \"hi\"\n"), "say &quot;hi&quot;&#10;");
        assert_eq!(text("ünïcode 😀"), "ünïcode 😀");
    }
}
