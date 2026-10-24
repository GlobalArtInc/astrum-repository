# Hex (Elixir and Erlang)

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
semantic version. `mix hex.publish --revert <version>` deletes a release,
`mix hex.retire <package> <version> <reason> --message <text>` marks it retired in the registry and
`mix hex.retire <package> <version> --unretire` takes that back; `mix hex.info <package>` and
`mix hex.user whoami` read the same API. The repository keeps no package owners: who may publish follows the privileges of the repository.
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
