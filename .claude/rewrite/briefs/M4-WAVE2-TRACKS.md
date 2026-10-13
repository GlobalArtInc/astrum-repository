# M4 wave 2: tracks, their branches and where each stopped (2026-10-13, 18:40 UTC)

Six agents worked in parallel (xhigh effort), one per track, each on its own branch from main `31e33ae`. The
owner stopped them at about 18:40 UTC to save the usage limit. Every branch is pushed to origin. Nothing of
them is merged into main or deployed.

A commit titled "Keep the unfinished ..." holds work the agent had not checked yet (no fmt, clippy, tests,
web checks or e2e ran on it). It must be checked, fixed and folded into proper commits on its branch before
anything reaches main. The other commits passed their checks when they were made (per agent; the merged
whole has not been checked).

## How to continue on another server
1. Set up the environment (see `.claude/CLAUDE.md`, "A fresh environment needs").
2. Per track: `git worktree add <wt>/m4-<track> origin/m4-<track>` on a local branch `m4-<track>`, an own
   `CARGO_TARGET_DIR`, `web/node_modules` with `corepack pnpm install --frozen-lockfile`.
3. Start one agent per track with `.claude/rewrite/briefs/M4-WAVE2-TASK.md` (adjust its paths) plus the track's
   scope below, and tell it: continue from the branch; first check the "Keep the unfinished ..." commit, fix
   what fails and fold it into proper commits (a soft reset and recommit on its own branch is fine).
4. Merge finished tracks into main by cherry-pick as Roman, run the full Rust, web and e2e checks on main,
   push, deploy to next. Connect `notify`'s mail trait to `outbound`'s SMTP sender (`astrum-mail`) when both
   are merged.

## Tracks

### outbound — branch `m4-outbound`, ports 18100-18199, database prefix `m4outbound_`
Scope: routing rules (`/v1/routing-rules`, applied to proxy and group requests, selector in repository settings,
pages with a path test); HTTP settings (user agent suffix, timeout, retries, HTTP/HTTPS proxy with auth and
non-proxy hosts) used by the outbound client; SSL truststore (`/v1/security/ssl`, `/v1/security/ssl/truststore`,
certificate pages, fix of the repository certificate check that called a missing endpoint); email (`/v1/email`,
`/v1/email/verify`, an SMTP sender other features can use, the settings page with a test send). Secrets
encrypted, masked in GET.
- Done: `c010ca6a` routing rules on the server and in the engine; `3e45cbbb` routing rule pages.
- Unchecked: `e034cc02` crates `astrum-outbound` (HTTP settings, proxy, TLS, truststore, certificate fetch) and
  `astrum-mail` (settings, SMTP), routes `outbound.rs` and `email.rs` with tests, web area `outbound` (HTTP,
  email, SSL pages), the routing rule selector in `RepositoryForm`. It stopped while writing the routing rules
  integration test.
- Left: check and finish the unchecked commit, e2e for the pages, UI parity screenshots.

### notify — branch `m4-notify`, ports 19200-19299 (127.0.0.1:18200 is taken by the owner's `tsh`), prefix `m4notify_`
Scope: notifications ported from our `astrum-notifications-plugin` (multi-replica safe delivery, webhooks
through the SSRF protection, email behind a small trait); settings replacing capabilities (base URL, UI settings,
branding with sanitized HTML) with an audit event per change; the nodes page.
- Done: `95387c5d` notifications on the server; `f6faeeeb` base URL, UI and branding settings on the server.
- Unchecked: `0eaef8a4` web area `notify` (notification channels, capability-style settings pages, nodes page),
  branding in the shell with an HTML sanitizer, `web/e2e/notify.spec.ts`. It stopped before building the web
  dist and binary to run that e2e spec.
- Left: check the unchecked commit, run its e2e, parity screenshots.

### support — branch `m4-support`, ports 18300-18399, prefix `m4support_`
Scope: logs viewer, runtime log levels, system information, health (`/v1/status`, `/status/writable`,
`/status/check`) and the header bell, support ZIP, API explorer over `/service/rest/swagger.json`, the
`/v1/user-telemetry/javascript` 404.
- Done: `5ad212d8`, `232730b5`, `6b9161ba`, `fc05efc0`, `b88fee7d`, `aa073d93` (server, pages, e2e, plan notes).
- Unchecked: nothing. It stopped while running the whole-project web checks before its report.
- Left: the final web checks and the full e2e suite once; confirm the user-telemetry item.

### compat — branch `m4-compat`, ports 18400-18499 (docker connector tests 18500-18599), prefix `m4compat_`
Scope: Nexus 2 style URLs (`/content/repositories/...`, `/content/groups/...`); docker connector ports,
subdomain routing and the port suggestion; M2 leftovers (SSRF of remote URLs and S3 endpoints, S3
test-connection, real sizes in the repository list, uniqueness of docker `subdomain` and terraform
`registryHost`, maven `.index` if cheap, npm audit, pypi tokenized search, docker blob garbage collection).
- Done: `3fd328bb` Nexus 2 URLs; `966f277b` docker connector ports and subdomain routing; `58db8550` port
  suggestion.
- Unchecked: `58d746fd` SSRF checks of proxy remote URLs and S3 endpoints when repositories are saved. It
  stopped while splitting its work into a server and a UI commit.
- Left: check the unchecked commit, then the rest of the M2 leftovers in order of value.

### fmt-a — branch `m4-fmt-a`, ports 18600-18699, prefix `m4fmta_`
Scope: yum, rubygems, gitlfs ported from our Java modules `public/common/components/formats/astrum-repository-
{yum,rubygems,gitlfs}`, each with hosted/proxy/group as the Java module has, upload specs, browse tree, usage
snippets, cleanup hooks, tested with the real client (dnf/createrepo_c, gem/bundler, git-lfs).
- Done: `03923fe5` yum hosted, proxy and group.
- Unchecked: `f1a8bc84` crate `astrum-format-rubygems` (gem parsing, Marshal specs, indexes) and its
  registration; it stopped while writing the format itself (`lib.rs`), before the UI part.
- Left: finish rubygems (server, UI, real `gem`/`bundler` tests), then gitlfs entirely.

### fmt-b — branch `m4-fmt-b`, ports 18700-18799, prefix `m4fmtb_`
Scope: terraform, huggingface, swift ported from our Java modules, same requirements as fmt-a (terraform/tofu,
huggingface_hub, swift requests reproduced from SE-0292 when the CLI cannot be installed).
- Done: `6e169fb7`, `4dce4a84` terraform; `871fc8a2`, `3c4036d8` Hugging Face; `22ec71e8`, `8e856bae` Swift.
- Unchecked: nothing. It stopped while touching the Hugging Face web part again (usage snippets, search keys,
  remote URL example).
- Left: confirm that part, the final checks and the full e2e suite once.

## Not started
Item 3 of the remaining work (open defects and M3 leftovers: idle-session countdown, response compression,
repository profile tabs, saving raw proxy query params / docker `latestPolicy` / ECR settings from Astrum One,
horizontal scroll at 1024px, server texts in English only, group blob stores in Astrum One) and M4c.
