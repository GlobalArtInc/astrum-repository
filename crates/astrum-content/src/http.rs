use std::sync::Arc;

use astrum_outbound::{ClientOptions, Snapshot};
use astrum_repo::config::Repository;
use astrum_security::SecretBox;
pub(crate) use astrum_security::ssrf::{Guard, GuardedResolver, literal_allowed};

const MAX_REDIRECTS: usize = 10;

pub(crate) fn build(
    repository: &Repository,
    guard: Arc<Guard>,
    outbound: &Snapshot,
    secrets: &SecretBox,
) -> Result<reqwest::Client, reqwest::Error> {
    let connection = repository
        .attributes
        .http_client
        .as_ref()
        .and_then(|client| client.connection.as_ref());
    let options = ClientOptions {
        user_agent_suffix: connection.and_then(|connection| connection.user_agent_suffix.clone()),
        timeout_seconds: connection.and_then(|connection| connection.timeout),
        use_trust_store: connection
            .and_then(|connection| connection.use_trust_store)
            .unwrap_or(false),
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
    outbound
        .configure(reqwest::Client::builder(), secrets, &options)
        .redirect(redirects)
        .dns_resolver(Arc::new(GuardedResolver::new(guard)))
        .build()
}
