use std::sync::Arc;
use std::time::Duration;

use astrum_repo::config::Repository;
pub(crate) use astrum_security::ssrf::{Guard, GuardedResolver, literal_allowed};

const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const MAX_REDIRECTS: usize = 10;

pub(crate) fn build(
    repository: &Repository,
    guard: Arc<Guard>,
) -> Result<reqwest::Client, reqwest::Error> {
    let connection = repository
        .attributes
        .http_client
        .as_ref()
        .and_then(|client| client.connection.as_ref());
    let timeout = connection
        .and_then(|connection| connection.timeout)
        .filter(|seconds| *seconds > 0)
        .map(|seconds| seconds as u64)
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS);
    let suffix = connection
        .and_then(|connection| connection.user_agent_suffix.clone())
        .filter(|suffix| !suffix.trim().is_empty());
    let agent = match suffix {
        Some(suffix) => format!("Astrum/{} {suffix}", env!("CARGO_PKG_VERSION")),
        None => format!("Astrum/{}", env!("CARGO_PKG_VERSION")),
    };
    let redirect_guard = guard.clone();
    let redirects = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            attempt.error("too many redirects")
        } else if !literal_allowed(&redirect_guard, attempt.url()) {
            attempt.error("the redirect goes to an address the SSRF protection blocks")
        } else {
            attempt.follow()
        }
    });
    reqwest::Client::builder()
        .user_agent(agent)
        .connect_timeout(Duration::from_secs(timeout))
        .read_timeout(Duration::from_secs(timeout))
        .redirect(redirects)
        .no_proxy()
        .dns_resolver(Arc::new(GuardedResolver::new(guard)))
        .build()
}
