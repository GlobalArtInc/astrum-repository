use super::*;

fn parsed(bytes: &[u8]) -> Parsed<ApkPackage> {
    let limits = Limits::DEFAULT;
    parse(bytes, &limits, &mut Budget::new(limits.facts_budget))
}

#[test]
fn the_origin_is_kept_and_a_missing_origin_stays_empty() {
    let parsed = parsed(b"P:musl\nV:1.2.4-r2\nA:x86_64\no:musl-src\n\nP:zlib\nV:1.3-r0\n\n");
    assert_eq!(parsed.malformed, 0);
    assert_eq!(parsed.packages.len(), 2);
    assert_eq!(parsed.packages[0].origin.as_deref(), Some("musl-src"));
    assert_eq!(parsed.packages[0].arch.as_deref(), Some("x86_64"));
    assert_eq!(parsed.packages[1].origin, None);
}

#[test]
fn a_malformed_stanza_is_skipped_and_counted_and_garbage_never_panics() {
    let parsed = parsed(b"P:good\nV:1\n\nV:no-name\n\nP:bad name\nV:2\n\n");
    assert_eq!(parsed.packages.len(), 1);
    assert_eq!(parsed.malformed, 2);
    for garbage in [&b"\xff\xfe\x00:\n\n:::\n"[..], b"", b"\n\n\n", b"P:", b"P"] {
        let _ = self::parsed(garbage);
    }
}

#[test]
fn a_field_over_one_kib_makes_the_stanza_malformed_and_is_never_cut() {
    let long = "a".repeat(records::FIELD_LIMIT + 1);
    let exact = "b".repeat(records::FIELD_LIMIT);
    let parsed = parsed(format!("P:{long}\nV:1\n\nP:{exact}\nV:1\n\nP:c\nV:1\n").as_bytes());
    assert_eq!(parsed.malformed, 1);
    let names: Vec<usize> = parsed.packages.iter().map(|p| p.name.len()).collect();
    assert_eq!(names, [records::FIELD_LIMIT, 1]);
}

#[test]
fn a_database_over_the_package_count_keeps_the_limit_and_says_so() {
    let mut text = String::new();
    for index in 0..21_000 {
        text.push_str(&format!("P:p{index}\nV:1\n\n"));
    }
    let parsed = parsed(text.as_bytes());
    assert_eq!(parsed.packages.len(), Limits::DEFAULT.max_db_packages);
    assert_eq!(parsed.truncated, Some(Truncation::Count));
}

#[test]
fn the_byte_budget_stops_retaining_packages() {
    let text = "P:aaaa\nV:1\n\nP:bbbb\nV:1\n\nP:cccc\nV:1\n\n";
    let mut budget = Budget::new(12);
    let parsed = parse(text.as_bytes(), &Limits::DEFAULT, &mut budget);
    assert_eq!(parsed.packages.len(), 2);
    assert_eq!(parsed.truncated, Some(Truncation::Budget));
    let kept: usize = parsed.packages.iter().map(ApkPackage::cost).sum();
    assert!(kept <= 12);
}
