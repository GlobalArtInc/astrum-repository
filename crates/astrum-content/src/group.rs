use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;

use astrum_repo::RepositoryType;
use astrum_repo::config::Repository;

use crate::{ContentError, Ctx};

const MAX_DEPTH: usize = 16;

pub async fn members(ctx: &Ctx) -> Result<Vec<Arc<Repository>>, ContentError> {
    let mut leaves = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(ctx.name().to_owned());
    let mut stack: Vec<(String, usize)> = names(&ctx.repository)
        .into_iter()
        .rev()
        .map(|name| (name, 1))
        .collect();
    while let Some((name, depth)) = stack.pop() {
        if depth > MAX_DEPTH || !seen.insert(name.clone()) {
            continue;
        }
        let Some(member) = ctx.content.repository(&name).await? else {
            continue;
        };
        if !member.online || member.format.id != ctx.format_id() {
            continue;
        }
        if let Some(path) = ctx.request_path()
            && !ctx.content.routing_allows(&member, path).await?
        {
            continue;
        }
        match member.kind {
            RepositoryType::Group => {
                stack.extend(
                    names(&member)
                        .into_iter()
                        .rev()
                        .map(|name| (name, depth + 1)),
                );
            }
            _ => leaves.push(member),
        }
    }
    Ok(leaves)
}

fn names(repository: &Repository) -> Vec<String> {
    repository
        .group()
        .and_then(|group| group.member_names.clone())
        .unwrap_or_default()
}

pub async fn writable_member(ctx: &Ctx) -> Result<Option<Arc<Repository>>, ContentError> {
    let Some(name) = ctx
        .repository
        .group()
        .and_then(|group| group.writable_member.clone())
    else {
        return Ok(None);
    };
    Ok(ctx
        .content
        .repository(&name)
        .await?
        .filter(|member| member.online && member.kind == RepositoryType::Hosted))
}

pub async fn first<T, F, Fut>(ctx: &Ctx, mut visit: F) -> Result<Option<(Ctx, T)>, ContentError>
where
    F: FnMut(Ctx) -> Fut,
    Fut: Future<Output = Result<Option<T>, ContentError>>,
{
    for member in members(ctx).await? {
        let member_ctx = ctx.member(member);
        match visit(member_ctx.clone()).await {
            Ok(Some(found)) => return Ok(Some((member_ctx, found))),
            Ok(None) => {}
            Err(error) => {
                tracing::info!(group = ctx.name(), member = member_ctx.name(), %error, "a group member failed; trying the next");
            }
        }
    }
    Ok(None)
}
