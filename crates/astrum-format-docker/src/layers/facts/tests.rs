use std::io::{Cursor, Read};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use tar::EntryType;

use super::*;
use crate::layers::fixtures::*;
use crate::layers::limits::Limits;

fn guard_for(deadline: Instant) -> Guard {
    Guard::new(Arc::new(AtomicBool::new(false)), deadline)
}

fn run(bytes: &[u8], limits: &Limits) -> Result<LayerFacts, Failure> {
    extract(
        bytes,
        limits,
        guard_for(Instant::now() + Duration::from_secs(3600)),
    )
}

fn facts(entries: &[Entry<'_>]) -> LayerFacts {
    run(&tar(entries), &Limits::DEFAULT).unwrap()
}

fn reasons(facts: &LayerFacts) -> String {
    facts.reasons.join("; ")
}

fn raw(parts: &[Vec<u8>]) -> Vec<u8> {
    let mut out: Vec<u8> = parts.concat();
    out.extend_from_slice(&END);
    out
}

#[test]
fn the_wanted_files_are_read_from_plain_gzip_and_zstd_layers_by_their_magic_bytes() {
    let plain = alpine_layer();
    for layer in [plain.clone(), gzip(&plain), zstd(&plain)] {
        let facts = run(&layer, &Limits::DEFAULT).unwrap();
        assert!(!facts.incomplete, "{}", reasons(&facts));
        assert_eq!(
            facts.os_release.as_ref().map(|os| os.id.as_str()),
            Some("alpine")
        );
        assert_eq!(facts.apk.as_ref().map(Vec::len), Some(2));
    }
}

#[test]
fn whiteouts_and_opaque_markers_of_the_wanted_locations_are_recorded_and_the_rest_ignored() {
    let facts = facts(&[
        Entry::File("var/lib/dpkg/.wh.status", b""),
        Entry::File("var/lib/dpkg/status.d/.wh.libc6", b""),
        Entry::File("./.wh.lib", b""),
        Entry::File("usr/share/.wh.docs", b""),
        Entry::File("var/.wh.cache", b""),
        Entry::File("lib/apk/db/.wh..wh..opq", b""),
        Entry::File("tmp/.wh..wh..opq", b""),
        Entry::File("var/.wh..wh.plnk", b""),
    ]);
    assert_eq!(
        facts.removed,
        ["var/lib/dpkg/status", "var/lib/dpkg/status.d/libc6", "lib"]
    );
    assert_eq!(facts.opaque, ["lib/apk/db"]);
    assert!(!facts.incomplete);
}

#[test]
fn dotdot_names_are_dropped_and_absolute_names_are_made_relative() {
    let facts = facts(&[
        Entry::File("../var/lib/dpkg/status", DPKG_DB.as_bytes()),
        Entry::File("a/../var/lib/dpkg/status", DPKG_DB.as_bytes()),
        Entry::File("/usr/lib/os-release", DEBIAN_OS.as_bytes()),
        Entry::File("var/lib/dpkg/status.d/sub/deep", DPKG_DB.as_bytes()),
        Entry::File("var/lib/dpkg/status.d/pkg.md5sums", b"abc  usr/x\n"),
    ]);
    assert_eq!(
        facts.os_release.as_ref().map(|os| os.path.as_str()),
        Some("usr/lib/os-release")
    );
    assert!(facts.apk.is_none() && facts.dpkg.is_none());
    assert!(facts.dpkg_status_d.is_empty());
    assert!(!facts.incomplete);
}

#[test]
fn a_link_at_a_database_path_makes_the_layer_incomplete_and_other_links_do_not() {
    for entries in [
        vec![Entry::Symlink("var/lib/dpkg/status", "elsewhere")],
        vec![Entry::Hardlink("lib/apk/db/installed", "elsewhere")],
        vec![Entry::Symlink("var/lib/dpkg/status.d/base", "elsewhere")],
    ] {
        let facts = facts(&entries);
        assert!(facts.incomplete);
        assert_eq!(facts.unreadable.len(), 1);
    }
    let facts = facts(&[
        Entry::Symlink("lib", "usr/lib"),
        Entry::Symlink("etc/os-release", "../usr/lib/os-release"),
        Entry::Symlink("var/lib/dpkg/status.d/pkg.md5sums", "x"),
    ]);
    assert!(!facts.incomplete, "{}", reasons(&facts));
    assert!(facts.unreadable.is_empty());
}

#[test]
fn a_huge_claimed_size_never_makes_the_reader_pass_what_the_body_holds_or_the_cap() {
    let five_gib = 5 * 1024 * 1024 * 1024;
    for name in ["var/lib/dpkg/status", "usr/share/blob"] {
        let mut layer = claim(name, five_gib, &[b'a'; 100]);
        layer.resize(layer.len() + 1024, 0);
        let length = layer.len() as u64;
        let (source, read) = Counting::new(Cursor::new(layer));
        let facts = extract(
            source,
            &Limits::DEFAULT,
            guard_for(Instant::now() + Duration::from_secs(60)),
        )
        .unwrap();
        assert!(facts.incomplete, "{name}");
        assert!(facts.dpkg.is_none());
        assert!(read.get() <= length, "{name}");
    }
    let capped = Limits {
        max_output: 8 * 1024 * 1024,
        ..Limits::DEFAULT
    };
    let header = claim("usr/share/blob", five_gib, &[]);
    let endless = Cursor::new(header[..512].to_vec()).chain(Repeat::new(&[0]));
    let (source, read) = Counting::new(endless);
    let facts = extract(
        source,
        &capped,
        guard_for(Instant::now() + Duration::from_secs(60)),
    )
    .unwrap();
    assert!(facts.incomplete);
    assert!(
        reasons(&facts).contains("larger than"),
        "{}",
        reasons(&facts)
    );
    assert!(
        read.get() <= capped.max_output + 128 * 1024,
        "{}",
        read.get()
    );
}

#[test]
fn a_gzip_bomb_is_cut_at_the_output_cap() {
    let zeros = vec![0u8; 8 * 1024 * 1024];
    let bomb = gzip(&tar(&[Entry::File("usr/share/zeros", &zeros)]));
    assert!(bomb.len() < 64 * 1024);
    let capped = Limits {
        max_output: 1024 * 1024,
        ..Limits::DEFAULT
    };
    let facts = run(&bomb, &capped).unwrap();
    assert!(facts.incomplete);
    assert!(
        reasons(&facts).contains("larger than"),
        "{}",
        reasons(&facts)
    );
}

fn stalled(source: impl Read, consumed: &std::rc::Rc<std::cell::Cell<u64>>) -> LayerFacts {
    let limits = Limits {
        stall_input: 64 * 1024,
        stall_output: 1024,
        ..Limits::DEFAULT
    };
    let facts = extract(
        source,
        &limits,
        guard_for(Instant::now() + Duration::from_secs(60)),
    )
    .unwrap();
    assert!(
        consumed.get() <= limits.stall_input + 256 * 1024,
        "{}",
        consumed.get()
    );
    facts
}

#[test]
fn members_and_frames_that_eat_input_without_output_are_cut_early() {
    let gzip_header = [0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 0xff];
    let members = Cursor::new(gzip_header.to_vec()).chain(Repeat::new(&[0, 0, 0, 0xff, 0xff]));
    let (source, read) = Counting::new(members);
    let facts = stalled(source, &read);
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("of input"), "{}", reasons(&facts));

    let frame = [0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x00];
    let blocks = Cursor::new(frame.to_vec()).chain(Repeat::new(&[0, 0, 0]));
    let (source, read) = Counting::new(blocks);
    let facts = stalled(source, &read);
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("of input"), "{}", reasons(&facts));

    let skippable = [0x50, 0x2a, 0x4d, 0x18, 0xff, 0xff, 0xff, 0xff];
    let hole = Cursor::new(skippable.to_vec()).chain(Repeat::new(&[0]));
    let (source, read) = Counting::new(hole);
    let facts = stalled(source, &read);
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("of input"), "{}", reasons(&facts));
}

