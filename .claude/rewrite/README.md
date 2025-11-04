# Rewriting Astrum Repository in Rust

Status: decided (owner's answers of 2026-10-12 are recorded below); M0, M1, M2 and M3 are done (2026-10-12);
M4 is done (2026-10-17): wave 1 (helm, nuget, tasks and cleanup, LDAP/OIDC/default role) and wave 2 with the
remaining own formats (see `briefs/M4-WAVE2-TRACKS.md`). M5 is done (2026-10-18): `astrum migrate` moved
production from the Java server, and https://astrum.globalart.hu runs on the Rust server since 2026-10-18 06:02
UTC. M6 is done (2026-10-18): the Java tree, the Maven and Yarn build, the Java image workflow, the upstream
attribution in NOTICE and, after the switch, the Java import (`crates/astrum-migrate*`) are gone, the docs
describe the Rust server, and the licence is MIT.
See "Where the Rust server stands" at the end for the code that exists.

## Why
Astrum Repository was a modified version of an open-source repository manager under the EPL-1.0 (the
upstream), and its web UI (Astrum One) grew out of the upstream preview UI;
`@globalart/astrum-react-shared-components` is a fork of the upstream component library. The owner wants
Astrum to carry no upstream code and no upstream attribution at all.
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
- Astrum stays compatible with the clients and tools of its existing interfaces:
  - the public REST API keeps its paths and JSON shapes (`/service/rest/v1/...`), so scripts,
    Terraform/Ansible providers and CI integrations written for it keep working;
  - client URLs keep the same layouts (`/repository/<name>/...`, Docker `/v2/...`);
  - the older URLs (`/content/repositories|groups|sites/...`, `/service/local/...`) keep working;
  - moving data from another server with the same REST API (`tools/repository-copy`,
    `docs/migrate/from-another-server.md`) stays.
  Compatibility is implemented from the public API documentation and black-box behaviour, never from
  upstream code. The upstream product and its vendor are not named anywhere in the tree (owner,
  2026-10-18).
- UI URLs are plain paths (`/browse/...`, `/admin/...`, `/settings`, `/user/...`, `/login`); old hash URLs
  (`#browse/...`, `#preview/...`, Classic URLs) redirect to them.
- The new web UI must look and behave exactly like the Astrum One UI on astrum.globalart.hu (owner,
  2026-10-12): same layout, spacing, fonts, colors, components, texts and behaviour on every page. It is
  reproduced from the running UI (screenshots, computed styles, behaviour) and our design tokens, never from
  the Astrum One code; a side-by-side QA report lists the differences to remove.
- After M3 the Rust server runs as a test instance at `next.astrum.globalart.hu` on the production VM
  (`prod1-astrum1`), next to the Java server, with its own database on `prod1-dbms1` and its own data
  directory; production stays on the Java server until M5.

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
3. The REST API must stay compatible with the existing one (paths and JSON shapes, see
   `inventory-server.md` and the public documentation of that API); implement it from that documented
   interface and black-box behaviour, not from the Java resources.
4. New English UI texts are written afresh. Russian texts are ours and can be reused.
5. No licence headers in files. The new code's licence goes in LICENSE once the Java tree is gone.

## Our own code (allowed as reference)
Written by us on top of the fork, from specifications or our own design. After M6 the `public/...` paths
below exist only in the git history before the commit "Remove the Java server and its build"
(`git log --diff-filter=D --oneline -- pom.xml`):
- Formats: yum, rubygems, terraform, huggingface, swift, gitlfs (`public/common/components/formats/
  astrum-repository-{yum,rubygems,terraform,huggingface,swift,gitlfs}`), and the OCI 1.1 work in the
  Docker module once merged (branch `format/oci`).
- `security/astrum-ldap-plugin`, `security/astrum-oidc-plugin`, `security/astrum-access-token-plugin`.
- `astrum-notifications-plugin`; download counters and usage dashboard
  (`astrum-repository-content/.../content/downloads`, `.../content/usage`); cleanup preview totals.
- `tools/i18n` (Ownlate tooling), `tools/repository-copy` (moving data from another server), the docs under
  `docs/`.
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
  tokens, content selectors, REST compatible with the existing API for security, repositories and blob stores, audit.
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
    the older URLs. LDAP, OIDC, notifications and the counters are ours and are ported from our code.
  - M4c Formats production does not use (likely p2, r, cocoapods, conan, conda, pub, ansible): only on
    request; the migrator reports repositories of a format the Rust server does not have yet.
- M5 Migration: `astrum-migrate` (from the Java Astrum and from other servers) against a copy of production data, side-by-side run with the Java
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
    directory is writable), 503 otherwise, as the public API documentation describes them; everything else 404.
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
  READ/WRITE, 1–3650 days or no expiry (`expiresInDays: null`), at most 100 per user; usable as the Basic password of its owner or as a Bearer
  token while `AccessTokenRealm` is active; managed only after signing in with a password; other users'
  tokens need `astrum:accesstokens:read|delete`), `GET /audit?limit&before&domain` (`astrum:audit:read`).

## M1 progress (2026-10-12)
- Repository configuration (`crates/astrum-repo/src/config`, `store.rs`, REST `routes/repositories.rs`): every
  format and type of the registry (59 combinations) can be created, read, updated and deleted with the
  documented JSON; 118 GET documents of the reference server were accepted unchanged by our POST. Writes take
  a transaction advisory lock and check names (ignoring case), blob store, group members (format, cycles,
  writable member), docker ports. Deleting a repository removes it from groups and clears a stale writable
  member. Secrets (proxy passwords, bearer tokens, signing keys and passphrases) are sealed and never
  returned; a PUT without them keeps them. `invalidate-cache` and `rebuild-index` answer but do nothing until
  M2. Left: SSRF check of remote URLs (with the HTTP client in M2), base URL setting, real sizes in the list,
  cleanup policy and routing rule names (M4b), uniqueness of docker `subdomain` and terraform `registryHost`.
- Blob stores (`crates/astrum-blob`, REST `routes/blobstores*`): `BlobStore` trait (put with sha1/sha256/md5,
  read, ranged read, exists, delete, totals, space, check); file store `content/<2 hex>/<2 hex>/<uuid>` with
  fsync + rename; S3 (aws-sdk-s3 on rustls/ring, multipart over 8 MiB, SSE/KMS, assume role, path style;
  expiration, signer, presigned URLs and failover buckets are stored but inert); group stores (writeToFirst,
  roundRobin; no REST yet); soft quotas; totals in `blob_store_metrics`; registry `state.blob_stores()`
  (`get`, `forget()` after config changes); the `default` file store is created once on a fresh installation
  (setting `blobstore.provisioned`). A store holding blobs cannot change path, bucket or prefix. S3 tests run
  an in-process s3s server (`astrum_testkit::s3`). Left: SSRF check of S3 endpoints, `test-connection`,
  soft delete and compaction, a metrics reconcile task, documentation of a lifecycle rule for aborted
  multipart uploads.
- Security REST (`routes/security/{users,roles,privileges,selectors,settings,validate}.rs`) and content
  selectors (`astrum-security/src/selector.rs`: CSEL with `format`/`path`, `== != =~ =^`, `&&`/`||` and
  `and`/`or`, parentheses; `=~` matches the whole string; regex features PostgreSQL would read differently
  are refused at save; `to_sql` gives a fragment with bind parameters, checked against the evaluator on
  PostgreSQL). Content selector privileges carry `format` and grant
  `astrum:repository-content-selector:<selector>:<format>:<repository>:<actions>`. SSRF settings live in
  setting `security.ssrf` (on by default; audit domain `security.ssrf`), enforced from M2 on. Deleting a user
  removes its role mapping, ends its sessions and deletes its access tokens. Deliberate differences from
  the reference server: changing any password (also one's own) needs `astrum:userschangepw:create` (the
  reference only lets full administrators do it); a caller can only add roles and privileges they hold to
  roles and users, and only create privileges whose permission they hold (no self-promotion); ids may not
  contain `:` or control characters; 400 instead of the reference's 500 for duplicates and self-deletion;
  SSRF settings need `astrum:*` like the reference but entries are validated. Not done: `PUT
  /security/jwt`, LDAP/OIDC lookups (M4b).
