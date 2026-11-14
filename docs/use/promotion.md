# Promote components

Promotion copies components from one hosted repository to another hosted repository of the same format, or moves them, on the server. A build lands in a staging repository, passes its checks, and the same bytes are promoted to the release repository: nothing is downloaded and uploaded again, and checksums are verified on the way.

Thirteen formats can be promoted: apt, Docker, Git LFS, Helm, Maven, npm, NuGet, PyPI, raw, RubyGems, Swift, Terraform and yum. The [formats](#formats) section says what each one moves. The other formats answer `unsupported` for an item. Proxy and group repositories are neither a source nor a target; a request that names one is refused with `400`.

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
| `conflict` | Different content is there and `conflict` is `fail`; or the target's write policy `ALLOW_ONCE` refuses to replace it; or an [immutable tag](../formats/docker.md#immutable-tags) protects a tag; or the content changed while the item ran. |
| `forbidden` | You lack a right the item needs. The message names the kind of right, never other content. |
| `notFound` | The component does not exist or you may not see it. |
| `unsupported` | The format has no promotion. |
| `failed` | Something went wrong, or the target refuses the write: a read-only (`DENY`) target refuses an item that would write. |

A result also carries `componentId` and `componentIds`, `assets` (each path with its status: `new`, `replaced`,
`unchanged` or `conflict`), `assetCount`, `bytesCopied` and `bytesReused`, `sourceKept` for a move, `retryable`, a
`warning` (for example that an image without a tag is removed by the next garbage collection) and `skippedChildren`
for a Docker index. The reason for `conflict`, `forbidden`, `notFound`, `unsupported` and `failed` is in `error`.
`assetCount` counts every asset of the component that the item covers, unchanged ones included; `bytesCopied` counts
only the bytes written. `retryable` is true when repeating can help (the content changed while the transfer ran, or
the request ran out of time before the item started) and false when it cannot (the target holds different content, or
an immutable tag protects a tag until the rule is changed).

## Formats

The table lists what each format moves and what the target writes or computes after the move. Components are
promoted as the target would store them: an item that the target's own upload path would not accept is refused.

| Format | Moves | After the move | Limits |
|---|---|---|---|
| apt | `.deb` packages, at the pool path an upload stores them. | The `Release`, `InRelease`, `Release.gpg` and `Packages` files of the distribution are rebuilt from the packages the target holds, right after the copy. They are signed with the target's key when it has one. | A `.udeb` is refused: its path does not match the pool path. A `.ddeb` from a proxy source is left out without an error. |
| Docker | The manifests of the image, its layers and config (blobs the target has are reused), the tags you name, and with `referrers` the artifacts attached to the image: signatures, SBOMs and attestations. | Nothing. | A move deletes only the tag, never the manifest or layers. A move by digest is refused. Under `ALLOW_ONCE` a tag moves only when it is `latest` and the repository's `latest` policy allows it. |
| Git LFS | The object, under its SHA-256 id. | Nothing. | The path must be the object's SHA-256 path. |
| Helm | The chart `.tgz` with its `.prov` provenance file, kept with the chart. | `index.yaml` is computed on read; nothing is written. | Charts up to 256 MiB. |
| Maven | Artifacts (JAR, POM and other files) at their paths. A POM keeps the attributes read from it. | Version metadata, the archetype catalog and the index are computed on read. Checksums are served from the stored digests of each file. | `.sha1`, `.md5`, `.sha256` and `.sha512` files are not copied. A plugin group's `maven-metadata.xml` is copied as its plugin list only; other metadata is not copied. The target's version policy and strict layout apply. |
| npm | The tarball and the version entry, merged into the target's packument under the package lock. | The `dist` tarball address, `shasum` and `integrity` of the version come from the bytes the target stored. | A tarball that does not match its manifest is refused before anything is committed. `latest` moves only to a version that is not older than the target's `latest`. Other dist-tags that point at the version move with it. |
| NuGet | `.nupkg` packages, and `.snupkg` symbols packages with their package. | The registration, catalog and version documents are computed on read. | A symbols package without its package in the bundle is accepted only when the target holds the package. An id that differs from a stored id only in case is refused. |
| PyPI | Source and wheel files, at the path an upload stores them (normalised name and version). | The `/simple/` pages are rendered on read. A wheel's `.metadata` file (PEP 658) is written from the METADATA inside the wheel. | The `.metadata` files of a bundle are not copied; the target writes its own. |
| raw | Every file. The component of a file is its parent directory. | Nothing. | None beyond the target's write policy. |
| RubyGems | `.gem` files, at the path their specification gives. | `specs.4.8.gz`, `latest_specs.4.8.gz`, `prerelease_specs.4.8.gz`, `versions` and `names` are computed on read. | The gem's path must match its specification. |
| Swift | The source archive and the manifests of the release (`Package.swift` and its tools-version variants). The release metadata (checksum, metadata, publish time, description, repository URLs, signing) travels as component attributes. | Nothing. | A manifest the target holds for the release and the source does not is kept. A `checksum` in the metadata that does not match the archive is refused. |
| Terraform | Module archives, and provider files: archives, `SHA256SUMS`, its signature and the manifest. | A `SHA256SUMS` and signature that the server generated are not copied. For each promoted provider the target regenerates them and signs with the repository's signing key, if it has one. | A `SHA256SUMS` uploaded by hand is copied as it is. A module version that the target already holds under another archive name is refused. |
| yum | RPM packages and other files. | `repodata` is not copied. The target schedules a rebuild for each directory a package landed in, shortly after the copy. | `repodata` paths are refused. A repository with the strict layout refuses non-RPM files and packages above its repodata depth. |

Not promotable:

- Hugging Face repositories are proxies only, so there is nothing to promote.
- Protobuf: the Buf registry keeps commit and label records that are not part of a module's content, so a target cannot derive them from a copy.
- These formats have no promotion: alpine, ansible, bazel, cargo, chef, chrome, cocoapods, composer, conan, conda, firefox, go, hex, jetbrains, nix, nx, openvsx, p2, pacman, pub, puppet, r, turbo and vagrant.

## Rights

Rights are checked with your own account, and content selectors apply to every path:

- **Read** on every source path of the component, and the right to browse the source at all. If you can see some of
  the paths of the component but not all of them, the answer is `forbidden`; if you can see none, it is `notFound`.
- **Add** on every path the item would hold in the target, also for content that is unchanged or in conflict, and
  **Edit** on every path of the item when the plan replaces anything.
- **Delete** on the source for a move.

A token with the READ scope cannot promote. The rights are read once when the request starts: a right taken away
while a long request runs does not stop it.

## What a run does

Each item is its own transfer, in its own transaction, and a failure of one item does not stop the others. The work
continues on the server when the client disconnects, so a request is never cut in half. A request has a time budget
of ten minutes; items that did not start in time answer `failed` with a hint that a repeat may succeed.

Each item is planned first, with the rules of a dry run. Only an item that would copy content is then written; an
item that is `unchanged`, `skipped` or in `conflict` writes nothing. Content that already exists in the target is
checked, not written again.

Under `conflict: replace`, content that differs is replaced only where the write policy of the target allows it:
`ALLOW` replaces, `ALLOW_ONCE` refuses (with the one tag exception of Docker described above), and `DENY` refuses.
An immutable tag is never replaced.

An attribute-only change is a difference in the attributes of a component whose assets are the same. Only formats that
keep attributes on the component have one; Swift does, for its release metadata. It counts as a replacement: the
assets of the component are reported as `replaced`, the item needs **Edit**, `conflict: fail` reports a conflict,
`skip` skips the item, and `ALLOW_ONCE` refuses it.

A copy can succeed while the derived data of the target is not refreshed. apt rebuilds its metadata and Terraform
regenerates the checksums of the promoted provider right after the copy; a failure there is logged. The answer does
not show these warnings yet. yum schedules its repodata rebuild in the background, and a failure of that is logged
only.

Limits per transfer: 10 000 assets and 100 components. Carried attributes (the Swift release metadata) are at most
64 KiB in total, nested at most 8 levels, and hold no control characters; tabs and line breaks are allowed in values
but not in names. The name `replication` is reserved and is never carried.

### Move

A move deletes the source component after a successful copy, and also when the target already holds identical
content, so that running it again completes a move that was cut short. The source is deleted only when every asset of
the item is in the target, copied or identical. It is never deleted after `skipped` or `conflict`. The component is
deleted with all its assets; `assetCount` in the result counts the assets the item covers.

The source is kept, with `sourceKept` true and the reason in `error`, when the item was skipped or is in conflict, when
the source changed while the move ran, when an immutable tag protects it, or when the delete fails.

For Docker a move deletes only the tag in the source, never the manifest and layers, which the garbage collection
removes when nothing refers to them; moving an image by digest is refused because a digest delete would take all its
tags.

### Docker and npm

A Docker item copies the manifests of the image, its layers and, with `referrers`, the signatures, SBOMs and
attestations attached to it, and the tags you name. Blobs the target already has are reused.

An npm item copies the tarball and merges the version into the packument of the target under the lock of the
package, with the address and checksums of the entry taken from the bytes the target stored. A tarball that does not
match its manifest is refused before anything is committed. The dist-tags that point at the version move with it
when the source lists them and `distTags` is on. `latest` moves to the promoted version only when that version is not
older than the `latest` the target holds, by semantic version precedence; a target without a `latest` takes it.

## Audit, notifications and agents

A run that copies a component is audited in the domain `repository.content` as `promoted`; a move that deleted its
source is audited as `moved`. The event carries the source, the target, the component, the assets, the bytes and how
the request came in (`rest` or `mcp`). Dry runs, and items that neither copy nor move, are not audited. Channels can
subscribe to the event `promoted` of the [notifications](web-interface.md): a promoted component raises one notice,
whether it is new in the target or only had assets replaced. A channel that takes `published`, `updated` and `deleted`
gets `promoted` too; other channels turn it on by hand.

Agents use the tool `promote_components` ([Agents over MCP](mcp.md)); it writes to the server, and `dryRun` previews.
