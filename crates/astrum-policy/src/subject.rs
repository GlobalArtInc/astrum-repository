pub const COMPONENT_PREFIX: &str = "c:";

pub fn component(id: i64) -> String {
    format!("{COMPONENT_PREFIX}{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_subject_is_the_prefix_and_the_id() {
        assert_eq!(component(42), "c:42");
        assert!(component(7).starts_with(COMPONENT_PREFIX));
    }
}
