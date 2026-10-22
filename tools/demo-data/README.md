# Demo data

`seed.py` fills a server with demo content over the REST API, for a test or demo server such as
https://test.astrum.globalart.hu. It needs only Python 3.9 or later and changes nothing that is already
there: a repository, role or cleanup policy with the same name is left as it is, and uploads that the
repository already holds are reported as existing.

```shell
ASTRUM_URL=https://test.astrum.globalart.hu ASTRUM_USER=admin ASTRUM_PASSWORD=... python3 tools/demo-data/seed.py
```

`ASTRUM_BLOB_STORE` names the blob store of the new repositories (`default` when unset). The user needs
the rights of an administrator.

What it creates:

- hosted, proxy and group repositories for npm (`npm-hosted`, `npm-proxy` of registry.npmjs.org,
  `npm-group`), PyPI (`pypi-hosted`, `pypi-proxy` of pypi.org, `pypi-group`) and Maven (`maven-releases`,
  `maven-snapshots`, `maven-central` of Maven Central, `maven-public`);
- `raw-hosted`, `helm-hosted`, `nuget-hosted`, `docker-hosted`, and proxies of nuget.org
  (`nuget.org-proxy`), Docker Hub (`docker-hub`), the Go module proxy (`go-proxy`) and crates.io
  (`crates-io`);
- the remote caches `turbo-cache` and `nx-cache`, with the cleanup policy `caches-unused-30-days`;
- small packages made on the fly: files in `raw-hosted`, `@demo/hello` and `demo-utils` in npm,
  `demo-hello` in PyPI, `com.example:demo-lib` in Maven, the chart `demo-app`, `Demo.Hello` in NuGet, the
  image `demo/hello` in `docker-hosted`, and an artifact in each cache;
- the role `developers`, which reads everything and publishes to the hosted repositories.

At the end it asks the proxies for a few well-known packages, so that they hold some cached content when
the server can reach the internet; a proxy that cannot is reported and left empty.
