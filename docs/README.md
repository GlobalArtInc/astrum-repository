# Astrum Repository documentation

Astrum Repository is a self-hosted artifact repository manager. One server hosts, proxies and groups
repositories for Alpine, Ansible Galaxy, APT, Bazel, Cargo, Chef Supermarket, Chromium extensions, CocoaPods, Composer, Conan, Conda, Docker and
other OCI artifacts, Firefox add-ons, Git LFS, Go, Helm, Hex, Hugging Face, JetBrains plugins, Maven, Nix, npm, NuGet, Open VSX, P2, pacman, Protobuf, pub, Puppet
Forge, PyPI, R, RubyGems, Swift, Terraform, Vagrant, Yum and raw files, and remote caches for Nx and Turborepo, with users, roles, content selectors, cleanup
policies, LDAP and OpenID Connect sign-in, and a web interface in twelve languages. It is a single program that
keeps its data in PostgreSQL and in blob stores on disk or in S3.

The same pages are published as a site at https://astrum.globalart.dev.

## Start

- [Quick start](getting-started.md)

## Install

- [System requirements](install/requirements.md)
- [Run the container image](install/container.md)
- [Build and run without a container](install/from-source.md)

## Configure

- [Runtime: configuration file, environment variables, data directory, base URL, logging](configure/runtime.md)
- [PostgreSQL](configure/postgresql.md)
- [Blob stores](configure/blob-stores.md)
- [Reverse proxy and TLS](configure/reverse-proxy.md)
- [Users, realms and sign-in](configure/authentication.md)
- [Outbound HTTP and proxy](configure/outbound-http.md)
- [SSL certificates](configure/ssl-certificates.md)
- [Capabilities](configure/capabilities.md)

## Repositories

- [Repositories](repositories/overview.md)
- [Cleanup policies](repositories/cleanup-policies.md)
- [Routing rules](repositories/routing-rules.md)
- [Formats](formats/README.md): one page per format with the settings of its clients

## Use

- [Web interface](use/web-interface.md)
- [Connect build tools to repositories](use/clients.md)
- [REST API](use/rest-api.md)
- [Agents over MCP](use/mcp.md)
- [Interface language](use/language.md)

## Operate

- [Tasks](operate/tasks.md)
- [Nodes and several servers](operate/nodes.md)
- [Monitoring and logs](operate/monitoring.md)
- [Backup and restore](operate/backup-restore.md)
- [Upgrade](operate/upgrade.md)

## Move from another installation

- [Move from another repository server](migrate/from-another-server.md)

## The site

`docs/` is also the source of the site. To preview it, in `docs/`:

```shell
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

The `Docs` workflow builds the site for every pull request that changes `docs/` and publishes it to
GitHub Pages from `main`. Pages are plain Markdown that read the same on GitHub; `index.md` is the home
page of the site, this file is the index on GitHub, and `formats/README.md` is served as `/formats/`. A
new page needs an entry in the sidebar in `.vitepress/config.mts` and in this list.
