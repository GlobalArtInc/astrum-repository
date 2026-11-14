use super::*;
use crate::inventory::purl::{self, Mapped};
use crate::layers::apk::ApkPackage;

fn limits() -> Limits {
    Limits::DEFAULT
}

fn os(path: &str, id: &str, version: &str) -> Option<OsRelease> {
    Some(OsRelease {
        path: path.to_owned(),
        id: id.to_owned(),
        version_id: Some(version.to_owned()),
    })
}

fn deb(name: &str, version: &str, source: Option<(&str, Option<&str>)>) -> DpkgPackage {
    DpkgPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        source: source.map(|(source, _)| source.to_owned()),
        source_version: source.and_then(|(_, version)| version.map(str::to_owned)),
        arch: Some("amd64".to_owned()),
    }
}

fn apk(name: &str, origin: Option<&str>) -> ApkPackage {
    ApkPackage {
        name: name.to_owned(),
        version: "1.2.4-r2".to_owned(),
        origin: origin.map(str::to_owned),
        arch: None,
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
            deb("libc6", "2.36-9", Some(("glibc", None))),
            deb("libc-bin", "2.36-9", Some(("glibc", None))),
            deb("bash", "5.2", None),
        ]),
        dpkg_status_d: vec![status_file("x", deb("x", "1", None))],
        ..LayerFacts::default()
    }
}

fn alpine_base() -> LayerFacts {
    LayerFacts {
        os_release: os("etc/os-release", "alpine", "3.19.1"),
        apk: Some(vec![
            apk("musl-utils", Some("musl")),
            apk("musl", Some("musl")),
        ]),
        ..LayerFacts::default()
    }
}

#[test]
fn later_layers_replace_the_database_whole_and_the_os_release_of_the_highest_layer_wins() {
    let upgrade = LayerFacts {
        os_release: os("usr/lib/os-release", "debian", "11"),
        dpkg: Some(vec![deb("libc6", "2.31-13", Some(("glibc", None)))]),
        ..LayerFacts::default()
    };
    let merged = merge(
        vec![debian_base(), upgrade, LayerFacts::default()],
        &limits(),
    );
    assert_eq!(names(&merged), ["glibc@2.31-13", "x@1"]);
    assert!(merged.packages.iter().all(|p| p.ecosystem == "Debian:11"));
    assert!(merged.complete);
    assert!(
        merged.packages[0]
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
    assert_eq!(
        names(&merge(vec![debian_base(), whiteout], &limits())),
        ["x@1"]
    );
    let parent = LayerFacts {
        removed: vec!["var/lib/dpkg".to_owned()],
        ..LayerFacts::default()
    };
    assert!(
        merge(vec![debian_base(), parent], &limits())
            .packages
            .is_empty()
    );
    let opaque = LayerFacts {
        opaque: vec!["var/lib/dpkg/status.d".to_owned()],
        dpkg_status_d: vec![status_file("y", deb("y", "2", None))],
        ..LayerFacts::default()
    };
    let merged = merge(vec![debian_base(), opaque], &limits());
    assert_eq!(names(&merged), ["glibc@2.36-9", "bash@5.2", "y@2"]);
}

#[test]
fn an_unreadable_upper_database_drops_the_lower_packages_and_makes_the_result_incomplete() {
    let broken = LayerFacts {
        unreadable: vec!["lib/apk/db/installed".to_owned()],
        ..LayerFacts::unavailable("lib/apk/db/installed is larger than 64 MiB")
    };
    let merged = merge(vec![alpine_base(), broken], &limits());
    assert!(merged.packages.is_empty());
    assert!(!merged.complete);
    let unreadable_part = LayerFacts {
        unreadable: vec!["var/lib/dpkg/status.d/x".to_owned()],
        ..LayerFacts::unavailable("a status file could not be read")
    };
    let merged = merge(vec![debian_base(), unreadable_part], &limits());
    assert_eq!(names(&merged), ["glibc@2.36-9", "bash@5.2"]);
    assert!(!merged.complete);
}

#[test]
fn the_matching_version_is_the_source_version_and_both_paths_agree_on_every_package() {
    let layer = LayerFacts {
        os_release: os("etc/os-release", "debian", "12"),
        dpkg: Some(vec![
            deb("libc6", "1:2.36-9+b1", Some(("glibc", Some("2.36-9")))),
            deb("tzdata", "2024a-0+deb12u1", None),
            deb("curl", "7.88.1-10", Some(("curl-src", None))),
        ]),
        ..LayerFacts::default()
    };
    let merged = merge(vec![layer], &limits());
    assert_eq!(
        names(&merged),
        [
            "glibc@2.36-9",
            "tzdata@2024a-0+deb12u1",
            "curl-src@7.88.1-10"
        ]
    );
    let alpine = merge(vec![alpine_base()], &limits());
    for package in merged.packages.iter().chain(&alpine.packages) {
        assert_eq!(
            purl::map(Some(&package.purl), ""),
            Mapped::Package(package.clone())
        );
    }
}

#[test]
fn apk_packages_use_the_origin_and_unsupported_or_incomplete_inputs_are_flagged() {
    let merged = merge(vec![alpine_base()], &limits());
    assert_eq!(names(&merged), ["musl@1.2.4-r2"]);
    assert_eq!(merged.packages[0].ecosystem, "Alpine:v3.19");
    assert!(merged.packages[0].purl.contains("upstream=musl"));

    let mut ubuntu = debian_base();
    ubuntu.os_release = os("etc/os-release", "ubuntu", "23.10");
    let merged = merge(vec![ubuntu.clone()], &limits());
    assert!(merged.packages.is_empty());
    assert!(!merged.complete);
    assert_eq!(
        purl::map(Some("pkg:deb/ubuntu/bash@5.2?distro=ubuntu-23.10"), ""),
        Mapped::Skipped
    );
    ubuntu.os_release = os("etc/os-release", "ubuntu", "22.04");
    let merged = merge(vec![ubuntu], &limits());
    assert_eq!(merged.packages[0].ecosystem, "Ubuntu:22.04:LTS");

    let distroless = merge(vec![LayerFacts::default()], &limits());
    assert!(distroless.packages.is_empty());
    assert!(distroless.complete);

    let missing = LayerFacts::unavailable("a layer blob is missing");
    let merged = merge(vec![missing, alpine_base()], &limits());
    assert!(!merged.complete);
    assert_eq!(merged.reasons, ["a layer blob is missing"]);
}

#[test]
fn a_hundred_and_twenty_eight_layers_fold_one_at_a_time_and_the_merged_list_has_a_budget() {
    let mut layers = (0..128).map(|index| LayerFacts {
        os_release: os("etc/os-release", "debian", "12"),
        dpkg: Some(vec![deb("pkg", &format!("{index}"), None)]),
        ..LayerFacts::default()
    });
    let mut merge = Merge::new(&limits());
    for layer in layers.by_ref() {
        merge.apply(layer);
    }
    assert_eq!(names(&merge.finish()), ["pkg@127"]);
    let tight = Limits {
        merged_budget: 64,
        ..limits()
    };
    let merged = merge_all(debian_base(), &tight);
    assert!(!merged.complete);
    assert!(merged.packages.len() < 3);
}

fn merge_all(layer: LayerFacts, limits: &Limits) -> Merged {
    merge(std::iter::once(layer), limits)
}
