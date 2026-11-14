mod common;

use astrum_sbom::{Error, Format, parse};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{cyclonedx, find, fixture, fixture_json, spdx};
use serde_json::json;

#[test]
fn spdx_documents_of_syft_and_trivy_parse() {
    for name in [
        "syft-spdx-2.2.json",
        "syft-spdx-2.3.json",
        "trivy-spdx-2.3.json",
    ] {
        let sbom = parse(&fixture(name)).unwrap();
        assert_eq!(sbom.format, Format::Spdx, "{name}");
        let left_pad = find(&sbom, "left-pad");
        assert_eq!(left_pad.version, "1.3.0", "{name}");
        assert_eq!(
            left_pad.purl.as_deref(),
            Some("pkg:npm/left-pad@1.3.0"),
            "{name}"
        );
        assert_eq!(left_pad.ecosystem.as_deref(), Some("npm"), "{name}");
        assert_eq!(left_pad.licenses, ["WTFPL"], "{name}");
        assert_eq!(find(&sbom, "itoa").ecosystem.as_deref(), Some("crates.io"));
        assert_eq!(
            find(&sbom, "github.com/google/uuid").ecosystem.as_deref(),
            Some("Go")
        );
    }
    let syft = parse(&fixture("syft-spdx-2.3.json")).unwrap();
    let root = find(&syft, "demo");
    assert!(root.licenses.is_empty());
}

#[test]
fn files_applications_and_sources_without_a_purl_are_not_packages() {
    let real = [
        "demo",
        "demo",
        "github.com/google/uuid",
        "itoa",
        "left-pad",
        "ms",
    ];
    let trivy_real = [
        "demo",
        "example.com/demo",
        "github.com/google/uuid",
        "itoa",
        "left-pad",
        "ms",
    ];
    let names = |file: &str| -> Vec<String> {
        let mut names: Vec<String> = parse(&fixture(file))
            .unwrap()
            .packages
            .into_iter()
            .map(|package| package.name)
            .collect();
        names.sort();
        names
    };
    assert_eq!(names("syft-spdx-2.3.json"), real);
    assert_eq!(names("trivy-spdx-2.3.json"), trivy_real);
    assert_eq!(names("syft-spdx-2.2.json").len(), 7);
    for version in ["1.4", "1.5", "1.6", "1.7"] {
        assert_eq!(names(&format!("syft-cyclonedx-{version}.json")), real);
    }
    let sbom = parse(&spdx(
        r#"{"name":"file","primaryPackagePurpose":"FILE"},
           {"name":"app","primaryPackagePurpose":"application"},
           {"name":"src","primaryPackagePurpose":"SOURCE"},
           {"name":"lib","primaryPackagePurpose":"LIBRARY"},
           {"name":"kept-file","primaryPackagePurpose":"FILE","externalRefs":[
             {"referenceType":"purl","referenceLocator":"pkg:generic/kept-file@1"}]}"#,
    ))
    .unwrap();
    let kept: Vec<&str> = sbom.packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(kept, ["lib", "kept-file"]);
    let sbom = parse(&cyclonedx(
        r#"{"type":"file","name":"/etc/passwd"},
           {"type":"file","name":"/etc/app.conf","purl":"pkg:generic/app.conf@1"},
           {"type":"application","name":"app"},
           {"type":"file","name":"holder","components":[{"type":"library","name":"inner"}]}"#,
    ))
    .unwrap();
    let kept: Vec<&str> = sbom.packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(kept, ["/etc/app.conf", "app", "inner"]);
}

#[test]
fn spdx_prefers_the_concluded_licence_and_drops_placeholders() {
    let sbom = parse(&spdx(
        r#"{"name":"a","licenseConcluded":"MIT","licenseDeclared":"Apache-2.0"},
           {"name":"b","licenseConcluded":"NOASSERTION","licenseDeclared":"BSD-3-Clause"},
           {"name":"c","licenseConcluded":"NONE","licenseDeclared":"NOASSERTION"},
           {"name":"d","licenseConcluded":"NOASSERTION","licenseDeclared":"NONE"},
           {"name":"e"},
           {"name":"f","licenseConcluded":"LicenseRef-x","externalRefs":[
             {"referenceType":"cpe23Type","referenceLocator":"cpe:2.3:a:f"},
             {"referenceType":"purl","referenceLocator":"pkg:pypi/f@1"}]}"#,
    ))
    .unwrap();
    assert_eq!(find(&sbom, "a").licenses, ["MIT"]);
    assert_eq!(find(&sbom, "b").licenses, ["BSD-3-Clause"]);
    for name in ["c", "d", "e"] {
        assert!(find(&sbom, name).licenses.is_empty(), "{name}");
    }
    let f = find(&sbom, "f");
    assert_eq!(f.licenses, ["LicenseRef-x"]);
    assert_eq!(f.purl.as_deref(), Some("pkg:pypi/f@1"));
    assert_eq!(f.ecosystem.as_deref(), Some("PyPI"));
    assert_eq!(f.version, "");
}

