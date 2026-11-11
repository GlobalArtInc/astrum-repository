# Homebrew

A Homebrew repository is a proxy only. `brew` downloads a signed index of all formulae and the
bottles (the prebuilt packages) from the internet; a proxy of it serves both from this server, so that
developer machines and CI on a closed network can run `brew install`. Its remote URL is
`https://formulae.brew.sh/api`, and it applies to the index only; the bottles always come from the
`homebrew/core` namespace of `https://ghcr.io`, which is fixed in the server.

There is no hosted or group form. Homebrew verifies the signature of every index file with a key
shipped in `brew` and has no switch to turn that off, so the proxy passes the files on byte for byte
and an own formula could not be added to them. Own formulae live in a git tap.

Point `brew` at the repository with two variables:

```shell
export HOMEBREW_API_DOMAIN=https://astrum.example.com/repository/brew/api
export HOMEBREW_ARTIFACT_DOMAIN=https://astrum.example.com/repository/brew
export HOMEBREW_ARTIFACT_DOMAIN_NO_FALLBACK=1
brew install jq
```

`HOMEBREW_API_DOMAIN` is where `brew` reads the index (`api/formula.jws.json`,
`api/internal/packages.<tag>.jws.json` and the other `.jws.json` files). `HOMEBREW_ARTIFACT_DOMAIN`
replaces `https://ghcr.io/`, so `brew` asks this server for `v2/homebrew/core/<formula>/manifests/<version>`
and `v2/homebrew/core/<formula>/blobs/sha256:<digest>`. `HOMEBREW_ARTIFACT_DOMAIN_NO_FALLBACK=1`
keeps `brew` from going to ghcr.io when this server fails, which would hide a broken repository.

`HOMEBREW_BOTTLE_DOMAIN` is not supported. It switches `brew` to flat file names built from the
formula name and version, which the server could map to a bottle only by guessing where the name ends
and the version begins; the content-addressed paths of `HOMEBREW_ARTIFACT_DOMAIN` give the same result
without guessing.

## What the proxy serves

Only these paths are ever sent to a remote; everything else is `404 Not Found`, so the repository
cannot be used to reach other addresses:

| Path | Fetched from | Kept |
| --- | --- | --- |
| `api/<name>.jws.json` and `api/internal/<name>.jws.json`, with `<name>` of `a-z`, `0-9`, `_` and `.` | the remote URL of the repository | for the metadata max age of the repository |
| `v2/homebrew/core/<image>/manifests/<tag>` | `https://ghcr.io/v2/homebrew/core/...` | for the metadata max age |
| `v2/homebrew/core/<image>/blobs/sha256:<digest>` | `https://ghcr.io/v2/homebrew/core/...` | for good: a blob is named by its digest |

An image is the formula name in one or two segments (`jq`, `openssl/3`), each of `a-z`, `0-9`, `.`,
`_` and `-`. Blobs are fetched with the anonymous pull token of ghcr.io and follow its redirect to
the download network without that token. The SHA-256 of a blob must equal the digest in its path: a
blob that does not is not stored and answers `502 Bad Gateway`. Index files and manifests are stored
with the bytes and the `Content-Type` of the remote, and a query string on a request is not passed on.

Casks download from the sites of their vendors and building a formula from source fetches its upstream
sources: neither goes through this repository. Only the `homebrew/core` bottles are proxied, not those
of other taps.

## Access

`brew` sends no credentials when it reads the index, so the repository has to allow reading by the
anonymous user (see [anonymous access](../configure/authentication.md#anonymous-access)) at least for the
`api/` paths, or only your network must reach it. For the bottles a repository that is not anonymous can
be read with an access token: `brew` sends `HOMEBREW_DOCKER_REGISTRY_TOKEN` as a bearer token, and
`HOMEBREW_DOCKER_REGISTRY_BASIC_AUTH_TOKEN` (the base64 of `user:token`) as basic authentication.

```shell
export HOMEBREW_DOCKER_REGISTRY_TOKEN=<access token>
```

Allowing anonymous reads means that anyone who can reach the repository can make it download and keep
bottles from `homebrew/core`: the paths are fixed and every blob is checked against its digest, but the
storage fills with whatever is asked for. Keep such a repository inside your network, or put a limit on
the blob store.

The repository answers `GET` and `HEAD` only (a `HEAD` for a bottle that is not cached fetches it, as a
`GET` does); a cached file is removed from the web interface or through the components API.
