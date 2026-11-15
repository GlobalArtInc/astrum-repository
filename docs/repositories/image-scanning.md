# Image scanning and the pull gate

The scanner judges Docker repositories per tag. For each tag component it builds an inventory of the image the
tag points at, from the packages named by an attached SBOM and the packages in the image's layers. The inventory
is matched with the same advisories as the packages of the other formats (see
[Vulnerability scanning](security-scanning.md)). The pull gate then refuses manifest requests for images that the
content policy of the repository blocks.

The policy fields are described in [Content policies](policies.md). The limits below are the values of this
version.

## What is judged

- A tag is a component. Pushing a tag, or moving it to another digest, queues the component when the repository has a
  policy. A manifest without a tag is not a component and is not queued by itself.
- An inventory belongs to one repository and one digest. Its key is `oci:<repository>:<digest>`. A verdict belongs to
  one repository and one digest too, with the subject `oci:<digest>`. An image pulled by tag or by digest gets the
  same verdict.
- The pull gate enforces the `vulnerabilities` block of a policy. It does not read the `licenses` or the `signatures`
  block.

## How an image gets an inventory

An inventory is a list of packages, each with an ecosystem, a name, a version and a purl, and a flag `complete`.
The component is `scanned` when the flag is set and `incomplete` when it is not.

### Packages named by an SBOM

The scanner reads the SBOMs and attestations that the repository holds for the digest. It finds them in three places:

- OCI referrers of the digest whose kind is SBOM or attestation (`docker.artifactKind`);
- the legacy tags `sha256-<hex>.sbom` and `sha256-<hex>.att` of the digest;
- BuildKit attestation manifests: entries of an index whose annotations are `vnd.docker.reference.type` set to
  `attestation-manifest` and `vnd.docker.reference.digest` set to the image.

The documents are read in the formats of [SBOMs](sbom.md): SPDX 2.x JSON, CycloneDX JSON, and in-toto statements or
DSSE envelopes around them. The reader takes at most four document layers per artifact.

What the reader does with a document:

- A document that does not parse, a document blob that the repository does not hold, or a document that does not fit
  the 32 MiB budget of one manifest makes the inventory incomplete.
- A document in a format the reader does not support (SPDX 3, or an attestation whose predicate is not an SBOM, such
  as provenance) is skipped. It does not make the inventory incomplete.
