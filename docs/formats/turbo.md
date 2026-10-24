# Turborepo remote cache

Hosted `turbo` repositories are a remote cache for [Turborepo](https://turborepo.dev): `turbo run` uploads
the outputs of its tasks and restores them on other machines and in CI instead of running the tasks again.
The server answers the Remote Cache API under `<repository>/v8/artifacts` (and under `<repository>/artifacts`,
the paths of the specification); the format has no proxy or group repositories.

```shell
export TURBO_API=https://astrum.example.com/repository/turbo-cache
export TURBO_TOKEN=<access token>
export TURBO_TEAM=web
turbo run build
```

The address and the team can live in `turbo.json` instead; keep the token in `TURBO_TOKEN`:

```json
{
  "$schema": "https://turborepo.dev/schema.json",
  "remoteCache": {
    "apiUrl": "https://astrum.example.com/repository/turbo-cache",
    "teamSlug": "web",
    "signature": true
  }
}
```

Turborepo sends `TURBO_TOKEN` as `Authorization: Bearer`, so it has to be an
[access token](../configure/authentication.md#access-tokens): a read token only restores outputs, a write
token also uploads them. Uploading a new artifact takes the add privilege of the repository, replacing one
the edit privilege. `turbo login` and `turbo link` talk to Vercel and are not needed.

The team (`TURBO_TEAM`, or `teamSlug`, or a `teamId` that starts with `team_`) keeps the caches of teams
apart: each artifact is stored as `<repository>/<team>/<hash>`, and requests without a team use the team
`_`. A team name holds letters, digits, `.`, `-` and `_`. Content selectors on these paths give teams
privileges of their own within one repository. The stored artifacts can also be downloaded at those paths,
which is what the repository browser links to.

With `"signature": true` and `TURBO_REMOTE_CACHE_SIGNATURE_KEY` set, Turborepo signs what it uploads and
refuses downloads without a valid signature. The server keeps the signature (`x-artifact-tag`), the task
duration and the commit that the client reports, and returns them with the artifact; it does not check
signatures itself. Old artifacts go away with a cleanup policy on the time they were last downloaded.
