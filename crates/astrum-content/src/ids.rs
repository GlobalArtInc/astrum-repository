use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

pub fn encode(repository: &str, id: i64) -> String {
    URL_SAFE_NO_PAD.encode(format!("{repository}:{id:x}"))
}

pub fn decode(value: &str) -> Option<(String, i64)> {
    let bytes = URL_SAFE_NO_PAD.decode(value.trim_end_matches('=')).ok()?;
    let text = String::from_utf8(bytes).ok()?;
    let (repository, id) = text.rsplit_once(':')?;
    let id = i64::from_str_radix(id, 16).ok()?;
    (!repository.is_empty() && id > 0).then(|| (repository.to_owned(), id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_reject_garbage() {
        let id = encode("maven-releases", 42);
        assert_eq!(decode(&id), Some(("maven-releases".to_owned(), 42)));
        assert_eq!(decode("not base64!"), None);
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode("nocolon")), None);
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode("repo:zz")), None);
        assert_eq!(decode(&URL_SAFE_NO_PAD.encode(":1")), None);
    }
}
