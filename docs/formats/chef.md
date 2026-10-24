# Chef Supermarket

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
