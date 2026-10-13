# Astrum Repository: project memory

Astrum Repository is GlobalArt's artifact repository manager, version 2.0.0, running in production at
https://astrum.globalart.hu. Today it is a modified version of Nexus Repository OSS (EPL-1.0) with a
React UI called Astrum One. A from-scratch Rust rewrite is under way (M0-M3 done, M4 in progress): read
`.claude/rewrite/README.md` before any work on it, and follow its clean-room rules. This repository is public: never commit
secrets.

## State (2026-10-12)
- Live: version 2.0.0 (numbering restarted from 2.0.0), the Classic ExtJS UI and the `preview/` URL
  prefix removed, Astrum One at the root of the hash (`#browse/...`, `#admin/...`, `#settings`), 23 formats
  (alpine, apt, cargo, cocoapods, composer, conan, conda, docker, gitlfs, go, helm, huggingface, maven2,
  npm, nuget, p2, pypi, r, raw, rubygems, swift, terraform, yum), Russian and English.
- OCI Distribution 1.1 lives inside the Docker format (live since image 20261012131009-0f089f9:
  referrers API, sha512, tag parameters, chunked uploads with 416, cross-repository mount, artifact kinds in
  search and Astrum One; deleting a manifest by digest deletes its referrers; cleanup deletes tags only).
  pub and Ansible Galaxy were stopped; they will be written in Rust.
- Rust rewrite: M0-M3 done (security, sessions, access tokens, audit, blob stores, repository
  configuration, the content engine with raw, maven2, npm, pypi and docker/OCI, the REST API, and the web UI in
  `web/` with every page whose backend exists); M4 wave 1 done (helm, nuget, tasks and cleanup,
  LDAP/OIDC/default role); M4 wave 2 and the remaining own formats are on six unmerged branches `m4-*` (see
  "Where the work stopped"). Agent briefs: `.claude/rewrite/briefs/`. Test instance live at https://next.astrum.globalart.hu (since 2026-10-12): the
  `Server` workflow publishes `globalartltd/astrum-repository:next-<YYYYMMDDHHMMSS>-<sha7>` on pushes to main
  (line `published ...` in the "container image" job); to deploy, set `astrum_next_version` in infra
  `infralive/ansible/roles/astrum/defaults/main.yaml`, commit `Update astrum_next_version to <tag>` as Roman,
  push, dispatch `provision.yml` (`apply`, `name=prod1-astrum1`). It runs as `astrum-next.service` on
  `prod1-astrum1`, port 8082, data `/opt/astrum-next/data`, database `astrum-next` on `prod1-dbms1`; route and
  certificate in gitops `clusters/prod1/platform/astrum/next.yaml`. Open question to the owner: keep the `astrum` schema plus a migrator, or tables
  identical to the Java server's.
- Known open defects: `/service/rest/v1/user-telemetry/javascript` 404 on every page; certificate check
  in the repository HTTP settings calls a missing truststore endpoint; saving raw proxy query params,
  docker `latestPolicy` and ECR settings from Astrum One resets them; horizontal scroll at 1024px; server
  texts (capability, task, realm, privilege descriptions) are English only; Docker `subdomain` routing is
  stored but not implemented; group blob stores cannot be managed from Astrum One.

## Where the work stopped (2026-10-13, 18:40 UTC)
- Done and on next (revision 31e33ae): M0-M3, helm, nuget, LDAP/OIDC/default role, tasks and cleanup
  policies, the UI parity work (shell, dashboard, browse with per-format trees, repository and blob store pages,
  search/upload/audit, security pages), and the fix for deleting a blob store in use on PostgreSQL 18
  (it reports RESTRICT violations as 23001, which sea-orm does not map).
- M4 wave 2 and the own formats were in progress on six branches, all pushed and NOT merged into main:
  `m4-outbound` (routing rules, HTTP, SSL truststore, email), `m4-notify` (notifications, base URL, UI settings,
  branding, nodes), `m4-support` (logs, log levels, system info, health + bell, support ZIP, API explorer),
  `m4-compat` (Nexus 2 URLs, docker connector ports/subdomains/port suggestion, M2 leftovers), `m4-fmt-a`
  (yum, rubygems, gitlfs), `m4-fmt-b` (terraform, huggingface, swift). The owner stopped the agents to save the
  usage limit. Per track: scope, finished commits, the unchecked "Keep the unfinished ..." commits, what is
  left and how to continue: `.claude/rewrite/briefs/M4-WAVE2-TRACKS.md`; the agent brief:
  `.claude/rewrite/briefs/M4-WAVE2-TASK.md`.
