use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::layers::facts::{LayerFacts, extract};
use crate::layers::fixtures::*;
use crate::layers::limits::Limits;
use crate::layers::stream::Guard;

const OS: &[u8] = ALPINE_OS.as_bytes();
const DPKG: &[u8] = DPKG_DB.as_bytes();

fn facts(parts: &[Vec<u8>]) -> LayerFacts {
    let mut bytes: Vec<u8> = parts.concat();
    bytes.extend_from_slice(&END);
    let guard = Guard::new(
        Arc::new(AtomicBool::new(false)),
        Instant::now() + Duration::from_secs(60),
    );
    extract(Cursor::new(bytes), &Limits::DEFAULT, guard).unwrap()
}

fn os_path(facts: &LayerFacts) -> Option<&str> {
    facts.os_release.as_ref().map(|os| os.path.as_str())
}

#[test]
fn the_ustar_magic_with_any_version_applies_the_prefix_and_other_magics_do_not() {
    let weird = Raw::new("status")
        .prefix(b"var/lib/dpkg")
        .magic(b"ustar\0", b"xx");
    assert!(facts(&[weird.with(DPKG)]).dpkg.is_some());
    let gnu = Raw::new("status")
        .prefix(b"var/lib/dpkg")
        .magic(b"ustar ", b" \0");
    assert!(facts(&[gnu.with(DPKG)]).dpkg.is_none());
    let v7 = Raw::new("status").prefix(b"var/lib/dpkg").magic(b"", b"");
    assert!(facts(&[v7.with(DPKG)]).dpkg.is_none());
}

#[test]
fn long_name_and_pax_headers_are_honoured_by_typeflag_without_a_magic() {
    let long = Raw::new("L")
        .magic(b"", b"")
        .flag(b'L')
        .with(b"var/lib/dpkg/status\0");
    let named = Raw::new("short").magic(b"", b"").with(DPKG);
    assert!(facts(&[long, named]).dpkg.is_some());
    let records = pax_record("path", "etc/os-release");
    let extended = Raw::new("pax").magic(b"", b"").flag(b'x').with(&records);
    let named = Raw::new("short").magic(b"", b"").with(OS);
    assert_eq!(os_path(&facts(&[extended, named])), Some("etc/os-release"));
}

#[test]
fn an_empty_pax_path_keeps_the_header_name_and_a_pax_size_is_the_data_length() {
    let empty = pax(&[("path", "")]);
    let named = Raw::new("etc/os-release").with(OS);
    assert_eq!(os_path(&facts(&[empty, named])), Some("etc/os-release"));
    let size = pax(&[("size", &OS.len().to_string())]);
    let zero = Raw::new("etc/os-release");
    let mut sized = zero.build();
    sized.extend_from_slice(OS);
    sized.resize(sized.len() + 512 - OS.len() % 512, 0);
    let after = Raw::new("var/lib/dpkg/status").with(DPKG);
    let found = facts(&[size, sized, after]);
    assert_eq!(os_path(&found), Some("etc/os-release"));
    assert!(found.dpkg.is_some());
}

#[test]
fn a_long_name_is_cut_at_its_first_nul_and_an_empty_one_keeps_the_header_name() {
    let cut = Raw::new("L").flag(b'L').with(b"etc/os-release\0junk");
    let ignored = Raw::new("ignored").with(OS);
    assert_eq!(os_path(&facts(&[cut, ignored])), Some("etc/os-release"));
    let empty = Raw::new("L").flag(b'L').with(b"\0etc/os-release");
    let named = Raw::new("usr/lib/os-release").with(OS);
    assert_eq!(os_path(&facts(&[empty, named])), Some("usr/lib/os-release"));
}

#[test]
fn paths_are_cleaned_like_a_path_and_what_escapes_the_root_is_kept_inside_it() {
    let found = facts(&[
        Raw::new("a/../var/lib/dpkg/status").with(DPKG),
        Raw::new("../../etc/os-release").with(OS),
    ]);
    assert!(found.dpkg.is_some());
    assert_eq!(os_path(&found), Some("etc/os-release"));
    let dotted = facts(&[Raw::new("./usr/./lib//os-release").with(OS)]);
    assert_eq!(os_path(&dotted), Some("usr/lib/os-release"));
}

