use sha3::Shake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};

pub const LENGTH: usize = 64;

pub type Value = [u8; LENGTH];

pub fn shake256(bytes: &[u8]) -> Value {
    let mut hasher = Shake256::default();
    hasher.update(bytes);
    let mut value = [0u8; LENGTH];
    hasher.finalize_xof().read(&mut value);
    value
}

pub fn files_digest<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> String {
    let mut lines: Vec<(&str, String)> = files
        .into_iter()
        .map(|(path, content)| (path, hex::encode(shake256(content))))
        .collect();
    lines.sort_by(|one, other| one.0.as_bytes().cmp(other.0.as_bytes()));
    let mut manifest = String::new();
    for (path, digest) in lines {
        manifest.push_str("shake256:");
        manifest.push_str(&digest);
        manifest.push_str("  ");
        manifest.push_str(path);
        manifest.push('\n');
    }
    format!("shake256:{}", hex::encode(shake256(manifest.as_bytes())))
}

pub fn label(value: &[u8]) -> String {
    format!("b5:{}", hex::encode(value))
}

pub fn module_digest<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a [u8])>,
    dependencies: &[Value],
) -> Value {
    let mut dependency_labels: Vec<String> =
        dependencies.iter().map(|value| label(value)).collect();
    dependency_labels.sort();
    dependency_labels.dedup();
    let mut lines = vec![files_digest(files)];
    lines.extend(dependency_labels);
    shake256(lines.join("\n").as_bytes())
}

pub fn parse(hex_value: &str) -> Option<Value> {
    hex::decode(hex_value).ok()?.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROTO: &[u8] = b"syntax = \"proto3\";\n\npackage acme.v1;\n";

    fn files() -> Vec<(&'static str, &'static [u8])> {
        vec![
            ("buf.md", b"# Greeter\n"),
            ("acme/v1/greeter.proto", PROTO),
            ("LICENSE", b"MIT\n"),
        ]
    }

    #[test]
    fn shake256_gives_sixty_four_bytes_of_the_standard_function() {
        assert_eq!(
            hex::encode(shake256(b"")),
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762fd75dc4ddd8c0f200cb05019d67b592f6fc821c49479ab48640292eacb3b7c4be"
        );
    }

    #[test]
    fn the_files_digest_covers_a_sorted_manifest() {
        assert_eq!(
            hex::encode(shake256(b"dep-a")),
            "59102da4beedf40b04f8d71b0e33ccf2ef6196bf54de01c42ee60ff6dd46f311f36e3a23be32a03ca18bd5fae6a7da78a85dd0b019d490b010e8bda688c4886e"
        );
        assert_eq!(
            files_digest(files()),
            "shake256:b9c541af22cb3c96634397e37551c8a182bbf08218320946a11ca43ed775cdd61eaa425f5126723ba2c76b613fb9d46c4d451b104e038499931487819e1a23a5"
        );
        let mut reversed = files();
        reversed.reverse();
        assert_eq!(files_digest(reversed), files_digest(files()));
    }

    #[test]
    fn without_dependencies_the_module_digest_hashes_the_files_digest() {
        assert_eq!(
            hex::encode(module_digest(files(), &[])),
            "ff9f74bc4262464040c528a3fd2bd5735664e294b773941fcb06062696b70fab2d6fac2ff60a462b54eb6c1d980202c9d786a9722567e3ce17428bca6516fd24"
        );
    }

    #[test]
    fn dependency_digests_are_sorted_as_strings_and_joined_with_newlines() {
        let first = shake256(b"dep-a");
        let second = shake256(b"dep-b");
        let expected = "96e4162b9e8529e7ec09577e59d9bd41590e3417da9666addbfa3d1b15f61dfc63ada696ebfe9831465b74d1d336a1d3b09e15fae4c5ead2e6a941fc749617d5";
        assert_eq!(
            hex::encode(module_digest(files(), &[first, second])),
            expected
        );
        assert_eq!(
            hex::encode(module_digest(files(), &[second, first, first])),
            expected
        );
    }

    #[test]
    fn a_digest_round_trips_through_its_hex_form() {
        let value = shake256(b"x");
        assert_eq!(parse(&hex::encode(value)), Some(value));
        assert_eq!(parse("abcd"), None);
        assert_eq!(parse("zz"), None);
        assert_eq!(label(&value), format!("b5:{}", hex::encode(value)));
    }
}
