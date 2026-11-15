# Replication

Replication copies hosted components between two Astrum servers. A **push** replication sends from the
**sender** to a hosted repository of the **receiver**: the sender connects, and nothing on the receiver
fetches from the sender. A **pull** replication does the opposite: the local server fetches components from
a repository of the **remote** and the remote never connects back (see [pull replication](#pull-replication)).
A **remote** is the other server, the receiver of a push or the source of a pull. A **replication** copies one
hosted repository of one side to one repository of the remote. The two servers are configured separately: a
push needs a service user and a token on the receiver, a pull needs a service user and a token on the remote.

Settings are under **Settings → Repository → Replication**, with the tabs **Remotes** and
**Replications**. The same settings are available over REST under `/service/rest/v1/replication/remotes`
and `/service/rest/v1/replications`. The status, repair and reconcile routes are described in
[status and repair](#status-and-repair).

## What replication is

The unit of transfer is a **component**, as the format names it: one version of one Maven artifact with
all its files, one Helm chart version, or one file of a raw repository. A change to a component queues it;
the sender then sends the whole component, not only the changed file.

Replication has two uses:

- **Disaster recovery copy.** Turn on **Propagate deletes**. A component deleted on the sender is deleted
  on the receiver too.
- **Delivery to another site.** Leave **Propagate deletes** off. The receiver keeps what was sent, and
  deletes on the sender do not reach it.

What is replicated:

| Format | Component | Notes |
|---|---|---|
| `maven2` | one group ID, artifact ID and version | All files of the version. The receiver skips checksum files and the archetype catalog and index and derives its own |
| `helm` | one chart name and version | The chart package |
| `raw` | one file | The namespace is the directory and the name is the full path; the version is empty |

Only these three formats have a transfer on this server. The receiver's repository must have the same
format as the source (`maven2` to `maven2`, and so on); a pull has the same rule for its local repository.

What is not replicated:

- Proxy and group repositories. The server refuses them as a source.
- Docker repositories. The form refuses them as a source with the reason `Docker images cannot be replicated yet: their
  transfer needs the image closure`. Protobuf repositories are refused with `the format protobuf takes no part in
  replication`.
- npm repositories. The form refuses them with the reason `npm packages cannot be replicated yet: the replication
  worker lacks the read rights the bundle of a version requires, dist-tag changes raise no replication event and the
  unpublish of a version fails the fingerprint check`. Use [promotion](../use/promotion.md) to move npm packages
  between instances.
- Components of other hosted formats. The server accepts the replication, but every component fails
  permanently with the reason `the format <id> takes no part in replication`; see [dead items](#dead-items).
- Repository settings, cleanup policies, routing rules and write policies, and users, roles and tokens.
- Deletes, unless **Propagate deletes** is on.
- Attribute-only changes, such as a pypi yank. The transfer of pypi, maven2, helm and raw does not carry component
  attributes; see [what replication does not do](#what-replication-does-not-do).
- Writes that the receiver makes for a replication, and writes a pull makes into its local repository. They are
  not captured, so they are not sent back.

Components stored before the replication was created are sent by its first reconcile run, which is queued
when the replication is created (see [reconcile](#reconcile)). Changes made while the replication is
disabled are not queued; when it is enabled again, a reconcile run sends what differs.

## Prepare the receiver

This section is for a push. The requirements of a pull replication are in
[pull replication](#pull-replication).

The receiver needs a service user that holds the replication privilege and the rights on the target
repository. Nothing in the catalogue grants them to a built-in role, so create a role for them.

1. **The privilege.** `as-replication-receive` (the permission `astrum:replication:receive`, described as
   "Receive replicated components from another server") lets its holder use every receiver route. Every
   holder is trusted equally: it can create, replace and delete replicated components in any hosted
   repository where it also holds the repository rights, and it can name any origin (see
   [security notes](#security-notes)). Only `as-admin` holds it by default, through all privileges.
2. **The repository rights.** Repository privileges are named `as-repository-view-<format>-<repository>-<action>`
   and exist for every repository. The service user needs, on the target repository:

   | Privilege | Needed for |
   |---|---|
   | `…-read` | Every receiver route; it checks which components and blobs are already present |
   | `…-add` | Uploading blobs and committing components |
   | `…-delete` | Deleting components (tombstones) |
   | `…-edit` | Replacing an existing version under **Allow Redeploy**; without it a changed version is refused |

   Each route needs its action on the whole repository. A content selector grants only part of a
   repository, and the receiver refuses partial grants.
3. **The role and the user.** For example, `replica-receiver` with the privileges
   `as-replication-receive`, `as-repository-view-raw-releases-replica-read`,
   `as-repository-view-raw-releases-replica-add` and `as-repository-view-raw-releases-replica-delete`,
   and a local user `replica-sender` with that role. The user needs a local password, because access
   tokens are created after a sign-in with a password (see [access tokens](authentication.md#access-tokens)).
4. **The token.** Sign in as `replica-sender` and create an access token under **Account menu → Access
   Tokens** with the access level **Read and write**. A read-only token is refused on every receiver route,
   because the receiver route needs a write permission. The token is shown once and is then stored on the
   sender, see [remotes](#remotes).
5. **The target repository.** It must be hosted, and its write policy decides what happens when a version
   that already exists on the receiver arrives with different content:

   | Write policy (UI label) | Stored as | What a changed existing version does |
   |---|---|---|
   | Allow Redeploy | `ALLOW` | Replaces it, but only when the service user also holds `…-edit`. Without `…-edit` it is refused as a conflict |
   | Disable Redeploy | `ALLOW_ONCE` | Refuses it with a conflict. New versions are accepted |
   | Read-Only | `DENY` | Refuses every upload and commit as read-only |

   A conflict or a refusal is permanent on the sender: the item becomes dead (see [dead items](#dead-items)).

## Remotes

**Settings → Repository → Replication → Remotes**, or `POST /service/rest/v1/replication/remotes`.
Listing and reading need `as-replication-read`; creating needs `as-replication-create`; changing and
checking need `as-replication-update`; deleting needs `as-replication-delete`.

| Field (UI label, REST name) | Rules |
|---|---|
| **Name**, `name` | 1 to 100 characters, no control characters. Unique ignoring case |
| **URL**, `url` | The address of the other server (the receiver of a push, the source of a pull), for example `https://astrum.example.com:8443`. See the rules below |
| **Token**, `token` | Required when a remote is created. 1 to 1024 characters, no spaces and no non-ASCII characters. Write-only |
| **Use the trust store**, `useTrustStore` | Off by default. On: connections trust the certificates of the [truststore](ssl-certificates.md#connections-that-use-the-truststore). If the truststore cannot be read, the system roots apply and a warning is logged |
| **Allow private network addresses**, `allowPrivateNetwork` | Off by default. See the private network rules below |
| **Max concurrency**, `maxConcurrency` | Workers that send components to this remote at the same time, from 1 to 16, default 2. See [workers](#remotes-workers-and-leases) |
| **Bandwidth limit, MiB/s** (UI), `maxBytesPerSecond` (REST) | Limit of the blob transfers with this remote, in bytes per second; empty or `null` means no limit. Uploads are throttled on a push, downloads on a pull |

The read-only fields `hasToken`, `remoteInstanceId`, `remoteVersion`, `checkedAt`, `failures`,
`suspendedUntil` and `lastError` are set by the server. The last four describe the remote's health, see
[failures and retries](#failures-and-retries).

### URL rules

- The URL holds a scheme, a host and an optional port, and nothing else: no path (a single `/` is
  accepted), no user name or password, no query and no fragment. It may not contain spaces, `@`, `\`, `?`
  or `#`, and it may be at most 2048 characters. The server stores it without the default port and without
  the trailing slash.
- `https` is expected and always accepted. Other schemes are refused.
- `http` is accepted only when **Allow private network addresses** is on, and only for a private or loopback
  address, or for a host name that resolves to such addresses. The reason is that the token is sent with
  every request and must not cross the internet in clear text. Loopback is still refused by the SSRF
  check, see below, so in practice `http` is for a receiver on a private network.
- A literal IP address on the internet is refused over `http`; use `https`.

### Private network addresses

With **Allow private network addresses** on, this remote may resolve to:

- RFC 1918 addresses (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`);
- `100.64.0.0/10` (carrier-grade NAT);
- IPv6 unique local addresses (`fc00::/7`), except the cloud metadata address `fd00:ec2::254`.

These stay refused even with the switch on: loopback (`127.0.0.0/8` and `::1`), link-local
(`169.254.0.0/16` and `fe80::/10`), site-local `fec0::/10`, `0.0.0.0/8`, the unspecified address,
multicast, broadcast and the metadata address `169.254.169.254`. The switch applies to this remote only.
The global SSRF exceptions still apply, see [SSRF protection](outbound-http.md#ssrf-protection).

### SSRF checks

- **On save.** A new remote, or a change of the URL or of the private network switch, is checked against
  the SSRF protection. The save is refused when the host resolves only to addresses the remote may not
  use. A name that does not resolve at the moment of the save is accepted; the connection check then fails.
- **At connect time.** Every connection resolves the host again and uses only the addresses the rules
  allow. When none is allowed, the connection is refused, and the sender treats it as a remote failure
  (see [failures and retries](#failures-and-retries)).
- **Through a proxy.** When the HTTP settings have a proxy for the remote's scheme (see
  [outbound HTTP and proxy](outbound-http.md#proxy-server)), the proxy connects to the receiver. Astrum still
  resolves the receiver's host name itself before each request, and refuses the request when any resolved
  address is blocked. A name Astrum cannot resolve is a temporary failure; the request is not sent to the
  proxy. The proxy host itself is allowed even when it has a private address.
- **Redirects** are not followed. A redirect answer is an error.
- The connection timeouts and retries of the HTTP settings do not apply. Replication uses its own: a 10
  second connection timeout, a 60 second limit on control requests, a 120 second limit without progress
  during a blob upload, and its own retry schedule.

### Updating a remote

- Changing the URL requires the token again in the same request: the old token is not sent to a new address.
  Without a new token the change is refused with `token: must be given again when the url changes`.
- A change of the URL clears the recorded instance ID and version of the remote; the next check or send
  records them again.
- Any update of the remote, even one that changes only its name, clears its failure count, its suspension
  and its last error at once. The sender resumes the remote within about 30 seconds; see
  [suspension](#suspension).
- Leave the token out to keep the stored one.

### Check connection

**Check connection** in the Remotes tab, or `POST /service/rest/v1/replication/remotes/{id}/check`. The
server calls `GET /service/rest/v1/replication/capabilities` on the receiver. The answer is:

```json
{"ok": true, "protocol": 1, "instanceId": "0b8e6f0a-...", "version": "3.0.4", "formats": ["helm", "maven2", "raw"]}
```

`formats` lists the formats the receiver can take. A failed check answers `200` with `"ok": false` and an
`error` object: `kind` is `retryable`, `permanent` or `refused`, and `message` holds the first 200
characters of the reason. A successful check also stores the receiver's instance ID, its version and the
time of the check (**Last checked**, **Remote version** in the table). A failed check stores only the time.

### Deleting a remote

A remote that a replication uses cannot be deleted. The server answers `409` with the names of the
replications that use it, for example `The remote is used by these replications: releases-to-replica`.
Delete those replications first.

## Replications

**Settings → Repository → Replication → Replications**, or `POST /service/rest/v1/replications`. Listing
and reading need `as-replication-read`. Creating needs `as-replication-create` and, for the source
repository, the privilege `as-repository-admin-<format>-<repository>-read`; a pull also needs
`as-repository-admin-<format>-<repository>-edit` on its local repository. Changing and deleting need the
update or delete privilege and the same repository privilege for the current source.

| Field (UI label, REST name) | Rules |
|---|---|
| **Name**, `name` | 1 to 100 characters, no control characters. Unique ignoring case, across replications |
| **Direction**, `direction` | `push` (default) or `pull`. Fixed at creation; see [the direction field](#the-direction-field) |
| **Source repository**, `sourceRepository` | An existing hosted repository. Proxy and group repositories are refused. For a pull this is the local repository that receives the components. Deleting the repository deletes its replications; renaming it renames the source |
| **Remote**, `remoteId` | The ID of an existing remote |
| **Remote repository**, `remoteRepository` | Letters, digits, `-`, `_` and `.`, starting with a letter, a digit or `-`; at most 200 characters. `remotes` and `capabilities` are refused, because the receiver uses them for its protocol routes. For a pull this is the repository of the remote that is read. Not checked against the other server on save: when it does not know the name, every item fails permanently with `404` |
| **Enabled**, `enabled` | Default on. A disabled replication takes no new changes and sends nothing; its queued items wait until it is enabled again |
| **Name filter**, `nameFilter` | Empty means every component. See the filter rules below |
| **Version filter**, `versionFilter` | Empty means every version. See the filter rules below |
| **Propagate deletes**, `propagateDeletes` | Default off. See [deletes](#deletes) |
| **Settle time, seconds**, `settleSeconds` | 0 to 3600, default 30. See [how it works](#how-it-works). A pull replication fetches without a settle time; the alert rule still reads this value |
| **Reconcile every, minutes**, `reconcileMinutes` | One of 5, 10, 15, 20, 30, 60, 120, 180, 240, 360, 480, 720 or 1440. Default 360 when left out. Other values are refused. See [reconcile](#reconcile) |
| **Alert channels**, `alertChannels` | A list of notification channel IDs, each of which must exist. Messages go to the enabled channels when the replication changes its alert state, see [alerts and metrics](#alerts-and-metrics) |

The read-only fields `lastSuccessAt`, `lastFailureAt`, `lastError`, `lastReconcileAt` and `alertState` are
set by the server:

- `lastError` holds the reason of the most recent failure of the replication, whether it came from a
  component or from the remote. A later success clears it.
- `lastReconcileAt` is set when a reconcile run completes. A run that stops early does not set it.
- `alertState` is `ok`, `failing` or `dead_items`; see [alerts](#alerts-and-metrics).

### Filter rules

The name filter is matched against the **subject** of the component, and the version filter against its
version. Both are regular expressions in Rust syntax that must match the whole text: `lib-.*` matches
`lib-core` and not `my-lib-core`, and `a|b` matches `b` and not `bc`.

The subject is built from the namespace and the name of the component, by these rules in this order:

1. If the namespace is empty, the subject is the name.
2. If the namespace is `/`, the subject is the name.
3. If the name starts with the namespace followed by `/`, the subject is the name. The namespace is not
   repeated.
4. Otherwise the subject is the namespace, a slash and the name.

| Format | Namespace | Name | Subject |
|---|---|---|---|
| `raw` | `/tools/1.4.0` | `/tools/1.4.0/tool-1.4.0.tar.gz` | `/tools/1.4.0/tool-1.4.0.tar.gz` |
| `raw` | `/` | `/top.txt` | `/top.txt` |
| `maven2` | `com.acme` | `lib` | `com.acme/lib` |
| `npm` (no transfer yet, so never replicated) | `acme` | `ui` | `acme/ui` |

Rules for the filters:

- At most 10 KiB of pattern text. A pattern that cannot compile, or that compiles to more than 1 MiB,
  is refused on save with `is too complex` or `is not a valid regular expression`.
- The version of a `raw` component is empty, so a version filter must match the empty text (`.*`);
  `.+` skips every raw component.
- The filters are checked when the item is sent, not when it is queued. Changing a filter applies to items
  sent after the change. Reconcile applies the same filters when it looks at the source.

### Deletes

With **Propagate deletes** off, a deleted component is not sent as a delete: the sender forgets that it
sent it, and the receiver keeps its copy. With it on, see [deletes and tombstones](#deletes-and-tombstones).
Deleting a replication forgets its queue and its record of sent components, so deletes of components it
sent earlier are no longer propagated.

The text of this subsection is for a push. For a pull, see [deletes in a pull](#deletes-in-a-pull).

## How it works

### Capture

Database triggers queue a component when it changes. Each enabled **push** replication of the source
repository gets one queue item per component, due after its settle time. A pull replication gets no items from
these triggers: its work comes only from reconcile (see [pull replication](#pull-replication)). These changes
are captured:

- a component is created or deleted;
- a file is added to a component, or the content of a file is replaced;
- a file is deleted from a component, or moved to another component (both components are queued);
- the attributes of a component change.

Writes that a replication makes on the receiver are not captured. Changes in a repository that has no
enabled replication are not captured either.

### Settle

A push component is sent after the settle time has passed since its last change, so a burst of uploads (a Maven
release with its checksums, for example) is one transfer. Every further change moves the time again. A
change while the component is being sent makes it run again after that transfer.

### One dispatcher per cluster

Every node runs the dispatcher, but one node at a time holds the PostgreSQL advisory lock `astrrepl` and
dispatches. The other nodes try for the lock every 10 seconds. The holder checks its database connection
every 15 seconds; when the check fails, it stops its workers within about 11 seconds, and another node takes
the lock and carries on. Every node serves the receiver routes, whether or not it dispatches.

The node that holds the lock also looks for remotes with due items every 30 seconds and evaluates the
alerts every 60 seconds.

### Remotes, workers and leases

- Each remote with due items gets its own workers: one worker per unit of **Max concurrency**. A slow or
  suspended remote does not hold up the workers of another remote.
- All replication database work shares a gate sized from the connection pool (the pool size minus 6, at
  least 1 and at most 32). A small pool slows every remote down.
- A worker leases its share of the due items, at most 20 at a time. A lease lasts 10 minutes. The worker
  renews the lease every 2 minutes during a transfer and while it uploads blobs. A lease that runs out, for
  example because the worker stopped, returns the item to the queue and counts as one attempt.
- A worker that finds nothing to do waits 5 seconds, then 10, 20 and 30 seconds (the longest wait), and
  goes back to 5 seconds when it gets work.
- A suspended remote is checked again when its suspension ends or after 30 seconds, whichever comes first.

### Per item

For each push item, in this order (a pull item has its own steps, see [pull replication](#pull-replication)):

1. The replication must exist, be enabled and target this remote. Otherwise the item is dropped without an
   error.
2. The component must match the filters. Otherwise the item is dropped.
3. The sender looks the component up in the source. When it is gone, the item is a delete: with
   **Propagate deletes** on, a tombstone is sent; otherwise the item is dropped. A delete for a component
   the sender never sent under this replication is dropped.
4. When the component's origin is the remote, the item is dropped (see [loops](#loops)). A component
   with no files is dropped.
5. Before the first send to a remote, the sender reads the receiver's capabilities (the handshake) and
   learns the receiver's instance ID.
6. The sender asks the receiver which blobs it lacks, in batches of 1,000 checksums, and uploads those
   first. Then it commits the component with its origin mark. If the commit reports missing blobs, the sender
   uploads again and commits once more.
7. The receiver must answer `copied` or `unchanged`. Any other answer makes the item dead. On success the
   sender sets **Last success** and, with **Propagate deletes** on, records the component's fingerprint and
   origin for later tombstones.

### Failures and retries

Each push transfer runs in phases: **handshake** (the receiver's capabilities), **negotiate** (the blob
check), **upload** (the blobs), **commit** (the component) and **retract** (the tombstone of a delete).
The phase decides whose failure it is. The rules below are for push. In a pull, a `403` in any phase suspends
the remote, and the phases differ; see [pull failures](#pull-failures).

**Remote failures** are failures of the receiver or of the way to it. They **suspend the remote**, and no
item is charged an attempt:

- in the handshake, any failure at all, including a `403` or a `404`;
- in the negotiate phase, no answer at all (connection refused, TLS error, timeout) and status `5xx`;
- in any phase, a status `401` (the token was refused), `408` or `429`, a refused address (the SSRF check
  or a redirect), and a stored token that cannot be opened with the key of this server.

**Item failures** are answers about the component. They count an attempt on the item, or, for the permanent
ones, make the item dead at once:

- **Dead at once:** `400`, `403` after the handshake, `404` (for example the remote repository does not
  exist), `409` (a conflict, see [the receiver](#prepare-the-receiver)), `413` (too large), `422`; a commit
  answer other than copied or unchanged; a source repository that no longer exists; a format that is not
  served or takes no part in replication.
- **Retried, dead after 12 attempts:** `5xx` outside the negotiate phase (a `Retry-After` header makes the
  wait at least that long); a commit that still lacks blobs after the second upload; a commit, upload or
  tombstone that gets no answer at all, which also suspends the remote; a local error while the component is
  prepared, such as a blob missing from its blob store or a component that changed while it was being sent.
  A lease that expired counts as an attempt too.

The wait before a retry of an item is 30 seconds, doubled after each attempt, up to one hour, with 20
percent random jitter. A suspension uses the same schedule, counted over the remote's consecutive failures.
Exceptions: a `401`, and a `403` in the handshake, suspend the remote for a fixed 15 minutes.

#### Suspension

A suspended remote is paused until **Suspended until**. While it is paused:

- nothing is leased from it, so its items keep their state and their attempts and simply wait;
- an item that was being sent when the pause began is put back without an attempt;
- the remote's `failures` count goes up by one for each suspension, and its last error is recorded.

A successful transfer clears the failure count, the suspension and the last error of the remote. An update of
the remote (see [updating a remote](#updating-a-remote)) clears them at once, so a token that was refused
starts working again as soon as the new token is saved.

The remote's fields in the REST answer:

- `failures`: suspensions since the last success or update.
- `suspendedUntil`: when the pause ends. The field keeps a past time after the pause; the remote is
  suspended only while the time is in the future. The replication status shows `suspendedUntil` only while it
  is in the future, and `null` otherwise.
- `lastError`: the reason of the most recent remote failure.

### Dead items

A dead item stays in the queue and is not retried by itself. A new change to the component queues it again
with a fresh count. In a push replication a reconcile run does not revive it. In a pull replication it does:
the next run that finds the component still missing or different queues it again (see [pull replication](#pull-replication)).

You can see dead items in four places:

- the replication's detail page, in the **Dead** tab (see [screens](#screens));
- `GET /service/rest/v1/replications/{id}/items?state=dead`;
- the MCP tool `get_replication_status`, which lists the first 20 of them;
- the server log, as the warning `a replication item is dead` with the replication ID, the component and
  the reason.

Each item shows its `lastError`. After you fix the cause (a token, a permission, a conflict on the receiver),
retry the items with the [retry route](#status-and-repair) or discard them. A retried item goes back to the
queue with its attempts reset and is sent at the next pass. A discarded item leaves the queue; if the
component still differs from the receiver, the next reconcile run queues it again.

### Deletes and tombstones

With **Propagate deletes** on, a component that the sender no longer has is removed on the receiver with a
tombstone. The tombstone carries the fingerprint and the origin that the sender recorded when it sent the
component. The receiver deletes the component only if its own copy has the same origin and the same
fingerprint.

- A component that the receiver no longer has counts as done.
- A component the sender never sent under this replication is not deleted, and the item is dropped.
- When the receiver's copy has changed, or its origin is not the sender, the tombstone is refused with
  `409` and the item is dead. The receiver keeps its copy.
- When the receiver protects the component under the rules of its format, for example a Docker tag under immutable
  tags, the tombstone is refused with `409` and the code `protected`, the item is dead, and the component stays.

The fingerprint covers the format, namespace, name and version, the path, SHA-256 and size of every file, and
the metadata, without the origin mark.

### Loops

The receiver marks every component it takes with its origin: the origin the sender declares, or the sender's
instance ID when none is declared. The mark is stored in the component's attributes as `replication` with
`origin` and `via`. Writes that replication makes on the receiver are not queued again. A sender skips a
component whose origin is the remote itself, both when it sends and when it reconciles.

Two servers that replicate the same repository to each other therefore do not send a component back to
where it came from. Server A sends a component with origin A. Server B stores it with that mark and does not
queue it. When B replicates to A, the component's origin is A's instance ID, so B drops it.

### Reconcile

Reconcile is the initial sync and the drift repair of a replication. Each replication has one reconcile
run, which is a system task (see [the task](#the-reconcile-task)). A run of a push replication does the following. A pull run pages the remote's inventory instead; see
[pull replication](#pull-replication).

1. pages the local components of the source repository, 500 at a time, and skips those outside the filters
   and those whose origin is the remote;
2. compares their fingerprints with the receiver in batches of 1,000 keys;
3. queues the components that differ, or that the receiver lacks, as ordinary items with the reason
   `reconcile`;
4. with **Propagate deletes** on, also pages the receiver's inventory and queues the keys that pass the
   filters, carry this instance's origin mark and are missing from the source, as deletes.

A run queues at most 100,000 items. When it reaches that limit it stops with the message `the limit of one
run was reached, run the reconcile again`. A run that finds the remote suspended, or the replication disabled,
is skipped with a message that says so. A run that fails because the remote does not answer ends the
suspension logic in the same way as a transfer does.

A run's message reports how many local components it looked at, how many were outside the filters, from the
remote or without assets, how many receiver entries it looked at, and how many items it queued.

**Reconcile every** sets how often the run happens. The allowed values are 5, 10, 15, 20, 30, 60, 120, 180,
240, 360, 480, 720 and 1440 minutes; the REST API refuses any other value with `must be one of 5, 10, ...`.
The minute of the run is offset per replication, derived from the replication's ID, so that the runs of
different replications do not start together:

| Period | Schedule |
|---|---|
| 5, 10, 15, 20 or 30 minutes | every period, starting at an offset within the first period of the hour |
| 60 to 720 minutes | every 1, 2, 3, 4, 6, 8 or 12 hours, at the replication's minute offset past the hour |
| 1440 minutes | once a day, at 00 hours and the replication's minute offset |

A replication stored with a period that is not in the list (possible only from older versions) runs at the
nearest allowed period below it, for example 45 minutes runs as 30. The screen offers the stored value as a
legacy choice and asks for one of the listed periods before it saves.

Runs also start by themselves:

- when the replication is created and is enabled;
- when a disabled replication is enabled;
- at server start, once for each enabled replication that has no reconcile task yet.

#### The reconcile task

Every replication has one system task, named `replication-reconcile-<replication name>` (with the replication
ID appended when another task already has that name). Its kind is **Replication - Reconcile a replication with
its remote**, and its property `replicationId` names the replication.

- The task is created with the replication, renamed when the replication is renamed, rescheduled when
  **Reconcile every** changes, and deleted with the replication.
- An operator can run it now, stop a run that is in progress, disable it (scheduled runs stop, the
  replication keeps working) and delete it.
- An operator cannot create such a task by hand, cannot give it another replication, and cannot find it among
  the task templates.
- A replication update does not create the task again after it was deleted. The task comes back at the next
  server start.

### Limits

| Limit | Value |
|---|---|
| Digests per missing-blob check | 1,000; the sender splits larger lists |
| Assets per bundle | 1 to 10,000 |
| Text fields in a bundle | 4,096 characters |
| Size of one blob | 64 GiB; a larger upload is refused with `413`. A pull refuses a bundle that names a larger blob, and the item is dead |
| Time for one blob download (pull) | At least 10 minutes, at most 6 hours, and otherwise the blob size at 256 KiB per second, so 1 GiB gets about 68 minutes. A longer download is retried |
| Staged blobs per repository on the receiver | 10,000; more are refused with `429` until some expire |
| Lifetime of a staged blob | 24 hours; expired staged blobs are removed by the task **Admin - Delete expired sessions and uploads** (see [tasks](../operate/tasks.md)) |
| Request body of the protocol routes | The server's default request body limit applies to JSON bodies: a commit whose bundle is larger than 2 MiB is refused with `413`, which makes the item dead |
| Items queued by one reconcile run | 100,000 |
| Workers per remote | 1 to 16 (**Max concurrency**) |
| Items leased by one worker at a time | At most 20 |

A staged blob is held against compaction for 48 hours, until its component is committed or the staging ends.

Staged bytes are not counted by the quota of the receiving repository until the component that uses them is
committed. The commit checks the quota again, under the usage lock of the repository, so a commit that would pass the
limit is refused and nothing is written.

## Pull replication

A pull replication copies components from a repository of a remote server into a hosted repository of the local
server. The local repository is the **target**; the repository of the remote is the **source**.

### What a pull is

- The local server opens every connection: the handshake, the inventory pages, the bundles and the blob downloads.
  The remote never connects back, and it sends nothing.
- No events cross between the servers. The remote does not know that a pull exists, so the local server learns of a
  change only at a reconcile run. The delay is the **Reconcile every** period, from 5 minutes to a day. A component
  that a run finds is fetched at once, without a settle time.
- A reconcile run does all the work. It compares the remote's inventory with the local repository and queues what
  differs; the filters apply to the remote's components.
- Remotes, workers, leases, status, retry and discard, alerts and metrics are the same as for a push. The workers of a
  remote serve its push and pull replications alike, within its **Max concurrency**.

### The direction field

- `direction` is `push` (the default) or `pull`. It is set when the replication is created. An update that changes it
  is refused: `cannot be changed; delete the replication and create it again with the other direction`.
- The unique key of a replication is `sourceRepository`, `remoteId` and `remoteRepository` together, for both
  directions. One local repository cannot both push to and pull from the same repository of a remote; the create
  answer says so.
- For a pull, the fields mean:
  - `sourceRepository` is the local repository, the target. It must be hosted and writable (a write policy of
    **Read-Only** is refused), and it must be online for the pull to run.
  - `remoteRepository` is the repository of the remote, the source.
  - `remoteId` is the remote to connect to.
- After creation, `remoteId` and `remoteRepository` of a pull cannot change: `cannot be changed for a pull replication;
  delete the replication and create a new one`.
- `nameFilter` and `versionFilter` apply to the remote's components, with the subject rules of
  [filter rules](#filter-rules).
- `propagateDeletes` applies to the local copies of pulled components; see [deletes in a pull](#deletes-in-a-pull).
- `settleSeconds` does not delay a pull, which has no settle time. The alert rule still reads it.
- `reconcileMinutes`, `enabled` and `alertChannels` work as they do for a push.

### Privileges on the remote

The token of a pull belongs to a service user on the remote. The user needs:

- The privilege `as-replication-pull` (`astrum:replication:pull`, described as "Let another server pull components
  from this one"). It opens the sender routes:
  - `GET /service/rest/v1/replication/{repo}/bundle?namespace=&name=&version=` answers the bundle of one component:
    its coordinates, the descriptors of its assets with sizes and digests, the attributes the format carries, and the
    origin mark. The origin is the component's `replication.origin`, or the remote's instance ID when it has none.
  - `GET /service/rest/v1/replication/{repo}/blobs/{sha256}` streams the bytes of one asset, found by its digest.
  - `GET /service/rest/v1/replication/{repo}/inventory?after=&limit=` lists the components of the repository in key
    order, each with its fingerprint and origin mark. The default page is 500 entries, the maximum 1,000. The privilege
    `as-replication-receive` also opens this route.
  - `GET /service/rest/v1/replication/capabilities` takes either privilege.
- `as-replication-receive` alone is not enough for the bundle and the blob routes: they answer `403`. The privilege
  `astrum:*` includes `astrum:replication:pull`.
- The read right on the repository, `as-repository-view-<format>-<repository>-read`, on the whole repository. The sender
  routes need a read on all of it: a content selector that grants only part of the repository gives `403`. A user who
  cannot read the repository at all gets `404`.

Create the role and the user on the remote the same way as a receiver (see [prepare the receiver](#prepare-the-receiver)),
with the privileges above. Sign in as that user and create an access token with the access level **Read and write**, as
for a receiver. The local server stores the token when the remote is added (step 3 of the worked example below).

### Creating a pull replication

`POST /service/rest/v1/replications` with `"direction": "pull"`. The creator needs `as-replication-create`, the
privilege `as-repository-admin-<format>-<repository>-read` and, for a pull, `as-repository-admin-<format>-<repository>-edit`,
both on the local repository. Creating a pull also queues a reconcile run at once, as a push does.

### What a pull does to the local repository

Reconcile queues the components, and each queued component then goes through these steps:

1. The replication must be enabled, and the component must pass the filters. Otherwise the item is dropped.
2. The local repository must be hosted, online, and its format must have a transfer.
3. The bundle is fetched from the remote. The answer must be the component that was asked for, in the local
   repository's format. A bundle without files is dropped.
4. When the bundle's origin is this server's instance ID, the item is dropped: the component came from here.
5. Each blob the local repository does not hold is downloaded by its digest, one at a time, into staging. The download
   must have the size and the SHA-256 that the bundle names. A mismatch is refused and nothing is left staged.
6. The component is committed with the same code the receiver uses. The write policy of the local repository decides a
   conflict: **Allow Redeploy** replaces a changed component, and **Disable Redeploy** refuses it, which makes the item
   dead. The Edit right is not checked for the server's own pulls.
7. The component is stored with an origin mark: `replication.origin` is the origin of the bundle, and `via` is
   `replication`. Each commit that copied content writes a `replicated` audit event.
8. With **Propagate deletes** on, the fingerprint and origin of what was pulled are recorded for later deletes.

Loops are prevented in two places. A pull's writes are not captured, so they are not queued for push replications of
the local repository. And a component whose origin is this server is never pulled back: the reconcile skips inventory
entries with this server's origin, and step 4 drops the rest.

A local change to a pulled component makes its fingerprint differ from the remote's. The next reconcile run therefore
pulls the remote's version again: under **Allow Redeploy** it replaces the local change, and under **Disable Redeploy**
the item is refused and dies.

### Deletes in a pull

With **Propagate deletes** on, a pull also removes its local copies when the remote drops them. A local component is
deleted only when all of these hold:

- its origin is the remote's instance ID. A component that reached the remote from a third server keeps that origin, and
  it is not deleted when the remote drops it;
- it passes the filters;
- the remote's inventory does not list it;
- its fingerprint and origin still match what was recorded when it was pulled. Otherwise the delete is refused and the
  item is dead.

Reconcile queues at most 1,000 deletes per run, and the next run continues. When the remote's inventory lists no
components at all, no delete is queued, and the run message says how many local components were held back. An empty or
wrongly pointed remote therefore cannot empty a mirror.

With **Propagate deletes** off, a component the remote drops stays local. Turning the option off, or changing either
filter, deletes the record of the pulled components, so later deletes of them are not propagated.

### Pull failures

Each pull request belongs to a phase: the **handshake**, the **inventory** of a reconcile run, the **bundle** and the
**blob** download. The rules differ from a push, and a `403` suspends the remote in any of them:

| Where | Answer from the remote | Result |
|---|---|---|
| Handshake, inventory, bundle | No answer, `5xx`, `401`, `403`, `408`, `429`, or a refused address | The remote is suspended. No item is charged an attempt |
| Any pull phase | `403`, for any reason, including a content selector on the sender's role | The remote is suspended |
| Bundle | `404` with the code `component_not_found` | The component is gone: with **Propagate deletes** it is deleted locally, otherwise the item is dropped |
| Bundle | `400`, another `404` (for example the remote repository does not exist), `409`, `413` or `422`, or a bundle that is not the component asked for, or of another format | The item is dead |
| Blob | No answer, `401`, `408` or `429` | The remote is suspended |
| Blob | `5xx`, a download longer than its time cap, a `404` for a blob the remote no longer holds, or a local error such as a full disk or a database error | The item is retried |
| Blob | A size or SHA-256 that differs from the bundle, or more bytes than the bundle names | The item is dead |
| Local | The local repository is read-only, or the commit is refused under **Disable Redeploy** | The item is dead |
| Local | The local repository is offline | The item is retried |

A retried item waits as it does for a push: 30 seconds, doubled after each attempt, up to one hour, and dead after 12
attempts. A dead pull item is revived by the next reconcile run that queues its component again, with its attempts
reset, and the pull then fetches it again.

### What the sender does not enforce

The remote serves what its hosted repositories hold. A bundle or a blob of a hosted repository is served to a pull
without the gates that apply to proxy repositories: cooldown, quarantine, and the vulnerability and licence gates apply
to proxy repositories only. The remote does not limit its pullers; the bandwidth limit that applies to a pull is the one
on the local server's remote record.

### Pull limits

| Limit | Value |
|---|---|
| Size of one blob | 64 GiB; a larger blob in a bundle makes the item dead |
| Time for one blob download | See the [limits](#limits) table: at least 10 minutes, at most 6 hours |
| Staged blobs per repository | 10,000, then retried with a delay until some expire; a staged blob lives 24 hours |
| Inventory entries per page | 500 by default, at most 1,000. A run reads at most 20,000 pages; a longer inventory fails the run |
| Items queued by one run | 100,000 |
| Deletes queued by one run | 1,000 |

The limits of a bundle (1 to 10,000 assets, 4,096 characters a text field) and the workers of a remote are the same as
for a push.

### Worked example: a customer site pulls from the vendor

The vendor instance is `https://astrum.example.org` and has a hosted raw repository `releases`. The customer instance is
`https://customer.example.net` and has a hosted raw repository `releases-mirror` with the write policy **Disable
Redeploy**: new files arrive, and a changed file is refused and shows as a dead item. `VENDOR_ADMIN_TOKEN` and
`CUSTOMER_ADMIN_TOKEN` are access tokens of the two `admin` users.

1. On the vendor, create the role and the user that the customer pulls with:

   ```shell
   curl -u "admin:$VENDOR_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://astrum.example.org/service/rest/v1/security/roles \
     -d '{"id": "customer-pull", "name": "Customer pull",
          "privileges": ["as-replication-pull",
                         "as-repository-view-raw-releases-read"],
          "roles": []}'

   curl -u "admin:$VENDOR_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://astrum.example.org/service/rest/v1/security/users \
     -d '{"userId": "customer-sync", "password": "<a long password>", "roles": ["customer-pull"]}'
   ```

2. Sign in to the vendor as `customer-sync` in the web interface, open **Account menu → Access Tokens**, and create a
   token with the access level **Read and write**. The token is shown once. Keep it as `PULL_TOKEN`.

3. On the customer, add the vendor as a remote. The answer holds the remote's `id`:

   ```shell
   curl -u "admin:$CUSTOMER_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://customer.example.net/service/rest/v1/replication/remotes \
     -d '{"name": "vendor", "url": "https://astrum.example.org", "token": "'"$PULL_TOKEN"'",
          "maxConcurrency": 2}'
   ```

4. Check the connection. Replace `<remote-id>` with the `id` from step 3. The answer is `"ok": true`, and `formats`
   includes `raw`:

   ```shell
   curl -u "admin:$CUSTOMER_ADMIN_TOKEN" -X POST \
     https://customer.example.net/service/rest/v1/replication/remotes/<remote-id>/check
   ```

5. Create the pull replication. The local repository is `releases-mirror`, and the name filter takes the tools of the
   vendor's raw repository (the subject of a raw component is its full path). The answer holds the replication's `id`:

   ```shell
   curl -u "admin:$CUSTOMER_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://customer.example.net/service/rest/v1/replications \
     -d '{"name": "vendor-releases", "direction": "pull",
          "sourceRepository": "releases-mirror", "remoteId": "<remote-id>",
          "remoteRepository": "releases", "nameFilter": "/tools/.*",
          "reconcileMinutes": 60}'
   ```

6. Watch the status. Replace `<replication-id>` with the `id` from step 5. After the first run `lastReconcileAt` is set,
   `lastSuccessAt` is set once a component has arrived, and `pending` falls back to 0 with `lastError` empty:

   ```shell
   curl -u "admin:$CUSTOMER_ADMIN_TOKEN" \
     https://customer.example.net/service/rest/v1/replications/<replication-id>/status
   ```

   A file that the mirror refuses shows as a dead item: list them with `GET .../items?state=dead`. Its `lastError`
   says `exist with different content`. The vendor logs `a bundle was served to a replication pull` for each component.

7. Download a file from the customer's mirror:

   ```shell
   curl -fO -u "admin:$CUSTOMER_ADMIN_TOKEN" \
     https://customer.example.net/repository/releases-mirror/tools/1.4.0/tool-1.4.0.tar.gz
   ```

## Status and repair

The status, the item list and the repair actions are REST routes under
`/service/rest/v1/replications/{id}`. The MCP tools in [MCP tools](#mcp-tools) call the same routes.

### Status

`GET /service/rest/v1/replications/{id}/status` needs `astrum:replication:read`. The answer:

```json
{
  "enabled": true,
  "pending": 3,
  "leased": 1,
  "dead": 0,
  "lagSeconds": 42,
  "lastSuccessAt": "...",
  "lastFailureAt": null,
  "lastError": null,
  "lastReconcileAt": "...",
  "alertState": "ok",
  "remote": {"suspendedUntil": null, "failures": 0, "lastError": null}
}
```

- `pending` counts the items waiting to be sent, `leased` those a worker holds, and `dead` those that gave up.
- `lagSeconds` is the age of the oldest item that is pending or leased; `null` when none waits.
- `remote` describes the remote, see [suspension](#suspension).

### Items

`GET /service/rest/v1/replications/{id}/items?state=dead&after=<id>&limit=<n>` lists the items in one state.

- `state` is required and is one of `dead`, `pending` or `leased`.
- `limit` is 1 to 200, default 50.
- `after` is the `id` of the last item of the previous page. Items come in ascending `id` order.

```json
{
  "items": [
    {"id": 17, "key": "com.acme/lib@1.0", "state": "dead", "reason": "event", "attempts": 1,
     "notBefore": "...", "firstQueuedAt": "...", "lastError": "the remote refused with status 409 (conflict): ..."}
  ],
  "next": null
}
```

`key` is the component as the receiver names it: `namespace/name@version`, with the parts percent-escaped
where needed. `reason` is `event`, `reconcile` or `retry`. `next` is the `id` to pass as `after` for the next
page, or `null` on the last page.

The items route needs `astrum:replication:read` and the right to view the source repository (the privilege
`as-repository-view-<format>-<repository>-read` or `…-browse`). The status route does not need the view right.

### Retry and discard

- `POST /service/rest/v1/replications/{id}/items/retry`, body `{"ids": [17, 18]}` or `{"all": true}`.
  Retry puts the dead items back in the queue with their attempts at 0, to be sent at the next pass. Answer:
  `{"retried": 2}`.
- `POST /service/rest/v1/replications/{id}/items/discard`, same body. Discard removes the dead items from
  the queue. Answer: `{"discarded": 2}`.

`ids` holds 1 to 1,000 ids. Give `ids` or `all`, not both; `all` must be `true` when it is given. Only dead
items are affected: an id of an item that is not dead is ignored.

These routes need `astrum:replication:update` and admin of the source repository (the privilege
`as-repository-admin-<format>-<repository>-read`).

### Reconcile now

`POST /service/rest/v1/replications/{id}/reconcile` queues a reconcile run now.

- `202 Accepted` with `{"run": <run id>}` when the run is queued.
- `202 Accepted` with `{"run": <run id>, "note": "the replication is disabled: the run will be skipped"}`
  when the replication is disabled. The run starts and then does nothing.
- `409 Conflict` with `{"id": "*", "message": "The reconcile of the replication \"<name>\" is already running or about to run"}`
  when a run is already active.
- `404` for an unknown replication.

The route needs `astrum:replication:update` and admin of the source repository.

### Audit

Changes to the items and reconcile requests are audited in the domain `replication`:
`replication.items.retried` and `replication.items.discarded` (with the replication ID, its name, whether
`all` was used, and the count) and `replication.reconcile.requested` (with the ID, the name and the run).

## Alerts and metrics

### Alert states

Each replication has an alert state, shown as **OK**, **Failing** or **Dead items**. The state is evaluated
once a minute by the node that holds the dispatcher lock, from these rules, checked in this order:

1. A disabled replication is `ok`, and its changes are silent.
2. A replication with at least one dead item is `dead_items`.
3. A replication is `failing` when its remote is suspended, or when both of these are true:
   - it has pending or leased items, and the oldest of them was first queued more than the settle time plus
     30 minutes ago;
   - it has had no success in the last 30 minutes, or never.
4. Otherwise it is `ok`.

The state is stored when it changes, whether or not a message is sent, so each change counts once.

### Messages

On each change of state, one message goes to each enabled notification channel listed in **Alert channels**.
Channels that are disabled or no longer exist are skipped. The messages are queued for delivery by the
notification channels (**Settings → System → Notifications**).

| Change | Message |
|---|---|
| to `failing` | **Replication is failing** |
| to `dead_items` | **Replication has dead items** |
| from `failing` or `dead_items` to `ok` | **Replication is healthy again**, with the replication and the remote only |

A failing or dead-items message gives the replication, the remote, the counts of waiting, in-progress and
dead items, whether the remote is paused, and the last error. The texts are in English and Russian; the
language follows the channel.

A replication that is disabled stays quiet: changing to or from a state while it is disabled sends nothing.

### Metrics

The metrics are in the Prometheus text format at `GET /service/metrics/prometheus`. They need the capability
**Metrics: Prometheus** to be on and a scraper with the privilege `as-metrics-all`; see
[Prometheus metrics](../operate/monitoring.md#prometheus-metrics). While the capability is off the path answers
404.

| Metric | Type | Labels | Meaning |
|---|---|---|---|
| `astrum_replication_pending` | gauge | `replication` | Components waiting to be sent (pending items, not the leased ones) |
| `astrum_replication_dead` | gauge | `replication` | Components the replication gave up on |
| `astrum_replication_lag_seconds` | gauge | `replication` | Age of the oldest pending or leased item; 0 when none waits |
| `astrum_replication_last_success_timestamp_seconds` | gauge | `replication` | When the replication last delivered a component; 0 when it never did |
| `astrum_replication_remote_suspended` | gauge | `remote` | 1 while the remote is suspended, 0 otherwise |
| `astrum_replication_transfers_total` | counter | `replication`, `result` | Transfer results on the dispatching node: `ok` (a component was delivered), `failed` (an attempt failed and the item stays queued, including items put back because the remote was paused), `dead` (the item became dead) |
| `astrum_replication_bytes_total` | counter | `replication` | Blob bytes sent to the remotes by the dispatching node, or, for a pull, blob bytes downloaded from the remote into staging. There is no direction label |

The label values are the replication name and the remote name. The two counters, `transfers_total` and
`bytes_total`, live in the memory of the node that dispatches. Each node counts only what it sent, and both
counters reset when the node restarts. Scrape every node if you need the totals, or use the gauges, which come
from the database and are the same on every node.

## Screens

- **Settings → Repository → Replication** opens the page **Replication** with the tabs **Remotes** and
  **Replications** (the page is `/admin/replication`).
- **Remotes** has the table **Replication remotes** with the columns **Name**, **URL**, **Health**, **Last
  checked** and **Remote version**, the buttons **Add remote**, **Edit remote**, **Check connection** and
  **Delete remote**, and the dialogs **New remote** and **Edit remote**.
  - **Health** shows **OK** in green, or **Suspended until** `<time>` in amber. Hovering shows the number of
    failures and the last error.
  - **Delete remote** asks for confirmation and says that a remote a replication uses cannot be deleted.
- **Replications** has the table **Replications** with the columns **Name**, **Source repository**,
  **Remote and repository**, **State**, **Pending**, **Dead**, **Lag**, **Last success** and **Filters**. Each
  row has an enable switch (**Enable replication** or **Disable replication**, with the name of the replication), **Edit** and
  **Delete**. The buttons **Add replication** and the dialogs **New replication** and **Edit replication**
  complete the page.
  - **State** shows **Disabled**, or the alert state: **OK**, **Failing** or **Dead items**.
  - **Filters** shows the name and version patterns, or **All components**.
  - Deleting asks for confirmation and says that components already sent stay on the remote.
- The replication dialog previews the filters on the components of the source repository in the browser:
  each component is marked **Would be sent** or **Would be skipped**. The server applies the same patterns
  when it sends, but the preview is not an answer from the server.
  - **Reconcile every, minutes** is a select with the 13 periods, named **Every 5 minutes** to **Every 30
    minutes**, **Every hour**, **Every 2 hours** to **Every 12 hours**, and **Daily**. The default is **Every 6
    hours**. A stored value that is not in the list is shown as "`<n>` minutes (not offered any more: pick one of
    the periods to save)", and the dialog does not save until one of the periods is chosen.
  - **Alert channels** is a multi-select of the notification channels.
- The detail page of a replication is `/admin/replication/replications/<id>`. It opens from the list. Its
  header shows the replication name and the remote with its repository. The button **Reconcile now** queues a
  reconcile run; it is disabled without the update right, and a refusal (for example a run already queued) is
  shown above the status. A started run shows the toast **Reconcile started**.
  - The status card shows **Pending**, **Leased**, **Dead**, **Lag**, **Last success**, **Last failure**,
    **Last error**, **Last reconcile**, **Remote failures** and **Remote last error**. While the remote is
    suspended it also shows **Suspended until** `<time>` **after** `<n>` **failures:** `<error>`.
  - The items table has the select **Show** with the states **Dead** (the default), **Pending** and **Leased**,
    and the columns **Key**, **Attempts**, **Last error** and **First queued**.
  - In the **Dead** state, with dead items present, each row has a checkbox and the buttons **Retry selected**
    (disabled until a row is selected), **Retry all dead**, **Discard selected** and **Discard all dead**. Each
    asks for confirmation. Retry says that the items go back to the queue with their attempts reset and are sent
    now. Discard says that discarded items leave the queue. After an action the toast says how many items were
    retried or discarded.
- The repository page shows the badge **Replicated to** `<n>` when enabled replications have this repository as
  their source. It needs the read right on replications and links to the replication page.
- The replication dialog has no direction choice. A pull replication is created and changed over REST; see
  [pull replication](#pull-replication).

## MCP tools

Agents can read the replications and repair dead items through four tools of the MCP server (see
[MCP](../use/mcp.md)). They call the REST routes above in the server, with the rights of the caller.

| Tool | What it does | Needs |
|---|---|---|
| `list_replications` | Every replication with its source repository, remote name, remote repository, enabled state, filters, propagate deletes, settle seconds, reconcile minutes and alert state. Tokens are never shown | `astrum:replication:read` |
| `get_replication_status` | The status of one replication (`id`). When it has dead items, the first 20 of them with `id`, `key`, `attempts` and `lastError` in `deadItems`; the ids can be passed to `retry_replication_items` | `astrum:replication:read`; the dead items also need the view right on the source repository |
| `retry_replication_items` | Requeues dead items: the ones given in `ids` (1 to 100 ids) or all of them with `all: true`. Answers with the count | `astrum:replication:update` and admin of the source repository |
| `run_replication_reconcile` | Queues a reconcile run now. When a run is already active, the answer says so with `started: false` and the message | `astrum:replication:update` and admin of the source repository |

No tool creates, edits or deletes a remote or a replication. The two repair tools are offered only to write
tokens. The tools that read use the reads of REST, so their answers are the same as the routes'.

## Troubleshooting

- **The replication does not send anything.** Read the status (`GET .../status`). Look at `remote.suspendedUntil`
  and `remote.lastError`, at the **Health** of the remote, and at `lastError` of the replication. A suspended
  remote is waiting; a failure that repeats shows the cause.
- **The remote is suspended.** Fix the cause the last error names: the receiver is down, the address is refused,
  the token was refused (see below). Then update the remote (`PUT`, with the token if it changed); the
  suspension ends at once and the sender resumes within about 30 seconds.
- **Dead items.** List them with `GET .../items?state=dead` and read each `lastError`. A `409` is a conflict on
  the receiver (a changed version under **Disable Redeploy**, or **Allow Redeploy** without `…-edit`); a `403`
  or a `404` is about rights or the repository. Fix the cause, then retry the items. If the component is no
  longer wanted, discard them.
- **The filter matches nothing.** Check the subject rule: raw components are matched by their full path,
  maven components by `group/artifact`, and a version filter on raw must match the empty text. The preview in
  the dialog shows the same subjects. A pattern matches the whole subject, not a part of it.
- **A component comes back to where it came from.** Check the origin mark (`replication` in the component's
  attributes) and make sure each remote URL points at the other instance and no other. A remote whose URL was
  changed records its instance ID again at the next check.
- **The token is refused.** The sender pauses the remote for 15 minutes after a `401`, or a `403` in the
  handshake. The token must belong to a service user that holds `as-replication-receive` (for a push) or
  `as-replication-pull` (for a pull) and the repository rights. Update the remote with the new token; the pause ends at once.

## What replication does not do

- **Chain components by events.** A component that B received from A is not queued on B, so it does not reach
  C by events. It reaches C only when B's reconcile run finds it missing there.
- **Formats without a transfer.** A component of a format with no transfer, such as protobuf, fails permanently.
  Docker and npm repositories cannot be replicated at all (see the list of what is not replicated above).
- **Delete by default.** Deletes are sent only with **Propagate deletes**.
- **Replicate attribute-only changes in this version.** A change that touches only the attributes of a component
  (a pypi yank, for example) queues an item, but the bundle of `pypi`, `maven2`, `helm` and `raw` does not carry
  component attributes, so the receiver gets the same content and shows no change. Such a change is not replicated.
- **Events between servers.** A push receiver never reads from the sender. A pull replication has no event
  feed from the remote either: it learns about changes only at its reconcile run, see [pull replication](#pull-replication).
- **Copy settings, policies or accounts.** Repository settings, cleanup policies, routing rules, write
  policies, users, roles and tokens stay on their server.
- **Revive dead items in a push.** A reconcile run leaves dead items of a push replication alone; a change to
  the component or a retry does that. A pull replication revives them at its reconcile run.

## Security notes

- **The token** is sealed with the [secrets key](runtime.md#secrets-key) before it is stored. No REST answer
  and no audit event contains it; a remote shows only `hasToken`. If the key of the sender changes, the stored
  token cannot be opened, and the check asks for the token again.
- **The receiver trusts the origin the sender declares.** It takes the origin from the bundle, or from the
  `X-Astrum-Instance` header when the bundle has none. A tombstone needs an origin and a fingerprint that
  match its copy, but every holder of `astrum:replication:receive` can name any origin.
- **A pull trusts the origin the remote declares.** The local server takes the origin from the remote's bundle,
  or the remote's instance ID when there is none. A remote that declares the origin of this server is skipped,
  and deletes of pulled components apply only to the remote's own origin; see [pull replication](#pull-replication).
- **Network exposure.** The receiver routes are `/service/rest/v1/replication/...` and are served on the same
  listener as the rest of the API. Nothing on this server limits them to the sender's address. Restrict them
  in the reverse proxy or the firewall when the receiver is reachable from outside; see
  [reverse proxy and TLS](reverse-proxy.md).
- **Audit.** Changes to remotes and replications, checks of remotes, retries and discards of items and reconcile
  requests are audited in the domain `replication`: `remote.created`, `remote.updated`, `remote.deleted`,
  `remote.checked`, `replication.created`, `replication.updated`, `replication.deleted`,
  `replication.items.retried`, `replication.items.discarded` and `replication.reconcile.requested`. The receiver
  audits each commit in the domain `repository.content` as `replicated` and each tombstone as `deleted`; both
  carry `"via": "replication"` and the origin. Sends from the sender are not audited. A pull writes the same
  two events into its local repository, and the remote logs each bundle and blob it serves to a pull at the info level.

## Worked example

Two servers: the sender `https://astrum.example.org` and the receiver `https://replica.example.org`. Both
have a hosted raw repository: `releases` on the sender and `releases-replica` on the receiver, with the
write policy **Disable Redeploy** on the receiver. `ASTRUM_TOKEN` holds an access token of the sender's
`admin` user, `REPLICA_ADMIN_TOKEN` one of the receiver's `admin` user, and `REPLICA_TOKEN` the token of
`replica-sender` on the receiver.

1. On the receiver, create the role and the user. Repository privileges exist as soon as the repository exists:

   ```shell
   curl -u "admin:$REPLICA_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://replica.example.org/service/rest/v1/security/roles \
     -d '{"id": "replica-receiver", "name": "Replica receiver",
          "privileges": ["as-replication-receive",
                         "as-repository-view-raw-releases-replica-read",
                         "as-repository-view-raw-releases-replica-add",
                         "as-repository-view-raw-releases-replica-delete"],
          "roles": []}'

   curl -u "admin:$REPLICA_ADMIN_TOKEN" -H 'Content-Type: application/json' \
     https://replica.example.org/service/rest/v1/security/users \
     -d '{"userId": "replica-sender", "password": "<a long password>", "roles": ["replica-receiver"]}'
   ```

2. Sign in to the receiver as `replica-sender` in the web interface, open **Account menu → Access Tokens**,
   and create a token with the access level **Read and write**. The token is shown once; keep it, because
   the sender's remote needs it in step 3 (`REPLICA_TOKEN` below).

3. On the sender, add the remote. The answer holds the remote's `id`:

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" -H 'Content-Type: application/json' \
     https://astrum.example.org/service/rest/v1/replication/remotes \
     -d '{"name": "replica", "url": "https://replica.example.org", "token": "'"$REPLICA_TOKEN"'",
          "maxConcurrency": 2}'
   ```

4. Check the connection. Replace `<remote-id>` with the `id` from step 3:

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" -X POST \
     https://astrum.example.org/service/rest/v1/replication/remotes/<remote-id>/check
   ```

   A working remote answers `"ok": true` with the receiver's `formats`, which include `raw`.

5. Add the replication with a name filter. The filter matches the subject of raw components, which is the full
   path, see [filter rules](#filter-rules). Creating the replication queues a reconcile run at once, so the
   components already in `releases` that match the filter are sent too:

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" -H 'Content-Type: application/json' \
     https://astrum.example.org/service/rest/v1/replications \
     -d '{"name": "releases-to-replica", "sourceRepository": "releases",
          "remoteId": "<remote-id>", "remoteRepository": "releases-replica",
          "nameFilter": ".*tool-.*", "settleSeconds": 30, "propagateDeletes": false,
          "reconcileMinutes": 360}'
   ```

6. Push a component to the sender. The upload queues it, and it is sent after the settle time:

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" --upload-file build/tool-1.4.0.tar.gz \
     https://astrum.example.org/repository/releases/tool/1.4.0/tool-1.4.0.tar.gz
   ```

7. Wait for the settle time and a few seconds more, then check the replication on the sender. `lastSuccessAt`
   is set and `lastError` is empty:

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" https://astrum.example.org/service/rest/v1/replications/<replication-id>
   ```

8. Download the file from the receiver with the receiver's service user:

   ```shell
   curl -fO -u "replica-sender:$REPLICA_TOKEN" \
     https://replica.example.org/repository/releases-replica/tool/1.4.0/tool-1.4.0.tar.gz
   ```

   If the file is not there, read the replication's status (`GET .../status`) and its dead items
   (`GET .../items?state=dead`), and look at the sender's log under **Settings → Support → Logs** for
   `a replication item is dead` or a retry.
