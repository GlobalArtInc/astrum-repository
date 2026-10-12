# Rewriting Astrum Repository in Rust

Status: planned, not started. Work starts after the usage limits reset (Saturday) or earlier if limits
allow. Nothing here is approved code; the plan below needs the owner's approval before the first line of
Rust is written.

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
3. The REST API paths and JSON shapes that external tools rely on (see `inventory-server.md`) may be kept
   for compatibility; implement them from the documented behaviour, not from the Java resources.
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
- PostgreSQL only (proposal: drop H2). Our own schema; the migrator reads the old schema.
- Blob stores: our own layout; the migrator moves or hard-links the old blob files (same filesystem) or
  reads the old layout once, so large stores are not copied twice.
- Client-facing URLs stay: `/repository/<name>/...` per format, Docker `/v2/...` with path routing and
  per-repository ports, Terraform discovery at the host root.
- UI: React + Radix Themes with our tokens, our own component layer; English and Russian through Ownlate;
  hash URLs kept and the old-URL mapping from `inventory-ui.md` preserved.
- Single binary + static UI in one container image (uid 200, port 8081, data in `/astrum-data`), built on
  GitHub-hosted runners (the repository is public).

## Milestones (each ends in a demonstrable state)
- M0 Skeleton: workspace, config, PostgreSQL migrations, health/status endpoints, container image, CI.
- M1 Core: blob stores (file, S3), users/roles/privileges/realms (local), sessions and CSRF, access
  tokens, content selectors, REST for security, repositories and blob stores, audit.
- M2 Engine + first formats: hosted/proxy/group engine with raw, maven2, npm, docker (incl. OCI 1.1),
  pypi; components/assets/search/browse API; uploads; tested with real clients.
- M3 UI: shell (header, menu, search panel, themes, languages, mobile), dashboard, browse, search,
  upload, settings for repositories, blob stores and security.
- M4 Everything else: remaining formats (alpine, apt, cargo, cocoapods, composer, conan, conda, go, helm,
  nuget, p2, r, rubygems, swift, terraform, yum, gitlfs, huggingface, pub, ansible), cleanup policies with
  previews, routing rules, tasks, settings that replace capabilities (base URL, UI settings, branding,
  default role, audit, webhooks), email, HTTP, SSL truststore, LDAP, OIDC, notifications, download
  counters and usage dashboard, logs and support pages.
- M5 Migration: `astrum-migrate` against a copy of production data, side-by-side run with the Java
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

## Open questions for the owner
- Drop H2 and support PostgreSQL only? (proposed: yes)
- Keep the Groovy script API? (proposed: drop; tasks and REST cover the use cases)
- Keep the migration-from-Nexus tool and its documentation (it names the third-party product only to
  import from it)?
- Keep the old Nexus 2 URL compatibility (`LegacyUrlCapability`)? (proposed: drop)
- Keep hash URLs in the UI or move to path URLs with redirects from the old ones?

## Inventories
- `inventory-server.md` — formats, REST API, security, tasks, blob stores, settings, configuration.
- `inventory-data.md` — database tables, secrets, blob store layout, versions (for the migrator).
- `inventory-ui.md` — routes, pages, actions, shell, what is upstream-only.
