# M3 brief for agents (web UI of the Rust rewrite of Astrum Repository)

M3 builds the new web UI of the Rust server: the shell plus every screen whose backend exists after M1 and
M2. Screens whose backend comes later (cleanup policies, routing rules, tasks, LDAP, OIDC, SSL, email, HTTP,
notifications, logs, support, capabilities replacements, API explorer) are NOT part of M3: no menu entries,
no placeholders for them.

Read first, in this order:
1. `.claude/CLAUDE.md` (working rules) and `.claude/rewrite/README.md` (plan, clean-room rules, the
   "M1 conventions", "M2 conventions", "M2 progress" and, once it exists, "M3 conventions" sections).
2. `.claude/rewrite/inventory-ui.md`: what the current UI does (screens, URLs, gates, actions, shell). It is
   the functional specification of the screens; behaviour and layout follow it, code does not.
3. `/home/user/wt/M2-BRIEF.md`: environment rules for Rust work (shared target dir, PostgreSQL, CI commands,
   reference server, credentials). They all apply here too.

## Clean-room rules for the UI (binding)
- Never open the Astrum One React/TypeScript/JavaScript code under `public/` (`astrum-ui-plugin`,
  `astrum-coreui-plugin`, `astrum-rapture`, their `src/frontend/**`), and never the ExtJS code. Close a file
  you opened by accident. Do not translate or paraphrase that code.
- Allowed, because it is ours:
  - the design tokens `public/common/components/astrum-coreui-plugin/src/frontend/src/styles/astrum-brand.css`
    (copy what you need into `web/src/styles/`);
  - the Russian dictionaries `public/**/src/main/resources/astrum-i18n/**/ru.json` (reuse Russian texts);
  - the client usage snippets of the formats in
    `public/common/components/astrum-ui-plugin/src/frontend/src/components/pages/search/details/dependencySnippets/usage/`
    (only for the snippet texts and commands, not as a code model);
  - the docs under `docs/`, `.claude/rewrite/*`, and screenshots of the running reference server's UI.
- English UI texts are written afresh (do not copy `en.json` dictionaries of the Java UI).
- The running reference server (`http://127.0.0.1:8081`, Java Astrum 2.0.0) may be looked at as a black box:
  open its UI with Playwright and take screenshots to match the look and the behaviour (it is our design).
  Sign in as admin (password file below; never print it). Do not change anything there except your own probe
  objects named `m3-<area>-...`, deleted afterwards. Old screenshots of that UI are in
  `/home/user/wt/*-shots/`.

## Stack (decided)
- `web/` at the repository root, outside the yarn workspaces of the Java UI: its own `package.json` with
  `"packageManager": "pnpm@10.18.0"` (run `corepack pnpm ...`, `COREPACK_ENABLE_DOWNLOAD_PROMPT=0`),
  `pnpm-lock.yaml` committed.
- Vite, React 19, TypeScript `strict`, Radix Themes 3 + Radix primitives, React Router 7 (library mode),
  TanStack Query 5 for server data, Biome for lint and format (its own `web/biome.json`), Vitest + Testing
  Library for unit and component tests, Playwright for end-to-end tests against the real Rust binary.
  Icons: `lucide-react`. Add other dependencies only when they clearly pay for themselves; say why in the
  commit body.
- Plain path URLs: `/` dashboard, `/browse`, `/browse/<repository>/<path>`, `/search`, `/search/custom`,
  `/search/component/...`, `/upload`, `/upload/<repository>`, `/audit`, `/settings`,
  `/admin/repository/...`, `/admin/security/...`, `/user/account`, `/user/access-tokens`, `/login`,
  `/onboarding`. Old hash URLs (`#browse/...`, `#admin/...`, `#settings`, `#user/...`, `#preview/...`,
  `#browse/welcome`, Classic URLs listed in `inventory-ui.md`) are redirected client side to their paths.
- The Rust server serves the built UI from a directory (`web.dir` / `ASTRUM_WEB_DIR`, default
  `/usr/share/astrum/web` in the image): hashed assets with a long cache, `index.html` with `no-cache`, an
  SPA fallback for GET/HEAD on paths outside `/repository/`, `/service/`, `/v2/`, `/content/`, `/.well-known/`,
  security headers (CSP `default-src 'self'`, no inline script, `frame-ancestors 'none'`, `nosniff`,
  `Referrer-Policy: same-origin`). Without the directory the server still runs and `/` answers a short text
  that the UI is not installed.
- The image builds the UI in a node stage and copies it in; CI runs lint, typecheck, unit tests and the build
  of `web/` on every push that touches it.

## Shared conventions (set by the foundation; do not change without saying so in your report)
- Source layout: `web/src/app/` (bootstrap, router, providers), `web/src/shell/` (header, menu, search panel
  host, theme, language), `web/src/api/` (fetch client, query keys, types of the REST JSON), `web/src/auth/`
  (state, session, permissions), `web/src/i18n/`, `web/src/ui/` (our component layer), `web/src/styles/`,
  and one folder per area under `web/src/areas/<area>/` holding its routes, pages, components, strings and
  tests. An area registers itself through one line in `web/src/app/areas.ts` (routes and menu entries).
