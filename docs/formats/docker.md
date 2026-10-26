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
