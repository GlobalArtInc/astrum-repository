use astrum_security::{Action, ContentSelector};

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

#[derive(Debug)]
pub enum Grant<'a> {
    Whole,
    Selected(Vec<&'a ContentSelector>),
    Nothing,
}

impl Grant<'_> {
    pub fn any(&self) -> bool {
        !matches!(self, Grant::Nothing)
    }
}

impl Ctx {
    pub fn grant(&self, view: View) -> Grant<'_> {
        if self.trusted {
            return Grant::Whole;
        }
        let Some(subject) = &self.subject else {
            return Grant::Nothing;
        };
        let format = self.format_id();
        let repository = self.name();
        if subject.is_permitted(&format!(
            "astrum:repository-view:{format}:{repository}:{}",
            view.name()
        )) {
            return Grant::Whole;
        }
        if subject.scope == astrum_security::Scope::Read
            && !matches!(view, View::Browse | View::Read)
        {
            return Grant::Nothing;
        }
        let mut selected: Vec<&ContentSelector> = Vec::new();
        for grant in subject.selectors() {
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
            if !(action && format_matches && repository_matches) {
                continue;
            }
            if let Some(selector) = self.security.selectors.get(&grant.selector)
                && !selected.iter().any(|known| known.name == selector.name)
            {
                selected.push(selector);
            }
        }
        match selected.is_empty() {
            true => Grant::Nothing,
            false => Grant::Selected(selected),
        }
    }

    pub fn allowed(&self, view: View, path: &str) -> bool {
        match self.grant(view) {
            Grant::Whole => true,
            Grant::Selected(selectors) => selectors
                .iter()
                .any(|selector| selector.selector.matches(self.format_id(), path)),
            Grant::Nothing => false,
        }
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