#[test]
fn a_passed_deadline_cuts_the_work_at_once() {
    let failure = extract(
        Cursor::new(gzip(&alpine_layer())),
        &Limits::DEFAULT,
        guard_for(Instant::now() - Duration::from_secs(1)),
    );
    assert!(matches!(failure, Err(Failure::Deadline)), "{failure:?}");
    let cancelled = Guard::new(
        Arc::new(AtomicBool::new(true)),
        Instant::now() + Duration::from_secs(60),
    );
    let failure = extract(Cursor::new(alpine_layer()), &Limits::DEFAULT, cancelled);
    assert!(matches!(failure, Err(Failure::Cancelled)), "{failure:?}");
}

#[test]
fn a_zstd_window_above_the_cap_and_too_many_gzip_members_end_the_layer_incomplete() {
    let window = [0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x90, 0x01, 0x00, 0x00];
    let facts = run(&window, &Limits::DEFAULT).unwrap();
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("window"), "{}", reasons(&facts));

    let layer = alpine_layer();
    let mut members = Vec::new();
    for chunk in layer.chunks(700) {
        members.extend_from_slice(&gzip(chunk));
    }
    let whole = run(&members, &Limits::DEFAULT).unwrap();
    assert!(!whole.incomplete, "{}", reasons(&whole));
    assert!(whole.apk.is_some());
    let capped = Limits {
        max_frames: 2,
        ..Limits::DEFAULT
    };
    let facts = run(&members, &capped).unwrap();
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("members"), "{}", reasons(&facts));
}

