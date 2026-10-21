# Astrum Repository documentation

Astrum Repository is a self-hosted artifact repository manager. One server hosts, proxies and groups
repositories for Alpine, Ansible Galaxy, APT, Cargo, Chef Supermarket, CocoaPods, Composer, Conan, Conda, Docker and
other OCI artifacts, Git LFS, Go, Helm, Hex, Hugging Face, Maven, Nix, npm, NuGet, Open VSX, P2, pacman, pub, Puppet
Forge, PyPI, R, RubyGems, Swift, Terraform, Vagrant, Yum and raw files, and remote caches for Nx and Turborepo, with users, roles, content selectors, cleanup
policies, LDAP and OpenID Connect sign-in, and a web interface in twelve languages. It is a single program that
keeps its data in PostgreSQL and in blob stores on disk or in S3.

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

## Operate

- [Backup and restore](operate/backup-restore.md)
- [Upgrade](operate/upgrade.md)
- [Monitoring and logs](operate/monitoring.md)

## Use

- [Connect build tools to repositories](use/clients.md)
- [Interface language](use/language.md)
- [Agents over MCP](use/mcp.md)

## Move from another installation

- [Move from another repository server](migrate/from-another-server.md)
