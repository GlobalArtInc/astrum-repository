# Blob stores

A blob store is where the files of components are kept. Every repository writes to exactly one blob
store, chosen when the repository is created. The database only holds the metadata, so a blob store and
the database belong together: see [backup and restore](../operate/backup-restore.md).

There are file, S3, remote and group blob stores. On a new installation the server creates one file blob
store called `default` in `<data>/blobs/default`. Manage the stores under **Settings → Repository →
Blob Stores**, or with the REST API under `/service/rest/v1/blobstores`. A store that repositories use
cannot be deleted.

## File blob stores

A file blob store keeps each blob as a file below its **Path**: an absolute path, or a path relative to
`<data>/blobs`. Blobs are written to a temporary name, flushed to disk and renamed, so a crash leaves no
half-written blob under its final name. They are spread over two levels of directories
(`content/<xx>/<yy>/<id>`), which keeps directories small.

- Put the path on a disk with room to grow and with the file system's inode count in mind; see
  [system requirements](../install/requirements.md#disk).
- The paths of two stores must not overlap.
- A store that holds blobs cannot change its path, and changing it would not move the files.
- In the container image mount a volume for every path outside `/astrum-data`.
- Local disks are the safe choice. A network file system works if it supports atomic rename and `fsync`;
  servers that share blobs need a shared file system or S3.

## S3 blob stores

An S3 blob store keeps blobs in a bucket of Amazon S3 or of a service that speaks the S3 API (MinIO,
Ceph, Cloudflare R2 and others).

| Setting | Meaning |
|---|---|
| Bucket name, **Region** | The bucket, created on save when it does not exist. `DEFAULT` takes the region from the environment |
| **Prefix** | A folder in the bucket for this store; empty uses the whole bucket |
| **Endpoint URL** | Address of an S3-compatible service; empty uses AWS |
| **Use path-style addresses** | Put the bucket into the path instead of the host name; most S3-compatible services need it |
| **Access key ID**, **Secret access key** | Credentials; leave empty to use the credentials the server finds in its environment, profile or instance role |
| **Role to assume**, **Session token** | An IAM role the server assumes with these credentials; the token is for temporary credentials |
| **Encryption** | Server-side encryption: none, S3-managed (SSE-S3) or AWS KMS (SSE-KMS, with an optional key ID) |

The secret key and the session token are stored sealed, see the [secrets key](runtime.md#secrets-key).
Large blobs are uploaded in parts of 8 MiB. A bucket, prefix or region cannot change once the store
holds blobs.

An upload that is interrupted can leave unfinished parts in the bucket, and S3 charges for them.
Add a lifecycle rule to the bucket that aborts incomplete multipart uploads after a few days:

```json
{
  "Rules": [
    {
      "ID": "abort-incomplete-uploads",
      "Status": "Enabled",
      "Filter": {},
      "AbortIncompleteMultipartUpload": { "DaysAfterInitiation": 7 }
    }
  ]
}
```

The setting for days before deleted blobs expire, failover buckets and pre-signed URLs are stored with
the configuration, but the server does not act on them yet; use bucket lifecycle rules for expiry.

The outgoing connection to the endpoint is subject to the SSRF protection: an endpoint on an internal
address has to be allowed first, see [runtime configuration](runtime.md#outbound-http-proxy).

## Remote blob stores

A remote blob store keeps its blobs on the disk of another server. That server runs the blob agent,
`astrum-blob-agent`, which ships in the same container image as the server. The agent keeps the blobs
in one directory, in the same layout as a file blob store, and answers the server over HTTP or HTTPS.
The server keeps the metadata and the totals of the store in its database, as for every other store.

| Setting | Meaning |
|---|---|
| **Agent address** | `http://` or `https://` address of the agent, such as `https://storage1.internal:8090`, without a path |
| **Agent token** | The token the agent was started with, at least 32 characters; stored sealed, see the [secrets key](runtime.md#secrets-key) |
| **Trusted certificate** | PEM certificate of the agent or of its CA, when the agent uses a certificate the public authorities did not sign |

The server checks that the agent answers, with the token, before it saves the store. A stored token is
kept when the address stays the same and the field is left empty; a new address needs the token again.
Changing the address does not move blobs: point it at the same agent under a new address, or at a copy
of its directory. The connection is subject to the SSRF protection like an S3 endpoint, so an agent on
an internal address has to be allowed first, see [runtime configuration](runtime.md#outbound-http-proxy).

A remote store is meant to be a member of a group (see below): that is how a store that repositories
already use gets the space of another server.

### Running the agent

Create a token and a directory owned by uid 200, the user of the image, then start the agent from the
image of the server:

```sh
openssl rand -hex 32 > /etc/astrum-agent/token
chmod 600 /etc/astrum-agent/token
mkdir -p /srv/astrum-blobs && chown 200:200 /srv/astrum-blobs
podman run -d --name astrum-blob-agent --restart=always \
  --entrypoint astrum-blob-agent \
  -v /srv/astrum-blobs:/astrum-agent \
  -v /etc/astrum-agent/token:/run/secrets/astrum-agent-token:ro \
  -e ASTRUM_AGENT_TOKEN_FILE=/run/secrets/astrum-agent-token \
  -p 8090:8090 \
  globalartltd/astrum-repository:<tag>
```

Use the same tag as the server. Outside a container the binary takes the same settings as flags:

| Flag | Variable | Default | Meaning |
|---|---|---|---|
| `--dir` | `ASTRUM_AGENT_DIR` | `/astrum-agent` | Directory that holds the blobs |
| `--listen` | `ASTRUM_AGENT_LISTEN` | `0.0.0.0:8090` | Address to listen on |
| `--token-file` | `ASTRUM_AGENT_TOKEN_FILE` | | File with the token; `ASTRUM_AGENT_TOKEN` takes the token itself |
| `--tls-certificate`, `--tls-key` | `ASTRUM_AGENT_TLS_CERTIFICATE`, `ASTRUM_AGENT_TLS_KEY` | | PEM certificate chain and key; with both the agent speaks HTTPS |
| | `ASTRUM_AGENT_LOG` | `info` | Log filter |

Every request needs the token as a bearer token. Let only the server reach the port, and use HTTPS
whenever the traffic leaves a private network. The agent writes each blob to a temporary name, flushes
it and renames it, and the server compares the size and SHA-256 the agent reports with what it sent, so
a broken upload leaves nothing behind.

## Group blob stores

A group presents several stores as one. Reads look in every member; a new blob goes to the member chosen
by the **Fill policy**:

| Fill policy | New blobs go to |
|---|---|
| `writeToFirst` | The first member |
| `roundRobin` | Each member in turn |
| `mostFreeSpace` | The member with the most free space; members that do not answer are left out, and a member with no limit of its own, such as S3, counts as the roomiest |

A group is a way to add disks and servers without moving files. Members must be existing file, S3 or
remote stores that are not in another group and not used by repositories directly; a member that holds
blobs cannot be removed from the group. A read that does not find a blob in a member that answers goes
on to the next member, so a member that is down only makes its own blobs unavailable.

### Growing a store that repositories use

The blob store of a repository cannot change, and a store that repositories use cannot join a group.
Convert such a store into a group instead: under **Blob Stores**, open the store and choose
**Convert to Group**, or call

```
POST /service/rest/v1/blobstores/group/convert/<name>/<new name of the store>
```

The store keeps its settings and its blobs under the new name and becomes the only member of a group
that takes over its name, so the repositories keep writing to `<name>`. The soft quota stays with the
group. Then add stores to the group, for example a remote store on another server, and choose the fill
policy `mostFreeSpace` or put the new store first with `writeToFirst`. A store that is a member of a
group, or a group, cannot be converted.

## Soft quotas

A store can have a **soft quota**: either a limit on the space it uses, or a minimum of free space.
Passing the quota does not stop writes; the check **Blob Stores Quota** under
**Settings → Support → Metrics and Health** turns red, which is what you alert on. The capability
**Blob Stores: Space Alert** sends that, a low free space and an unreachable store to notification
channels; see [monitoring](../operate/monitoring.md#blob-store-space-alerts).

## Reclaiming space

Deleting a component or an asset, by hand or through a cleanup policy, deletes its blob. These tasks
under **Settings → System → Tasks** take care of the rest:

| Task | What it does |
|---|---|
| Admin - Cleanup repositories using their associated policies | Runs the cleanup policies assigned to repositories; daily by default |
| Docker - Delete unused manifests and images | Deletes untagged manifests and unused layers of hosted Docker repositories that are older than the deploy offset (24 hours by default); daily by default |
| Admin - Compact blob store | Deletes blobs that no asset refers to, such as the leftovers of interrupted uploads, if they are older than the grace period (one day by default); daily by default |
| Repair - Recalculate blob store storage | Counts the blobs and bytes of a store again, if the totals shown drifted |

The totals of a store (blob count and bytes) are kept in the database and updated as blobs come and go.
