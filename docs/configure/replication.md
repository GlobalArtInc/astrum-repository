# Replication

Replication sends hosted components from one Astrum server, the **sender**, to a hosted repository of
another Astrum server, the **receiver**. The sender pushes; nothing on the receiver fetches from the
sender. A **remote** is the receiving server. A **replication** copies one hosted repository of the
sender to one repository of a remote. The sender and the receiver are configured separately: the receiver
needs a service user and a token, the sender needs a remote and a replication.

Settings are under **Settings → Repository → Replication**, with the tabs **Remotes** and
**Replications**. The same settings are available over REST under `/service/rest/v1/replication/remotes`
and `/service/rest/v1/replications`.

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
format as the source (`maven2` to `maven2`, and so on).

What is not replicated:

- Proxy and group repositories. The server refuses them as a source.
- Components of other hosted formats. The server accepts the replication, but every component fails
  permanently with the reason `the format <id> takes no part in replication`; see [dead items](#dead-items).
- Repository settings, cleanup policies, routing rules and write policies, and users, roles and tokens.
- Components stored before the replication was created or enabled. There is no initial copy: a component
  is sent when it changes again. Changes made while the replication is disabled are not queued.
- Deletes, unless **Propagate deletes** is on.

## Prepare the receiver

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
| **URL**, `url` | The address of the receiver, for example `https://astrum.example.com:8443`. See the rules below |
| **Token**, `token` | Required when a remote is created. 1 to 1024 characters, no spaces and no non-ASCII characters. Write-only |
| **Use the trust store**, `useTrustStore` | Off by default. On: connections trust the certificates of the [truststore](ssl-certificates.md#connections-that-use-the-truststore). If the truststore cannot be read, the system roots apply and a warning is logged |
| **Allow private network addresses**, `allowPrivateNetwork` | Off by default. See the private network rules below |
| **Max concurrency**, `maxConcurrency` | Components sent at the same time, from 1 to 16, default 2 |
| **Bandwidth limit, MiB/s** (UI), `maxBytesPerSecond` (REST) | Limit of the blob uploads to this remote, in bytes per second; empty or `null` means no limit. Only blob uploads are throttled |

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
  allow. When none is allowed, the connection is refused, and the sender retries the item like a temporary
  failure.
- **Through a proxy.** When the HTTP settings have a proxy for the remote's scheme (see
  [outbound HTTP and proxy](outbound-http.md#proxy-server)), the proxy connects to the receiver. Astrum still
  resolves the receiver's host name itself before each request, and refuses the request when any resolved
  address is blocked. A name Astrum cannot resolve is a temporary failure; the request is not sent to the
  proxy. The proxy host itself is allowed even when it has a private address.
- **Redirects** are not followed. A redirect answer is an error.
- The connection timeouts and retries of the HTTP settings do not apply. Replication uses its own: a 10
  second connection timeout, a 60 second limit on control requests, and its own retry schedule.

### Updating a remote

- Changing the URL requires the token again in the same request: the old token is not sent to a new address.
  Without a new token the change is refused with `token: must be given again when the url changes`.
- A change of the URL clears the recorded instance ID and version of the remote; the next check or send
  records them again.
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
repository, the privilege `as-repository-admin-<format>-<repository>-read`. Changing and deleting need the
update or delete privilege and the same repository privilege for the current source.

| Field (UI label, REST name) | Rules |
|---|---|
| **Name**, `name` | 1 to 100 characters, no control characters. Unique ignoring case, across replications |
| **Source repository**, `sourceRepository` | An existing hosted repository. Proxy and group repositories are refused. Deleting the repository deletes its replications; renaming it renames the source |
| **Remote**, `remoteId` | The ID of an existing remote |
| **Remote repository**, `remoteRepository` | Letters, digits, `-`, `_` and `.`, starting with a letter, a digit or `-`; at most 200 characters. `remotes` and `capabilities` are refused, because the receiver uses them for its protocol routes. Not checked against the receiver on save: when the receiver does not know it, every item fails permanently with `404` |
| **Enabled**, `enabled` | Default on. A disabled replication queues nothing, and its queued items are dropped when they fall due |
| **Name filter**, `nameFilter` | Empty means every component. See the filter rules below |
| **Version filter**, `versionFilter` | Empty means every version. See the filter rules below |
| **Propagate deletes**, `propagateDeletes` | Default off. See [deletes](#deletes) |
| **Settle time, seconds**, `settleSeconds` | 0 to 3600, default 30. See [how it works](#how-it-works) |
| **Alert channels**, `alertChannels` | A list of notification channel IDs, each of which must exist. The field is stored and checked, but nothing on this server sends to these channels yet |

The read-only fields `lastSuccessAt`, `lastFailureAt` and `lastError` are set by the sender. `lastError`
holds the reason of the most recent failure of any component of the replication and is not cleared by a
later success.

### Filter rules

The name filter is matched against `<namespace>/<name>`, or against `<name>` when the namespace is empty.
The version filter is matched against the version. Both are regular expressions in Rust syntax that must
match the whole text: `lib-.*` matches `lib-core` and not `my-lib-core`, and `a|b` matches `b` and not `bc`.

- At most 10 KiB of pattern text. A pattern that cannot compile, or that compiles to more than 1 MiB,
  is refused on save with `is too complex` or `is not a valid regular expression`.
- For `raw` the namespace is the directory and the name is the full path, so the text matched is the
  directory, a slash and the path again: `/a/b//a/b/hello.txt`. A pattern such as `.*hello.*` matches it.
  The version of a `raw` component is empty, so a version filter must match the empty text (`.*`);
  `.+` skips every raw component.
- The filters are checked when the item is sent, not when it is queued. Changing a filter applies to items
  sent after the change.

### Deletes

With **Propagate deletes** off, a deleted component is not sent as a delete: the sender forgets that it
sent it, and the receiver keeps its copy. With it on, see [how it works](#how-it-works) for the tombstone
rules. Deleting a replication forgets its queue and its record of sent components, so deletes of
components it sent earlier are no longer propagated.

### Screens

- **Settings → Repository → Replication** (the tile **Replication**) opens the page **Replication**.
- **Remotes** has the table **Replication remotes** with the columns **Name**, **URL**, **Last checked** and
  **Remote version**, the buttons **Add remote**, **Edit remote**, **Check connection** and **Delete remote**,
  and the dialogs **New remote** and **Edit remote**.
- **Replications** has the table **Replications** with the columns **Name**, **Source repository**,
  **Remote and repository**, **State** and **Filters**, the buttons **Add replication**, **Enable**,
  **Disable**, **Edit** and **Delete**, and the dialogs **New replication** and **Edit replication**.
- The replication dialog previews the filters on the components of the source repository in the browser:
  each component is marked **Would be sent** or **Would be skipped**. The server applies the same patterns
  when it sends, but the preview is not an answer from the server.

## How it works

1. **Capture.** Database triggers queue a component when it is created or deleted, and when a file is stored
   in it or a file's content is replaced. Each enabled replication of the source repository gets one queue
   item per component, due after its settle time. Only these changes are captured; a change that only
   touches metadata is not.
2. **Settle.** A component is sent after the settle time has passed since its last change, so a burst of
   uploads (a Maven release with its checksums, for example) is one transfer. Every further change moves the
   time again. A change while the component is being sent makes it run again after that transfer.
3. **One dispatcher per cluster.** Every node runs the dispatcher, but one node at a time holds the
   PostgreSQL advisory lock `astrrepl`. The other nodes try for the lock every 10 seconds. The holder checks
   its database connection every 15 seconds; when that fails it stops its workers, and another node takes
   over. Queue items are leased for 10 minutes and the lease is renewed while blobs are uploaded. An item whose
   lease runs out goes back to the queue.
4. **Passes.** The dispatcher serves each remote that has due items with up to its max concurrency workers,
   each taking up to 20 items per pass. When nothing is due it waits 5 seconds.
5. **Per item.** The sender checks that the replication is enabled, that the component matches the filters
   and that its origin is not the receiver (see [loops](#loops)). It builds the component's bundle, which
   lists its files with their checksums. It asks the receiver which of the blobs it lacks, uploads those, and
   then commits the component with its origin mark. If the receiver still lacks a blob at commit, the sender
   uploads it and commits again; a second shortage is a temporary failure. On success the sender records the
   component's fingerprint and origin, and sets **Last success**.
6. **Retries and backoff.** A temporary failure waits 30 seconds, doubled after each failure, up to one hour,
   with 20 percent random jitter. A `Retry-After` header from the receiver makes the wait at least that long
   (between 1 second and one hour). A component that fails 12 times in a row is dead. Temporary failures are:
   connection and timeout errors, no progress for 120 seconds during an upload, status `408`, `429` and `5xx`,
   a refusal by the SSRF check or a redirect, and local errors such as a database error.

### Dead items

A **permanent** failure makes the item dead at once. Permanent failures are every other status from the
receiver, such as `400`, `401`, `403`, `404`, `409` and `413`, and local rejections such as an unsupported
format, a source repository that no longer exists, or a receiver answer that is neither copied nor unchanged.
Some examples:

- an expired or revoked token (`401`) makes every item for that remote dead on its first attempt;
- a remote repository that does not exist (`404`) makes every item of the replication dead;
- a changed version under **Disable Redeploy** or without `…-edit` (`409`) makes that component dead;
- a bundle that the receiver refuses as too large (`413`) makes that component dead.

Dead items stay in the queue, and nothing retries them. A later change to the component queues it again
with a fresh count. There is no screen and no REST call that lists dead items on this server. The server
logs a warning `a replication item is dead` with the replication ID, the component and the reason. The
replication's `lastError` shows the latest failure. After you fix the cause, change the component, or delete
and recreate the replication to start its queue again.

### Deletes and tombstones

With **Propagate deletes** on, a component that the sender no longer has is removed on the receiver with a
tombstone. The tombstone carries the fingerprint and the origin that the sender recorded when it sent the
component. The receiver deletes the component only if its own copy has the same origin and the same
fingerprint.

- A component that the receiver no longer has counts as done.
- A component the sender never sent under this replication is not deleted, and the item is dropped.
- When the receiver's copy has changed, or its origin is not the sender, the tombstone is refused with
  `409` and the item is dead. The receiver keeps its copy.

The fingerprint covers the format, namespace, name and version, the path, SHA-256 and size of every file, and
the metadata, without the origin mark.

### Loops

The receiver marks every component it takes with its origin: the origin the sender declares, or the sender's
instance ID when none is declared. The mark is stored in the component's attributes as `replication` with
`origin` and `via`. Writes that replication makes on the receiver are not queued again. A sender skips a
component whose origin is the remote itself. Two servers that replicate the same repository to each other
therefore do not send a component back to where it came from.

### Limits

| Limit | Value |
|---|---|
| Digests per missing-blob check | 1,000; the sender splits larger lists |
| Assets per bundle | 1 to 10,000 |
| Text fields in a bundle | 4,096 characters |
| Size of one blob | 64 GiB; a larger upload is refused with `413` |
| Staged blobs per repository on the receiver | 10,000; more are refused with `429` until some expire |
| Lifetime of a staged blob | 24 hours; expired staged blobs are removed by the task **Admin - Delete expired sessions and uploads** (see [tasks](../operate/tasks.md)) |
| Request body of the protocol routes | The server's default request body limit applies to JSON bodies: a commit whose bundle is larger than 2 MiB is refused with `413`, which makes the item dead |

A staged blob is held against compaction for 48 hours, until its component is committed or the staging ends.

## Security notes

- **The token** is sealed with the [secrets key](runtime.md#secrets-key) before it is stored. No REST answer
  and no audit event contains it; a remote shows only `hasToken`. If the key of the sender changes, the stored
  token cannot be opened, and the check asks for the token again.
- **The receiver trusts the origin the sender declares.** It takes the origin from the bundle, or from the
  `X-Astrum-Instance` header when the bundle has none. A tombstone needs an origin and a fingerprint that
  match its copy, but every holder of `astrum:replication:receive` can name any origin.
- **Network exposure.** The receiver routes are `/service/rest/v1/replication/...` and are served on the same
  listener as the rest of the API. Nothing on this server limits them to the sender's address. Restrict them
  in the reverse proxy or the firewall when the receiver is reachable from outside; see
  [reverse proxy and TLS](reverse-proxy.md).
- **Audit.** Changes to remotes and replications, and checks of remotes, are audited in the domain
  `replication`: `remote.created`, `remote.updated`, `remote.deleted`, `remote.checked`,
  `replication.created`, `replication.updated` and `replication.deleted`. The receiver audits each commit
  in the domain `repository.content` as `replicated` and each tombstone as `deleted`; both carry
  `"via": "replication"` and the origin. Sends from the sender are not audited.

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

5. Add the replication with a name filter. The filter matches the full path text of raw components, see
   [filter rules](#filter-rules):

   ```shell
   curl -u "admin:$ASTRUM_TOKEN" -H 'Content-Type: application/json' \
     https://astrum.example.org/service/rest/v1/replications \
     -d '{"name": "releases-to-replica", "sourceRepository": "releases",
          "remoteId": "<remote-id>", "remoteRepository": "releases-replica",
          "nameFilter": ".*tool-.*", "settleSeconds": 30, "propagateDeletes": false}'
   ```

6. Push a component to the sender. Only the upload starts the replication; components stored earlier are not
   sent:

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

   If the file is not there, look at the sender's log under **Settings → Support → Logs** for
   `a replication item is dead` or a retry, and at `lastError` of the replication.
