# Puppet Forge

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
