# sccache remote cache

Hosted `sccache` repositories are a self-hosted cache for [sccache](https://github.com/mozilla/sccache), the compiler
cache that can keep its objects on a WebDAV server. Rust and C/C++ builds upload the objects they compile and download
them on other machines and in CI instead of compiling them again. The server answers the small part of WebDAV that the
sccache client uses; it is not a general WebDAV server.

Developers point sccache at a repository through environment variables. The access token is a secret: keep it in the
environment of a developer machine or in a secret of the CI pipeline, not in a committed file.

```text
SCCACHE_WEBDAV_ENDPOINT=https://astrum.example.com/repository/sccache-cache
SCCACHE_WEBDAV_TOKEN=<access token>
```

For Rust, also set `RUSTC_WRAPPER=sccache`. For C and C++, run the compiler through sccache, for example `sccache gcc -c
a.c -o a.o`. The access token is an [access token](../configure/authentication.md#access-tokens) and is sent as a
bearer token. It can also be given as `SCCACHE_WEBDAV_PASSWORD`, with `SCCACHE_WEBDAV_USERNAME` set to the name of the
token's owner.

Developers use a token with the read right only, so their builds restore objects from the cache, and they switch
uploads off:

```text
SCCACHE_WEBDAV_RW_MODE=READ_ONLY
```

CI uses a token with the write right and uploads its objects. Without a write right a token gets a 403 on upload, and
sccache falls back to read-only mode when its start-up write check fails.

## Protocol

The repository answers under its address; the sccache client builds the paths below it.

| Request | Answer |
| ------- | ------ |
| `GET` and `HEAD` on a stored entry | 200 with the bytes, `Range` honoured; 404 for a missing entry or a directory path |
| `PUT` on an entry | 201 when the entry is new, 204 when it replaces one; 403 without the write right |
| `PROPFIND` with `Depth: 0` | 207. For a stored entry: its size, modification time and ETag. For any other valid path, the root included: a collection |
| `PROPFIND` with `Depth: 1` or `infinity`, or without `Depth` | 403: a listing is not offered |
| `MKCOL` | 201, and nothing is stored: directories are virtual |
| `DELETE` on an entry | 204; 404 when the entry is missing. It needs the delete right |
| `OPTIONS` | 200 with `DAV: 1` and `Allow: GET,HEAD,PUT,DELETE,PROPFIND,MKCOL,OPTIONS` |

Only these methods are supported; any other method is 405. The format has no listing, no locking, no `COPY` or `MOVE`,
and no proxy or group repositories, since a shared cache is the point of the format.

A path is a list of at most 16 segments of at most 512 characters in all, each made of letters, digits, `.`, `_` and
`-`; the segments `.` and `..` are refused. The client stores objects under `k[0]/k[1]/k[2]/key`, so a typical path has
four segments. A path outside these rules is 400, and 404 for `GET` and `HEAD`.

Entries are stored as `application/octet-stream` whatever their bytes, so a browser never runs an entry as a page. An
entry is stored as the asset `<repository>/<path>`, and its component is the last segment of the path.

A `PUT` to an existing entry replaces it, because the entries are deterministic and the last upload is as good as the
first. The creation wizard starts a hosted `sccache` repository with the `allow` write policy for this reason. A
repository created through REST must set `"writePolicy": "ALLOW"` itself. Under `allow_once` the first entry is kept, and
a repeated upload answers 204 without replacing it.

An administrator can delete an entry, and the key can then be uploaded again. Old entries go away with a cleanup policy
on the time they were last downloaded.

The client's start-up check reads `.sccache_check` with `GET`, writes it with `PUT`, and reads the root with `PROPFIND`
before it trusts the cache. A token without the write right gets a 403 on the `PUT`, and the client then works read-only.
