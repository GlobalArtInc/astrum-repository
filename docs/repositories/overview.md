# Repositories

A repository is a named place that clients read packages from and publish them to. It has a format
(Maven, npm, Docker, raw and so on), a type (hosted, proxy or group) and a blob store, and it is served
at `https://astrum.example.com/repository/<name>/`. How each client is pointed at a repository is on the
page of its format, see [formats](../formats/README.md).

The content of a repository is made of components and assets. A component is one version of a package,
named by its group or namespace, its name and its version; an asset is one file. Most assets belong to a
component, while indexes and other metadata files usually belong to none.

Manage repositories under **Settings → Repository → Repositories**, or with the REST API under
`/service/rest/v1/repositories`.

## Types

| Type | What it does |
|---|---|
| Hosted | Keeps what is published to it: your own builds and packages |
| Proxy | Fetches content from a remote repository, such as Maven Central or the npm registry, on first request and keeps a copy, so builds keep working while the remote is slow or down |
| Group | Puts several repositories of one format, hosted and proxy, behind one URL; it holds no content of its own |

Point clients at a group for reading and at a hosted repository for publishing.

## Creating a repository

Click **Create Repository**, choose the format and the type, and fill in the configuration. The name,
the format, the type and the blob store cannot be changed later. To change a repository, open it from
the list; the settings are on its **Settings** tab.

A name may hold letters, digits, `-`, `_` and `.`, must not start with `_` or `.`, and is at most 200
characters long. Names are unique without regard to case: `Maven-Releases` and `maven-releases` cannot
both exist.

## Settings of every repository

