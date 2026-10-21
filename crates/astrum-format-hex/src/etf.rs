use serde_json::{Map, Number, Value};

pub const CONTENT_TYPE: &str = "application/vnd.hex+erlang";

const VERSION: u8 = 131;
const NEW_FLOAT: u8 = 70;
const SMALL_INTEGER: u8 = 97;
const INTEGER: u8 = 98;
const ATOM: u8 = 100;
const SMALL_TUPLE: u8 = 104;
const LARGE_TUPLE: u8 = 105;
const NIL: u8 = 106;
const STRING: u8 = 107;
const LIST: u8 = 108;
const BINARY: u8 = 109;
const SMALL_BIG: u8 = 110;
const SMALL_ATOM: u8 = 115;
const MAP: u8 = 116;
const ATOM_UTF8: u8 = 118;
const SMALL_ATOM_UTF8: u8 = 119;
const MAX_DEPTH: usize = 32;
const MAX_ITEMS: usize = 100_000;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("the body is not an Erlang term the API understands: {0}")]
pub struct EtfError(String);

pub fn encode(value: &Value) -> Vec<u8> {
    let mut out = vec![VERSION];
    write(&mut out, value);
    out
}

fn write(out: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Null => atom(out, "nil"),
        Value::Bool(true) => atom(out, "true"),
        Value::Bool(false) => atom(out, "false"),
        Value::Number(number) => match (number.as_i64(), number.as_u64(), number.as_f64()) {
            (Some(value), _, _) => integer(out, value as i128),
            (None, Some(value), _) => integer(out, value as i128),
            (_, _, Some(value)) => {
                out.push(NEW_FLOAT);
                out.extend_from_slice(&value.to_be_bytes());
            }
            _ => atom(out, "nil"),
        },
        Value::String(text) => {
            out.push(BINARY);
            out.extend_from_slice(&(text.len() as u32).to_be_bytes());
            out.extend_from_slice(text.as_bytes());
        }
        Value::Array(items) => {
            if items.is_empty() {
                out.push(NIL);
                return;
            }
            out.push(LIST);
            out.extend_from_slice(&(items.len() as u32).to_be_bytes());
            for item in items {
                write(out, item);
            }
            out.push(NIL);
        }
        Value::Object(map) => {
            let present: Vec<(&String, &Value)> =
                map.iter().filter(|(_, item)| !item.is_null()).collect();
            out.push(MAP);
            out.extend_from_slice(&(present.len() as u32).to_be_bytes());
            for (key, item) in present {
                write(out, &Value::String(key.clone()));
                write(out, item);
            }
        }
    }
}

fn atom(out: &mut Vec<u8>, name: &str) {
    out.push(SMALL_ATOM_UTF8);
    out.push(name.len() as u8);
    out.extend_from_slice(name.as_bytes());
}

fn integer(out: &mut Vec<u8>, value: i128) {
    if (0..=255).contains(&value) {
        out.push(SMALL_INTEGER);
        out.push(value as u8);
    } else if let Ok(small) = i32::try_from(value) {
        out.push(INTEGER);
        out.extend_from_slice(&small.to_be_bytes());
    } else {
        let mut magnitude = value.unsigned_abs();
        let mut digits = Vec::new();
        while magnitude > 0 {
            digits.push((magnitude & 0xff) as u8);
            magnitude >>= 8;
        }
        out.push(SMALL_BIG);
        out.push(digits.len() as u8);
        out.push(u8::from(value < 0));
        out.extend_from_slice(&digits);
    }
}

