use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Part {
    Any,
    Set(BTreeSet<String>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Permission {
    parts: Vec<Part>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("\"{0}\" is not a permission: use parts separated by ':' and values separated by ','")]
pub struct PermissionError(pub String);

impl Permission {
    pub fn parse(text: &str) -> Result<Self, PermissionError> {
        let invalid = || PermissionError(text.to_owned());
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(invalid());
        }
        let mut parts = Vec::new();
        for part in trimmed.split(':') {
            let values: BTreeSet<String> = part
                .split(',')
                .map(|value| value.trim().to_lowercase())
                .collect();
            if values.iter().any(String::is_empty) {
                return Err(invalid());
            }
            parts.push(match values.contains("*") {
                true => Part::Any,
                false => Part::Set(values),
            });
        }
        Ok(Self { parts })
    }

    pub fn implies(&self, other: &Permission) -> bool {
        for (index, wanted) in other.parts.iter().enumerate() {
            match (self.parts.get(index), wanted) {
                (None, _) => return true,
                (Some(Part::Any), _) => {}
                (Some(Part::Set(_)), Part::Any) => return false,
                (Some(Part::Set(granted)), Part::Set(wanted)) => {
                    if !wanted.is_subset(granted) {
                        return false;
                    }
                }
            }
        }
        self.parts
            .iter()
            .skip(other.parts.len())
            .all(|part| *part == Part::Any)
    }

    pub fn last_values(&self) -> Option<BTreeSet<&str>> {
        match self.parts.last()? {
            Part::Any => None,
            Part::Set(values) => Some(values.iter().map(String::as_str).collect()),
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .parts
            .iter()
            .map(|part| match part {
                Part::Any => "*".to_owned(),
                Part::Set(values) => values.iter().cloned().collect::<Vec<_>>().join(","),
            })
            .collect();
        f.write_str(&parts.join(":"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn implies(granted: &str, wanted: &str) -> bool {
        Permission::parse(granted)
            .unwrap()
            .implies(&Permission::parse(wanted).unwrap())
    }

    #[test]
    fn everything_under_a_prefix() {
        assert!(implies("astrum:*", "astrum:users:read"));
        assert!(implies("astrum", "astrum:users:read"));
        assert!(implies("*", "astrum:repository-view:maven2:central:read"));
        assert!(!implies("astrum:*", "other:users:read"));
    }

    #[test]
    fn values_and_lists() {
        assert!(implies("astrum:users:read,create", "astrum:users:read"));
        assert!(implies(
            "astrum:users:read,create",
            "astrum:users:create,read"
        ));
        assert!(!implies("astrum:users:read", "astrum:users:read,create"));
        assert!(!implies("astrum:users:read", "astrum:users:*"));
        assert!(implies("astrum:users:*", "astrum:users:delete"));
    }

    #[test]
    fn repository_permissions() {
        let granted = "astrum:repository-view:maven2:*:read,browse";
        assert!(implies(
            granted,
            "astrum:repository-view:maven2:central:read"
        ));
        assert!(implies(
            granted,
            "astrum:repository-view:maven2:releases:browse"
        ));
        assert!(!implies(granted, "astrum:repository-view:npm:central:read"));
        assert!(!implies(
            granted,
            "astrum:repository-view:maven2:central:add"
        ));
        assert!(implies(
            "astrum:repository-view:*:*:*",
            "astrum:repository-view:npm:x:delete"
        ));
    }

    #[test]
    fn a_longer_grant_needs_wildcards_at_the_end() {
        assert!(implies("astrum:users:read:*", "astrum:users:read"));
        assert!(!implies("astrum:users:read:own", "astrum:users:read"));
    }

    #[test]
    fn matching_ignores_case_and_spaces() {
        assert!(implies("Astrum:Users:READ", "astrum:users:read"));
        assert!(implies(
            "astrum:users: read , create",
            "astrum:users:create"
        ));
    }

    #[test]
    fn malformed_permissions_are_refused() {
        for bad in ["", " ", "astrum::read", "astrum:users:", "astrum:a,,b"] {
            assert!(Permission::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn display_is_canonical() {
        let permission = Permission::parse("Astrum:users:update,read").unwrap();
        assert_eq!(permission.to_string(), "astrum:users:read,update");
        assert_eq!(Permission::parse("a:*,b").unwrap().to_string(), "a:*");
    }
}
