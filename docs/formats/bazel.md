# Bazel

Repositories are module registries for Bazel's Bzlmod: the files that `bazel` reads from a registry named
with `--registry`. Hosted repositories take the files of modules, proxies cache another registry such as the
Bazel Central Registry (`https://bcr.bazel.build`), and groups merge their members. A registry is a set of
files under the repository address:

| Path | Content |
| ---- | ------- |
| `bazel_registry.json` | Registry-wide settings: `mirrors` and `module_base_path` |
| `modules/<module>/metadata.json` | The versions of a module, who maintains it, which versions are yanked |
| `modules/<module>/<version>/MODULE.bazel` | The `MODULE.bazel` that Bazel resolves dependencies with |
| `modules/<module>/<version>/source.json` | Where the sources come from: an `archive`, a `git_repository` or a `local_path` |
| `modules/<module>/<version>/patches/<file>` | Patches that `source.json` lists for an archive or a Git source |
| `modules/<module>/<version>/overlay/<file>` | Files that `source.json` lists to lay over the unpacked archive |

Point Bazel at a repository in `.bazelrc`:

```text
common --registry=https://astrum.example.com/repository/bazel-group
```

`--registry` replaces the default Bazel Central Registry, and a registry listed earlier wins. Bazel itself
depends on modules of the Central Registry (`bazel_tools` needs `rules_license` and `platforms`, for
example), so a hosted repository alone is not enough: either point `--registry` at a group that has your
hosted repository and a proxy of `https://bcr.bazel.build` as members, or list the Central Registry after
the hosted repository:

```text
common --registry=https://astrum.example.com/repository/bazel-hosted
common --registry=https://bcr.bazel.build
```

