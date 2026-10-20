# Run the container image

The image is published on Docker Hub as `globalartltd/astrum-repository`.

| Tag | Meaning |
|---|---|
| `<YYYYMMDDHHMMSS>-<sha7>`, for example `20261018120000-3fa9c1e` | One build of the `main` branch; pin one of these in production |
| `next-<YYYYMMDDHHMMSS>-<sha7>` | The same build as the tag without `next-`, for test instances |
| `<version>`, for example `3.0.0` | A release |
| `latest` | The newest release |
| `edge` | The newest build of the `main` branch, for testing |

The image runs the server as uid/gid 200, listens on port 8081 and keeps all state under
`/astrum-data`. Mount a volume there; without it every restart starts from scratch. The server needs a
PostgreSQL database, see [PostgreSQL](../configure/postgresql.md).

## Quick start

```shell
mkdir -p /srv/astrum/data
chown 200:200 /srv/astrum/data

podman run -d --name astrum \
  -p 8081:8081 \
  -v /srv/astrum/data:/astrum-data:Z \
  -e ASTRUM_DATABASE_URL=postgres://astrum:change-me@db.example.com:5432/astrum \
  docker.io/globalartltd/astrum-repository:latest
```

`docker run` takes the same arguments (drop the `:Z` SELinux label if SELinux is not enforcing).

On start the server creates its schema in the database and applies the migrations before it listens.
It is ready when
`curl -fs http://localhost:8081/service/rest/v1/status/writable` returns HTTP 200. The image also
checks itself: `podman exec astrum astrum check` exits with 0 when the server answers.

## First sign-in

On the first start, when the database holds no users, the server creates the `admin` user with a random
password and writes it to `/astrum-data/admin.password`:

```shell
podman exec astrum cat /astrum-data/admin.password
```

Sign in at `http://<host>:8081` as `admin` with that password. The setup page asks for a new password,
deletes the file, lets you decide whether anonymous users may read repositories and offers to create the
usual repositories of the formats you pick:

| Format | Repositories |
|---|---|
| Docker | `docker-hosted` (redeploy allowed), `docker-hub` (proxy of Docker Hub), `docker` (group of both) |
| Maven | `maven-releases`, `maven-snapshots`, `maven-central` (proxy of Maven Central), `maven-public` (group) |
| npm | `npm-hosted`, `npm-proxy` (proxy of registry.npmjs.org), `npm` (group) |
| PyPI | `pypi-hosted`, `pypi-proxy` (proxy of pypi.org), `pypi` (group) |
| NuGet | `nuget-hosted`, `nuget.org-proxy` (proxy of the NuGet v3 feed), `nuget-group` (group) |
| Helm | `helm-hosted` |
| Go | `go-proxy` (proxy of proxy.golang.org) |
| Raw | `raw-hosted` (redeploy allowed) |

They land in the `default` blob store, and repositories with those names that exist already are left as
they are. Proxy repositories need the server to reach the internet, through **Settings → System → HTTP**
when it goes through a proxy.

## Configuration

The image sets `ASTRUM_DATA_DIR=/astrum-data`, `ASTRUM_LISTEN=0.0.0.0:8081` and
`ASTRUM_WEB_DIR=/usr/share/astrum/web`. Everything else is configured with environment variables or with
the file `/astrum-data/etc/astrum.toml`; see [runtime configuration](../configure/runtime.md) for all
settings. The ones a container usually needs:

| Variable | Default | Purpose |
|---|---|---|
| `ASTRUM_DATABASE_URL` | none, required | PostgreSQL address, `postgres://user:password@host:5432/database` |
| `ASTRUM_DATABASE_SCHEMA` | `astrum` | Schema for the tables, created on the first start |
| `ASTRUM_DATABASE_MAX_CONNECTIONS` | `20` | Upper bound of database connections |
| `ASTRUM_LOG` | `info,sqlx=warn` | Log filter, for example `info,astrum_content=debug` |
| `ASTRUM_LOG_FORMAT` | `text` | `json` writes one JSON object per line |
| `ASTRUM_NODE_NAME` | the host name | Name of this server in logs and under **Settings → System → Nodes** |

