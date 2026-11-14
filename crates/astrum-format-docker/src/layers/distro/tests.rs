use super::*;

fn ecosystem(text: &str) -> Option<String> {
    let os = parse_os_release("etc/os-release", text.as_bytes())?;
    detect(&os).map(|distro| distro.ecosystem)
}

#[test]
fn os_release_maps_to_the_osv_ecosystem_of_each_family() {
    assert_eq!(
        ecosystem("ID=alpine\nVERSION_ID=3.19.1\n").as_deref(),
        Some("Alpine:v3.19")
    );
    assert_eq!(
        ecosystem("ID=debian\nVERSION_ID=\"12\"\n").as_deref(),
        Some("Debian:12")
    );
    assert_eq!(
        ecosystem("ID=ubuntu\nVERSION_ID=\"22.04\"\n").as_deref(),
        Some("Ubuntu:22.04:LTS")
    );
    assert_eq!(
        ecosystem("ID=ubuntu\nVERSION_ID='23.10'\n").as_deref(),
        Some("Ubuntu:23.10")
    );
}

#[test]
fn an_unknown_or_unversioned_distribution_has_no_ecosystem() {
    assert_eq!(ecosystem("ID=fedora\nVERSION_ID=40\n"), None);
    assert_eq!(ecosystem("ID=debian\n"), None);
    assert_eq!(ecosystem("NAME=nothing\n"), None);
    assert_eq!(ecosystem("# ID=alpine\nID=\nVERSION_ID=3.19\n"), None);
}
