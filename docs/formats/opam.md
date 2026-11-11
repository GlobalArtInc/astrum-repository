# OPAM

OPAM is the package manager of OCaml. Its repositories are plain trees of files: `opam update` downloads
one archive, `index.tar.gz`, that lists every package of the repository, and the source archives come
from the `url` of each package, first from the repository's own cache. An OPAM repository here is
either hosted, where a team publishes its own packages with `curl -T`, or a proxy of
`https://opam.ocaml.org` for closed networks. There are no group repositories: opam ranks several
repositories itself (`opam repo add ... --rank`), and merging their indexes would mean unpacking
archives from other servers.

Add a repository to opam, for every switch and ahead of the others:

```shell
opam repo add astrum https://astrum.example.com/repository/ocaml --all-switches --set-default
```

For a proxy in a closed network remove the default repository, so that only this server answers:

```shell
opam repo add astrum https://astrum.example.com/repository/opam-proxy --all-switches --set-default
opam repo remove default --all
opam update
```

## Credentials

opam has no setting for credentials. A repository that is not readable by the anonymous user (see
[anonymous access](../configure/authentication.md#anonymous-access)) would need them in the URL
(`https://user:<access token>@astrum.example.com/repository/ocaml`). That has **not been checked with a
real opam client**, so do not rely on it: until it has been, allow anonymous reads on the repository, or
let only your own network reach it.

## Layout

A hosted repository holds these paths; the server generates the first two.

| Path | What it is |
| --- | --- |
| `repo` | `opam-version: "2.0"` and `archive-mirrors: "cache"`, which tells opam to look in `cache/` first |
| `index.tar.gz` | the archive opam downloads on every `opam update` |
| `packages/<name>/<name>.<version>/opam` | the opam file of a version |
| `packages/<name>/<name>.<version>/files/<path>` | a file the package ships with its opam file, such as a patch |
| `cache/<sha256 or sha512>/<xx>/<digest>` | a source archive, named by its checksum, `<xx>` being the first two characters |

`index.tar.gz` is rebuilt after every upload and every delete of a file under `packages/`, inside a
lock per repository, so concurrent uploads leave an index that holds all of them. It holds `repo` and then
every file under `packages/`, sorted by path, as plain files owned by uid 0 with mode 0644 and the time of
the upload. The server only stores the opam files and does not read them: opam parses them. If an
upload fails after the file has been stored, the next read or write of the index rebuilds it. A
repository holds at most 50 000 index entries, `repo` included; an upload that would add more answers
`409 Conflict` with "too many package files".

The package files of one repository (every file under `packages/`, which is what the index reads) take at
most 32 MiB together; an upload that would pass that answers `413 Payload Too Large` and stores nothing,
and replacing a file counts only the difference. If a rebuild of the index fails, for example because a
blob of a package was lost, a read answers the previous index and the server tries again at most every 30
seconds; a repository that has no index yet answers `503 Service Unavailable` until the rebuild works.
Names that differ only by case (`Foo` and `foo`) collide on case-insensitive file systems, which many
opam clients run on, so do not mix them in one repository.

## Publishing

opam has no publish command, so files go up with `curl -T`. The user needs the add right on the
repository (and edit to replace a file, delete to remove one); give `curl` the access token as the password.

```shell
export REPO=https://astrum.example.com/repository/ocaml

curl -u user:$TOKEN -T hello.opam $REPO/packages/hello/hello.1.0/opam
curl -u user:$TOKEN -T fix.patch $REPO/packages/hello/hello.1.0/files/patches/fix.patch

SUM=$(sha256sum hello-1.0.tar.gz | cut -d' ' -f1)
curl -u user:$TOKEN -T hello-1.0.tar.gz $REPO/cache/sha256/${SUM:0:2}/$SUM
```

The `url` of the opam file can be any address, even one that does not resolve from your network, if the
archive is also in the cache under the same checksum, because opam asks the cache first:

```
opam-version: "2.0"
synopsis: "Hello"
build: ["dune" "build" "-p" name "-j" jobs]
url {
  src: "https://example.com/hello-1.0.tar.gz"
  checksum: "sha256=<the same digest>"
}
```

Then `opam update` and `opam install hello` find the package, and the archive comes from the cache.
Delete a file with `curl -X DELETE` on the same path; the index is rebuilt.

| Request | Answer |
| --- | --- |
| a new file | `201 Created` |
| a file that exists is replaced, or a cache file that exists is sent again | `204 No Content` |
| the `opam` file of a version that exists, on a repository whose write policy is "allow once" | `409 Conflict` |
| the same on a read-only repository, or anything that breaks a rule below | `400 Bad Request` |
| `repo` or `index.tar.gz` | `400 Bad Request`: they are generated |

The rules, checked on every upload:

- A name starts with a letter or digit and holds letters, digits, `_`, `+` and `-`, at most 64 bytes. A
  version holds letters, digits, `_`, `+`, `.`, `~` and `-`, at most 64 bytes. The directory of a package
  must be exactly `<name>.<version>`.
- An `opam` file is UTF-8 text of at most 256 KiB. A file under `files/` is at most 1 MiB, has at most 8
  path segments, and a segment holds letters, digits, `.`, `_`, `+` and `-` and is not `.` or `..`.
- The body of a cache file must have the digest in its path, and `<xx>` must be the first two characters
  of it, in lowercase hexadecimal (64 characters for sha256, 128 for sha512), or the upload is refused
  and nothing is stored. A cache file is stored as sent and has no size limit of its own other than the
  blob store.

Other paths and methods are refused like in the other formats; a repository answers `GET`, `HEAD`,
`PUT` and `DELETE`.

## Proxy

A proxy of `https://opam.ocaml.org` (set as the remote URL) keeps what opam asks for, byte for byte, and
the relative `archive-mirrors: "cache"` in its `repo` makes opam ask the proxy for archives too. Only
these paths are ever sent to the remote; anything else, `packages/...` included, is `404 Not Found`
and the remote is not asked:

| Path | Kept |
| --- | --- |
| `index.tar.gz` and `repo` | for the metadata max age of the repository (about 5 MB for the index) |
| `cache/<sha256 or sha512>/<xx>/<digest>`, with `<xx>` the first two characters of a digest of 64 or 128 lowercase hexadecimal characters | for good: a file is named by its digest |

The digest of every cache file is checked against its path before the file is stored: a file that does
not match is not stored and answers `502 Bad Gateway`. A proxy answers `GET` and `HEAD` only.
Packages whose archives opam.ocaml.org does not hold in its cache are downloaded by opam from the `url`
of each package, from the sites of their authors, and do not go through this repository.