- Strings: every user-visible text lives in `web/src/areas/<area>/strings/{en,ru}.json` (or the shell's and
  ui's own files); keys are `<area>.<page>.<name>`; plurals use `Intl.PluralRules` categories
  (`one/few/many/other`); no texts in components. A check (`pnpm i18n:check`) fails when en and ru keys differ
  or a used key is missing.
- Permissions: the UI reads the caller's permission strings from the state endpoint and matches them with the
  same wildcard rules as the server (`:` parts, `,` values, `*`, case-insensitive). Routes and menu entries hide
  what the caller cannot use; the server stays the authority.
- API client: same-origin `fetch` with `X-Astrum-UI: true`, `X-Astrum-CSRF` from the `astrum_csrf` cookie on
  unsafe methods, JSON errors parsed from our `ApiError` shapes (404/403 `{"id","message"}`, 400
  `[{"id","message"}]`), 401 raises the session-expired dialog.
- Look: our tokens, dark theme by default, light and system themes, the login page always light, violet
  accent, header with menu toggle, global search, theme toggle, language menu, help menu and user menu, a left
  rail on desktop and a drawer on compact screens (phone at most 639 px, compact at most 1023 px). Every page
  must work at 390 px wide without horizontal page scroll, and in both themes.
- Accessibility: keyboard reachable, visible focus, labels on every control, dialogs trap focus.
- Tests: logic and components with Vitest; each page gets at least one Playwright scenario that runs against
  the real Rust binary (see "End-to-end" below). Take screenshots of each page at 1440x900 and 390x844 in dark
  and light into `/home/user/wt/shots/m3-<area>/` (not committed) and list them in your report.

## End-to-end
- Playwright uses the preinstalled Chromium: `executablePath: '/opt/pw-browsers/chromium'` (never run
  `playwright install`).
- The e2e setup starts the Rust binary given in `ASTRUM_BIN` (default `/home/user/wt/m1-target/debug/astrum`)
  with a fresh data directory and a fresh database on `ASTRUM_E2E_DATABASE_URL` (default
  `postgres://postgres@127.0.0.1:5433/postgres`; created and dropped by the setup), `ASTRUM_WEB_DIR` pointing
  at `web/dist`, on a free local port; it reads `admin.password` from the data directory.
- Build the binary with the workspace commands from the M2 brief before running e2e.

## Environment
- Your worktree, branch, port range and database prefix are given in your task. Work only there; do not push;
  commit as Roman (see M2 brief). Create the worktree's `web/node_modules` with `corepack pnpm install` (the
  pnpm store hardlinks packages, so it is cheap).
- Disk is limited (about 8 GB free): no extra Rust target dirs, delete screenshots and data dirs you no longer
  need, never copy `node_modules`.
- Rust work follows the M2 brief exactly (`CARGO_TARGET_DIR=/home/user/wt/m1-target CARGO_INCREMENTAL=0`,
  workspace commands only, fmt, clippy `-D warnings`, tests with `ASTRUM_TEST_DATABASE_URL`).
- Web checks before you report: `corepack pnpm lint`, `corepack pnpm typecheck`, `corepack pnpm test`,
  `corepack pnpm i18n:check`, `corepack pnpm build`, and your e2e scenarios with `corepack pnpm e2e`.
- No comments in code; explain in commit messages. Plain English in git.

## Report
What you built (routes, endpoints, components, decisions and deviations from the current UI and why), the
commits, test counts (unit and e2e), screenshots taken, files touched outside your area, and what is left.

## Phase B additions (pages; read these too)
- The foundation is on main: read `web/README.md` and the README sections "M3 conventions" and
  "M3 progress" (the internal UI API: `/service/rest/internal/ui/...`) before writing anything, and look at
  `web/src/areas/dashboard` and `web/src/areas/settings` as the model areas. Use the components in
  `web/src/ui`; add a generic component there only when two areas clearly need it, keep it small and tested,
  and list it in your report (other agents work in parallel; prefer components inside your own area).
- Shared files you may touch with one-line edits: `web/src/app/areas.ts` (register your areas). Do not edit
  other areas' folders, the shell or `src/app` beyond that; say in your report if they need a change.
- Server endpoints you need and that do not exist: add them in your own file
  `crates/astrum-api/src/routes/internal/ui/<area>.rs` (one line in that folder's `mod.rs`), with tests in
  `crates/astrum-api/tests/ui_<area>.rs`, following the M2 brief's Rust rules. Keep them small.
- Your own binary for e2e and manual runs: after `cargo build --workspace` copy
  `/home/user/wt/m1-target/debug/astrum` to `/home/user/wt/bin/astrum-<area>` at once (other agents overwrite
  the shared one) and use that path as `ASTRUM_BIN`.
- Component page address (shared between the browse and search areas):
  `/search/component/<encodeURIComponent(format + ":" + group + ":" + name)>?version=<version>&tab=<tab>`
  (group empty when the format has none). Search results link there; the browse area builds the page.
- Test data for e2e: create repositories, users and content through the REST API in the test setup (raw PUT,
  maven PUT, npm publish JSON, pypi upload multipart, docker blobs/manifests are all available on the Rust
  server); never depend on the reference server in e2e.
