# Inventory: what the server does today

Behaviour and contracts of Astrum Repository 2.0.0 (main at f9b3bc36), surveyed for the Rust rewrite.
Describes behaviour only; do not port the Java code (see `README.md` in this folder). Paths:
`C/` = `public/common/components/`, `F/` = `C/formats/astrum-repository-`, REST under `/service/rest`.

## Frame
- One edition (CORE); no licensing code. Single node only (clustering/HA flags are dead remnants).
- Storage: PostgreSQL (production) or embedded H2; ~130 migration steps.
- The upstream-compatible REST API is a contract: `tools/nexus-migrate/nexus-migrate.py` and
  `docs/migrate/from-nexus.md` use it, and a repository JSON from `GET /v1/repositories/<fmt>/<type>/<name>`
  is accepted unchanged by the matching `POST`.
- Every repository is served under `/repository/<name>/` (`C/astrum-repository-httpbridge`).

## Repository JSON shared by all formats
`name`, `online`, `storage{blobStoreName, strictContentTypeValidation, writePolicy ALLOW|ALLOW_ONCE|DENY}`,
`cleanup{policyNames}`, `component{proprietaryComponents}`; proxy: `proxy{remoteUrl, contentMaxAge,
metadataMaxAge, preserveEncodedCharacters}`, `negativeCache{enabled, timeToLive}`, `httpClient{blocked,
autoBlock, connection{retries, timeout, userAgentSuffix, enableCircularRedirects, enableCookies,
useTrustStore}, authentication{type username|ntlm|bearerToken, ...}}`, `routingRuleName`; group:
`group{memberNames[, writableMember]}`. `firewall` and `replication` are stored but inert.

## Formats (23)
H = hosted, P = proxy, G = group; Upload = components API (`POST /v1/components?repository=`).

