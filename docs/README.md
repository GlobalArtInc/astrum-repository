# Astrum Repository documentation

Astrum Repository is a self-hosted artifact repository manager. One server hosts, proxies and groups
repositories for Alpine, APT, Cargo, CocoaPods, Composer, Conan, Conda, Docker, Git LFS, Go, Helm,
Hugging Face, Maven, npm, NuGet, P2, PyPI, R, RubyGems, Swift, Terraform, Yum and raw files, with users,
roles, content selectors, cleanup policies and OpenID Connect sign-in.

## Install

- [System requirements](install/requirements.md)
- [Run the container image](install/container.md)
- [Run the distribution without a container](install/distribution.md)

## Configure

- [Runtime: data directory, ports, memory, base URL](configure/runtime.md)
- [PostgreSQL](configure/postgresql.md)
- [Reverse proxy and TLS](configure/reverse-proxy.md)
- [Users, realms and sign-in](configure/authentication.md)

## Operate

- [Backup and restore](operate/backup-restore.md)
- [Upgrade](operate/upgrade.md)
- [Monitoring and logs](operate/monitoring.md)

## Use

- [Connect build tools to repositories](use/clients.md)
- [Interface language](use/language.md)

## Move from Nexus Repository

- [Migrate from Nexus Repository 3](migrate/from-nexus.md)