pub fn decode(bytes: &[u8]) -> Result<Value, EtfError> {
    let mut reader = Reader { bytes, at: 0 };
    if reader.byte()? != VERSION {
        return Err(EtfError(
            "it does not start with the version byte".to_owned(),
        ));
    }
    let value = reader.value(0)?;
    if reader.at != bytes.len() {
        return Err(EtfError("bytes follow the term".to_owned()));
    }
    Ok(value)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, count: usize) -> Result<&[u8], EtfError> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| EtfError("it ends too early".to_owned()))?;
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    fn byte(&mut self) -> Result<u8, EtfError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<usize, EtfError> {
        let taken = self.take(2)?;
        Ok(u16::from_be_bytes([taken[0], taken[1]]) as usize)
    }

    fn u32(&mut self) -> Result<usize, EtfError> {
        let taken = self.take(4)?;
        Ok(u32::from_be_bytes([taken[0], taken[1], taken[2], taken[3]]) as usize)
    }

    fn count(&mut self, length: usize) -> Result<usize, EtfError> {
        match length <= MAX_ITEMS {
            true => Ok(length),
            false => Err(EtfError("it holds too many items".to_owned())),
        }
    }

    fn text(&mut self, length: usize) -> Result<String, EtfError> {
        String::from_utf8(self.take(length)?.to_vec())
            .map_err(|_| EtfError("a text is not UTF-8".to_owned()))
    }

    fn atom(name: String) -> Value {
        match name.as_str() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            "nil" | "undefined" => Value::Null,
            _ => Value::String(name),
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, EtfError> {
        if depth > MAX_DEPTH {
            return Err(EtfError("it is nested too deeply".to_owned()));
        }
        let tag = self.byte()?;
        Ok(match tag {
            SMALL_INTEGER => Value::Number(Number::from(self.byte()?)),
            INTEGER => {
                let taken = self.take(4)?;
                Value::Number(Number::from(i32::from_be_bytes([
                    taken[0], taken[1], taken[2], taken[3],
                ])))
            }
            NEW_FLOAT => {
                let taken = self.take(8)?;
                let mut raw = [0; 8];
                raw.copy_from_slice(taken);
                Number::from_f64(f64::from_be_bytes(raw))
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            SMALL_BIG => {
                let length = self.byte()? as usize;
                let negative = self.byte()? != 0;
                let digits = self.take(length)?;
                if length > 8 {
                    return Err(EtfError("an integer is too large".to_owned()));
                }
                let magnitude = digits
                    .iter()
                    .rev()
                    .fold(0u64, |total, digit| (total << 8) | u64::from(*digit));
                match negative {
                    true => i64::try_from(magnitude)
                        .map(|value| Value::Number(Number::from(-value)))
                        .map_err(|_| EtfError("an integer is too large".to_owned()))?,
                    false => Value::Number(Number::from(magnitude)),
                }
            }
            ATOM | ATOM_UTF8 => {
                let length = self.u16()?;
                Self::atom(self.text(length)?)
            }
            SMALL_ATOM | SMALL_ATOM_UTF8 => {
                let length = self.byte()? as usize;
                Self::atom(self.text(length)?)
            }
            BINARY => {
                let length = self.u32()?;
                Value::String(self.text(length)?)
            }
            STRING => {
                let length = self.u16()?;
                Value::String(
                    self.take(length)?
                        .iter()
                        .map(|byte| char::from(*byte))
                        .collect(),
                )
            }
            NIL => Value::Array(Vec::new()),
            LIST => {
                let length = self.u32()?;
                let length = self.count(length)?;
                let mut items = Vec::with_capacity(length.min(1024));
                for _ in 0..length {
                    items.push(self.value(depth + 1)?);
                }
                match self.value(depth + 1)? {
                    Value::Array(tail) if tail.is_empty() => {}
                    _ => return Err(EtfError("an improper list is not supported".to_owned())),
                }
                Value::Array(items)
            }
            SMALL_TUPLE | LARGE_TUPLE => {
                let length = match tag {
                    SMALL_TUPLE => self.byte()? as usize,
                    _ => self.u32()?,
                };
                let length = self.count(length)?;
                let mut items = Vec::with_capacity(length.min(1024));
                for _ in 0..length {
                    items.push(self.value(depth + 1)?);
                }
                Value::Array(items)
            }
            MAP => {
                let length = self.u32()?;
                let length = self.count(length)?;
                let mut map = Map::new();
                for _ in 0..length {
                    let key = match self.value(depth + 1)? {
                        Value::String(key) => key,
                        Value::Number(number) => number.to_string(),
                        Value::Bool(flag) => flag.to_string(),
                        _ => return Err(EtfError("a map key is not a text".to_owned())),
                    };
                    let value = self.value(depth + 1)?;
                    map.insert(key, value);
                }
                Value::Object(map)
            }
            other => return Err(EtfError(format!("the term type {other} is not supported"))),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn values_encode_like_term_to_binary() {
        assert_eq!(
            encode(&json!({"a": "b"})),
            [
                131, 116, 0, 0, 0, 1, 109, 0, 0, 0, 1, b'a', 109, 0, 0, 0, 1, b'b'
            ]
        );
        assert_eq!(encode(&json!(7)), [131, 97, 7]);
        assert_eq!(encode(&json!(-1)), [131, 98, 255, 255, 255, 255]);
        assert_eq!(
            encode(&json!(4_294_967_296_i64)),
            [131, 110, 5, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(encode(&json!([])), [131, 106]);
        assert_eq!(
            encode(&json!({"a": null, "b": 1})),
            [131, 116, 0, 0, 0, 1, 109, 0, 0, 0, 1, b'b', 97, 1]
        );
        assert_eq!(
            encode(&json!([true, null])),
            [
                131, 108, 0, 0, 0, 2, 119, 4, b't', b'r', b'u', b'e', 119, 3, b'n', b'i', b'l', 106
            ]
        );
    }

    #[test]
    fn decoded_terms_round_trip() {
        let value = json!({
            "reason": "security",
            "message": "use 2.0",
            "count": 300,
            "big": 4_294_967_296_i64,
            "negative": -5_000_000_000_i64,
            "ratio": 0.5,
            "flags": [true, false],
            "nested": {"empty": []},
        });
        assert_eq!(decode(&encode(&value)).unwrap(), value);
        assert_eq!(
            decode(&[131, 104, 2, 100, 0, 2, b'o', b'k', 107, 0, 2, b'h', b'i']).unwrap(),
            json!(["ok", "hi"])
        );
        assert_eq!(
            decode(&[131, 115, 3, b'a', b'b', b'c']).unwrap(),
            json!("abc")
        );
        for broken in [
            &[][..],
            &[130, 97, 1],
            &[131, 109, 0, 0, 0, 9, b'a'],
            &[131, 97, 1, 0],
            &[131, 108, 0, 0, 0, 1, 97, 1, 97, 2],
            &[131, 116, 0, 0, 0, 1, 106, 97, 1],
            &[131, 200],
        ] {
            assert!(decode(broken).is_err(), "{broken:?}");
        }
    }
}