| Format | Recipes | Client protocol under `/repository/<name>/` | Format attributes | Upload | Auth |
|---|---|---|---|---|---|
| alpine | H P G | `{path}/APKINDEX.tar.gz` (hosted generates + signs, group merges), `*.apk`, `*.rsa.pub` | `alpineSigning{keypair, passphrase}` | yes | basic |
| apt | H P | `dists/<dist>/{Release,InRelease,Packages,by-hash}`, `pool/...`, snapshots `/snapshots/<id>/...`; metadata rebuilt (debounced) | hosted `apt{distribution}`, proxy `apt{distribution, flat, enforceDistribution}`, `aptSigning` | yes | basic |
| cargo | H P G | sparse index (`config.json`, `1/a`, `2/ab`, `3/a/abc`, `ab/cd/...`), `api/v1/crates/{n}/{v}/download`, `PUT api/v1/crates/new`, yank/unyank, search, owners, `/me` | `cargo{requireAuthentication}`, group `writableMember` | no | `CargoToken` |
| cocoapods | P | CDN trunk: `CocoaPods-version.yml`, `all_pods*.txt`, `deprecated_podspecs.txt`, `Specs/...podspec.json`, `pods/...` source tarballs | – | no | basic |
| composer | H P G | `packages.json`, `p/` `p2/{vendor}/{project}.json`, `packages/list.json`, dist zips, `PUT packages/upload/...` | – | yes | basic |
| conan | H P G | `v1`/`v2` ping, users/authenticate, check_credentials, `v1/conans/...`, `v2/conans/.../revisions/.../files` | proxy `conanProxy{conanVersion V1|V2}` | no | `ConanToken` |
| conda | H P G | `{subdir}/repodata.json[.bz2|.zst]`, `current_repodata.json`, `channeldata.json`, packages; group merges | – | yes | basic |
| docker | H P G | Registry v2 `/v2/*` (`_catalog`, `tags/list`, `manifests`, `blobs`, `blobs/uploads`, `referrers`), token `/v2/token`; reached by path routing `/v2/<repo>/<image>`, per-repo HTTP/HTTPS port, or `/repository/<name>/v2/...`; Docker and OCI media types (OCI 1.1 work in progress) | `docker{forceBasicAuth, httpPort, httpsPort, subdomain (stored, not implemented), pathEnabled}`, `dockerProxy{indexType HUB|REGISTRY|CUSTOM, indexUrl, cacheForeignLayers, foreignLayerUrlWhitelist}`, hosted `storage.latestPolicy`, group `writableMember` | no | `DockerToken` |
| gitlfs (ours) | H | `info/lfs/objects/batch`, `objects/{oid}`, `locks`, `locks/verify`, `locks/{id}/unlock` | – | yes | basic, access token |
| go | H P G | GOPROXY `{module}/@v/list`, `@v/{ver}.{info,mod,zip}`, `@latest` | – | yes | basic |
| helm | H P | `index.yaml` (hosted regenerates), `*.tgz`, `*.tgz.prov`, `PUT`, `POST api/charts`, `DELETE` | – | yes | basic |
| huggingface (ours) | P | `api/{models,datasets,spaces}/{repo}[/revision/{rev}]`, tree, paths-info, `[datasets/|spaces/]{repo}/resolve/{rev}/{file}`; cached by commit; token only to the Hub | (bearer token in httpClient) | no | basic in `HF_ENDPOINT` |
| maven2 | H P G | Maven 2 layout, `maven-metadata.xml`, checksums, `archetype-catalog.xml`, `.index/`; group merges metadata; default repos `maven-releases`, `maven-snapshots`, `maven-central`, `maven-public` unless `astrum.skipDefaultRepositories` | `maven{versionPolicy RELEASE|SNAPSHOT|MIXED, layoutPolicy STRICT|PERMISSIVE, contentDisposition}`, proxy auth `preemptive` | yes | basic |
| npm | H P G | packument `/{pkg}`, `/@scope/pkg`, tarballs, publish/unpublish (`PUT` with `_attachments`), `-/package/.../dist-tags`, `-/v1/search`, `-/all`, `-/npm/v1/security/audits[/quick]`, `-/ping`, `-/whoami`, login `-/user/org.couchdb.user:<n>`, `-/user/token/{t}` | proxy `npm{removeQuarantined, removeNonCataloged}` (inert) | no | `NpmToken` |
| nuget | H P G | V3 `index.json`, registration, content, query, autocomplete, catalog; V2 OData `$metadata`, `Packages()`, `FindPackagesById()`, `Search()`, push `api/v2/package`; Chocolatey, `.snupkg` | proxy `nugetProxy{nugetVersion V2|V3, queryCacheItemMaxAge}` | yes | `NuGetApiKey` |
| p2 | P | `p2.index`, `content.*`, `artifacts.*`, `composite*`, `features/`, `plugins/`, `binary/` | – | no | basic |
| pypi | H P G | `simple/`, `simple/{name}/` (`data-requires-python`), `packages/...`, twine upload `POST /` | – | yes | basic |
| r | H P G | `{path}/PACKAGES[.gz|.rds]`, files, `PUT`; group merges | – | yes | basic |
| raw | H P G | any path `GET/PUT/DELETE`, Range | `raw{contentDisposition, forwardQueryParameters, excludedQueryParameters}` | yes (multi-file) | basic |
| rubygems (ours) | H P G | compact index `versions`, `info/<gem>`, `names`; `specs.4.8.gz` family, `quick/Marshal.4.8/*.gemspec.rz`, `gems/*.gem`, `api/v1/dependencies`, `POST api/v1/gems`, `DELETE api/v1/gems/yank`, `api/v1/api_key` | – | yes | `RubyGemsApiKey` |
| swift (ours) | H P G | SE-0292: `/{scope}/{name}`, `/{scope}/{name}/{ver}`, `.zip`, `Package.swift`, `identifiers?url=`, `login`, `availability`, `PUT` publish | group `writableMember` | yes | basic, access token |
| terraform (ours) | H P G | `.well-known/terraform.json`, `v1/modules/...` (download 204 + `X-Terraform-Get`), `v1/providers/...`, network mirror `index.json`/`{ver}.json`, `token`; root discovery per registry host; signed download links (15 min) | `terraform{requireAuthentication, registryHost, upstreamHost}`, hosted `terraformSigning` | yes | `TerraformToken` |
| yum (ours) | H P G | `[.../]repodata/repomd.xml[.asc|.key]`, `repodata/*`, `*.rpm`; hosted rebuilds repodata (2 s debounce); group merges | hosted `yum{repodataDepth, deployPolicy STRICT|PERMISSIVE}`, `yumSigning` | yes | basic |