#[test]
fn cyclonedx_1_4_to_1_7_parse() {
    for version in ["1.4", "1.5", "1.6", "1.7"] {
        let sbom = parse(&fixture(&format!("syft-cyclonedx-{version}.json"))).unwrap();
        assert_eq!(sbom.format, Format::CycloneDx, "{version}");
        let ms = find(&sbom, "ms");
        assert_eq!(ms.version, "2.1.3", "{version}");
        assert_eq!(ms.purl.as_deref(), Some("pkg:npm/ms@2.1.3"), "{version}");
        assert_eq!(ms.ecosystem.as_deref(), Some("npm"), "{version}");
        assert_eq!(ms.licenses, ["MIT"], "{version}");
        assert_eq!(find(&sbom, "demo").purl, None);
    }
}

#[test]
fn cyclonedx_versions_from_1_2_to_1_7_are_accepted() {
    for version in ["1.2", "1.3", "1.7"] {
        let document = format!(
            r#"{{"bomFormat":"CycloneDX","specVersion":"{version}","components":[{{"name":"a"}}]}}"#
        );
        assert_eq!(
            parse(document.as_bytes()).unwrap().packages.len(),
            1,
            "{version}"
        );
    }
    let newer = br#"{"bomFormat":"CycloneDX","specVersion":"1.8","components":[]}"#;
    assert!(is_unsupported(parse(newer)));
}

#[test]
fn nested_components_are_flattened_and_purl_types_are_mapped() {
    let sbom = parse(&fixture("cyclonedx-nested-1.5.json")).unwrap();
    assert_eq!(sbom.packages.len(), 10);
    assert!(sbom.packages.iter().all(|package| package.name != "shop"));
    assert_eq!(find(&sbom, "web").licenses, ["Apache-2.0", "Custom Terms"]);
    assert_eq!(find(&sbom, "json").licenses, ["MIT OR GPL-2.0-only"]);
    let expected = [
        ("web", Some("Maven")),
        ("json", Some("PyPI")),
        ("deep", Some("RubyGems")),
        ("libssl", Some("Debian:12")),
        ("musl", Some("Alpine:v3.19")),
        ("glibc", None),
        ("Newtonsoft.Json", Some("NuGet")),
        ("blob", None),
        ("weird", None),
        ("nameless-purl", None),
    ];
    for (name, ecosystem) in expected {
        assert_eq!(find(&sbom, name).ecosystem.as_deref(), ecosystem, "{name}");
    }
}

