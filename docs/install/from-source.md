# Build and run without a container

The server is one binary, `astrum`, plus the files of the web interface. Build both from the source
tree; the container image does exactly this, see the `Dockerfile` at the root.

## Build

You need the Rust toolchain named in `rust-toolchain.toml` (rustup installs it on the first `cargo` call),
a C compiler, and Node.js 22 with corepack for the web interface.

```shell
cargo build --release --locked --bin astrum

cd web
corepack pnpm install --frozen-lockfile
corepack pnpm build
```

The binary is `target/release/astrum` and the interface is the directory `web/dist`. To make the
binary report its commit, set `ASTRUM_GIT_REVISION=$(git rev-parse --short=7 HEAD)` for the `cargo build`.

## Install

```shell
useradd --system --home-dir /var/lib/astrum --shell /usr/sbin/nologin astrum
install -d -o astrum -g astrum /var/lib/astrum
install -D -m 0755 target/release/astrum /opt/astrum/bin/astrum
cp -r web/dist /opt/astrum/web
install -d -m 0750 -o root -g astrum /etc/astrum
```

`/etc/astrum/astrum.toml`, readable by the service user only because it holds the database password:

```toml
data_dir = "/var/lib/astrum"
listen = "127.0.0.1:8081"

[database]
url = "postgres://astrum:change-me@127.0.0.1:5432/astrum"

[web]
dir = "/opt/astrum/web"
```

```shell
chown root:astrum /etc/astrum/astrum.toml
chmod 0640 /etc/astrum/astrum.toml
```

See [runtime configuration](../configure/runtime.md) for every setting and
[PostgreSQL](../configure/postgresql.md) for preparing the database.

## Run it as a service

`/etc/systemd/system/astrum.service`:

```ini
[Unit]
Description=Astrum Repository
Wants=network-online.target
After=network-online.target postgresql.service

[Service]
User=astrum
Group=astrum
Environment=ASTRUM_CONFIG=/etc/astrum/astrum.toml
ExecStart=/opt/astrum/bin/astrum serve
Restart=on-failure
TimeoutStopSec=180
LimitNOFILE=65536
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
```

```shell
systemctl daemon-reload
systemctl enable --now astrum
journalctl -u astrum -f
```

The first start creates the schema, applies the migrations, creates the
`default` blob store under `/var/lib/astrum/blobs/default` and writes the password of the `admin` user
to `/var/lib/astrum/admin.password`. Sign in as described in
[run the container image](container.md#first-sign-in).

## Commands

| Command | Does |
|---|---|
| `astrum serve` | Brings the database schema up to date and serves requests; the default when no command is given |
| `astrum migrate` | Brings the database schema up to date and exits |
| `astrum check` | Exits with 0 when the server on the configured address answers `GET /service/rest/v1/status` with 200 |
| `astrum version` | Prints the version and the revision |

The options `--config`, `--data-dir`, `--listen` and `--web-dir` come before or after the command and
override the environment variables and the file.
