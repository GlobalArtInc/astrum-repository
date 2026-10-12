use astrum_security::Action;

use crate::{ContentError, Ctx};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Browse,
    Read,
    Edit,
    Add,
    Delete,
}

impl View {
    pub fn name(self) -> &'static str {
        match self {
            View::Browse => "browse",
            View::Read => "read",
            View::Edit => "edit",
            View::Add => "add",
            View::Delete => "delete",
        }
    }

    fn action(self) -> Action {
        match self {
            View::Browse => Action::Browse,
            View::Read => Action::Read,
            View::Edit => Action::Edit,
            View::Add => Action::Add,
            View::Delete => Action::Delete,
        }
    }
}

impl Ctx {
    pub fn allowed(&self, view: View, path: &str) -> bool {
        if self.trusted {
            return true;
        }
        let Some(subject) = &self.subject else {
            return false;
        };
        let format = self.format_id();
        let repository = self.name();
        if subject.is_permitted(&format!(
            "astrum:repository-view:{format}:{repository}:{}",
            view.name()
        )) {
            return true;
        }
        if subject.scope == astrum_security::Scope::Read
            && !matches!(view, View::Browse | View::Read)
        {
            return false;
        }
        subject.selectors().iter().any(|grant| {
            let action = grant
                .actions
                .iter()
                .any(|granted| *granted == Action::All || *granted == view.action());
            let format_matches = grant.format == "*" || grant.format == format;
            let repository_matches = grant.repository == "*"
                || grant.repository.eq_ignore_ascii_case(repository)
                || grant
                    .repository
                    .strip_prefix("*-")
                    .is_some_and(|wanted| wanted == format);
            action
                && format_matches
                && repository_matches
                && self
                    .security
                    .selectors
                    .get(&grant.selector)
                    .is_some_and(|selector| selector.selector.matches(format, path))
        })
    }

    pub fn require(&self, view: View, path: &str) -> Result<(), ContentError> {
        if self.allowed(view, path) {
            return Ok(());
        }
        match &self.subject {
            Some(subject) if !subject.anonymous => Err(ContentError::Forbidden),
            _ => Err(ContentError::Unauthorized),
        }
    }
}
