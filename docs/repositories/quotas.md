# Quotas

A quota is a size limit on one repository. It is stored with the repository as `storage.quota`, so the
repository form, the REST routes that create and update repositories and the MCP tools all read and write
the same settings.

```json
{
  "storage": {
    "quota": { "limitBytes": 10737418240, "warnPercent": 90, "enforce": true }
  }
}
```

- `limitBytes`: the limit in bytes, above 0. `null` (or no `quota` at all) means no quota.
- `warnPercent`: the share of the limit, from 1 to 100, from which the repository is reported as near full. The default is 90.
- `enforce`: `true` refuses uploads that would take the repository over its limit. Only hosted repositories can enforce.

## Which repositories have a quota

- **Hosted** repositories can have a hard limit (`enforce: true`) or only a warning (`enforce: false`).
- **Proxy** repositories have a soft limit only. A proxy over its limit keeps caching and reports the warning. Free the space with [cleanup policies](cleanup-policies.md).
- **Group** repositories have no quota. A group with a `quota` is refused when saved.

## What is counted

The quota counts the logical size of the assets in the repository: the sum of the sizes of its assets. A blob that
two docker repositories share counts in both. The counter is kept by database triggers on the assets, so
deleting assets, deleting components and cleanup update it as well.

## What happens at the limit

Once a hosted repository with `enforce: true` would go over its limit, the upload is refused with HTTP 413. The
message names the repository and the limit. Nothing stays behind: the file that was being stored is discarded.

- Docker answers 413 with the OCI error code `DENIED`.
- Protobuf answers with the Connect code `failed_precondition`.
- Pub has its own error shape and reports `QuotaExceeded`.

These writes always pass, also while the repository is over its limit:

- deletes and reads;
- a replacement of an existing file that does not make the repository bigger, including one that makes it smaller.

When a write is refused both by the write policy (`DENY`, or `ALLOW_ONCE` for a file that exists) and by the quota, the
write policy refusal is the one reported.

When two uploads each fit alone but not together, exactly one of them is refused.

Promotion previews (`dry run`) do not check the quota.

## Usage

The usage of a repository is shown on its page, in the **Summary** tab: the bytes stored, the number of assets, the
limit, and a bar. The bar turns amber from `warnPercent` and red at 100 percent. A repository without a quota shows its size only.

The usage is also available from the REST API, `GET /service/rest/v1/repositories/{name}/usage`, which answers
`assets`, `bytes`, `quota` and `percentUsed`, and from the MCP tools `get_repository_usage` and `set_repository_quota`
([MCP](../use/mcp.md)). Prometheus exports `astrum_repository_bytes` and `astrum_repository_quota_bytes`
([monitoring](../operate/monitoring.md)).

## Limitations

- An upload that stores several files, or a file together with its generated index, is not atomic against the quota.
  Examples are a yum or apt repository's metadata, a docker manifest and its tag, and a raw or Maven upload of several
  files. The limit can refuse a later file after an earlier one was already stored. The result can be a stored
  package with a stale index until space is freed and the next upload regenerates the index.
- The early refusal compares the size the request declares with the limit, not with the room left. A full repository
  therefore reads a body up to the limit before it refuses.
- Promotion previews (`dry run`) do not check the quota.
