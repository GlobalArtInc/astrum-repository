# Inventory: the data a production installation holds today

What the Rust server or its migrator must read to take over an installation of Astrum Repository 2.0.0
(PostgreSQL backed) without loss. This describes our own data for interoperability; it is not code to
copy. Source paths are given for reference only.

Confirm against a live database with `\dt` and
`SELECT version,description,type,script,success FROM flyway_schema_history ORDER BY installed_rank;`.

## Conventions
- PostgreSQL types: UUID, JSONB (Jackson-written UTF-8 text), BYTEA.
- Tables are created at startup with `CREATE TABLE IF NOT EXISTS` (MyBatis DAO XML `createSchema` /
  `extendSchema`); later columns and most indexes come from Flyway Java steps.
- 23 format table prefixes: `alpine apt cargo cocoapods composer conan conda docker gitlfs go helm
  huggingface maven2 npm nuget p2 pypi r raw rubygems swift terraform yum`.
- `repository_id`, `component_id`, `asset_id`, `asset_blob_id` are identity INTs unique only within a
  format; global tables carry `format`. `<fmt>_content_repository.config_repository_id` = `repository.id`
  (UUID, no FK).
- Attribute JSON is nested plain JSON (timestamps often epoch millis). Keep values byte-for-byte where
  possible. Sensitive values are secret references `"_<n>"` (see Secrets), never inline.

## Configuration tables
| Table | Columns | Notes |
|---|---|---|
| `repository` | id UUID PK, name UK, recipe_name (`maven2-hosted`, `npm-group`, ...), online, routing_rule_id, attributes JSONB | attributes per facet: `storage{blobStoreName,strictContentTypeValidation,writePolicy,dataStoreName,latestPolicy}`, `component{proprietaryComponents}`, `cleanup{policyName:[...]}` (internal UI answers `policyNames`), `proxy{remoteUrl,contentMaxAge,metadataMaxAge,preserveEncodedCharacters}`, `negativeCache{enabled,timeToLive}`, `httpclient{blocked,autoBlock,connection{...},authentication{type,username,password:"_N",bearerTokenId:"_N",preemptive}}`, `group{memberNames}`, plus format objects (`maven{versionPolicy,layoutPolicy,contentDisposition}`, `docker{...}`, `raw{contentDisposition}`, `aptSigning`, `yumSigning`, `terraform{registryHost,upstreamHost,...}`, ...). Signing keys are stored in plain form. Checked 2026-10-18: some rows (e.g. `maven-central` of a fresh install) hold `blocked`/`autoBlock` inside `httpclient.connection`; the Java REST ignores those and reports the top-level keys, missing ones as `false`. Groups of maven2 hold an empty `maven{}`; proxies of maven2 an empty `maven-indexer{}`. `routing_rule_id` points at `repository_routing_rule.id`. |
| `blob_store_configuration` | id UUID, name UK, type (`File`,`S3`,`Group`), attributes | File `{"file":{"path":"default"}}` (relative to `<data>/blobs/` or absolute); S3 `{"s3":{bucket,prefix,region,accessKeyId,secretAccessKey:"_N",sessionToken:"_N",endpoint,forcepathstyle,...}}`; Group `{"group":{members,fillPolicy}}` (the 2.0.0 UI refuses to create groups: "Blob store groups are not enabled"); optional `blobStoreQuotaConfig{quotaType,quotaLimitBytes}` |
| `repository_routing_rule` | id, name UK, mode (ALLOW/BLOCK), description, matchers JSONB (regex array) | |
| `cleanup_policy` | name PK, notes, format, mode, criteria JSONB | criteria (strings): `lastBlobUpdated`, `lastDownloaded` (ages in seconds, e.g. `"7776000"` for 90 days), `isPrerelease` (`"true"`/`"false"`), `regex`, `retain`, `sortBy`; format `ALL_FORMATS` for all formats; mode `delete` |
| `selector_configuration` | name PK, type (`csel`/`jexl`), description, attributes `{"expression":...}` | content selectors |
| `capability_storage_item` | id UUID, version, type, enabled, notes, properties JSONB (flat string map) | passwords are secret ids; webhooks, base URL, ... |
| `email_configuration` | single row (id=1): enabled, host, port, username, password (secret id), fromaddress, subjectprefix, starttls*, sslonconnectenabled, sslcheckserveridentityenabled, astrumtruststoreenabled | |
| `http_client_configuration` | single row: connection, proxy, authentication JSONB | outbound HTTP; password secret id |
| `astrum_key_value` | key PK, type, value JSONB `{"value": ...}`, created | `astrum.ldap.servers` (ours; `{servers:[...]}`, `authPasswordId` secret id), `astrum.oidc.configuration` (`clientSecretId` secret id), `astrum.oidc.users` (`{users:{<id>:{userId,subject,firstName,lastName,email,groups,lastLogin(ms)}}}`), `astrum.notifications.channels` (ours; `{channels:[...]}` with `secretId`, `proxyPasswordSecretId`, `signingSecretId`, `authorizationSecretId`), `ssrf.protection.config` (`{enabled,allowedIPs,allowedDomains}`), `astrum.community.eula.accepted`, upgrade flags |
| `logging_overrides` | id, name, level | absent in a 2.0.0 installation checked on 2026-10-18: overrides live in `<data>/etc/logback/logback-overrides.xml` (`<logger name='ROOT' level='WARN'/>`) |
| `script` | name, type, content | |
| `key_store_data` | name, bytes BYTEA (JKS `ssl`) | default store passwords are constants in `astrum-ssl-plugin/.../KeyStoreManagerConfigurationImpl.java` |
| `trusted_ssl_certificate` | alias PK, pem | the SSL truststore |

