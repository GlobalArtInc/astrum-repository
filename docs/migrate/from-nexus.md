# Migrate from Nexus Repository 3

Astrum started from the Nexus Repository 3 code base, so the REST API, the repository settings and the
client URLs (`/repository/<name>/`) are the same. The internal database is not: Astrum does not open a
Nexus database. Migration therefore means standing up Astrum next to Nexus, recreating the
configuration through the REST API and copying the hosted content with each format's own protocol.

## What moves and what does not

| Item | How |
|---|---|
| Blob stores | Recreated empty with the same names |
| Repositories (hosted, proxy, group) and their settings | Recreated through the REST API with the same names, so client URLs only change host |
| Hosted content | Copied artifact by artifact (see below) |
| Proxy caches | Not copied; they fill again on first use |
| Roles, privileges, content selectors | Recreated through the REST API; built-in IDs change from `nx-...` to `as-...` (`nx-admin` becomes `as-admin`) |
| Local users | Recreated with new passwords: Nexus does not export password hashes |
| npm, NuGet and other tokens | Issued again for the new users |
| Cleanup policies, routing rules, scheduled tasks | Recreated |
| Realms, anonymous access, HTTP proxy, base URL | Set again; `NexusAuthenticatingRealm` becomes `AstrumAuthenticatingRealm` |
| Component and asset history (upload dates, uploaders) | Lost: copied artifacts get the migration time and the migration user |

## Procedure

1. **Inventory Nexus.** List what exists and how big it is:

   ```shell
   curl -u admin https://nexus.example.com/service/rest/v1/repositories
   curl -u admin https://nexus.example.com/service/rest/v1/blobstores
   curl -u admin https://nexus.example.com/service/rest/v1/security/roles
   curl -u admin https://nexus.example.com/service/rest/v1/security/users
   ```

   Note which hosted repositories hold content that cannot be rebuilt from CI.

2. **Install Astrum** on its own host name with PostgreSQL, and finish the onboarding wizard. Follow
   [container installation](../install/container.md) and [PostgreSQL](../configure/postgresql.md).

3. **Recreate the configuration** in this order: blob stores, content selectors, repositories
   (hosted first, then proxies, then groups), privileges, roles, users, realms, anonymous access,
   cleanup policies and tasks. For a repository the Nexus answer of
   `GET /service/rest/v1/repositories/<format>/<type>/<name>` is accepted as is by
   `POST /service/rest/v1/repositories/<format>/<type>` on Astrum.

4. **Copy hosted content**, format by format, reading from Nexus and writing to Astrum with the same
   tools that clients use:

   | Format | Copy with |
   |---|---|
   | Maven | download every asset listed by `/service/rest/v1/assets?repository=<name>` and `PUT` it to the same path |
   | raw | the same as Maven |
   | npm | download each version's tarball and publish it with `npm publish <tarball>`, or `PUT` the package document with the tarball attached |
   | Docker | `skopeo copy --all docker://nexus.example.com/<repo>/<image>:<tag> docker://astrum.example.com/<repo>/<image>:<tag>` for every tag |
   | Helm | download each chart `.tgz` and upload it with `curl --upload-file` |
   | NuGet | download each `.nupkg` and `dotnet nuget push` it |
   | PyPI | download each file and upload it with `twine` |
   | APT | download each `.deb` and post it to the hosted repository |

   Copying can run while Nexus is in use; run it again before the switch to pick up what was published
   in between. Compare the component counts per repository afterwards.

5. **Switch over.**
   - Put Nexus into read-only mode (`POST /service/rest/v1/read-only/freeze`) so nothing new lands
     there;
   - run the content copy once more;
   - point the host names or CI settings at Astrum and hand out the new credentials and tokens;
   - watch Astrum's request log and the CI pipelines for 401, 403 and 404 responses.

6. **Decommission Nexus** once nothing has requested it for a while. Keep its data directory until
   the first Astrum backup has been restored successfully at least once.
