use std::io::Write;

const SYMSXP: u32 = 1;
const LISTSXP: u32 = 2;
const CHARSXP: u32 = 9;
const INTSXP: u32 = 13;
const STRSXP: u32 = 16;
const VECSXP: u32 = 19;
const NILVALUE: u32 = 254;
const HAS_ATTRIBUTES: u32 = 1 << 9;
const HAS_TAG: u32 = 1 << 10;
const UTF8: u32 = 1 << 3;
const ASCII: u32 = 1 << 6;
const WRITER: i32 = 0x0003_0500;
const READER: i32 = 0x0003_0500;
const ENCODING: &str = "UTF-8";

struct Out(Vec<u8>);

impl Out {
    fn int(&mut self, value: i32) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    fn flags(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_be_bytes());
    }

    fn length(&mut self, value: usize) {
        self.int(i32::try_from(value).expect("an R vector is shorter than 2^31"));
    }

    fn chars(&mut self, value: Option<&str>) {
        match value {
            None => {
                self.flags(CHARSXP);
                self.int(-1);
            }
            Some(text) => {
                let levels = if text.is_ascii() { ASCII } else { UTF8 };
                self.flags(CHARSXP | (levels << 12));
                self.length(text.len());
                self.0.extend_from_slice(text.as_bytes());
            }
        }
    }

    fn strings(&mut self, values: &[Option<&str>], attributes: bool) {
        self.flags(STRSXP | if attributes { HAS_ATTRIBUTES } else { 0 });
        self.length(values.len());
        for value in values {
            self.chars(*value);
        }
    }

    fn tagged(&mut self, name: &str) {
        self.flags(LISTSXP | HAS_TAG);
        self.flags(SYMSXP);
        self.chars(Some(name));
    }
}

pub struct Matrix<'a> {
    pub rows: usize,
    pub cells: Vec<Option<&'a str>>,
    pub row_names: Option<Vec<&'a str>>,
    pub column_names: Vec<&'a str>,
}

pub fn serialize(matrix: &Matrix<'_>) -> Vec<u8> {
    let mut out = Out(b"X\n".to_vec());
    out.int(3);
    out.int(WRITER);
    out.int(READER);
    out.length(ENCODING.len());
    out.0.extend_from_slice(ENCODING.as_bytes());
    out.strings(&matrix.cells, true);
    out.tagged("dim");
    out.flags(INTSXP);
    out.int(2);
    out.length(matrix.rows);
    out.length(matrix.column_names.len());
    out.tagged("dimnames");
    out.flags(VECSXP);
    out.int(2);
    match &matrix.row_names {
        Some(names) => {
            let names: Vec<Option<&str>> = names.iter().map(|name| Some(*name)).collect();
            out.strings(&names, false);
        }
        None => out.flags(NILVALUE),
    }
    let columns: Vec<Option<&str>> = matrix.column_names.iter().map(|name| Some(*name)).collect();
    out.strings(&columns, false);
    out.flags(NILVALUE);
    out.0
}

pub fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(bytes)
        .expect("writing into memory does not fail");
    encoder.finish().expect("writing into memory does not fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    const R_MATRIX: &str = "580a000000030004030300030500000000055554462d380000021000000004000400090000000261620004000900000002636400000009ffffffff0000800900000002c3bc0000040200000001000400090000000364696d0000000d0000000200000002000000020000040200000001000400090000000864696d6e616d6573000000130000000200000010000000020004000900000002616200040009000000026364000000100000000200040009000000075061636b6167650004000900000007446570656e6473000000fe";
    const R_EMPTY: &str = "580a000000030004030300030500000000055554462d3800000210000000000000040200000001000400090000000364696d0000000d0000000200000000000000020000040200000001000400090000000864696d6e616d65730000001300000002000000fe000000100000000200040009000000075061636b616765000400090000000756657273696f6e000000fe";

    fn without_writer(hex: &str) -> String {
        format!("{}{}", &hex[..12], &hex[20..])
    }

    #[test]
    fn a_character_matrix_is_written_as_r_writes_it() {
        let matrix = Matrix {
            rows: 2,
            cells: vec![Some("ab"), Some("cd"), None, Some("\u{fc}")],
            row_names: Some(vec!["ab", "cd"]),
            column_names: vec!["Package", "Depends"],
        };
        assert_eq!(
            without_writer(&hex(&serialize(&matrix))),
            without_writer(R_MATRIX)
        );
        let empty = Matrix {
            rows: 0,
            cells: vec![],
            row_names: None,
            column_names: vec!["Package", "Version"],
        };
        assert_eq!(
            without_writer(&hex(&serialize(&empty))),
            without_writer(R_EMPTY)
        );
    }

    #[test]
    fn the_file_is_gzip_compressed() {
        let packed = gzip(b"X\n");
        assert_eq!(&packed[..2], &[0x1f, 0x8b]);
    }
}