## Security tables
| Table | Columns | Notes |
|---|---|---|
| `security_user` | id PK, first_name, last_name, password, status (`active`/`disabled`/`changepassword`), email, version | hashes below |
| `role` | id PK, name, description, privileges JSONB (ids), roles JSONB (ids), read_only, version | |
| `privilege` | id PK, name, description, type, properties JSONB, read_only, version | types: `wildcard{pattern}`, `application{domain,actions}`, `repository-view`/`repository-admin{format,repository,actions}`, `repository-content-selector{contentSelector,format,repository,actions}`, `script{name,actions}`; `actions` is one comma string of permission actions (`read,create,update` for application, `browse,read,add` for repositories, `*` for all); built-in privileges are not stored |
| `user_role_mapping` | user_id, user_lo, source (`default`,`LDAP`,`OAuth2`), roles JSONB, version; PK (user_id, source) | built-in roles `as-admin`, `as-anonymous` are not in `role` |
| `realm_configuration` | single row: realm_names JSONB (ordered) | e.g. `AstrumAuthenticatingRealm`, `DockerToken`, `NpmToken`, `NuGetApiKey`, `LdapRealm`, `AccessTokenRealm`, `DefaultRole`, `OAuth2Realm`, plus the new format token realms |
| `anonymous_configuration` | single row: enabled, user_id, realm_name | |
| `api_key` (legacy v1) | encrypted primary_principal/token, principals BYTEA | copied to v2 by a task; rows kept |
| `api_key_v2` | username, principals BYTEA, domain, access_key (first half), secret (secret id of second half), created, created_by_* | domains `NuGetApiKey`, `NpmToken`, `DockerToken`, `ConanToken`, `CargoToken`, `RubyGemsApiKey`, `TerraformToken`, ...; full key = access_key (first 18 characters of a UUID) + decrypted secret (the other 18); clients send the UUID for NuGet and `<domain>.<UUID>` for npm, Docker, RubyGems and Terraform |
| `access_token` (ours) | id, username, principals BYTEA, name, token_hash (hex SHA-256 of the full token), token_hint (first 11 chars), scope, created, expires, last_used | token = `astrum_` + 40 alphanumerics |
| `jwt_secret`, `jwt_session` | | transient; regenerate (sessions end) |

## System tables
- `deployment_id`, `node_id` (single rows) — keep.
- `upgrade_tasks` — background upgrade queue; the migrator should require it to be empty.
- `audit_events` (id, domain, type, context, timestamp, initiator, node_id, attributes) — history.
- Quartz 2.3.2 tables (`qrtz_*`, sched_name `astrum`). Task definitions live in
  `qrtz_job_details.job_data` / `qrtz_triggers.job_data` as Java-serialized `JobDataMap` (String→String):
  `.id .typeId .name .enabled .visible .alertEmail .notificationCondition .created .updated lastRunState.*`,
  task fields (`repositoryName`, `blobstoreName`, ...), triggers `schedule.type .cronExpression .startAt
  .daysToRun .clientTimeZone`. A Rust migrator needs a Java-serialization reader (or a one-off export
  helper). Only job and trigger definitions need to move. Checked 2026-10-18: `schedule.type` is one of
  manual, once, hourly, daily, weekly, monthly, cron; `startAt` is ISO UTC; weekly `daysToRun` holds day
  names (`MON,WED`), monthly day numbers with `999` for the last day, already shifted to UTC by the
  server (the client offset is not kept); cron keeps `clientTimeZone` (`Z`, `Etc/UTC` or a zone id).
- `flyway_schema_history` — see Versions.

