# Vagrant

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
