use serde::{Deserialize, Serialize};

use crate::config::problems::{Problems, blank_to_none};
use crate::config::{Check, Context};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub member_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable_member: Option<String>,
}

impl Group {
    pub fn members(&self) -> &[String] {
        self.member_names.as_deref().unwrap_or_default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.members()
            .iter()
            .any(|member| member.eq_ignore_ascii_case(name))
    }

    pub fn forget(&mut self, name: &str) -> bool {
        let before = self.members().len();
        if let Some(members) = &mut self.member_names {
            members.retain(|member| !member.eq_ignore_ascii_case(name));
        }
        let mut changed = before != self.members().len();
        if self
            .writable_member
            .as_deref()
            .is_some_and(|member| member.eq_ignore_ascii_case(name))
        {
            self.writable_member = None;
            changed = true;
        }
        changed
    }
}

impl Check for Group {
    fn check(&mut self, context: &Context<'_>, _stored: Option<&Self>, problems: &mut Problems) {
        if let Some(members) = &mut self.member_names {
            members
                .iter_mut()
                .for_each(|member| *member = member.trim().to_owned());
            members.retain(|member| !member.is_empty());
        }
        if self.members().is_empty() {
            problems.add("memberNames", "must not be empty");
        }
        blank_to_none(&mut self.writable_member);
        if !context.spec.has_writable_member() {
            self.writable_member = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::Spec;

    fn check(format: &str, value: serde_json::Value) -> (Group, usize) {
        let mut group: Group = serde_json::from_value(value).unwrap();
        let context = Context::new(Spec::from_api(format, "group").unwrap(), &str::to_owned);
        let mut problems = Problems::default();
        group.check(&context, None, &mut problems);
        (group, problems.finish().err().unwrap_or_default().len())
    }

    #[test]
    fn a_group_has_members() {
        assert_eq!(check("raw", json!({})).1, 1);
        assert_eq!(check("raw", json!({"memberNames": []})).1, 1);
        assert_eq!(check("raw", json!({"memberNames": [" "]})).1, 1);
        let (group, problems) = check("raw", json!({"memberNames": ["a", "B"]}));
        assert_eq!(problems, 0);
        assert!(group.contains("b"));
    }

    #[test]
    fn only_some_formats_have_a_writable_member() {
        let given = json!({"memberNames": ["a"], "writableMember": "a"});
        assert_eq!(
            check("docker", given.clone()).0.writable_member.as_deref(),
            Some("a")
        );
        assert_eq!(check("npm", given).0.writable_member, None);
        assert_eq!(
            check("cargo", json!({"memberNames": ["a"], "writableMember": ""}))
                .0
                .writable_member,
            None
        );
    }

    #[test]
    fn a_forgotten_member_leaves_every_place() {
        let mut group = Group {
            member_names: Some(vec!["a".to_owned(), "b".to_owned(), "A".to_owned()]),
            writable_member: Some("a".to_owned()),
        };
        assert!(group.forget("a"));
        assert_eq!(group.members(), ["b"]);
        assert_eq!(group.writable_member, None);
        assert!(!group.forget("c"));
    }
}
