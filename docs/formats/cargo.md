# Cargo

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
