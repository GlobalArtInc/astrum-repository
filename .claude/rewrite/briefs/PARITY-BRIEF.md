# Parity brief: make the new web UI match Astrum One exactly

Owner's decision (Decisions in `.claude/rewrite/README.md`): every page of the new web UI of the Rust server
must look and behave exactly like the same page of the current production UI (Astrum One,
astrum.globalart.hu). The shell, shared look and the dashboard are already done; you do one area.

Read first: `.claude/CLAUDE.md`; `.claude/rewrite/README.md` (Decisions, clean-room rules, M3 conventions and
progress, the dashboard notes); `web/README.md` (conventions incl. the Astrum One look section and the
settings tile catalogue); the QA report `/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/qa/report/findings.md`
(ranked findings F001..F111 with exact old/new descriptions and fixes) and `index.html` next to it
(side-by-side screenshots in `shots/<page>/`, computed-style tables).

Old UI to copy from: the running Java server `http://127.0.0.1:8081` (Astrum 2.0.0 = production). Admin
password in `/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/e2e/admin-password.txt`:
read it into a variable, never print it. You may create probe objects there named `p-<area>-...` and must
delete them; change no settings. Measure with Playwright (`executablePath: '/opt/pw-browsers/chromium'`,
`getComputedStyle`), compare screenshots side by side, copy layout, spacing, sizes, fonts, colors, radii,
icons, component kinds, field and column order, empty/loading states and behaviour.

Clean room (binding): never open the Java UI source (React/ExtJS) or upstream Java under `public/`. Russian
texts of the old UI are ours: copy them exactly (from the running UI or `public/**/astrum-i18n/**/ru.json`).
English texts stay our own wording in the old style (Title Case where the old UI uses it). Our own Java code
listed in the README's "Our own code" section may be read.

Environment: your worktree, branch, ports and database prefix are in your task; `web/node_modules` is
installed (`corepack pnpm`, `COREPACK_ENABLE_DOWNLOAD_PROMPT=0`). Rust only if your task needs server
changes: `CARGO_TARGET_DIR=/home/user/wt/m1-target CARGO_INCREMENTAL=0`, workspace commands only,
`ASTRUM_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5433/postgres`; copy your binary to
`/home/user/wt/bin/astrum-<area>` before e2e. Without Rust changes use `ASTRUM_BIN=/home/user/wt/bin/astrum-main`.
Scratch only under `/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/p-<area>/`;
never touch other agents' worktrees, servers or databases. Disk is shared and limited; delete what you create.
Other agents change other areas at the same time: keep edits to shared files (`src/ui`, `src/shell`,
`src/app`, other areas) minimal and backward compatible, and list them in your report.

Done means: the area's findings closed (say which F-numbers, and why any are left), Rust fmt/clippy/tests if
you touched Rust, web lint, typecheck, test, i18n:check, build, your e2e scenarios updated and the full e2e
suite run once, before/after screenshot pairs (old vs new, 1440 and 390, dark) in
`/home/user/wt/shots/p-<area>/`. Commit as Roman (`git -c user.name="Roman Khabarov" -c
user.email=roman@globalart.dev commit`), plain English, imperative subject, a body that says why, no
trailers, no mention of AI tools, in logical commits. Do not push. Report briefly.
