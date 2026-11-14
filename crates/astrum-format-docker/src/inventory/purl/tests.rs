use super::*;

fn mapped(purl: &str) -> Option<(String, String, String)> {
    match map(Some(purl), "") {
        Mapped::Package(package) => Some((package.ecosystem, package.name, package.version)),
        _ => None,
    }
}

fn row(ecosystem: &str, name: &str, version: &str) -> Option<(String, String, String)> {
    Some((ecosystem.to_owned(), name.to_owned(), version.to_owned()))
}

#[test]
fn every_language_purl_type_maps_to_its_osv_ecosystem_and_name() {
    let cases = [
        ("pkg:npm/lodash@4.17.21", row("npm", "lodash", "4.17.21")),
        (
            "pkg:npm/%40angular/core@17.0.0",
            row("npm", "@angular/core", "17.0.0"),
        ),
        (
            "pkg:npm/@angular/core@17.0.0",
            row("npm", "@angular/core", "17.0.0"),
        ),
        (
            "pkg:pypi/Django_Rest.Framework@3.1",
            row("PyPI", "django-rest-framework", "3.1"),
        ),
        (
            "pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1",
            row("Maven", "org.apache.logging.log4j:log4j-core", "2.14.1"),
        ),
        (
            "pkg:golang/github.com/gin-gonic/gin@v1.9.0",
            row("Go", "github.com/gin-gonic/gin", "v1.9.0"),
        ),
        ("pkg:cargo/serde@1.0.1", row("crates.io", "serde", "1.0.1")),
        (
            "pkg:nuget/Newtonsoft.Json@13.0.1",
            row("NuGet", "Newtonsoft.Json", "13.0.1"),
        ),
        ("pkg:gem/rails@7.0.0", row("RubyGems", "rails", "7.0.0")),
        (
            "pkg:composer/monolog/monolog@3.0.0",
            row("Packagist", "monolog/monolog", "3.0.0"),
        ),
        ("pkg:pub/http@1.0.0", row("Pub", "http", "1.0.0")),
        ("pkg:hex/phoenix@1.7.0", row("Hex", "phoenix", "1.7.0")),
    ];
    for (purl, expected) in cases {
        assert_eq!(mapped(purl), expected, "{purl}");
    }
}

#[test]
fn deb_and_apk_need_a_distro_from_the_qualifier_or_the_namespace_and_use_the_source_package() {
    let cases = [
        (
            "pkg:deb/debian/libc6@2.36-9?arch=amd64&distro=debian-12.5&upstream=glibc%402.36-9",
            row("Debian:12", "glibc", "2.36-9"),
        ),
        (
            "pkg:deb/debian/bash@5.2?distro=debian-12",
            row("Debian:12", "bash", "5.2"),
        ),
        (
            "pkg:deb/ubuntu/libssl3@3.0.2?distro=ubuntu-22.04&upstream=openssl",
            row("Ubuntu:22.04:LTS", "openssl", "3.0.2"),
        ),
        ("pkg:deb/ubuntu/libssl3@3.0.2?distro=ubuntu-23.10", None),
        ("pkg:deb/debian/bash@5.2", None),
        ("pkg:deb/debian/bash@5.2?distro=bookworm", None),
        (
            "pkg:apk/alpine/musl@1.2.4-r2?distro=alpine-3.19.1&upstream=musl",
            row("Alpine:v3.19", "musl", "1.2.4-r2"),
        ),
        (
            "pkg:apk/alpine/busybox-extras@1.36?distro=3.18.4&upstream=busybox",
            row("Alpine:v3.18", "busybox", "1.36"),
        ),
        ("pkg:apk/alpine/musl@1.2.4-r2", None),
        ("pkg:apk/wolfi/musl@1.2.4?distro=wolfi-20230201", None),
        (
            "pkg:deb/debian/libc6@2.36?distro=debian-12&epoch=1",
            row("Debian:12", "libc6", "1:2.36"),
        ),
    ];
    for (purl, expected) in cases {
        assert_eq!(mapped(purl), expected, "{purl}");
    }
}

#[test]
fn unknown_types_and_unusable_purls_are_skipped_and_a_missing_purl_is_not_listed() {
    for purl in [
        "pkg:rpm/fedora/bash@5.2?distro=fedora-39",
        "pkg:generic/tool@1",
        "pkg:github/actions/checkout@v4",
        "pkg:maven/log4j-core@2.14.1",
        "pkg:npm/lodash",
        "pkg:npm/%FF@1",
        "not a purl",
    ] {
        assert_eq!(map(Some(purl), ""), Mapped::Skipped, "{purl}");
    }
    assert_eq!(map(None, "1"), Mapped::Unlisted);
    assert_eq!(map(Some(""), "1"), Mapped::Unlisted);
    let long = format!("pkg:npm/{}@1", "a".repeat(2000));
    assert_eq!(map(Some(&long), ""), Mapped::Skipped);
}

#[test]
fn a_purl_without_a_version_takes_the_version_of_the_package() {
    assert_eq!(
        map(Some("pkg:npm/lodash"), "4.0.0"),
        Mapped::Package(PackageRef {
            ecosystem: "npm".to_owned(),
            name: "lodash".to_owned(),
            version: "4.0.0".to_owned(),
            purl: "pkg:npm/lodash".to_owned(),
        })
    );
}
