use base64::Engine;
use base64::alphabet;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};

use crate::error::Error;
use crate::text::{Text, Whole};

const PAYLOAD_TYPE: &str = "application/vnd.in-toto+json";

const CONFIG: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
const STANDARD: GeneralPurpose = GeneralPurpose::new(&alphabet::STANDARD, CONFIG);
const URL_SAFE: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, CONFIG);

pub(crate) fn open(payload_type: &Text, payload: &Whole) -> Result<Vec<u8>, Error> {
    if payload_type.0.as_deref() != Some(PAYLOAD_TYPE) {
        return Err(Error::unsupported("the envelope payload type"));
    }
    let Some(payload) = payload.0.as_deref() else {
        return Err(Error::malformed("the envelope has no payload"));
    };
    let compact: Vec<u8> = payload
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    STANDARD
        .decode(&compact)
        .or_else(|_| URL_SAFE.decode(&compact))
        .map_err(|_| Error::malformed("the envelope payload is not base64"))
}