#[test]
fn entry_and_file_caps_end_the_layer_or_the_file_but_whiteouts_are_still_read() {
    let layer = tar(&[
        Entry::File("var/lib/dpkg/status", DPKG_DB.as_bytes()),
        Entry::File("var/lib/dpkg/status.d/.wh.old", b""),
        Entry::Dir("a/"),
        Entry::Dir("b/"),
    ]);
    let few = Limits {
        max_entries: 3,
        ..Limits::DEFAULT
    };
    let facts = run(&layer, &few).unwrap();
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("entries"));
    let small = Limits {
        max_file: 16,
        ..Limits::DEFAULT
    };
    let facts = run(&layer, &small).unwrap();
    assert!(facts.incomplete);
    assert!(facts.dpkg.is_none());
    assert_eq!(facts.unreadable, ["var/lib/dpkg/status"]);
    assert_eq!(facts.removed, ["var/lib/dpkg/status.d/old"]);
}

#[test]
fn long_names_are_honoured_and_an_oversized_name_or_pax_block_makes_the_layer_incomplete() {
    let long = "x".repeat(150);
    let mut builder = ::tar::Builder::new(Vec::new());
    let mut header = ::tar::Header::new_gnu();
    let path = format!("var/lib/dpkg/status.d/{long}");
    header.set_size(DPKG_DB.len() as u64);
    header.set_mode(0o644);
    builder
        .append_data(&mut header, &path, DPKG_DB.as_bytes())
        .unwrap();
    builder
        .append_pax_extensions([("path", b"etc/os-release".as_slice())])
        .unwrap();
    let mut header = ::tar::Header::new_gnu();
    header.set_size(ALPINE_OS.len() as u64);
    header.set_mode(0o644);
    builder
        .append_data(&mut header, "ignored-name", ALPINE_OS.as_bytes())
        .unwrap();
    let layer = builder.into_inner().unwrap();
    let facts = run(&layer, &Limits::DEFAULT).unwrap();
    assert!(!facts.incomplete, "{}", reasons(&facts));
    assert_eq!(facts.dpkg_status_d.len(), 1);
    assert!(facts.dpkg_status_d[0].path.ends_with(&long));
    assert_eq!(
        facts.os_release.as_ref().map(|os| os.id.as_str()),
        Some("alpine")
    );
    let tiny = Limits {
        max_meta: 8,
        ..Limits::DEFAULT
    };
    let facts = run(&layer, &tiny).unwrap();
    assert!(facts.incomplete);
    assert!(
        reasons(&facts).contains("long name or pax"),
        "{}",
        reasons(&facts)
    );
    assert!(facts.dpkg_status_d.is_empty());

    let mut builder = ::tar::Builder::new(Vec::new());
    builder
        .append_pax_extensions([("path", "y".repeat(100).as_bytes())])
        .unwrap();
    let mut header = ::tar::Header::new_gnu();
    header.set_size(ALPINE_OS.len() as u64);
    header.set_mode(0o644);
    builder
        .append_data(&mut header, "etc/os-release", ALPINE_OS.as_bytes())
        .unwrap();
    let facts = run(&builder.into_inner().unwrap(), &tiny).unwrap();
    assert!(facts.incomplete);
    assert!(facts.os_release.is_none());
}

