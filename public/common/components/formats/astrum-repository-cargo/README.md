# Cargo repositories

Astrum serves Rust crates with the [sparse index protocol](https://doc.rust-lang.org/cargo/reference/registry-index.html#sparse-protocol)
and the cargo [Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html). Three recipes exist:

- `cargo-hosted`: publish, yank and unyank with `cargo`, index files are generated from the stored crates.
- `cargo-proxy`: caches a remote sparse registry, for crates.io use `https://index.crates.io/` as the remote URL.
  Index files are metadata (refreshed after the metadata max age), crate files are content.
- `cargo-group`: merges the index files of its members (the first member wins for a version) and serves a crate
  from the first member that has it. With a writable member, `cargo publish` and `cargo yank` through the group
  go to that hosted member.

## Create repositories

    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/cargo/hosted \
      -d '{"name":"cargo-hosted","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"allow_once"}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/cargo/proxy \
      -d '{"name":"cargo-proxy","online":true,"storage":{"blobStoreName":"default","strictContentTypeValidation":true},"proxy":{"remoteUrl":"https://index.crates.io/","contentMaxAge":1440,"metadataMaxAge":60},"negativeCache":{"enabled":true,"timeToLive":60},"httpClient":{"blocked":false,"autoBlock":true}}'
    curl -u admin -H 'Content-Type: application/json' http://localhost:8081/service/rest/v1/repositories/cargo/group \
      -d '{"name":"cargo-group","online":true,"storage":{"blobStoreName":"default"},"group":{"memberNames":["cargo-hosted","cargo-proxy"],"writableMember":"cargo-hosted"}}'

Every type accepts `"cargo":{"requireAuthentication":true}`: anonymous requests are then refused even when
anonymous access is enabled, and `config.json` announces `auth-required`.

## Tokens

Cargo sends its token in the `Authorization` header as it is. Two kinds of tokens work:

- A Cargo token, `CargoToken.<key>`. An administrator activates the **Cargo Bearer Token Realm** once
  (Security > Realms, or `PUT /service/rest/v1/security/realms/active` with `CargoToken` in the list). A user
  then opens `<repository URL>/me` in a browser or with `curl -u <user> <repository URL>/me` and gets their token;
  the same token works for every Cargo repository the user can read. `DELETE <repository URL>/me` revokes it,
  the next `GET` creates a new one.
- Basic credentials: `Basic <base64 of user:password>` used as the token, no realm needed.

## Configure cargo

`~/.cargo/config.toml`:

    [registries.astrum]
    index = "sparse+https://repo.example.com/repository/cargo-group/"

    [registry]
    global-credential-providers = ["cargo:token"]

    [source.crates-io]
    replace-with = "astrum"

Then:

    cargo login --registry astrum <token>
    cargo publish --registry astrum
    cargo yank --registry astrum --version 0.1.0 my-crate
    cargo yank --registry astrum --version 0.1.0 --undo my-crate
    cargo search --registry astrum my-crate

Dependencies on crates of the registry are declared with `registry = "astrum"` in `Cargo.toml`.

`config.json` sets `auth-required` when the repository requires authentication or the request was
authenticated. When anonymous access is disabled, cargo receives `401` for `config.json`, retries with its
token and from then on sends the token with every index and download request. Cargo only sends tokens to
auth-required registries through a configured credential provider, hence `global-credential-providers` above.

## Paths

| Path | Purpose |
| --- | --- |
| `/config.json` | `dl` is `<repository URL>/api/v1/crates`, `api` is the repository URL |
| `/1/a`, `/2/ab`, `/3/a/abc`, `/ab/cd/abcd...` | index files, one JSON line per version |
| `/api/v1/crates/<name>/<version>/download` | crate download |
| `/crates/<name>/<version>/<name>-<version>.crate` | stored crate asset, also downloadable |
| `PUT /api/v1/crates/new` | publish |
| `DELETE /api/v1/crates/<name>/<version>/yank`, `PUT .../unyank` | yank and unyank |
| `GET /api/v1/crates?q=<text>&per_page=<n>` | search, forwarded to the remote API for proxies |
| `GET /api/v1/crates/<name>/owners` | the user who published the crate; owner changes are accepted and ignored |
| `GET`, `DELETE /me` | issue or revoke the Cargo token of the current user |
