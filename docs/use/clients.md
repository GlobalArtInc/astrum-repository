# Connect build tools to repositories

Every repository is served at `https://<host>/repository/<repository name>/`. The examples use
`astrum.example.com` and repository names that follow a `<format>-hosted`, `<format>-proxy`,
`<format>-group` convention; use the names you created. Point clients at a group when you read and at
a hosted repository when you publish.

Clients authenticate with the user name and password of a local user, or with a token where the format
has one. See [users, realms and sign-in](../configure/authentication.md) for the realms that tokens
need.

## Maven

`~/.m2/settings.xml`:

```xml
<settings>
  <mirrors>
    <mirror>
      <id>astrum</id>
      <mirrorOf>*</mirrorOf>
      <url>https://astrum.example.com/repository/maven-public/</url>
    </mirror>
  </mirrors>
  <servers>
    <server>
      <id>astrum</id>
      <username>ci-deployer</username>
      <password>${env.ASTRUM_PASSWORD}</password>
    </server>
  </servers>
</settings>
```

Publishing, in the project's `pom.xml`:

```xml
<distributionManagement>
  <repository>
    <id>astrum</id>
    <url>https://astrum.example.com/repository/maven-releases/</url>
  </repository>
  <snapshotRepository>
    <id>astrum</id>
    <url>https://astrum.example.com/repository/maven-snapshots/</url>
  </snapshotRepository>
</distributionManagement>
```

## Gradle

```kotlin
repositories {
    maven {
        url = uri("https://astrum.example.com/repository/maven-public/")
        credentials {
            username = providers.environmentVariable("ASTRUM_USER").get()
            password = providers.environmentVariable("ASTRUM_PASSWORD").get()
        }
    }
}
```

## npm, pnpm, Yarn

Activate the **npm Bearer Token Realm** first. Then:

```shell
npm config set registry https://astrum.example.com/repository/npm-group/
npm login --registry=https://astrum.example.com/repository/npm-group/ --auth-type=legacy
```

`npm login` stores a token in `~/.npmrc`. For CI write the token directly:

```ini
registry=https://astrum.example.com/repository/npm-group/
//astrum.example.com/repository/npm-group/:_authToken=${NPM_TOKEN}
//astrum.example.com/repository/npm-hosted/:_authToken=${NPM_TOKEN}
```

and publish with `npm publish --registry=https://astrum.example.com/repository/npm-hosted/`, or set
`publishConfig.registry` in `package.json`. A token for a user can also be requested without a browser:

```shell
curl -X PUT -H 'Content-Type: application/json' \
  -d '{"name":"ci-npm","password":"…"}' \
  https://astrum.example.com/repository/npm-hosted/-/user/org.couchdb.user:ci-npm
```

## Docker