#[test]
fn a_sparse_entry_at_a_wanted_path_makes_the_layer_incomplete() {
    let layer = raw(&[block("var/lib/dpkg/status", 0, EntryType::GNUSparse, None)]);
    let facts = run(&layer, &Limits::DEFAULT).unwrap();
    assert!(facts.incomplete);
    assert_eq!(facts.unreadable, ["var/lib/dpkg/status"]);

    let mut builder = ::tar::Builder::new(Vec::new());
    builder
        .append_pax_extensions([("GNU.sparse.major", b"1".as_slice())])
        .unwrap();
    let mut header = ::tar::Header::new_gnu();
    header.set_size(DPKG_DB.len() as u64);
    header.set_mode(0o644);
    builder
        .append_data(&mut header, "var/lib/dpkg/status", DPKG_DB.as_bytes())
        .unwrap();
    let facts = run(&builder.into_inner().unwrap(), &Limits::DEFAULT).unwrap();
    assert!(facts.incomplete);
    assert!(facts.dpkg.is_none());

    let elsewhere = raw(&[block("usr/share/sparse", 0, EntryType::GNUSparse, None)]);
    assert!(!run(&elsewhere, &Limits::DEFAULT).unwrap().incomplete);
}

#[test]
fn the_size_of_a_header_only_entry_is_ignored_like_the_extractors_do() {
    let layer = raw(&[
        block("hidden-in-data", 1024, EntryType::Symlink, Some("x")),
        with_data(
            block(
                "var/lib/dpkg/status",
                DPKG_DB.len() as u64,
                EntryType::Regular,
                None,
            ),
            DPKG_DB.as_bytes(),
        ),
    ]);
    let facts = run(&layer, &Limits::DEFAULT).unwrap();
    assert!(!facts.incomplete, "{}", reasons(&facts));
    assert_eq!(facts.dpkg.as_ref().map(Vec::len), Some(2));
}

#[test]
fn the_archive_ends_at_two_zero_blocks_or_eof_and_one_zero_block_before_data_is_malformed() {
    let os = with_data(
        block(
            "etc/os-release",
            ALPINE_OS.len() as u64,
            EntryType::Regular,
            None,
        ),
        ALPINE_OS.as_bytes(),
    );
    let mut stray = vec![0u8; 512];
    stray.extend_from_slice(&os);
    stray.extend_from_slice(&END);
    let facts = run(&stray, &Limits::DEFAULT).unwrap();
    assert!(facts.incomplete);
    assert!(
        reasons(&facts).contains("zero block"),
        "{}",
        reasons(&facts)
    );
    assert!(facts.os_release.is_none());

    let mut ended = os.clone();
    ended.extend_from_slice(&[0u8; 512]);
    assert!(!run(&ended, &Limits::DEFAULT).unwrap().incomplete);
    assert!(!run(&os, &Limits::DEFAULT).unwrap().incomplete);
}

#[test]
fn truncated_and_bit_flipped_copies_of_one_layer_never_panic() {
    let layer = tar(&[
        Entry::Dir("etc/"),
        Entry::File("etc/os-release", DEBIAN_OS.as_bytes()),
        Entry::File("var/lib/dpkg/status", DPKG_DB.as_bytes()),
        Entry::File("var/lib/dpkg/status.d/base", DPKG_DB.as_bytes()),
        Entry::File("var/lib/dpkg/.wh.old", b""),
        Entry::Symlink("etc/link", "os-release"),
    ]);
    let mut cases = 0;
    for bytes in [layer.clone(), gzip(&layer), zstd(&layer)] {
        let step = (bytes.len() / 60).max(1);
        for cut in (0..bytes.len()).step_by(step) {
            let _ = run(&bytes[..cut], &Limits::DEFAULT);
            cases += 1;
        }
        for position in (0..bytes.len()).step_by(step) {
            for bit in [0u8, 3, 7] {
                let mut flipped = bytes.clone();
                flipped[position] ^= 1 << bit;
                let _ = run(&flipped, &Limits::DEFAULT);
                cases += 1;
            }
        }
    }
    assert!(cases > 300, "{cases}");
}

#[test]
fn a_gzip_or_zstd_layer_cut_short_is_never_taken_for_a_complete_one() {
    let layer = alpine_layer();
    for bytes in [gzip(&layer), zstd(&layer)] {
        for cut in (1..bytes.len() / 2).step_by(7) {
            let facts = run(&bytes[..cut], &Limits::DEFAULT).unwrap();
            assert!(facts.incomplete, "{cut}");
        }
    }
}
