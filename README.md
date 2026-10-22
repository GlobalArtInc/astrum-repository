# Astrum Repository

Astrum Repository is GlobalArt's artifact repository manager. It hosts, proxies and groups Alpine,
Ansible Galaxy, APT, Bazel, Cargo, Chef Supermarket, CocoaPods, Composer, Conan, Conda, Docker and other OCI artifacts, Git LFS,
Go, Helm, Hex, Hugging Face, JetBrains plugins, Maven, Nix, npm, NuGet, Open VSX, P2, pacman, Protobuf, pub, Puppet Forge, PyPI, R, RubyGems, Swift,
Terraform, Vagrant, Yum and raw repositories, and keeps remote caches for Nx and Turborepo. It is one server written in
Rust, with a web interface in twelve languages, that keeps its data in PostgreSQL and in blob stores on
disk or in S3.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-dark.png">
  <img alt="The Astrum Repository dashboard: downloads, components, storage and downloads per day" src="docs/images/dashboard-light.png">
</picture>

## Quick start

You need a PostgreSQL database for the server; see [PostgreSQL](docs/configure/postgresql.md).

```shell
podman run -d --name astrum -p 8081:8081 -v astrum-data:/astrum-data \
  -e ASTRUM_DATABASE_URL=postgres://astrum:change-me@db.example.com:5432/astrum \
  docker.io/globalartltd/astrum-repository:latest
podman exec astrum cat /astrum-data/admin.password
```

Open http://localhost:8081 and sign in as `admin` with that password.

Installation, configuration, operations and client setup are covered in the
[documentation](docs/README.md).

## Build

Requirements:

- the Rust toolchain named in `rust-toolchain.toml` (rustup installs it) and a C compiler
- Node.js 22 with corepack (pnpm is pinned in `web/package.json`)
- PostgreSQL 16 or newer, for the tests and for running the server

```shell
cargo build --release --locked --bin astrum
cd web && corepack pnpm install --frozen-lockfile && corepack pnpm build
```

or build the container image, which does both:

```shell
docker build -t astrum-repository:local .
```

## Run

```shell
export ASTRUM_DATABASE_URL=postgres://astrum:change-me@127.0.0.1:5432/astrum
cargo run --bin astrum -- --data-dir .data/astrum --listen 127.0.0.1:8081 --web-dir web/dist
```

The server listens on the given address and keeps its data in `.data/astrum`, which git ignores. The
first start creates the schema in the database and writes the generated administrator password to
`.data/astrum/admin.password`. For work on the interface, `ASTRUM_URL=http://127.0.0.1:8081 corepack pnpm dev`
in `web/` serves it with live reload and forwards the API to the running server; see
[web/README.md](web/README.md).

## Test

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
ASTRUM_TEST_DATABASE_URL=postgres://postgres:astrum@127.0.0.1:5432/postgres cargo test --workspace --locked

cd web
corepack pnpm lint && corepack pnpm typecheck && corepack pnpm test && corepack pnpm i18n:check
```

Every Rust test gets its own database on the server named by `ASTRUM_TEST_DATABASE_URL`. The end-to-end
tests of the web interface are described in [web/README.md](web/README.md).

## Layout

| Path | Holds |
|---|---|
| `crates/` | The server: `astrum-server` is the binary, `astrum-api` the REST API, `astrum-content` the repository engine, `astrum-format-<id>` one crate per format |
| `web/` | The web interface (React, TypeScript, Vite) |
| `docs/` | Installation, configuration, operation, client setup and migration guides |
| `tools/` | `i18n` for the translations of the interface, `repository-copy` for moving data from another repository server, `side-by-side` for comparing two servers |
| `Dockerfile` | Builds the container image from the two above |

## Translations

The web interface is translated in Ownlate: see the [progress of every language and help translate](https://platform.ownlate.com/workspaces/c08d5107-8ec7-4c80-9f46-e36fa7eeab3a/projects/f1f93c0e-4902-434d-a551-e94ba7e1e304/progress).
The image takes the approved translations when it is built; `tools/i18n` holds the tooling.

## License

See [LICENSE.txt](LICENSE.txt). [NOTICE.txt](NOTICE.txt) lists the third-party software the server and the
web interface use.
