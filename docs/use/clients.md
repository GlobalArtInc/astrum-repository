# Connect build tools to repositories

Every repository is served at `https://<host>/repository/<repository name>/`. The examples use
`astrum.example.com` and repository names that follow a `<format>-hosted`, `<format>-proxy`,
`<format>-group` convention; use the names you created. Point clients at a group when you read and at
a hosted repository when you publish.

Clients authenticate with the user name and password of a user, with an
[access token](../configure/authentication.md#access-tokens) in place of the password (the way for CI
and for users who sign in through OpenID Connect), or with a token of their format where it has one.
See [users, realms and sign-in](../configure/authentication.md) for the realms that tokens need.
The formats this server supports are Alpine, Ansible Galaxy, APT, Cargo, Chef Supermarket, CocoaPods, Composer,
Conan, Conda, Docker and other OCI artifacts, Git LFS, Go, Helm, Hex, Hugging Face, Maven, Nix, npm, NuGet,
Open VSX, P2, pacman, pub, Puppet Forge, PyPI, R, RubyGems, Swift, Terraform, Vagrant, Yum and raw files,
and remote caches for Nx and Turborepo.

That address belongs to the clients and has no page of its own: a browser opening it gets a page that
links to the HTML index of the repository at `https://<host>/service/rest/repository/browse/<repository
name>/` and to the repository in the web interface, provided the user may browse it. The HTML index lists
folders and files with links to download them; the summary of the repository in Settings links to it too.

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

## Chef Supermarket

Repositories serve Chef Infra cookbooks through the Supermarket API: `universe` for Berkshelf and
Policyfiles, and `api/v1/cookbooks`, `api/v1/cookbooks/<name>`, `api/v1/cookbooks/<name>/versions/<version>`
(`latest`, or the version with dots or underscores), its `download` and `api/v1/search?q=` for
`knife supermarket`. Hosted repositories keep the cookbooks you publish; proxies cache another Supermarket
such as `https://supermarket.chef.io`, keeping its universe and documents as metadata (fetched again after
the metadata max age) and the archives as content; groups merge the universes, cookbook documents and
search results of their members, and the first member that has a version serves its archive. Every address
in the answers (`location_path`, `download_url`, `latest_version`, `file`) points back at the repository
that was asked.

