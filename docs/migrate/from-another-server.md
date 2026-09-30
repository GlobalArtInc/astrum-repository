# Move from another repository server

Astrum can take over from another repository server that offers the same REST API under
`/service/rest/v1/`, the same repository settings JSON and the same client URLs (`/repository/<name>/`),
for example an older Astrum installation. Astrum does not read the other server's database. Moving
therefore means standing up Astrum next to the old server, recreating the configuration through the REST
API and copying the hosted content with each format's own protocol.

## What moves and what does not

| Item | How |
|---|---|
| Blob stores | Recreated empty with the same names |
| Repositories (hosted, proxy, group) and their settings | Recreated through the REST API with the same names, so client URLs only change host |
| Hosted content | Copied artifact by artifact (see below) |
| Proxy caches | Not copied; they fill again on first use |
| Roles, privileges, content selectors | Recreated through the REST API; built-in IDs are mapped to Astrum's (`as-admin`, `as-repository-view-...`) |
| Local users | Recreated with new passwords: the REST API does not export password hashes |
| npm, NuGet and other tokens | Issued again for the new users |
| Cleanup policies, routing rules, scheduled tasks | Recreated |
| Realms, anonymous access, HTTP proxy, base URL | Set again; realms Astrum does not offer are left out |
| Component and asset history (upload dates, uploaders) | Lost: copied artifacts get the migration time and the migration user |

## Procedure

1. **Inventory the old server.** List what exists and how big it is:

   ```shell
   curl -u admin https://old.example.com/service/rest/v1/repositories
   curl -u admin https://old.example.com/service/rest/v1/blobstores
   curl -u admin https://old.example.com/service/rest/v1/security/roles
   curl -u admin https://old.example.com/service/rest/v1/security/users
   ```

   Note which hosted repositories hold content that cannot be rebuilt from CI.

2. **Install Astrum** on its own host name with PostgreSQL, and finish the onboarding wizard. Follow
   [container installation](../install/container.md) and [PostgreSQL](../configure/postgresql.md).

3. **Recreate the configuration** in this order: blob stores, content selectors, repositories
   (hosted first, then proxies, then groups), privileges, roles, users, realms, anonymous access,
   cleanup policies and tasks. For a repository the old server's answer of
   `GET /service/rest/v1/repositories/<format>/<type>/<name>` is accepted as is by
   `POST /service/rest/v1/repositories/<format>/<type>` on Astrum.

4. **Copy hosted content**, format by format, reading from the old server and writing to Astrum with the same
   tools that clients use:

   | Format | Copy with |
   |---|---|
   | Maven | download every asset listed by `/service/rest/v1/assets?repository=<name>` and `PUT` it to the same path |
   | raw | the same as Maven |
   | npm | download each version's tarball and publish it with `npm publish <tarball>`, or `PUT` the package document with the tarball attached |
   | Docker | `skopeo copy --all docker://old.example.com/<repo>/<image>:<tag> docker://astrum.example.com/<repo>/<image>:<tag>` for every tag |
   | Helm | download each chart `.tgz` and upload it with `curl --upload-file` |
   | NuGet | download each `.nupkg` and `dotnet nuget push` it |
   | PyPI | download each file and upload it with `twine` |
   | APT | download each `.deb` and post it to the hosted repository |

   Copying can run while the old server is in use; run it again before the switch to pick up what was published
   in between. Compare the component counts per repository afterwards.

5. **Switch over.**
   - put the old server into read-only mode (`POST /service/rest/v1/read-only/freeze`) so nothing new
     lands there;
   - run the content copy once more;
   - point the host names or CI settings at Astrum and hand out the new credentials and tokens;
   - watch Astrum's request log and the CI pipelines for 401, 403 and 404 responses.

6. **Decommission the old server** once nothing has requested it for a while. Keep its data directory until
   the first Astrum backup has been restored successfully at least once.

## With the copy tool

[`tools/repository-copy`](../../tools/repository-copy/README.md) does steps 3 and 4 for the formats it knows.
A run with the Java Astrum 2.0.0 as the source and Astrum 3 as the target (2026-10-18: 42 repositories, 109
configuration objects, 39 MiB of hosted content) showed what to expect:

- **Proxies of internal hosts.** Astrum 3 checks every proxy's remote URL against its SSRF protection
  when the repository is created, so proxies of hosts on private networks fail in the `config` step
  until their addresses are in the allow list (`PUT /service/rest/v1/security/ssrf-protection`). Add
  them before `config` and run it again; repositories that already exist are skipped.
- **Remote credentials.** Passwords and bearer tokens of proxies are not exported by the source API. The
  tool drops the authentication block with a warning (for bearer tokens too since 2026-10-18; Astrum 3
  refuses a bearer-token block without its token) and the credentials are set again by hand.
- **Routing rules** of proxies are carried over: Astrum 3 accepts the rule as `routingRule`, as the
  tool sends it, as well as `routingRuleName`.
- **Cleanup policies that keep the newest versions** (`retain`, `sortBy`) are refused by Astrum 3,
  which does not have that criterion yet; recreate them with age and download criteria or keep them on the
  list of open differences.
- **Maven.** Astrum 3 derives `maven-metadata.xml`, the checksum files and `archetype-catalog.xml` on every
  read instead of storing them, so the REST API of the target lists fewer assets than the source (compare
  component counts, not asset counts). The tool skips these files when the target already serves them and
  does not need the "Rebuild Maven metadata" task, which Astrum 3 does not have.
- **Formats.** The tool copies maven2, raw, npm, nuget, helm, pypi, docker and apt content. The
  content of its other formats (alpine, cargo, cocoapods, composer, conan, conda, go, p2, r, yum, rubygems, gitlfs,
  terraform, swift and huggingface) is not copied by the tool.
- **Docker** connector ports are dropped as before (`--keep-docker-ports` keeps them; Astrum 3 opens a
  listener for each), and only tagged manifests are copied.

Check the result with the side-by-side harness in read-only mode, which compares configuration, listings,
downloads and index documents of both servers without writing to either:

```shell
tools/side-by-side/side-by-side.py existing \
  --reference-url https://old.example.com --reference-password-file old.pw \
  --candidate-url https://astrum.example.com --candidate-password-file astrum.pw
```

Its report lists the differences that are expected after this kind of migration (assets the target
derives, credentials set by hand, connector ports) and everything else.
