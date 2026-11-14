use super::*;
use crate::layers::apk::ApkPackage;
use crate::layers::facts::StatusFile;

const ROOM: usize = 1000;

fn os(path: &str, id: &str, version: &str) -> Option<OsRelease> {
    Some(OsRelease {
        path: path.to_owned(),
        id: id.to_owned(),
        version_id: Some(version.to_owned()),
    })
}

fn deb(name: &str, version: &str, source: Option<&str>) -> DpkgPackage {
    DpkgPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        source: source.map(str::to_owned),
        source_version: None,
        arch: Some("amd64".to_owned()),
    }
}

fn status_file(name: &str, package: DpkgPackage) -> StatusFile {
    StatusFile {
        path: format!("var/lib/dpkg/status.d/{name}"),
        packages: vec![package],
    }
}

fn names(merged: &Merged) -> Vec<String> {
    merged
        .packages
        .iter()
        .map(|package| format!("{}@{}", package.name, package.version))
        .collect()
}

fn debian_base() -> LayerFacts {
    LayerFacts {
        os_release: os("etc/os-release", "debian", "12"),
        dpkg: Some(vec![
            deb("libc6", "2.36-9", Some("glibc")),
            deb("libc-bin", "2.36-9", Some("glibc")),
            deb("bash", "5.2", None),
        ]),
        dpkg_status_d: vec![status_file("x", deb("x", "1", None))],
        ..LayerFacts::default()
    }
}

#[test]
fn later_layers_replace_the_database_whole_and_the_os_release_of_the_highest_layer_wins() {
    let upgrade = LayerFacts {
        os_release: os("usr/lib/os-release", "debian", "11"),
        dpkg: Some(vec![deb("libc6", "2.31-13", Some("glibc"))]),
        ..LayerFacts::default()
    };
    let noise = LayerFacts::default();
    let merged = merge(&[debian_base(), upgrade, noise], ROOM);
    assert_eq!(names(&merged), ["glibc@2.31-13", "x@1"]);
    assert!(merged.packages.iter().all(|p| p.ecosystem == "Debian:11"));
    assert!(merged.complete);
    let first = &merged.packages[0];
    assert_eq!(first.version, "2.31-13");
    assert!(
        first
            .purl
            .starts_with("pkg:deb/debian/libc6@2.31-13?arch=amd64&distro=debian-11&upstream=glibc")
    );
}

#[test]
fn whiteouts_delete_a_database_and_an_opaque_directory_hides_the_lower_status_files() {
    let whiteout = LayerFacts {
        removed: vec!["var/lib/dpkg/status".to_owned()],
        ..LayerFacts::default()
    };
    let merged = merge(&[debian_base(), whiteout], ROOM);
    assert_eq!(names(&merged), ["x@1"]);

    let parent_removed = LayerFacts {
        removed: vec!["var/lib/dpkg".to_owned()],
        ..LayerFacts::default()
    };
    assert!(
        merge(&[debian_base(), parent_removed], ROOM)
            .packages
            .is_empty()
    );

    let opaque = LayerFacts {
        opaque: vec!["var/lib/dpkg/status.d".to_owned()],
        dpkg_status_d: vec![status_file("y", deb("y", "2", None))],
        ..LayerFacts::default()
    };
    let merged = merge(&[debian_base(), opaque], ROOM);
    assert_eq!(names(&merged), ["glibc@2.36-9", "bash@5.2", "y@2"]);
}

#[test]
fn apk_packages_use_the_origin_and_an_unknown_distribution_or_a_lower_incomplete_layer_is_flagged()
{
    let alpine = LayerFacts {
        os_release: os("etc/os-release", "alpine", "3.19.1"),
        apk: Some(vec![
            ApkPackage {
                name: "musl-utils".to_owned(),
                version: "1.2.4-r2".to_owned(),
                origin: Some("musl".to_owned()),
                arch: None,
            },
            ApkPackage {
                name: "musl".to_owned(),
                version: "1.2.4-r2".to_owned(),
                origin: Some("musl".to_owned()),
                arch: None,
            },
        ]),
        ..LayerFacts::default()
    };
    let merged = merge(std::slice::from_ref(&alpine), ROOM);
    assert_eq!(names(&merged), ["musl@1.2.4-r2"]);
    assert_eq!(merged.packages[0].ecosystem, "Alpine:v3.19");
    assert!(merged.packages[0].purl.contains("upstream=musl"));

    let unknown = LayerFacts {
        os_release: os("etc/os-release", "fedora", "40"),
        ..alpine.clone()
    };
    let merged = merge(&[unknown], ROOM);
    assert!(merged.packages.is_empty());
    assert!(!merged.complete);

    let distroless = merge(&[LayerFacts::default()], ROOM);
    assert!(distroless.packages.is_empty());
    assert!(distroless.complete);

    let broken = LayerFacts::unavailable("a layer blob is missing");
    let merged = merge(&[broken, alpine], ROOM);
    assert!(!merged.complete);
    assert_eq!(merged.reasons, ["a layer blob is missing"]);
}
