# Rewriting Astrum Repository in Rust

Status: decided (owner's answers of 2026-10-12 are recorded below); M0 is done (2026-10-12), M1 is in
progress (foundation done; blob stores, repository configuration and security REST being built in parallel).
See "Where the Rust server stands" at the end for the code that exists.

## Why
Astrum Repository today is a modified version of Nexus Repository OSS (EPL-1.0), and its web UI (Astrum
One) grew out of the upstream preview UI; `@globalart/astrum-react-shared-components` is a fork of the
upstream component library. The owner wants Astrum to carry no Nexus code and no Nexus attribution at all.
A translation of the Java code into Rust would still be a derivative work and would keep the attribution,
so the new server and UI are written from scratch.

## Decisions (owner, 2026-10-12)
- A standalone Rust service with a Keel-like stack (axum, sea-orm, PostgreSQL), not a product on the Keel
  kernel (Keel's one-installation-per-host model and function dispatcher do not fit a high-traffic
  artifact server).
- Code lives in this repository (`astrum-repository`) next to the Java tree; the Java tree and the
  upstream attribution are removed only after the Rust server has replaced it in production and the data
  is migrated. Git history will keep the old code; that is accepted.
- The product keeps its name, Astrum Repository, and its address astrum.globalart.hu.
- Scope: the whole project — every format, the REST API, the UI, and the migration of production data.
- OCI support belongs to the Docker format, not a separate format.
- pub and Ansible Galaxy are written directly in Rust (their Java versions were stopped).
- PostgreSQL only; no embedded H2.
- No Groovy script API (`/v1/script`, `script` task and privilege type are dropped).
- Astrum stays compatible with Nexus Repository for clients and tools:
  - the public REST API keeps the Nexus 3 paths and JSON shapes (`/service/rest/v1/...`), so scripts,
    Terraform/Ansible providers and CI integrations written for Nexus keep working;
  - client URLs keep the same layouts (`/repository/<name>/...`, Docker `/v2/...`);
  - the Nexus 2 style URLs (`/content/repositories|groups|sites/...`, `/service/local/...`) keep working;
  - migration from Nexus 3 (`tools/nexus-migrate`, `docs/migrate/from-nexus.md`) stays, next to the
    migration from the current Java Astrum.
  Compatibility is implemented from the public API documentation and black-box behaviour, never from
  upstream code. Nexus is named only where compatibility or migration needs it.
- UI URLs are plain paths (`/browse/...`, `/admin/...`, `/settings`, `/user/...`, `/login`); old hash URLs
  (`#browse/...`, `#preview/...`, Classic URLs) redirect to them.

## Clean-room rules (every session and agent must follow them)
1. Do not read the Java sources of upstream-derived modules, or the Astrum One React code, as a model,
   and never translate or paraphrase them. Close the file if you opened it by accident.
2. Allowed sources:
   - public protocol specifications (Maven, npm, Docker/OCI distribution, PyPI simple/upload, NuGet V2/V3,
     Go module proxy, Cargo sparse index, Helm, APT, Alpine APKINDEX, Conda, Conan, Composer, CocoaPods
     CDN, P2, R CRAN-like, RubyGems, Swift registry SE-0292, Terraform registry protocols, Git LFS, Hugging
     Face Hub, Yum repodata, pub v2, Galaxy v3, ...);
   - the running Astrum server as a black box: requests from real clients and the responses they get;
   - the data layout in `inventory-data.md`, only to read existing installations;
   - the inventories in this folder (behaviour, URLs, settings, screens);
   - our own code (see "Our own code" below) and our own design tokens and Russian translations.
3. The REST API must stay compatible with Nexus 3 (paths and JSON shapes, see `inventory-server.md` and
   the public Nexus REST API documentation); implement it from that documented interface and black-box
   behaviour, not from the Java resources.
4. New English UI texts are written afresh. Russian texts are ours and can be reused.
5. No licence headers in files. The new code's licence goes in LICENSE once the Java tree is gone.

## Our own code (allowed as reference)
Written by us on top of the fork, from specifications or our own design:
- Formats: yum, rubygems, terraform, huggingface, swift, gitlfs (`public/common/components/formats/
  astrum-repository-{yum,rubygems,terraform,huggingface,swift,gitlfs}`), and the OCI 1.1 work in the
  Docker module once merged (branch `format/oci`).
- `security/astrum-ldap-plugin`, `security/astrum-oidc-plugin`, `security/astrum-access-token-plugin`.
- `astrum-notifications-plugin`; download counters and usage dashboard
  (`astrum-repository-content/.../content/downloads`, `.../content/usage`); cleanup preview totals.
- `tools/i18n` (Ownlate tooling), `tools/nexus-migrate` (migration from Nexus 3), the docs under `docs/`.
- The Astrum One design: `astrum-coreui-plugin/src/frontend/src/styles/astrum-brand.css` tokens, the
  dark default theme, header/menu/mobile layout, settings hub tiles, header search panel, usage blocks
  of the formats. Reuse the design, rewrite the components.
Everything else in `public/` is upstream-derived.

## Target architecture (proposal)
- Cargo workspace at the repository root (`Cargo.toml`, `crates/`), UI in `web/`.
- Crates (names indicative): `astrum-server` (binary, config, startup), `astrum-db` (sea-orm entities,
  migrations), `astrum-blob` (file and S3 stores, our own on-disk layout), `astrum-security` (users,
  roles, privileges, content selectors, realms, sessions, tokens, LDAP, OIDC), `astrum-repo` (repository
  model, hosted/proxy/group engine, proxy cache, negative cache, HTTP client, routing rules, cleanup,
  write policies), `astrum-search` (PostgreSQL full-text and trigram), `astrum-tasks` (scheduler and task
  types), `astrum-api` (public REST v1 compatible paths + internal API for the UI), `astrum-format-<id>`
  (one crate per format), `astrum-migrate` (import from an existing installation), `astrum-notify`,
  `astrum-audit`, `astrum-i18n`.
- PostgreSQL only. Our own schema; the migrator reads the old schema.
- Blob stores: our own layout; the migrator moves or hard-links the old blob files (same filesystem) or
  reads the old layout once, so large stores are not copied twice.
- Client-facing URLs stay: `/repository/<name>/...` per format, Docker `/v2/...` with path routing and
  per-repository ports, Terraform discovery at the host root.
- UI: React + Radix Themes with our tokens, our own component layer; English and Russian through Ownlate;
  path URLs served by the server (single-page fallback for UI paths that do not collide with
  `/repository/`, `/service/`, `/v2/`, `/content/`, `/.well-known/`); old hash URLs redirect, including
  the old-URL mapping from `inventory-ui.md`.
- Single binary + static UI in one container image (uid 200, port 8081, data in `/astrum-data`), built on
  GitHub-hosted runners (the repository is public).

## Milestones (each ends in a demonstrable state)
- M0 Skeleton: workspace, config, PostgreSQL migrations, health/status endpoints, container image, CI.
- M1 Core: blob stores (file, S3), users/roles/privileges/realms (local), sessions and CSRF, access
  tokens, content selectors, Nexus-compatible REST for security, repositories and blob stores, audit.
- M2 Engine + first formats: hosted/proxy/group engine with raw, maven2, npm, docker (incl. OCI 1.1),
  pypi; components/assets/search/browse API; uploads; tested with real clients.
- M3 UI: shell (header, menu, search panel, themes, languages, mobile), dashboard, browse, search,
  upload, settings for repositories, blob stores and security.
- M4 Everything else, in three parts so each ends in a working server and the least used work comes last
  (the whole of M4 was estimated at 180-300% of a weekly limit; ordered this way it is about 100-170% before
  M4c, which may never be needed):
  - M4a Formats production uses: take the list of repositories from production first and port only the
    formats that have repositories there. Our own formats (yum, rubygems, terraform, huggingface, swift,
    gitlfs) are ported from our own code and cost less than the clean-room ones.
  - M4b Operations: cleanup policies with previews, routing rules, tasks, settings that replace
    capabilities (base URL, UI settings, branding, default role, audit, webhooks), email, HTTP, SSL
    truststore, LDAP, OIDC, notifications, download counters and usage dashboard, logs and support pages,
    Nexus 2 style URLs. LDAP, OIDC, notifications and the counters are ours and are ported from our code.
  - M4c Formats production does not use (likely p2, r, cocoapods, conan, conda, pub, ansible): only on
    request; the migrator reports repositories of a format the Rust server does not have yet.
- M5 Migration: `astrum-migrate` (from the Java Astrum and from Nexus 3) against a copy of production data, side-by-side run with the Java
  server comparing client behaviour, switch production, rollback plan.
- M6 Removal: delete the Java tree and `@globalart/astrum-react-shared-components` usage, new LICENSE,
  drop the upstream attribution from NOTICE, update docs.

## Testing approach
- Every format is accepted only when its real client works against the new server (mvn, gradle, npm,
  docker/oras/helm, pip/twine, dnf, gem/bundler, terraform, huggingface_hub, git-lfs, cargo, go, nuget,
  composer, conda, conan, ...). Where a client cannot be installed, reproduce its requests exactly.
- Side-by-side harness: the same client scripts against the Java server and the Rust server, comparing
  outcomes (status, bodies where meaningful, headers clients rely on).
- Clients that may call public hosts by default run without internet access, so no credentials leak.

## Inventories
- `inventory-server.md` — formats, REST API, security, tasks, blob stores, settings, configuration.
- `inventory-data.md` — database tables, secrets, blob store layout, versions (for the migrator).
- `inventory-ui.md` — routes, pages, actions, shell, what is upstream-only.

## Where the Rust server stands
M0 (2026-10-12): the skeleton runs, builds into an image and is checked by CI; nothing of it is deployed.
- `Cargo.toml` (workspace, version `3.0.0-dev`: the Rust server replaces 2.x as 3.0), `rust-toolchain.toml`
  (1.98.0), crates:
  - `astrum-db`: `DatabaseConfig`/`DatabaseUrl` (the password never shows in logs or errors), `Database`
    (sea-orm pool with the search path set to the schema, `astrum` by default, created on connect;
    `migrate` under a session advisory lock so replicas starting together do not race on the ledger;
    `ping`; `accepts_writes` = not a standby and not `transaction_read_only`), migrations in
    `crates/astrum-db/migrations` (`<ms>-<slug>.up.sql`/`.down.sql`, applied by `sqlmig`, our crate, into
    the `migration` ledger), `setting` (key/jsonb store), `read_only` (read-only mode kept as a setting).
  - `astrum-api`: `AppState` and `router`; `GET /service/rest/v1/status` (200 when the database answers)
    and `/status/writable` (200 when the database accepts writes, read-only mode is off and the data
    directory is writable), 503 otherwise, as the public Nexus API documents them; everything else 404.
  - `astrum-server`: binary `astrum` with `serve` (default), `migrate`, `check` (exit 0 when the local
    server's status is 200, used by the image's HEALTHCHECK), `version`; configuration from flags >
    environment > file > defaults: `--config`/`ASTRUM_CONFIG` (default `<data dir>/etc/astrum.toml`),
    `--data-dir`/`ASTRUM_DATA_DIR` (`/astrum-data`), `--listen`/`ASTRUM_LISTEN` (`0.0.0.0:8081`),
    `ASTRUM_DATABASE_URL` (`database.url`, required to serve), `ASTRUM_DATABASE_SCHEMA`,
    `ASTRUM_DATABASE_MAX_CONNECTIONS` (at least 3), `ASTRUM_LOG` (`info,sqlx=warn`), `ASTRUM_LOG_FORMAT`
    (`text`/`json`). Graceful stop on SIGTERM/SIGINT.
  - `astrum-testkit`: `TestDatabase` creates a database per test on the server named by
    `ASTRUM_TEST_DATABASE_URL` and drops it afterwards.
- Image: `docker/server.Dockerfile` (+ `.dockerignore` next to it), build context the repository root;
  Debian trixie slim, uid/gid 200, `/astrum-data`, port 8081, `HEALTHCHECK astrum check`.
- CI: `.github/workflows/server.yml` (fmt, clippy `-D warnings`, tests against a PostgreSQL 16 service, then
  the image is built and started against PostgreSQL and checked). It is not pushed anywhere yet. The Java
  `Image` workflow ignores pushes that only touch the Rust server or `.claude/`.
- Local checks: `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings`
  and `ASTRUM_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5433/postgres cargo test --workspace --locked`
  (the local PostgreSQL 16 of the e2e setup trusts local connections). A local docker daemon starts with
  `dockerd &`; Docker Hub rate-limits anonymous pulls here, so pull bases from `mirror.gcr.io/library/...`
  and tag them; package installs inside builds cannot reach the network from this sandbox.

## M1 conventions (foundation of 2026-10-12)
- Interface reference: `reference/openapi-m1.json` (paths and JSON schemas of the security, blob store,
  repository and status REST API, descriptions stripped), `reference/builtin-privileges.json`,
  `builtin-roles.json`, `realms-available.json`, `user-sources.json`, `users-default.json`,
  `anonymous.json`, `repository-privileges-sample.json`, `responses.txt` (status codes and bodies of
  errors). Captured from the running server; ids and names are the compatibility surface (`as-*`
  privileges and roles, realm ids), English descriptions are ours.
- Tables (`crates/astrum-db/migrations/*-create-core-tables`): `security_user`, `security_role`,
  `security_privilege` (custom privileges only; built-in and per-repository privileges are generated),
  `security_user_role` (user, source → roles), `content_selector`, `blob_store`, `repository` (FK to
  `blob_store`, so a used store cannot be deleted), `audit_event`. Settings keys: `security.realms`,
  `security.anonymous`, `read_only`.
- `astrum-security`: `Permission` (wildcard parts `:`, values `,`, `*`; case-insensitive), privilege kinds
  → permissions: application `astrum:<domain>:<create|read|update|delete|...>` (ADD→create, EDIT→update),
  `astrum:repository-view:<fmt>:<repo>:<browse|read|edit|add|delete>`,
  `astrum:repository-admin:<fmt>:<repo>:<...>`, wildcard as written; content selector privileges are
  grants evaluated by content access (M2). `SecurityModel` is an in-memory snapshot; every write to users,
  roles, privileges, settings or repositories must be followed by `Security::reload()`. Passwords: argon2id;
  old `$shiro1$SHA-512$...` hashes verify and are rewritten on sign-in. Basic sign-ins are cached for 5
  minutes per (user, password, stored hash). `SecretBox` (AES-256-GCM, key in `<data>/etc/secrets.key` or
  `ASTRUM_SECRETS_KEY_FILE`) seals secrets kept in configuration JSON as `astrum:v1:...`; REST never returns
  them. `bootstrap` upserts the built-in roles, default realms and anonymous settings, and on an empty user
  table creates `admin` (random password in `<data>/admin.password`) and `anonymous`.
- `astrum-api`: `AppState` (db, data dir, security, secrets, `audit()`), `Caller` extractor with
  `require(permission)` (401 + `WWW-Authenticate: Basic realm="Astrum Repository"` for anonymous or no
  caller, 403 otherwise; the challenge is dropped when the request carries `X-Astrum-UI: true`),
  `ApiError` (404 `{"id":"*","message"}`, 400 `[{"id","message"}]` from `Validation`, 415, 500 with a log
  reference), `Json` extractor (415 without a JSON content type). Routes live in `src/routes/<area>.rs`,
  each `pub fn routes() -> Router<AppState>` with full paths under `/service/rest`. Audit events: domain
  per area (`security.user`, `security.role`, `security.privilege`, `security.realm`, `security.anonymous`,
  `security.contentselector`, `blobstore`, `repository`), kind `created|updated|deleted`, context = the
  object's name. Tests use `crates/astrum-api/tests/support` (`Fixture` with a migrated, bootstrapped
  database and the admin password).
- Sessions, tokens and audit (ours, internal API under `/service/rest/internal`):
  `GET|POST|DELETE /session` (JSON `{username, password}`; cookies `astrum_session` (HttpOnly) and
  `astrum_csrf`; session rows in `user_session` keyed by the SHA-256 of the token, ended when the password
  hash changes, the user can no longer sign in, or after `security.sessionTimeoutMinutes` idle (0 = never);
  unsafe requests through a session need header `X-Astrum-CSRF` equal to the cookie and a same-origin
  `Sec-Fetch-Site`; 10 failed sign-ins in 5 minutes pause the user's sign-in with 429),
  `GET|POST /access-tokens`, `DELETE /access-tokens/{id}` (`astrum_` + 40 characters, SHA-256 stored, scope
  READ/WRITE, 1–3650 days, at most 100 per user; usable as the Basic password of its owner or as a Bearer
  token while `AccessTokenRealm` is active; managed only after signing in with a password; other users'
  tokens need `astrum:accesstokens:read|delete`), `GET /audit?limit&before&domain` (`astrum:audit:read`).
