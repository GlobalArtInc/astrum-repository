# M4 brief for agents (Rust rewrite of Astrum Repository: formats and operations)

M4 brings the Rust server to what production needs: the formats production uses (M4a) and the operations
features (M4b), each with its server part AND its web UI pages, tested with real clients and end to end.

Read first, in this order, and follow them; everything in them still applies unless this brief says
otherwise:
1. `.claude/CLAUDE.md`, `.claude/rewrite/README.md` (plan, decisions, clean-room rules, M1/M2/M3 conventions
   and progress).
2. `/home/user/wt/M2-BRIEF.md` (Rust environment, shared target dir, PostgreSQL, CI commands, reference server,
   credentials, formats as crates, real clients).
3. `/home/user/wt/M3-BRIEF.md` (web UI rules, clean-room rules for the UI, stack, conventions, e2e) and
   `web/README.md`.

## Rules added for M4
- Clean room: the Java format modules and everything under `public/` that is upstream-derived stay closed.
  Our own code listed in the README's "Our own code" section may be read and ported (LDAP, OIDC, access
  tokens, default role, notifications, download counters and usage dashboard, cleanup preview totals,
  our formats yum/rubygems/terraform/huggingface/swift/gitlfs, the OCI work). Before porting a file, check
  that it is in that list.
- Every feature is complete only with its UI: pages in `web/src/areas/<area>/` registered in
  `web/src/app/areas.ts`, settings tiles where they belong, strings in en and ru, unit tests and at least one
  Playwright scenario against the real binary, screenshots at 1440x900 and 390x844 in dark and light into
  `/home/user/wt/shots/m4-<area>/`.
- Texts: write `strings/{en,ru}.json` only. Do NOT write to Ownlate; the coordinator syncs new keys when
  merging. If `web/` has an i18n pull check that compares with Ownlate, run only `corepack pnpm i18n:check`.
- Server endpoints: public REST under `/service/rest/v1/...` keeps its paths and JSON shapes (public
  REST documentation and black-box requests to the reference server); internal UI endpoints under
  `/service/rest/internal/ui/<area>/...` in `crates/astrum-api/src/routes/internal/ui/<area>.rs`. Settings go
  through `astrum-db` `setting` with an audit event per change. New tables or columns: new migration pairs
  only.
- Background work (tasks, cleanup, flushing, expiry) must be safe with several replicas: take a PostgreSQL
  advisory lock per job run, and keep state in the database, never only in memory.
- Your worktree, branch, ports and database prefix are given in your task. Copy your binary to
  `/home/user/wt/bin/astrum-<area>` before e2e. Keep scratch files under
  `/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/m4-<area>/` and nowhere else;
  never touch other agents' worktrees, servers, databases or files.
- Disk is limited (about 14 GB free for everyone): delete client caches, data dirs, images and test files you
  created; no extra Rust target dirs.
- Before you report: Rust fmt, clippy `-D warnings`, `cargo test --workspace --locked`; web lint, typecheck,
  test, i18n:check, build; your e2e specs and the full e2e suite. Commit as Roman in logical commits; do not
  push.

## Report
What you built (endpoints with JSON shapes, pages, decisions and deviations from the reference server and
why), clients and versions you ran with the commands, commits, test counts (Rust, unit, e2e), screenshots,
files touched outside your area, and what is left.

## Wave 2 additions (read these too; they override the rules above where they differ)
- Every page you build must look and behave exactly like the same page of the old Astrum One UI on
  `http://127.0.0.1:8081` (owner's decision; see `/home/user/wt/PARITY-BRIEF.md` for how to compare, the
  clean-room limits and the QA report `scratchpad/qa/report/findings.md`). Settings tiles: register `{ id, to,
  need }` with the id of the old tile from the catalogue in `web/src/areas/settings/sections.ts`.
- Rust builds: `export CARGO_TARGET_DIR=/home/user/wt/m1-target CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`
  (no debug info, to save disk; five agents share it). If cargo seems to skip your changes, touch the changed
  sources and rebuild.
- Texts: upload your new and changed web keys to Ownlate yourself when you finish (`mcp__ownlate__import_segments`
  in project `f1f93c0e-4902-434d-a551-e94ba7e1e304`, file `119d5931-d6cc-4e49-aaa6-ce23ad9b6257`, keys `web.<key>`,
  plurals as `web.<key>#<form>`; source texts first without language, then Russian with approve=true; batches of
  at most 140; read back on "session expired"). Never delete or overwrite keys you did not write.
- Commit each finished piece as soon as its checks pass, so nothing is lost if you are stopped. One old/new
  screenshot pair per page at 1440 dark. Run the full e2e suite once at the end.