## Content tables (per format prefix)
| Table | Columns | Keys |
|---|---|---|
| `<fmt>_content_repository` | repository_id INT, config_repository_id UUID, attributes, created, last_updated | UK config_repository_id |
| `<fmt>_component` | component_id, repository_id, namespace, name, kind, version, normalized_version, attributes, created, last_updated, entity_version (`maven2_component` + base_version) | UK (repository_id, namespace, name, version) |
| `<fmt>_asset` | asset_id, repository_id, path, kind, component_id, asset_blob_id, last_downloaded, attributes, created, last_updated, asset_blob_size | UK (repository_id, path) |
| `<fmt>_asset_blob` | asset_blob_id, blob_ref UK, blob_size, content_type, checksums JSONB `{sha1,md5,sha256,sha512}`, blob_created, created_by, created_by_ip, external_metadata `{etag,lastModified}`, added_to_repository | |
| `<fmt>_browse_node` | derived (rebuild) | |
- asset attributes: `content{last_modified,etag}`, `cache{last_verified,cache_token,invalidated}`,
  `checksum{...}`, and a format object (`maven2{groupId,artifactId,version,baseVersion,classifier,extension,asset_kind}`, `npm{...}`, `docker{...}`).
- Extra tables: `apt_key_value` (repository_id, category, key, value), `swift_package_url`
  (repository_id, component_id, url; the URL lower-cased without scheme and `.git`, e.g. `github.com/acme/list`,
  while the component attributes keep the URLs as published), `gitlfs_lock` (lock_id identity, repository_id,
  path without a leading slash, ref_name, owner, locked_at; UK repository_id, path).
- Stored per format (confirmed on 2.0.0): checksum files of Maven uploads are assets (`ARTIFACT_SUBORDINATE`,
  `REPOSITORY_METADATA`), hosted index files are assets too (helm `HELM_INDEX`, rubygems `INDEX`, yum `REPOMD`
  and `REPODATA`, pypi `INDEX` pages), hosted wheels have no PEP 658 `.metadata` assets, every docker tag is an
  asset with its own blob of the manifest bytes, npm hosted package roots are `PACKAGE_ROOT` assets holding the
  document, and proxies keep what the remote sent (a Helm index byte for byte, npm package documents re-serialized
  with their tarball URLs unchanged; cache state in `cache.last_verified`, remote headers in `content.etag` /
  `content.last_modified`).
- Global: `asset_download_count` (format, asset_id, repository_id, component_id, download_count,
  last_downloaded; PK format, asset_id) and `asset_download_daily` (download_day, format, repository_id,
  component_id NOT NULL, `0` for files without a component, download_count; per component and day, not per asset)
  — ours, primary data.
- Rebuildable or discardable: browse nodes, `search_components` / `search_assets` (pg_trgm, tsvector),
  `blob_store_metrics`, `soft_deleted_blobs` (rebuild from `.properties`), `normalized_version`,
  `asset_blob_size`, `jwt_*`, Quartz runtime tables, `hf_xet_presigned_url_store`, `upgrade_tasks`,
  `flyway_schema_history`.
- May exist from older builds (tolerate): `malware_remediator_task`, `helm_key_value`, `yum_key_value`,
  `npm_key_value`, `pypi_key_value`, `distributed_auth_ticket_cache`.

## Secrets
- A secret reference is the string `"_<id>"` → row in `secrets` (id, purpose, key_id, secret as a PHC
  string, user_id, created, last_updated).
- PHC: `$<alg>$iv=<hex>,key_iteration=<n>,key_len=<bits>,hmac=<hex>$<salt b64>$<ciphertext b64>`.
  Decrypt: PBKDF2 (`PBKDF2WithHmacSHA1` default 1024/128, or `PBKDF2WithHmacSHA256` 10000/256; legacy
  alg id `AES/CBC/PKCS5Padding` = SHA1) over the password = `astrum.mybatis.cipher.password` when
  `key_id` is NULL, else the matching key in secrets.json; verify HMAC-SHA256(key, iv‖ciphertext) when
  present; AES-CBC/PKCS5 with the hex iv; UTF-8.
- secrets.json: path from `astrum.secrets.file` or `ASTRUM_SECRETS_KEY_FILE`; shape
  `{"active","keys":[{"id","key"}],"fixedEncryption":{keyId,salt,iv,keyIterations},"previousFixedEncryption"}`.
  Production writes it from Ansible (`etc/secrets.json`, active `astrum-1`).
- "MyBatis cipher" (fixed salt/iv from `astrum.mybatis.cipher.*`, or `fixedEncryption`): encrypts the
  `principals` BYTEA (Java-serialized Shiro principal collection — easier to rebuild from username +
  realm) and legacy `api_key` columns. Source: `astrum-datastore-mybatis/.../MyBatisCipher.java`.
