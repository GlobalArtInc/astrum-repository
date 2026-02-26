# PostgreSQL

Astrum Repository keeps its metadata in PostgreSQL: repositories, components and assets, users, roles,
settings, tasks, sessions and the audit log. There is no embedded database. The artifacts themselves
are in [blob stores](blob-stores.md).

PostgreSQL 16 or newer is supported; the server is tested against 16 and 18.

## Prepare the database

```sql
CREATE ROLE astrum LOGIN PASSWORD 'change-me';
CREATE DATABASE astrum OWNER astrum ENCODING 'UTF8' TEMPLATE template0;
```

The server creates and migrates its tables itself on start. It puts them into a schema called `astrum`
and creates the schema when it does not exist, so the role must own the database or be allowed to create
schemas in it. The first migration also runs `CREATE EXTENSION IF NOT EXISTS pg_trgm` for the search;
the extension is trusted, so the owner of the database may create it, but on a managed service you may
have to enable it for the database first.

To use another schema, create it (owned by the role) or let the server create it, and set
`database.schema` (`ASTRUM_DATABASE_SCHEMA`): lower-case letters, digits and underscores.

Allow the server's address in `pg_hba.conf`, for example:

```text
host  astrum  astrum  10.0.0.0/8  scram-sha-256
```

## Connect the server

Give the server the address of the database as a URL, in the configuration file or in the environment:

```toml
[database]
url = "postgres://astrum:change-me@db.example.com:5432/astrum"
schema = "astrum"
max_connections = 20
```

```shell
ASTRUM_DATABASE_URL=postgres://astrum:change-me@db.example.com:5432/astrum
```

| Key | Environment variable | Default | Meaning |
|---|---|---|---|
| `database.url` | `ASTRUM_DATABASE_URL` | none, required | `postgres://` or `postgresql://`, user, password, host, port and database; special characters in the password are percent-encoded |
| `database.schema` | `ASTRUM_DATABASE_SCHEMA` | `astrum` | Schema for the tables |
| `database.max_connections` | `ASTRUM_DATABASE_MAX_CONNECTIONS` | `20` | Most connections this server opens, at least 3 |

Without a database address the server refuses to start and says where to set it. The password is never
printed: logs and errors show `***` in its place.

`astrum migrate` connects, brings the schema up to date and exits; use it to check the connection and
the permissions before the first start.

## TLS to the database

Add the connection options to the URL:

```text
postgres://astrum:change-me@db.example.com:5432/astrum?sslmode=verify-full&sslrootcert=/astrum-data/etc/db-ca.crt
```

`sslmode` takes `disable`, `allow`, `prefer`, `require`, `verify-ca` and `verify-full`. Use
`verify-full` when the database is not on the same host or a trusted network.

## Connection pool

A server keeps at most `max_connections` connections open (20 by default). With several servers on one
database the limit of the database must cover all of them; check `max_connections` in `postgresql.conf`
and leave room for other applications and for backups. A busy instance with many concurrent CI jobs may
need 50 or more connections.

## Schema changes

The server applies the migrations it ships on start, while holding an advisory lock, so servers that
start together do not race. Migrations are one-way for the server: an older version cannot run against a
migrated database, see [upgrade](../operate/upgrade.md).

Do not point the server at a standby: it needs a primary that accepts writes, and
`GET /service/rest/v1/status/writable` answers 503 otherwise.

## Moving data from another installation

Repositories, users and content of another repository server are moved with the tools described in
[move from another repository server](../migrate/from-another-server.md).
