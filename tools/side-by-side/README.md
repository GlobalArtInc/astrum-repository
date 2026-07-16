# side-by-side

Runs the same client scenarios against two repository servers, a reference and Astrum Repository (the
candidate), and reports where their answers differ. It was the check of M5: before production moved from
the Java server to the Rust server on 2026-10-18, every difference was either fixed or written down in
[`allow-list.json`](allow-list.json) with its reason. Since the Java server is gone, the reference is
another repository server whose data is being moved
([`docs/migrate/from-another-server.md`](../../docs/migrate/from-another-server.md)), or a Java Astrum
server built from the history.

A single Python 3.9+ program using the standard library, plus the real clients where they are installed:
curl, mvn (3.9 or later), npm, pip, gem and bundle, git and git-lfs, docker (the tool starts its own
`dockerd`). Formats without an installed client (NuGet, Helm, Terraform, Swift, Yum) are driven with the
exact requests their clients send.

## Two modes

```
./side-by-side.py run      --candidate-url http://127.0.0.1:18600 \
    --reference-password-file ref.pw --candidate-password-file cand.pw
./side-by-side.py existing --reference-url https://old.example --candidate-url https://new.example \
    --reference-password-file ref.pw --candidate-password-file cand.pw [--repository 'maven-*'] [--max-assets 200]
./side-by-side.py list
```

- `run` creates the same repositories, users, roles, privileges, selectors and routing rules on both
  servers (every name starts with `--prefix`, default `m5s-`), runs the scenarios and deletes what it
  created; `--keep` leaves the repositories and their content in place (useful as migration test data).
  It writes to both servers, so it must not be pointed at production.
- `existing` only reads. It compares what both servers already hold: the configuration and security
  listings, then for every repository present on both (hosted and group by default, `--types` adds
  `proxy`) its configuration, components, assets, a download of each asset (size and SHA-256, streamed;
  `--max-assets 0` for all of them), an anonymous download, a HEAD and the format's index documents (Maven
  metadata, npm package documents, PyPI simple pages, Docker tags and manifests, Helm index, NuGet flat
  container, RubyGems compact index, Yum repomd, Terraform and Swift version lists). Repositories of
  formats the candidate does not serve are reported as kept aside and left out of the other listings. This
  is the mode for a server running on migrated data; it compared production during the switch.

Both servers are given as base URL, user (default `admin`) and a password file; passwords are never taken on
the command line or printed. The same values can come from `SBS_REFERENCE_URL`, `SBS_REFERENCE_USER`,
`SBS_REFERENCE_PASSWORD_FILE`, `SBS_CANDIDATE_URL`, `SBS_CANDIDATE_USER` and `SBS_CANDIDATE_PASSWORD_FILE`.
`--insecure` skips TLS verification, `--timeout` sets the socket timeout.

## Scenarios of `run`

| Scenario | What it does |
|---|---|
| `rest` | Public REST API: repositories, settings, components, assets, search (by name, wildcard, q, format, checksum, sort, download), blob stores, users, roles, privileges, realms, anonymous access, SSRF settings, content selectors, routing rules, tasks, status, upload specifications, HTTP, email, capabilities; creating, changing and deleting objects, and invalid requests |
| `access` | Anonymous callers, a wrong password, an unknown user, a malformed Basic header and an unknown Bearer token against REST and content URLs; a user with read rights on one repository; a disabled user; repeated failed sign-ins; missing paths, path traversal, unknown methods |
| `raw`, `raw-proxy` | Write policies, strict content types, ranges, conditional requests (ETag, dates, If-Match), groups, curl; a proxy of a local upstream |
| `maven`, `maven-proxy` | `mvn deploy:deploy-file` of releases, snapshots and classifiers, `mvn dependency:get` through a group, metadata and checksums, layout policies, REST search by Maven keys including the download of the latest snapshot |
| `npm`, `npm-proxy` | `npm publish`, `view`, `dist-tag`, `deprecate`, `install`, `unpublish`, `ping`, `whoami`, `search`; package documents, abbreviated documents, tarballs |
| `pypi`, `pypi-proxy` | Uploads as twine sends them (including a wrong digest), simple pages in HTML and PEP 691 JSON, PEP 658 metadata, `pip download` |
| `docker` | Registry API v2 and OCI 1.1 requests (monolithic and chunked uploads, manifests, indexes, referrers, tags with paging, catalog, mounts, deletes), then the docker CLI: login, push, pull, logout, anonymous pull |
| `helm`, `nuget`, `rubygems`, `gitlfs`, `yum`, `terraform`, `swift` | The protocol of each format: uploads, index documents, downloads, deletes; `gem push/install/yank` and `bundle lock`; `git lfs` push, clone, pull and locks |

