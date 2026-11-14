use std::collections::{BTreeMap, BTreeSet};

use astrum_db::audit::Event;
use astrum_repo::config::{Repository, valid_name};
use astrum_repo::{RepositoryType, TokenChange};
use astrum_security::Subject;
use serde_json::json;

use super::{DOMAIN, TOKEN_DOMAIN};
use crate::{ApiError, AppState, Caller};

const MAX_GROUP_DEPTH: usize = 10;

pub(crate) async fn resolve_repositories(
    state: &AppState,
    listed: &[String],
) -> Result<Vec<Repository>, ApiError> {
    if listed.is_empty() {
        return Err(ApiError::invalid(
            "repositories",
            "list at least one repository",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut found = Vec::with_capacity(listed.len());
    for name in listed {
        if !valid_name(name) {
            return Err(ApiError::invalid(
                "repositories",
                format!("\"{name}\" is not a repository name; wildcards are not allowed"),
            ));
        }
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err(ApiError::invalid(
                "repositories",
                format!("\"{name}\" is listed twice"),
            ));
        }
        let repository = astrum_repo::store::find(state.db().conn(), name)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(|| {
                ApiError::invalid(
                    "repositories",
                    format!("the repository \"{name}\" does not exist"),
                )
            })?;
        found.push(repository);
    }
    Ok(found)
}

fn can_read(subject: &Subject, repository: &Repository) -> bool {
    subject.is_permitted(&format!(
        "astrum:repository-view:{}:{}:read",
        repository.format.id, repository.name
    ))
}

pub(crate) async fn require_readable(
    state: &AppState,
    subject: &Subject,
    listed: &[Repository],
) -> Result<(), ApiError> {
    if listed
        .iter()
        .any(|repository| !can_read(subject, repository))
    {
        return Err(ApiError::Forbidden(
            "you cannot read a listed repository".to_owned(),
        ));
    }
    let mut seen: BTreeSet<String> = listed
        .iter()
        .map(|repository| repository.name.to_ascii_lowercase())
        .collect();
    let mut pending: Vec<(Vec<String>, usize)> = listed
        .iter()
        .filter(|repository| repository.kind == RepositoryType::Group)
        .map(|repository| (members(repository), 1))
        .collect();
    while let Some((names, depth)) = pending.pop() {
        for name in names {
            if !seen.insert(name.to_ascii_lowercase()) {
                continue;
            }
            let Some(member) = astrum_repo::store::find(state.db().conn(), &name)
                .await
                .map_err(ApiError::internal)?
            else {
                continue;
            };
            if !can_read(subject, &member) {
                return Err(ApiError::Forbidden(
                    "you cannot read everything a listed group contains".to_owned(),
                ));
            }
            if member.kind == RepositoryType::Group {
                if depth >= MAX_GROUP_DEPTH {
                    return Err(ApiError::invalid(
                        "repositories",
                        format!("a listed group nests more than {MAX_GROUP_DEPTH} levels deep"),
                    ));
                }
                pending.push((members(&member), depth + 1));
            }
        }
    }
    Ok(())
}

fn members(group: &Repository) -> Vec<String> {
    group
        .group()
        .map(|group| group.members().to_vec())
        .unwrap_or_default()
}

pub(crate) async fn audit_released_tokens(
    state: &AppState,
    caller: &Caller,
    repository: &str,
    tokens: &[TokenChange],
) {
    let mut customers: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
    for token in tokens {
        if let Some(customer) = token.customer.as_deref() {
            let counts = customers.entry(customer).or_default();
            match token.removed {
                true => counts.1 += 1,
                false => counts.0 += 1,
            }
        }
        if !token.removed {
            continue;
        }
        let mut attributes = json!({
            "id": token.id,
            "user": token.user_id,
            "repositories": token.repositories,
            "reason": "repository-deleted",
        });
        if let Some(customer) = &token.customer {
            attributes["customer"] = json!(customer);
        }
        state
            .audit(
                caller,
                Event::new(TOKEN_DOMAIN, "deleted", &token.name).with(attributes),
            )
            .await;
    }
    for (customer, (narrowed, deleted)) in customers {
        state
            .audit(
                caller,
                Event::new(DOMAIN, "tokens-updated", customer).with(json!({
                    "repository": repository,
                    "narrowed": narrowed,
                    "deleted": deleted,
                })),
            )
            .await;
    }
}
