# Upgrade

Upgrades are one-way: on start the new version brings the database schema up to date. An older version
cannot run against a migrated database, so the way back is a restore from backup.

## Procedure

1. Read the release notes of every version between the running one and the target.
2. [Back up](backup-restore.md) the database, the blob stores and the secrets key.
3. Stop the server and wait until it has exited. On SIGTERM it finishes in-flight requests, stops its
   tasks and saves its download counters; give it up to three minutes. With several servers on one
   database, stop all of them.
4. Change the version:
   - container: change the image tag, for example
     `globalartltd/astrum-repository:20261018120000-3fa9c1e` (see
     [the tags of the image](../install/container.md));
   - built from source: install the new binary and web interface next to the old ones and point the
     service at them. The data directory stays where it is.
5. Start the server and follow the log. The migrations it applies are logged as they run. To apply
   them before serving, run `astrum migrate` with the new version first.
6. Check **Settings → Support → Metrics and Health**, sign in, and download a few artifacts from hosted
   and proxy repositories.

Never run two versions against the same database at the same time: servers that start together are
safe, because migrations run under a database lock, but a server of the old version does not understand
the new schema.

## Rolling back

Stop the server, restore the backup taken in step 2 and start the previous version.

## Upgrade notes

### 3.0.2

The update needs no steps of its own; the server adds one table to the database at start. Some settings
that were saved before but had no effect now work, which can change what an existing installation does:

- A task with a **Notification Email** sends email after its runs, as its **Send Notification** setting
  says, once an email server is set up; see [tasks](tasks.md).
- LDAP servers and OpenID Connect with the truststore option on trust the certificates under
  **Settings → Security → SSL Certificates** besides those of the system; see
  [SSL certificates](../configure/ssl-certificates.md).
- Servers that share a database pick up changes of users, roles, realms and blob stores made on another
  server within seconds instead of at their next start; see [nodes](nodes.md).

### 3.0

Version 3.0 is a new server written in Rust, with a new web interface. It replaces the 2.x line, which
ran on the Java virtual machine and an embedded or external database. It cannot be installed over a 2.x
data directory or database: the data is moved with the migration tools, see
[move from another repository server](../migrate/from-another-server.md). What changes for clients and
administrators:

- Repository URLs, the Docker `/v2/` API and the REST API under `/service/rest/v1` keep their paths and
  JSON shapes. The older URLs under `/content/` and `/service/local/` still work.
- The web interface uses plain paths (`/browse/...`, `/admin/...`, `/settings`). Old links with a hash,
  such as `#browse/...`, are redirected.
- PostgreSQL is required; there is no embedded database.
- Settings that lived in `astrum.properties`, the JVM options and `astrum-store.properties` are replaced
  by `astrum.toml` and environment variables, see [runtime configuration](../configure/runtime.md). The
  container listens on 8081, runs as uid 200 and keeps its data in `/astrum-data`, as before.
- The scripting API (`/service/rest/v1/script`) and Groovy tasks are gone.
- Prometheus metrics at `/service/metrics/prometheus` are off until the capability **Metrics:
  Prometheus** is created, and their names are new; see [monitoring](monitoring.md#prometheus-metrics).
