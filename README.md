# Astrum Repository

Astrum Repository is GlobalArt's artifact repository manager. It serves Alpine, APT, Cargo, CocoaPods,
Composer, Conan, Conda, Docker, Go, Helm, Maven, npm, NuGet, P2, PyPI, R and raw repositories on an
embedded H2 database.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/dashboard-dark.png">
  <img alt="The Astrum Repository dashboard: downloads, components, storage and downloads per day" src="docs/images/dashboard-light.png">
</picture>

## Quick start

```shell
podman run -d --name astrum -p 8081:8081 -v astrum-data:/astrum-data \
  docker.io/globalartltd/astrum-repository:latest
podman exec astrum cat /astrum-data/admin.password
```

Open http://localhost:8081 and sign in as `admin` with that password.

Installation, configuration, operations and client setup are covered in the
[documentation](docs/README.md).

## Build

Requirements:

- Java 25+
- Maven 3.9.6+
- Node.js with corepack (Yarn 4 is pinned in `package.json`)

```shell
corepack enable
corepack yarn install
mvn clean install -Ppublic -DskipTests
```

Yarn uses the `node-modules` linker (`.yarnrc.yml`); Plug'n'Play breaks the frontend build.

## Run

```shell
hack/run.sh
```

The server listens on port 8081 and keeps its data in `.data/astrum3` (override with `ASTRUM_DATA`),
outside `target/`, so rebuilds do not wipe it. The first start writes the generated administrator
password to `.data/astrum3/admin.password`.

## License

Astrum Repository is a modified version of Nexus Repository OSS and is distributed under the
[Eclipse Public License 1.0](LICENSE.txt). See [NOTICE.txt](NOTICE.txt) for attribution and the parts
also available under the MIT License.
