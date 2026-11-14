# SBOMs

An SBOM is a document that lists the packages inside an image. A build tool can attach one to an image as an
OCI referrer, or store it under a legacy tag. Astrum Repository finds the SBOMs of an image and reads their
package lists. It shows the packages and does not keep the documents: each request reads the document again.

## Formats

| Format | Read |
|---|---|
| SPDX 2.x JSON | Yes. The `spdxVersion` must start with `SPDX-2.` |
| CycloneDX 1.2 to 1.7 JSON | Yes |
| in-toto statements with an SPDX or CycloneDX predicate | Yes, the predicate is read. Attestations of the image are read the same way |
| DSSE envelopes with the in-toto payload type around either of them | Yes, the payload is read from the envelope |
| SPDX 3, XML and other SBOM formats | No. They answer `422` with the category `unsupported` |

The server looks at the media types of the document layers: `application/spdx+json`,
`application/vnd.cyclonedx+json`, `application/vnd.in-toto+json`, `application/vnd.dsse.envelope.v1+json` and
`text/spdx+json`. It reads at most four of them per request, in the order of the manifest, and the first one that
parses is used.

## Finding the SBOM of an image

The supply-chain listing of an image shows the SBOMs and attestations the server holds for it:

`GET /service/rest/v1/docker/{repository}/supply-chain?image=<name>&reference=<tag or digest>`

Each entry of `sboms` has a `digest` (the digest of the SBOM artifact) and a `format`. The artifacts are found as
OCI referrers of the image whose `docker.artifactKind` marks them as SBOMs or attestations. The server derives that
kind from the manifest content (its artifact type, config and layer media types), not from the tag name. The
artifacts are also found through the legacy tags `sha256-<hex>.sbom` and `sha256-<hex>.att`. The `format` in the listing is a hint taken from the artifact
type or the media type: `spdx`, `cyclonedx` or `unknown`. The format of the document as read is in the answer of
the SBOM route. Signatures and the rest of the listing are described in [image signatures](image-signatures.md).

## Reading the packages

`GET /service/rest/v1/docker/{repository}/sbom`

| Parameter | Required | Meaning |
|---|---|---|
| `image` | yes | The image name |
| `digest` | yes | The digest of the SBOM or attestation artifact, as listed in `sboms`. A tag is refused with `404` |
| `page` | no | The page, from 1. Default 1 |
| `filter` | no | A text to look for in the name, the purl and the licences of each package, without regard to case. At most 200 characters |

The answer:

```json
{
  "format": "spdx",
  "total": 2,
  "page": 1,
  "pageSize": 100,
  "packages": [
    {
      "name": "openssl",
      "version": "3.0.13",
      "purl": "pkg:deb/debian/openssl@3.0.13-0+deb12u1?arch=amd64&distro=debian-12",
      "licenses": ["Apache-2.0"],
      "ecosystem": "Debian:12"
    }
  ]
}
```

- `format` is `spdx` or `cyclonedx`, as read from the document.
- `total` is the number of packages after the filter. A page holds 100 packages.
- The packages are sorted by name, then by version, before the page is taken.
- `purl` is `null` when the package has none. `ecosystem` is `null` unless the purl names a known ecosystem.
- `licenses` holds the licences as the document writes them. They are not normalised. An SPDX package gives its
  concluded licence, or its declared licence when that is missing or is `NOASSERTION` or `NONE`. A CycloneDX
  entry gives its licence id, its licence name or its expression, and at most 32 entries are kept per package.

### Ecosystems

The ecosystem is derived from the purl type:

| Purl type | Ecosystem |
|---|---|
| `npm` | `npm` |
| `maven` | `Maven` |
| `pypi` | `PyPI` |
| `golang` | `Go` |
| `cargo` | `crates.io` |
| `nuget` | `NuGet` |
| `gem` | `RubyGems` |
| `deb` | `Debian:<major>` for a `distro=debian-<major>` qualifier, and `Ubuntu:<yy.04>:LTS` for an Ubuntu long-term release such as `distro=ubuntu-24.04`. Other distributions have none |
| `apk` | `Alpine:v<major>.<minor>` for a `distro=alpine-<major>.<minor>` qualifier |

Any other purl type gives `null`.

### Errors

Errors have the body `{"id": "*", "message": "…", "category": "…"}`.

| Status | Category | When |
|---|---|---|
| `400` | | `image` or `digest` is missing, `page` is not a number from 1, `filter` is longer than 200 characters, or the repository is not a Docker repository |
| `404` | | The digest is not an SBOM or attestation of the image, or a document blob is not stored in the repository |
| `413` | `too_large` | The documents read for the request are larger than 32 MiB |
| `422` | `malformed`, `too_deep`, `too_many_packages` or `unsupported` | The document cannot be read: it is not valid JSON, it nests deeper than 64 levels, it has more than 200 000 packages, or it is not a supported format. Also `unsupported` when the artifact has no document layer |
| `503` | `busy` | Three SBOMs are already being read on the server. The answer has `Retry-After: 5` |

## Limits

- 32 MiB of document text per request, across the layers read.
- At most four document layers are read per request.
- 200 000 packages per document. A document with more fails with `too_many_packages`.
- JSON nested deeper than 64 levels is refused.
- A string longer than 1 KiB is cut to 1 KiB. Its package is kept.
- At most 32 licences per package.
- At most three documents are parsed at a time per server. The others get `503` with `busy`.

## What is left out

- CycloneDX components of type `file` that have no purl.
- SPDX packages with no purl whose primary purpose is `FILE`, `APPLICATION` or `SOURCE`.
- Packages with no name. An empty purl counts as no purl, so such a package is kept with `purl: null`.

## The child index

An index manifest (a multi-platform image) lists its platform images as children. The server records these
children in the table `docker_manifest_child` when a manifest is stored. The image signature check uses that
table, so that a signature on an index covers its platform images (see [image signatures](image-signatures.md)).

Index manifests stored before this table existed have no recorded children. The task
**Docker - Index manifest children** (task kind `docker.children`, see [tasks](../operate/tasks.md)) records them.
Its property `repository` names one hosted or proxy Docker repository, or `*` for all of them. Groups are not
accepted. Run it once for each Docker repository that holds indexes, after the upgrade that introduced the
child index.