Berkshelf, Policyfiles and the reading `knife supermarket` commands (`download`, `install`, `list`,
`search`, `show`) send no credentials, so let the anonymous user read the repositories they use (see
[anonymous access](../configure/authentication.md#anonymous-access)). A `Berksfile`:

```ruby
source "https://astrum.example.com/repository/chef-group"

cookbook "acme", "~> 1.2"
```

A `Policyfile.rb`:

```ruby
name "web"
default_source :supermarket, "https://astrum.example.com/repository/chef-group"
run_list "acme::default"
cookbook "acme", "~> 1.2"
```

`knife` takes the repository as its Supermarket site:

```shell
knife supermarket download acme 1.2.0 -m https://astrum.example.com/repository/chef-group
knife supermarket search acme -m https://astrum.example.com/repository/chef-group
```

`knife supermarket share` signs its request with the Chef client key of the user (the `X-Ops-*` headers),
which this server cannot check, so it is answered `401` with a hint. Publish the same multipart form with
`curl` and a user or [access token](../configure/authentication.md#access-tokens) with write access to a
hosted repository: a `tarball` part with the cookbook archive and an optional `cookbook` part with JSON
that names the category (`Other` when it is left out, or the category of the versions before):

```shell
knife cookbook metadata from file acme/metadata.rb
tar -czf acme.tgz acme
curl -u "ci-deployer:$ASTRUM_TOKEN" -F 'cookbook={"category":"Web Servers"}' -F tarball=@acme.tgz \
  https://astrum.example.com/repository/chef-hosted/api/v1/cookbooks
```

The archive is a gzip-compressed tar with the cookbook in a folder named after it. The server reads the
name, the version (`1.2` or `1.2.3`), the dependencies, the platforms and the description from
`<name>/metadata.json`, which `knife cookbook metadata` writes from `metadata.rb`; an archive with only
`metadata.rb` is refused with that hint. The answer is `201 Created` with the cookbook document; errors
carry `error_messages` and `error_code` as on the Supermarket, and a version that exists is refused with
`409` where the write policy does not allow redeploying. The upload page and the components API take the
same archives. `DELETE api/v1/cookbooks/<name>/versions/<version>` removes a version and
`DELETE api/v1/cookbooks/<name>` the whole cookbook, with the same credentials; the universe follows at
once, also when components are deleted in the web interface or by a cleanup policy.

Search and the cookbook list of a proxy are passed on to its remote. Tools, users, followers,
collaborators and ratings of the Supermarket are not served, and hosted cookbooks are never deprecated.

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

## Hex (Elixir and Erlang)

Repositories serve the Hex registry (`names`, `versions` and `packages/<name>`), package tarballs
(`tarballs/<name>-<version>.tar`), documentation (`docs/<name>-<version>.tar.gz`) and the public key of
the registry (`public_key`) to Mix and rebar3. Every registry file is a gzipped, signed protobuf message;
clients check the signature with the public key of the repository and check that the file names the
repository they asked.

Hosted and group repositories sign with a key of their own. The server makes an RSA key the first time
the repository needs one, when its registry or public key is first read or a package is first
published, and keeps it in the configuration of the
repository, sealed with the secrets key of the server; it is never shown, the repository document only
says `"hexSigning": {}`. To keep the key of a repository that moves from another server, give its
unencrypted PEM private key as `hexSigning.keypair` when you create or update the repository, or in the
**Registry Signing** section of the form; a new key needs the new public key on every client. A group
merges the registries of its members (the first member wins for a release) and signs the result with
the group's key, so clients of a group use the public key of the group, not those of its members. A group
checks the registry of a proxy member against the public key that proxy serves and leaves out what does
not match.

Add a hosted repository or a group to Mix under its own name, which is part of what is signed:

```shell
curl -fsS -o hex-group.pem https://astrum.example.com/repository/hex-group/public_key
mix hex.repo add hex-group https://astrum.example.com/repository/hex-group --public-key hex-group.pem
```

`--fetch-public-key <fingerprint>` instead of `--public-key` downloads the key and compares its
fingerprint; the page at the address of the repository shows the fingerprint (`SHA256:...`, the form
`ssh-keygen -l` prints) to users who may read the repository. Where anonymous users may not read it, add
`--auth-key <access token>`: Mix sends the [access token](../configure/authentication.md#access-tokens)
as `Authorization` with every request to the repository. Name the repository in the dependencies that
come from it:

```elixir
defp deps do
  [
    {:acme_tools, "~> 1.4", repo: "hex-group"}
  ]
end
```

Dependencies of a package keep the repository its metadata names: those from hex.pm
(`repository: "hexpm"`) come from hex.pm, or from the mirror Mix uses for it. rebar3 takes the repository
in `rebar.config` or `~/.config/rebar3/rebar.config` with the contents of the public key, and
`repo_key => <<"<access token>">>` where the repository needs a sign-in:

```erlang
{hex, [{repos, [#{name => <<"hex-group">>,
                  repo_url => <<"https://astrum.example.com/repository/hex-group">>,
                  repo_public_key => <<"-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----\n">>}]}]}.
```

A proxy of `https://repo.hex.pm` passes the signed registry through byte for byte, so it still names the
repository `hexpm` and verifies with the key of hex.pm that Mix and rebar3 ship with. Use a proxy, not a
group, as the mirror of hex.pm (`mix hex.config mirror_url <url>` keeps it for Mix):

```shell
export HEX_MIRROR=https://astrum.example.com/repository/hex-proxy   # Mix
export HEX_CDN=https://astrum.example.com/repository/hex-proxy      # rebar3
```

The registry files are metadata and are fetched again after the metadata age of the proxy; tarballs never
change and stay cached. Registry files hold no addresses, so a proxy has nothing to rewrite; it also
passes `docs/` and `installs/` (what `mix local.hex` fetches from a mirror) through.

Publish to a hosted repository through the Hex API under `<repository URL>/api`. The API key is an access
token of a user who may add to the repository (and edit, for `--replace`); Mix sends it as
`Authorization: <token>`:

```shell
HEX_API_URL=https://astrum.example.com/repository/hex-hosted/api \
HEX_API_KEY="$ASTRUM_TOKEN" mix hex.publish --yes
```

`mix hex.publish` uploads the package and then its documentation, which the repository serves at
`docs/<name>-<version>.tar.gz`; `mix hex.publish package` uploads only the package. A version that is
already there is refused unless you publish with `--replace`, and then only where the write policy allows
redeploying. The repository also refuses tarballs that are not of version 3, whose `CHECKSUM` does not
match, or whose `metadata.config` names no valid package name (lower-case letters, digits and `_`) or
semantic version. `mix hex.publish --revert <version>` deletes a release, `mix hex.retire <package>
<version> <reason> --message <text>` marks it retired in the registry and `mix hex.retire <package>
<version> --unretire` takes that back; `mix hex.info <package>` and `mix hex.user whoami` read the same
API. The repository keeps no package owners: who may publish follows the privileges of the repository.
rebar3_hex publishes the same way when the repository entry also has
`api_url => <<"https://astrum.example.com/repository/hex-hosted/api">>` and the access token as its
`api_key`.

Tarballs also go up through the components API in the field `hex.asset`, or with a `PUT` to their
place:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file acme_tools-1.4.0.tar \
  https://astrum.example.com/repository/hex-hosted/tarballs/acme_tools-1.4.0.tar
```

The registry of a hosted repository is written again whenever a release is published, retired or deleted,
also when it is deleted from the web interface or by a cleanup policy; the task **Hex - Rebuild Hex
registries** writes it anew, and with **Reset proxy and group registries** drops the cached registry
files of proxies and the merged ones of groups.

## Nix

Nix repositories are binary caches: Nix substitutes store paths from them, and `nix copy` uploads store
paths to hosted ones. A proxy of `https://cache.nixos.org` and a group of it and a hosted repository give
clients both in one substituter. Nix reads the cache with `GET` of `nix-cache-info`, `<hash>.narinfo` and
the NAR file the narinfo names (`nar/<hash>.nar.xz` and the like), and optionally `<hash>.ls`,
`log/<derivation>` and `realisations/<id>.doi`. The address of the repository without the trailing slash
is the store URL. In `/etc/nix/nix.conf` (restart `nix-daemon` on a multi-user installation):

```text
substituters = https://astrum.example.com/repository/nix-group
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= astrum-1:<public key>
netrc-file = /etc/nix/netrc
```

Nix sends the credentials of the netrc file with every request to the host, for reading and uploading; a
repository that anonymous users may read needs none. Use an
[access token](../configure/authentication.md#access-tokens) as the password:

```text
machine astrum.example.com
login ci-reader
password <access token>
```

The server does not sign store paths: Nix accepts a substituted path only with a signature by a key in
`trusted-public-keys`, so list the key of cache.nixos.org for the proxy and your own key for what you
upload. `nix-cache-info` comes from the Nix settings of the repository (the `nix` section over REST):
`storeDir` (`/nix/store`), `priority` (40, the priority of cache.nixos.org; Nix asks lower numbers first)
and `wantMassQuery` (on). A group answers with its own settings, not those of its members.

To upload, make a signing key once, give its public key to the clients and let `nix copy` sign while it
uploads to a hosted repository (the user needs the add privilege, and edit to replace files when the write
policy allows redeploying):

```shell
nix key generate-secret --key-name astrum-1 > /etc/nix/astrum-1.secret
nix key convert-secret-to-public < /etc/nix/astrum-1.secret
nix copy --to 'https://astrum.example.com/repository/nix-hosted?secret-key=/etc/nix/astrum-1.secret' ./result
```

`nix store sign --key-file /etc/nix/astrum-1.secret --recursive ./result` signs before a plain
`nix copy --to https://astrum.example.com/repository/nix-hosted ./result`; add `&write-nar-listing=true`
to upload the `.ls` listings, and `nix store copy-log --to <store URL> ./result` uploads build logs.
`nix copy` asks for the narinfo and the NAR file with `HEAD` and uploads what is missing with `PUT`, the
NAR file before its narinfo. The repository refuses a narinfo that is compressed (leave `narinfo-compression` unset), that names
a store path outside its store directory or under another hash than its file name, whose `URL` is not a
NAR file uploaded to the repository before, whose `FileSize` or `FileHash` differ from that file, or whose
hashes, references, deriver, signatures or content address are malformed; `nix copy` prints the reason.
Listings and logs keep the `Content-Encoding` they were uploaded with. With the write policy "Allow
once" a second narinfo for a store path is refused, while the same NAR file again is accepted.

Every store path is a component: its group is the hash part, its name and version are split from the name
of the store path the way `nix-env` does (`hello` and `2.12.1` for `…-hello-2.12.1`), and its assets are the
narinfo, the listing and the NAR file. Two store paths with identical contents share one NAR file, which
then belongs to neither component. Deleting a component, or `DELETE` of `<hash>.narinfo`, removes the
narinfo, its listing and the NAR file unless another narinfo still uses it; a narinfo whose NAR file is
gone is no longer served. The task "Nix - Delete NAR files no narinfo uses" removes the NAR files of
uploads that never got their narinfo and of shared files whose store paths were all deleted, once they
are older than its deploy offset.

Proxies keep narinfo files as metadata for the maximum metadata age (the form starts proxies at 60
minutes) and remember misses in the negative cache (60 minutes in the form; Nix itself remembers misses
for an hour), since most narinfo lookups of a client are misses. NAR files never change and are not
checked again. A proxy refuses a narinfo of the remote that names another store path or another store
directory with 502, rewrites a `URL` that points under the remote to a relative one so that clients
download through the proxy, and leaves `URL`s on other hosts alone. Listings and logs keep the
`Content-Encoding` of the remote (cache.nixos.org compresses them). Groups answer narinfo, NAR and other
files from the first member that has them; upload to the hosted repository itself.

## Open VSX (VS Code extensions)

Repositories serve VS Code extensions (`.vsix` packages) to editors built on VS Code that read an
Open VSX or Marketplace-compatible extension gallery, such as VSCodium, code-server, Theia, Gitpod and
OpenVSCode Server, and to the `ovsx` command line client. Hosted repositories take `ovsx publish`, proxies
cache another Open VSX registry such as `https://open-vsx.org` (the registry root is the remote URL), and
groups merge their members, the first member winning for each version and target platform.

Point an editor at a repository, usually a group of a hosted repository and a proxy, in the
`extensionsGallery` section of its `product.json`. VSCodium also reads a `product.json` in its user data
folder (`~/.config/VSCodium/product.json` on Linux, `~/Library/Application Support/VSCodium/product.json`
on macOS, `%APPDATA%\VSCodium\product.json` on Windows); restart the editor after changing it:

```json
{
  "extensionsGallery": {
    "serviceUrl": "https://astrum.example.com/repository/openvsx-group/vscode/gallery",
    "itemUrl": "https://astrum.example.com/repository/openvsx-group/vscode/item",
    "resourceUrlTemplate": "https://astrum.example.com/repository/openvsx-group/vscode/unpkg/{publisher}/{name}/{version}/{path}",
    "extensionUrlTemplate": "https://astrum.example.com/repository/openvsx-group/vscode/gallery/{publisher}/{name}/latest"
  }
}
```

Editors that take the gallery from the environment read the same addresses from variables:

```shell
export VSCODE_GALLERY_SERVICE_URL=https://astrum.example.com/repository/openvsx-group/vscode/gallery
export VSCODE_GALLERY_ITEM_URL=https://astrum.example.com/repository/openvsx-group/vscode/item
export VSCODE_GALLERY_EXTENSION_URL_TEMPLATE='https://astrum.example.com/repository/openvsx-group/vscode/gallery/{publisher}/{name}/latest'
export VSCODE_GALLERY_RESOURCE_URL_TEMPLATE='https://astrum.example.com/repository/openvsx-group/vscode/unpkg/{publisher}/{name}/{version}/{path}'
```

Editors send no credentials to their gallery, so the repository they use must be readable anonymously:
keep anonymous access on and narrow `as-anonymous` to that repository, or let only your network reach it.
The repositories answer cross-origin requests, so editors that run in a browser can use them too.

`ovsx` takes the repository address as its registry URL:

```shell
npx ovsx search yaml -r https://astrum.example.com/repository/openvsx-group
npx ovsx get redhat.vscode-yaml -t linux-x64 -r https://astrum.example.com/repository/openvsx-group
```

Publish to a hosted repository. The personal access token of `ovsx` (`-p` or `OVSX_PAT`) is an
[access token](../configure/authentication.md#access-tokens) of this server, or `<user>:<password>`:

```shell
npx ovsx publish acme.hello-1.2.0.vsix -r https://astrum.example.com/repository/openvsx-hosted -p "$ASTRUM_TOKEN"
npx ovsx publish -i acme.hello-1.2.0@linux-x64.vsix -r https://astrum.example.com/repository/openvsx-hosted -p "$ASTRUM_TOKEN"
```

A package must hold `extension.vsixmanifest` and `extension/package.json`, both naming the same publisher,
name and semantic version; the publisher is the namespace of the extension. Builds for target platforms
(`vsce package --target linux-x64`, `darwin-arm64`, `web` and the others) are kept side by side as one
version, and a platform without a build of its own gets the universal one. Pre-release builds
(`vsce package --pre-release`) are listed as `pre-release` and become `latest` only while there is no
release. Unless the write policy allows redeploying, a version and platform that is already there with
other content is refused, and `ovsx publish --skip-duplicate` passes over it. Namespaces need no
creation: `ovsx create-namespace` succeeds for users who may publish, and `ovsx verify-pat` tells whether
the token may publish to the repository. `ovsx unpublish` deletes versions from a hosted repository for
users with the delete privilege. The upload page and the components API take `.vsix` files in the field
`openvsx.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F openvsx.asset=@acme.hello-1.2.0.vsix \
  "https://astrum.example.com/service/rest/v1/components?repository=openvsx-hosted"
```

The repositories serve the registry API under `api/` (namespaces, extension metadata per version and
target platform, `versions`, `version-references`, the files of a version under `.../file/`, `api/-/search`,
`api/-/query`, `api/v2/-/query` and `api/-/publish`) and the gallery API under `vscode/` (`gallery/extensionquery`,
`gallery/{publisher}/{name}/latest`, `gallery/publishers/.../vspackage`, `asset/...`, `unpkg/...` and
`item`). Hosted repositories read the files of a version from inside its package and keep no signatures.
Proxies keep the answers of the remote as metadata with every address pointed at the proxy, fetch each
package and file on first use and check packages against the SHA-256 file of the remote.

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

## pacman (Arch Linux)

`pacman` downloads `<Server>/<section name>.db` (with `-Fy` also `<section name>.files`), then the
packages that database lists and their `.sig` files from the same `Server`. In `/etc/pacman.conf` it puts
the section name in place of `$repo` and the architecture in place of `$arch`.

A hosted repository keeps its packages in folders named after their architecture, `x86_64/`,
`aarch64/` and so on, and `any/` for `arch=any` packages; the folders may sit below others, for example
`custom/os/x86_64/`. For every architecture folder it generates the databases under whatever name pacman
asks for (`<name>.db`, `<name>.files` and the same with `.tar.gz`), so the section can have any name. The
databases of a folder also list the packages of the `any/` folder next to it, and those packages are served
from every architecture folder. A database lists the newest version of each package in pacman's version
order; older package files stay downloadable.

```ini
[astrum]
SigLevel = Optional TrustAll
Server = https://astrum.example.com/repository/pacman-hosted/$arch
```

```shell
sudo pacman -Sy astrum-tool
```

pacman sends no credentials of its own. Where anonymous users may not read the repository, put a user and
an [access token](../configure/authentication.md#access-tokens) into the address,
`Server = https://<user>:<token>@astrum.example.com/repository/pacman-hosted/$arch`; pacman passes it on
as Basic authentication.

With a signing key (Settings > Repositories > Database Signing, an ASCII-armored OpenPGP private key, for
example from `gpg --export-secret-key --armor`, optionally with a passphrase) the repository signs every
database it serves and answers `<database>.sig`; the public key is served as `<repository name>.key`, and
the page at the repository address shows its fingerprint. Add it to the keyring of pacman once, then
require signed databases:

```shell
curl -O https://astrum.example.com/repository/pacman-hosted/pacman-hosted.key
sudo pacman-key --add pacman-hosted.key
sudo pacman-key --lsign-key <fingerprint>
```

```ini
[astrum]
SigLevel = Required DatabaseRequired
Server = https://astrum.example.com/repository/pacman-hosted/$arch
```

A package signature uploaded next to its package (`<package file>.sig`) is served as it is and written
into the database as `%PGPSIG%`; `SigLevel = Required` then needs the key of the packager in the keyring.

A proxy of an Arch Linux mirror, for example `https://geo.mirror.pkgbuild.com/`, keeps the layout of the
mirror, so name the repositories of the mirror as sections:

```ini
[core]
Server = https://astrum.example.com/repository/pacman-proxy/$repo/os/$arch

[extra]
Server = https://astrum.example.com/repository/pacman-proxy/$repo/os/$arch
```

The proxy fetches databases again after the metadata age and keeps packages and their signatures. The
official databases are unsigned, the official package signatures pass through; a proxy with a signing key
signs the databases it serves instead of passing the signatures of the remote through.

A group asks its members for the same path and merges the databases that more than one member has (the
first member wins for a package name), so a hosted repository can override packages of a proxy by storing
them in the same folders, for example `core/os/x86_64/`. A group signs the databases it serves with its own
key; without one it passes the database and the signature of a single member through and leaves a merge of
several members unsigned.

Upload a package to a hosted repository under the name its `.PKGINFO` gives,
`<pkgname>-<pkgver>-<arch>.pkg.tar.<zst|xz|gz|bz2>` (or `.pkg.tar`), into a folder of its architecture,
and its signature next to it; the repository rewrites the databases of that folder right away:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file astrum-tool-1.4.0-1-x86_64.pkg.tar.zst \
  https://astrum.example.com/repository/pacman-hosted/x86_64/astrum-tool-1.4.0-1-x86_64.pkg.tar.zst
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig \
  https://astrum.example.com/repository/pacman-hosted/x86_64/astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig
```

The upload page and the components API take packages and signatures in the fields `pacman.asset1`,
`pacman.asset2` and so on; `pacman.directory` names the folders above the architecture folder, and the
architecture folder comes from the package:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F pacman.directory=custom/os \
  -F pacman.asset1=@astrum-tool-1.4.0-1-x86_64.pkg.tar.zst \
  -F pacman.asset2=@astrum-tool-1.4.0-1-x86_64.pkg.tar.zst.sig \
  "https://astrum.example.com/service/rest/v1/components?repository=pacman-hosted"
```

`repo-add` is not needed: deleting a package (its signature goes with it) rewrites the databases too, and
the task **pacman - Rebuild pacman databases** writes them anew, reading the file lists from the packages.

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

## Puppet Forge

Repositories serve Puppet modules through the Forge API v3 (`/v3/modules`, `/v3/releases`, `/v3/files`) that
`puppet module install`, r10k, Bolt and librarian-puppet read: hosted repositories take the archives that
`pdk build` and `puppet module build` write, proxies cache another Forge such as
`https://forgeapi.puppet.com`, and groups merge the releases of their members. The Forge address of a
repository is its address without the trailing slash, for example
`https://astrum.example.com/repository/puppet-group`; the clients add `/v3/...` to it.

Where anonymous users may not read the repository, the clients send an
[access token](../configure/authentication.md#access-tokens) of this server in the `Authorization`
header. `puppet module install` takes the repository from `puppet.conf` (`puppet config print config`
shows where it is) or from `--module_repository`, and the header from `forge_authorization`:

```ini
[main]
module_repository = https://astrum.example.com/repository/puppet-group
forge_authorization = <access token>
```

```shell
puppet module install puppetlabs-stdlib
puppet module install acme-tools --version 1.4.0 \
  --module_repository https://astrum.example.com/repository/puppet-group
```

r10k (`r10k.yaml`) and Bolt (`bolt-project.yaml` or `bolt-defaults.yaml`) take the token with the
`Bearer` prefix:

```yaml
forge:
  baseurl: 'https://astrum.example.com/repository/puppet-group'
  authorization_token: 'Bearer <access token>'
```

```yaml
module-install:
  forge:
    baseurl: https://astrum.example.com/repository/puppet-group
    authorization_token: Bearer <access token>
```

librarian-puppet reads the address from the `forge` line of the Puppetfile; r10k reads that line only
with `allow_puppetfile_override`:

```ruby
forge 'https://astrum.example.com/repository/puppet-group'

mod 'puppetlabs-stdlib', '9.6.0'
mod 'acme-tools', '1.4.0'
```

Clients that take credentials only in the address can use
`https://<user>:<access token>@astrum.example.com/repository/puppet-group`; the server reads them as Basic
authentication.

Publish to a hosted repository with `pdk release publish` (after `pdk build`, from the module directory),
with `rake module:push` of puppet-blacksmith, or with any client that posts the archive as the
multipart field `file` to `/v3/releases`:

```shell
pdk release publish --forge-token "$ASTRUM_TOKEN" \
  --forge-upload-url https://astrum.example.com/repository/puppet-hosted/v3/releases
curl -H "Authorization: Bearer $ASTRUM_TOKEN" -F file=@pkg/acme-tools-1.4.0.tar.gz \
  https://astrum.example.com/repository/puppet-hosted/v3/releases
```

puppet-blacksmith reads the address and the token from `~/.puppetforge.yml` (or
`BLACKSMITH_FORGE_URL` and `BLACKSMITH_FORGE_API_KEY`):

```yaml
---
url: https://astrum.example.com/repository/puppet-hosted
api_key: <access token>
```

The archive is a gzip-compressed tar with `<owner>-<name>-<version>/metadata.json`; that file must name
the module as `<owner>-<name>` or `<owner>/<name>` (the owner of letters and digits, the name of lower-case
letters, digits and underscores starting with a letter), give a semantic version and list its
dependencies as `name` and `version_requirement`. Owners are compared in lower case. A version that is
already there is refused with 409 unless the write policy allows redeploying. The JSON body of the Forge
API (`{"file": "<base64 archive>"}`) works too, and the components API takes the same archives in the
field `puppet.asset`. `DELETE /v3/releases/<owner>-<name>-<version>` removes a release and
`DELETE /v3/modules/<owner>-<name>` all releases of a module; the metadata follows at once.

Release documents of hosted repositories carry `metadata.json` verbatim, `file_md5`, `file_sha256`,
`file_size` and `file_uri`; `readme`, `changelog`, `license` and `reference` are `null`. Proxies keep the
module and release documents and the release list of each module as metadata, point every `file_uri` at
`/v3/files/` of the proxy and check each archive against `file_sha256` (or `file_md5`) of its release.
Their release lists leave out `readme`, `changelog`, `license` and `reference`, which the document of a
single release keeps. Module searches (`/v3/modules?query=`) and release lists without `module` go to the
remote of a proxy each time. Groups merge the releases of their members (the first member wins for a
release) and their search results, of which they take the first thousand.

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

## Vagrant

Repositories serve Vagrant boxes with the box metadata that `vagrant box add`, `vagrant box outdated` and
`vagrant box update` read: hosted repositories take uploads, proxies cache another box server such as
Vagrant Cloud (`https://vagrantcloud.com`), and groups merge the versions of their members. The metadata
of the box `<org>/<name>` is at `<repository>/<org>/<name>`, and also at
`<repository>/api/v2/vagrant/<org>/<name>`, the address Vagrant asks first, and at
`<repository>/<org>/<name>/metadata.json`; each box file is at
`<repository>/<org>/<name>/<version>/<provider>/<architecture>.box`.

To add boxes by their short name, make a repository the box server of Vagrant:

```shell
export VAGRANT_SERVER_URL=https://astrum.example.com/repository/vagrant-group
export VAGRANT_CLOUD_TOKEN=<access token>
vagrant box add acme/base --provider virtualbox
```

Vagrant sends `VAGRANT_CLOUD_TOKEN` as `Authorization: Bearer` with every request to the host of
`VAGRANT_SERVER_URL`, for the metadata and the box files alike; use an
[access token](../configure/authentication.md#access-tokens) and leave it out where anonymous users may
read the repository. `vagrant cloud auth login --token <access token>` stores the token in
`~/.vagrant.d/data/vagrant_login_token` instead, after checking it against
`<repository>/api/v2/authenticate`; a sign-in with a user name and password there is refused, because
the server does not hand out tokens that way. Vagrant warns once per run that it sends the token to a
server other than Vagrant Cloud and waits five seconds. Do not set `VAGRANT_SERVER_ACCESS_TOKEN_BY_URL`:
the server does not read tokens from the `access_token` query parameter.

A Vagrantfile that names the metadata address keeps `vagrant box outdated` and the update check of
`vagrant up` on this server, with or without `VAGRANT_SERVER_URL`:

```ruby
Vagrant.configure("2") do |config|
  config.vm.box = "acme/base"
  config.vm.box_url = "https://astrum.example.com/repository/vagrant-group/acme/base"
  config.vm.box_version = "~> 1.2"
end
```

Where anonymous users may not read the repository, keep `VAGRANT_SERVER_URL` on the host of `box_url`
and `VAGRANT_CLOUD_TOKEN` set: Vagrant sends the token only to that host, and credentials written into
`box_url` reach the metadata but not the box files it lists.

Upload a box to a hosted repository with `PUT`, naming its version, provider and architecture in the
path. The architecture is one of the values Vagrant compares with the host: `amd64`, `arm64`, `i386`,
`arm`, `ppc64le`, `ppc64`, `mips64le`, `mips64`, `mipsle`, `mips`, `s390x`, or `unknown` for a box that
names none:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file package.box \
  https://astrum.example.com/repository/vagrant-hosted/acme/base/1.2.0/virtualbox/amd64.box
```

The repository computes the SHA-256 checksum, lists the file in the metadata of the box with
`checksum_type` `sha256` and answers `201` with the checksum and the download address. With strict
content type validation (the default) it refuses a file that is not a tar, tar.gz, zip, bzip2, xz or
zstd archive, and unless the write policy allows redeploying it refuses a second upload to the same
path. Of several architectures of a provider, `amd64` is the default architecture, else `unknown`, else
the first; a single one is always the default. Deleting a box file takes it out of the metadata, and a
version without files disappears from it. The upload page and the components API take boxes as well,
with the fields `vagrant.organization`, `vagrant.name`, `vagrant.version`, `vagrant.provider`,
`vagrant.architecture`, an optional `vagrant.description` of the version and the file in
`vagrant.asset`:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F vagrant.organization=acme -F vagrant.name=base \
  -F vagrant.version=1.2.0 -F vagrant.provider=virtualbox -F vagrant.architecture=amd64 \
  -F vagrant.asset=@package.box \
  "https://astrum.example.com/service/rest/v1/components?repository=vagrant-hosted"
```

Proxies read the metadata of a box from `<remote>/<org>/<name>` with `Accept: application/json`, which
Vagrant Cloud and every server that works as `VAGRANT_SERVER_URL` answer. They point the provider
addresses at themselves, fetch each box file from the address the remote listed, following its
redirects to object storage, and check it against the checksum of the metadata (md5, sha1, sha256 or
sha512); a cached box whose checksum still matches the metadata is not fetched again. For private
boxes of Vagrant Cloud give the proxy a bearer token in its HTTP settings; it is sent only to the host of
the remote URL. Groups merge the versions of their members, the first member winning for a provider
and architecture of a version; upload to the hosted repository itself. `vagrant cloud publish` and the
other `vagrant cloud` commands that manage boxes are not served.

## Nx remote cache

Hosted `nx` repositories are a self-hosted remote cache for [Nx](https://nx.dev) 20.8 and later: Nx uploads
the outputs of its tasks and restores them on other machines and in CI instead of running the tasks again.
The server answers the custom remote cache API of Nx, `GET` and `PUT` on `<repository>/v1/cache/<hash>`;
the format has no proxy or group repositories.

```shell
export NX_SELF_HOSTED_REMOTE_CACHE_SERVER=https://astrum.example.com/repository/nx-cache
export NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN=<access token>
npx nx run-many -t build
```

Nx sends the token as `Authorization: Bearer`, so it has to be an
[access token](../configure/authentication.md#access-tokens): a read token only restores outputs, a write
token also uploads them, and Nx skips the upload quietly when the token may not write. A stored output is
never replaced: a second upload of the same hash answers 409, which Nx expects. Leave the workspace
unconnected to Nx Cloud (no `nxCloudId` or `nxCloudAccessToken` in `nx.json`), or Nx uses the cloud
instead. Each output is stored as `<repository>/<hash>` and can be downloaded there, which is what the
repository browser links to; old outputs go away with a cleanup policy on the time they were last
downloaded.

## Turborepo remote cache

Hosted `turbo` repositories are a remote cache for [Turborepo](https://turborepo.dev): `turbo run` uploads
the outputs of its tasks and restores them on other machines and in CI instead of running the tasks again.
The server answers the Remote Cache API under `<repository>/v8/artifacts` (and under `<repository>/artifacts`,
the paths of the specification); the format has no proxy or group repositories.

```shell
export TURBO_API=https://astrum.example.com/repository/turbo-cache
export TURBO_TOKEN=<access token>
export TURBO_TEAM=web
turbo run build
```

The address and the team can live in `turbo.json` instead; keep the token in `TURBO_TOKEN`:

```json
{
  "$schema": "https://turborepo.dev/schema.json",
  "remoteCache": {
    "apiUrl": "https://astrum.example.com/repository/turbo-cache",
    "teamSlug": "web",
    "signature": true
  }
}
```

Turborepo sends `TURBO_TOKEN` as `Authorization: Bearer`, so it has to be an
[access token](../configure/authentication.md#access-tokens): a read token only restores outputs, a write
token also uploads them. Uploading a new artifact takes the add privilege of the repository, replacing one
the edit privilege. `turbo login` and `turbo link` talk to Vercel and are not needed.

The team (`TURBO_TEAM`, or `teamSlug`, or a `teamId` that starts with `team_`) keeps the caches of teams
apart: each artifact is stored as `<repository>/<team>/<hash>`, and requests without a team use the team
`_`. A team name holds letters, digits, `.`, `-` and `_`. Content selectors on these paths give teams
privileges of their own within one repository. The stored artifacts can also be downloaded at those paths,
which is what the repository browser links to.

With `"signature": true` and `TURBO_REMOTE_CACHE_SIGNATURE_KEY` set, Turborepo signs what it uploads and
refuses downloads without a valid signature. The server keeps the signature (`x-artifact-tag`), the task
duration and the commit that the client reports, and returns them with the artifact; it does not check
signatures itself. Old artifacts go away with a cleanup policy on the time they were last downloaded.

## Raw files

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file build/tool-1.4.0.tar.gz \
  https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
curl -fO https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
```