- Anonymous users see only raw, maven2, npm, docker, helm and nuget repositories on production; the Rust server
  has all of them. Formats of hidden repositories are unknown.
- Owner to decide: SSRF page (exists only in the new UI), storing uploaded maven checksum files, the formats of
  production repositories hidden from anonymous users, the database layout question above.
- Agent briefs are in `.claude/rewrite/briefs/` (M1-M4, PARITY, M4-WAVE2-*; they name the paths of the
  container they were written in: put worktrees, target dirs and scratch files wherever fits and adjust the
  paths), the old/new UI findings in `.claude/rewrite/qa-findings.md`.
- A fresh environment needs: the Rust toolchain of `rust-toolchain.toml`; PostgreSQL for tests
  (`ASTRUM_TEST_DATABASE_URL`; 16 or 18 both work since 31e33ae); Node 22 with corepack pnpm for `web/`;
  Chromium for Playwright at `/opt/pw-browsers/chromium` or `ASTRUM_E2E_CHROMIUM` (on Ubuntu 26.04
  `playwright install` refuses the OS, but its downloaded chromium-1140 runs: symlink
  `~/.cache/ms-playwright/chromium-1140/chrome-linux/chrome`); `gh` logged in for workflow dispatch; for UI
  parity the Java server built from `public/` running locally with PostgreSQL as the reference; infra and gitops
  repositories for deploys to next.
- Building the Java reference server from scratch: use JDK 25 (with JDK 21 the test compilation of conan,
  conda and others fails) and `mvn -B -q -Ppublic -DskipTests -Dnpm.skipTests=true -Denforcer.skip=true install`
  from the root (not `-Dmaven.test.skip`, which skips the test jars other modules need). Then a data dir with
  `etc/astrum.properties` and `etc/fabric/astrum-store.properties` as in "Build and test", started with
  JDK 25 from `public/selfhosted/assemblies/astrum-repository-core/target/assembly`.
- Local services after a restart: PostgreSQL `pg_ctl -D /var/tmp/astrum-pg/data -l /var/tmp/astrum-pg/pg.log
  -o '-p 5433 -k /tmp' start` as `postgres` (`/usr/lib/postgresql/<16|18>/bin`); the reference Java server on
  port 8081 with its own start script next to its data dir.

## Working rules
- Commits: author and committer Roman Khabarov <roman@globalart.dev>
  (`git -c user.name="Roman Khabarov" -c user.email=roman@globalart.dev commit ...`, also for
  cherry-pick and rebase). Plain English, imperative subject, a body that says why. No mention of AI
  tools, no Co-Authored-By or other trailers.
- Push straight to `main` (no pull requests unless asked); keep the session's assigned branch in sync
  with `main` when there is one.
- Deploy after every change that reaches `main`:
  1. Wait for the `Image` workflow (GitHub-hosted runners) and read the tag from the line
     `published globalartltd/astrum-repository:<tag>` in its job log (tag `YYYYMMDDHHMMSS-<sha7>`).
  2. In `GlobalArtInc/infra`, set `astrum_version` in `infralive/ansible/roles/astrum/defaults/main.yaml`
     to the tag, commit `Update astrum_version to <tag>` as Roman, push to `main`.
  3. Dispatch `provision.yml` on `main` with inputs `command=apply`, `filter=name=prod1-astrum1`; it runs
     on the self-hosted runners of the private infra repository (7–13 minutes). If OpenTofu init times out,
     re-run the failed jobs once.
- Never ask for, print or store passwords, tokens or keys. Clients under test must not reach public hosts
  with credentials (cut them off from the internet).
- Licence attribution lives only in LICENSE/NOTICE, never in file headers; never hide where code came
  from.
- Do not relax the SSRF protection for notification webhooks without an explicit request. Do not change
  the mail setup (Mailu, mail1) in infra without a go-ahead.
- Stop processes by pid; never `pkill -f`.
- Answer the owner in Russian; everything in git is in English.

