# Promote components

Promotion copies components from one hosted repository to another hosted repository of the same format, or
moves them, on the server. A build lands in a staging repository, passes its checks, and the same bytes are
promoted to the release repository: nothing is downloaded and uploaded again, and checksums are verified on the way.

Promotion works for Docker images, npm packages, Maven, Helm and raw components. Other formats answer
`unsupported` for an item. Proxy and group repositories are neither a source nor a target.

## From the web interface

Open a component and press **Promote** on the row of the repository that holds it, or tick up to 100 results in
the search and press **Promote selected** (all from one repository). The dialog asks for:

- **Target**: the hosted repositories of the same format where you may add content.
- **Move**: delete the source after copying. Off by default.
- **If content differs**: *fail* (default), *skip* or *replace*.
- **Extra tags** (Docker): more tags for the image in the target.

**Preview** shows what a real run would do without writing anything. With **Move** on, the real run is possible only
after a preview of the current choices. The result table shows one line per item: its status, the assets and bytes,
errors and warnings, whether the source was kept, and a hint to try again when a repeat may succeed.

## With REST

```shell
curl -u "$USER:$TOKEN" -H 'Content-Type: application/json' \
  -X POST https://astrum.example.com/service/rest/v1/promotions -d '{
    "source": "staging", "target": "releases",
    "items": [{"namespace": "com.acme", "name": "app", "version": "1.4.0"}],
    "conflict": "fail", "dryRun": true }'
```

The body takes `source` and `target` (repository names), `items` (at most 100), and the options `move` (default
false), `conflict` (`fail`, `skip` or `replace`), `dryRun`, `referrers` (Docker: take the signatures and other
artifacts attached to an image) and `distTags` (npm: move the dist-tags of the version, default true; other formats
ignore it). An item names a component by `componentId`, or by `namespace`, `name` and `version`, or, for Docker, by
`image` with a `tag` or a `digest` and optional `targetTags`. Names that contain invisible formatting characters are
refused; use the `componentId` for them.

The answer is `{dryRun, results: [...]}` with one result per item, in order:

| Status | Meaning |
|---|---|
| `copied` | The component is in the target now. |
| `unchanged` | The target already holds identical content. |
| `skipped` | Different content was there and `conflict` is `skip`. |
| `conflict` | Different content is there and `conflict` is `fail`, the write policy or an [immutable tag](../formats/docker.md#immutable-tags) forbids replacing it, or the content changed while the item ran. |
| `forbidden` | You lack a right the item needs. The message names the kind of right, never other content. |
| `notFound` | The component does not exist or you may not see it. |
| `unsupported` | The format has no promotion. |
| `failed` | Something went wrong; the text says what. |

A result also carries `assetCount`, `bytesCopied` and `bytesReused`, `sourceKept` for a move, `retryable`, a `warning`
(for example that an image without a tag is removed by the next garbage collection) and `skippedChildren` for a Docker
index. `retryable` is true when repeating can help (the content changed while the transfer ran, or the request ran out
of time before the item started) and false when it cannot (the target holds different content, or an immutable tag
protects a tag until the rule is changed).

## Rights

Rights are checked with your own account, and content selectors apply to every path:

- **Read** on every source path of the component, and the right to browse the source at all; otherwise the answer is
  as for a component that does not exist.
- **Add** on every new path in the target, also for content that is unchanged or in conflict, and **Edit** on all
  paths when the plan replaces anything.
- **Delete** on the source for a move.

A token with the READ scope cannot promote. The rights are read once when the request starts: a right taken away
while a long request runs does not stop it.

## What a run does

Each item is its own transfer, in its own transaction, and a failure of one item does not stop the others. The work
continues on the server when the client disconnects, so a request is never cut in half. A request has a time budget
of ten minutes; items that did not start in time answer `failed` with a hint that a repeat may succeed.

Content that already exists in the target is checked, not written again. Under `conflict: replace` content that
differs is replaced only where the write policy of the target allows it (`ALLOW`; `ALLOW_ONCE` and `DENY` refuse), and
never over an immutable tag. A dry run follows the same rules and writes nothing.

### Move

A move deletes the source component after a successful copy, and also when the target already holds identical
content, so that running it again completes a move that was cut short. It never deletes after `skipped` or
`conflict`, and never when the copy found nothing to write. The component is deleted with all its assets, including
generated ones such as checksum files; `assetCount` counts only what was copied. The source is kept, with `sourceKept`
and the reason in the result, when the source changed while the move ran, when an immutable tag protects it, or when
the delete fails.

For Docker a move deletes only the tag in the source, never the manifest and layers, which the garbage collection
removes when nothing refers to them; moving an image by digest is refused because a digest delete would take all its
tags.

### Docker and npm

A Docker item copies the manifests of the image, its layers and, with `referrers`, the signatures, SBOMs and
attestations attached to it, and the tags you name. Blobs the target already has are reused.

An npm item copies the tarball and merges the version into the packument of the target under the lock of the
package, with the address and checksums of the entry taken from the bytes the target stored. A tarball that does not
match its manifest is refused before anything is committed. The dist-tags that point at the version move with it
when the source lists them and `distTags` is on; `latest` moves to the promoted version if the source says so.

## Audit, notifications and agents

Every real run is audited in the domain `repository.content` as `promoted`, with the source, the target, the
component, the assets, the bytes and how the request came in (`rest` or `mcp`); dry runs are not audited. Channels can
subscribe to the event `promoted` of the [notifications](web-interface.md): a promoted component raises one notice,
whether it is new in the target or only had assets replaced. A channel that takes `published`, `updated` and `deleted`
gets `promoted` too; other channels turn it on by hand.

Agents use the tool `promote_components` ([Agents over MCP](mcp.md)); it writes to the server, and `dryRun` previews.
