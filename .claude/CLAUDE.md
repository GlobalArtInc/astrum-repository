# Astrum Repository: project memory

Astrum Repository is GlobalArt's artifact repository manager, version 2.0.0, running in production at
https://astrum.globalart.hu. Today it is a modified version of Nexus Repository OSS (EPL-1.0) with a
React UI called Astrum One. A from-scratch Rust rewrite is under way (M0 done): read
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
- Rust rewrite: M0 done; M1 foundation, sessions, access tokens and audit done; blob stores, repository
  configuration and security REST were being built by agents in `/home/user/wt/m1-{blob,repos,secrest}`
  (branches `m1/*`, shared build dir `/home/user/wt/m1-target`, brief `/home/user/wt/M1-BRIEF.md`).
- Known open defects: `/service/rest/v1/user-telemetry/javascript` 404 on every page; certificate check
  in the repository HTTP settings calls a missing truststore endpoint; saving raw proxy query params,
  docker `latestPolicy` and ECR settings from Astrum One resets them; horizontal scroll at 1024px; server
  texts (capability, task, realm, privilege descriptions) are English only; Docker `subdomain` routing is
  stored but not implemented; group blob stores cannot be managed from Astrum One.

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
