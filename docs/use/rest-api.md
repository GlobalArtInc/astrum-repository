# REST API

Everything the web interface manages can also be done over HTTP with JSON: repositories, components,
search, blob stores, users and roles, tasks and the settings of the server. The API lives under
`/service/rest/v1/` on the same address as the web interface and the repositories, and it checks the
same users, privileges and audit log. The examples use `astrum.example.com`; `ASTRUM_TOKEN` holds an
[access token](../configure/authentication.md#access-tokens) of the user `ci-deployer`.

## Where it lives

| Path | What it is |
|---|---|
| `/service/rest/v1/...` | The REST API described on this page |
| `/service/rest/swagger.json` | The OpenAPI 3 document of the API; it needs no sign-in |
| `/service/rest/repository/browse/<repository>/` | A plain HTML index of a repository, with download links |
| `/repository/<repository>/...` | Repository content for build tools, see [connect build tools](clients.md) |
| `/service/metrics/prometheus` | Prometheus metrics, see [monitoring](../operate/monitoring.md#prometheus-metrics) |
| `/mcp` | The Model Context Protocol endpoint for agents, see [agents over MCP](mcp.md) |

Paths under `/service/rest/internal/` belong to the web interface. They are not in the OpenAPI document,
and scripts should use the `/v1` paths instead.

## The API page

**Settings → System → API**, also **API** in the main menu, renders the OpenAPI document with Swagger UI:
the operations grouped by area, a filter, the request and response models, and **Try it out** to send a
request from the browser. Requests sent from this page run as the signed-in user, with the session of the
browser. The page needs the privilege `astrum:settings:read` (`as-admin` has it).

The document lists most of the `/v1` operations. The tables at the end of this page also list routes that
are not in it yet: capabilities, cleanup policies, remote and group blob stores, LDAP, SSL certificates and
the system information.

## Authentication

A client signs in on every request; there is no separate login call.

- **Basic authentication** with a user name and password: `curl -u ci-deployer:<password>`. This works for
  local users and for LDAP users with their directory password.
- **Access tokens**: as the password of Basic authentication (`curl -u "ci-deployer:$ASTRUM_TOKEN"`), or as
  `Authorization: Bearer astrum_...`. Users who sign in through OpenID Connect have no password and use a
  token. A token with the access **Read only** is allowed only reading and browsing; anything that changes
  the server needs a **Read and write** token. See [access tokens](../configure/authentication.md#access-tokens).

Wrong credentials are answered `401`, also when anonymous access is on: a request with credentials never
falls back to the anonymous user. The tokens that agents get through OAuth for `/mcp` are refused
everywhere else.

Every endpoint checks the privileges of the user, for example `astrum:tasks:read` to list tasks and
`astrum:tasks:start` to run one; the role `as-admin` has all of them. Repositories and their content are
checked per repository, so a user sees only the repositories, components and assets they may browse or
read.

### Requests from the web interface

The web interface signs in once and keeps a session in the cookie `astrum_session`. A request that relies
on that cookie and changes something (any method other than `GET`, `HEAD` and `OPTIONS`) must carry the
header `X-Astrum-CSRF` with the value of the cookie `astrum_csrf`, and is refused with `403` when the
browser marks it as coming from another site. Scripts that send their credentials in `Authorization` need
neither.

## Anonymous access

While **Settings → Security → Anonymous Access** allows it, requests without credentials run as the
anonymous user with the rights of its role (`as-anonymous` by default): they can list the repositories and
search the content that role may read. When anonymous access is off, such requests are answered `401` with
the challenge `WWW-Authenticate: Basic realm="Astrum Repository"`. See
[anonymous access](../configure/authentication.md#anonymous-access).

The OpenAPI document, `/v1/status` and `/v1/status/writable` answer without credentials either way. The
upload specifications under `/v1/formats` need a signed-in user.

## Requests and responses

- Request bodies are JSON with `Content-Type: application/json`; a body without that header is answered
  `415`. Uploads to `/v1/components` are `multipart/form-data`.
- Responses are JSON. Times are in RFC 3339, such as `2026-10-24T09:15:02.118+00:00`.
- Addresses in responses, such as `url` and `downloadUrl`, start with the
  [base URL](../configure/runtime.md#base-url) when one is set, and with the address of the request
  otherwise.

## Pagination

The lists of components and assets and the search results come in pages:

```json
{
  "items": [ ... ],
  "continuationToken": "b2Zmc2V0OjUw"
}
```

Pass the token back as the query parameter `continuationToken` to get the next page, with the other
parameters unchanged. The last page has `"continuationToken": null`. A page holds 100 items for
`/v1/components` and `/v1/assets` and 50 for the search endpoints. The token is opaque; one that the server
cannot read is answered `400`. `/v1/tasks` uses the same shape but returns every task at once.

## Errors

Most errors are answered with JSON:

| Status | When | Body |
|---|---|---|
| `400` | The request is not valid | A list of problems, each with the field in `id` (`*` when it concerns the whole request) and a `message`: `[{"id": "PARAMETER remoteUrl", "message": "..."}]` |
| `401` | No valid credentials | `{"id": "*", "message": "authentication is required"}` and a `WWW-Authenticate` challenge |
| `403` | The user lacks the privilege | `{"id": "*", "message": "you do not have the permission this needs"}` |
| `404` | The repository, component or other object does not exist, or the user may not see it | `{"id": "*", "message": "..."}`; a path that is not a route at all answers `404` with an empty body |
| `409` | The request conflicts with the current state, such as running a task that already runs | `{"id": "*", "message": "..."}` |
| `415` | A JSON body without `Content-Type: application/json` | `{"id": "*", "message": "..."}` |
| `500` | The server failed | `{"id": "*", "message": "the server failed; the log names it <reference>"}`; search the [log](../operate/monitoring.md#logs) for the reference |

Some errors that come from the content of a repository, such as a few refusals of an upload and the
errors of the HTML index, are plain text with the message instead.

## Examples

### List repositories

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" https://astrum.example.com/service/rest/v1/repositories
```

```json
[
  {
    "name": "maven-releases",
    "format": "maven2",
    "type": "hosted",
    "url": "https://astrum.example.com/repository/maven-releases",
    "online": true,
    "attributes": {},
    "size": 73400320
  }
]
```

The list holds the repositories the user may see; proxies carry their remote address in
`attributes.proxy.remoteUrl`, and `size` is the space their files take, in bytes. The complete settings of
one repository are at `/v1/repositories/<format>/<type>/<name>`, for example
`/v1/repositories/maven/hosted/maven-releases`; see [repositories](../repositories/overview.md) for what
they hold. In these paths Maven is called `maven`, every other format by its name in the `format` field.

### Search components and assets

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" \
  "https://astrum.example.com/service/rest/v1/search?repository=maven-releases&maven.groupId=com.example&name=my-library"
```

```json
{
  "items": [
    {
      "id": "bWF2ZW4tcmVsZWFzZXM6MmE",
      "repository": "maven-releases",
      "format": "maven2",
      "group": "com.example",
      "name": "my-library",
      "version": "1.0.0",
      "assets": [
        {
          "downloadUrl": "https://astrum.example.com/repository/maven-releases/com/example/my-library/1.0.0/my-library-1.0.0.jar",
          "path": "/com/example/my-library/1.0.0/my-library-1.0.0.jar",
          "id": "bWF2ZW4tcmVsZWFzZXM6MmI",
          "repository": "maven-releases",
          "format": "maven2",
          "checksum": { "md5": "...", "sha1": "...", "sha256": "...", "sha512": "..." },
          "contentType": "application/java-archive",
          "fileSize": 18342,
          "lastModified": "2026-10-24T09:15:02.118+00:00",
          "lastDownloaded": null
        }
      ]
    }
  ],
  "continuationToken": null
}
```

The assets carry more fields than shown here, such as the uploader, the blob store and the attributes of
their format. `/v1/search/assets` takes the same parameters and returns the matching assets on their own,
in the shape of the `assets` above.

| Parameter | Matches |
|---|---|
| `q` | A keyword: components whose name, group or version, some descriptive attributes of their format, or the path of one of their files contain it, ignoring case. With several words separated by spaces, a component matches when any of them does |
| `repository` | One repository; a group searches its members |
| `format` | The format, such as `maven2`, `npm` or `docker` |
| `group`, `name`, `version` | The group, name or version, ignoring case; `*` stands for any characters, so `version=1.2.*` finds every 1.2 version |
| `md5`, `sha1`, `sha256`, `sha512` | An asset with this checksum |
| `<format>.<attribute>` | An attribute of the format, such as `maven.groupId`, `maven.extension`, `npm.scope` or `docker.imageTag`; `*` works here too, and an empty value matches where the attribute is missing or empty |
| `sort`, `direction` | `name`, `version`, `group` or `repository`, and `asc` or `desc`; without `sort` the results come in the order they were stored |
| `continuationToken` | The next page |

The **Advanced Search** page of the web interface (`/search/custom`) lists the format attributes that can be
searched.

### Download the file a search finds

`/v1/search/assets/download` takes the same parameters and redirects (`302`) to the download address of
the asset when exactly one matches, or to the first one when `sort` is given. Several matches without
`sort` are answered `400`, none `404`:

```shell
curl -L -u "ci-deployer:$ASTRUM_TOKEN" -o my-library.jar \
  "https://astrum.example.com/service/rest/v1/search/assets/download?repository=maven-releases&maven.groupId=com.example&maven.artifactId=my-library&version=1.0.0&maven.extension=jar&maven.classifier="
```

The empty `maven.classifier` leaves out the `sources` and `javadoc` jars of the same version.

### Upload a component

`POST /v1/components?repository=<name>` takes a multipart form and uploads to a hosted repository, the
same way as the **Upload** page. It answers `204` when the component is stored. The fields are named after
the format: component fields as `<format>.<field>`, each file as `<format>.asset<N>` and the fields of that
file as `<format>.asset<N>.<field>`.

A raw file:

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" \
  -F raw.directory=/tools \
  -F raw.asset1=@build/tool-1.4.0.tar.gz \
  -F raw.asset1.filename=tool-1.4.0.tar.gz \
  "https://astrum.example.com/service/rest/v1/components?repository=raw-hosted"
```

A Maven jar with a generated POM:

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" \
  -F maven2.groupId=com.example -F maven2.artifactId=my-library -F maven2.version=1.0.0 \
  -F maven2.generate-pom=true \
  -F maven2.asset1=@target/my-library-1.0.0.jar -F maven2.asset1.extension=jar \
  "https://astrum.example.com/service/rest/v1/components?repository=maven-releases"
```

When a file with the extension `pom` and no classifier is part of the upload, the coordinates are read
from it instead. `GET /v1/formats/upload-specs` lists the fields of every format that takes uploads, and
`GET /v1/formats/<format>/upload-specs` those of one; the [page of each format](../formats/README.md)
describes its own.

### Delete a component

Take the `id` of the component from a search or from `/v1/components?repository=<name>`:

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" -X DELETE \
  https://astrum.example.com/service/rest/v1/components/bWF2ZW4tcmVsZWFzZXM6MmE
```

The answer is `204`. Deleting a component deletes all its assets, and needs the right to delete every one
of them. `DELETE /v1/assets/<id>` deletes a single file.

### Run a task

Look the task up by its type, then start it by its `id`:

```shell
curl -u "ci-deployer:$ASTRUM_TOKEN" \
  "https://astrum.example.com/service/rest/v1/tasks?type=repository.cleanup"

curl -u "ci-deployer:$ASTRUM_TOKEN" -X POST \
  https://astrum.example.com/service/rest/v1/tasks/<id>/run
```

The run is queued and the answer is `204`. A task that already runs or is about to run is answered `409`,
a disabled one `405`. `GET /v1/tasks/<id>` shows `currentState`, `lastRun` and `lastRunResult`; see
[tasks](../operate/tasks.md) for the task types.

### Status and health

```shell
curl -fs https://astrum.example.com/service/rest/v1/status/writable
```

The status endpoints and what they check are described in [monitoring](../operate/monitoring.md).

## Endpoints

Paths are relative to `/service/rest`.

### Repositories

| Method and path | What it does |
|---|---|
| `GET /v1/repositories` | The repositories the user may see, with format, type, address, state and size |
| `GET /v1/repositories/{name}` | The summary of one repository, without its size |
| `DELETE /v1/repositories/{name}` | Deletes the repository and everything in it |
| `POST /v1/repositories/{format}/{type}` | Creates a repository; `{type}` is `hosted`, `proxy` or `group` |
| `GET /v1/repositories/{format}/{type}/{name}` | The complete settings of a repository |
| `PUT /v1/repositories/{format}/{type}/{name}` | Changes the settings of a repository |
| `POST /v1/repositories/{name}/invalidate-cache` | Proxy and group repositories: marks the cached content so that it is checked against the remote again |
| `POST /v1/repositories/{name}/rebuild-index` | Accepted for hosted and proxy repositories; on a proxy it marks the cached content like `invalidate-cache` |
| `GET /v1/repositorySettings` | The complete settings of every repository the user may read the settings of |

### Components and assets

| Method and path | What it does |
|---|---|
| `GET /v1/components?repository={name}` | The components of a repository, 100 a page |
| `POST /v1/components?repository={name}` | Uploads a component to a hosted repository from a multipart form |
| `GET /v1/components/{id}` | One component with its assets |
| `DELETE /v1/components/{id}` | Deletes a component and its assets |
| `GET /v1/assets?repository={name}` | The assets of a repository, 100 a page |
| `GET /v1/assets/{id}` | One asset |
| `DELETE /v1/assets/{id}` | Deletes an asset |
| `GET /v1/formats/upload-specs` | The upload fields of every format that takes uploads |
| `GET /v1/formats/{format}/upload-specs` | The upload fields of one format |

### Search and browse

| Method and path | What it does |
|---|---|
| `GET /v1/search` | Searches components, 50 a page |
| `GET /v1/search/assets` | Searches assets, 50 a page |
| `GET /v1/search/assets/download` | Redirects to the download address of the asset a search finds |
| `GET /repository/browse/{name}/` | An HTML index of a folder of the repository, with download links |

### Blob stores

| Method and path | What it does |
|---|---|
| `GET /v1/blobstores` | The blob stores with their type, blob count, size, free space and soft quota |
| `DELETE /v1/blobstores/{name}` | Deletes a blob store |
| `GET /v1/blobstores/{name}/quota-status` | Whether the store is over its soft quota |
| `POST /v1/blobstores/file` | Creates a file blob store |
| `GET`, `PUT /v1/blobstores/file/{name}` | Reads or changes a file blob store |
| `POST /v1/blobstores/s3` | Creates an S3 blob store |
| `GET`, `PUT /v1/blobstores/s3/{name}` | Reads or changes an S3 blob store |
| `POST /v1/blobstores/test-connection` | Tries the connection settings of an S3 blob store |
| `POST /v1/blobstores/remote` | Creates a remote blob store |
| `GET`, `PUT /v1/blobstores/remote/{name}` | Reads or changes a remote blob store |
| `POST /v1/blobstores/group` | Creates a group blob store |
| `GET`, `PUT /v1/blobstores/group/{name}` | Reads or changes a group: its members and fill policy |
| `POST /v1/blobstores/group/convert/{name}/{newNameForOriginal}` | Turns a store that repositories use into a group of the same name, with the store under its new name as the only member |

See [blob stores](../configure/blob-stores.md).

### Cleanup policies and routing rules

| Method and path | What it does |
|---|---|
| `GET`, `POST /v1/cleanup-policies` | Lists or creates cleanup policies |
| `GET`, `PUT`, `DELETE /v1/cleanup-policies/{name}` | Reads, changes or deletes a cleanup policy |
| `GET`, `POST /v1/routing-rules` | Lists or creates routing rules |
| `GET`, `PUT`, `DELETE /v1/routing-rules/{name}` | Reads, changes or deletes a routing rule |

See [cleanup policies](../repositories/cleanup-policies.md) and [routing rules](../repositories/routing-rules.md).

### Users, roles and privileges

| Method and path | What it does |
|---|---|
| `GET /v1/security/users` | Lists users; `source` picks the user source and `userId` filters by ID |
| `POST /v1/security/users` | Creates a local user |
| `PUT`, `DELETE /v1/security/users/{userId}` | Changes or deletes a user |
| `PUT /v1/security/users/{userId}/change-password` | Sets the password of a local user |
| `GET /v1/security/user-sources` | The user sources |
| `GET`, `POST /v1/security/roles` | Lists or creates roles |
| `GET /v1/security/roles/assignable` | The roles the user may grant to others |
| `GET`, `PUT`, `DELETE /v1/security/roles/{id}` | Reads, changes or deletes a role |
| `GET /v1/security/privileges` | Lists privileges |
| `POST /v1/security/privileges/{type}` | Creates a privilege of the type `application`, `wildcard`, `repository-view`, `repository-admin` or `repository-content-selector` |
| `PUT /v1/security/privileges/{type}/{name}` | Changes a privilege |
| `GET`, `DELETE /v1/security/privileges/{name}` | Reads or deletes a privilege |
| `GET`, `POST /v1/security/content-selectors` | Lists or creates content selectors |
| `GET`, `PUT`, `DELETE /v1/security/content-selectors/{name}` | Reads, changes or deletes a content selector |

See [users, roles and privileges](../configure/authentication.md#users-roles-and-privileges).

### Sign-in and security settings

| Method and path | What it does |
|---|---|
| `GET /v1/security/realms/available` | The realms the server knows |
| `GET`, `PUT /v1/security/realms/active` | The active realms, in the order they are tried |
| `GET`, `PUT /v1/security/anonymous` | The anonymous access settings |
| `GET`, `PUT /v1/security/ssrf-protection` | Which addresses outgoing requests of the server may not reach |
| `GET`, `POST /v1/security/ldap` | Lists or adds LDAP servers |
| `GET /v1/security/ldap/templates` | Presets of user and group settings for common directories |
| `POST /v1/security/ldap/change-order` | Sets the order the LDAP servers are tried in |
| `DELETE /v1/security/ldap/cache` | Drops the cached LDAP users and sign-ins |
| `POST /v1/security/ldap/verify-connection` | Tries the connection settings of an LDAP server |
| `POST /v1/security/ldap/verify-user-mapping` | Lists the first users and groups the settings find |
| `POST /v1/security/ldap/verify-login` | Tries the password of a user against the settings |
| `GET`, `PUT`, `DELETE /v1/security/ldap/{name}` | Reads, changes or deletes an LDAP server |
| `GET /v1/security/ssl?host={host}&port={port}` | Fetches the certificate a server presents |
| `GET`, `POST /v1/security/ssl/truststore` | Lists or adds trusted certificates |
| `DELETE /v1/security/ssl/truststore/{id}` | Removes a trusted certificate |

See [users, realms and sign-in](../configure/authentication.md) and [SSL certificates](../configure/ssl-certificates.md).

### System

| Method and path | What it does |
|---|---|
| `GET`, `PUT /v1/http` | The settings of outbound HTTP: user agent, timeout, retries and proxies |
| `GET`, `PUT`, `DELETE /v1/email` | The mail server settings; `DELETE` removes them |
| `POST /v1/email/verify` | Sends a test message |
| `GET`, `POST /v1/capabilities` | Lists or creates capabilities |
| `GET /v1/capabilities/types` | The capability types and their fields |
| `GET`, `PUT`, `DELETE /v1/capabilities/{id}` | Reads, changes or deletes a capability |
| `GET /v1/tasks` | Lists tasks; `type` filters by task type |
| `POST /v1/tasks` | Creates a task |
| `GET /v1/tasks/templates` | The task types with their fields and defaults |
| `GET /v1/tasks/templates/{type}` | One task type |
| `GET`, `PUT`, `DELETE /v1/tasks/{id}` | Reads, changes or deletes a task |
| `POST /v1/tasks/{id}/run` | Starts a task |
| `POST /v1/tasks/{id}/stop` | Stops a running task |

See [outbound HTTP](../configure/outbound-http.md), [capabilities](../configure/capabilities.md) and
[tasks](../operate/tasks.md).

### Status and support

| Method and path | What it does |
|---|---|
| `GET /v1/status` | Whether the database answers |
| `GET /v1/status/writable` | Whether the server accepts writes |
| `GET /v1/status/check` | The named health checks |
| `POST /v1/support/supportzip` | Builds a support ZIP and returns it |
| `POST /v1/support/supportzippath` | Builds a support ZIP in the data directory and returns its path |
| `GET /atlas/system-information` | Version, node, directories, runtime, operating system, database and blob stores of the server; needs `astrum:atlas:read` |
| `GET /swagger.json` | The OpenAPI document |

See [monitoring and logs](../operate/monitoring.md).