Enable **Path-based routing** on the Docker repositories (see
[reverse proxy](../configure/reverse-proxy.md#docker-repositories)). The repository name is the first
part of the image name:

```shell
docker login astrum.example.com
docker pull astrum.example.com/docker-group/library/nginx:1.27
docker tag app:1.0 astrum.example.com/docker-hosted/team/app:1.0
docker push astrum.example.com/docker-hosted/team/app:1.0
```

podman, buildah, skopeo, crane and containerd use the same names. For anonymous pulls activate the
**Docker Bearer Token Realm** and allow anonymous access.

### OCI artifacts

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

What is attached to a manifest lives and dies with it. Deleting a manifest by its digest (through the
registry API or in the browser) deletes the manifests whose `subject` it is, recursively, and the tags
of the tag schema derived from its digest (`sha256-<hex>`, `.sig`, `.att`, `.sbom`). Cleanup policies
delete tags only and never the manifest a tag points at, so they never delete a subject while referrers
remain, and an image pinned by its digest keeps working after its tag was cleaned up. Layers are not
reference counted and stay when manifests go.

## Helm

```shell
helm repo add astrum https://astrum.example.com/repository/helm-group/ \
  --username ci-deployer --password "$ASTRUM_PASSWORD"
helm repo update
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file mychart-1.2.0.tgz \
  https://astrum.example.com/repository/helm-hosted/
```

## PyPI

`pip.conf` (or `PIP_INDEX_URL`):

```ini
[global]
index-url = https://astrum.example.com/repository/pypi-group/simple
```

Publishing with twine:

```shell
twine upload --repository-url https://astrum.example.com/repository/pypi-hosted/ \
  -u ci-deployer -p "$ASTRUM_PASSWORD" dist/*
```

uv and Poetry take the same URLs as an extra index.

## NuGet

```shell
dotnet nuget add source https://astrum.example.com/repository/nuget-group/index.json \
  --name astrum --username ci-deployer --password "$ASTRUM_PASSWORD" --store-password-in-clear-text
```

For publishing activate the **NuGet API-Key Realm**, take the API key from the user menu
(**NuGet API Key**) and push:

```shell
dotnet nuget push MyLib.1.0.0.nupkg \
  --source https://astrum.example.com/repository/nuget-hosted/ --api-key "$NUGET_API_KEY"
```

## Go

```shell
export GOPROXY=https://astrum.example.com/repository/go-proxy/,direct
export GONOSUMDB=git.example.com
```

Credentials for a protected proxy go into `~/.netrc` (`machine astrum.example.com login … password …`).

## Cargo

`~/.cargo/config.toml`:

```toml
[registries.astrum]
index = "sparse+https://astrum.example.com/repository/cargo-group/"

[registry]
global-credential-providers = ["cargo:token"]

[source.crates-io]
replace-with = "astrum"
```

Activate the **Cargo Bearer Token Realm**, get a token from
`curl -u <user> https://astrum.example.com/repository/cargo-hosted/me` and run
`cargo login --registry astrum <token>`. Publish with `cargo publish --registry astrum` against a
registry entry that points at `cargo-hosted` or at a group whose first writable member is hosted.

## Swift

Swift Package Manager resolves packages from a registry by identifier, `scope.name`, since Swift 5.7 and
publishes to one since Swift 5.9:

```shell
swift package-registry set --global https://astrum.example.com/repository/swift-group/
swift package-registry login https://astrum.example.com/repository/swift-group/ --username ci-deployer
```

and in `Package.swift`: `.package(id: "mona.LinkedList", from: "1.1.0")`. Add `--scope mona` to `set`
to use the registry only for one scope. Publish from the package directory to a hosted repository, or to
a group whose `writableMember` (set over the REST API) is hosted; `package-metadata.json` next to
`Package.swift` becomes the
release metadata:

```shell
swift package-registry publish mona.LinkedList 1.1.0 \
  --url https://astrum.example.com/repository/swift-hosted/
```

Swift only uses HTTPS registries. For a server on plain HTTP add `--allow-insecure-http` to `set` and
`publish`; `login` always switches to HTTPS, so put the credentials into `~/.netrc` instead
(`machine astrum.example.com login ci-deployer password …`). A Swift proxy caches another Swift registry;
its remote URL is the registry's base URL. Identifier lookups (`swift package resolve
--replace-scm-with-registry`) work for releases whose metadata lists `repositoryURLs`.

## Conan

```shell
conan remote add astrum https://astrum.example.com/repository/conan-group/
conan remote login astrum ci-deployer
conan upload "mylib/*" -r astrum-hosted -c
```

Uploads need the **Conan Bearer Token Realm** and go to a hosted repository added as its own remote.
A Conan proxy speaks either the Conan 2 (default) or the Conan 1 protocol; pick it in the proxy's
settings to match your clients.

## Composer

```shell
composer config repo.packagist composer https://astrum.example.com/repository/composer-proxy/
```

## CocoaPods

```shell
pod repo add-cdn astrum https://astrum.example.com/repository/cocoapods-proxy/
```

and in the `Podfile`: `source 'https://astrum.example.com/repository/cocoapods-proxy/'`.

## Conda

```shell
conda install -c https://astrum.example.com/repository/conda-proxy/main numpy
```

or add the channel to `.condarc`. The path after the repository name follows the remote's layout.

## Hugging Face

A Hugging Face proxy caches models, datasets and spaces of the Hugging Face Hub; its remote URL is
`https://huggingface.co/`. `huggingface_hub`, `transformers`, `datasets`, the `hf` command and every
other tool that honours `HF_ENDPOINT` download through it:

```shell
export HF_ENDPOINT=https://ci-reader:${ASTRUM_TOKEN}@astrum.example.com/repository/huggingface-proxy
export HF_HUB_DOWNLOAD_TIMEOUT=600
hf download openai-community/gpt2 config.json
python -c 'from huggingface_hub import snapshot_download; snapshot_download("openai-community/gpt2")'
```

The proxy fetches a file completely before it answers, so the first download of a large file can
take longer than the ten seconds clients wait by default; `HF_HUB_DOWNLOAD_TIMEOUT` raises that limit.
Hub clients only send Hugging Face tokens on their own, so the user name and password or access
token of this server go into `HF_ENDPOINT` as above. For gated and private repositories, put a
Hugging Face access token into the proxy's HTTP authentication as a bearer token. The proxy sends it
to the Hub only, never to the storage the Hub redirects large files to.

A branch or tag is resolved to its commit through the revision information, which is refreshed after
the metadata max age. Files are cached by commit and never refreshed, so a pinned commit is served
from the cache even while the Hub is unreachable, and a file that did not change between commits is
stored once. Search finds a commit by the repository id (the component name), the commit (the
version), `huggingface.revision` (a branch or tag that pointed at it) and `huggingface.file`.

Uploads, the Xet protocol (clients fall back to plain HTTP downloads) and searching the Hub
(`/api/models?search=`) are not supported.

## R

```r
options(repos = c(astrum = "https://astrum.example.com/repository/r-group/"))
install.packages("jsonlite")
```

## APT

`/etc/apt/sources.list.d/astrum.list` for a proxy of `http://deb.debian.org/debian`:

```text
deb https://astrum.example.com/repository/apt-debian/ trixie main
```

Uploading to a hosted repository (it signs its index with the repository's GPG key):

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -H 'Content-Type: multipart/form-data' \
  --data-binary @mypkg_1.0_amd64.deb https://astrum.example.com/repository/apt-hosted/
```

## Alpine

`/etc/apk/repositories` for a proxy of `https://dl-cdn.alpinelinux.org/alpine/`:

```text
https://astrum.example.com/repository/alpine-proxy/v3.22/main
https://astrum.example.com/repository/alpine-proxy/v3.22/community
```

## Git LFS

A Git LFS hosted repository stores the large files of git repositories; the git repository itself
stays on your git server. Point git-lfs at the repository's LFS API, `<repository URL>/info/lfs`, and
commit the setting so every clone uses it:

```shell
git config -f .lfsconfig lfs.url https://astrum.example.com/repository/lfs-hosted/info/lfs
git add .lfsconfig
git lfs track "*.psd"
git push
```

git-lfs asks for the user name and password of this server through the git credential helper; an
access token works as the password. Downloading needs read access, uploading and file locks
(`git lfs lock`, `git lfs locks`, `git lfs unlock`) need add access, and releasing another user's lock
with `--force` needs delete access. Objects are stored and searched by their SHA-256 and browsed under
`objects/<first two>/<next two>/<id>`.

## Yum (dnf)

`/etc/yum.repos.d/astrum.repo` for a hosted repository that generates its repodata at the root:

```ini
[astrum]
name=Astrum
baseurl=https://astrum.example.com/repository/yum-hosted/
enabled=1
gpgcheck=0
```

When the repository signs its metadata, add `repo_gpgcheck=1` and
`gpgkey=https://astrum.example.com/repository/yum-hosted/repodata/repomd.xml.key`. A repository
with a repodata depth of 2 keeps one repodata per directory pair, so the `baseurl` ends with them,
for example `.../yum-hosted/el9/x86_64/`. For a proxy of `https://download.rockylinux.org/pub/rocky/`
the `baseurl` follows the remote's layout: `.../rocky-proxy/9/BaseOS/x86_64/os/`.

Uploading to a hosted repository; it rebuilds the repodata a few seconds later:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file hello-2.12.1-3.el9.x86_64.rpm \
  https://astrum.example.com/repository/yum-hosted/Packages/hello-2.12.1-3.el9.x86_64.rpm
```

## Terraform

Give the Terraform group a registry host (Settings > Repositories > Terraform Settings), here
`astrum.example.com`; Astrum then answers Terraform's service discovery on that host. Terraform needs HTTPS.

```hcl
module "vpc" {
  source  = "astrum.example.com/acme/vpc/aws"
  version = "1.2.0"
}
```

`~/.terraformrc`, with a token from `curl -u <user> https://astrum.example.com/repository/terraform-group/token`
once the **Terraform Bearer Token Realm** is active:

```hcl
credentials "astrum.example.com" {
  token = "TerraformToken.<key>"
}

provider_installation {
  network_mirror {
    url = "https://astrum.example.com/repository/terraform-group/"
  }
}
```

The network mirror is optional; it installs every provider, also those of `registry.terraform.io`, through the
group. Publishing, signing and the proxy are described in
`public/common/components/formats/astrum-repository-terraform/README.md`.

## P2 (Eclipse)

Use `https://astrum.example.com/repository/p2-proxy/` as the update site in Eclipse or in a Tycho
target platform.

## Raw files

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file build/tool-1.4.0.tar.gz \
  https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
curl -fO https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
```