- Dev builds optimise `argon2` and `blake2` (`[profile.dev.package.*]`) so API tests stay fast.
- Agents building Rust share `/home/user/wt/m1-target` with `CARGO_INCREMENTAL=0` and only `--workspace`
  commands; the disk allowance is about 39 GB in total.

## M2 conventions (content engine, 2026-10-12)
- Reference: `reference/openapi-m2.json` (components, assets, search, upload specs, browse, routing rules,
  HTTP settings) and `reference/content-responses.txt` (raw hosted behaviour of the old server: 201 on PUT,
  409 on a second PUT under ALLOW_ONCE, 400 on a content type mismatch, ETag `"<sha1>"`, 206/416 ranges,
  304, 404 texts, 405 with `Allow`, component/asset/search JSON, raw upload spec).
- Tables: `component` (repository, format, namespace = the API's `group`, name, version, kind, attributes
  JSONB; unique per repository/namespace/name/version) and `asset` (repository, format, path with a leading
  `/`, kind, component, blob store + blob id, size, content type, sha1/sha256/sha512/md5, attributes JSONB,
  uploader, uploader IP, created, blob updated, last downloaded, download count, last verified, remote ETag
  and Last-Modified; unique per repository/path). Format attributes live under the format id in
  `attributes` (`{"maven2": {...}}`) and are returned as the asset's `<format>` object. Both tables cascade
  from `repository`; deleting a repository also deletes its blobs.
- `crates/astrum-content` (engine): `Content` (in `AppState::content()`; formats are registered in
  `crates/astrum-server/src/formats.rs`), `Ctx` (repository, security snapshot, caller, base URL, client IP;
  `require(View::Read|Browse|Add|Edit|Delete, path)` checks `astrum:repository-view:...` and content selector
  grants; group members are read with `ctx.member(..)`, which skips checks), trait `Format` (`handle` the
  whole `/repository/<name>/<path>` request, `get` an asset for group/search use, optional upload spec,
  upload and search keys), `store` (find, `put` with write policy and strict content type check, delete,
  components, list under a prefix), `serve` (ETag, Last-Modified, Range, 304, HEAD, Content-Disposition,
  download counting flushed every 10 s), `proxy::fetch` (max age per content/metadata, conditional
  revalidation, stale-on-error, negative cache, single flight, auto-block with backoff, Basic/Bearer remote
  auth, SSRF protection on DNS answers, IP literals and redirects, retries) and `proxy::request` for formats
  that must rewrite remote answers, `group::members|first|writable_member`, `fetch` (the generic
  hosted/proxy/group lookup). Content errors are `text/plain` with the status the old server used.
- REST in `crates/astrum-api/src/routes/components.rs`: components and assets (page of 10, opaque
  continuation token), search (`q`, `repository`, `format`, `group`, `name`, `version` with `*`, checksums,
  sort/direction, format keys declared by formats as JSON paths into attributes), `search/assets`,
  `search/assets/download` (302 for exactly one match), upload specs, multipart upload into temporary files
  handed to the format, HTML browse at `/service/rest/repository/browse/<repo>/<path>/`. Listings show what
  the caller may browse or read, including content selector grants per asset. Extra top-level routes of a
  format (Docker `/v2`) go through `astrum_api::router_with` from `formats::routes()`.
- Formats: one crate each, `crates/astrum-format-<id>` (raw is the model: `crates/astrum-format-raw`),
  integration tests in `crates/astrum-api/tests/<id>.rs` using `Fixture` plus a real client run documented in
  the report.

