# M2 brief for agents (Rust rewrite of Astrum Repository)

You work on the from-scratch Rust server of Astrum Repository (repository GlobalArtInc/astrum-repository).
Read first, in this order:
1. `.claude/CLAUDE.md` (working rules) and `.claude/rewrite/README.md` (plan, clean-room rules, M0 and the
   "M1 conventions" section). The clean-room rules are binding: never read or paraphrase the Java sources
   under `public/` or the Astrum One React code. Allowed: `.claude/rewrite/reference/*` (the interface
   captured from the running server), `.claude/rewrite/inventory-*.md`, public protocol specs, the public
   REST API documentation, and black-box requests to the reference server below.
2. The existing Rust code in `crates/` (it is ours): `astrum-db` (entities, settings, audit), `astrum-security`
   (permissions, privileges, catalogue, model, `Security`, `SecretBox`, bootstrap), `astrum-api` (`AppState`,
   `Caller`, `ApiError`, `Validation`, `Json`, routes, `tests/support`), `astrum-server`.

## Environment
- Your worktree and branch are given in your task. Work only there. Commit with
  `git -c user.name="Roman Khabarov" -c user.email=roman@globalart.dev commit`, plain English, imperative
  subject, a body that says why, no trailers, no mention of AI tools. Do not push. Do not touch other worktrees.
- Build output: always `export CARGO_TARGET_DIR=/home/user/wt/m1-target` (shared by every agent to save disk;
  cargo may wait for a lock now and then, that is expected). Disk is limited: do not create other target dirs.
- PostgreSQL 16 for tests: `export ASTRUM_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5433/postgres`
  (trusts local connections; each test creates and drops its own database).
- Before you report, run exactly what CI runs and make it pass:
  `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked` (update `Cargo.lock` with a plain `cargo build` first when you add
  dependencies, then commit it).
- Reference server (the current Java Astrum 2.0.0, a black box): `http://127.0.0.1:8081`, user `admin`, password
  in `/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/e2e/admin-password.txt`.
  Read it into a shell variable; never print, log or commit it. You may create objects there to observe
  behaviour, named `m1-<your area>-...`, and you must delete them afterwards. Do not change its settings
  (realms, anonymous, users other than your own test objects). Do not stop or restart it.
- Never send credentials to public hosts. Do not run `pkill -f`; stop processes by pid.
- No comments in code (explain in commit messages); match the style of the existing crates: small modules,
  `thiserror` errors, tests next to the code and integration tests in `tests/`.

## Shared conventions (do not change without saying so in your report)
- Routes: put yours in your own module(s) under `crates/astrum-api/src/routes/`; the router already merges
  `routes::security`, `routes::blobstores`, `routes::repositories`. Full paths, e.g.
  `/service/rest/v1/security/users`. Use `Caller::require("astrum:<domain>:<action>")` before doing anything,
  `crate::Json` for request bodies, `axum::Json` for responses, `ApiError`/`Validation` for errors.
- JSON field names and shapes follow `.claude/rewrite/reference/openapi-m1.json` exactly (clients and the
  migrator depend on them). Status codes follow the reference server (verify by black-box requests when in
  doubt); error bodies use our `ApiError` shapes.
- After any write that changes users, roles, privileges, realms, anonymous settings or repositories call
  `state.security().reload().await?`.
- Record an audit event after each successful change: `state.audit(&caller, Event::new(domain, kind, name))`
  (see the conventions in the README for domains and kinds). Never put secrets into audit attributes.
- Secrets in configuration (passwords, keys, tokens) are stored sealed with `state.secrets().seal(..)` and are
  never returned by GET; a PUT that leaves a secret empty or absent keeps the stored one.
- New tables or columns: add a new migration pair in `crates/astrum-db/migrations/` named
  `<current epoch millis>-<slug>.up.sql/.down.sql` plus an entity; never edit existing migrations.
- Shared files you may need to touch (keep such edits minimal and list them in your report):
  root `Cargo.toml` (workspace members are `crates/*`; add workspace dependencies alphabetically),
  `Cargo.lock`, `crates/astrum-api/Cargo.toml`, `crates/astrum-api/src/routes/mod.rs`,
  `crates/astrum-server/src/server.rs` (startup hooks).

## Report
When done, report: what you built (endpoints, behaviour decisions, where you followed or deviated from the
reference server and why), the commits, test counts, files touched outside your area, and what is left.

## M2 additions (read these too)
- Build rule (disk is limited to about 12 GB free): always `export CARGO_TARGET_DIR=/home/user/wt/m1-target
  CARGO_INCREMENTAL=0`, and use only workspace forms: `cargo build|clippy|test --workspace ...` (filter tests by
  name after `--`). Never `-p` subsets, never another target dir. Do not leave big files behind; delete client
  caches, downloaded binaries and test data you created when you finish.
- Read `.claude/rewrite/README.md` sections "M2 conventions" and the code of `crates/astrum-content` and
  `crates/astrum-format-raw` (the model format) before writing anything. `crates/astrum-api/tests/raw.rs` shows
  how to test a format through the API (`Fixture`, a local upstream server for proxies, `allow_loopback` for
  the SSRF protection).
- Your format lives in its own crate `crates/astrum-format-<id>`; register it in
  `crates/astrum-server/src/formats.rs` (one line in `register`, and in `routes` only if you need top-level
  routes); integration tests go in `crates/astrum-api/tests/<id>.rs` (add your crate to the dev-dependencies of
  `crates/astrum-api/Cargo.toml`). Keep changes to `crates/astrum-content` minimal and generic, and list them in
  your report (other agents work in parallel against the same engine).
- Bearer tokens of a format realm: `state.security().issue_format_token(realm, user_id, lifetime)` returns
  `"<Realm>.<secret>"`; the auth middleware already accepts `Authorization: Bearer <Realm>.<secret>` while the
  realm is active (`astrum_security::format_tokens`).
- The Java format modules under `public/` are upstream-derived: do not read them. Work from the protocol
  specifications, `.claude/rewrite/reference/*` and black-box requests to the reference server on 127.0.0.1:8081
  (it has repositories maven-releases, maven-snapshots, maven-central (proxy), maven-public (group),
  npm-internal, npmjs (proxy), pypi-internal, docker-internal, raw-files; create your own probe repositories
  named `m2-<area>-...` and delete them afterwards).
- Real clients are required. Run the Rust server binary (`cargo build --workspace`, then
  `/home/user/wt/m1-target/debug/astrum serve` with `ASTRUM_DATA_DIR`, `ASTRUM_LISTEN=127.0.0.1:<your port>`,
  `ASTRUM_DATABASE_URL=postgres://postgres@127.0.0.1:5433/<your database>`; create the database with
  `su postgres -s /bin/sh -c "psql -h /tmp -p 5433 -c 'create database <name>'"`, drop it at the end) and test
  with the real client against it. For proxies use a local upstream (another repository of the same Rust server
  or the reference server) and set the SSRF allow list (`PUT /service/rest/v1/security/ssrf-protection` as
  admin with `allowedIPs: ["127.0.0.1"]`). Clients must never send credentials to public hosts: give them their
  own HOME/config, and point any default public registry at a dead address.
- Report also: which client versions you ran, which commands, and what they printed when something failed.
