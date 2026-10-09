# nexus-migrate

Copies configuration and hosted content from a Nexus Repository 3 server (3.96 Community Edition)
into an Astrum Repository server. Astrum keeps the Nexus REST API, the repository settings JSON and the
`/repository/<name>/` client URLs, so the migration talks to both servers over HTTP only. No database
access is needed.

It is a single Python 3.9+ script. It uses only the standard library.

## Credentials

Passwords are read from files and are never accepted on the command line or printed:

```
--source-url https://nexus.example.com  --source-user admin  --source-password-file nexus.pw
--target-url https://astrum.example.com --target-user admin  --target-password-file astrum.pw
```

Use an administrator account on both sides. `--insecure` skips TLS verification, and `--timeout`
sets the socket timeout (default 300 s).

## Usage

```
./nexus-migrate.py inventory --source-url ... --source-password-file ... --output inventory.json
./nexus-migrate.py config    --source-url ... --target-url ... --new-passwords-file new-passwords.tsv [--dry-run]
./nexus-migrate.py content   --source-url ... --target-url ... [--repository NAME ...] [--workers 8] [--dry-run]
```

Every subcommand can be re-run. Anything that already exists on the target is skipped and reported.
If something fails, the exit code is non-zero.

### inventory

Describes the source:
- repositories, with their format, type and blob store; hosted repositories also get component and
  asset counts and their total size
- blob stores, local users, custom roles, privileges and content selectors
- active realms, anonymous access, cleanup policies and routing rules

The JSON goes to `--output`, and a readable summary is printed.

### config

Recreates the source configuration on the target in this order:

1. File blob stores. They are created under the target's blob directory with the store name as the
   path; `--keep-blob-store-paths` uses the source paths instead. S3 and group blob stores are reported
   and must be created by hand.
2. Content selectors.
3. Cleanup policies. The public `/v1/cleanup-policies` API is used when it exists; otherwise the
   tool falls back to the UI's `/internal/cleanup-policies`. On a re-run, policies that were missing
   earlier are attached to existing repositories.
4. Routing rules.
5. Repositories: hosted, then proxy, then group (nested groups last). Each repository is read with
   `GET /v1/repositories/<format>/<type>/<name>` and posted to `/v1/repositories/<format>/<type>`
   (`maven2` maps to `maven` in the path). Docker repositories get `docker.pathEnabled: true`, so
   Astrum serves them at `/v2/<repository>/`. Connector ports are dropped unless you pass
   `--keep-docker-ports`.
6. Custom privileges. A wildcard pattern `nexus:...` becomes `astrum:...`.
7. Custom roles, created in dependency order. References to built-in roles and privileges
   (`nx-admin`, `nx-repository-view-...`) are mapped to the target's `as-...` IDs, and so are the
   built-in roles of the users in step 8.
8. Local users other than `admin` and `anonymous`. Each one gets a new random password, appended
   to `--new-passwords-file` (mode 0600, `user<TAB>password` per line).
9. Anonymous access.
10. Active realms. `Nexus*` realms map to `Astrum*`; realms the target does not offer are dropped
    and reported. Realms already active on the target stay active.

`--dry-run` prints what would be created without changing anything.

### content

Copies hosted repositories: all of them, or only those named with `--repository` (repeatable).
`--workers` sets the number of parallel transfers. Files are streamed through temporary files in
`--temp-dir`, one per running transfer, and checksums are verified on the way. Anything already on
the target is skipped, so an interrupted run can be resumed.

| Format | How it is copied | Skipped when |
| --- | --- | --- |
| maven2, raw | every asset is PUT to the same path; Maven metadata is uploaded last | same path and sha1 exist |
| npm | each version is published with a PUT of a publish document: the manifest, the dist-tags pointing at it and the base64 tarball (streamed). Publish times are kept | the version exists |
| nuget | the `.nupkg` is pushed with a multipart PUT to `/repository/<name>/` | id and normalized version exist |
| helm | `.tgz` and `.tgz.prov` are PUT to `/repository/<name>/<file>` | same file name exists |
| pypi | legacy upload (`:action=file_upload`), with metadata read from the package and the `.asc` signature if there is one | same file name exists |
| docker | each tag is copied over the registry API (details below) | the tag has the same digest |
| apt | each `.deb` is POSTed to `/repository/<name>/` | same file name exists |

Other hosted formats are reported as not handled, with their counts.

For Maven, if any artifact still has no `maven-metadata.xml` after the copy, the tool creates a
"Rebuild Maven metadata" task for that repository on the target and runs it. The source's own
metadata files are then put back.

The Docker copy reads from `<source>/repository/<name>/v2/` and writes to `<target>/v2/<name>/`.
It handles Docker v2 and OCI manifests, manifest lists and OCI indexes (child manifests are copied
first). Before each upload it checks with HEAD whether the target already has the blob. Uploads go
through `POST .../blobs/uploads/` and then `PUT ...?digest=`. A `WWW-Authenticate: Bearer` challenge
is answered by requesting a token from the realm with Basic credentials.

At the end, each repository is listed with its component and asset counts on the source and the target.

## Recommended run

```
./nexus-migrate.py inventory ... --output before.json
./nexus-migrate.py config ... --dry-run
./nexus-migrate.py config ... --new-passwords-file new-passwords.tsv
./nexus-migrate.py content ... --workers 8 --temp-dir /var/tmp/migrate
./nexus-migrate.py content ... --workers 8 --temp-dir /var/tmp/migrate
```

The second `content` run copies whatever was added on the source in the meantime. Point clients at
the target once they have switched over.

## Not migrated

- Secrets that the API does not export:
  - proxy remote passwords: the authentication block is dropped, with a warning
  - apt signing key passphrases: a warning is printed
  - user passwords: users get new ones
  - user tokens, npm tokens and NuGet API keys
- LDAP, SAML and other non-local users; external role mappings only come over as roles.
- Tasks, capabilities (base URL, e-mail, HTTP proxy), scripts, webhooks, the SSL trust store and
  non-file blob stores.
- Content of proxy and group repositories (proxies refill from their remotes).
- Hosted formats other than the eight above.
- Asset metadata that the target sets on upload: blob creation time, uploader and last download time.
  For npm, the publish times are kept.
- Docker manifests that no tag reaches. NuGet unlisted state, PyPI yanked state.
