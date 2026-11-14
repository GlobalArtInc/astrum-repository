pub const COMPONENT_PREFIX: &str = "c:";
pub const IMAGE_PREFIX: &str = "oci:";

pub fn component(id: i64) -> String {
    format!("{COMPONENT_PREFIX}{id}")
}

pub fn image(digest: &str) -> String {
    format!("{IMAGE_PREFIX}{digest}")
}

pub fn image_digest(subject: &str) -> Option<&str> {
    subject
        .strip_prefix(IMAGE_PREFIX)
        .filter(|digest| !digest.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_subject_is_the_prefix_and_the_id() {
        assert_eq!(component(42), "c:42");
        assert!(component(7).starts_with(COMPONENT_PREFIX));
    }

    #[test]
    fn an_image_subject_is_the_prefix_and_the_digest_and_reads_back() {
        let subject = image("sha256:ab");
        assert_eq!(subject, "oci:sha256:ab");
        assert_eq!(image_digest(&subject), Some("sha256:ab"));
        assert_eq!(image_digest("oci:"), None);
        assert_eq!(image_digest("c:7"), None);
    }
}