| Setting | Meaning |
|---|---|
| **Online** | Whether clients can reach the repository. An offline repository answers `503`, and groups skip it. **Take Offline** and **Bring Online** in the list of repositories do the same |
| **Blob Store** | Where the files of the repository are kept, see [blob stores](../configure/blob-stores.md). A store that is a member of a blob store group cannot be chosen; choose the group. To give a repository more space later, [convert its store into a group](../configure/blob-stores.md#growing-a-store-that-repositories-use) |
| **Strict Content Type Validation** | Checks that a file uploaded to the repository is what its name says: the type detected from its first bytes must fit the type its extension stands for, or the upload is refused with `400`. Files a proxy fetches are not checked |
| **Domain** | An optional domain of its own for the repository, see [reverse proxy](../configure/reverse-proxy.md#a-domain-of-its-own-for-a-repository) |

Formats add sections of their own, for example **Maven 2**, **Docker Registry API Support** or the
signing keys of APT, Yum, Alpine and pacman repositories. They are described on the page of each format.

## Hosted repositories

**Deployment Policy** decides what may be published:

| Deployment Policy | REST value | Effect |
|---|---|---|
| **Allow Redeploy** | `ALLOW` | New files are stored and existing files can be replaced |
| **Disable Redeploy** | `ALLOW_ONCE` | New files are stored; replacing a file that exists is refused with `409`. The default |
| **Read-Only** | `DENY` | Every upload is refused |

A hosted repository can have [cleanup policies](cleanup-policies.md).

## Proxy repositories

A proxy answers a request from its copy while that copy is fresh, and otherwise asks the remote. When it
checks a file again, it sends the `ETag` and `Last-Modified` of its copy, so an unchanged file is not
downloaded again. When the remote does not answer, is blocked, answers with an error or no longer has
the file, the proxy serves the copy it has, however old.

**Proxy** section:

| Setting | Meaning |
|---|---|
| **Remote Storage** | The `http://` or `https://` address of the remote repository |
| **Use the Astrum Repository Truststore** | Shown for an `https://` remote: trust the certificates under **Settings → Security → SSL Certificates** in addition to those of the system, for a remote with a private CA or a self-signed certificate. **View Certificate** shows the certificate of the remote |
| **Blocked** | Never connect to the remote; the proxy serves only what it has |
| **Auto Blocking** | After the remote fails (no answer, a `5xx` or `429` answer), stop asking it for 40 seconds, twice as long after each further failure up to 20 minutes; the first good answer ends the block. On by default |
| **Maximum Component Age** | Minutes a cached file counts as fresh before the remote is asked again |
| **Maximum Metadata Age** | The same for metadata, such as indexes and version lists, which change when new versions appear |

The format decides which files are components and which are metadata. An age of `-1` keeps a copy for
good: the remote is not asked again until the cache is invalidated. The form suggests 1440 minutes (a
day) for both, and `-1` for the components of Maven and Nix proxies, whose files do not change once
published.

**Negative Cache** section: with **Not Found Cache** on, a proxy remembers for **Negative Cache TTL
(minutes)** that the remote answered `404` or `410` for a file it does not have, and answers not found
without asking again. A TTL of `0` remembers nothing. The negative cache is kept in memory and is
cleared when the settings of the repository are saved or its cache is invalidated.

**Routing Rule** section: a [routing rule](routing-rules.md) keeps some requests away from the proxy,
for example the names of your own packages. **None (allow every request)** is the default.

**HTTP** section:

| Setting | Meaning |
|---|---|
| **Authentication Type** | **Username** sends a user name and password with Basic authentication; **Bearer token** sends a token as `Authorization: Bearer`. Credentials are sent only to the scheme, host and port of the remote URL, never to other hosts the remote sends the proxy to. Passwords and tokens are stored sealed and never shown again; leave the field empty to keep the stored one |
| **Authenticate Preemptively** | Maven proxies only: send the user name and password with the first request. Without it a Maven proxy sends them after the remote answers `401`; other proxies always send them |
| **User-Agent Suffix** | Text added to the `User-Agent` header, which starts with `Astrum/<version>` |
| **Connection Retries** | How often to retry a request that could not connect or timed out, 0 to 10 |
| **Connection Timeout** | Seconds to wait for the connection and for data, 1 to 3600 |

The last three are under **HTTP Request Settings**. Left empty, retries and timeout come from
**Settings → System → HTTP**, which defaults to 2 retries and 60 seconds; that page also holds the
outbound proxy the server uses. The server refuses a remote URL on an internal address unless the SSRF
protection allows it, see [runtime configuration](../configure/runtime.md#outbound-http-proxy).

A proxy can have [cleanup policies](cleanup-policies.md) that delete cached components; they are
fetched again on the next request.

**Invalidate Cache** (in the list of repositories and on the repository page) makes a proxy check every
cached file with the remote on its next request, whatever its age, and clears the negative cache and an
automatic block. On a group it applies to the group alone, not to its members.

## Group repositories

**Member Repositories** lists the members in order; **Move Up** and **Move Down** change it. Members must
be repositories of the same format. A group can be a member of another group, but not of itself, directly
or through other groups.

A group answers a request by asking its members in order. A member that is a group is expanded in place,
so its members are asked at that position. Members that are offline, and members whose routing rule
blocks the path, are skipped; a member that fails is passed over for the next one. For a file, the first
member that has it answers, so put hosted repositories before proxies to make sure your own packages win.
For documents that list versions, such as `maven-metadata.xml` of Maven or the package documents of npm,
the group merges what its members return.

Publishing goes to hosted repositories. Only Cargo, Docker and Swift groups take uploads: there
**Writable Repository** names a hosted member that receives what is published to the group. Without
one, publishing to the group is refused.

A group has no cleanup policies; attach them to its members.

## REST API

| Method and path | Use |
|---|---|
| `GET /service/rest/v1/repositories` | The repositories you may see, with their URL and size |
| `GET /service/rest/v1/repositorySettings` | The full settings of every repository you may administer |
| `POST /service/rest/v1/repositories/<format>/<type>` | Create a repository; answers `201` |
| `GET`, `PUT /service/rest/v1/repositories/<format>/<type>/<name>` | Read or replace the settings; `PUT` answers `204` |
| `DELETE /service/rest/v1/repositories/<name>` | Delete the repository |
| `POST /service/rest/v1/repositories/<name>/invalidate-cache` | Invalidate the cache of a proxy or group |

`<format>` is the format as the server names it, for example `raw`, `npm` or `docker`; Maven is
`maven`. `<type>` is `hosted`, `proxy` or `group`. Create a Maven proxy and a group in front of it:

```sh
curl -u admin -X POST https://astrum.example.com/service/rest/v1/repositories/maven/proxy \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "maven-central",
    "online": true,
    "storage": { "blobStoreName": "default", "strictContentTypeValidation": true },
    "proxy": {
      "remoteUrl": "https://repo1.maven.org/maven2/",
      "contentMaxAge": -1,
      "metadataMaxAge": 1440
    },
    "negativeCache": { "enabled": true, "timeToLive": 1440 },
    "httpClient": { "blocked": false, "autoBlock": true },
    "maven": { "versionPolicy": "RELEASE", "layoutPolicy": "STRICT" }
  }'

curl -u admin -X POST https://astrum.example.com/service/rest/v1/repositories/maven/group \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "maven-public",
    "online": true,
    "storage": { "blobStoreName": "default", "strictContentTypeValidation": true },
    "group": { "memberNames": ["maven-releases", "maven-central"] }
  }'
```

A hosted repository takes `"writePolicy"` in `storage`. The sections of a proxy read:

```json
"httpClient": {
  "blocked": false,
  "autoBlock": true,
  "connection": { "retries": 3, "timeout": 120, "userAgentSuffix": "ci", "useTrustStore": true },
  "authentication": { "type": "username", "username": "reader", "password": "<password>" }
},
"routingRuleName": "no-internal-groups",
"cleanup": { "policyNames": ["proxy-unused-90d"] }
```

`authentication` takes `"type": "bearerToken"` with `"bearerToken"` in place of the user name and
password. A Cargo, Docker or Swift group takes `"writableMember"` next to `"memberNames"`.

To change a repository, read its settings, change them and send the whole document back with `PUT`.
Passwords, tokens and signing keys are never returned; left out, the stored ones are kept.

## Browsing and deleting content

**Browse** in the main menu lists the repositories you may browse. Open one to see its content as a
tree; a group shows the content of its members. Select a file or a component to see its details (path,
size, content type, who uploaded it, when it was created, updated and last downloaded, and how often) and
the usage snippets for its format.

With the delete privilege of a repository (`as-repository-view-<format>-<repository>-delete`), the
details offer:

- **Delete File** deletes one asset. A component whose last file goes is deleted with it.
- **Delete Component** deletes the component with all its files.
- **Delete Folder** deletes every file below the folder that you may delete; files you may not delete
  stay and are counted.

Content cannot be deleted through a group; delete it in the member that holds it. Deleting is final and
deletes the files in the blob store at once. The HTML index of a repository, for scripts and plain
browsers, is described in [connect build tools](../use/clients.md).

Over REST, find components with `GET /service/rest/v1/search` (parameters such as `repository`, `name`
and `version`) or list them with `GET /service/rest/v1/components?repository=<name>`, both paged with
`continuationToken`; assets are listed with `GET /service/rest/v1/assets?repository=<name>`. Delete them
by their `id`:

```sh
curl -u admin 'https://astrum.example.com/service/rest/v1/search?repository=maven-releases&name=app&version=1.0.0'
curl -u admin -X DELETE https://astrum.example.com/service/rest/v1/components/<id>
curl -u admin -X DELETE https://astrum.example.com/service/rest/v1/assets/<id>
```

## Deleting a repository

**Delete** in the list of repositories or on the repository page asks you to type the name of the
repository, then:

- a hosted repository is deleted with every component and asset published to it, and their files are
  deleted from the blob store;
- a proxy is deleted with everything it cached; the remote is not touched;
- a group is deleted with its address; its members and their content stay.

A deleted repository is also taken out of every group it was a member of, and out of the **Writable
Repository** of a group. There is no undo: restore from a [backup](../operate/backup-restore.md) to get a
deleted repository back. Files that could not be deleted from the blob store are removed later by the
task that compacts blob stores, see [reclaiming space](../configure/blob-stores.md#reclaiming-space).