- More than eight artifacts for one image, or 200 or more referrers, make the inventory incomplete.
- A purl that cannot be mapped (see [Package URLs and ecosystems](#package-urls-and-ecosystems)) makes it incomplete.
  A package without a purl is not listed and does not make it incomplete. A document whose packages all fail to map
  makes it incomplete.

### Packages in the layers

The scanner reads the layers in order, applies the whiteout markers of the OCI layer format (`.wh.<name>` removes a
path, `.wh..wh..opq` hides the lower content of a directory), and keeps only the files that identify the
distribution and its packages:

- the OS release file, `etc/os-release` or `usr/lib/os-release`;
- the Alpine database `lib/apk/db/installed`, the dpkg database `var/lib/dpkg/status`, and the files of
  `var/lib/dpkg/status.d/` (distroless images).

The distribution comes from `ID` and `VERSION_ID` of the OS release file. An image has an ecosystem only when the ID
and the version give one:

- `debian` with a numeric major version: `Debian:<major>`, read from the dpkg databases;
- `ubuntu`, long-term releases only (an even year and month `04`, such as `24.04`): `Ubuntu:<yy.mm>:LTS`, read from the
  dpkg databases. Other Ubuntu releases have no ecosystem;
- `alpine` with a major and minor version: `Alpine:v<major>.<minor>`, read from the apk database.

Packages are named by their source package, which is what the advisory data is keyed by: the origin of an Alpine
package, and the source name and source version of a Debian package.

An image with packages in its databases but without a supported distribution (no OS release file, no `VERSION_ID`,
or an ID that has no ecosystem) is incomplete, with the reason "the image has installed packages but its distribution
is unknown or unsupported". This includes distroless images whose OS release file is missing. A database of the
other package manager (dpkg status in an Alpine image, apk database in a Debian image) is skipped, and the inventory
is incomplete.

### Merge

The SBOM packages are added first, then the packages of the layers. The inventory keeps one entry per ecosystem, name
and version. When an SBOM package and a layer package have the same identity, the SBOM entry is kept, with its purl.

The inventory of one image holds at most 200,000 distinct packages. More than that makes it incomplete.

### Package URLs and ecosystems

The purl type decides the ecosystem of a package read from an SBOM:

| Purl type | Ecosystem | Name in the inventory | Matched by advisories |
|---|---|---|---|
| `npm` | `npm` | `scope/name` | yes |
| `maven` | `Maven` | `groupId:artifactId` (the namespace is required) | yes |
| `pypi` | `PyPI` | the name | yes |
| `golang` | `Go` | the module path | yes |
| `cargo` | `crates.io` | the name | yes |
| `nuget` | `NuGet` | the name | yes |
| `gem` | `RubyGems` | the name | no |
| `composer` | `Packagist` | `vendor/name` (the namespace is required) | no |
| `pub` | `Pub` | the name | no |
| `hex` | `Hex` | the name | no |
| `deb` with `distro=debian-<major>` or `distro=ubuntu-<yy.mm>` | `Debian:<major>` or `Ubuntu:<yy.mm>:LTS` | the `upstream` source name, else the name | yes |
| `apk` with `distro=alpine-<major>.<minor>` | `Alpine:v<major>.<minor>` | the name | yes |

The versions of `deb` packages are the source versions, with the epoch when the purl has one. Any other purl type,
and a `deb` or `apk` purl without a supported distribution qualifier, makes the inventory incomplete.

The matcher compares versions only for Maven, npm, PyPI, Go, crates.io, NuGet, and the Debian, Ubuntu and Alpine
ecosystems. A package of the other ecosystems in the table is counted as unresolved and never produces a finding.

### Advisory ecosystems that produce findings

A package produces findings only from the advisories of its ecosystem that the `advisories.sync` task has imported. The
default list of the task is `Maven,npm,PyPI,Go,crates.io,NuGet,Alpine,Debian` (`DEFAULT_ECOSYSTEMS`). The name
`Debian` covers every `Debian:<major>` and the name `Alpine` covers every `Alpine:v<major>.<minor>`.

`Ubuntu` is not in the default list. Packages of Ubuntu images produce no findings until you add `Ubuntu` to the
**Ecosystems** field of the task. Use the name that the OSV bucket uses for the ecosystem (the names are listed in
`ecosystems.txt` at the root of the bucket). Then run the task again.

### Index images

An index (a multi-platform image) is judged on the union of its children:

- The children are the platform manifests that this repository records for the index (table `docker_manifest_child`).
  An index with more than 50 children is incomplete, and only the first 50 are read.
- A child that the repository does not hold makes the index incomplete. A child that is itself an index makes it
  incomplete. A child that is an artifact (signature, SBOM or attestation) is skipped.
- The SBOMs of the index and of each child are read as described above, and the packages of each child's layers are
  read too. The packages of all of them are merged.
- When neither the SBOMs of the index nor any child contributes to the inventory, the index has no inventory.

A platform manifest that is pulled by digest has no inventory of its own in this version. The gate judges it through
its index (see [Index and platform images](#index-and-platform-images)).

### Images with no packages

An image with no SBOM and no package database, whose layers are read in full, gets no inventory. Its verdict then
reads `scan: missing`. With `unscanned: block` the gate refuses such an image.

## Layer extraction

### Limits

Each limit applies to one layer unless the table says otherwise.

| Limit | Value | What happens when it is reached |
|---|---|---|
| Layers in an image | 128 | Incomplete. No layer is read |
| Decompressed size of a layer | 4 GiB | The layer is cut. Incomplete |
| Compressed members (frames) of a layer | 1,000,000 | The layer is cut. Incomplete |
| Zstd window | 128 MiB | The layer is cut. Incomplete |
| Gzip header | 64 KiB | The archive is damaged. Incomplete |
| Input without output | 256 MiB of compressed input for less than 1 MiB of output | The layer is cut. Incomplete |
| Tar entries | 1,000,000 | The layer stops. Incomplete |
| Long names and pax records | 64 KiB | The layer stops. Incomplete |
| A package database or OS release file | 64 MiB | The file is not read. Incomplete |
| Package records kept | 20,000 | The rest is not read. Incomplete |
| Package data | 8 MiB | The rest is not read. Incomplete |
| Files in `status.d` | 4,096 | The extra files are not read. Incomplete |
| Whiteouts and unreadable databases | 4,096 | Incomplete |
| dpkg status files and package bytes, across all layers | 16 MiB | Incomplete |
| Distinct packages of an image | 200,000 | Incomplete |
| Document text read for the SBOMs of one manifest | 32 MiB | The document is not read. Incomplete |
| Time for the layers of one inventory | the scan time less 30 seconds | Incomplete |
| Time for one inventory | 10 minutes | The scan fails and is retried |

Other limits of the layer code:

- One layer is read at a time per node. The other scans wait for it.
- At most three SBOM documents are parsed at a time per node. The inventory waits for a slot. The SBOM viewer answers
  `503` instead.
- Parsed facts of a layer are cached (see [Cache](#cache)). A cached layer is not read again.

### What makes an inventory incomplete

The inventory is incomplete for any of these causes. The reasons are logged, and the first 16 distinct reasons of an
image are kept.

- The image has more than 128 layers, or there is no time left to read the layers, or reading them takes longer than
  the time left.
- A layer is larger than 4 GiB when decompressed, has more than 1,000,000 compressed members, asks for a window above
  128 MiB, or reads more than 256 MiB of input for less than 1 MiB of output.
- A layer has more than 1,000,000 entries, a long name or a pax record over 64 KiB, a pax header that cannot be
  parsed, or a zero block followed by more data.
- The archive is damaged. The reader reports the error and keeps the packages that it read.
- A package database or an OS release file is larger than 64 MiB, ends before its declared size, or is a link, a
  device, a sparse file or another entry that is not a regular file. A path of the layer could not be read.
- A package database has records that cannot be read. A layer lists more than 20,000 packages, or holds more than
  8 MiB of package data. A layer has more than 4,096 `status.d` files, unreadable databases or whiteouts of databases.
- A whiteout and an entry for the same path are in one layer. A package database path has a parent that is a file in
  that layer. The reader reports both as ambiguous, and the inventory is incomplete.
- The dpkg status files and the package bytes together exceed 16 MiB, or the image has more than 200,000 distinct
  packages.
- Packages of the other package manager are present and skipped.
- The image has installed packages but its distribution is unknown or unsupported.
- A layer is not held by a hosted or group repository. For a proxy, the remote has no layer, or the fetch fails with a
  non-internal error.
- An SBOM document is damaged, is missing from the repository, is too large, or has packages that do not map. The
  image has more than eight artifacts or 200 referrers. An index has more than 50 children, a child that the
  repository does not hold, or a child that is an index.

### Errors that are retried

Some failures are not incomplete. They fail the attempt, and the worker retries it after 1, 5 and 30 minutes. After the
fourth failure the component is `failed`:

- reading a layer from the blob store fails;
- a proxy fails to fetch a layer with an internal error;
- the inventory runs longer than the scan timeout of 10 minutes;
- the database fails.

### Malformed and ambiguous archives

The reader detects the compression from the first bytes of the layer: gzip, zstd, or plain tar. The media type of the
layer is not used. A damaged archive or an ambiguous entry is recorded as a reason, and the inventory is incomplete.

### Missing layers and proxies

A hosted or group repository must hold every layer of the image. A missing layer makes the inventory incomplete.

A proxy fetches a missing layer through the proxy path and stores it as it stores any blob. If the remote has no
layer, or the fetch fails, the inventory is incomplete.

### Cache

The parsed facts of a layer are cached per layer digest. The cache is shared by all repositories, and it holds
facts only, not the bytes of the layer. The facts of an incomplete read are cached too. A result is not cached when
the layer is missing, when the read is cut short by the deadline or by a cancellation, or when the facts are larger
than 8 MiB. An entry is used only when the layer reader that wrote it is the current version. An older entry is
replaced when the layer is read again.

## When images are scanned

These are the triggers for Docker components. The general rules are in [Vulnerability scanning](security-scanning.md#when-scans-run).

- A tag push creates or changes the component of the tag. It is queued when the repository has a policy.
- Attaching a policy to the repository queues every component of the repository.
- In a repository with a policy, an SBOM or an attestation pushed later queues the tags that describe the image. These
  are the tags of its subject and of up to 16 indexes that list the subject. A legacy `.sbom` or `.att` tag does the same for the digest it names.
  The queue takes at most 5,000 tags per push. The rest wait for the `scan.repository` task or a rescan.
- The `scan.repository` task and `POST /service/rest/v1/security/scan` queue components as described in
  [Vulnerability scanning](security-scanning.md#when-scans-run).
- An advisory import matches the packages whose advisories changed. The images that contain those packages are judged
  again. No rescan is needed.

### Extractor versions and re-scan after an upgrade

The inventory of a Docker component records extractor version 2. The other formats record version 1. The layer facts
cache has its own version, 3.

This version does not queue the existing Docker tags by itself. To scan them with the layer extraction, run the
`scan.repository` task for the repository:

- without **Extract again**, it queues the components that were never scanned, that were scanned by an older
  extractor, whose last scan was `failed` or `incomplete`, and the Docker components that are `unsupported`. An
  unsupported Docker component is queued until it has an inventory;
- with **Extract again**, it queues every component.

The queue settings of each node are those of [Vulnerability scanning](security-scanning.md#the-queue): `scan.workers`
(2 by default, at most 8), with a batch of 10 components per worker. The verdict sweeper runs every minute. It marks
the verdicts of images whose policy revision changed, and the verdicts of images that have no verdict. It removes the
verdicts of digests that no manifest or tag holds.

## The pull gate

### Requests covered

The gate runs on every `GET` and `HEAD` of a manifest, by tag or by digest, in hosted, proxy and group repositories.
It runs before the conditional-request check, so an `If-None-Match` header does not skip it. The manifest preview of
the web interface runs the same check.

In a group, the gate judges the member that serves the manifest, with that member's policy and verdicts. A repository
without a policy is served without a check.

Blobs, tag lists, the catalog and the referrers API are not covered (see [What the gate does not protect](#what-the-gate-does-not-protect)).

### Modes

The verdict is judged against the current revision of the policy. The table gives what a pull receives.

| Mode | The image has a verdict of the current revision | The image has no verdict of the current revision |
|---|---|---|
| `off` | Served | Served |
| `report` | A verdict in state `report` is shown as **Would be blocked**. Other verdicts are served | A platform image whose index has a `report` verdict of the same revision is shown as **Would be blocked**. Otherwise served |
| `block` | Refused when the state is `block`, `error` or any other state the gate does not know, or when the blocked flag is set. With `unscanned: block`, refused when the state is `unscanned`. Otherwise served | For a platform image, see [Index and platform images](#index-and-platform-images). Otherwise `unscanned: allow` serves it, and `unscanned: block` refuses it and queues the scan |

### Stale verdicts

A verdict belongs to one revision of the policy. When the policy changes, its verdicts are stale until they are
judged again:

- In `block` mode a stale verdict that blocks keeps refusing the image until the image is judged again.
- A stale verdict that does not block is ignored, and the image is treated as having no verdict.
- The verdict sweeper runs every minute. It marks the stale verdicts for judgement, and the verdict workers judge them.

### Index and platform images

A platform image that is pulled by digest is judged through its index when it has no verdict of its own:

- When the index is refused, the platform image is refused too, with the reason "the image is part of a blocked index"
  and the reason of the index.
- When the index has a complete pass of the same revision (no findings, no unscanned state, not blocked), the platform
  image is served.
- Otherwise the platform image follows the `unscanned` setting, as an image with no verdict.

A platform image that has a verdict of its own is judged on that verdict.

### The refusal

A refused pull gets HTTP status `403 Forbidden` and an OCI error body with the code `DENIED`:

```json
{"errors": [{"code": "DENIED", "message": "blocked by policy strict: 2 blocking advisories: GHSA-xxxx-xxxx-xxxx, GHSA-yyyy-yyyy-yyyy"}]}
```

The message is at most 400 characters. A longer message is cut and ends with an ellipsis. Its details are one of:

- `N blocking advisories: <identifiers>`, with at most eight identifiers, and `and K more` for the rest. Package names,
  versions, and the text of the advisories are not in the message;
- the error of the policy, such as `the policy cannot be read`;
- `the image has not been scanned (scan: <state>)`, where the state is `incomplete`, `missing` or `matching`;
- `the image is blocked`, when none of the above applies.

With `unscanned: block`, a refusal of an image that has no verdict says one of these:

- `the image is being scanned, retry`: the scan of its tags is queued;
- `no scan data for this image digest`: the digest has no tag, so there is nothing to queue;
- `the scan of the image could not be queued, retry`: the queue write failed or timed out.

When the gate cannot read the verdict, the pull gets `503` with the code `UNAVAILABLE` and the message `the content
policy could not be checked; try again`. It is never served.

### Unscanned images

- With `unscanned: allow`, the image is served. The gate does not wait for a scan and does not queue the image.
- With `unscanned: block`, the image is refused, and the gate queues its tags that are not queued yet. At most 16 tags
  are queued for one digest. The queue does not reset a scan that is already queued.
- When the image already has a verdict, a pull does not queue it again. Queue it with a rescan (see
  [Operator guidance](#operator-guidance)).

### What is exempt

A manifest is exempt from the gate when it is marked as an artifact and it has no filesystem layer.

A manifest is marked when either:

- it has a `subject`;
- its tag is a legacy digest tag with the suffix `.sig`, `.att` or `.sbom` (`sha256-<hex>.sig` and the others);
- its artifact kind is `signature`, `attestation` or `sbom`, and it is requested by a digest or by a tag without
  one of the suffixes above.

A tag whose name ends in `.sig`, `.att` or `.sbom` but is not a legacy digest tag is never marked by its kind.

A manifest has a filesystem layer when any string in it, a key or a value at any depth, contains `.tar`, `rootfs` or
`image.layer`, without regard to case. A manifest that is not JSON has one, so it is not exempt.

An image with a `subject` and a filesystem layer is judged, whatever its marks.

### Audit

A refusal writes an audit entry with the action `blocked` in the policy domain. There is one entry per digest and
repository per hour. Its attributes are the repository, the digest, the policy, the state, and the tag when the pull
was by tag. The reasons are not in the entry.

## What the gate does not protect

- **Blobs** by digest, `GET` and `HEAD`. The layers of a blocked image can be downloaded. Mounts from another
  repository (`from=`) are not checked either.
- **`tags/list`**, **`_catalog`** and the **referrers API** (`/v2/<name>/referrers/<digest>`). They list blocked images.
- **Exempt artifact manifests**: signatures, attestations and SBOM manifests by the rule above, also when their subject
  is blocked.
- **Repositories without a policy.** A group has no policy of its own. It applies the policy of the member that serves
  the image, and a member without a policy is not judged.
- **Unscanned images under `unscanned: allow`.** They are served.
- **Pushes.** Uploads and manifest `PUT` requests are not checked, so a blocked image can be pushed.
- **The first pull through a proxy under `unscanned: allow`.** The proxy fetches the manifest and stores it before
  there is a verdict, and the manifest is served.
- **Incomplete inventories under `unscanned: allow`.** The packages that were read are judged, so a blocking finding
  among them refuses the image. The packages that were not read are not judged.
- **Images with no packages.** They are judged as `unscanned` with `scan: missing`. Under `allow` they are served.
- **The licence and signature blocks** of a policy. The gate does not read them.

## Operator guidance

### Why is an image blocked or unscanned?

1. Read the message of the `403` response. It names the policy, and it lists the advisories that block, or it names
   the scan state.
2. Read the audit log. An entry with the action `blocked` shows the repository, the digest, the policy, the state and
   the tag.
3. For the vulnerabilities of a tag, use the component views in [Vulnerability scanning](security-scanning.md#findings).
4. For the packages of an SBOM, use the SBOM listing and the SBOM route in [SBOMs](sbom.md). These routes show the
   SBOM documents. No endpoint of this version lists the packages read from the layers. The server logs them with
   the reasons when the layers are not read in full.

The scan states of an image mean:

- `incomplete`: some layers or documents were not read. Look at the logged reasons, then at the causes in
  [What makes an inventory incomplete](#what-makes-an-inventory-incomplete).
- `missing`: the image has no SBOM and no package database. If the image really has no packages, this is expected.
  Add an SBOM, or allow unscanned images.
- `matching`: the inventory is complete, and its packages are being matched. Wait for the matcher.

### How to unblock

- **Exception by advisory.** Add an exception with the advisory identifier or an alias, a reason, and an expiry date
  when you can. The exception applies to the findings of that advisory in the images it covers.
- **Exception by name.** Add an exception with a component name. It matches the tags that point at the digest, and it
  can be narrowed to a namespace or version. At most 50 tags are considered.
- **Exception by digest.** Add an exception with the image digest. It applies to that digest in every tag.

  Exceptions are created in the interface under `/admin/security/policy-exceptions`, or with
  `POST /service/rest/v1/security/exceptions` (see [Content policies](policies.md#exceptions)).
- **Fix.** Push a fixed image under the tag. The new digest is queued and judged when its scan finishes.
- **Rescan.** `POST /service/rest/v1/security/scan` with `{"repository": "<name>"}`, optionally with `componentId`,
  queues the components again. The `scan.repository` task does the same for a repository.
- **Unscanned.** Wait for the scan to finish. Set `unscanned: allow` only as a decision that you accept, because it
  also serves images that the scanner did not read.

### After you change the policy

- A change to a policy, or to its exceptions, changes the revision of the policy. The verdict sweeper marks the
  verdicts with findings and the stale verdicts within about a minute, and the verdict workers judge them.
- Until a stale blocking verdict is judged again, the image stays refused.
- Attaching a policy queues every component of the repository. The first scans can take a while on a large
  repository.
- Follow the rollout in [Content policies](policies.md#rollout). Set `unscanned: block` last.
