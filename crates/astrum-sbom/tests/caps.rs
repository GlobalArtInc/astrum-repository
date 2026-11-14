mod common;

use astrum_sbom::{
    Error, Format, MAX_DEPTH, MAX_INPUT_BYTES, MAX_LICENSES, MAX_PACKAGES, MAX_STRING_BYTES, parse,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use common::{cyclonedx, find, fixture, spdx};

fn spdx_with_depth(depth: usize) -> Vec<u8> {
    let nest = depth - 1;
    format!(
        r#"{{"spdxVersion":"SPDX-2.3","packages":[{{"name":"a"}}],"extra":{}{}}}"#,
        "[".repeat(nest),
        "]".repeat(nest)
    )
    .into_bytes()
}

fn names(count: usize) -> String {
    vec![r#"{"name":"a"}"#; count].join(",")
}

#[test]
fn input_over_the_limit_is_refused_before_parsing() {
    assert_eq!(parse(&vec![0u8; MAX_INPUT_BYTES + 1]), Err(Error::TooLarge));
    let mut document = spdx(r#"{"name":"a"}"#);
    document.resize(MAX_INPUT_BYTES, b' ');
    assert_eq!(parse(&document).unwrap().packages.len(), 1);
    document.push(b' ');
    assert_eq!(parse(&document), Err(Error::TooLarge));
}

#[test]
fn more_packages_than_the_limit_are_refused() {
    let spdx_limit = parse(&spdx(&names(MAX_PACKAGES))).unwrap();
    assert_eq!(spdx_limit.packages.len(), MAX_PACKAGES);
    assert_eq!(
        parse(&spdx(&names(MAX_PACKAGES + 1))),
        Err(Error::TooManyPackages)
    );
    assert_eq!(
        parse(&cyclonedx(&names(MAX_PACKAGES + 1))),
        Err(Error::TooManyPackages)
    );
    let half = MAX_PACKAGES / 2 + 1;
    let nested = format!(
        r#"{{"name":"root","components":[{}]}},{}"#,
        names(half),
        names(half)
    );
    assert_eq!(parse(&cyclonedx(&nested)), Err(Error::TooManyPackages));
}

#[test]
fn nesting_deeper_than_the_limit_is_refused() {
    let accepted = parse(&spdx_with_depth(MAX_DEPTH)).unwrap();
    assert_eq!(accepted.format, Format::Spdx);
    assert_eq!(parse(&spdx_with_depth(MAX_DEPTH + 1)), Err(Error::TooDeep));
}

#[test]
fn brackets_inside_strings_do_not_count_as_nesting() {
    let noise = "[".repeat(200) + &r#"\""#.repeat(3) + &"{".repeat(200) + r#"\\"#;
    let document = spdx(&format!(r#"{{"name":"{noise}"}},{{"name":"b"}}"#));
    assert_eq!(parse(&document).unwrap().packages.len(), 2);
}

#[test]
fn nested_components_obey_the_depth_limit() {
    let chain = |levels: usize| {
        let mut open = String::new();
        let mut close = String::new();
        for level in 0..levels {
            open.push_str(&format!(r#"{{"name":"c{level}","components":["#));
            close.push_str("]}");
        }
        cyclonedx(&format!("{open}{close}"))
    };
    assert_eq!(parse(&chain(30)).unwrap().packages.len(), 30);
    assert_eq!(parse(&chain(40)), Err(Error::TooDeep));
}

fn wrapped(depth: usize) -> (String, String) {
    let statement = format!(
        r#"{{"_type":"https://in-toto.io/Statement/v1","predicateType":"https://spdx.dev/Document","predicate":{}}}"#,
        String::from_utf8(spdx_with_depth(depth)).unwrap()
    );
    let envelope = format!(
        r#"{{"payloadType":"application/vnd.in-toto+json","payload":"{}"}}"#,
        STANDARD.encode(&statement)
    );
    (statement, envelope)
}

#[test]
fn the_depth_of_a_wrapped_document_is_checked_after_unwrapping() {
    let (statement, envelope) = wrapped(MAX_DEPTH - 1);
    assert_eq!(parse(statement.as_bytes()).unwrap().packages.len(), 1);
    assert_eq!(parse(envelope.as_bytes()).unwrap().packages.len(), 1);
    let (statement, envelope) = wrapped(MAX_DEPTH);
    assert_eq!(parse(statement.as_bytes()), Err(Error::TooDeep));
    assert_eq!(parse(envelope.as_bytes()), Err(Error::TooDeep));
}

#[test]
fn long_strings_are_cut_on_a_character_boundary() {
    for unit in ["€", "é", "😀"] {
        let long = unit.repeat(MAX_STRING_BYTES);
        let document = spdx(&format!(
            r#"{{"name":"{long}","versionInfo":"{long}","licenseDeclared":"{long}","externalRefs":[{{"referenceType":"purl","referenceLocator":"pkg:npm/{long}"}}]}}"#
        ));
        let sbom = parse(&document).unwrap();
        let package = &sbom.packages[0];
        let values = [
            package.name.as_str(),
            package.version.as_str(),
            package.licenses[0].as_str(),
            package.purl.as_deref().unwrap(),
        ];
        for value in values {
            assert!(value.len() <= MAX_STRING_BYTES, "{unit}");
            assert!(value.len() > MAX_STRING_BYTES - unit.len(), "{unit}");
        }
        assert_eq!(package.name, unit.repeat(MAX_STRING_BYTES / unit.len()));
        assert_eq!(package.ecosystem.as_deref(), Some("npm"));
    }
}

#[test]
fn a_package_keeps_at_most_32_licences() {
    let licenses = (0..100)
        .map(|index| format!(r#"{{"license":{{"id":"L-{index}"}}}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let sbom = parse(&cyclonedx(&format!(
        r#"{{"name":"a","licenses":[{licenses}]}}"#
    )))
    .unwrap();
    let kept = &sbom.packages[0].licenses;
    assert_eq!(kept.len(), MAX_LICENSES);
    assert_eq!(kept[0], "L-0");
    assert_eq!(kept[MAX_LICENSES - 1], format!("L-{}", MAX_LICENSES - 1));
}

#[test]
fn a_document_of_the_cap_sizes_still_parses_a_real_fixture() {
    let sbom = parse(&fixture("syft-cyclonedx-1.6.json")).unwrap();
    assert_eq!(find(&sbom, "ms").version, "2.1.3");
}
