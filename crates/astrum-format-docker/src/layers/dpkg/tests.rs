use super::*;

const ROOM: usize = 1000;

#[test]
fn source_is_read_with_and_without_a_version_and_only_installed_packages_count() {
    let text = "Package: libc6\nStatus: install ok installed\nVersion: 2.36-9\nSource: glibc (2.36-9)\nArchitecture: amd64\n\
Description: x\n y: not a field\n\n\
Package: bash\nStatus: install ok installed\nVersion: 5.2\n\n\
Package: old\nStatus: deinstall ok config-files\nVersion: 1\n\n\
Package: held\nStatus: install ok half-configured\nVersion: 1\n\n\
Package: nostatus\nVersion: 1\n";
    let parsed = parse(text.as_bytes(), ROOM);
    assert_eq!(parsed.malformed, 0);
    let names: Vec<&str> = parsed.packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["libc6", "bash"]);
    assert_eq!(parsed.packages[0].source.as_deref(), Some("glibc"));
    assert_eq!(parsed.packages[0].source_version.as_deref(), Some("2.36-9"));
    assert_eq!(parsed.packages[1].source, None);
}

#[test]
fn an_installed_record_without_a_version_is_malformed_and_a_status_d_file_is_one_stanza() {
    let broken = parse(b"Package: x\nStatus: install ok installed\n\n", ROOM);
    assert!(broken.packages.is_empty());
    assert_eq!(broken.malformed, 1);
    let file = parse(
        b"Package: base-files\nVersion: 12.4+deb12u5\nStatus: install ok installed\nArchitecture: amd64\n",
        ROOM,
    );
    assert_eq!(file.packages.len(), 1);
    assert_eq!(file.packages[0].version, "12.4+deb12u5");
    for garbage in [&b"\xff\x00\n\n: :\n"[..], b"", b"   \n\t\n", b"Status"] {
        let _ = parse(garbage, ROOM);
    }
}
