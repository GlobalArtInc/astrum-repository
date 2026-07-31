# Connect build tools to repositories

Every repository is served at `https://<host>/repository/<repository name>/`. The examples use
`astrum.example.com` and repository names that follow a `<format>-hosted`, `<format>-proxy`,
`<format>-group` convention; use the names you created. Point clients at a group when you read and at
a hosted repository when you publish.

Clients authenticate with the user name and password of a user, with an
[access token](../configure/authentication.md#access-tokens) in place of the password (the way for CI
and for users who sign in through OpenID Connect), or with a token of their format where it has one.
See [users, realms and sign-in](../configure/authentication.md) for the realms that tokens need.
The formats this server supports are Alpine, Ansible Galaxy, APT, Cargo, CocoaPods, Composer, Conan, Conda, Docker and other OCI
artifacts, Git LFS, Go, Helm, Hugging Face, Maven, npm, NuGet, P2, pub, PyPI, R, RubyGems, Swift, Terraform,
Yum and raw files.

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

`npm login` stores a token in `~/.npmrc`. For CI write a token directly, an npm token or an
[access token](../configure/authentication.md#access-tokens):

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

Choose **Use path routing** as the routing mode of the Docker repositories (see
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

What is attached to a manifest lives and dies with it. Deleting a manifest by its digest through the
registry API deletes the manifests whose `subject` it is, recursively, the tags that point at it and the
tags of the tag schema derived from its digest (`sha256-<hex>`, `.sig`, `.att`, `.sbom`). Deleting a tag
deletes only the tag. Layers are not removed with the manifests; the task **Docker - Delete unused
manifests and images** deletes the layers that no manifest refers to, and untagged manifests, once they
are older than its deploy offset (24 hours by default).

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

## RubyGems

Activate the **RubyGems API Key Realm** to publish. `gem signin` asks for the user name and password
and stores the user's API key for the repository:

```shell
gem signin --host https://astrum.example.com/repository/rubygems-hosted/
gem push mylib-1.0.0.gem --host https://astrum.example.com/repository/rubygems-hosted/
gem yank mylib -v 1.0.0 --host https://astrum.example.com/repository/rubygems-hosted/
```

To install, use a group (or a proxy of rubygems.org) as the source, with credentials in the URL or in
Bundler's configuration (keyed by host name):

```shell
gem install mylib --clear-sources --source https://astrum.example.com/repository/rubygems-group/
bundle config set --global astrum.example.com "ci-reader:$ASTRUM_PASSWORD"
```

and in the `Gemfile`: `source "https://astrum.example.com/repository/rubygems-group/"`. To use the
repository as a mirror of rubygems.org run
`bundle config set --global mirror.https://rubygems.org https://astrum.example.com/repository/rubygems-group/`.

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
group.

A hosted repository takes module packages at `v1/modules/<namespace>/<name>/<system>/<version>/<file>`
and the release files of a provider at `v1/providers/<namespace>/<type>/<version>/<file>`, one `PUT`
per file:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file vpc-aws-1.2.0.tar.gz \
  https://astrum.example.com/repository/terraform-hosted/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file terraform-provider-demo_2.0.0_linux_amd64.zip \
  https://astrum.example.com/repository/terraform-hosted/v1/providers/acme/demo/2.0.0/terraform-provider-demo_2.0.0_linux_amd64.zip
```

When an upload brings no `SHA256SUMS` file for a provider, the server writes it and signs it with the
signing key of the repository.

## Alpine

`apk` reads `<repository URL>/<architecture>/APKINDEX.tar.gz`, so a repository line names the directory
above the architecture folders, for example `v3.20/main`. When the repository has a signing key, save its
public key under the name it is served with, `<repository name>.rsa.pub`, before the first update:

```shell
wget -O /etc/apk/keys/alpine-hosted.rsa.pub \
  https://astrum.example.com/repository/alpine-hosted/alpine-hosted.rsa.pub
echo "https://astrum.example.com/repository/alpine-hosted/v3.20/main" >> /etc/apk/repositories
apk update
apk add tool
```

Repositories without a key serve unsigned indexes, which `apk` reads only with `--allow-untrusted`. A
proxy with a key adds its signature in front of the remote's, so the official Alpine keys keep working;
a group signs its merged indexes with its own key, and without one passes the index of a single member
through as that member serves it and leaves a merge of several members unsigned.

Upload a package to a hosted repository under the name `apk` asks for, `<name>-<version>.apk`, in the
directory of its architecture (`noarch` packages take the architecture of the directory); the
repository then writes and signs the `APKINDEX.tar.gz` of that directory:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool-1.4.0-r0.apk \
  https://astrum.example.com/repository/alpine-hosted/v3.20/main/x86_64/tool-1.4.0-r0.apk
```

The signing key is a PEM-encoded RSA private key, for example from `openssl genrsa -out alpine.key 4096`,
optionally encrypted with a passphrase.

## Ansible Galaxy

Repositories serve Ansible collections through the Galaxy v3 API that `ansible-galaxy` speaks: hosted
repositories take `ansible-galaxy collection publish`, proxies cache another Galaxy server such as
`https://galaxy.ansible.com`, and groups merge the versions of their members. Name the repository in
`ansible.cfg` next to your playbooks or in `~/.ansible.cfg`; the token is an
[access token](../configure/authentication.md#access-tokens) of this server (`username` and `password`
work too) and can be left out where anonymous users may read the repository:

```ini
[galaxy]
server_list = astrum

[galaxy_server.astrum]
url = https://astrum.example.com/repository/ansible-group/
token = <access token>
```

```shell
ansible-galaxy collection install community.general
ansible-galaxy collection install acme.tools:1.4.0 -s https://astrum.example.com/repository/ansible-group/
```

`requirements.yml` can name the repository of each collection:

```yaml
collections:
  - name: acme.tools
    version: ">=1.4.0"
    source: https://astrum.example.com/repository/ansible-group/
```

Publish an archive built with `ansible-galaxy collection build` to a hosted repository:

```shell
ansible-galaxy collection publish acme-tools-1.4.0.tar.gz \
  -s https://astrum.example.com/repository/ansible-hosted/ --token "$ASTRUM_TOKEN"
```

The archive must be named `<namespace>-<name>-<version>.tar.gz`, and its `MANIFEST.json` must name the same
collection with a semantic version; otherwise the import task that `ansible-galaxy` waits for fails and the
client prints the reason. The upload page and the components API take the same archives. The collection
list of a proxy (`api/v3/collections/`) shows the collections it has cached, not the whole remote. Roles
of the older Galaxy API (`ansible-galaxy role install`) are not served.

## APT

`/etc/apt/sources.list.d/astrum.list` for a repository whose distribution is `bookworm`, after importing
the signing key of the repository (its public key is set on the repository's APT signing settings):

```text
deb https://astrum.example.com/repository/apt-hosted/ bookworm main
```

Upload a package to a hosted repository:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -H "Content-Type: multipart/form-data" \
  --data-binary "@tool_1.4.0_amd64.deb" https://astrum.example.com/repository/apt-hosted/
```

## Cargo

Repositories speak the sparse index protocol. A hosted registry in `.cargo/config.toml`:

```toml
[registries.cargo-hosted]
index = "sparse+https://astrum.example.com/repository/cargo-hosted/"
```

Publish with an [access token](../configure/authentication.md#access-tokens) of this server:

```shell
cargo login --registry cargo-hosted
cargo publish --registry cargo-hosted
```

A proxy of crates.io replaces the default source:

```toml
[source.crates-io]
replace-with = "cargo-proxy"

[source.cargo-proxy]
registry = "sparse+https://astrum.example.com/repository/cargo-proxy/"
```

## CocoaPods

CocoaPods repositories are proxies of a CocoaPods CDN source, such as trunk at `https://cdn.cocoapods.org/`.
Name the repository as the source of the Podfile:

```ruby
source 'https://astrum.example.com/repository/cocoapods-proxy/'

target 'App' do
  pod 'Alamofire', '~> 5.9'
end
```

or add it once to the local spec repositories, for `pod spec cat` and `pod repo update`:

```shell
pod repo add-cdn astrum https://astrum.example.com/repository/cocoapods-proxy/
```

CocoaPods signs in only through `~/.netrc`, which `pod` and the `curl` it downloads archives with both read:

```text
machine astrum.example.com
login ci-reader
password <password or access token>
```

A podspec whose source is an HTTP(S) archive is served with that address pointed at
`pods/<pod>/<version>/<file>` of the repository, so the archive is downloaded and cached through it; pods
with a Git source are still cloned from their Git server. Because the served podspecs differ from those of
trunk, the `SPEC CHECKSUMS` in `Podfile.lock` change when a project moves between trunk and the proxy.

## Composer

```shell
composer config repositories.composer-proxy composer https://astrum.example.com/repository/composer-proxy/
composer require monolog/monolog
```

Upload a package archive to a hosted repository:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file package.zip \
  https://astrum.example.com/repository/composer-hosted/packages/upload/acme/tool/1.4.0
```

## Conan

The protocol version (Conan 1 or Conan 2) is chosen when the repository is created.

```shell
conan remote add conan-hosted https://astrum.example.com/repository/conan-hosted/
conan install --requires=zlib/1.3.1 -r conan-hosted
conan upload tool/1.4.0 -r conan-hosted
```

## Conda

A repository is a conda channel, and a hosted repository can hold several channels below it: packages
uploaded to `main/noarch/` belong to the channel `.../conda-hosted/main`. The server writes
`repodata.json` (with `.bz2` and `.zst` copies) and `current_repodata.json` for every platform
subdirectory and `channeldata.json` for every channel. A proxy points at a channel, for example
`https://conda.anaconda.org/conda-forge/`, and a group merges the `repodata.json` of its members.

```shell
conda install -c https://astrum.example.com/repository/conda-group --override-channels numpy
micromamba create -p ./env -c https://astrum.example.com/repository/conda-hosted/main --override-channels tool
```

In `~/.condarc`, to use the repository instead of the default channels:

```yaml
channels:
  - https://astrum.example.com/repository/conda-group
```

Upload a `.conda` or `.tar.bz2` package to its channel and subdirectory; the name, version, build and
subdirectory must match the package's `info/index.json`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool-1.4.0-h1_0.conda \
  https://astrum.example.com/repository/conda-hosted/main/linux-64/tool-1.4.0-h1_0.conda
```

Clients sign in with the user name and password in the channel address
(`https://user:password@host/...`) or through `~/.netrc`. Sharded repodata and JLAP patches are not
served, so clients read the whole `repodata.json`.

## Go

A proxy of the public module proxy, set once:

```shell
go env -w GOPROXY=https://astrum.example.com/repository/go-proxy,direct
```

Modules of a hosted repository are not in the public checksum database:

```shell
go env -w GOPROXY=https://astrum.example.com/repository/go-hosted,direct
go env -w GONOSUMDB=example.com/acme/tool
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file v1.4.0.zip \
  https://astrum.example.com/repository/go-hosted/example.com/acme/tool/@v/v1.4.0.zip
```

Upper-case letters in module paths and versions are written as `!` and the lower-case letter, as the
module proxy protocol requires.

## R

Repositories have the layout of CRAN: source packages in `src/contrib/` (older versions in
`src/contrib/Archive/<package>/`), Windows binaries in `bin/windows/contrib/<R version>/` and macOS
binaries in `bin/macosx/[<platform>/]contrib/<R version>/`. A group of a hosted repository and a proxy of
CRAN in `~/.Rprofile`:

```r
options(repos = c(CRAN = "https://astrum.example.com/repository/r-group"))
install.packages("data.table")
```

Without anonymous access, put a user and an [access token](../configure/authentication.md#access-tokens)
into the address: `https://ci-reader:<token>@astrum.example.com/repository/r-group`.

Upload a package to a hosted repository at its path; the file name is `<package>_<version>` with
`.tar.gz` for source, `.zip` for Windows and `.tgz` for macOS packages:

```shell
R CMD build tool
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file tool_1.4.0.tar.gz \
  https://astrum.example.com/repository/r-hosted/src/contrib/tool_1.4.0.tar.gz
```

Hosted repositories write `PACKAGES`, `PACKAGES.gz` and `PACKAGES.rds` of each directory themselves
after every upload and delete; they list the latest version of each package, and older versions stay
downloadable at their paths, for example with
`install.packages("https://astrum.example.com/repository/r-hosted/src/contrib/Archive/tool/tool_1.3.0.tar.gz", repos = NULL)`.
They do not write `src/contrib/Meta/archive.rds`, so `remotes::install_version` finds only the versions
listed in `PACKAGES`. The task "Repair - Rebuild R repository metadata (PACKAGES)" writes the files again.

## P2 (Eclipse)

P2 repositories are proxies of Eclipse update sites such as `https://download.eclipse.org/releases/2024-03/`.
In Eclipse, choose Help > Install New Software > Add and enter the repository as the location:

```text
https://astrum.example.com/repository/p2-proxy/
```

A Tycho build names it in `pom.xml`:

```xml
<repositories>
  <repository>
    <id>p2-proxy</id>
    <layout>p2</layout>
    <url>https://astrum.example.com/repository/p2-proxy/</url>
  </repository>
</repositories>
```

The p2 director installs from it into the Eclipse that runs it (add `-destination` for another one):

```shell
eclipse -nosplash -application org.eclipse.equinox.p2.director \
  -repository https://astrum.example.com/repository/p2-proxy/ \
  -installIU org.eclipse.jdt.feature.group
```

Child repositories of a composite that lie outside the remote of the proxy, on the same host or on
another one, are rewritten to `.remote/<scheme>/<host>/...` under the repository and fetched through the
proxy, so clients without internet access get them too; only locations that a composite of the remote
names are fetched there. The proxy drops the mirror and download statistics properties of artifact
repositories, so clients download every artifact through it. Eclipse replaces `http://` with `https://`
in repository addresses: serve the repository over HTTPS, or start Eclipse with `-Dp2.httpRule=allow`.

## pub (Dart and Flutter)

The hosted URL of a pub repository is its address without the trailing slash, for example
`https://astrum.example.com/repository/pub-group`. `dart pub` and `flutter pub` send a token as
`Authorization: Bearer`; use an [access token](../configure/authentication.md#access-tokens) of your user
and add it once per repository (the command asks for the token, `--env-var ASTRUM_TOKEN` reads it from
the environment in CI):

```shell
dart pub token add https://astrum.example.com/repository/pub-group
```

A repository that anonymous users may not read answers 401 to a client without a token, and the client
prints where to create one. `dart pub` sends tokens only over HTTPS, except to `localhost`.

A dependency from the repository in `pubspec.yaml`:

```yaml
dependencies:
  mylib:
    hosted: https://astrum.example.com/repository/pub-group
    version: ^1.2.0
```

`dart pub add 'mylib:{hosted: https://astrum.example.com/repository/pub-group, version: ^1.2.0}'` writes
the same lines. To take every package through a group of a hosted repository and a proxy of
`https://pub.dev`, set `PUB_HOSTED_URL=https://astrum.example.com/repository/pub-group` for `dart pub` and
`flutter pub`.

To publish, name the hosted repository in the `pubspec.yaml` of the package and run `dart pub publish`
(`--force` skips the question):

```yaml
publish_to: https://astrum.example.com/repository/pub-hosted
```

The repository refuses an archive without `pubspec.yaml` at its top, a name that is not a lower-case
Dart identifier and, unless the write policy allows redeploying, a version that it already has with
other content; `dart pub publish` shows the reason. Archives also go up through the components API in
the field `pub.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F pub.asset=@mylib-1.2.0.tar.gz \
  "https://astrum.example.com/service/rest/v1/components?repository=pub-hosted"
```

Retract a version or mark a package discontinued with the option paths of pub.dev; clients then skip
the retracted version and report the discontinued package:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -X PUT -H 'Content-Type: application/json' \
  -d '{"isRetracted": true}' \
  https://astrum.example.com/repository/pub-hosted/api/packages/mylib/versions/1.2.0/options
curl -u "ci-deployer:$ASTRUM_PASSWORD" -X PUT -H 'Content-Type: application/json' \
  -d '{"isDiscontinued": true, "replacedBy": "mylib2"}' \
  https://astrum.example.com/repository/pub-hosted/api/packages/mylib/options
```

Proxies keep the version listings, security advisories and package name completion data of the remote
as metadata, point `archive_url` at themselves and fetch each archive from the address the remote
listed, checking it against `archive_sha256`. Groups merge the listings of their members (the first
member wins for a version) and their advisories; publish to the hosted repository itself.

## Raw files

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file build/tool-1.4.0.tar.gz \
  https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
curl -fO https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
```