#[test]
fn a_pax_sparse_entry_is_judged_by_its_real_name_and_a_wanted_one_is_incomplete() {
    let records = pax(&[
        ("GNU.sparse.name", "var/lib/dpkg/status"),
        ("GNU.sparse.major", "1"),
        ("GNU.sparse.minor", "0"),
    ]);
    let renamed = Raw::new("GNUSparseFile.0/status").with(b"map");
    let found = facts(&[records, renamed]);
    assert!(found.incomplete);
    assert_eq!(found.unreadable, ["var/lib/dpkg/status"]);
    assert!(found.dpkg.is_none());
    let records = pax(&[("GNU.sparse.major", "1")]);
    let plain = Raw::new("var/lib/dpkg/status").with(b"map");
    assert!(facts(&[records, plain]).incomplete);
}

#[test]
fn a_global_pax_header_drops_what_is_pending_and_a_second_pax_header_replaces_the_first() {
    let named = pax(&[("path", "etc/os-release")]);
    let global = Raw::new("global")
        .flag(b'g')
        .with(&pax_record("comment", "x"));
    let entry = Raw::new("usr/lib/os-release").with(OS);
    assert_eq!(
        os_path(&facts(&[named.clone(), global, entry.clone()])),
        Some("usr/lib/os-release")
    );
    let other = pax(&[("comment", "y")]);
    assert_eq!(
        os_path(&facts(&[named, other, entry])),
        Some("usr/lib/os-release")
    );
}

#[test]
fn an_extended_flag_above_zero_is_followed_for_every_old_gnu_sparse_extension() {
    let sparse = Raw::new("usr/share/sparse")
        .flag(b'S')
        .magic(b"ustar ", b" \0");
    let mut first = sparse.extended(2).build();
    let mut one = vec![0u8; 512];
    one[504] = 2;
    first.extend_from_slice(&one);
    first.extend_from_slice(&[0u8; 512]);
    let after = Raw::new("etc/os-release").with(OS);
    let found = facts(&[first, after]);
    assert_eq!(os_path(&found), Some("etc/os-release"));
    assert!(!found.incomplete, "{:?}", found.reasons);
}

#[test]
fn the_signed_header_checksum_is_accepted_like_the_unsigned_one() {
    let name = "usr/share/caf\u{e9}".as_bytes();
    let odd = Raw::new("x").bytes(name).signed().with(b"data");
    let after = Raw::new("etc/os-release").with(OS);
    let found = facts(&[odd, after]);
    assert_eq!(os_path(&found), Some("etc/os-release"));
    assert!(!found.incomplete, "{:?}", found.reasons);
}

#[test]
fn whiteouts_are_read_by_name_for_any_entry_type_and_a_clash_in_one_layer_is_incomplete() {
    let found = facts(&[
        Raw::new("var/lib/dpkg/.wh.status")
            .flag(b'2')
            .link("x")
            .build(),
        Raw::new("lib/apk/db/.wh..wh..opq").flag(b'5').build(),
    ]);
    assert_eq!(found.removed, ["var/lib/dpkg/status"]);
    assert_eq!(found.opaque, ["lib/apk/db"]);
    assert!(!found.incomplete);
    let file = Raw::new("var/lib/dpkg/status").with(DPKG);
    let whiteout = Raw::new("var/lib/dpkg/.wh.status").with(b"");
    for parts in [[file.clone(), whiteout.clone()], [whiteout, file]] {
        let clash = facts(&parts);
        assert!(clash.incomplete);
        assert_eq!(clash.unreadable, ["var/lib/dpkg/status"]);
    }
}

#[test]
fn a_non_directory_at_an_ancestor_of_a_database_makes_the_layer_incomplete_but_lib_does_not() {
    let broken = [
        Raw::new("var/lib/dpkg")
            .flag(b'2')
            .link("/usr/lib/dpkg")
            .build(),
        Raw::new("var").with(b"file"),
        Raw::new("lib/apk/db").flag(b'2').link("x").build(),
        Raw::new("var/lib/dpkg/status.d")
            .flag(b'1')
            .link("x")
            .build(),
    ];
    for entry in broken {
        let found = facts(&[entry]);
        assert!(found.incomplete);
        assert_eq!(found.removed.len(), 1);
    }
    let fine = facts(&[
        Raw::new("lib").flag(b'2').link("usr/lib").build(),
        Raw::new("usr/lib").flag(b'2').link("x").build(),
        Raw::new("var/lib").flag(b'5').build(),
        Raw::new("var/lib/dpkg/").flag(b'5').build(),
    ]);
    assert!(!fine.incomplete, "{:?}", fine.reasons);
    assert!(fine.removed.is_empty());
}