Cross-format: HTML directory browse `GET /service/rest/repository/browse/{repo}/{path}`; old Nexus 2 URLs
only via `LegacyUrlCapability` (off). Not yet built: pub, Ansible Galaxy (planned directly in Rust).

## Public REST API (`/v1`, ~600 methods; U = used by Astrum One)
- Repositories: `GET /repositories` U, `GET/DELETE /repositories/{name}` U, `POST /{name}/invalidate-cache`
  and `/rebuild-index` U, `GET/DELETE /{name}/browse` U, `POST /repositories/{fmt}/{hosted|proxy|group}`
  and `GET/PUT .../{name}` U for every format, `GET /repositorySettings` U, `GET /formats/upload-specs` U.
- Components/assets: `GET/POST /components`, `GET/DELETE /components/{id}`, `GET /assets`,
  `GET/DELETE /assets/{id}` (U).
- Search: `GET /search` U, `/search/assets` U, `/search/assets/download`, `/search/repositories` U,
  `/search/versions` U, `/search/suggest`; params `q repository format group name version prerelease sort`,
  checksums, format keys (`maven.groupId`, `npm.scope`, `nuget.id`, `yum.*`, `rubygems.*`, `swift.*`,
  `terraform.*`, `huggingface.*`, ...).
- Security: users (`GET/POST`, `PUT/DELETE /{id}`, `/{id}/change-password`), `user-sources`, roles
  (+`assignable`), privileges (CRUD per type: application, wildcard, repository-view, repository-admin,
  repository-content-selector, script), realms (`available`, `active`), anonymous, content-selectors,
  LDAP (ours: CRUD, `change-order`, `cache`, `templates`, three verify calls), `PUT /security/jwt`,
  `GET/PUT /security/ssrf-protection` (`enabled`, `allowedIPs`, `allowedDomains`), secrets/apikeys
  re-encrypt.
- Blob stores: `GET /blobstores`, `DELETE /{name}`, `/{name}/quota-status`, `file` and `s3` CRUD,
  `test-connection`. No group REST (the UI calls a non-existent one).
- Routing rules CRUD; tasks (`GET/POST`, `templates[/{typeId}]`, `GET/PUT/DELETE /{id}`, `/{id}/run`,
  `/{id}/stop`); capabilities CRUD + types; `POST /cleanup/run`, `GET /cleanup/run/{id}`; email
  (`GET/PUT/DELETE`, `verify`); `GET/PUT /http` (ours: proxies, nonProxyHosts, auth, timeout, retries,
  user agent); scripts (Groovy, creation off by default); status (`/status`, `/status/writable`,
  `/status/check`); system (`/system/node`, `/lifecycle/*`); support (`/support/supportzip[path]`,
  `/atlas/system-information`, `/wonderland/authenticate`, `/wonderland/download/{file}`); metrics
  (`/metrics/{data,ping,threads,prometheus}`); `/swagger.{json,yaml}`; deprecated `/beta/*` aliases.
- Not present: licensing, audit REST, user tokens, public cleanup-policy CRUD.

## Internal UI API
`internal/ui/repositories` (list, details, filtered, recipes, repository/{n}, signing-passphrase),
`internal/cleanup-policies` (CRUD, `{name}/repositories`, `criteria/formats`, previews, CSV),
`internal/ui/blobstores` (+types, quotaTypes, usage), privileges/types, roles, roles/sources,
realms/types, anonymous-settings, content-selectors/preview, routing-rules (+preview, test), user
(+password), access-tokens (ours), oauth2 (OIDC), notifications (ours, + test), webhooks, onboarding,
upload/{repo}, status-check, loggingConfiguration, `internal/logging/logs`, usage-dashboard (ours),
audit-log (+export), api/permissions, security/access-check, docker/suggest-port, nuget-api-key.
Session: `POST/DELETE /service/rapture/session` (base64 form; sets JWT cookie `ASTRUMSESSIONID`; access
tokens refused). ExtDirect still used: `rapture_Security.getPermissions/authenticationToken`,
`coreui_Browse.read`, `coreui_Component.*`, `coreui_Repository.readReferences*`, `coreui_Blobstore.read*`,
`coreui_Role.read`, `coreui_Webhook.*`, `node_NodeAccess.nodes`, `ssl_Certificate.*`, `ssl_TrustStore.*`
(SSL has no REST API). Other web endpoints: `/oidc/login`, `/oidc/callback`, `/static/rapture/i18n-<lang>.js`.