- User passwords: default `$shiro1$SHA-512$<iterations>$<salt b64>$<hash b64>` (SHA-512 of
  salt‖password, re-hashed iterations-1 times; 2.0.0 writes 50000 iterations); optional PBKDF2 PHC;
  legacy unsalted hex SHA-1/MD5. The anonymous user holds the placeholder `unused`.
- The default cipher password (no `astrum.mybatis.cipher.password`, no secrets.json) is `changeme`:
  checked on 2026-10-18 by the HMAC of real rows.
- Legacy secret values (pre-2.2) without `_`: maven-style `{...}`/`~{...}~` cipher or base64 legacy
  PBE — the migrator should require none remain or decode them (`astrum-crypto/.../PasswordCipher.java`,
  `LegacyCipherFactoryImpl.java`).
- Secret ids appear in: repository http client auth, S3 keys, capability passwords, email password,
  global HTTP auth, LDAP `authPasswordId`, OIDC client secret, notification channels, `api_key_v2.secret`.
- Node keystores `keystores/node/{private.ks,trusted.ks}` (JKS; default passwords are constants in
  `astrum-self-hosted/.../node/KeyStoreManagerConfigurationImpl.java`).

## Blob stores on disk
- File store at `<data>/blobs/<path>`; `metadata.properties` with `type=file/1`.
- `blob_ref` current form `store@<uuid>@yyyy-MM-ddTHH:mm` (UTC) → `content/yyyy/MM/dd/HH/mm/<uuid>.bytes`
  and `.properties`. Legacy forms (`store@<id>`, `store@<node>:<id>`, `store:<id>@<node>`) →
  `content/vol-%02d/chap-%02d/<id>` with vol = |javaStringHash(id) % 43| + 1, chap = |javaStringHash(id)
  % 47| + 1 (Java int32 hash, sign-preserving `%`). In file names `. \ : /` become `-`. `tmp$` ids →
  `content/tmp/`, `path$` ids → `content/directpath/<name>`.
- `.properties`: Java Properties (ISO-8859-1, `\uXXXX`, first line a timestamp comment). Keys
  `@BlobStore.blob-name`, `@BlobStore.content-type`, `@BlobStore.created-by`, `@BlobStore.created-by-ip`,
  `@Bucket.repo-name`, optional `@BlobStore.temporary-blob`, `@BlobStore.direct-path`, `@External.etag`,
  `@External.last-modified`; `sha1`, `size`, `creationTime` (epoch ms), `lastDownloaded` (ISO).
- Soft delete: original `.properties` gets `deleted=true`, `deletedReason`, `deletedDateTime`,
  `softDeletedLocation`; a copy with `originalLocation` goes into the deletion-time bucket; `.bytes`
  stays until compaction; a row in `soft_deleted_blobs` (record_id, blob_id, source_blob_store_name,
  deleted_date, date_path_ref). A REST delete on the 2.0.0 reference left neither the mark nor a row right away.
- Other files: `reconciliation/<yyyy-MM-dd>` journal (`yyyy-MM-dd HH:mm:ss,<blobId>,<dateBased>`),
  legacy `deletions.index`, `*metrics.properties`.
- S3: `<prefix>/content/<same location>.bytes|.properties`, `<prefix>/metadata.properties`, same
  soft-delete scheme.

## Versions
- Flyway 12.4 Java migrations, baseline `1`, out-of-order allowed; descriptions are class names
  (`Z_NNN_` prefix for ordered rounds); repeatable steps have NULL version.
- Current maximum 2.166. Astrum refuses a database from a later version.
- Migrator rules: max(version) = 2.166 exactly; every row `success = true`; every description known;
  `upgrade_tasks` empty; compare versions as integer tuples (2.166 > 2.17); optionally require no legacy
  `blob_ref` and no non-`_` secret values.

## Data directory
- Default `<install>/../globalart-work/astrum3` or `-Dkaraf.data`; in the container `/astrum-data`.
- `etc/astrum.properties` (port, context path, secrets file, mybatis cipher, security flags),
  `etc/fabric/astrum-store.properties` (jdbcUrl, username, password, schema, pool; or
  `ASTRUM_DATASTORE_ASTRUM_*` env), `etc/logback/`, `admin.password` (first run), secrets.json,
  `keystores/node/*`, `blobs/`, `log/` (`astrum.log`, `request.log`, `audit/audit.log`, `tasks/*.log`).
- Discardable: `tmp/`, `downloads/`, `restore-from-backup/`, stale H2 `db/astrum.mv.db`; OrientDB
  leftovers `db/{config,security,component}` block startup.