The variables are read at start. A secret such as the database password is better kept out of the
command line: use your runtime's secret mechanism (`podman secret` with `type=env`, Docker or
Kubernetes secrets as environment variables), or put it into a configuration file that only uid 200
can read and point `ASTRUM_CONFIG` at it:

```toml
[database]
url = "postgres://astrum:change-me@db.example.com:5432/astrum"
```

## Rootless podman

The data directory must be writable by uid 200 inside the container. With rootless podman map the
host user that owns the directory to that uid:

```shell
podman run -d --name astrum \
  --userns keep-id:uid=200,gid=200 \
  -p 8081:8081 \
  -v /srv/astrum/data:/astrum-data:Z \
  -e ASTRUM_DATABASE_URL=... \
  docker.io/globalartltd/astrum-repository:latest
```

### systemd with a quadlet

`~/.config/containers/systemd/astrum.container` for a rootless user, or
`/etc/containers/systemd/astrum.container` for a system service:

```ini
[Unit]
Description=Astrum Repository
Wants=network-online.target
After=network-online.target

[Container]
Image=docker.io/globalartltd/astrum-repository:20261018120000-3fa9c1e
ContainerName=astrum
PublishPort=8081:8081
Volume=/srv/astrum/data:/astrum-data:Z
UserNS=keep-id:uid=200,gid=200
Secret=astrum-database-url,type=env,target=ASTRUM_DATABASE_URL
Environment=ASTRUM_LOG_FORMAT=json

[Service]
Restart=always
TimeoutStopSec=180
LimitNOFILE=65536

[Install]
WantedBy=default.target
```

```shell
printf '%s' 'postgres://astrum:change-me@db.example.com:5432/astrum' | podman secret create astrum-database-url -
systemctl --user daemon-reload
systemctl --user start astrum
```

The image defines its own health check (`astrum check`), which Quadlet and `podman ps` use. Give the
service time to stop: on SIGTERM the server finishes in-flight requests, stops its tasks and saves the
download counters, so `TimeoutStopSec` should be well above the default 10 seconds.

## Compose with PostgreSQL

```yaml
services:
  postgres:
    image: docker.io/library/postgres:18
    environment:
      POSTGRES_DB: astrum
      POSTGRES_USER: astrum
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets: [db_password]
    volumes:
      - postgres:/var/lib/postgresql
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "astrum", "-d", "astrum"]
      interval: 10s

  astrum:
    image: docker.io/globalartltd/astrum-repository:latest
    depends_on:
      postgres:
        condition: service_healthy
    ports:
      - "8081:8081"
    environment:
      ASTRUM_CONFIG: /run/secrets/astrum_config
    secrets: [astrum_config]
    volumes:
      - astrum:/astrum-data
    stop_grace_period: 3m

secrets:
  db_password:
    file: ./db_password.txt
  astrum_config:
    file: ./astrum.toml

volumes:
  postgres:
  astrum:
```

with `astrum.toml` next to the compose file holding the same password as `db_password.txt`:

```toml
[database]
url = "postgres://astrum:change-me@postgres:5432/astrum"
```

Compose secrets are mounted read-only and readable by every user in the container, uid 200 included.

## Building the image yourself

```shell
docker build -t astrum-repository:local .
```

The Dockerfile at the root of the source tree has three stages: it builds the web interface with Node.js
and pnpm, builds the server with the Rust toolchain of `rust-toolchain.toml`, and copies both into a
small Debian image. A full build compiles the whole dependency tree and takes 10 to 20 minutes the first
time; BuildKit cache mounts make later builds faster. `--build-arg VERSION=...` and
`--build-arg GIT_REVISION=...` set the image label and the revision the server reports.