## Security
- Realms: local users (`AstrumAuthenticatingRealm`), authorizing realm, `LdapRealm` (ours; several
  servers, 5 min cache, dynamic/static groups), `OAuth2Realm` (OIDC with discovery, PKCE or confidential,
  groups claim → roles), `AccessTokenRealm` (ours; `astrum_` + 40 chars, SHA-256 stored, scope READ/WRITE,
  expiry 1–3650 days or never, ≤100 per user, used as password, not for UI login), `DefaultRole` (capability), format
  token realms (`DockerToken`, `NpmToken`, `NuGetApiKey`, `ConanToken`, `CargoToken`, `TerraformToken`,
  `RubyGemsApiKey`).
- Anonymous `{enabled, userId=anonymous, realmName}`; built-in roles `as-admin`, `as-anonymous`.
- Privileges: application (domain + actions, e.g. `astrum:users:read`), wildcard (Shiro-style string),
  `repository-view` (`astrum:repository-view:<fmt>:<repo>:<browse|read|edit|add|delete|*>`, auto-created per
  repository), `repository-admin`, `repository-content-selector`, `script`; ~45 built-in `as-*` privileges.
  Roles nest privileges and roles and can map external LDAP/OIDC groups.
- Content selectors (CSEL): identifiers `format`, `path`; operators `== != =~ =^ && ||`, parentheses,
  string literals; evaluated in memory and compiled to SQL for search/browse.
- Sessions: JWT cookie (HMAC secret from the secret store, claims user/realm/userSessionId), refreshed by
  a filter, revocation table checked per request (logout, password change, disable/delete), expiry from
  `astrum.jwt.expiry` or UI `sessionTimeout` (0 = never, our default).
- CSRF: unsafe methods with a session need header `AS-ANTI-CSRF-TOKEN` = cookie; `Sec-Fetch-Site` must be
  same-origin or none. Login rate limiting per user; SSRF protection on outbound requests (metadata IPs
  always blocked, allowlist); `X-Astrum-UI: true` suppresses the Basic challenge; X-Frame-Options, HSTS.

## Tasks
Schedules: manual, once, hourly, daily, weekly, monthly, cron, now; `alertEmail`,
`notificationCondition`; per-run log `log/tasks/`. User-visible types: `repository.cleanup`,
`repository.purge-unused`, `repository.rebuild-index`, `create.browse.nodes`,
`repository.trim-browse-tree`, `assetBlob.cleanup`, `blobstore.compact`, `blobstore.delete-temp-files`,
`blobstore.metrics.reconcile`, `h2.backup.task`, `security.purge-api-keys`,
`security.secrets.re-encrypt`, `script`, `repository.maven.{rebuild-metadata, remove-snapshots,
purge-unused-snapshots, publish-dotindex, unpublish-dotindex, repair-base-version}`,
`repository.yum.rebuild.metadata`, `repository.apt.rebuild.metadata`, `external.blobstore.metadata`.
Hidden: automatic yum/apt rebuilds, `repository.search.update`, `audit.events.cleanup` (90 days),
`tasklog.cleanup` (30 days), version normalisation, index and secret migrations. Docker uploads expire
after 24 h (no GC task).

## Blob stores
File (path; default `<data>/blobs/default`; soft delete + compact), S3 (region, bucket, prefix,
expiration, keys/role/session token, SSE or KMS, endpoint, signer, path style, pool size, pre-signed
redirects, failover buckets), Group (members + fill policy writeToFirst|roundRobin; ExtDirect only),
quotas (space remaining / used, soft), metrics in the DB.

