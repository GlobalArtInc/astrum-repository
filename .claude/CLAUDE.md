# Astrum Repository: project memory

Astrum Repository is GlobalArt's artifact repository manager, running in production at
https://astrum.globalart.hu. It is a Rust server (Cargo workspace in `crates/`) with a React web UI in
`web/`, written from scratch to replace the earlier Java server, a modified upstream repository manager
with the Astrum One UI. M6 removed that server and its attribution; its code is in the git history before the commit
"Remove the Java server and its build" (find it with `git log --diff-filter=D --oneline -- pom.xml`). The
decisions, the clean-room rules, the milestones and the history of the rewrite were kept in `.claude/rewrite/`,
removed on 2026-10-20 (find it with `git log --diff-filter=D --oneline -- .claude/rewrite/README.md`). This
repository is public: never commit secrets.

## State
- The server is version `3.0.0-dev` in `Cargo.toml` (the Java line was 2.x). It serves the 23 formats
  of the Java server: alpine, apt, cargo, cocoapods, composer, conan, conda, docker (with OCI Distribution 1.1),
  gitlfs, go, helm, huggingface, maven2, npm, nuget, p2, pypi, r, raw, rubygems, swift, terraform and yum, and
  two new ones, pub and Ansible Galaxy (`ansible`); since 2026-10-21 also Open VSX (`openvsx`), pacman,
  Vagrant, Nix (binary cache), Hex, Puppet Forge (`puppet`) and Chef Supermarket (`chef`), built in parallel
  from one brief, one worktree each. The web interface speaks twelve languages, pulled from Ownlate at build.
- Configuration comes from flags, `ASTRUM_*` environment variables and `<data>/etc/astrum.toml`; PostgreSQL
  is the only database; the image runs as uid 200 on port 8081 with its data in `/astrum-data`. The user
  documentation is in `docs/`.
- Blob stores are file, S3, remote and group (`crates/astrum-blob`). A remote store keeps blobs on another
  server through `astrum-blob-agent` (`crates/astrum-blob-agent`, a second binary in the same image; HTTP
  under `/v1`, bearer token); a group grows a store onto it, with the fill policy `mostFreeSpace`, and
  `POST /service/rest/v1/blobstores/group/convert/{name}/{newNameForOriginal}` turns a store that
  repositories use into a group. No production store uses an agent yet (`docs/configure/blob-stores.md`).
- MCP for agents at `POST /mcp` (`crates/astrum-mcp` is the JSON-RPC protocol, `crates/astrum-api/src/routes/mcp`
  the tools; `docs/use/mcp.md`): stateless Streamable HTTP, access tokens as bearer, the tools call the REST
  routes in process (`crate::loopback`) with the caller already resolved, so rights, validation and audit are
  those of REST. OAuth 2.1 for agents (`crates/astrum-security/src/oauth.rs`, `crates/astrum-api/src/routes/oauth.rs`,
  consent page `web/src/areas/user/AuthorizePage.tsx`): dynamic registration, PKCE S256, tokens `astrum_at_`/`astrum_rt_`
  accepted only on `/mcp`; rate limits in `crates/astrum-api/src/limits.rs`; MCP changes carry `via: mcp` in audit.
- Monitoring capabilities (`crates/astrum-api/src/routes/capabilities.rs`, `docs/operate/monitoring.md`):
  `metrics.prometheus` serves `GET /service/metrics/prometheus` (`crates/astrum-api/src/metrics.rs`, a
  hand-written text exposition; request counters come from a layer in `on_domains`, the rest is measured
  per scrape) to callers with `as-metrics-all`, 404 while off; `blobstores.space` (`crates/astrum-api/src/space.rs`)
  checks free space, soft quotas and reachability on one node at a time (advisory lock `astrspac`), keeps
  what it reported in the setting `monitoring.blobStoreSpace.state` and queues messages in
  `notification_delivery` for the channels it names (texts in `crates/astrum-notify/src/alerts.rs`).
- A repository can answer on a domain of its own (`domain` section: `ROOT` whole domain, `PATH` a path on a
  shared domain; `crates/astrum-api/src/domains.rs`, `docs/configure/reverse-proxy.md`). No production
  repository uses one yet: the owner wants `docker-onprem` on `docker-prem.globalart.hu` later, and gitops
  (a route for the host) and DNS are not set up for it.