## M2 progress (2026-10-12)
Done: the content engine, raw, maven2, npm, pypi and docker (with OCI 1.1), the components, assets, search,
upload and browse API; 417 tests; a run of the binary against PostgreSQL served every format. Real clients:
mvn and Gradle 8.14, npm, pip and twine, docker, crane, oras and the OCI conformance suite (986 of 986).
Agent brief template for formats: `/home/user/wt/M2-BRIEF.md`.
- Engine: `store::write_blob` + `adopt` (write, verify, then record), `proxy::fetch_from` (the format resolves
  the remote URL, possibly on another host; credentials go only to the remote's own origin), `proxy::send`
  with a method and the caller's own Authorization replacing configured credentials, preemptive `false`
  sends credentials only after a 401.
- Format tokens (`astrum-security::format_tokens`, table `format_token`): NpmToken and DockerToken bearer
  tokens. npm sign-in ignores a stale token from `.npmrc` and is judged by the password.
- maven2: metadata and checksums are derived on every read, not stored (uploaded ones are validated and
  dropped); groups merge metadata and the archetype catalog. Left: gavec, `.index` generation.
- npm: documents are built from components; proxy tarball locations assume the npmjs layout. Left: audit
  (404), search endpoint `/-/v1/search`.
- pypi: PEP 503/592/629/658/691/700/714; hosted pages are rendered, not stored. Left: tokenized search.
- docker: path routing and `/repository/<name>/v2/`, upload sessions in `docker_upload`, referrers, mounts,
  sha512. Left: per-repository ports and subdomain routing, blob garbage collection (layers stay after
  deletes), REST deletes that cascade to tags and referrers, a Bearer challenge from the authentication
  layer (it answers with Basic before the format sees the request).

## M3 conventions (web UI foundation, 2026-10-12)
- Layout: `web/` (own `package.json` with pnpm 10.18 through corepack, `pnpm-lock.yaml`, `biome.json`;
  scripts `dev` (Vite with `/service`, `/repository`, `/v2` proxied to `ASTRUM_URL`), `build`, `lint`,
  `typecheck`, `test`, `i18n:check`, `e2e`). `src/app` (bootstrap, router, `area.ts`, `areas.ts`, `legacy.ts`),
  `src/shell` (header, menu, drawer, search panel host, theme, 404), `src/auth` (UI state, permissions, login,
  guard, session watcher), `src/api`, `src/ui` (component layer), `src/i18n`, `src/styles` (tokens copied from
  `astrum-brand.css`: dark by default, light, system; Radix `blue` is the Astrum violet; Geist self-hosted),
  `src/areas/<area>/` (pages, components, `strings/`, tests). `web/README.md` is the how-to for page authors.
- Area registration: `src/areas/<area>/index.ts` exports an `Area { id, routes, menu, settings, search }` and
  `src/app/areas.ts` lists it (one line). Routes `{ path, page | load, need, requiresUser }` live under the
  shell (`load` = lazy default export). Menu entries `{ id, to, label, icon, group: main|admin, order, need,
  visible, activeFor }` (orders: dashboard 10, search 20, browse 30, upload 40, API 50, settings 900). Settings
  tiles `{ id, to, need }` name an id of the catalog in `src/areas/settings/sections.ts`, which places them in
  the old sections (repository, security, support, system) and order with the old texts and icons (other ids
  bring `section, title, description, icon, order`); the hub hides sections without visible tiles and the
  Settings entry shows when any tile is visible. Search providers `{ id, order, need, Results({ query, close
  }), submit(query) }` render `SearchGroup`/`SearchItem` from `src/shell/search` in the dropdown under the
  header field (Ctrl+K/Cmd+K, arrows, Enter, Esc; a full-screen search bar on compact screens). Paths: `/`, `/browse[/<repo>/<path>]`, `/search`,
  `/search/custom`, `/search/component/...`, `/upload[/<repo>]`, `/audit`, `/settings`,
  `/admin/repository/<page>[/...]`, `/admin/security/<page>[/...]`, `/user/account`, `/user/access-tokens`,
  `/login?returnTo=`; `/admin`, `/admin/repository`, `/admin/security` redirect to their first page.
  `src/app/legacy.ts` maps old hash URLs onto these (e.g. `#browse/repository-profile/x` →
  `/admin/repository/repositories/x/profile`, `#admin/security/users:jane` → `/admin/security/users/jane`);
  change both together.
- Strings: `strings/{en,ru}.json` per area (and shell, ui, auth); keys `<area>.<page>.<name>` starting with
  the folder name; plurals as objects of `Intl.PluralRules` forms; `useT()`, `t(key, vars)`, `t.rich(key,
  nodes)`; keys written as literals. `pnpm i18n:check` fails on differing keys, plural forms or placeholders
  and on used keys missing from the dictionaries. Format names (`ui/formats.ts`) and the logo are the only
  texts in code. The language (cookie `astrum-language`, else the browser) switches without a reload. The web
  dictionaries are in Ownlate with the Java ones (keys `web.<key>`, plural forms `web.<key>#<form>`):
  `tools/i18n` checks them, `pull` writes the `ru.json` files and the `i18n` workflow fails when they differ.
- UI state: `GET /service/rest/internal/ui/state` (`routes/internal/ui_state.rs`) answers `version`,
  `revision`, `user` (`userId`, `name`, `email`, `source`; null for strangers and anonymous callers),
  `anonymousAccess`, `permissions` (the caller's permission strings plus content selector grants as
  `astrum:repository-content-selector:<selector>:<format>:<repo>:<actions>`), `formats` (`id`, `apiName`,
  `types` of the registered formats), `baseUrl` (from the request), `sessionTimeoutMinutes`, and
  `adminPasswordFile` while `<data>/admin.password` exists, only to strangers and holders of `astrum:*`.
  It is fetched before the router starts and on every return to the tab.
- Permissions: `src/auth/permissions.ts` matches like `astrum-security` `Permission`; a need is a permission
  string, `{ all }`, `{ any }` or `{ some: prefix }` (holds anything under the prefix). `useCan(need)`,
  `useSession()`, `Guard` (anonymous callers go to the login page, signed-in ones see "cannot open").
  Without anonymous access every shell page sends strangers to `/login?returnTo=`.
- API client: same-origin `fetch` with `X-Astrum-UI: true` and `X-Astrum-CSRF` from `astrum_csrf` on unsafe
  methods; `api.get/post/put/delete`, `` path`...${x}` ``, `withQuery`; `useApi(path)` (query key
  `['api', path]`), `useInvalidateApi()(prefix)`; `ApiError` (`status`, `message`, `fieldErrors`),
  `describeError(t, error)`. A 401 clears the user: with anonymous access a dialog offers to sign in again,
  without it the login page opens with the return address. Signing out reloads the page.
- Components (`src/ui`): `Page`, `DataTable` (sort, client pages or "show more", server "load more",
  skeleton, empty/error, row links and actions, cards on phones), `useForm` and fields (text, number, secret
  with `stored`, select, multi-select, checkbox, switch, textarea; 400 field errors land on their fields),
  `FormError`, `ConfirmDialog`/`useConfirm`, `useToast`, `CopyButton`, `CodeBlock`, `QueryTabs` (`?tab=`),
  `EmptyState`, `ErrorState`, `FormatBadge`/`TypeBadge`/`StatusBadge`, `FileSize`, `DateTime`,
  `RelativeTime`, `UnsavedChangesGuard`. Pages work at 390 px without horizontal scroll, in both themes.
- Tests: Vitest + Testing Library next to the code; `renderWithApp(element, { state, path, areas, routes })`
  and `fakeServer({ 'GET /path': handler })` in `src/test`. 148 unit tests at the foundation.
- End-to-end: Playwright in `web/e2e` with the preinstalled Chromium; every spec file gets its own server: the
  fixture starts `ASTRUM_BIN` on a free port with a fresh data dir and database (`ASTRUM_E2E_DATABASE_URL`,
  `ASTRUM_E2E_DATABASE_PREFIX`) serving `web/dist`, and drops both afterwards; `test`/`signIn` from
  `e2e/fixtures.ts`; `ASTRUM_SHOTS_DIR=<dir> pnpm e2e screenshots` takes 1440x900 and 390x844 shots in both
  themes. The shared target dir is rebuilt by every agent, so copy your binary before pointing `ASTRUM_BIN`
  at it.
- Serving (`crates/astrum-server/src/web.rs`): `web.dir` / `ASTRUM_WEB_DIR` / `--web-dir` (default
  `/usr/share/astrum/web`); GET/HEAD outside `/repository`, `/service`, `/v2`, `/content`, `/.well-known` get
  the file or `index.html`; `/assets/` cached for a year (a missing asset is 404), the rest `no-cache` with an
  ETag; CSP `default-src 'self'` without inline scripts (inline styles allowed for Radix), `frame-ancestors
  'none'`, `nosniff`, `Referrer-Policy: same-origin`. Without the directory `/` says the UI is not installed.
- CI and image: `docker/server.Dockerfile` builds `web/` in a node stage (frozen lockfile) into
  `/usr/share/astrum/web`; `server.yml` runs on `web/**` with a `web` job (install, lint, typecheck, test,
  i18n check, build) and the image job checks `/` and `/browse/x`; `image.yml` ignores `web/**`.

## M3 progress (2026-10-12)
- M3 UI API (`crates/astrum-api/src/routes/internal/ui/`, tests `tests/ui_*.rs`, shared helpers
  `tests/ui_support`): internal endpoints under `/service/rest/internal/ui`, camelCase JSON, showing only
  what the caller may see (whole repositories through repository-view browse/read, single assets through
  content selector grants; `Ctx::grant` in `astrum-content` says which, and the selectors' SQL translation
  filters aggregates). `GET /repositories` (every repository the caller may browse, read, administer or add
  to, with url, blob store, remote URL, visible members and containing groups, component/asset counts, size
  and last upload from one aggregate query (null where the content is hidden), `limited` when only
  selectors give access, and flags browse/read/add/edit/delete/admin/adminEdit/adminDelete);
  `GET /repositories/recipes` (registry format/type pairs whose format is served); `GET|DELETE
  /browse/{repository}?path=` (direct children of a folder: folders with child counts, a folder named after
  the version of its only component as a `component` node, assets with public API ids, size, content
  type, last modified and a download URL through the browsed repository; groups show the union of their
  members, first member wins; missing folders 404; the delete removes what the caller may delete under a
  folder through the content store, audits each asset and emptied component, refuses groups and the root,
  and answers `{deletedAssets, deletedComponents, skippedAssets}`); `GET /search/versions?format=&group=&name=`
  (newest first: numbers compared as numbers, pre-releases before releases); `GET /usage-dashboard?days=&repository=`
  (the overview of Astrum One, same JSON as the old server: visible repositories, totals with proxy cache and
  today's downloads, downloads per UTC day of the period from `asset_download_day` (per asset and day, filled
  by the download flush, kept 180 days), top 10 components in the period and of all time, repositories by
  downloads and by size, latest components, components not downloaded for 90 days; it replaced
  `GET /dashboard`); `GET /blobstores` (usage, quota state, users, groups, `deletable`);
  `POST /content-selectors/preview` (first 100 matches and `more`); `GET /privileges/types` (built from
  `PrivilegeKind::fields`, which the privileges API validates with); `GET|PUT /user` and
  `PUT /user/password`. Changing one's own password needs no `astrum:userschangepw:create`, a deliberate
  exception because the caller proves the current password (rate limited like sign-in; the other sessions
  end, the current one stays). Also new: audit filters `q`, `initiator`, `kind`/`type`, `context`,
  `repository`, `since`, `until` on `GET /service/rest/internal/audit`, and group blob stores
  (`POST /v1/blobstores/group`, `GET|PUT /v1/blobstores/group/{name}`: members must be existing plain
  stores in no other group and not used by repositories, members holding blobs cannot be removed, and
  repositories cannot be created on a group member).
- M3 pages (2026-10-12, merged from four parallel branches): dashboard (`/`), browse (`/browse`, tree and details
  at `/browse/<repo>/<path>`, folder paths end with `/`; asset details through
  `GET /internal/ui/browse/{repo}/asset?path=` judged by the browsed repository), component page
  (`/search/component/<format:group:name>?version=&tab=`), usage snippets for raw, maven2, npm, pypi and docker
  (`web/src/areas/browse/usage`), search (`/search`, `/search/custom` with format keys and a files mode, the
  header panel), upload (`/upload[/<repo>]`, forms from upload specs; npm now has an upload spec), audit
  (`/audit`, facets from `GET /internal/ui/audit/facets`), repository settings (list, create from recipes,
  edit with settings/membership/audit tabs; saving writes back only changed fields onto the stored document,
  checked against 118 reference GET documents), blob stores (file, S3, group), content selectors with live
  preview, privileges (forms from the privilege types), roles, users, anonymous access, realms, SSRF
  protection, account, access tokens, onboarding for administrators while `admin.password` exists. `/v1/search`
  sorts by name, version, group and repository and accepts `*` in `q`. `OrderedListField` and area
  `watchers` were added to the foundation. Tests at the end of M3: 487 Rust, 770 web unit, 52 e2e (every spec
  file gets its own server). The web dictionaries (1387 keys) are in Ownlate under the `web.` prefix.
