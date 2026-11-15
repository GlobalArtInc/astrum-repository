# Docker

Choose **Use path routing** as the routing mode of the Docker repositories (see
[reverse proxy](../configure/reverse-proxy.md#docker-repositories)). The repository name is the first
part of the image name:

```shell
docker login astrum.example.com
docker pull astrum.example.com/docker-group/library/nginx:1.27
docker tag app:1.0 astrum.example.com/docker-hosted/team/app:1.0
docker push astrum.example.com/docker-hosted/team/app:1.0
```

podman, buildah, skopeo, crane and containerd use the same names. The [quick start](../getting-started.md)
walks through a proxy of Docker Hub, a group and a hosted repository with these commands.

## Anonymous pulls

A new Docker repository has **Force Basic Authentication** set: every request must carry credentials, so
`docker login` is needed even when anonymous access is on. For pulls without a login allow
[anonymous access](../configure/authentication.md#anonymous-access) and clear that option in the
repository clients pull from, usually the group; the members of a group can keep it. The
**Anonymous Access** column of the list of repositories says **Yes** for a repository that an anonymous
`docker pull` can reach.

The **Docker Bearer Token Realm**, which is off by default, makes the server challenge clients with a
bearer token from `/v2/token` instead of asking for Basic credentials. Signing in with `docker login`
works either way.

## OCI artifacts

Docker repositories implement the OCI Distribution Specification 1.1 and store any OCI artifact, not
only container images: Helm charts, signatures, SBOMs, attestations, Wasm modules, AI models. Add
`--plain-http` to oras and helm when the server is reached over plain HTTP.

```shell
helm registry login astrum.example.com
helm push mychart-1.2.0.tgz oci://astrum.example.com/docker-hosted/charts
helm install mychart oci://astrum.example.com/docker-group/charts/mychart --version 1.2.0

oras push astrum.example.com/docker-hosted/team/model:q4 \
  --artifact-type application/vnd.cncf.model.manifest.v1+json weights.gguf:application/vnd.cncf.model.weight.v1.raw
oras attach --artifact-type application/spdx+json \
  astrum.example.com/docker-hosted/team/app:1.0 sbom.spdx.json:application/spdx+json
oras discover astrum.example.com/docker-group/team/app:1.0

cosign sign --key cosign.key astrum.example.com/docker-hosted/team/app@sha256:...
notation sign astrum.example.com/docker-hosted/team/app@sha256:...
```

Signatures, SBOMs and attestations attached through `subject` are listed by the referrers API
(`GET /v2/<name>/referrers/<digest>`, filtered with `?artifactType=`). A hosted repository also lists
what a client kept under the tag of the referrers tag schema (`sha256-<hex>`). A proxy asks its remote,
reads that tag when the remote has no referrers API, and keeps listing the referrers it has cached when
the remote is down. A group lists the referrers of all its members. A mount from another Docker
repository (`from=<repository>/<name>`) copies the blob when the user may read it there.

Search and the repository browser tell artifacts apart: `docker.artifactKind` is `image`, `helm`,
`signature`, `sbom`, `attestation`, `wasm`, `model` or `other`.

What is attached to a manifest lives and dies with it. Deleting a manifest by its digest through the
registry API deletes the manifests whose `subject` it is, recursively, the tags that point at it and the
tags of the tag schema derived from its digest (`sha256-<hex>`, `.sig`, `.att`, `.sbom`). Deleting a tag
deletes only the tag. Layers are not removed with the manifests; the task **Docker - Delete unused
manifests and images** deletes the layers that no manifest refers to, and untagged manifests, once they
are older than its deploy offset (24 hours by default).

## Immutable tags

A release tag such as `v1.2.0` should keep pointing at the image it was released with. A hosted Docker
repository lists the tags that may never change in **Immutable tags**. A tag that matches is created once;
pushing it again with the same digest succeeds (a rerun of a pipeline), but it can never be moved to
another image and never be deleted.

| Setting | Meaning |
| --- | --- |
| Immutable tags | Tag patterns, one per line. Empty means the feature is off. |
| Mutable exceptions | Tags that match a pattern above and one of these stay free to change, for example `latest`. |

A pattern is matched against the whole tag: `*` stands for any characters, `?` for one character and
`[a-z]` for a class, so `v*` protects `v1.2.0` and `[0-9]*` protects `1.0`. A list takes at most 32
patterns of at most 128 characters, made of the characters tags are made of and `*?[]`. Through the REST
API the lists are `storage.immutableTags.patterns` and `storage.immutableTags.exceptions` of the
repository:

```json
{ "storage": { "immutableTags": { "patterns": ["v*", "[0-9]*"], "exceptions": ["latest"] } } }
```

The rule applies at once, also to tags that exist already, and it is enforced where content changes, not
only by the registry API:

- A push that would move a protected tag to another digest is refused with `400 DENIED` and a text like
  `tag v1.2.0 of team/app is immutable in repository docker-hosted (rule v*)`. Pushing the same digest
  again is accepted.
- Deleting a protected tag, or deleting by digest a manifest that a protected tag points at, is refused
  with the same text, and nothing is deleted: not the manifest, not its other tags, not the signatures
  attached to it. Deleting the other tags of an image, or an image without protected tags, works as before.
- The REST API answers `409` to the delete of a component or an asset that holds a protected tag. The
  browser skips the protected files when a folder is deleted and says how many it kept; it answers `409`
  when nothing but protected files were left.
- Cleanup policies keep a component that holds a protected tag and the task summary says how many were
  kept. The garbage collection of unused manifests never removes a manifest that a tag reaches.
- The tags of the referrers tag schema (`sha256-<hex>` with `.sig`, `.att` or `.sbom`) are always free, so
  `cosign sign` can sign an image again.
- Deleting the whole repository still works.

Administrators have no bypass. To change or remove a protected tag, edit the rule (the change is audited
as an update of the repository), change the tag, and add the rule back.

Limits to know about:

- The rule protects tags, not what they reach. A user with the right to delete can still delete a layer or
  the child manifest of an index through the registry API, which makes the protected image impossible to
  pull. Grant the delete right on a repository with immutable tags only to those who need it, for example
  with a [content selector](../configure/authentication.md).
- If the stored rule cannot be read, for instance after a bad edit of the repository document, every
  existing tag of the repository counts as protected until the rule is fixed, and the error names the
  broken field. New tags can still be created, and the repository and the rule can still be changed or
  deleted.

## Vulnerability scanning and the pull gate

A Docker repository with a [content policy](../repositories/policies.md) is scanned per tag. The scanner reads the
SBOM attached to an image and the packages in its layers, and matches them with the advisory data. In `block` mode
the pull gate refuses `GET` and `HEAD` of a manifest of a blocked image with HTTP `403` and the OCI error code
`DENIED`. Blobs, tag lists and the referrers API are not checked. The rules are in
[Image scanning and the pull gate](../repositories/image-scanning.md), and the scanner itself is described in
[Vulnerability scanning](../repositories/security-scanning.md).
