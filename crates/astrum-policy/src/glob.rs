pub fn matches(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !text.starts_with(first) || text.len() < first.len() + last.len() || !text.ends_with(last) {
        return false;
    }
    let mut rest = &text[first.len()..text.len() - last.len()];
    for middle in &parts[1..parts.len() - 1] {
        match rest.find(middle) {
            Some(at) => rest = &rest[at + middle.len()..],
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn stars_match_any_run_and_nothing_else_is_special() {
        let patterns = ["*.example.com", "localhost", "10.*", "@globalart/*"];
        assert!(matches(patterns[0], "repo.example.com"));
        assert!(!matches(patterns[0], "example.com"));
        assert!(!matches(patterns[0], "repo.example.org"));
        assert!(matches(patterns[1], "localhost"));
        assert!(!matches(patterns[1], "LOCALHOST"));
        assert!(matches(patterns[2], "10.1.2.3"));
        assert!(matches(patterns[3], "@globalart/ui"));
        assert!(!matches(patterns[3], "@other/ui"));
        assert!(matches("*", "anything"));
        assert!(matches("a*b*c", "axxbyyc"));
        assert!(!matches("a*b*c", "acb"));
        assert!(matches("", ""));
        assert!(!matches("", "x"));
        assert!(!matches("a?c", "abc"));
    }
}
