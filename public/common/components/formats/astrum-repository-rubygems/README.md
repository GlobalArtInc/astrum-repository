# RubyGems repositories

Astrum serves gems to `gem` and Bundler with the [compact index](https://guides.rubygems.org/rubygems-org-compact-index-api/)
that current clients read, the legacy Marshal indexes (`specs.4.8.gz` and the quick specifications) that `gem sources`,
`gem search --remote` and `gem install` still fetch, the dependency API of older Bundler versions and the push and yank
calls of the [RubyGems.org API](https://guides.rubygems.org/rubygems-org-api/). Three recipes exist:

- `rubygems-hosted`: `gem push`, `gem yank` and uploads through the components API. Every index is generated from the
  stored gems; `/versions`, `/names` and `specs.4.8.gz` are kept as assets and rebuilt after a push, a yank or a
  deletion, `/info/<gem>` and the quick specifications are built on request.
- `rubygems-proxy`: caches a remote gem server, for RubyGems.org use `https://rubygems.org/` as the remote URL. Index
  files are metadata (refreshed after the metadata max age), gems and quick specifications are content.
- `rubygems-group`: merges the compact index of its members (the first member wins for a version) and builds the
  legacy indexes from the merged `/versions`; gems and quick specifications come from the first member that has them.
  The merged files are kept as assets of the group and rebuilt when a member's file changes.

A gem is a component named after the gem with its version; every platform of a version is an asset of that component,
stored at `/gems/<name>-<version>[-<platform>].gem` with the gem specification in its `rubygems` attributes. Search
accepts `rubygems.platform`, `rubygems.summary` and `rubygems.description` besides name and version, cleanup policies
support the release type and retain-N, and a version with a letter is a prerelease.

## Create repositories

    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/rubygems/hosted \
      -d '{"name":"gems-hosted","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow_once"}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/rubygems/proxy \
      -d '{"name":"gems-proxy","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true},"proxy":{"remoteUrl":"https://rubygems.org/","contentMaxAge":1440,"metadataMaxAge":60},"negativeCache":{"enabled":true,"timeToLive":60},"httpClient":{"blocked":false,"autoBlock":true}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/rubygems/group \
      -d '{"name":"gems","online":true,"storage":{"blobStoreName":"default"},"group":{"memberNames":["gems-hosted","gems-proxy"]}}'

`GET` and `PUT /service/rest/v1/repositories/rubygems/{hosted,proxy,group}/<name>` read and update them.

## API keys

`gem push` and `gem yank` send the API key in the `Authorization` header as it is. Two kinds of keys work:

- A RubyGems API key, `RubyGemsApiKey.<key>`. An administrator activates the **RubyGems API Key Realm** once
  (Security > Realms, or `PUT /service/rest/v1/security/realms/active` with `RubyGemsApiKey` in the list). A user then
  runs `gem signin` with `RUBYGEMS_HOST` set to the repository URL and the user name and password of this server, or
  opens `<repository URL>/api/v1/api_key` with `curl -u <user>`; the same key works for every RubyGems repository the
  user can reach. `DELETE <repository URL>/api/v1/api_key` revokes it, the next request creates a new one.

  Use `RUBYGEMS_HOST` rather than `gem signin --host`: with `--host` alone, gem 3.5 still asks RubyGems.org for the
  profile of the user and sends it the user name and password typed for this server.
- Basic credentials: `Basic <base64 of user:password>` used as the key (for example in `GEM_HOST_API_KEY`), no realm
  needed.

Pushing needs the add privilege and yanking the delete privilege on the repository; both are checked again against
the path of the gem, so content selectors on `/gems/...` apply. A yank deletes the gem of that version and platform.

## Configure gem and Bundler

    gem sources --add https://repo.example.com/repository/gems/
    gem install rails --clear-sources --source https://repo.example.com/repository/gems/

    export RUBYGEMS_HOST=https://repo.example.com/repository/gems-hosted
    gem signin
    gem push my_gem-1.0.0.gem
    gem yank my_gem --version 1.0.0
    gem yank my_gem --version 1.0.0 --platform x86_64-linux

`Gemfile`:

    source "https://repo.example.com/repository/gems/"

To send every request for RubyGems.org through a proxy or group instead:

    bundle config set --global mirror.https://rubygems.org https://repo.example.com/repository/gems/

When anonymous access is disabled, put the credentials into the source URL
(`https://user:password@repo.example.com/...`) or give them to Bundler with
`bundle config set --global repo.example.com user:password`.

## Paths

| Path | Purpose |
| --- | --- |
| `/versions`, `/info/<gem>`, `/names` | compact index; the ETag is the MD5 of the file, as Bundler expects |
| `/specs.4.8.gz`, `/latest_specs.4.8.gz`, `/prerelease_specs.4.8.gz` | legacy indexes, gzipped Marshal arrays of `[name, Gem::Version, platform]` |
| `/quick/Marshal.4.8/<name>-<version>[-<platform>].gemspec.rz` | deflated Marshal of the gem specification |
| `/gems/<name>-<version>[-<platform>].gem` | gem download |
| `GET /api/v1/dependencies?gems=<a>,<b>` | dependency API, Marshal, built from `/info/<gem>` of the repository |
| `POST /api/v1/gems` | push, the body is the gem file |
| `DELETE /api/v1/gems/yank` | yank, form fields `gem_name`, `version` and `platform` |
| `GET`, `POST`, `DELETE /api/v1/api_key` | issue or revoke the RubyGems API key of the current user |
