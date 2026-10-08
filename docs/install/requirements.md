# System requirements

## Sizing

| Workload | CPU | Memory | Heap and direct memory |
|---|---|---|---|
| Small team, up to ~20 repositories and a few thousand components | 4 cores | 8 GB | 2703 MB each |
| Department, tens of repositories, CI traffic | 8 cores | 16 GB | 4 GB heap, 6 GB direct memory |
| Large, hundreds of repositories, heavy proxy and Docker traffic | 12+ cores | 32 GB | 8 GB heap, 12 GB direct memory |

Heap and direct memory are set together through `ASTRUM_JAVA_OPTS` (container) or the JVM options of
the service (distribution); see [runtime configuration](../configure/runtime.md#memory). Leave at least
a quarter of the machine's memory to the operating system and its page cache.

## Disk

- **Blob stores** hold the artifacts themselves and need most of the space. Size them for the hosted
  content plus the proxy caches and keep about 20% free; when the disk under a blob store fills up,
  uploads and proxy downloads into it fail. A soft quota on the blob store warns before that happens.
- **Data directory** (`/astrum-data` in the container) holds the embedded H2 database when PostgreSQL
  is not used, logs, temporary files and the default file blob store. Put it on local, low-latency
  storage; network file systems are only suitable for blob stores.
- Use a file system with enough inodes; Docker and npm repositories create many small files.

## Software

- **Java 25** or newer when running the distribution. The container image ships its own JRE.
- **PostgreSQL 14 or newer** for production databases; Astrum is developed and tested against
  PostgreSQL 18. The embedded H2 database is meant for evaluation and small single-node
  installations; see [PostgreSQL](../configure/postgresql.md).
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
| 8081/tcp | HTTP for the UI, the REST API and all repositories, including Docker through `/v2` |
| additional ports | Optional per-repository Docker connectors, only when a Docker repository is configured with its own HTTP port |

Outbound HTTPS is needed for proxy repositories and for OpenID Connect providers. If outbound traffic
goes through a proxy, configure it under **Settings → System → HTTP**.
