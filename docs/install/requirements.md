# System requirements

## Sizing

The server is one native process with no heap or runtime to size. Start small and watch the load.

| Workload | CPU | Memory for the server |
|---|---|---|
| Small team, up to ~20 repositories | 2 cores | 2 GB |
| Department, tens of repositories, CI traffic | 4 cores | 4 GB |
| Large, hundreds of repositories, heavy proxy and Docker traffic | 8+ cores | 8 GB |

These are starting points, not measured limits. PostgreSQL needs its own share of the machine when it
runs on the same host; leave memory for the operating system's page cache as well, because it serves
the hot artifacts.

## Disk

- **Blob stores** hold the artifacts themselves and need most of the space. Size them for the hosted
  content plus the proxy caches and keep about 20% free; when the disk under a blob store fills up,
  uploads and proxy downloads into it fail. A soft quota on the blob store warns before that happens;
  see [blob stores](../configure/blob-stores.md).
- **Data directory** (`/astrum-data` in the container) holds the default file blob store, logs,
  temporary files, the secrets key and the configuration file. Put it on local, low-latency storage;
  network file systems are only suitable for blob stores.
- Use a file system with enough inodes; Docker and npm repositories create many small files.

## Software

- **PostgreSQL 16 or newer.** It is the only database; there is no embedded one. The server is tested
  against PostgreSQL 16 and 18. See [PostgreSQL](../configure/postgresql.md).
- A **container runtime** (Docker, podman, Kubernetes) for the published image, or the Rust and Node.js
  toolchains to build from source; see [build and run without a container](from-source.md).
- A **reverse proxy** that terminates TLS (nginx, Envoy, Caddy, HAProxy or a Kubernetes gateway); see
  [reverse proxy and TLS](../configure/reverse-proxy.md).

## Operating system limits

The server keeps many files and sockets open. Raise the open file limit of the service user to at
least 65536:

```ini
# systemd unit
[Service]
LimitNOFILE=65536
```

The container image inherits the limit from the container runtime; podman and Docker default to a
high enough value.

## Network

| Port | Purpose |
|---|---|
| 8081/tcp | HTTP for the web interface, the REST API and all repositories, including Docker through `/v2` |
| additional ports | Optional per-repository Docker connectors, only when a Docker repository is configured with its own HTTP or HTTPS port |

The server answers at the root of its host name; it cannot be mounted under a path prefix.

Outbound HTTPS is needed for proxy repositories, OpenID Connect providers, webhooks and S3. If outbound
traffic goes through a proxy, configure it under **Settings → System → HTTP**; the server does not use
the `HTTP_PROXY` and `HTTPS_PROXY` environment variables.
