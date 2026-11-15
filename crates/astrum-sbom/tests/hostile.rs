mod common;

use astrum_sbom::parse;
use common::fixture;

const FIXTURES: [&str; 4] = [
    "trivy-spdx-2.3.json",
    "syft-cyclonedx-1.6.json",
    "cosign-statement-spdx.json",
    "cosign-dsse-cyclonedx.json",
];

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn truncated_documents_are_errors() {
    for name in FIXTURES {
        let bytes = fixture(name);
        let end = bytes.trim_ascii_end().len();
        assert!(parse(&bytes).is_ok(), "{name}");
        for cut in (0..end).step_by(7) {
            assert!(parse(&bytes[..cut]).is_err(), "{name} cut at {cut}");
        }
    }
}

#[test]
fn flipped_bits_never_panic() {
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut parsed = 0usize;
    let mut refused = 0usize;
    for name in FIXTURES {
        let original = fixture(name);
        for _ in 0..120 {
            let mut bytes = original.clone();
            for _ in 0..1 + next(&mut state) % 3 {
                let at = (next(&mut state) % bytes.len() as u64) as usize;
                bytes[at] ^= 1 << (next(&mut state) % 8);
            }
            match parse(&bytes) {
                Ok(_) => parsed += 1,
                Err(_) => refused += 1,
            }
        }
    }
    assert_eq!(parsed + refused, FIXTURES.len() * 120);
    assert!(refused > 0);
}