- Production since 2026-10-18 06:02 UTC: gitops `clusters/prod1/platform/astrum/httproute.yaml` sends
  https://astrum.globalart.hu to the service `astrum-next:8082`, which is `astrum-next.service` on
  `prod1-astrum1` (rootless podman of the user `podman`, port 8082, data `/opt/astrum-next/data`, database
  `astrum-next` on `prod1-dbms1`); the infra role still names the Rust server `astrum-next`. The test host
  next.astrum.globalart.hu is gone (gitops c186d87).
- The Java server still runs on `prod1-astrum1` (`astrum.service`, port 8081, image
  `20261018044844-0da8ef5`, data `/opt/astrum/data`, database `astrum-repository`) as the rollback target,
  frozen at the state of the switch: writes after 06:02 exist only in the Rust server. The rollback (gitops
  patches, copy back with `tools/repository-copy`, which had another name in that commit) and the retirement of the Java container
  (`infra-2-retire-java.patch`) are in `git show 1940681a:docs/migrate/switch-production.md` and next to it in
  that commit. Backups of the Java database and `etc` taken at the switch are in `/opt/astrum-next/backup`.
  The one-off migration tasks of the infra role are removed (infra history: `tasks/next-migrate.yaml`); the
  import is in images up to `next-20261018054030-290bcf3` only.

## Where the work stands
- Done: M0-M4 (the six M4 wave 2 branches `m4-*` are history, do not continue on them), M4c (apt, cargo, composer, conan, go; then alpine, cocoapods, conda, p2, r, pub and Ansible Galaxy,
  one agent and branch `m4c-<fmt>` each, brief `/home/user/wt/M4C2-ENV.md` of that container), M5 (the
  migration, three rehearsals and the production switch of 2026-10-18) and M6 (removal of the Java tree and,
  after the switch, of the Java import; licence MIT; since 2026-10-18 the tree names neither the upstream
  product nor its vendor). Web keys are in Ownlate (all `web.*` keys as of 2026-10-18; stale `web.*` keys
  there are not in the dictionaries).
