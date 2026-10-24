# Nodes and several servers

Several servers can run against one database and the same blob stores. How to set them up (node names,
the shared secrets key, upgrading them together) is in
[runtime configuration](../configure/runtime.md#several-servers-on-one-database). This page describes the
**Nodes** page and what the servers share and what each one keeps for itself.

## The Nodes page

**Settings → System → Nodes** lists the servers that run against this database. Every server records
itself in the database every 15 seconds, and the page shows those heard from within the last minute,
in the order they started:

| Column | Meaning |
|---|---|
| **Node Name** | `ASTRUM_NODE_NAME`, or the host name when it is not set; **Current node** marks the server that answered the page |
| **Node Identity** | A random ID the server takes at every start |

The note above the list says whether the server runs alone or how many servers share the database. A
server that stops cleanly takes itself off the list at once; one that crashed drops off after a minute.
The page needs the privilege `as-settings-read`.

## What the servers share

Whatever is stored in the database is shared: repositories and their content, blob store
configurations, cleanup policies and routing rules, tasks and their runs, capabilities, the email server,
notification channels and the notifications waiting to be sent, web sessions and the audit log. Settings
that are read often are kept for a few seconds by each server, so a change made through one server
reaches the others after at most:

| Setting | Delay |
|---|---|
| Base URL, domains of repositories | 5 seconds |
| [HTTP settings](../configure/outbound-http.md), [SSL certificates](../configure/ssl-certificates.md), SSRF protection, routing rules | 10 seconds |
| Log levels set under **Settings → Support → Logging** | 30 seconds |

A web session works on every server, so a load balancer needs no sticky sessions. Use
`GET /service/rest/v1/status/writable` as its health check, see [monitoring](monitoring.md#health-endpoints).

## What each server keeps for itself

- **Users and security settings.** Each server loads users, roles, privileges, role mappings, content
  selectors, realms, anonymous access, the session timeout, the LDAP servers, OpenID Connect and the
  default role into memory when it starts, and loads them again when they are changed through that
  server. The other servers keep what they loaded until they restart: a user created, a role changed or a
  password changed through one server is unknown or outdated on the others, and the web session of such a
  user can be ended when one of its requests reaches another server. Make such changes through one server
  and restart the others afterwards, one at a time.
- **Caches.** The LDAP cache (`ldap.cache_ttl`), the cache of answers a remote did not have and the
  automatic blocking of proxy repositories whose remote fails are kept per server; **Clear Cache** on the
  LDAP page clears the cache of the server that serves the request.
- **Failed sign-ins.** Ten failed sign-ins within five minutes block a user on the server that counted
  them, not on the others.
- **Logs, support ZIPs and metrics** belong to the server that wrote them or served the request; see
  [runtime configuration](../configure/runtime.md#logging) and [monitoring](monitoring.md#prometheus-metrics).
- **Docker connector ports.** Every server opens the ports of the Docker repositories that have one, so the
  load balancer can send those ports to any server.
- **Paths of file blob stores.** A relative path is resolved below `<data>/blobs` of each server, so every
  server must find the same files there: mount the same shared directory on every server, give the store an
  absolute path on shared storage, or use S3; see [blob stores](../configure/blob-stores.md).

## Tasks on several servers

Every server runs a scheduler that looks for due tasks every five seconds. A due task is queued once,
whichever server sees it first, and the run is taken by any server with a free slot; a lock in
PostgreSQL makes sure that a task runs on one server at a time. Each server runs at most four tasks at
once.

- **Stop** reaches the server running the task through the database, within a few seconds.
- When a server stops in the middle of a run, another server, or the same one after its restart, marks the
  run failed once the database has closed the stopped server's connection; every server checks for such
  runs every 30 seconds. The task can then run again.
- The health check **Scheduler** and the metric `astrum_scheduler_last_tick_timestamp_seconds` describe the
  server that answers.

Notifications are sent by one server at a time, and the blob store space alert checks the stores on one
server at a time, so neither is sent twice. See [tasks](tasks.md) for running and stopping tasks.
