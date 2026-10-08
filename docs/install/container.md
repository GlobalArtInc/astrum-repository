# Run the container image

The image is published on Docker Hub as `globalartltd/astrum-repository`.

| Tag | Meaning |
|---|---|
| `latest` | The newest release |
| `<version>`, for example `3.96.4-01` | A specific release; pin this in production |
| `edge` | The newest build of the `main` branch, for testing |

The image runs the server as uid/gid 200, listens on port 8081 and keeps all state under
`/astrum-data`. Mount a volume there; without it every restart starts from scratch.

## Quick start

```shell
mkdir -p /srv/astrum/data
chown 200:200 /srv/astrum/data

podman run -d --name astrum \
  -p 8081:8081 \
  -v /srv/astrum/data:/astrum-data:Z \
  docker.io/globalartltd/astrum-repository:latest
```

`docker run` takes the same arguments (drop the `:Z` SELinux label if SELinux is not enforcing).

Startup usually takes under a minute. The server is ready when
`curl -fs http://localhost:8081/service/rest/v1/status/writable` returns HTTP 200.

## First sign-in

On the first start the server generates the password of the `admin` user and writes it to
`/astrum-data/admin.password`:

```shell
podman exec astrum cat /astrum-data/admin.password
```

Sign in at `http://<host>:8081` as `admin` with that password. The onboarding wizard asks for a new
password, deletes the file and lets you decide whether anonymous users may read repositories.

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `ASTRUM_JAVA_OPTS` | `-Xms2703m -Xmx2703m -XX:MaxDirectMemorySize=2703m` | JVM options; see [memory](../configure/runtime.md#memory) |
| `ASTRUM_DATA` | `/astrum-data` | Data directory inside the container |
| `ASTRUM_DATASTORE_ASTRUM_JDBCURL` | unset (embedded H2) | PostgreSQL JDBC URL; see [PostgreSQL](../configure/postgresql.md) |
| `ASTRUM_DATASTORE_ASTRUM_USERNAME` | unset | PostgreSQL user |
| `ASTRUM_DATASTORE_ASTRUM_PASSWORD` | unset | PostgreSQL password |
| `ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE` | unset | File to read the PostgreSQL password from, for secrets mounted as files |
| `ASTRUM_DATASTORE_ASTRUM_MAXIMUMPOOLSIZE` | `30` | Upper bound of database connections |

Pass secrets such as the database password through your runtime's secret mechanism
(`podman secret`, Docker secrets, Kubernetes secrets) rather than on the command line.

## Rootless podman

The data directory must be writable by uid 200 inside the container. With rootless podman map the
host user that owns the directory to that uid:

```shell
podman run -d --name astrum \
  --userns keep-id:uid=200,gid=200 \
  -p 8081:8081 \
  -v /srv/astrum/data:/astrum-data:Z \
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
Image=docker.io/globalartltd/astrum-repository:3.96.4-01
ContainerName=astrum
PublishPort=8081:8081
Volume=/srv/astrum/data:/astrum-data:Z
UserNS=keep-id:uid=200,gid=200
Environment=ASTRUM_JAVA_OPTS=-Xms2703m -Xmx2703m -XX:MaxDirectMemorySize=2703m
Environment=ASTRUM_DATASTORE_ASTRUM_JDBCURL=jdbc:postgresql://db.example.com:5432/astrum
Environment=ASTRUM_DATASTORE_ASTRUM_USERNAME=astrum
Secret=astrum-db-password,type=env,target=ASTRUM_DATASTORE_ASTRUM_PASSWORD
HealthCmd=curl -fs http://localhost:8081/service/rest/v1/status/writable
HealthStartPeriod=3m

[Service]
Restart=always
TimeoutStopSec=180
LimitNOFILE=65536

[Install]
WantedBy=default.target
```

```shell
printf '%s' 'the-database-password' | podman secret create astrum-db-password -
systemctl --user daemon-reload
systemctl --user start astrum
```

Give the service time to stop: the server finishes in-flight uploads and flushes its state on
shutdown, so `TimeoutStopSec` should be well above the default 10 seconds.

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
      ASTRUM_DATASTORE_ASTRUM_JDBCURL: jdbc:postgresql://postgres:5432/astrum
      ASTRUM_DATASTORE_ASTRUM_USERNAME: astrum
      ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE: /run/secrets/db_password
    secrets: [db_password]
    volumes:
      - astrum:/astrum-data
    stop_grace_period: 3m

secrets:
  db_password:
    file: ./db_password.txt

volumes:
  postgres:
  astrum:
```

## Building the image yourself

```shell
docker build -t astrum-repository:local .
```

The Dockerfile at the root of the source tree builds the server with Maven inside a JDK stage and
copies the result into a JRE image. A full build downloads the Maven and npm dependencies and takes
10 to 20 minutes on the first run; BuildKit cache mounts make later builds faster.