- Not done: retiring
  the Java container on `prod1-astrum1` (owner's call; the runbook asked for two quiet weeks first),
  renaming `astrum-next` to `astrum` in infra and gitops afterwards.
- Owner deferred (2026-10-18): no limit on failed sign-ins, unknown Bearer tokens answer 401 instead of
  anonymous, uploader and client address visible to readers, task types without a counterpart and cleanup
  `retain`, the Swift upload form and `/v1/lifecycle/phase`. On `docker-prod` (`forceBasicAuth`) the Java
  server challenged with Basic, the Rust server challenges with Bearer.
- Owner to decide, from M4: the Rust SSRF guard also blocks 100.64/10, 0/8, multicast and broadcast, which
  the Java server allowed; unchanged proxy remote URLs are not re-checked on save and unresolvable hosts are
  accepted (the Java server refused both); group repositories accept routing rules over REST but the form
  offers none; Swift proxies answer 503; the HTTP client of the server does not use the environment proxy
  (`HTTPS_PROXY`), only the HTTP settings; the SSRF page exists only in the new UI and is kept out of the
  settings hub; storing uploaded maven checksum files.
- A fresh environment needs: the Rust toolchain of `rust-toolchain.toml`; PostgreSQL for tests
  (`ASTRUM_TEST_DATABASE_URL`; 16 or 18 both work); Node 22 with corepack pnpm for `web/`; Chromium for
  Playwright at `/opt/pw-browsers/chromium` or `ASTRUM_E2E_CHROMIUM` (on Ubuntu 26.04 `playwright install`
  refuses the OS, but its downloaded chromium-1140 runs: symlink
  `~/.cache/ms-playwright/chromium-1140/chrome-linux/chrome`); `gh` logged in for workflow dispatch; infra
  and gitops repositories for deploys. Production can be checked from a session only anonymously (status,
  repository and asset listings, downloads, the Docker token flow); never with real credentials.
- Disk: a cloud session has about 40 GB, and one build of every workspace test binary takes about 4 GB.
  Run `cargo test --workspace` in batches of `--test` names and delete the test executables after each batch;
  give each agent its own `CARGO_TARGET_DIR`. A target dir copied with `cp -al` shares the cargo lock files
  (`debug/.cargo-lock`, `.cargo-build-lock`, `.cargo-artifact-lock`) with its source, so recreate them in the
  copy or the builds wait on each other; and rustc rewrites some outputs in place, so the artifacts of the
  workspace crates leak between copies that build different branches: delete everything matching
  `*astrum*` under the copy's `debug/` (and `debug/incremental`) before its first build.
  One `CARGO_TARGET_DIR` shared by several worktrees leaks the same way, because cargo names workspace
  artifacts by their path inside the workspace: give every worktree its own target directory (and
  `CARGO_BUILD_JOBS=3` on the 16 GB Mac), never a shared one.
- Local services after a restart: PostgreSQL `pg_ctl -D /var/tmp/astrum-pg/data -l /var/tmp/astrum-pg/pg.log
  -o '-p 5433 -k /tmp' start` as `postgres` (`/usr/lib/postgresql/<16|18>/bin`).

## Working rules
- Commits: author Roman Khabarov <roman@globalart.dev>
  (`git -c user.name="Roman Khabarov" -c user.email=roman@globalart.dev -c commit.gpgsign=false commit ...`,
  also for cherry-pick and rebase). Plain English, a body that says why. No mention of
  AI tools, no Co-Authored-By or other trailers.
- Commit and pull request titles follow Conventional Commits (owner, 2026-10-20): `type(scope): summary`,
  with `type` one of `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `scope` the
  part of the tree (`web`, `api`, `blob`, `mcp`, `monitoring`, `docker`, ...) and the summary imperative and
  lowercase, without a final period, e.g. `feat(monitoring): serve prometheus metrics`. The squash commit
  takes the title of its pull request.
- Commits on `main` must show as Verified (owner, 2026-10-18), and only commits GitHub creates itself do:
  the cloud container signs with a key GitHub does not know (Unverified), an unsigned push shows no badge,
  and the session proxy refuses GraphQL and REST writes of commits. So push the work to the session's
  branch, open a pull request into `main` with the GitHub MCP tools and squash-merge it at once
  (`merge_method: squash`, the commit's own title and body, `expectedHeadSha` the full branch sha); the
  squash commit has Roman as author, GitHub as committer and a valid signature. Then restart the session
  branch from `main` (`git checkout -B <branch> origin/main`, push with `--force-with-lease`). The same for
  infra and gitops. Pull requests are only this vehicle: open, merge, no review.
- Deploy after every change that reaches `main` and touches the Rust server, the web UI, the Dockerfile or
  `LICENSE.txt`/`NOTICE.txt` (the `Server` workflow does not run for other changes, so documentation needs no
  deploy):
  1. Wait for the `Server` workflow (GitHub-hosted runners). Its "container image" job prints one line
     `published globalartltd/astrum-repository:<tag>` per tag: first the production tag
     `YYYYMMDDHHMMSS-<sha7>`, then the same image as `next-YYYYMMDDHHMMSS-<sha7>`. It also moves `edge`; a
     pushed `v*` tag publishes `<version>` and `latest`.
  2. In `GlobalArtInc/infra`, set `astrum_next_version` in `infralive/ansible/roles/astrum/defaults/main.yaml`
     to the production tag, commit `chore(astrum): update astrum_next_version to <tag>` as Roman, land it on `main` as
     above. Never put
     a Rust tag into `astrum_version`: that variable still deploys the Java container.
  3. Dispatch `provision.yml` on `main` with inputs `command=apply`, `filter=name=prod1-astrum1`; it runs
     on the self-hosted runners of the private infra repository (7–13 minutes). If OpenTofu init times out,
     re-run the failed jobs once.
- Never ask for, print or store passwords, tokens or keys. Clients under test must not reach public hosts
  with credentials (cut them off from the internet).
- Licence attribution lives only in LICENSE/NOTICE, never in file headers. Never copy code from other
  products. Do not name the upstream product or its vendor anywhere in the tree, in identifiers or in
  commit messages (owner, 2026-10-18); describe interfaces by their paths instead.
- Do not relax the SSRF protection for notification webhooks without an explicit request. Do not change
  the mail setup (Mailu, mail1) in infra without a go-ahead.
- Stop processes by pid; never `pkill -f`.
- Answer the owner in Russian; everything in git is in English.

## Layout
- Rust workspace: `Cargo.toml`, `rust-toolchain.toml`, `crates/` (`astrum-server` is the binary; `astrum-api`
  the REST API and the internal UI API; `astrum-content` the repository engine; `astrum-format-<id>` one
  crate per format; `astrum-security`, `astrum-blob`, `astrum-repo`, `astrum-tasks`, `astrum-db` with the
  migrations in `crates/astrum-db/migrations`, and others).
- Web UI: `web/` (`src/app` bootstrap and router, `src/shell`, `src/auth`, `src/api`, `src/ui` component
  layer, `src/i18n`, `src/styles` tokens, `src/areas/<area>/` pages with their `strings/`, `e2e/`
  Playwright specs); `web/README.md` is the how-to for page authors.
- `Dockerfile` and `.dockerignore` at the root build the web UI and the server into the image.
- Docs: `docs/` (install, configure, operate, use, migrate). Tools: `tools/i18n` (Ownlate tooling),
  `tools/repository-copy` (moving data from another server, and the copy back of a rollback), `tools/side-by-side`
  (compares a reference server with this one).
- CI: `.github/workflows/server.yml` (fmt, clippy, tests against PostgreSQL 16, the web checks, then the
  image is built, started against PostgreSQL, checked and published) and `i18n.yml`, on `ubuntu-latest`.

## Build and test
- Rust, from the root: `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings` and
  `ASTRUM_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5433/postgres cargo test --workspace --locked`
  (the local PostgreSQL trusts local connections; CI uses a PostgreSQL 16 service with a password).
- Web, in `web/`: `corepack pnpm install --frozen-lockfile`, then
  `corepack pnpm lint && corepack pnpm typecheck && corepack pnpm test && corepack pnpm i18n:check && corepack pnpm build`.
  End to end: `ASTRUM_BIN=<copy of the astrum binary> corepack pnpm e2e` (every spec file starts its own
  server on a fresh data dir and database; needs a build of `web/dist`, PostgreSQL and Chromium).
- Local run: `ASTRUM_DATABASE_URL=postgres://... cargo run --bin astrum -- --data-dir <data> --listen
  127.0.0.1:<port> --web-dir web/dist`; the first start writes `<data>/admin.password`. `ASTRUM_URL=http://127.0.0.1:<port>
  corepack pnpm dev` serves the UI with live reload. UI sign-in over REST: `POST /service/rest/internal/session`
  with JSON `{username, password}`.
- Image: `docker build -t astrum-server:local .` from the root.

## Translations (Ownlate)
- User-visible texts live in `web/src/**/strings/`; `en.json` is the source and is written by hand and committed.
  The other `<lang>.json` files are written by `node tools/i18n/i18n.mjs pull` at image build (the `Dockerfile`'s
  `web` stage, token as build secret `ownlate_token`), git ignores them and they must never be committed.
  Run `corepack pnpm i18n:check` in `web/` and `node tools/i18n/i18n.mjs check` after changing them.
- The Ownlate keys are `web.<key>`, plural forms `web.<key>#<form>`. `import ru <file>` rewrites each
  dictionary with only the keys in the file: build the file from `node tools/i18n/i18n.mjs catalogue ru` plus
  the new keys, then import.
- Ownlate project `f1f93c0e-4902-434d-a551-e94ba7e1e304`, file `119d5931-d6cc-4e49-aaa6-ce23ad9b6257`.
  Write source texts first (no language), then Russian with approve=true; keep batches under ~150; a
  "session expired" error may still have applied the write. The CI workflow pulls and checks them when the secrets are set.

## Parallel agents (how it worked)
- One worktree per task: `git worktree add -b <branch> /home/user/wt/<name> origin/main`; install
  `web/node_modules` in it (`corepack pnpm install --frozen-lockfile`) or symlink the main checkout's; own
  `CARGO_TARGET_DIR`, port and database.
- Merge with cherry-pick. Typical conflicts: `Cargo.lock` (regenerate it with cargo), the workspace
  dependency list in `Cargo.toml`, `crates/astrum-server/src/formats.rs`, `web/src/app/areas.ts` and
  `web/src/areas/settings/sections.ts`. Never resolve a non-list conflict (whole test blocks) by merging
  lists.
