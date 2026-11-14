use std::sync::atomic::AtomicBool;

use super::*;
use crate::layers::fixtures::*;
use crate::layers::limits::Limits;

fn run(bytes: &[u8], limits: &Limits) -> Result<LayerFacts, Failure> {
    extract(bytes, limits, Arc::new(AtomicBool::new(false)))
}

fn facts(entries: &[Entry<'_>]) -> LayerFacts {
    run(&tar(entries), &Limits::DEFAULT).unwrap()
}

fn reasons(facts: &LayerFacts) -> String {
    facts.reasons.join("; ")
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
fn links_are_never_followed_and_dotdot_names_are_dropped_while_absolute_names_are_made_relative() {
    let facts = facts(&[
        Entry::Symlink("etc/os-release", "../usr/lib/os-release"),
        Entry::Hardlink("lib/apk/db/installed", "elsewhere"),
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
    assert!(facts.apk.is_none());
    assert!(facts.dpkg.is_none());
    assert!(facts.dpkg_status_d.is_empty());
}

#[test]
fn a_huge_claimed_size_with_a_short_body_ends_the_layer_incomplete_without_allocating() {
    let five_gib = 5 * 1024 * 1024 * 1024;
    for name in ["etc/os-release", "usr/share/blob"] {
        let mut layer = claim(name, five_gib, &[b'a'; 100]);
        layer.resize(layer.len() + 1024, 0);
        let facts = run(&layer, &Limits::DEFAULT).unwrap();
        assert!(facts.incomplete, "{name}");
        assert!(facts.os_release.is_none());
    }
}

#[test]
fn a_gzip_bomb_is_cut_at_the_output_cap_and_a_hostile_ratio_is_cut_early() {
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
    let ratio = Limits {
        ratio_floor: 1024 * 1024,
        max_ratio: 50,
        ..Limits::DEFAULT
    };
    let facts = run(&bomb, &ratio).unwrap();
    assert!(facts.incomplete);
    assert!(reasons(&facts).contains("inflates"), "{}", reasons(&facts));
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
        Entry::File("etc/os-release", ALPINE_OS.as_bytes()),
        Entry::File("var/lib/dpkg/.wh.status", b""),
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
    assert!(facts.os_release.is_none());
    assert_eq!(facts.removed, ["var/lib/dpkg/status"]);
}

#[test]
fn long_names_are_honoured_and_an_oversized_name_block_leaves_the_entry_unnamed() {
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
    assert!(facts.dpkg_status_d.is_empty());
    assert!(facts.os_release.is_none());
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