## Layout
- Java/Maven modules: `public/common/components/` (formats in `formats/astrum-repository-<fmt>`,
  security plugins in `security/`), self-hosted assembly in
  `public/selfhosted/assemblies/astrum-repository-core`.
- Astrum One: `astrum-ui-plugin/src/frontend/src/components/preview/**` (pages), shell and router in
  `astrum-coreui-plugin/src/frontend/src` (`GlobalHeader/`, `LeftNavigationMenu/`, `routerConfig/`),
  design tokens in `astrum-coreui-plugin/src/frontend/src/styles/astrum-brand.css`.
- i18n dictionaries: `<module>/src/main/resources/astrum-i18n/<dict>/{en,ru}.json`; tooling `tools/i18n`.
- Docs: `docs/` (install, configure, operate, use, migrate). Container: `Dockerfile`, `docker/`.
- Rust server (the rewrite): `Cargo.toml`, `crates/`, `docker/server.Dockerfile`; not deployed yet.
- CI: `.github/workflows/image.yml` (builds and pushes the Java image; skips pushes that only touch the
  Rust server or `.claude/`), `server.yml` (Rust checks and image) and `i18n.yml`, on `ubuntu-latest`.

## Build and test
- Modules, from `public/common/components`:
  `mvn -o -q -Denforcer.skip=true -DskipTests -Dnpm.skipTests=true install -pl astrum-ui-plugin,astrum-coreui-plugin`
  (security module path is `security/astrum-security`; after adding a module also run
  `mvn -o -q -N -Denforcer.skip=true install` there so the parent pom's dependency management is current).
- Assembly: `cd public/selfhosted/assemblies/astrum-repository-core && mvn -o -q -Denforcer.skip=true -DskipTests clean install`.
- The local JDK is 21 (CI uses 25): conan's test compilation fails locally; build it with
  `-Dmaven.test.skip=true`.
- Jest, from a plugin dir: `../../../../node_modules/.bin/jest --config src/frontend/jest.config.js <paths>`
  (add `--maxWorkers=2 --testTimeout=60000` under load). Known failing suite: `UsageHelper.test.js`
  (expects links.globalart.com, code uses .dev).
- Local run: data dir with `etc/astrum.properties` (`application-port=<port>`) and, for PostgreSQL,
  `etc/fabric/astrum-store.properties` (`jdbcUrl`, `username`, `password`); start
  `java -Xmx1g -Dkaraf.data=<data> -Djava.io.tmpdir=<data>/tmp -jar bin/astrum-repository-core-*.jar` from
  `target/assembly`; first start writes `<data>/admin.password`; finish onboarding over REST
  (`PUT /service/rest/internal/ui/onboarding/change-admin-password`). UI sign-in:
  `POST /service/rapture/session` with base64 `username` and `password` form fields.

## Translations (Ownlate)
- User-visible strings live in the `*Strings.js(x)` files; run `node tools/i18n/i18n.mjs extract` then
  `check` after changing them.
- `import ru <file>` rewrites each dictionary with only the keys in the file: build the file from
  `node tools/i18n/i18n.mjs catalogue ru` plus the new keys, then import.
- Ownlate project `f1f93c0e-4902-434d-a551-e94ba7e1e304`, file `119d5931-d6cc-4e49-aaa6-ce23ad9b6257`.
  Write source texts first (no language), then Russian with approve=true; keep batches under ~150; a
  "session expired" error may still have applied the write. The CI check pulls from Ownlate and fails if
  the committed translations differ.

## Parallel agents (how it worked)
- One worktree per task: `git worktree add -b <branch> /home/user/wt/<name> origin/main`; a
  `node_modules` folder of symlinks to the main checkout's packages except
  `node_modules/@globalart/{astrum-ui-plugin,astrum-coreui-plugin,astrum-rapture}`, which point at the
  worktree's own packages; symlink `astrum-ui-plugin/node_modules` and `astrum-rapture/node_modules`;
  a hardlinked Maven repo `cp -al ~/.m2/repository /home/user/m2/<name>`; own port and database.
- Merge with cherry-pick. Typical conflicts: module lists and dependency blocks in the poms (split into
  separate `<dependency>` blocks), `BrowseStrings.js`, `en.json` (regenerate with `extract`),
  `usage/index.ts` and the generator list in `usage/__tests__/usage.test.ts` (sorted union), docs format
  lists. Never resolve a non-list conflict (whole test blocks) by merging lists.
