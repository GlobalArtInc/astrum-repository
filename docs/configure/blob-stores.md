# Blob stores

A blob store is where the files of components are kept. Every repository writes to exactly one blob
store, chosen when the repository is created. The database only holds the metadata, so a blob store and
the database belong together: see [backup and restore](../operate/backup-restore.md).

On a new installation the server creates one file blob store called `default` in
`<data>/blobs/default`. Manage the stores under **Settings → Repository → Blob Stores**, or with the
REST API under `/service/rest/v1/blobstores`. A store that repositories use cannot be deleted.

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

## Group blob stores

A group presents several stores as one. Reads look in every member; a new blob goes to the member chosen
by the **Fill policy**: the first member that can take it (`writeToFirst`) or each member in turn
(`roundRobin`). A group is a way to add disks without moving files. Members must be existing file or S3
stores that are not in another group and not used by repositories directly; a member that holds blobs
cannot be removed from the group.

## Soft quotas

A store can have a **soft quota**: either a limit on the space it uses, or a minimum of free space.
Passing the quota does not stop writes; the check **Blob Stores Quota** under
**Settings → Support → Metrics and Health** turns red, which is what you alert on.

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
