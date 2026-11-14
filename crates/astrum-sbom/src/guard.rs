use crate::error::Error;
use crate::{MAX_DEPTH, MAX_INPUT_BYTES};

pub(crate) fn input(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(Error::TooLarge);
    }
    depth(bytes)
}

pub(crate) fn depth(bytes: &[u8]) -> Result<(), Error> {
    let mut open = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for &byte in bytes {
        if in_string {
            match (escaped, byte) {
                (true, _) => escaped = false,
                (false, b'\\') => escaped = true,
                (false, b'"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' | b'[' => {
                open += 1;
                if open > MAX_DEPTH {
                    return Err(Error::TooDeep);
                }
            }
            b'}' | b']' => open = open.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn object(bytes: &[u8]) -> Result<(), Error> {
    match bytes.iter().find(|byte| !byte.is_ascii_whitespace()) {
        Some(b'{') => Ok(()),
        Some(b'<') => Err(Error::unsupported("XML documents")),
        _ => Err(Error::malformed("the document is not a JSON object")),
    }
}

pub(crate) fn without_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes)
}