Bazel sends the credentials of `~/.netrc`, or of the file the `NETRC` variable names, to the host of a
registry. Use the user name and an [access token](../configure/authentication.md#access-tokens) as the
password, and leave the file out where anonymous users may read the repository:

```text
machine astrum.example.com login ci-deployer password <access token>
```

Then `bazel mod graph`, `bazel mod deps` and `bazel fetch` resolve through the repository, and a
`bazel_dep(name = "rules_demo", version = "1.2.0")` in `MODULE.bazel` finds the module.

## Publishing modules

Upload the files of a version with `PUT`, to the path they have in the registry. Upload the patches and
overlays first, then `MODULE.bazel` and `source.json` last, because a version is listed in `metadata.json`
once it has both of those two files:

```shell
base=https://astrum.example.com/repository/bazel-hosted/modules/rules_demo
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file fix.patch "$base/1.2.0/patches/fix.patch"
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file MODULE.bazel "$base/1.2.0/MODULE.bazel"
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file source.json "$base/1.2.0/source.json"
```

The upload page and the components API take the same files with the fields `bazel.module`, `bazel.version`
and, for each file, `bazel.asset<N>` and its place in the registry in `bazel.asset<N>.filename`
(`MODULE.bazel`, `source.json`, `metadata.json`, `patches/<file>` or `overlay/<file>`; a file that is
uploaded under a `.patch` or `.diff` name without a path goes to `patches`). The files are written in the
order above whatever the order of the form, so a form that carries a whole version leaves a complete one:

```shell
curl -u "ci-deployer:$ASTRUM_PASSWORD" -F bazel.module=rules_demo -F bazel.version=1.2.0 \
  -F bazel.asset1=@fix.patch -F bazel.asset2=@MODULE.bazel -F bazel.asset3=@source.json \
  "https://astrum.example.com/service/rest/v1/components?repository=bazel-hosted"
```

The repository checks what it can before it stores a file:

- A module name has lowercase letters, digits, `.`, `-` and `_`, starts with a letter and ends with a letter
  or digit; a version is a Bazel version such as `1.2.0`, `1.2.0-rc1` or `1.2.0.bcr.1`.
- `MODULE.bazel` is UTF-8 text. If its `module()` call gives the name or the version as a literal, it has to
  be the name and the version of the path: Bazel refuses a file that declares another one.
- `source.json` is a JSON object. An `archive` (the default `type`) needs a `url` and an `integrity` such as
  `sha256-<base64>`, a `git_repository` needs a `remote` and a `commit` or `tag`, a `local_path` needs a
  `path`. The names under `patches` and `overlay` must be plain relative file names.
- The integrity that `source.json` lists for a patch or an overlay file is compared with the file, whichever
  of the two arrives first; a mismatch is refused with `400`.
- A file of a version is never replaced: a second `PUT` to the same path answers `409`, whatever the write
  policy of the repository, because clients and lock files record the content of registry files. Delete the
  file to publish it again, or publish a new version (`1.2.0.bcr.1` is the usual name for a fix). A write
  policy of *Read-only* refuses every upload.

Deleting `MODULE.bazel` or `source.json` takes the version out of `metadata.json`.

`modules/<module>/metadata.json` is generated: its `versions` are the complete versions of the module in
Bazel's order, and `yanked_versions` is an empty object. To add `homepage`, `maintainers`, `repository`,
`deprecated` or `yanked_versions`, `PUT` a `metadata.json` of your own; the server keeps those fields and
always replaces `versions`. Unlike the files of a version it can be replaced, and `DELETE` brings back the
generated one:

```shell
cat > metadata.json <<'EOF'
{
  "homepage": "https://example.org/rules_demo",
  "maintainers": [{"name": "A. Maintainer", "github": "maintainer"}],
  "yanked_versions": {"1.1.0": "crashes on Windows, use 1.2.0"}
}
EOF
curl -u "ci-deployer:$ASTRUM_PASSWORD" --upload-file metadata.json \
  https://astrum.example.com/repository/bazel-hosted/modules/rules_demo/metadata.json
```

Bazel stops with an error when it resolves to a yanked version, quoting the reason, unless
`--allow_yanked_versions` allows it. `bazel_registry.json` is generated as `{"mirrors": []}` and can be
replaced and deleted the same way.

## Archives and mirrors

A registry holds the description of a module, not its sources: Bazel downloads the archive from the `url` of
`source.json`, so the machines that run Bazel need to reach that address. To keep the archives on this server
too, use either of two ways:

- Put the archives into a `raw` repository and write its URL into `url`, for example
  `https://astrum.example.com/repository/bazel-archives/rules_demo-1.2.0.tar.gz`. Bazel checks the download
  against `integrity`, so the file cannot change unnoticed.
- Set `mirrors` in `bazel_registry.json`. For each source URL Bazel first tries the mirror followed by the
  URL without its protocol: with the mirror `https://astrum.example.com/repository/bazel-mirror/`, the archive
  `https://github.com/org/repo/archive/v1.tar.gz` is asked for at
  `https://astrum.example.com/repository/bazel-mirror/github.com/org/repo/archive/v1.tar.gz` and fetched from
  the original address when the mirror answers `404` (Bazel prints a warning for the failed attempt). Fill
  the `raw` repository `bazel-mirror` with the archives at those paths.

`mirror_urls` in `source.json` lists extra addresses to try after `url`. A `local_path` source is resolved
on the machine that runs Bazel, so with a registry served over HTTP only absolute paths work.

## Proxies and groups

A proxy takes `https://bcr.bazel.build` or another registry as its remote URL. It keeps the files of a
version (`MODULE.bazel`, `source.json`, patches and overlays) for good, whatever its content max age, because
a registry does not change them; delete a cached file to fetch it again. `bazel_registry.json` and
`metadata.json` are fetched again when they are older than the metadata max age of the proxy, since new
versions and yanks arrive there, and an unreachable remote is answered from the stale copy. Only the paths
above are forwarded, nothing else on the remote can be reached through the proxy. Archives are not cached:
Bazel downloads them itself.

A group answers like one registry. The `metadata.json` of a module lists the versions of all members that
know it, with the fields of the first member that sets them and the yanked versions of every member; a
file of a version comes from the first member, in the order of the group, that has it. The `mirrors` of
`bazel_registry.json` are collected from all members and apply to every archive Bazel downloads through
the group.

A module name that a hosted member of the group knows is never asked of the proxies of the group, wherever
the proxies stand in the member list: `metadata.json` and the files of a version are those of the hosted
members alone. A public registry therefore cannot put a version of the same name in front of your private
module (dependency confusion), and cannot add versions to it either. Name the modules of the hosted
repository differently from public ones if you want both to resolve, or keep them in separate registries.
Among several hosted members, or among several proxies, the order of the group decides.
