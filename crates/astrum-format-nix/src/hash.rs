use base64::Engine;
use base64::engine::general_purpose::STANDARD;

const ALPHABET: &[u8; 32] = b"0123456789abcdfghijklmnpqrsvwxyz";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake3,
}

impl Algorithm {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            "blake3" => Some(Self::Blake3),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
            Self::Blake3 => "blake3",
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::Md5 => 16,
            Self::Sha1 => 20,
            Self::Sha256 | Self::Blake3 => 32,
            Self::Sha512 => 64,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash {
    pub algorithm: Algorithm,
    pub digest: Vec<u8>,
}

impl Hash {
    pub fn parse(text: &str) -> Result<Self, String> {
        let invalid = || format!("{text} is not a hash such as sha256:<nix32 digest>");
        let (algorithm, encoded, sri) = match (text.split_once(':'), text.split_once('-')) {
            (Some((name, encoded)), _) if Algorithm::parse(name).is_some() => {
                (Algorithm::parse(name), encoded, false)
            }
            (_, Some((name, encoded))) => (Algorithm::parse(name), encoded, true),
            _ => (None, "", false),
        };
        let algorithm = algorithm.ok_or_else(invalid)?;
        let size = algorithm.size();
        let digest = match sri {
            true => STANDARD.decode(encoded).ok(),
            false if encoded.len() == size * 2 => hex::decode(encoded).ok(),
            false if encoded.len() == nix32_length(size) => from_nix32(encoded, size),
            false => STANDARD.decode(encoded).ok(),
        }
        .filter(|digest| digest.len() == size)
        .ok_or_else(invalid)?;
        Ok(Self { algorithm, digest })
    }

    pub fn hex(&self) -> String {
        hex::encode(&self.digest)
    }

    pub fn nix32(&self) -> String {
        format!("{}:{}", self.algorithm.name(), nix32(&self.digest))
    }

    pub fn is_sha256(&self) -> bool {
        self.algorithm == Algorithm::Sha256
    }
}

pub fn nix32_length(size: usize) -> usize {
    (size * 8).div_ceil(5)
}

pub fn nix32(bytes: &[u8]) -> String {
    (0..nix32_length(bytes.len()))
        .rev()
        .map(|position| {
            let digit = (0..5)
                .map(|offset| position * 5 + offset)
                .enumerate()
                .filter(|(_, bit)| bit / 8 < bytes.len() && (bytes[bit / 8] >> (bit % 8)) & 1 == 1)
                .fold(0, |digit, (offset, _)| digit | (1 << offset));
            ALPHABET[digit] as char
        })
        .collect()
}

pub fn from_nix32(text: &str, size: usize) -> Option<Vec<u8>> {
    if text.len() != nix32_length(size) {
        return None;
    }
    let mut bytes = vec![0u8; size];
    for (index, character) in text.bytes().enumerate() {
        let digit = ALPHABET.iter().position(|known| *known == character)?;
        let position = text.len() - 1 - index;
        for offset in (0..5).filter(|offset| (digit >> offset) & 1 == 1) {
            let bit = position * 5 + offset;
            *bytes.get_mut(bit / 8)? |= 1 << (bit % 8);
        }
    }
    Some(bytes)
}

pub fn is_nix32(text: &str) -> bool {
    text.bytes().all(|character| ALPHABET.contains(&character))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn nix32_matches_the_digests_of_a_public_cache() {
        let empty = hex::decode(EMPTY_SHA256).unwrap();
        assert_eq!(
            nix32(&empty),
            "0mdqa9w1p6cmli6976v4wi0sw9r4p5prkj7lzfd1877wk11c9c73"
        );
        let compressed =
            hex::decode("e29cbc93d3c0ddfd3522f7b2ffcc650724cb47fd9a16b755a74eec0902ad99f8")
                .unwrap();
        assert_eq!(
            nix32(&compressed),
            "1y4rml10kv2flxavf5lszm3wn907cp6gzcpp48szvpf0sf9vr772"
        );
        assert_eq!(
            from_nix32("1y4rml10kv2flxavf5lszm3wn907cp6gzcpp48szvpf0sf9vr772", 32),
            Some(compressed)
        );
        assert_eq!(nix32(&[0; 20]).len(), 32);
    }

    #[test]
    fn nix32_refuses_foreign_letters_and_overflow() {
        assert_eq!(
            from_nix32("0mdqa9w1p6cmli6976v4wi0sw9r4p5prkj7lzfd1877wk11c9c7e", 32),
            None
        );
        assert_eq!(from_nix32("zz", 1), None);
        assert_eq!(from_nix32("7z", 1), Some(vec![255]));
        assert!(is_nix32("0123456789abcdfghijklmnpqrsvwxyz"));
        assert!(!is_nix32("e"));
    }

    #[test]
    fn hashes_are_read_in_every_encoding() {
        let sri = Hash::parse("sha256-FePFYIlMuycIXPZbWi7LGEiMmZSX9FMbaQenWBzm1Sc=").unwrap();
        let nix =
            Hash::parse("sha256:09ymwqf5i9q7d4dm7x4pjjcqqj0qrcp5lnznbh42gfsci5hcbqqm").unwrap();
        assert_eq!(sri, nix);
        assert_eq!(
            nix.nix32(),
            "sha256:09ymwqf5i9q7d4dm7x4pjjcqqj0qrcp5lnznbh42gfsci5hcbqqm"
        );
        let base16 = Hash::parse(&format!("sha256:{}", sri.hex())).unwrap();
        assert_eq!(base16, sri);
        assert!(base16.is_sha256());
        assert_eq!(
            Hash::parse("sha1:0000000000000000000000000000000000000000")
                .unwrap()
                .algorithm,
            Algorithm::Sha1
        );
        for bad in [
            "sha256:abc",
            "sha256",
            "md4:00",
            "sha256-AAAA",
            "sha256:09ymwqf5i9q7d4dm7x4pjjcqqj0qrcp5lnznbh42gfsci5hcbqqe",
        ] {
            assert!(Hash::parse(bad).is_err(), "{bad}");
        }
    }
}
