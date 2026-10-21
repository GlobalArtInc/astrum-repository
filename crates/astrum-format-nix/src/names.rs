use crate::hash;

pub const HASH_LENGTH: usize = 32;
const MAX_NAME: usize = 211;

pub fn valid_hash(text: &str) -> bool {
    text.len() == HASH_LENGTH && hash::is_nix32(text)
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+-._?=".contains(c))
}

pub fn split_base(base: &str) -> Option<(&str, &str)> {
    let (hash, name) = base.split_at_checked(HASH_LENGTH)?;
    let name = name.strip_prefix('-')?;
    (valid_hash(hash) && valid_name(name)).then_some((hash, name))
}

pub fn valid_base(base: &str) -> bool {
    split_base(base).is_some()
}

pub fn package(name: &str) -> (&str, &str) {
    let bytes = name.as_bytes();
    (0..bytes.len().saturating_sub(1))
        .find(|&index| bytes[index] == b'-' && !bytes[index + 1].is_ascii_alphabetic())
        .map(|index| (&name[..index], &name[index + 1..]))
        .unwrap_or((name, ""))
}

pub fn valid_nar_file(file: &str) -> bool {
    let Some((stem, rest)) = file.split_once(".nar") else {
        return false;
    };
    let extension_ok = match rest {
        "" => true,
        other => other.strip_prefix('.').is_some_and(|extension| {
            !extension.is_empty()
                && extension.len() <= 8
                && extension.bytes().all(|c| c.is_ascii_alphanumeric())
        }),
    };
    !stem.is_empty() && stem.len() <= 128 && hash::is_nix32(stem) && extension_ok
}

pub fn narinfo_path(hash: &str) -> String {
    format!("/{hash}.narinfo")
}

pub fn listing_path(hash: &str) -> String {
    format!("/{hash}.ls")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_path_names_follow_the_store_rules() {
        assert!(valid_name("hello-2.12.1"));
        assert!(valid_name("python3.12-requests-2.32.3?x=y+z_"));
        assert!(!valid_name(".hidden"));
        assert!(!valid_name("with space"));
        assert!(!valid_name(""));
        assert!(!valid_name(&"a".repeat(212)));
        assert_eq!(
            split_base("mw9ls4zxx884y7yq0a04ildfdjfy1vn2-nix-2.34.9"),
            Some(("mw9ls4zxx884y7yq0a04ildfdjfy1vn2", "nix-2.34.9"))
        );
        assert_eq!(split_base("mw9ls4zxx884y7yq0a04ildfdjfy1vn2"), None);
        assert_eq!(split_base("ew9ls4zxx884y7yq0a04ildfdjfy1vn2-nix"), None);
        assert_eq!(split_base("mw9ls4zxx884y7yq0a04ildfdjfy1vn2_nix"), None);
    }

    #[test]
    fn package_names_split_at_the_first_dash_before_a_non_letter() {
        assert_eq!(package("hello-2.12.1"), ("hello", "2.12.1"));
        assert_eq!(
            package("bash-interactive-5.2p37-man"),
            ("bash-interactive", "5.2p37-man")
        );
        assert_eq!(package("source"), ("source", ""));
        assert_eq!(package("nix-2.34.9.drv"), ("nix", "2.34.9.drv"));
        assert_eq!(package("trailing-"), ("trailing-", ""));
    }

    #[test]
    fn nar_files_are_a_digest_and_a_compression_suffix() {
        for good in [
            "0xfhhhb46a15v23pvkw97a5h578d1qkl34w351p5xxh52f1jbhyl.nar.zst",
            "1w1fff338fvdw53sqgamddn1b2xgds473pv6y13gizdbqjv4i5p3.nar.xz",
            "1w1fff338fvdw53sqgamddn1b2xgds473pv6y13gizdbqjv4i5p3.nar",
        ] {
            assert!(valid_nar_file(good), "{good}");
        }
        for bad in [
            ".nar.xz",
            "x.tar.xz",
            "1w1fff338f.nar.",
            "1w1fff338f.nar.x-z",
            "1w1fff338e.nar",
            "1w1fff338f.narxz",
        ] {
            assert!(!valid_nar_file(bad), "{bad}");
        }
    }
}