## Other features
- Cleanup policies: `lastBlobUpdated`, `lastDownloaded` (days) and regex for all formats; `isPrerelease`
  for maven2, nuget, yum, apt, rubygems; keep latest N (`retain` + `sortBy`) for yum, apt, terraform,
  rubygems (maven2/docker behind flags); preview list, CSV and totals (ours); many repositories per policy.
- Routing rules (ALLOW/BLOCK path regexes on proxies, preview, test).
- Capabilities that matter: `baseurl`, `rapture.settings` (title, sessionTimeout, timeouts, status
  intervals), `rapture.branding` (header/footer HTML), `defaultrole`, `audit` (retention), `webhook.global`
  (audit/repository, url, HMAC secret), `webhook.repository` (component/asset), `http.forwarded`,
  `StorageSettings` (`lastDownloadedInterval`), `browse.trim`, `scheduling.scheduler`, `node.identity`,
  `onboarding-wizard`, `customs3regions`, `LegacyUrlCapability`. In the rewrite these become plain settings.
- Email (test mail, task failure mail), global HTTP settings, SSL truststore (fetch certificate from host).
- Audit: DB table + `log/audit/audit.log`; domains security.*, repository, component, asset, blobstore,
  capability, cleanupPolicy, ContentSelector, RoutingRule, email, httpclient, logging, tasks, script,
  system.notifications, database-migration.
- Notifications (ours): telegram, slack (also Mattermost/Rocket.Chat), webhook (`X-Astrum-Event`,
  `X-Astrum-Delivery`, `X-Astrum-Signature: sha256=<HMAC>`); events component published/updated/deleted;
  repository filter, language, proxy global/none/custom; debounce 10 s, max 60 s; retries on 429/5xx.
- Download counters and usage dashboard (ours): in-memory counter flushed to DB; `astrum.downloads.*`,
  `astrum.usage.cacheSeconds`.
- i18n: cookie `astrum-language` → Accept-Language → English; server texts English only.
- Onboarding: change admin password (deletes `admin.password`), anonymous access.
- Health checks: CPU, scheduler, upgrade tasks, blob store state and quota, default admin password,
  default role, script plugin, re-encryption needed, default encryption key.
- Support: support zip, system information, logging configuration, log viewer, Prometheus metrics.
- Upstream-only or dead (do not rebuild): Firewall, IQ, malware, proprietary content, replication,
  freeze, recovery mode, user tokens, service-account tokens, SAML, Crowd, tags, datastore UI, clustering,
  zero downtime, analytics/telemetry, outreach.

## Configuration and startup
- Container: `docker/entrypoint.sh` creates `$ASTRUM_DATA/{tmp,javaprefs}`, reads
  `ASTRUM_DATASTORE_ASTRUM_PASSWORD_FILE`, runs the jar; uid 200, port 8081. Dev runner `hack/run.sh`.
- Properties: classpath defaults → `<install>/etc/astrum-default.properties` → `<data>/etc/astrum.properties`
  → system properties. Keys that matter: `application-port`, `application-host`, `astrum-context-path`,
  `astrum.jwt.*`, `astrum.session.*`, `astrum.secrets.file`, `astrum.onboarding.enabled`,
  `astrum.skipDefaultRepositories`, `astrum.security.randompassword`, `astrum.scripts.allowCreation`,
  `astrum.ldap.cache.ttl`, `astrum.notifications.*`, `astrum.proxy.allowPrivateNetworks`,
  `astrum.security.anticsrftoken.enabled`, `astrum.auth.ratelimit.enabled`, `astrum.format.*.enabled`.
- Datastore: env `ASTRUM_DATASTORE_ASTRUM_{JDBCURL,USERNAME,PASSWORD,SCHEMA,MAXIMUMPOOLSIZE,ADVANCED}` →
  system properties → `<data>/etc/fabric/astrum-store.properties` → H2.

## Known discrepancies today
1. Docker `subdomain` routing is stored and documented but not implemented.
2. Astrum One cannot manage group blob stores (calls a non-existent REST API).
3. Astrum One calls many non-existent upstream endpoints (hidden by gates).
4. `docs/configure/authentication.md` omits the RubyGems and Access Token realms.
