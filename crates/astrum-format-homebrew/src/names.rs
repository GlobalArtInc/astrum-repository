const CORE: [&str; 3] = ["v2", "homebrew", "core"];
const IMAGE_SEGMENT_MAX: usize = 64;
const TAG_MAX: usize = 128;
const DIGEST_PREFIX: &str = "sha256:";
const DIGEST_HEX: usize = 64;
const API_SUFFIX: &str = ".jws.json";
const API_FILE_MAX: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Api { endpoint: String },
    Blob { image: String, hex: String },
    Manifest { image: String, tag: String },
}

fn api_file(name: &str) -> bool {
    name.len() <= API_FILE_MAX
        && name.strip_suffix(API_SUFFIX).is_some_and(|stem| {
            !stem.is_empty()
                && stem
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
        })
}

fn image_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= IMAGE_SEGMENT_MAX
        && segment != "."
        && segment != ".."
        && segment.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
}

fn image(segments: &[&str]) -> Option<String> {
    ((1..=2).contains(&segments.len()) && segments.iter().all(|segment| image_segment(segment)))
        .then(|| segments.join("/"))
}

fn tag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= TAG_MAX
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn digest_hex(value: &str) -> Option<&str> {
    let hex = value.strip_prefix(DIGEST_PREFIX)?;
    (hex.len() == DIGEST_HEX
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
    .then_some(hex)
}

pub fn parse(path: &str) -> Option<Item> {
    let segments: Vec<&str> = path.strip_prefix('/')?.split('/').collect();
    match segments.as_slice() {
        ["api", file] if api_file(file) => Some(Item::Api {
            endpoint: (*file).to_owned(),
        }),
        ["api", "internal", file] if api_file(file) => Some(Item::Api {
            endpoint: format!("internal/{file}"),
        }),
        [v2, homebrew, core, rest @ ..] if [*v2, *homebrew, *core] == CORE && rest.len() >= 3 => {
            let (image_segments, tail) = rest.split_at(rest.len() - 2);
            let image = image(image_segments)?;
            match tail {
                ["blobs", reference] => Some(Item::Blob {
                    image,
                    hex: digest_hex(reference)?.to_owned(),
                }),
                ["manifests", reference] if tag(reference) => Some(Item::Manifest {
                    image,
                    tag: (*reference).to_owned(),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn api_endpoints_are_a_closed_set_of_names() {
        for good in [
            "/api/formula.jws.json",
            "/api/internal/packages.arm64_sonoma.jws.json",
            "/api/cask_tap_migrations.jws.json",
        ] {
            assert!(matches!(parse(good), Some(Item::Api { .. })), "{good}");
        }
        for bad in [
            "/api/formula.json",
            "/api/.jws.json",
            "/api/Formula.jws.json",
            "/api/internal/x/y.jws.json",
            "/api/internal/",
            "/api/other/x.jws.json",
            "/api/x.jws.json/",
            "/api/x-y.jws.json",
            "/api",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn api_file_names_are_capped() {
        let named =
            |length: usize| format!("/api/{}{API_SUFFIX}", "a".repeat(length - API_SUFFIX.len()));
        assert!(parse(&named(128)).is_some());
        assert_eq!(parse(&named(129)), None);
    }

    #[test]
    fn images_have_one_or_two_segments() {
        let blob = |image: &str| format!("/v2/homebrew/core/{image}/blobs/sha256:{HEX}");
        assert!(matches!(parse(&blob("jq")), Some(Item::Blob { .. })));
        assert!(matches!(parse(&blob("openssl/3")), Some(Item::Blob { .. })));
        for bad in ["a/b/c", "Jq", "a/..", "../a", "a b", "a@3", &"x".repeat(65)] {
            assert_eq!(parse(&blob(bad)), None, "{bad}");
        }
        assert_eq!(parse("/v2/homebrew/core/blobs/sha256:00"), None);
    }

    #[test]
    fn digests_are_lowercase_sha256() {
        let path = |digest: &str| format!("/v2/homebrew/core/jq/blobs/{digest}");
        assert!(parse(&path(&format!("sha256:{HEX}"))).is_some());
        for bad in [
            format!("sha256:{}", HEX.to_uppercase()),
            format!("sha256:{}", &HEX[1..]),
            format!("sha256:{HEX}0"),
            format!("sha512:{HEX}"),
            HEX.to_owned(),
        ] {
            assert_eq!(parse(&path(&bad)), None, "{bad}");
        }
    }

    #[test]
    fn tags_are_plain_names() {
        let path = |tag: &str| format!("/v2/homebrew/core/openssl/3/manifests/{tag}");
        assert!(matches!(
            parse(&path("3.4.0-1")),
            Some(Item::Manifest { .. })
        ));
        assert!(matches!(
            parse(&path("Ab_c.9")),
            Some(Item::Manifest { .. })
        ));
        for bad in ["", ".", "..", "a:b", "a%2fb", "a\\b", &"t".repeat(129)] {
            assert_eq!(parse(&path(bad)), None, "{bad}");
        }
        assert_eq!(parse("/v2/homebrew/core/a/b/c/manifests/1"), None);
    }

    #[test]
    fn nothing_else_is_an_item() {
        for bad in [
            "/",
            "/https:/example.org/x",
            "/v2/homebrew/other/jq/manifests/1",
            "/v2/homebrew/core/jq/tags/list",
            "/v2/homebrew/core/jq/manifests",
            "/v2/homebrew/core/jq/manifests/1/",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }
}