Proxy scenarios need both servers to allow proxies to 127.0.0.1 (their SSRF protection refuses it by
default). `--allow-loopback reference --allow-loopback candidate` adds `127.0.0.1` to the allow list of
that server while the scenarios run and restores the previous settings afterwards; without it the proxy
scenarios are skipped. The upstream they proxy is a small HTTP server of the tool on `--port-base`
(default 18610); the recording proxies in front of the servers take the next 89 ports.

Real clients get their own HOME, caches and configuration under the work directory (`--work-dir`, deleted
afterwards unless `--keep-work`). They talk only to the two servers and the local upstream, with a user the
tool creates for the run (`<prefix>client`, random password, deleted at the end). The wrong-password probes
use a user of their own as well: both servers slow down a user after a few failed sign-ins, and the
Java server keeps refusing even the right password for a while.

## What is compared

Each scenario runs on the reference first, then on the candidate, and every step is compared by name:

- the status code;
- the headers clients rely on: Content-Type (media type and parameters, `charset=utf-8` ignored),
  Content-Length and ETag (exactly when both bodies are the same bytes, otherwise only whether they are
  present), Last-Modified (present or not), Location (after normalization), Docker-Content-Digest,
  Docker-Distribution-Api-Version, Docker-Upload-UUID (present), WWW-Authenticate (scheme), Content-Range,
  Accept-Ranges, Content-Disposition, Allow, Range, OCI-Subject, OCI-Filters-Applied, X-Error-Code, Link;
- the body, depending on the step: the same bytes; normalized text; JSON compared as data (lists matched
  by `name`, `path`, `id` and similar keys, order ignored unless the step says otherwise); the shape or the
  key structure of JSON; gunzipped bytes; format documents reduced to their facts (Maven metadata, Helm
  index entries, PyPI links, Yum repomd);
- for real clients, the exit code and what they produced, and every request the client made through a
  recording proxy in front of the server, step by step.

Before comparing, base URLs and host names become `<base>` and `<host>`, and UUIDs, ISO and HTTP dates,
Maven timestamps and epoch milliseconds become placeholders; opaque ids and continuation tokens are not
compared, nor are download counters in `run` mode. JSON keys that only the candidate sends are not
differences (clients ignore unknown keys); `--strict-json` shows them. Error answers (4xx and 5xx on both
sides) are compared by status only, unless a step compares error codes (Docker).

## Report and allow list

The report lists each scenario with its number of steps that are equal, that differ only in allowed ways,
and that differ otherwise, and prints every unexpected difference with both values (`--show-allowed` also
prints the allowed ones, `--max-lines` limits the lines per step). `--report-json` writes everything,
including all differences and the open entries, as JSON. The exit code is 0 when no unexpected difference
is left, 1 otherwise, 2 when the tool could not run.

Each entry of `allow-list.json` matches differences by step id (`<scenario>/<step>`), field
(`status`, `header.<name>`, `body...`, `value...`, `exists`) and optionally the two values, with `*` as the
only wildcard, and must give a `reason`. `status` is `accepted` (decided, nothing to do) or `open` (known,
listed in the report for the owner to decide). `mode` limits an entry to `run` or `existing`.

## Adding a scenario

A scenario is a function decorated with `@scenario(name)` in `sbs/scenarios/<module>.py` that receives a
trial `t` and runs the same code against either server: `t.create_repository(...)`, `t.step(name, method,
path, ..., spec=Spec(...))`, `t.json_step(...)`, `t.fact(name, value)`, `t.run(name, argv)` for a client,
and `with t.recording(name) as base:` to point a client at a recording proxy. Step names must not depend on
the server (no timestamps or ids in them).
