# Runtime configuration

The server reads a few settings at start: where to listen, which database to use, where its data
directory is. Everything else (repositories, blob stores, users, realms, tasks, the base URL, the
outbound HTTP proxy, email) is stored in the database and changed in the web interface or through the
REST API, without a restart.

A setting at start comes from the first of these that sets it:

1. a command-line option (`--config`, `--data-dir`, `--listen`, `--web-dir`);
2. an environment variable;
3. the configuration file;
4. the default.

An environment variable that is empty counts as unset.

## Configuration file

The file is [TOML](https://toml.io). The server looks for `<data dir>/etc/astrum.toml` and starts
without one when it is missing. A file named with `--config` or `ASTRUM_CONFIG` must exist. Keys the
server does not know are refused, so a typo stops the start instead of being ignored.

```toml
listen = "0.0.0.0:8081"
data_dir = "/astrum-data"
secrets_key_file = "/astrum-data/etc/secrets.key"

[database]
url = "postgres://astrum:change-me@db.example.com:5432/astrum"
schema = "astrum"
max_connections = 20

[log]
filter = "info,sqlx=warn"
format = "text"

[web]
dir = "/usr/share/astrum/web"

[ldap]
cache_ttl = "5m"

[connectors]
tls_certificate = "/astrum-data/etc/tls/chain.pem"
tls_key = "/astrum-data/etc/tls/key.pem"
```

| Key | Environment variable | Option | Default | Meaning |
|---|---|---|---|---|
| | `ASTRUM_CONFIG` | `--config` | `<data dir>/etc/astrum.toml` | Path of the configuration file |
| `listen` | `ASTRUM_LISTEN` | `--listen` | `0.0.0.0:8081` | Address and port to listen on |
| `data_dir` | `ASTRUM_DATA_DIR` | `--data-dir` | `/astrum-data` | The [data directory](#data-directory) |
| `secrets_key_file` | `ASTRUM_SECRETS_KEY_FILE` | | `<data dir>/etc/secrets.key` | File with the [key that seals stored secrets](#secrets-key) |
| `database.url` | `ASTRUM_DATABASE_URL` | | none, required | PostgreSQL address, see [PostgreSQL](postgresql.md) |
| `database.schema` | `ASTRUM_DATABASE_SCHEMA` | | `astrum` | Schema for the tables; lower-case letters, digits and underscores |
| `database.max_connections` | `ASTRUM_DATABASE_MAX_CONNECTIONS` | | `20` | Most connections the server opens, at least 3 |
| `log.filter` | `ASTRUM_LOG` | | `info,sqlx=warn` | Log levels, see [logging](#logging) |
| `log.format` | `ASTRUM_LOG_FORMAT` | | `text` | `text` or `json` |
| `web.dir` | `ASTRUM_WEB_DIR` | `--web-dir` | `/usr/share/astrum/web` | Directory of the built web interface |
| `ldap.cache_ttl` | `ASTRUM_LDAP_CACHE_TTL` | | `5m` | How long LDAP users, their groups and successful sign-ins are cached; `90`, `90s`, `5m`, `2h`, `1d` or `0` |
| `connectors.tls_certificate` | `ASTRUM_CONNECTOR_TLS_CERTIFICATE` | | none | PEM certificate chain for Docker connectors with an HTTPS port |
| `connectors.tls_key` | `ASTRUM_CONNECTOR_TLS_KEY` | | none | PEM private key for it; set both or neither |

`ASTRUM_NODE_NAME` has no file key: it names this server in logs and under
**Settings → System → Nodes** and defaults to the host name.

The database password never appears in logs or in error messages.

## Data directory

Everything the server writes outside the database and the blob stores lives in one data directory:
`/astrum-data` in the container image. Back it up and keep it across upgrades.

```text
<data>/
  admin.password            generated admin password, removed after the first password change
  blobs/default/            the default file blob store
  etc/astrum.toml           the configuration file, if you create one
  etc/secrets.key           the key that seals stored secrets, created on the first start
  log/astrum-<node>.log     application log of this server
  downloads/                support ZIPs waiting for download, removed after a day
  tmp/                      temporary files, safe to clear while the server is stopped
```

Other blob stores live where their path points; see [blob stores](blob-stores.md).

## Secrets key

Passwords and keys that the server must hand on to other systems are kept in the database sealed with
AES-256-GCM: the passwords of proxy repositories, bearer tokens, signing keys, S3 secret keys, the LDAP
bind password, the OpenID Connect client secret, the email password, the passwords of outbound HTTP
proxies and the secrets of notification channels. The REST API never returns them.

The key is 32 random bytes in base64 in `<data>/etc/secrets.key` (or the file named by
`secrets_key_file`). The server creates it with restrictive permissions on the first start. A server
that finds a key file it cannot read, or that does not hold 32 bytes, refuses to start.

- Keep the file with the backups of the database: a database restored without the key it was written
  with cannot open its secrets. See [backup and restore](../operate/backup-restore.md).
- Several servers on one database must use the same key file.
- To keep the key out of the data directory, create the file in advance and point `secrets_key_file` at
  it.

## Sessions

A sign-in in the web interface creates a session stored in the database, tied to the cookies
`astrum_session` (HttpOnly) and `astrum_csrf`. Users stay signed in across restarts and upgrades, and
every server on the same database accepts the same cookie. A session ends when the user signs out,
changes the password, is deactivated or deleted, or after being idle for the session timeout.

Sessions do not expire by default. To sign idle users out, set **Session timeout** (minutes; `0` means
never) under **Settings → System → Capabilities → UI: Settings**. A user who fails to sign in ten
times within five minutes cannot sign in for a while and gets HTTP 429.

Command-line clients do not use sessions: they send the user name and password, an
[access token](authentication.md#access-tokens) or a format-specific token with every request.

## Base URL

Links the server generates (in repository indexes, Docker auth challenges, emails and the web
interface) are built from the incoming request. Behind a reverse proxy that is correct only when the
proxy forwards the original host and scheme (`X-Forwarded-Host` or `Host`, and `X-Forwarded-Proto`). If
it cannot, set the base URL explicitly under **Settings → System → Capabilities → Base URL**, for
example `https://astrum.example.com`; a base URL that is set wins over the address of the request.

## Outbound HTTP proxy

Proxy repositories use the outbound proxy configured under **Settings → System → HTTP**, see
[outbound HTTP and proxy](outbound-http.md). OpenID Connect, S3 blob stores and remote blob store agents
connect directly, and notification channels connect directly unless they name a proxy of their own. The
`HTTP_PROXY` and `HTTPS_PROXY` environment variables are not used.
Requests to loopback, private and other internal addresses are refused by the SSRF protection; the
page `/admin/security/ssrf-protection` (it has no tile in Settings) lists the addresses and domains
that stay allowed.

## Logging

The server writes its log to standard error and to `<data>/log/astrum-<node>.log` (32 MiB per file,
four rotated files; files of servers that stopped writing are removed after 30 days). Read it under
**Settings → Support → Logs**.

`log.filter` uses the [tracing filter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives):
a default level plus optional levels per module, for example `info,astrum_content=debug,sqlx=warn`. The
levels of single modules can also be changed at runtime under **Settings → Support → Logging**; those
overrides are stored in the database and survive restarts. `ASTRUM_LOG_FORMAT=json` writes one JSON object
per line to standard error for log collectors.

## Several servers on one database

Several servers can run against the same database and the same blob stores (a shared file system or
S3). They share the database, so repositories, content and settings are the same on all of them, and
they appear under **Settings → System → Nodes**; users and security settings changed through one server
reach the others only when they restart, see [nodes](../operate/nodes.md). Give each
one its own `ASTRUM_NODE_NAME` if the host names are not unique. Logs and support ZIPs belong to the
server that served the request. Upgrade all servers together, see
[upgrade](../operate/upgrade.md).
