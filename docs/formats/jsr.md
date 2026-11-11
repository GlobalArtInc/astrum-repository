# JSR (Deno and TypeScript)

`jsr` repositories serve packages in the layout of [JSR](https://jsr.io), the registry for Deno and TypeScript. A
hosted repository keeps your own packages, which you publish with `deno publish`; a proxy keeps a copy of
`https://jsr.io`, so `jsr:@std/path` works in a closed network; a group puts both behind one address, so a hosted
package can import `jsr:@std/*` through it. The format serves the registry that Deno reads; it does not serve the
website of JSR, search, documentation pages or the JSR command line tool.

Deno finds the registry through the environment variable `JSR_URL`. Give it the address of the repository with a
trailing slash:

```shell
export JSR_URL=https://astrum.example.com/repository/jsr-group/
```

Then `jsr:` specifiers resolve against the repository. A short name for a package goes into the import map of
`deno.json`:

```json
{
  "imports": {
    "@acme/hello": "jsr:@acme/hello@^0.1.0",
    "@std/path": "jsr:@std/path@^1"
  }
}
```

A repository that anonymous users may not read answers 401. Deno sends no credentials to it unless it is told to, so give
it an [access token](../configure/authentication.md#access-tokens) through `DENO_AUTH_TOKENS`, which holds the token
and the host; a read token is enough to import:

```shell
export DENO_AUTH_TOKENS=<access token>@astrum.example.com
```

## Publishing

Publishing needs a hosted repository, or a group with a writable hosted member, on a [domain of its own](../configure/reverse-proxy.md#a-domain-of-its-own-for-a-repository)
that uses the whole domain (`ROOT` routing). Deno takes the address in `JSR_URL`, replaces its path with `/api/` and
calls the publish API there, so the calls arrive at the root of the host and the address under `/repository/` cannot
take them. Reading works at either address. Send the domain to this server from the reverse proxy, then publish with an
access token that may add to the repository:

```shell
export JSR_URL=https://jsr.example.com/
deno publish --token <access token>
```

`deno.json` names the package and its version and lists the entry points:

```json
{
  "name": "@acme/hello",
  "version": "0.1.0",
  "license": "MIT",
  "exports": "./mod.ts"
}
```

`exports` is a path or an object of paths; every path must be a file of the package. The name has the form
`@scope/package`: a scope has 2 to 32 and a package 1 to 58 characters of lowercase letters, digits and `-`, and neither
starts with `-`. The version is a [semantic version](https://semver.org), with a prerelease and build metadata if you
like. Deno itself refuses to publish a package without a `license` field or a `LICENSE` file. The name and version in the
config must match the ones in the address, which Deno takes from the same file.

Publishing a package that does not exist yet needs no extra step: the server answers Deno's check for the package with
200 when the package exists or when the token may add to the repository, and creates the package with its first version.

- A version is never replaced. Publishing it again answers 400 with the code `duplicateVersionPublish`, which Deno
  prints as "already published", whatever the write policy of the repository and the rights of the token; two
  publishes of one version at the same time have one winner and one such answer. An administrator can delete a version
  and publish it again.
- A token needs the add right for every file of the package and for the version's `_meta.json`, so a content selector
  that allows only some paths refuses a package that reaches beyond them. A repository with the write policy "Deny"
  takes no publishes. Publishing to a proxy answers 405, and so does publishing to a group that has no writable member.
- Deno resolves the `jsr:` imports of a package at the same `JSR_URL` while it checks the package for publishing. A
  hosted repository holds only its own packages, so a package that imports `jsr:@std/*` is published through a group
  whose writable member is the hosted repository and that also holds a proxy of `https://jsr.io` (see below). The group
  needs a domain with `ROOT` routing for this, like a hosted repository.
- Interactive authorization (`deno publish` without `--token`) and publishing from GitHub Actions with OIDC are not
  supported. Create an access token and pass it with `--token`.

### The archive

Deno sends the package as a gzipped tar with `Content-Encoding: gzip`; the server keeps the bytes as they arrive. The
archive comes from a client and is read with limits, and a publish that breaks one is refused with 400 (413 for the
sizes) before anything is stored:

- at most 20 MiB compressed and 100 MiB expanded, counted while the archive is read, and at most 2000 entries;
- regular files only: a symbolic link, hard link, device or fifo is refused, and directory entries are skipped;
- no path that is absolute (one leading `/` or `./` is dropped first), has an empty segment or a `.` or `..` segment,
  contains a backslash, a NUL or another control character, is not UTF-8, is longer than 512 bytes, appears twice, or
  has a segment that starts with a letter and a colon (`C:`);
- one gzip stream and nothing after it: a body made of several gzip streams, or with data behind the archive, is refused.

Nothing in the archive is ever written to disk under its own name; the files wait in an unnamed temporary file, which
takes up to 100 MiB of disk for the time of one publish. The config file is the one the `config` query parameter
names, `/deno.json` by default, and it is read as JSON with comments: `//` and `/* */` comments, which do not nest and
which are left alone inside strings, trailing commas before `}` and `]`, and a byte order mark are accepted, so a
`deno.jsonc` works.

Publishing is limited in two steps, and the limits are shared by all JSR repositories of the server:

- Receiving: at most 32 packages are received at once. A slot is taken before the body is read and kept until the
  package is complete and the writing slot is won, so at most 32 compressed bodies (up to 20 MiB each) wait in memory
  besides those being written. The whole body must arrive within 5 minutes, or the answer is 408 and the slot is free
  again; a client that sends a byte at a time cannot hold a slot longer than that.
- Writing: at most eight packages are unpacked and stored at once, fewer when the database connection pool is small
  (half its size, at least one), so that publishes cannot use up the connections. The slot is taken once the body has
  been read and kept until the version is stored, which includes the unpacking, the unnamed temporary file of up to
  100 MiB and the lock on the version. A publish that waits more than 30 seconds for either slot is answered 503
  with `Retry-After`; publish again.

Once the body has been read, the write goes on even if the client disconnects, so a cancelled publish leaves either the
whole version or nothing. Only a crash of the server in the middle of a write can leave files without their `_meta.json`;
they are not listed or counted as a version, and the next publish of that version clears them.

A package may contain a file named `_meta.json` at its top: it is the file `@<scope>/<name>/<version>/_meta.json`, not
the metadata of the version, which is `@<scope>/<name>/<version>_meta.json`.

## What is served

| Path | Answer |
| ---- | ------ |
| `@<scope>/<name>/meta.json` | `scope`, `name`, `latest` (the highest version that is not a prerelease; left out when there is none) and `versions`, an object with a key for every published version |
| `@<scope>/<name>/<version>_meta.json` | The `manifest` (size and `sha256-` checksum of each file) and the `exports` of the version, written at publish time and never changed |
| `@<scope>/<name>/<version>/<file>` | The file, byte for byte as published, with a type taken from its extension |

The type of a file comes from its extension and never from its content: `.ts`, `.mts` and `.d.ts` are
`application/typescript`, `.js` and `.mjs` are `application/javascript`, `.tsx` is `text/tsx`, `.jsx` is `text/jsx`,
`.json` is `application/json`, and anything else is `application/octet-stream`. The answers support `Range` and
conditional requests. A caller sees only the versions whose `_meta.json` the caller may read.

An asset is stored as `<repository>/@<scope>/<name>/<version>/<file>`, and the files of one version are one component.

## Proxy

A proxy needs the remote `https://jsr.io`. It asks for `meta.json` with `Accept: application/json` and re-checks it
after the metadata max age of the repository; it asks for `<version>_meta.json` with `Accept: application/json` and for
files with `Accept: */*`, and keeps them until the content max age passes. It never asks for `text/html`, which would
make the remote return its web page. The proxy sends no credentials unless the repository's HTTP client settings name
them. It does not check that what the remote answers is the JSON or the file it asked for, so a page the remote returns
by mistake can be kept in the cache until its max age passes or an administrator deletes it.

## Group

The `meta.json` of a group is the union of the versions of its members; when two members hold the same version, the
first member in the group's order wins, and `latest` is worked out again over the union. Any other path comes from the
first member that has it. A package that a hosted member holds is never asked of the proxies in the group, so a proxy
cannot shadow or extend a package you publish.

A group takes publishes when it has a writable member, a hosted repository that is one of its members; set it with
`"writableMember"` next to `"memberNames"` over the REST API or in the group's settings. Deno's check for the package and
the publish itself go to that member and are stored there. The token needs the add right on the group, for the files of
the package and for the version's `_meta.json`, as for the other formats whose groups take writes; the right on the
member alone does not let a token publish through the group. A version the member already holds answers
`duplicateVersionPublish`. A group without a writable member answers 405. Everything else a group serves is unchanged,
so a package published through the group is read back from it at once.

```shell
export JSR_URL=https://jsr.example.com/
export DENO_AUTH_TOKENS=<access token>@jsr.example.com
deno publish --token <access token>
```

Here `jsr.example.com` is the domain of a group made of a hosted repository (its writable member) and a proxy of
`https://jsr.io`; the package may import `jsr:@std/path@1`.
