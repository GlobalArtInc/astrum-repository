const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

fn strip_comments(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    let mut in_string = false;
    while index < input.len() {
        let byte = input[index];
        if in_string {
            output.push(byte);
            match byte {
                b'\\' => {
                    if let Some(&next) = input.get(index + 1) {
                        output.push(next);
                        index += 1;
                    }
                }
                b'"' => in_string = false,
                _ => {}
            }
            index += 1;
            continue;
        }
        match (byte, input.get(index + 1)) {
            (b'"', _) => {
                in_string = true;
                output.push(byte);
                index += 1;
            }
            (b'/', Some(b'/')) => {
                while index < input.len() && input[index] != b'\n' && input[index] != b'\r' {
                    index += 1;
                }
                output.push(b' ');
            }
            (b'/', Some(b'*')) => {
                let rest = &input[index + 2..];
                let Some(end) = rest.windows(2).position(|pair| pair == b"*/") else {
                    return Err("a /* comment is never closed".to_owned());
                };
                index += 2 + end + 2;
                output.push(b' ');
            }
            _ => {
                output.push(byte);
                index += 1;
            }
        }
    }
    Ok(output)
}

fn strip_trailing_commas(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut in_string = false;
    let mut index = 0;
    while index < input.len() {
        let byte = input[index];
        if in_string {
            output.push(byte);
            match byte {
                b'\\' => {
                    if let Some(&next) = input.get(index + 1) {
                        output.push(next);
                        index += 1;
                    }
                }
                b'"' => in_string = false,
                _ => {}
            }
        } else if byte == b'"' {
            in_string = true;
            output.push(byte);
        } else if byte == b',' {
            let next = input[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_ascii_whitespace());
            let previous = output.iter().rev().find(|seen| !seen.is_ascii_whitespace());
            let trailing = matches!(next, Some(b'}' | b']'))
                && !matches!(previous, None | Some(b'[' | b'{' | b','));
            if !trailing {
                output.push(byte);
            }
        } else {
            output.push(byte);
        }
        index += 1;
    }
    output
}

pub fn to_json(input: &[u8]) -> Result<Vec<u8>, String> {
    let input = input.strip_prefix(&BOM).unwrap_or(input);
    Ok(strip_trailing_commas(&strip_comments(input)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parse(text: &str) -> Result<Value, String> {
        let plain = to_json(text.as_bytes())?;
        serde_json::from_slice(&plain).map_err(|error| error.to_string())
    }

    #[test]
    fn comments_outside_strings_go_and_inside_strings_stay() {
        let text = "{\n  // the name\n  \"name\": \"@a/b\", /* inline */ \"url\": \"http://x/*y*/ // z\",\n  \"esc\": \"q\\\"//\\\\\" // end\n}";
        assert_eq!(
            parse(text).unwrap(),
            json!({"name": "@a/b", "url": "http://x/*y*/ // z", "esc": "q\"//\\"})
        );
    }

    #[test]
    fn a_lone_carriage_return_ends_a_line_comment() {
        assert_eq!(
            parse("{\"a\": 1 // note\r, \"b\": 2}").unwrap(),
            json!({"a": 1, "b": 2})
        );
        assert_eq!(
            parse("{\"a\": 1, // note\r\n \"b\": 2}").unwrap(),
            json!({"a": 1, "b": 2})
        );
    }

    #[test]
    fn trailing_commas_before_a_closing_bracket_go() {
        assert_eq!(
            parse("{\"a\": [1, 2,], \"b\": {\"c\": 1, /* x */ },\n}").unwrap(),
            json!({"a": [1, 2], "b": {"c": 1}})
        );
        assert_eq!(parse("{\"a\": \"x,]\", }").unwrap(), json!({"a": "x,]"}));
        assert!(parse("{\"a\": 1,, }").is_err());
        assert!(parse("[,]").is_err());
    }

    #[test]
    fn a_byte_order_mark_is_dropped() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"{\"a\": 1}");
        let plain = to_json(&bytes).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&plain).unwrap(),
            json!({"a": 1})
        );
    }

    #[test]
    fn an_unclosed_block_comment_is_an_error_and_nesting_is_not_supported() {
        assert!(
            to_json(b"{\"a\": 1 /* open")
                .unwrap_err()
                .contains("never closed")
        );
        assert!(parse("{\"a\": 1 /* a /* b */ */ }").is_err());
        assert!(parse("{\"a\": 1} // no newline at the end").is_ok());
    }

    #[test]
    fn plain_json_passes_through_and_the_depth_limit_stays() {
        assert_eq!(
            parse("{\"a\": [1, {\"b\": null}]}").unwrap(),
            json!({"a": [1, {"b": null}]})
        );
        let deep = format!("{}{}", "[".repeat(200), "]".repeat(200));
        assert!(parse(&deep).is_err());
    }
}
