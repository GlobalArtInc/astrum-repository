# Bazel remote cache

Hosted `bazelcache` repositories are a self-hosted [remote cache](https://bazel.build/remote/caching) for Bazel:
Bazel uploads the results of its actions and the blobs they produce, and downloads them on other machines and in CI
instead of building them again. The server answers Bazel's HTTP remote cache protocol. This format is unrelated to the
[Bazel](bazel.md) module registry format, which serves the modules that Bzlmod resolves.

Point Bazel at a repository in `.bazelrc`; the address needs no trailing slash:

```text
build --remote_cache=https://astrum.example.com/repository/bazel-cache
build --remote_header=Authorization=Bearer <access token>
```

The header carries an [access token](../configure/authentication.md#access-tokens). Keep it out of the `.bazelrc` of a
workspace, which is committed: put the header into `~/.bazelrc` on a developer machine, or into a secret of the CI
pipeline.

Developers use a token with the read right only, so their builds restore results from the cache, and they turn uploads
off. CI uses a token with the write right and uploads its results:

```text
build --remote_upload_local_results=false
```

```text
build --remote_upload_local_results=true
```

The first line is for developers and the second for CI. Without a write right a token gets a 403 on upload.

## Protocol

The repository answers two namespaces under its address:

| Path | Content |
| ---- | ------- |
| `GET` and `PUT` on `ac/<hash>` | The action cache: the serialized result of an action, stored exactly as it arrives |
| `GET` and `PUT` on `cas/<hash>` | The content-addressable store: a blob |

A `<hash>` is the SHA-256 digest of the content, written as 64 hexadecimal characters. Keys are stored in lower case.
Only SHA-256 is supported, which is the Bazel default; other digest functions are not.

- A `GET` of a stored key answers 200 with the bytes, and a missing key answers 404.
- A `PUT` answers 200 when the entry is stored. A key that already exists answers 200 as well and keeps the first
  entry, because the entries are content-addressed and a non-2xx answer makes Bazel count a failure.
- A `PUT` to `cas/<hash>` is verified: the body must hash to its key, or the answer is 400 and nothing is stored. The
  bodies of `ac/` entries are not verified, since they are not content-addressed.
- A key that is not 64 hexadecimal characters is 404 on `GET` and 400 on `PUT`, and a path under another first segment
  is 404.

An entry is stored as `<repository>/ac/<hash>` or `<repository>/cas/<hash>` and is listed as the component
`ac/<hash>` or `cas/<hash>`. An administrator can delete an entry, and the key can then be uploaded again; old entries
go away with a cleanup policy on the time they were last downloaded. The format has no proxy or group repositories.