- Audit log (parity work, 2026-10-13): `GET /internal/ui/audit-log` answers the old page's query (page, limit,
  startDate, endDate, comma lists of categories and types, repository, initiator and context text) with `items` and
  the old `pagination` object; categories group domains as the old page does (security, repository, protection,
  configuration for the rest; sign-ins and sign-outs are security). Audit initiators are `user/address` when the
  client address is known (X-Forwarded-For, X-Real-IP or the peer), and a failed UI sign-in is audited as
  `security.session failed`.
- Left after M3: idle-session countdown, response compression, docker port suggestion, repository profile
  tabs "structure", "security" and "system", tree filter and type-ahead, connection
  examples for formats added later, session dialog on a 401 during upload.
- Support (M4 wave 2, branch `m4-support`): crate `astrum-logging` writes the tracing output also to
  `<data>/log/astrum-<node>.log` (node `ASTRUM_NODE_NAME`, else the host name; 32 MiB, four rotations, files of
  other nodes pruned after 30 days) and reloads the filter at runtime; overrides live in setting `logging.levels`,
  applied at start and every 30 s. Endpoints: `GET /internal/logging/logs[/{file}?bytesCount=-N]`,
  `POST /internal/logging/log/mark`, `GET|PUT|DELETE /internal/ui/loggingConfiguration[/{name}]` plus
  `POST .../{name}/reset` and `.../reset` (names are Rust module paths or `ROOT`), `GET /v1/status/check` and
  `/internal/ui/status-check` (CPUs, disk, blob store quota/readiness, database and writes, admin password file,
  default role realm, log files, scheduler heartbeat `Tasks::last_tick`), `GET /service/rest/atlas/system-information`
  (no environment; the database password is masked), `POST /v1/support/supportzip[path]` and
  `GET /service/rest/wonderland/download/{file}` (own ZIP writer; `<data>/downloads`, pruned after a day),
  `GET /service/rest/swagger.json` (the routed subset of `crates/astrum-api/tests/data/openapi-reference.json`;
  the `openapi` test fails when routes and document differ, `ASTRUM_WRITE_OPENAPI=1` rewrites it). Pages under
  `/admin/support/...` and `/api`, the header bell shows a red dot while a check fails. Per node: logs, marks and
  support ZIPs live on the replica that served the request. The new UI loads no user-telemetry script (the old
  UI's 404 on every page); `e2e/support.spec.ts` fails on any failing request or console error on the main pages.
- Own formats (M4 wave 2, branch `m4-fmt-b`), ported from our Java modules: terraform
  (`crates/astrum-format-terraform`: module and provider registry protocols and the provider network mirror for
  hosted, proxy and group; `/.well-known/terraform.json` at the host root for the repository whose `registryHost`
  matches the request's host; package locations signed with an HMAC for 15 minutes, the secret in the settings
  table; SHA256SUMS written and signed with the repository key when uploads bring none; TerraformToken bearer tokens
  at `/repository/<name>/token`), huggingface (proxy only: revision info, trees and paths-info from the cached
  revision, files stored once per commit as `<type>/<repo>/<commit>/<file>` with the Hub's headers and
  `X-Error-Code`; search keys `huggingface.revision` and `huggingface.file`, and the revisions also match `q`
  through `Format::keyword_attributes`), swift (SE-0292 hosted and group: publish, releases, metadata, manifests,
  archives, identifiers, login; proxies answer 503). Web: usage blocks, the Terraform settings and signing
  sections, search keys and the old remote examples. Checked with terraform 1.16.2 and 1.9.8, huggingface_hub
  2.2.0 and replayed SE-0292 requests.
- Notifications, settings and nodes (M4 wave 2, track notify): `crates/astrum-notify`, ported from our
  `astrum-notifications-plugin`: channels (Telegram, Slack-compatible webhook, JSON webhook with
  `X-Astrum-Event`, `X-Astrum-Delivery` and an optional HMAC-SHA256 `X-Astrum-Signature`, and email) with
  repositories, events, message language and a proxy mode per channel; secrets sealed, audit domain
  `system.notifications`; REST `GET|POST /internal/ui/notifications`, `PUT|DELETE .../{id}`,
  `POST .../test` and `.../{id}/test` (`astrum:settings:read|update`). Triggers on `component` and `asset`
  capture changes in the writing transaction; one replica at a time (advisory lock) folds the buffer after
  10 s of quiet (60 s at most) into an outbox and delivers it with 4 attempts and backoff, so replicas
  neither lose nor double messages. Webhooks pass the SSRF protection for the host and every resolved
  address. Email goes through the `MailSender` trait (`Notifier::set_mail_sender`); without one, email
  deliveries fail with "Email is not set up". `ASTRUM_NOTIFICATIONS_TELEGRAM_API_URL` points Telegram at
  a local server in tests. Base URL, UI settings and branding are plain settings (`system.baseUrl`,
  `ui.settings`, `ui.branding`, audit domain `capability`) served under the capability paths of the existing API
  (`/v1/capabilities[/types|/{id}]`, types `baseurl`, `rapture.settings`, `rapture.branding`); a set base
  URL wins over the request's address (cached a few seconds per replica); the UI state carries the title
  and the branding snippets, which the shell sanitizes against an allow list. Every replica writes a
  heartbeat into `node` (`GET /internal/ui/nodes`). Pages `/admin/system/notifications`,
  `/admin/system/capabilities[/create|/<type>]`, `/admin/system/nodes` match Astrum One; the capability
  list offers only the three types above, and email is a fourth messenger the old page did not have.
- Outbound (M4 wave 2, 2026-10-17): routing rules (`astrum-repo::routing`, table `routing_rule`; public
  `/v1/routing-rules` with the existing shape, renames move repositories along, a rule in use cannot be deleted;
  internal `/internal/ui/routing-rules` list/get, `test` (per-matcher, whole-path match) and `preview` tree;
  proxies and groups, also through docker `/v2/<repo>/...` path routing, answer a blocked path with 403 and
  the reference text, groups skip blocked members; the API accepts `routingRuleName` on groups, the form offers
  it only for proxies like the old one). `astrum-outbound`: global HTTP settings (setting `http.client`,
  `GET/PUT /v1/http`, passwords sealed and masked as `#~ASTRUM~PLACEHOLDER~PASSWORD~#`, used by every proxy
  client: user agent, timeout, retries, HTTP/HTTPS proxies with Basic auth (NTLM fields stored, not spoken) and
  non-proxy host globs; proxy hosts pass the SSRF guard, proxied targets are still checked when they resolve),
  the SSL truststore (table `ssl_certificate`, `GET /v1/security/ssl?host&port&protocolHint` through the SSRF
  guard with SMTP STARTTLS for 25/587, `GET/POST /v1/security/ssl/truststore` (PEM text), `DELETE .../{id}`,
  internal `POST /internal/ui/ssl/certificate-details`; a stored certificate is trusted as an authority and,
  like a Java truststore, as the server's own leaf, names are still checked; used by proxies with
  `useTrustStore` and by mail). Saving SSRF settings now reaches proxies at once. `astrum-mail`: email settings
  (setting `email`, `GET/PUT/DELETE /v1/email`, `POST /v1/email/verify` with the address as the body, 400 with
  `{success:false, reason}` on failure) and an SMTP client of our own (SSL on connect, STARTTLS
  optional/required, AUTH PLAIN/LOGIN, identity check, truststore); other code sends mail with
  `state.mailer().send(&astrum_mail::Message { to, subject, text })` (`Mailer::new(outbound)` outside the API;
  `MailError::Disabled` while email is off). Web areas `routing` and `outbound` (System -> HTTP, Email Server;
  Security -> SSL Certificates with add, details and a certificate dialog also used by the proxy form),
  matched to the old pages by measurement; e2e `web/e2e/routing.spec.ts` and `outbound.spec.ts` (local proxy,
  SMTP sink and HTTPS server).
- Yum (M4 wave 2, `crates/astrum-format-yum`, ported from our Java module): hosted repositories generate
  `repodata` (primary, filelists, other as gzip plus `repomd.xml`, like createrepo_c) at `repodataDepth`
  directories, rebuilt after each package change (debounced, PostgreSQL advisory lock per base) and checked
  against a fingerprint of the packages on every `repomd.xml` read, so deletes through the REST API, browse
  or cleanup also show; the previous generation of files is kept for clients mid-fetch; `deployPolicy`
  STRICT/PERMISSIVE; `yumSigning` signs `repomd.xml.asc` (rpgp) and serves `repomd.xml.key`; proxies cache
  metadata and packages and mark cached repodata stale when `repomd.xml` lists other checksums; groups merge
  member repodata (gz/xz/bz2/zstd, first location wins) under a merge key and pass a single member through;
  upload spec with `directory`, search keys `yum.name|version|release|architecture`, the visible task
  `repository.yum.rebuild.metadata`, cleanup pre-releases (`~` in the version or a release starting `0.`).
  Real clients: dnf 4.24 (hosted, depth 2 with `repo_gpgcheck`, proxy, group), createrepo_c output compared.
- RubyGems (M4 wave 2, `crates/astrum-format-rubygems`, ported from our Java module): hosted repositories take
  `gem push` (`POST /api/v1/gems`) and `gem yank` (`DELETE /api/v1/gems/yank`) and answer the compact index
  (`/versions`, `/names`, `/info/<gem>`, ETag = MD5 as Bundler expects), the legacy Marshal indexes
  (`specs`, `latest_specs`, `prerelease_specs.4.8.gz`), quick gemspecs and the dependency API, all built on
  request from the stored gems (the Java module stores them as assets; building them keeps deletes through the
  REST API, browse or cleanup visible at once); proxies cache indexes as metadata and gems/gemspecs as content
  (cached gems get their specification and component); groups merge `/versions` like the Java module (names in
  one member pass through, shared names are merged from `/info`) and keep the merged `/versions` and specs as
  assets of the group under a signature of the members' files. `GET|POST|DELETE /api/v1/api_key` gives each
  user one kept key `RubyGemsApiKey.<secret>` (table `api_key`, sealed) while the realm is active; Basic
  credentials work as a key too. Upload spec `asset`, search keys `rubygems.platform|summary|description`,
  browse by gem name and version with the index files at the root, cleanup pre-releases (a letter in the
  version). Real clients: gem 3.5.22 (signin, push, yank, install from hosted, proxy, group and a proxy of the
  Java server) and Bundler 4.0.18 (install through a group, as a mirror of rubygems.org, incremental update).
- Git LFS (M4 wave 2, `crates/astrum-format-gitlfs`, ported from our Java module; hosted only, as there): the
  batch API (basic transfer, sha256, at most 1000 objects, per-object errors and permissions: read to download,
  add to upload), upload `PUT .../objects/<oid>` checked against the SHA-256 (an object already held is kept),
  verify, download by id or at the asset path `/objects/<2>/<2>/<oid>`, all under `<repository>/info/lfs` or
  the repository root; the lock API (create with the "already created lock" conflict, list with path, id,
  cursor and limit, verify ours/theirs, unlock, `force` needs delete) in table `gitlfs_lock` (one lock per path,
  dropped with the repository); API errors are `application/vnd.git-lfs+json` `{message}`, a stranger gets the
  Basic challenge git-lfs needs. Objects are components named by their id; upload spec `asset` stores a file
  under its SHA-256; browse by asset path; cleanup by age and last download. Real client: git-lfs 3.4.1 (push,
  clone, fetch, lock/locks/unlock, `locksverify` refusing another user's locked file).
- Compat (M4 wave 2, branch `m4-compat`): the older URLs `/content/repositories|groups|sites/<name>/...`,
  `/service/local/repositories/<name>/content/...` and `/service/local/repo_groups/<name>/content/...` go through
  the `/repository/<name>/` dispatch (always on). Docker connectors: one listener per `httpPort`/`httpsPort`
  (refreshed after REST changes and every 10 s, so every replica follows; HTTPS needs
  `ASTRUM_CONNECTOR_TLS_CERTIFICATE`/`_KEY`), `/v2/` there and on a host starting with a repository's `subdomain`
  (Host or X-Forwarded-Host) belongs to that repository; ports, subdomains and Terraform registry hosts are unique;
  `GET /internal/ui/docker/suggest-port?exclude=` and the form's "Suggest available port". Saving a proxy remote URL
  or an S3 endpoint runs the SSRF guard of outgoing requests with the old messages (an unchanged value is not judged
  again; a host that does not resolve is accepted); the repository forms show refused saves like Astrum One (banner on
  create, red text in the button bar on edit, error toast). `GET /v1/repositories` reports `size` in bytes of the
  repository's own assets. `POST /v1/blobstores/test-connection` takes the S3 body and answers 204 or the old 400
  "Connection failed, ...". npm proxies forward `npm audit` (`/-/npm/v1/security/...`) to their remote, groups to
  their first online proxy member, hosted answers 400. `/v1/search` `q`: space-separated alternatives, each matched as
  a substring (`*` for any characters) or as a sequence of words split at non-alphanumerics, the last one a prefix
  (`report_builder` finds `report-builder`). Tasks `repository.maven.publish-dotindex` and `unpublish-dotindex` wrote
  and removed `.index/` (Maven Indexer data format 1) in hosted Maven repositories; they were removed on
  2026-10-18 because that format names its files after the upstream product (proxies still pass `.index/`
  through). Docker blob garbage collection is
  the `repository.docker.gc` task of wave 1.
- Migration from the Java Astrum (M5, track config, `crates/astrum-migrate`; removed in M6 after the switch, in
  the history before "Remove the import of the Java Astrum installation"): `astrum migrate --source-data <java
  data> [--source-database URL] [--source-schema] [--source-secrets] [--dry-run] [--force] [--report FILE]` (without a
  source it still only migrates the schema). Preflight: Flyway 2.166 with known, successful steps only, empty
  `upgrade_tasks`, every row of `secrets` opens (PBKDF2-SHA1/256 + AES-CBC, HMAC checked; cipher password from
  `etc/astrum.properties` or `ASTRUM_SOURCE_CIPHER_PASSWORD`, keys from secrets.json); the Java database is opened
  read-only (`default_transaction_read_only`), a target that is the Java schema or a data directory overlapping the Java
  one is refused, a non-empty target needs `--force` (drops the Rust schema). Objects are mapped through the existing
  validators (`Repository::prepare`, routing `validate`, cleanup `validate`, `EmailRequest`/`HttpSettings::prepare`,
  notification `validate`, `Schedule::from_request`), secrets re-sealed with the Rust secret box; Quartz `JobDataMap`s
  are read by `javaser` (Java Object Serialization Stream Protocol). Moved: blob stores (file directories created, S3,
  groups, quotas), routing rules, cleanup policies (seconds to days), content selectors, repositories (formats not served
  go to the setting `migration.java.unsupportedRepositories` with sizes), users with their hashes (`astrum-security`
  now also verifies PBKDF2 PHC and unsalted SHA-1/MD5 hex, all rehashed to argon2id on sign-in), privileges, roles,
  role mappings (default, LDAP, OAuth2), realms, anonymous, LDAP servers, OIDC settings and users, NuGet/RubyGems keys
  (`api_key`), npm/Terraform tokens (`format_token`), access tokens, base URL, UI settings, branding, default role,
  email, HTTP, truststore, SSRF, ROOT log level, notification channels, seven task types with ids and schedules, audit
  history, deployment/node ids; then the built-in roles, store and tasks are added. `migrate_content` only writes
  `content: not migrated in this build` into the report (the content track replaces its body). Report as text and
  JSON (`Report`: counts per kind, skipped with reasons, warnings, unsupported repositories, `content`). Tests:
  `crates/astrum-migrate/tests/java_astrum.rs` on a sanitized dump of the reference configuration tables
  (`tests/data/astrum-2.0.0.sql`). Operator guide: `docs/migrate/from-java-astrum.md`.
- Content migration (M5, track content, `crates/astrum-migrate-content`): `migrate(&Context { source: &PgPool,
  source_data_dir, target, blob_stores, dry_run }) -> ContentReport` (serde; counts per repository and format,
  not migrated repositories with sizes, skipped assets with reasons, warnings; `text()`), called by `astrum migrate`
  after the configuration step. It reads `<fmt>_content_repository|component|asset|asset_blob`, the download
  counters, `gitlfs_lock` and `swift_package_url`, resolves blob references (date based and legacy volume/chapter
  layouts, file, S3 and group stores) and adopts each blob under its old id with `FileBlobStore::adopt_file` (hard
  link, copy across filesystems) or `S3BlobStore::copy_in` (server-side copy; `put_as` and `read_key` move between
  file and S3), checking size and SHA-1. Each format crate has an `import` module (`plan` = take over, derived or
  skip; `describe` = the `PutAsset` the format writes itself, using `astrum_content::import::Imported`), so what
  the Rust server derives (Maven checksums and hosted metadata, hosted index pages, repodata, gem indexes, Helm
  and NuGet hosted indexes) is rebuilt, not copied. Idempotent per asset (same blob id = unchanged; generated files
  get ids derived from the source blob), so a second run takes over only what changed. Tests: synthetic Java rows for
  every format, S3 (s3s fake), groups and broken blobs; `tests/reference.rs` migrates a copy of the reference server
  when `ASTRUM_MIGRATE_SOURCE_URL`, `_SOURCE_DATA`, `_REFERENCE_URL` and `_REFERENCE_PASSWORD_FILE` are set and
  compares every asset byte for byte; `examples/migrate_content.rs` runs the step alone. Operator page:
  `docs/migrate/from-java-astrum-content.md`.
- Cargo (M4c, `crates/astrum-format-cargo`): sparse index (`config.json` with `dl`, `api` and `auth-required` when
  the repository needs sign-in or anonymous users cannot read it; index files `1/`, `2/`, `3/<c>/`, `<ab>/<cd>/`
  built from the components, with `pubtime`, `features2` and `v: 2` for `dep:`/`?/` features), `PUT
  api/v1/crates/new`, yank/unyank, search, owners (read; changes answer that privileges manage them), downloads at
  `api/v1/crates/<n>/<v>/download` and the stored path `crates/<n>/<v>/<n>-<v>.crate`, `/me` giving one kept
  `CargoToken.<uuid>` per user while the realm is active (409 otherwise, as the reference); raw `Authorization`
  values may be such a token, an access token or `Basic ...`. Proxies cache index files as metadata and crates
  through the remote's `dl` template (crates.io: `https://index.crates.io`); groups merge index files by version
  and publish through the writable member. Web: the Cargo Settings section, usage (config.toml or source
  replacement for proxies, cargo add, Cargo.toml, cargo publish), the component snippet. Checked with cargo 1.98.
- Go (M4c, `crates/astrum-format-go`): GOPROXY `@v/list`, `.info`, `.mod`, `.zip`, `@latest` for hosted, proxy and
  group, assets under the case-encoded paths like the reference (component names decoded); hosted takes `PUT
  <module>/@v/<version>.zip` and the upload spec (module and version read from the zip when empty) and stores the
  zip's `go.mod`; groups merge lists and pick the latest. `astrum-content::zip` (a small zip reader and writer) is
  shared with the formats below. Web: browse puts the files of a version under `<module>/<version>/`, usage (go get,
  go env with GONOSUMDB for hosted, go.mod, upload). Checked with go 1.24.7 (`go mod tidy` through a group of a
  hosted repository and a proxy of proxy.golang.org).
- APT (M4c, `crates/astrum-format-apt`): hosted repositories take a `.deb` as a raw `POST` to the repository root
  (also with the `multipart/form-data` header of the old usage snippet, or a real multipart form) and through the
  upload spec, store it at `pool/<letter>/<package>/<package>_<version>_<arch>.deb` (component group = architecture)
  and write `dists/<distribution>/main/binary-<arch>/Packages`, `.gz`, `.bz2`, `Release`, `InRelease` and
  `Release.gpg` (signed with `aptSigning`) as assets, like the reference; the metadata is rebuilt after uploads and
  deletes and whenever a `dists/` read finds a different fingerprint of the packages (advisory lock per repository),
  so REST, browse and cleanup deletes show too. Release section headers carry no trailing space (a clearsigned
  file with one fails gpg). Proxies cache `dists/` as metadata and packages as content with a component from the
  file name. Task `repository.apt.rebuild.metadata` (full rebuild for hosted, reset of cached metadata for proxies);
  cleanup knows pre-releases (`~` in the version). Web: the APT Settings and APT Signing sections, browse under
  `metadata/` and `packages/<letter>/<name>/<version>/`, usage (sources.list, apt-get, upload). The UI repository
  list carries `distribution`. Checked with apt 2.8.3 (`apt-get update` and `download` with signed-by, through the
  hosted repository and a proxy of it).
- Composer (M4c, `crates/astrum-format-composer`): `packages.json` (`metadata-url`, `providers-url`, plus
  `available-packages` on hosted and `list` on proxies, like the reference), `p2/<vendor>/<project>[~dev].json` in
  the minified composer/2.0 form, `p/<vendor>/<project>.json`, `packages/list.json`, dists at
  `<vendor>/<project>/<version>/<vendor>-<project>-<version>.zip` (sha1 as reference and shasum). Hosted takes `PUT
  packages/upload/<vendor>/<project>/<version>` and the upload spec (asset fields vendor, project, version) and reads
  `composer.json` from the top or the only folder of the zip; the metadata is built from the components on every read
  and copies kept under `p/` and `p2/` for browse are refreshed when they differ. Proxies keep the remote's
  `packages.json` and `p2` files and rewrite each dist address to themselves, fetching the original on first download;
  groups merge versions (first member wins). Web: usage (composer config, composer require, composer.json, upload),
  the component snippet, the packagist remote example. Checked with Composer 2.8.12 (`composer install` with
  packagist.org disabled through a group of a hosted repository and a proxy of repo.packagist.org).
- Conan (M4c, `crates/astrum-format-conan`): the Conan 2 REST API (`v1/ping` with the `revisions` capability,
  `v2/users/authenticate` issuing a 30-day `ConanToken.…` bearer token while the realm is active (403 with the
  reference text otherwise; a top-level route because it needs the security service), `check_credentials`, recipe and
  package `revisions`, `latest`, `files`, file GET/PUT/DELETE, package `search` from `conaninfo.txt`, `v2/conans/search`)
  for hosted, proxy (V2 only; V1 proxies answer 404) and group. Files live at
  `<name>/<version>/<user>/<channel>/<rrev>/export/<file>` and `.../package/<pkgid>/<prev>/<file>` like the reference
  (component group = `user/channel`, empty for `_/_`), with the asset attributes behind the search keys
  `conan.baseVersion|channel|revision|packageId|packageRevision`; checksum deploy answers 404 so clients send the
  file. Proxies cache the JSON answers under their API path and files under the hosted layout; groups merge revision
  lists newest first. Web: the Conan section of proxy forms (protocol version fixed after creation), browse without the
  `_/_` segments, usage (conan remote, install, conanfile.txt, upload), search criteria. Checked with Conan 2.32.0
  (`conan upload`, `install`, `search`, `list` against hosted, and `install` through a group of a proxy).
- CocoaPods (M4c, `crates/astrum-format-cocoapods`): proxy only, like the reference, of a CocoaPods CDN source
  (`CocoaPods-version.yml`, `all_pods.txt`, `all_pods_versions_<a>_<b>_<c>.txt`, `deprecated_podspecs.txt`,
  `Specs/<shards>/<pod>/<version>/<pod>.podspec.json`); index files and podspecs are cached as metadata (trunk
  deprecates podspecs by rewriting them, and redirects them to jsDelivr, which the proxy follows). A podspec whose
  `source` is an HTTP(S) archive is served with `source.http` pointing at `pods/<pod>/<version>/<file name of the
  original address>`: only the `source` member is replaced, the rest stays byte for byte, and its `headers` are
  sent upstream by the proxy instead of the client. The cached podspec stays the remote's copy, so the original
  address is read from it when the archive is first fetched (the podspec path comes from the cached
  `prefix_lengths`) and kept as `cocoapods.sourceUrl` on the archive asset; Git sources stay as they are.
  Podspecs and archives form the component (pod, version); browse shows `<pod>/<version>/` with the index files at
  the root; cleanup knows pre-releases (a letter or `-`). Web: usage (Podfile, `pod repo add-cdn`, `~/.netrc`, the
  only way the client signs in), the component snippet, the trunk remote example. Checked with CocoaPods 1.17.0 on
  Linux (`pod repo add-cdn`, `spec which`, `spec cat`, `repo update` with ETags, `pod install` of HTTP-source pods
  with sha256, anonymously and with `~/.netrc` credentials) against a local CDN that redirects podspecs to a
  second host.
- Side by side (M5, branch `m5-sbs`): `tools/side-by-side/` (Python standard library plus the real clients)
  runs the same scenarios against the Java reference and a Rust server and compares status, the headers
  clients use and normalized bodies; a difference fails the run unless `allow-list.json` names it with a
  reason (`accepted`, or `open` for the owner). `run` creates `m5s-` objects on both servers (REST, access,
  every served format, proxies with `--allow-loopback`); `existing` only reads and compares migrated data
  (listings, every download by SHA-256, index documents). On 2026-10-18: 849 steps plus 69 proxy steps, none
  unexplained. Fixed on the way: REST page sizes (100, search 50), version sort, empty Maven classifier,
  search download and sign-in, `If-Match`/`If-Unmodified-Since` 412, uploader address behind a proxy, raw
  query settings only on proxies, Swift `Link` headers, Terraform upload field, `routingRule` alias, Docker
  `registryUrl`, group listings with their members' content. `tools/repository-copy` now also copies into the
  Rust server, which makes it the copy-back of a rollback. The production switch and its rollback:
  `docs/migrate/switch-production.md` with the gitops and infra patches next to it.
- Conda (M4c second group, `crates/astrum-format-conda`): channels under `/repository/<name>/`, optionally below
  a channel path (`main/linux-64/...`); valid subdirs are `noarch` and `<platform>-<arch>`. Hosted takes `PUT
  <channel>/<subdir>/<name>-<version>-<build>.conda|.tar.bz2` (the documented upload layout) and the
  upload spec (asset plus optional component field `channel`; the subdir comes from the package), reads
  `info/index.json`, `about.json` and `run_exports.json` from the tarball or from the `info-*.tar.zst` inside a
  `.conda`, refuses a file name or subdir that does not match the index, and stores the package with component
  group = channel and the index under the `conda` attributes. Per channel it keeps `repodata.json` (records =
  index + md5, sha256, size; `packages` and `packages.conda`, `repodata_version` 1), `.bz2`, `.zst`,
  `current_repodata.json` (every build of the newest version per name, conda version order in `version.rs`)
  and `channeldata.json` as assets, rewritten under an advisory lock when a fingerprint of the channel's
  packages differs (checked on every index read, after uploads and deletes); a subdir without packages answers
  an empty index generated on the fly. Proxies cache index files as metadata and packages as content (component
  and `subdir`/`build` from the path); groups pass one member through and otherwise merge members' `repodata.json`
  per subdir with a raw JSON scanner (`raw.rs`, no tree, first member wins per file name), derive `.bz2`, `.zst`
  and `current_repodata.json` from the merge and merge `channeldata.json`, all kept as group assets under a
  merge key of the members' SHA-256; `info.base_url` of members is not honoured. `repodata.jlap` and sharded
  repodata answer 404. Index files carry `Cache-Control: no-cache, max-age=0` (micromamba otherwise never
  revalidates a copy without max-age). Search keys `conda.subdir|build|license`, task
  `repository.conda.rebuild.metadata` (hosted rewrite, proxy reset), no cleanup pre-release rule (the old server
  had none). Index files are not filtered by content selectors (like APT and Yum). Web: usage (conda install,
  .condarc, micromamba, upload), the component snippet, the conda-forge remote example, the translated channel
  upload field, search criteria. Checked with micromamba 2.9.0 (`create` from hosted with `.conda` and
  `.tar.bz2`, anonymous and signed in, and through a group of a hosted repository and a proxy of a local static
  channel).
- R (M4c, `crates/astrum-format-r`): the CRAN-like layout (`src/contrib`, `src/contrib/Archive/<package>`,
  `bin/windows/contrib/<R version>`, `bin/macosx/[<platform>/]contrib/<R version>`), each directory with
  `PACKAGES`, `PACKAGES.gz` and `PACKAGES.rds`. Hosted repositories take `<package>_<version>.<tar.gz|tar.bz2|
  tar.xz|zip|tgz>` by a PUT to its path and through the upload spec (`r.asset`, optional `r.asset.pathId`: a
  directory or a full path; source packages default to `src/contrib`, Windows zips to the tree of their `Built`
  field), check `DESCRIPTION` against the file name, folder and tree, and keep it as the asset's `r` attribute
  (component = name and version, no group). The index files of a directory are written like
  `tools::write_PACKAGES` (latest version per package, standard fields plus `MD5sum`, `write.dcf` wrapping) and
  kept as assets under a fingerprint of the packages that every index read checks; `PACKAGES.rds` is written in
  the R serialization format (version 3, gzip). Proxies cache index files as metadata and packages as content;
  groups merge the members' `PACKAGES.gz`, first member wins per package and version, cached in memory under a
  signature of the members' index assets. Task `repository.r.rebuild.metadata`. Not written:
  `src/contrib/Meta/archive.rds` (`remotes::install_version` sees only listed versions). Checked with R 4.3.3
  (`install.packages` with dependencies through a group of a hosted repository and a proxy; the hosted index
  files equal `write_PACKAGES` output, `PACKAGES.rds` `identical()` after `readRDS`).
- P2 (M4c second group, `crates/astrum-format-p2`): proxy only, like the reference. Client paths `p2.index`,
  `content|artifacts.{jar,xml,xml.xz}`, `compositeContent|compositeArtifacts.{jar,xml}` and files in `features/`,
  `plugins/`, `binary/` at any location (other paths 404, folders answer a short HTML page). Metadata is cached as
  `metadata` assets; composite and artifact repository files are rewritten once per remote version (`p2.checked`,
  set only while the blob is the one that was read): children are written relative to the composite, children
  outside the remote (same host or another) become `.remote/<scheme>/<host[:port]>/...` and are recorded in
  `p2.remoteChildren`, and a `.remote` path is fetched only under such a recorded child, through the SSRF guard and
  without the remote's credentials; `p2.mirrorsURL`, `p2.mirrorsBaseURL` and `p2.statsURI` are dropped from the
  repository properties (size fixed, the rest kept byte for byte); rewritten xz files hold uncompressed LZMA2 chunks
  (lzma-rs has no real encoder). Content metadata passes through unchanged, `references` included. Features,
  plugins and binaries are `artifact` assets with a component (group = folder; name and version from the artifact
  descriptor of the location's cached artifacts metadata, else the file name split at the first `_` followed by an
  OSGi version), the descriptor's properties and `pluginName` (`Bundle-Name` or the feature `label`, localized from
  the jar) behind the search key `p2.pluginName` and `q`; browse shows them under `<folder>/<id>/<version>/`. Web:
  usage (Eclipse software site, Tycho `<layout>p2</layout>`, p2 director), the component snippet (director and a
  `.target` unit), the plugin name criterion, the Eclipse release as remote example. Checked with the p2 director of
  Eclipse 4.31 and Tycho 4.0.9 through a proxy of a composite of two repositories made by the p2 publisher (one child
  on another host; the control run straight against the remote fetched the mirror list and the statistics URI, the
  proxied one did not), and with replayed client requests that included the 2024-03 release `artifacts.xml.xz`.
  Recent p2 turns `http://` into `https://` unless `-Dp2.httpRule=allow` is set.
- Alpine (M4c second group, `crates/astrum-format-alpine`): hosted repositories take `.apk` packages (v2 format)
  by `PUT <directory>/<name>-<version>.apk` and through the upload spec (asset plus optional `directory`, default
  the architecture), stored only under the name apk asks for and in the directory of their architecture (`noarch`
  takes the directory's, like abuild's `--rewrite-arch`); the `.PKGINFO` fields are kept as asset attributes and
  every directory with packages gets an `APKINDEX.tar.gz` (DESCRIPTION and APKINDEX in the field order of `apk
  index`, `C:` = SHA-1 of the compressed control stream), rebuilt after uploads and deletes and whenever a read
  finds another fingerprint of the packages (advisory lock per directory). `alpineSigning` (PEM RSA key, PKCS#1 or
  PKCS#8, also encrypted: PBES2 with PBKDF2 and AES/3DES-CBC, or the legacy `DEK-Info` form; our own small DER
  reader, since the `pkcs5` crate is not in the lock) signs like abuild-sign: a cut tar with
  `.SIGN.RSA.<repository>.rsa.pub` (SHA-1 PKCS#1 v1.5, blinded) gzipped in front of the index, the public key at
  `/<repository>.rsa.pub`; the UI repository list carries `alpineKey`. Proxies cache indexes as metadata and
  packages as content (attributes read lazily); with a key they put their own signature stream in front of the
  remote's (cached per index blob), so either key verifies. Groups merge member indexes (first record per
  name/version/architecture wins) and sign with their own key; without one a single member's index passes through
  with its signature and a merge of several stays unsigned. Task `repository.alpine.rebuild.metadata` (rebuild
  hosted, drop cached and merged indexes), cleanup pre-releases `_alpha|_beta|_pre|_rc`. Web: Alpine Signing section
  for all types, usage (key download, `/etc/apk/repositories`, `apk add`, upload) and the component snippet. Checked
  with apk-tools 2.14.4 and 3.0.8 (`apk.static`: `update`, `add`, `fetch`, `upgrade` against hosted, a proxy of real
  Alpine v3.23 files signed with the Alpine key, signed and unsigned groups; a client without the key gets
  "UNTRUSTED signature"); the index of real Alpine packages is rendered byte for byte like Alpine's own.
- pub (M4c second group, `crates/astrum-format-pub`; the Java server never shipped it, so there is no
  inventory row: written from the Hosted Pub Repository Specification version 2 and the `dart pub`
  documentation) for hosted, proxy (of `https://pub.dev`) and group. `api/packages/<package>` answers the
  `application/vnd.pub.v2+json` listing (versions in pub_semver order, `latest` = newest live release, else
  newest pre-release, `archive_url` back at the repository, `archive_sha256`, `published`, `retracted`,
  `isDiscontinued`/`replacedBy`), plus `.../versions/<version>`, `.../advisories`,
  `api/package-name-completion-data`, archives at `packages/<package>/versions/<version>.tar.gz` (stored
  there; `api/archives/<package>-<version>.tar.gz` is an alias) and an HTML page at the root. Publishing:
  `GET api/packages/versions/new` (signed in, add privilege) gives `api/packages/versions/upload`; the
  multipart upload (field `file`, 100 MiB) is checked at once (gzip tar with `pubspec.yaml` at its top,
  lower-case Dart identifier, semantic version, write policy: redeploy only under ALLOW with edit, an
  identical archive is accepted again) and answers 204 with a Location of `.../finalize?upload=<outcome
  sealed with the secret box>`, so any replica answers the finalize with the success or error JSON that
  `dart pub publish` prints; 401/403 come back at once. The pubspec is kept as JSON in the component
  attributes (`pub.pubspec`, `description`, `retracted`, `discontinued`, `replacedBy`); assets carry
  name, version, description, sdk, homepage and repository. Options of pub.dev: `PUT
  api/packages/<p>/options` (`isDiscontinued`, `replacedBy`) and `.../versions/<v>/options`
  (`isRetracted`), edit privilege. Credentials: access tokens as Bearer (what `dart pub token add` stores)
  or Basic; a 401 carries `WWW-Authenticate: Bearer realm="pub", message="<where to create a token>"`
  before the Basic challenge, a 403 the Bearer challenge only (the client keeps its token then). Listings
  are filtered by content selectors per archive path. Proxies cache listings, advisories and completion
  data as metadata (requested with the v2 Accept header), rewrite `archive_url` and fetch each archive
  from the address the remote listed (other hosts without the remote's credentials), deleting one whose
  SHA-256 differs from `archive_sha256` (502). Groups merge listings (first member wins per version,
  package flags of the first member that has the package) and advisories; they take no publishes. Upload
  spec `asset`, `q` also matches the description, browse `<package>/<version>/<package>-<version>.tar.gz`,
  cleanup pre-releases (a hyphen before any `+`). Web: usage (dart pub token add, pubspec.yaml `hosted:`,
  dart pub add with a descriptor, publish_to for hosted, PUB_HOSTED_URL for proxies and groups), the
  component snippet, the Dart logo, the pub.dev remote example. Checked with the Dart SDK 3.13.5 cut off
  from the internet: `dart pub token add`, `dart pub publish` (success, the same archive again, a changed
  archive refused with the server's message, no token: the 401 message), `dart pub get`/`run` of an app
  through a group of the hosted repository and a proxy of a local pub.dev stand-in, `dart pub add` with a
  hosted descriptor, retraction and discontinuation shown by `get` and `outdated`, advisories of the proxy
  reported by `get`.
- Ansible Galaxy (M4c second group, `crates/astrum-format-ansible`, format id `ansible`; new, the Java server never
  had it): collections only, from the public Galaxy v3 API and the behaviour of `ansible-galaxy`, for hosted, proxy
  and group. `api/` answers `available_versions` `{"v3": "v3/"}` (the repository root is an HTML page, so the
  client falls back to `api/` as it does for galaxy.ansible.com); `api/v3/collections/`, `.../<ns>/<name>/`,
  `.../versions/` (newest first by SemVer) and `.../versions/<v>/` (`download_url` absolute to this repository,
  `artifact.sha256`, `metadata.dependencies`, `manifest`, `requires_ansible` from `meta/runtime.yml`); lists page
  with `limit`/`offset` and `meta.count`, and `links` are paths from the host root because the client replaces
  the path of its own URL with them. Archives live at `<ns>/<name>/<version>/<ns>-<name>-<version>.tar.gz`
  (component group = namespace). Hosted repositories take `POST api/v3/artifacts/collections/` (multipart `file`,
  optional `sha256`; `ansible-galaxy` sends the file with `Content-Transfer-Encoding: base64`, which is decoded on
  the way in): permissions, write policy, the file name and the checksum answer at once (v3 error JSON
  `{"errors": [{status, code, title, detail}]}`), problems of `MANIFEST.json` (names, SemVer, dependencies, the
  file name must match) end the import task as failed; tasks are in table `ansible_import` (kept 7 days) at
  `api/v3/imports/collections/<id>/`, always finished because the import runs during the upload. Versions and
  documents are built from the components on every request. `DELETE .../versions/<v>/` removes a hosted version.
  Proxies discover the remote's v3 root like the client (the URL, else `api/`; 10 minutes in memory), cache the
  collection document (summary kept in the asset attributes for the list), the whole version list (all remote
  pages, following `links.next`) and version documents as metadata under `api/v3/collections/<ns>/<name>/`, and
  archives as content from the cached `download_url` (redirects followed), refusing an archive whose SHA-256
  differs from the version document; addresses are rewritten when served. The proxy's collection list shows what
  it has cached. Groups merge version lists and summaries (first member wins per version). `Authorization: Token
  <token>` is read like Bearer by `astrum-api` auth. Search keys `ansible.tag`, `ansible.license`; `q` also
  matches description and tags; cleanup pre-releases by SemVer. Web: picker text and logo, galaxy.ansible.com
  as remote example, usage (ansible.cfg, install `-s`, requirements.yml, publish), the component snippet, search
  criteria. Checked with ansible-core 2.21.5: `collection publish` (hosted; refusals and a failed import shown by
  the client), `install` with dependencies through a group of a hosted repository and a proxy of another hosted
  repository, `--pre` and `requirements.yml`, `download`, `verify`, anonymous installs, a 105-version collection
  paged by the client and by the proxy. Not done: roles (v1 API), signatures, `deprecated` for hosted
  collections, Galaxy's `requires_ansible` requirement on upload, namespaces API.
