# PostgreSQL

Without any database settings the server uses an embedded H2 database in `<data>/db`. H2 is a single
file inside the server process: it cannot be inspected or backed up while the server runs, it can be
damaged by a hard power-off, and only one server can use it. Use PostgreSQL for anything that matters.

PostgreSQL 14 or newer is supported. Astrum is developed and tested against PostgreSQL 18.

## Prepare the database

```sql
CREATE ROLE astrum LOGIN PASSWORD 'change-me';
CREATE DATABASE astrum OWNER astrum ENCODING 'UTF8' TEMPLATE template0;
```

The server creates and migrates its tables itself on start; the role only needs to own the database.
To keep Astrum's tables out of `public`, create a schema owned by the role and set `schema` below.

Allow the server's address in `pg_hba.conf`, for example:

```text
host  astrum  astrum  10.0.0.0/8  scram-sha-256
```

## Connect the server

The server looks for the settings of its `astrum` data store in three places and uses the first one
that defines a `jdbcUrl`:

1. environment variables `ASTRUM_DATASTORE_ASTRUM_<ATTRIBUTE>`;
2. JVM system properties `astrum.datastore.astrum.<attribute>`;
3. the file `<data>/etc/fabric/astrum-store.properties`.

When none of them is set, the embedded H2 database is used.

| Attribute | Environment variable | Required | Meaning |
|---|---|---|---|
| `jdbcUrl` | `ASTRUM_DATASTORE_ASTRUM_JDBCURL` | yes | `jdbc:postgresql://<host>:<port>/<database>` plus driver options |
| `username` | `ASTRUM_DATASTORE_ASTRUM_USERNAME` | yes | Database role |
| `password` | `ASTRUM_DATASTORE_ASTRUM_PASSWORD` | yes | Its password; the container image also reads `ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE` |
| `schema` | `ASTRUM_DATASTORE_ASTRUM_SCHEMA` | no | Schema for the tables; default is the role's search path |
| `maximumPoolSize` | `ASTRUM_DATASTORE_ASTRUM_MAXIMUMPOOLSIZE` | no | Most connections the server opens, default `30` |
| `advanced` | `ASTRUM_DATASTORE_ASTRUM_ADVANCED` | no | Extra pool or driver settings, one `key=value` per line |

### Environment variables

```shell
ASTRUM_DATASTORE_ASTRUM_JDBCURL=jdbc:postgresql://db.example.com:5432/astrum
ASTRUM_DATASTORE_ASTRUM_USERNAME=astrum
ASTRUM_DATASTORE_ASTRUM_PASSWORD=change-me
```

### Properties file

`<data>/etc/fabric/astrum-store.properties`, readable only by the service user:

```properties
jdbcUrl=jdbc:postgresql://db.example.com:5432/astrum
username=astrum
password=change-me
maximumPoolSize=30
```

On start the log confirms which source was used:

```text
EnvironmentDataStoreConfigurationSource - Loaded 'astrum' data store configuration from Environment variables
MyBatisDataStore - astrum - MyBatis databaseId: PostgreSQL
```

## TLS to the database

Add the PostgreSQL JDBC driver's SSL options to the URL:

```text
jdbc:postgresql://db.example.com:5432/astrum?sslmode=verify-full&sslrootcert=/astrum-data/etc/db-ca.crt
```

## Connection pool

The server keeps at most `maximumPoolSize` connections (30 by default) and two idle ones. On a database
server shared with other applications make sure `max_connections` leaves room for all of them; a busy
instance with many concurrent CI jobs may need 50 to 100 connections.

Any HikariCP setting can be passed through `advanced`:

```properties
advanced=minimumIdle=5\n\
connectionTimeout=20000
```

Driver properties that are not pool settings (for example `socketTimeout`) go through the same
attribute and are handed to the PostgreSQL driver.

## Moving an existing H2 installation

There is no in-place converter from H2 to PostgreSQL. Start a new instance on PostgreSQL and copy
repositories, users and content into it, as described in
[migrate from Nexus Repository 3](../migrate/from-nexus.md); the same procedure works between two
Astrum instances.
