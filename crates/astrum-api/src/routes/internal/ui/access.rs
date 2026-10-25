use std::collections::BTreeSet;
use std::sync::Arc;

use astrum_content::{Ctx, Grant, View};
use astrum_repo::Repository;
use astrum_security::selector::Columns;
use astrum_security::{Selector, Subject};
use sea_orm::{DbBackend, Statement, Value as SqlValue};
use serde::Serialize;

use crate::{ApiError, AppState, Caller};

#[derive(Clone, Debug)]
pub(super) enum Reach {
    Whole,
    Selected(Vec<Selector>),
    Hidden,
}

impl Reach {
    pub(super) fn visible(&self) -> bool {
        !matches!(self, Reach::Hidden)
    }

    pub(super) fn sees(&self, format: &str, path: &str) -> bool {
        match self {
            Reach::Whole => true,
            Reach::Selected(selectors) => selectors
                .iter()
                .any(|selector| selector.matches(format, path)),
            Reach::Hidden => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Flags {
    pub browse: bool,
    pub read: bool,
    pub add: bool,
    pub edit: bool,
    pub delete: bool,
    pub admin: bool,
    pub admin_edit: bool,
    pub admin_delete: bool,
}

#[derive(Clone)]
pub(super) struct Seen {
    pub repository: Arc<Repository>,
    pub reach: Reach,
    pub flags: Flags,
    pub limited: bool,
}

pub(super) struct Survey {
    pub seen: Vec<Seen>,
    pub all: Vec<Arc<Repository>>,
}

impl Survey {
    pub(super) fn find(&self, name: &str) -> Option<&Seen> {
        self.seen.iter().find(|seen| seen.repository.name == name)
    }

    pub(super) fn names(&self) -> BTreeSet<&str> {
        self.seen
            .iter()
            .map(|seen| seen.repository.name.as_str())
            .collect()
    }
}

pub(super) fn subject(caller: &Caller) -> Result<Subject, ApiError> {
    caller.subject().cloned().ok_or(ApiError::Unauthorized)
}

pub(super) fn context(state: &AppState, subject: &Subject, repository: Arc<Repository>) -> Ctx {
    state.content().context(
        repository,
        state.security().model(),
        Some(subject.clone()),
        "",
        "",
    )
}

pub(super) fn reach_of(ctx: &Ctx) -> Reach {
    let mut selectors: Vec<Selector> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for view in [View::Browse, View::Read] {
        match ctx.grant(view) {
            Grant::Whole => return Reach::Whole,
            Grant::Selected(found) => {
                for selector in found {
                    if !names.contains(&selector.name) {
                        names.push(selector.name.clone());
                        selectors.push(selector.selector.clone());
                    }
                }
            }
            Grant::Nothing => {}
        }
    }
    match selectors.is_empty() {
        true => Reach::Hidden,
        false => Reach::Selected(selectors),
    }
}

pub(super) fn seen(state: &AppState, subject: &Subject, repository: Arc<Repository>) -> Seen {
    let ctx = context(state, subject, repository.clone());
    let admin = |action: &str| {
        subject.is_permitted(&format!(
            "astrum:repository-admin:{}:{}:{action}",
            repository.format.id, repository.name
        ))
    };
    let reach = reach_of(&ctx);
    let flags = Flags {
        browse: ctx.grant(View::Browse).any(),
        read: ctx.grant(View::Read).any(),
        add: ctx.grant(View::Add).any(),
        edit: ctx.grant(View::Edit).any(),
        delete: ctx.grant(View::Delete).any(),
        admin: admin("read"),
        admin_edit: admin("edit"),
        admin_delete: admin("delete"),
    };
    let limited = matches!(reach, Reach::Selected(_));
    Seen {
        repository,
        reach,
        flags,
        limited,
    }
}

pub(super) fn anonymous_reads(
    state: &AppState,
    anonymous: Option<&Subject>,
    repository: &Arc<Repository>,
) -> bool {
    let Some(anonymous) = anonymous else {
        return false;
    };
    !repository.forces_basic_auth()
        && context(state, anonymous, repository.clone())
            .grant(View::Read)
            .any()
}

pub(super) async fn survey(state: &AppState, caller: &Caller) -> Result<Survey, ApiError> {
    let subject = subject(caller)?;
    let all: Vec<Arc<Repository>> = astrum_repo::store::list(state.db().conn())
        .await?
        .into_iter()
        .map(Arc::new)
        .collect();
    let seen = all
        .iter()
        .map(|repository| seen(state, &subject, repository.clone()))
        .filter(|seen| {
            seen.reach.visible()
                || seen.flags.admin
                || seen.flags.add
                || seen.flags.edit
                || seen.flags.delete
        })
        .collect();
    Ok(Survey { seen, all })
}

#[derive(Default)]
pub(super) struct Sql {
    values: Vec<SqlValue>,
}

impl Sql {
    pub(super) fn bind(&mut self, value: impl Into<SqlValue>) -> String {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }

    pub(super) fn statement(&self, sql: &str) -> Statement {
        Statement::from_sql_and_values(DbBackend::Postgres, sql, self.values.clone())
    }

    pub(super) fn selectors(&mut self, selectors: &[Selector]) -> String {
        let columns = Columns::new("a.format", "a.path");
        let parts: Vec<String> = selectors
            .iter()
            .map(|selector| {
                let fragment = selector.to_sql(&columns, self.values.len() + 1);
                self.values
                    .extend(fragment.values.into_iter().map(SqlValue::from));
                fragment.sql
            })
            .collect();
        format!("({})", parts.join(" OR "))
    }

    pub(super) fn reach(&mut self, reach: &Reach) -> Option<String> {
        match reach {
            Reach::Whole => Some("TRUE".to_owned()),
            Reach::Selected(selectors) => Some(self.selectors(selectors)),
            Reach::Hidden => None,
        }
    }

    pub(super) fn visible<'a>(
        &mut self,
        seen: impl IntoIterator<Item = &'a Seen>,
    ) -> Option<String> {
        let mut whole: Vec<String> = Vec::new();
        let mut clauses: Vec<String> = Vec::new();
        for seen in seen {
            match &seen.reach {
                Reach::Whole => whole.push(seen.repository.name.clone()),
                Reach::Selected(selectors) => {
                    let name = self.bind(seen.repository.name.clone());
                    let fragment = self.selectors(selectors);
                    clauses.push(format!("(a.repository = {name} AND {fragment})"));
                }
                Reach::Hidden => {}
            }
        }
        if !whole.is_empty() {
            let names = self.bind(whole);
            clauses.insert(0, format!("a.repository = ANY({names})"));
        }
        match clauses.is_empty() {
            true => None,
            false => Some(format!("({})", clauses.join(" OR "))),
        }
    }
}

pub(super) fn like_prefix(prefix: &str) -> String {
    format!(
        "{}%",
        prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}