#[test]
fn distro_packages_map_to_the_scanner_ecosystems_or_to_none() {
    let cases = [
        ("pkg:deb/debian/curl@7?distro=debian-12", Some("Debian:12")),
        (
            "pkg:deb/debian/curl@7?arch=amd64&distro=debian-11.9#x",
            Some("Debian:11"),
        ),
        (
            "pkg:deb/ubuntu/curl@7?distro=ubuntu-22.04",
            Some("Ubuntu:22.04:LTS"),
        ),
        (
            "pkg:deb/ubuntu/curl@7?distro=ubuntu-24.04.1",
            Some("Ubuntu:24.04:LTS"),
        ),
        ("pkg:deb/ubuntu/curl@7?distro=ubuntu-23.10", None),
        ("pkg:deb/ubuntu/curl@7?distro=ubuntu-21.04", None),
        ("pkg:deb/debian/curl@7?distro=debian-sid", None),
        ("pkg:deb/debian/curl@7?distro=kali-2024", None),
        ("pkg:deb/debian/curl@7", None),
        ("pkg:deb/debian/curl@7?distro=debian", None),
        (
            "pkg:apk/alpine/musl@1?distro=alpine-3.19",
            Some("Alpine:v3.19"),
        ),
        (
            "pkg:apk/alpine/musl@1?distro=alpine-3.19.1",
            Some("Alpine:v3.19"),
        ),
        ("pkg:apk/alpine/musl@1?distro=alpine-edge", None),
        ("pkg:apk/alpine/musl@1?distro=alpine-3", None),
        ("pkg:apk/alpine/musl@1", None),
        ("pkg:rpm/fedora/glibc@2?distro=fedora-39", None),
        ("pkg:rpm/fedora/glibc@2", None),
        ("pkg:NPM/left-pad@1", Some("npm")),
        ("not-a-purl", None),
        ("pkg:", None),
    ];
    for (purl, ecosystem) in cases {
        let document = cyclonedx(&format!(r#"{{"name":"a","purl":"{purl}"}}"#));
        let sbom = parse(&document).unwrap();
        assert_eq!(sbom.packages[0].ecosystem.as_deref(), ecosystem, "{purl}");
    }
}

#[test]
fn statements_and_envelopes_give_the_same_packages_as_their_documents() {
    for (statement, envelope, format) in [
        (
            "cosign-statement-spdx.json",
            "cosign-dsse-spdx.json",
            Format::Spdx,
        ),
        (
            "cosign-statement-cyclonedx.json",
            "cosign-dsse-cyclonedx.json",
            Format::CycloneDx,
        ),
    ] {
        let from_statement = parse(&fixture(statement)).unwrap();
        let from_envelope = parse(&fixture(envelope)).unwrap();
        assert_eq!(from_statement.format, format, "{statement}");
        assert_eq!(from_statement, from_envelope, "{envelope}");
        assert_eq!(find(&from_statement, "left-pad").licenses, ["WTFPL"]);
    }
}

#[test]
fn statements_accept_any_field_order_and_versioned_predicate_types() {
    let document = fixture_json("syft-spdx-2.3.json");
    let statement = json!({
        "predicate": document,
        "predicateType": "https://spdx.dev/Document/v2.3",
        "subject": [],
        "_type": "https://in-toto.io/Statement/v1",
    });
    let sbom = parse(statement.to_string().as_bytes()).unwrap();
    assert_eq!(sbom.format, Format::Spdx);
    assert_eq!(sbom.packages.len(), 6);
    let encoded = STANDARD.encode(statement.to_string());
    let envelope = json!({
        "signatures": [],
        "payload": encoded,
        "payloadType": "application/vnd.in-toto+json",
    });
    assert_eq!(parse(envelope.to_string().as_bytes()).unwrap(), sbom);
}

fn envelope(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    json!({"payloadType": payload_type, "payload": STANDARD.encode(payload), "signatures": []})
        .to_string()
        .into_bytes()
}

fn is_unsupported(result: Result<astrum_sbom::Sbom, Error>) -> bool {
    matches!(result, Err(Error::Unsupported(_)))
}

#[test]
fn unknown_shapes_and_versions_are_unsupported() {
    let statement = fixture("cosign-statement-spdx.json");
    assert!(is_unsupported(parse(
        br#"{"@context":"https://spdx.org/rdf/3.0.1/spdx-context.jsonld","@graph":[]}"#
    )));
    assert!(is_unsupported(parse(
        br#"{"spdxVersion":"SPDX-3.0","packages":[]}"#
    )));
    assert!(is_unsupported(parse(
        br#"<?xml version="1.0"?><bom xmlns="http://cyclonedx.org/schema/bom/1.5"/>"#
    )));
    assert!(is_unsupported(parse(
        br#"{"bomFormat":"CycloneDX","specVersion":"2.0","components":[]}"#
    )));
    assert!(is_unsupported(parse(
        br#"{"bomFormat":"Other","specVersion":"1.5"}"#
    )));
    assert!(is_unsupported(parse(
        br#"{"_type":"https://in-toto.io/Statement/v1","predicateType":"https://slsa.dev/provenance/v1","predicate":{}}"#
    )));
    for predicate in ["https://spdx.dev/Document", "https://cyclonedx.org/bom"] {
        let v1 = format!(
            r#"{{"_type":"https://in-toto.io/Statement/v0.1","predicateType":"{predicate}","predicate":{{"Data":"opaque","Timestamp":"now"}}}}"#
        );
        assert!(is_unsupported(parse(v1.as_bytes())), "{predicate}");
        assert!(is_unsupported(parse(&envelope(
            "application/vnd.in-toto+json",
            v1.as_bytes()
        ))));
    }
    assert!(is_unsupported(parse(&envelope(
        "application/json",
        &statement
    ))));
    assert!(is_unsupported(parse(&envelope(
        "application/vnd.in-toto+json",
        &fixture("syft-spdx-2.3.json")
    ))));
}

#[test]
fn an_envelope_inside_an_envelope_is_refused() {
    let inner = envelope(
        "application/vnd.in-toto+json",
        &fixture("cosign-statement-spdx.json"),
    );
    let outer = envelope("application/vnd.in-toto+json", &inner);
    assert_eq!(
        parse(&outer),
        Err(Error::Unsupported(
            "an envelope inside an envelope".to_owned()
        ))
    );
}

#[test]
fn broken_documents_are_malformed_and_never_echoed() {
    for input in [
        &b""[..],
        b"   ",
        b"[1,2]",
        b"\"text\"",
        b"{\"spdxVersion\":\"SPDX-2.3\",\"packages\":",
        b"{\"spdxVersion\":\"SPDX-2.3\",\"packages\":{\"secret-marker\":1}}",
        b"{\"spdxVersion\":\"SPDX-2.3\"} trailing-marker",
        b"{\"bomFormat\":\"CycloneDX\",\"components\":[]}",
        b"{\"_type\":\"https://in-toto.io/Statement/v1\",\"predicateType\":\"https://spdx.dev/Document\"}",
        b"{\"payloadType\":\"application/vnd.in-toto+json\",\"payload\":\"***\"}",
        b"{\"payloadType\":\"application/vnd.in-toto+json\"}",
    ] {
        match parse(input) {
            Err(Error::Malformed(reason)) => {
                assert!(!reason.contains("marker"), "{reason}");
                assert!(!reason.contains("***"), "{reason}");
            }
            other => panic!("{:?} gave {other:?}", String::from_utf8_lossy(input)),
        }
    }
}

#[test]
fn nameless_entries_are_skipped_and_odd_scalars_are_ignored() {
    let sbom = parse(&cyclonedx(
        r#"{"version":"1"},{"name":"","version":"1"},{"name":7,"version":"1"},{"name":"ok","version":2,"purl":null}"#,
    ))
    .unwrap();
    assert_eq!(sbom.packages.len(), 1);
    let ok = find(&sbom, "ok");
    assert_eq!(ok.version, "");
    assert_eq!(ok.purl, None);
}

#[test]
fn null_lists_a_leading_bom_and_wrapped_base64_are_tolerated() {
    let sbom = parse(&spdx(
        r#"{"name":"a","externalRefs":null},{"name":"b","externalRefs":[]}"#,
    ))
    .unwrap();
    assert_eq!(sbom.packages.len(), 2);
    let sbom = parse(&cyclonedx(
        r#"{"name":"a","licenses":null,"components":null},{"name":"b","components":[]}"#,
    ))
    .unwrap();
    assert_eq!(sbom.packages.len(), 2);
    let empty = parse(br#"{"spdxVersion":"SPDX-2.3","packages":null}"#).unwrap();
    assert!(empty.packages.is_empty());
    let empty =
        parse(br#"{"bomFormat":"CycloneDX","specVersion":"1.6","components":null}"#).unwrap();
    assert!(empty.packages.is_empty());

    let mut with_bom = vec![0xEF, 0xBB, 0xBF];
    with_bom.extend_from_slice(&fixture("syft-cyclonedx-1.6.json"));
    assert_eq!(parse(&with_bom).unwrap().packages.len(), 6);

    let statement = fixture("cosign-statement-spdx.json");
    let encoded = STANDARD.encode(&statement);
    let wrapped: Vec<String> = encoded
        .as_bytes()
        .chunks(76)
        .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
        .collect();
    let payload = format!("{}\n \t", wrapped.join("\r\n"));
    let document = json!({"payloadType": "application/vnd.in-toto+json", "payload": payload});
    let expected = parse(&statement).unwrap();
    assert_eq!(parse(document.to_string().as_bytes()).unwrap(), expected);
    let mut inner_bom = vec![0xEF, 0xBB, 0xBF];
    inner_bom.extend_from_slice(&statement);
    assert_eq!(
        parse(&envelope("application/vnd.in-toto+json", &inner_bom)).unwrap(),
        expected
    );
}

#[test]
fn repeated_keys_inside_a_package_are_malformed() {
    for document in [
        spdx(r#"{"name":"a","name":"b"}"#),
        cyclonedx(r#"{"name":"a","name":"b"}"#),
        cyclonedx(r#"{"name":"a","components":[],"components":[]}"#),
    ] {
        assert!(matches!(parse(&document), Err(Error::Malformed(_))));
    }
}
