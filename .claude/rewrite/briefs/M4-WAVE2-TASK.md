<!-- Written for the second container of 2026-10-13 (paths under /home/developer/well). On another server keep the rules and adjust the paths: main checkout, worktrees, target dirs, scratch, reference server password file. -->

# M4 wave 2 task brief (environment of 2026-10-13, second container)

You finish one part of M4 of the Rust rewrite of Astrum Repository. Read, in this order, and follow them
(everything in them applies unless this file says otherwise):
1. `.claude/CLAUDE.md` (working rules: commits as Roman, clean room, no trailers, English in git).
2. `.claude/rewrite/README.md` (plan, decisions, clean-room rules, "Our own code", conventions, "Where the Rust
   server stands": know what exists before writing anything).
3. `.claude/rewrite/briefs/M2-BRIEF.md`, `M3-BRIEF.md`, `M4-BRIEF.md` (incl. "Wave 2 additions"),
   `PARITY-BRIEF.md`, and `web/README.md`. UI findings old/new: `.claude/rewrite/qa-findings.md`.

## Paths in this environment (they replace the paths in the briefs)
- Main checkout (read only for you): `/home/developer/well/astrum-repository`. Your worktree and branch are in
  your task; work only there. `/home/user/wt` in the briefs = `/home/developer/well/wt`.
- Rust: `export PATH=$HOME/.cargo/bin:$PATH CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` and your OWN target
  dir `CARGO_TARGET_DIR=/home/developer/well/wt/target-<name>` (pre-seeded; there is enough disk, do not share
  it). Workspace commands only. Tests: `ASTRUM_TEST_DATABASE_URL=postgres://postgres@127.0.0.1:5433/postgres`.
  Build with `-j 3` (`CARGO_BUILD_JOBS=3`): six agents share 12 cores.
- PostgreSQL 18 (the briefs say 16; same use) on 127.0.0.1:5433, user `postgres`, trust auth:
  `psql -h 127.0.0.1 -p 5433 -U postgres`. Create databases only with your prefix and drop them at the end.
- Your binary for e2e/manual runs: copy `$CARGO_TARGET_DIR/debug/astrum` to `/home/developer/well/wt/bin/astrum-<name>`
  and use it as `ASTRUM_BIN`.
- Web: `cd web && COREPACK_ENABLE_DOWNLOAD_PROMPT=0 corepack pnpm install --frozen-lockfile` in your worktree
  (store is hardlinked, cheap). Playwright Chromium: the one `npx playwright install chromium` put in
  `~/.cache/ms-playwright` (the briefs say `/opt/pw-browsers/chromium`; if the config hardcodes that path, set
  the env override the config supports or ask the coordinator, do not edit the config for everyone).
- Reference server (the old Java Astrum 2.0.0 = production, a black box): `http://127.0.0.1:8081` (running,
  PostgreSQL database `astrum_ref`, onboarding done); admin password in
  `/home/developer/well/wt/ref/admin-password.txt` (read it into a variable, never print it). Other agents use
  it at the same time: probe objects there are named `<name>-...` and deleted; change no global settings on it
  except what you restore at once. Do not start, stop or reconfigure it; tell the coordinator if it is down.
  Never send credentials to https://astrum.globalart.hu or write there (anonymous reads are fine).
- Processes you did not start are not yours: `tsh` on 127.0.0.1:18200 and the http.server on 8099 belong to
  the owner. Never stop them; stop only what you started, by pid, and stop it before you report.
- Scratch files, screenshots, data dirs: `/home/developer/well/wt/scratch/<name>/` only; screenshots in
  `/home/developer/well/wt/shots/<name>/`.
- Ownlate is not reachable from this environment: write `strings/{en,ru}.json` only and list the new and
  changed web keys in your report (the coordinator uploads them later).
- Never touch other agents' worktrees, servers, databases or files. Stop processes by pid, never `pkill -f`.

## Done means
Server part + UI pages (exactly like the old Astrum One page where one exists) + tests: Rust fmt, clippy
`-D warnings`, `cargo test --workspace --locked`; web lint, typecheck, test, i18n:check, build; Playwright
scenario(s) for your pages against your binary, and the full e2e suite once at the end. Real clients where
a format or protocol has one. Commit each finished piece as Roman as soon as its checks pass (logical commits,
plain English, imperative subject, body says why, no trailers, no AI mention). Do not push, do not merge.
Update `.claude/rewrite/README.md` "Where the Rust server stands" with a short paragraph for what you added
(append at the end of that section; keep it terse, the coordinator resolves conflicts).

## Report (final message, under 400 words)
What you built (endpoints, pages), deviations from the reference and why, clients/versions run, commit
hashes, test counts (Rust, unit, e2e), new/changed web i18n keys (count and prefix), files touched outside
your area, what is left or decided by you that the owner should know.
