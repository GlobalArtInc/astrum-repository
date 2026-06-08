use crate::chart::{Chart, ChartError};

pub const MAX_PROVENANCE: usize = 1024 * 1024;

const BEGIN: &str = "-----BEGIN PGP SIGNED MESSAGE-----";
const SIGNATURE: &str = "-----BEGIN PGP SIGNATURE-----";

pub fn signed_text(text: &str) -> Result<String, ChartError> {
    let invalid = |reason: &str| ChartError::Invalid(format!("the provenance file {reason}"));
    let mut lines = text.lines().skip_while(|line| line.trim_end() != BEGIN);
    if lines.next().is_none() {
        return Err(invalid("is not a PGP signed message"));
    }
    let mut lines = lines.skip_while(|line| !line.trim().is_empty());
    if lines.next().is_none() {
        return Err(invalid("has no signed text"));
    }
    let mut body = Vec::new();
    let mut signed = false;
    for line in lines {
        if line.trim_end() == SIGNATURE {
            signed = true;
            break;
        }
        body.push(line.strip_prefix("- ").unwrap_or(line));
    }
    if !signed {
        return Err(invalid("has no signature"));
    }
    Ok(body.join("\n"))
}

pub fn chart(text: &str) -> Result<Chart, ChartError> {
    let body = signed_text(text)?;
    let metadata: Vec<&str> = body.lines().take_while(|line| *line != "...").collect();
    Ok(Chart::parse(metadata.join("\n").as_bytes())?.metadata())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROV: &str = "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA512\n\napiVersion: v2\nappVersion: 1.16.0\ndescription: Demo chart\nkeywords:\n- - demo\nmaintainers:\n- - email: jane@example.org\n  name: Jane Doe\nname: demo\nversion: 0.4.0\n\n...\nfiles:\n  demo-0.4.0.tgz: sha256:6acae082348dc5d91145a0319e66cc3cd94999e30bb8504d5ec75fde4d376f8b\n-----BEGIN PGP SIGNATURE-----\n\nwsBcBAEBCgAQBQJqzTW/\n-----END PGP SIGNATURE-----\n";

    #[test]
    fn the_signed_chart_metadata_is_read() {
        let chart = chart(PROV).unwrap();
        assert_eq!(chart.name, "demo");
        assert_eq!(chart.version, "0.4.0");
        assert_eq!(chart.keywords, vec!["demo"]);
        assert_eq!(chart.maintainers[0].name.as_deref(), Some("Jane Doe"));
        assert!(
            signed_text(PROV)
                .unwrap()
                .contains("demo-0.4.0.tgz: sha256:")
        );
    }

    #[test]
    fn broken_files_are_refused() {
        assert!(chart("hello").is_err());
        assert!(chart(&PROV.replace(SIGNATURE, "nothing")).is_err());
        assert!(chart(&PROV.replace("name: demo\n", "")).is_err());
        assert!(chart(&PROV.replace("\r\n", "\n").replace('\n', "\r\n")).is_ok());
    }
}
