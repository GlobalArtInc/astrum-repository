use super::*;

const ROOM: usize = 1000;

#[test]
fn the_origin_is_kept_and_a_missing_origin_stays_empty() {
    let parsed = parse(
        b"P:musl\nV:1.2.4-r2\nA:x86_64\no:musl-src\n\nP:zlib\nV:1.3-r0\n\n",
        ROOM,
    );
    assert_eq!(parsed.malformed, 0);
    assert_eq!(parsed.packages.len(), 2);
    assert_eq!(parsed.packages[0].origin.as_deref(), Some("musl-src"));
    assert_eq!(parsed.packages[0].arch.as_deref(), Some("x86_64"));
    assert_eq!(parsed.packages[1].origin, None);
}

#[test]
fn a_malformed_stanza_is_skipped_and_counted_and_garbage_never_panics() {
    let parsed = parse(b"P:good\nV:1\n\nV:no-name\n\nP:bad name\nV:2\n\n", ROOM);
    assert_eq!(parsed.packages.len(), 1);
    assert_eq!(parsed.malformed, 2);
    for garbage in [&b"\xff\xfe\x00:\n\n:::\n"[..], b"", b"\n\n\n", b"P:", b"P"] {
        let _ = parse(garbage, ROOM);
    }
}

#[test]
fn a_field_longer_than_one_kib_is_cut_and_the_room_limits_the_list() {
    let long = "a".repeat(5000);
    let text = format!("P:{long}\nV:1\n\nP:b\nV:1\n\nP:c\nV:1\n\n");
    let parsed = parse(text.as_bytes(), 2);
    assert_eq!(parsed.packages[0].name.len(), records::FIELD_LIMIT);
    assert_eq!(parsed.packages.len(), 2);
    assert!(parsed.truncated);
}
