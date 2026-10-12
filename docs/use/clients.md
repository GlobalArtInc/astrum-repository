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

## P2 (Eclipse)

Use `https://astrum.example.com/repository/p2-proxy/` as the update site in Eclipse or in a Tycho
target platform.

## Raw files

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file build/tool-1.4.0.tar.gz \
  https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
curl -fO https://astrum.example.com/repository/raw-releases/tool/1.4.0/tool-1.4.0.tar.gz
```
