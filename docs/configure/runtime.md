# Runtime configuration

## Data directory

Everything the server writes lives in one data directory: `/astrum-data` in the container image, or
the directory passed as `-Dkaraf.data` to the distribution. Back it up and keep it across upgrades.

```text
<data>/
  admin.password            generated admin password, removed after the first password change
  blobs/default/            the default file blob store
  db/astrum.mv.db           embedded H2 database (absent when PostgreSQL is used)
  etc/astrum.properties     server settings, created with commented defaults on the first start
  etc/fabric/               optional astrum-store.properties for PostgreSQL
  etc/logback/              optional logging overrides
  keystores/                keys for secrets and the server's own keystore
  log/astrum.log            application log
  log/request.log           HTTP access log
  log/audit/                audit log, when auditing is enabled
  log/tasks/                one log file per task run
  restore-from-backup/      drop H2 backup files here to restore them on the next start
  tmp/                      temporary files, safe to clear while the server is stopped
```

## Secret encryption key

Stored secrets (proxy and remote repository passwords, the OpenID Connect client secret, the email
password) are encrypted in the database. Out of the box they use a key built into the server,
which is the same for every installation. For production, give the server its own key in a JSON file
that only the service user can read:

```json
{
  "active": "2026-10",
  "keys": [
    { "id": "2026-10", "key": "a long random string, for example the output of openssl rand -base64 48" }
  ]
}
```

and point `astrum.secrets.file` in `<data>/etc/astrum.properties` at it:

```properties
astrum.secrets.file=/astrum-data/etc/secrets.json
```

Do this before the first start of a new installation, so that no secret is ever stored with the
built-in key.

Keep this file with the backups of the database: a database restored without the key file it was
written with cannot decrypt its secrets. To rotate, add a new key to `keys`, make it `active` and
restart: new secrets use the active key, and existing ones stay readable as long as their key remains
in the list.

## astrum.properties

`<data>/etc/astrum.properties` overrides the defaults shipped in `<install>/etc/astrum-default.properties`.

| Property | Default | Meaning |
|---|---|---|
| `application-port` | `8081` | HTTP port |
| `application-host` | `0.0.0.0` | Address to listen on |
| `astrum-context-path` | `/` | Path the server is mounted under, for example `/astrum` |
| `astrum-args` | Jetty XML files | Jetty configuration to load; add `${jetty.etc}/jetty-https.xml` to serve HTTPS directly |
| `astrum.onboarding.enabled` | `true` | Show the onboarding wizard to the first administrator |
| `astrum.scripts.allowCreation` | `false` | Allow creating scripts through the scripting API |

Restart the server after editing the file.

Prefer a [reverse proxy](reverse-proxy.md) for TLS over serving HTTPS from Jetty: certificates are then
renewed without touching the server.

## Memory

The JVM needs a heap and direct memory. Direct memory backs network and file buffers and should be
at least as large as the heap.

| Machine memory | Heap | Direct memory |
|---|---|---|
| 8 GB | 2703 MB | 2703 MB |
| 16 GB | 4 GB | 6 GB |
| 32 GB | 8 GB | 12 GB |

Container image:

```shell
-e ASTRUM_JAVA_OPTS="-Xms4g -Xmx4g -XX:MaxDirectMemorySize=6g"
```

Distribution: put the same flags on the `java` command line or into `ASTRUM_JAVA_OPTS` of the systemd
environment file.

Set `-Xms` equal to `-Xmx` so the heap is reserved at start and the server does not compete with the
page cache later.

## Base URL

Links the server generates (in repository indexes, Docker auth challenges, emails and the UI) are
built from the incoming request. Behind a reverse proxy that is correct only when the proxy forwards
the original host and scheme (`Host` and `X-Forwarded-Proto`). If it cannot, set the base URL
explicitly under **Settings → System → Capabilities → Base URL**, for example
`https://astrum.example.com`.

## Outbound HTTP proxy

Proxy repositories and OpenID Connect use the outbound proxy configured under
**Settings → System → HTTP**. JVM proxy system properties and the `http_proxy` environment variables
are not used.

## Logging

The log level of any logger can be changed at runtime under **Settings → Support → Logging**. For a
permanent change copy `<install>/etc/logback/logback.xml` to `<data>/etc/logback/logback.xml`, edit it
and restart.
