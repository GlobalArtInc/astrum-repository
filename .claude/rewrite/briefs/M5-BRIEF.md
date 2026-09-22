# M5 brief for agents (Rust rewrite of Astrum Repository: migration)

M5 moves an installation of the Java Astrum Repository 2.0.0 (PostgreSQL backed) to the Rust server without
loss, proves that clients see the same behaviour on both, and prepares the production switch with a rollback
plan. Read first, in this order, and follow them:
1. `.claude/CLAUDE.md` (working rules) and `.claude/rewrite/README.md` (decisions, clean-room rules, "Our own
   code", conventions, "Where the Rust server stands").
2. `.claude/rewrite/inventory-data.md`: what the Java installation stores (tables, secrets, blob layout,
   versions). It is the specification of the source side. Confirm every detail against a live database and
   data directory before relying on it; where it is wrong, fix it in the same change.
3. `.claude/rewrite/briefs/M2-BRIEF.md` (Rust environment, real clients) and `M4-WAVE2-TASK.md` ("Done
   means", report).

## Decisions for M5
- Own schema plus a migrator (the README's target architecture): the Rust server keeps its `astrum` schema;
  `astrum migrate` reads the Java database and data directory and writes the Rust database and blob stores.
  The Java database and data directory are only read, never written: production can always go back.
- Clean room: the Java database schema and the files of a data directory are black-box data and may be
  inspected freely (`\d`, `SELECT`, file listings, hex dumps). Upstream-derived Java code stays closed; the
  source paths named in the inventory are not to be opened. Our own code from "Our own code" may be read.
- The migrator refuses an unknown source: Flyway maximum version, failed migrations, a non-empty
  `upgrade_tasks`, secrets it cannot decrypt. It runs again safely on the same target (idempotent per object
  or a clear "target not empty" refusal with `--force` to wipe the Rust schema first) and writes a report
  (counts per table and format, skipped objects with the reason, warnings) as text and JSON.
- Formats the Rust server does not have (alpine, apt, cargo, cocoapods, composer, conan, conda, go, p2, r):
  the report lists their repositories and sizes; their configuration is kept in a side table or file so a
  later M4c can import them; their content is not migrated.
- Blob files: on the same filesystem the migrator hard-links the old `.bytes` files into the Rust layout (no
  copy, no change to the old files); otherwise it copies. S3 stores: server-side copy inside the bucket, or
  an explicit `--s3-adopt` mode only if the Rust S3 store can read the old keys without a copy. Soft-deleted
  blobs are not migrated.
- Sessions and JWT secrets are not migrated (users sign in again). Passwords are: the Rust server accepts the
  Java hash formats (`$shiro1$SHA-512$...`, PBKDF2 PHC, legacy unsalted hex) and rehashes on the next
  successful sign-in.
- Never touch production from an agent session: no reads of the production database, no deploys, no infra or
  gitops pushes. Production steps are written as a runbook and prepared changes for the coordinator.

## Done means
Code + tests (unit, integration against PostgreSQL with a source database built from a real Java
installation, see below) + docs (`docs/migrate/from-java-astrum.md` for the operator), Rust fmt, clippy
`-D warnings`, the workspace tests in batches (see "Disk" in `.claude/CLAUDE.md`), web checks and e2e where
the UI changes. Commit each finished piece as Roman; do not push or merge. Update "Where the Rust server
stands" in `.claude/rewrite/README.md` with a short paragraph.

## Source data for tests
The reference Java server (Astrum 2.0.0) runs on `http://127.0.0.1:8081` with its database `astrume2e` on the
local PostgreSQL (127.0.0.1:5433, user `postgres`, trust) and its data directory
`/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/e2e/pgdata` (blobs under `blobs/`;
no secrets.json, so secrets use the default cipher password). Its admin password is in
`/tmp/claude-0/-home-user/e3c42964-05d8-56ab-b35a-04586d3a8a00/scratchpad/e2e/admin-password.txt` (read it into
a variable, never print it). It is shared: add test objects through its REST API or UI with your track prefix
(`m5c-`, `m5n-`, `m5s-`) and leave them in place for the other tracks (they are migration test data); never
delete objects you did not create; do not stop or reconfigure the server. To test against a frozen copy,
`pg_dump` the database into your own database (`createdb -T` does not work while the server is connected) and
read blobs from the data directory without writing to it (hard links from your scratch dir are fine: same
filesystem).
