use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Digests {
    pub sha1: String,
    pub sha256: String,
    pub md5: String,
}

#[derive(Default)]
pub(crate) struct Hasher {
    sha1: Sha1,
    sha256: Sha256,
    md5: Md5,
    size: u64,
}

impl Hasher {
    pub(crate) fn update(&mut self, bytes: &[u8]) {
        self.sha1.update(bytes);
        self.sha256.update(bytes);
        self.md5.update(bytes);
        self.size += bytes.len() as u64;
    }

    pub(crate) fn finish(self) -> (u64, Digests) {
        (
            self.size,
            Digests {
                sha1: hex::encode(self.sha1.finalize()),
                sha256: hex::encode(self.sha256.finalize()),
                md5: hex::encode(self.md5.finalize()),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digests_match_the_known_values() {
        let mut hasher = Hasher::default();
        hasher.update(b"hello ");
        hasher.update(b"world");
        let (size, digests) = hasher.finish();
        assert_eq!(size, 11);
        assert_eq!(digests.sha1, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
        assert_eq!(
            digests.sha256,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert_eq!(digests.md5, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }
}
