use std::collections::HashMap;

use crate::gem::{ANY, split_requirement};

#[derive(Default)]
pub struct Writer {
    pub out: Vec<u8>,
    symbols: HashMap<String, usize>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            out: vec![4, 8],
            symbols: HashMap::new(),
        }
    }

    pub fn nil(&mut self) -> &mut Self {
        self.out.push(b'0');
        self
    }

    pub fn bool(&mut self, value: bool) -> &mut Self {
        self.out.push(if value { b'T' } else { b'F' });
        self
    }

    pub fn integer(&mut self, value: i64) -> &mut Self {
        self.out.push(b'i');
        self.long(value);
        self
    }

    pub fn string(&mut self, value: &str) -> &mut Self {
        self.out.extend_from_slice(b"I\"");
        self.bytes(value.as_bytes());
        self.long(1);
        self.symbol("E");
        self.out.push(b'T');
        self
    }

    pub fn symbol(&mut self, name: &str) -> &mut Self {
        match self.symbols.get(name) {
            Some(index) => {
                self.out.push(b';');
                self.long(*index as i64);
            }
            None => {
                let index = self.symbols.len();
                self.symbols.insert(name.to_owned(), index);
                self.out.push(b':');
                self.bytes(name.as_bytes());
            }
        }
        self
    }

    pub fn array(&mut self, size: usize) -> &mut Self {
        self.out.push(b'[');
        self.long(size as i64);
        self
    }

    pub fn hash(&mut self, size: usize) -> &mut Self {
        self.out.push(b'{');
        self.long(size as i64);
        self
    }

    pub fn strings(&mut self, values: &[String]) -> &mut Self {
        self.array(values.len());
        for value in values {
            self.string(value);
        }
        self
    }

    pub fn user_marshal(&mut self, class: &str) -> &mut Self {
        self.out.push(b'U');
        self.symbol(class)
    }

    pub fn user_defined(&mut self, class: &str, data: &[u8]) -> &mut Self {
        self.out.push(b'u');
        self.symbol(class);
        self.bytes(data);
        self
    }

    pub fn object(&mut self, class: &str, variables: usize) -> &mut Self {
        self.out.push(b'o');
        self.symbol(class);
        self.long(variables as i64);
        self
    }

    pub fn gem_version(&mut self, version: &str) -> &mut Self {
        self.user_marshal("Gem::Version");
        self.array(1);
        self.string(version)
    }

    pub fn gem_requirement(&mut self, requirements: &[String]) -> &mut Self {
        let any = [ANY.to_owned()];
        let effective = if requirements.is_empty() {
            &any[..]
        } else {
            requirements
        };
        self.user_marshal("Gem::Requirement");
        self.array(1);
        self.array(effective.len());
        for requirement in effective {
            let (operator, version) = split_requirement(requirement)
                .unwrap_or_else(|_| (">=".to_owned(), "0".to_owned()));
            self.array(2);
            self.string(&operator);
            self.gem_version(&version);
        }
        self
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.long(bytes.len() as i64);
        self.out.extend_from_slice(bytes);
    }

    fn long(&mut self, value: i64) {
        if value == 0 {
            self.out.push(0);
            return;
        }
        if value > 0 && value < 123 {
            self.out.push((value + 5) as u8);
            return;
        }
        if value > -124 && value < 0 {
            self.out.push(((value - 5) & 0xff) as u8);
            return;
        }
        let mut buffer = [0u8; 9];
        let mut x = value;
        let mut count = 0;
        for slot in buffer.iter_mut().skip(1) {
            *slot = (x & 0xff) as u8;
            x >>= 8;
            count += 1;
            if x == 0 || x == -1 {
                break;
            }
        }
        buffer[0] = if value < 0 {
            (-(count as i8)) as u8
        } else {
            count as u8
        };
        self.out.extend_from_slice(&buffer[..=count]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_written_in_marshal_4_8() {
        let mut writer = Writer::new();
        writer.array(2).string("a").integer(300);
        assert_eq!(writer.out, b"\x04\x08[\x07I\"\x06a\x06:\x06ETi\x02\x2c\x01");
        let mut writer = Writer::new();
        writer.array(2).symbol("x").symbol("x");
        assert_eq!(writer.out, b"\x04\x08[\x07:\x06x;\x00");
    }
}
