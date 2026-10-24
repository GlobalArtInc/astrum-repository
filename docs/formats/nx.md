# Nx remote cache

Hosted `nx` repositories are a self-hosted remote cache for [Nx](https://nx.dev) 20.8 and later: Nx uploads
the outputs of its tasks and restores them on other machines and in CI instead of running the tasks again.
The server answers the custom remote cache API of Nx, `GET` and `PUT` on `<repository>/v1/cache/<hash>`;
the format has no proxy or group repositories.

```shell
export NX_SELF_HOSTED_REMOTE_CACHE_SERVER=https://astrum.example.com/repository/nx-cache
export NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN=<access token>
npx nx run-many -t build
```

Nx sends the token as `Authorization: Bearer`, so it has to be an
[access token](../configure/authentication.md#access-tokens): a read token only restores outputs, a write
token also uploads them, and Nx skips the upload quietly when the token may not write. A stored output is
never replaced: a second upload of the same hash answers 409, which Nx expects. Leave the workspace
unconnected to Nx Cloud (no `nxCloudId` or `nxCloudAccessToken` in `nx.json`), or Nx uses the cloud
instead. Each output is stored as `<repository>/<hash>` and can be downloaded there, which is what the
repository browser links to; old outputs go away with a cleanup policy on the time they were last
downloaded.
