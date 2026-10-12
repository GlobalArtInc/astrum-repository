# Inventory: the Astrum One UI as it is today

What the current UI offers, so the Rust rewrite can rebuild it from scratch. This is a description of
behaviour and URLs, not code to copy. Astrum One is derived from the upstream preview UI: do not take
its code as a model (see `README.md` in this folder).

Paths: `[ui]` = `public/common/components/astrum-ui-plugin/src/frontend/src`,
`[core]` = `public/common/components/astrum-coreui-plugin/src/frontend/src`.
Endpoints: `v1/` = `/service/rest/v1/`, `iui/` = `/service/rest/internal/ui/`, `int/` =
`/service/rest/internal/`, `ED x.y` = ExtDirect RPC `POST /service/extdirect`. Permissions drop the
`astrum:` prefix; `ADMIN` = `astrum:*`.

## Routing
- 134 router states: 29 browse, 98 admin (7 dev-only), 5 user, login, 404; 52 routed page components.
- Hash routing (`#browse/welcome`). Old `#preview/...` and Classic URLs are rewritten by
  `[ui]/utils/previewHeritageNavigation.js`; keep that mapping working in the rewrite (examples:
  `#admin/repository/cleanuppolicies` → `cleanup-policies`, `#admin/system/api` → `#browse/api`,
  `#browse/browse:repo:path` → `#browse/repo/path`, `#user/NuGetApiToken` → `#user/nugetapitoken`,
  unmapped Classic URLs → dashboard).
- Router guard (`[ui]/router/createRouter.js`): visibility check incl. ancestors, redirect to
  `login?returnTo=<base64 hash>`, retry when permissions arrive late, unsaved-changes prompt, document
  title, unknown `admin|settings|user` paths → login, others → 404.
- Gates (`[ui]/interface/NavigationUtils.js`): bundle, licenseValid, statesEnabled, permissions (AND),
  requiresPermission, requiresAnyPermission, permissionPrefix(es), editions, capability, requiresUser,
  notClustered, anonymousAccessOrHasUser.
- This build is the core edition: pages gated on PRO or on missing plugins (saml, crowd, usertoken,
  pro-datastore, tags, service accounts, support request) never show.

## Browse area (`#browse/...`)
| URL | Page | Gate | Backend | Actions |
|---|---|---|---|---|
| `welcome?tab=overview` | usage dashboard | anonymous on or signed in | `iui/usage-dashboard?days=&repository=` | period/repository pickers, KPI tiles, downloads per day chart, top components, repositories by downloads/size, recent, unused (+cleanup link for ADMIN). Announcements tab = upstream outreach iframe (drop). |
| `browse?format=` | repository list | anonymous or user + any `repository-view*`/`repository-content-selector*` | `iui/repositories/details/filtered?page&pageSize&nameFilter` | cards/table, format/type/status filters, paging |
| `{repo}/`, `{repo}/{path}?tab=` | tree + detail panel | inherited | `iui/repositories/repository/{name}`, tree `ED coreui_Browse.read` (also `v1/repositories/{n}/browse`), `v1/components/{id}`, `ED coreui_Component.readAsset/canDelete*`, `DELETE v1/components/{id}`, `DELETE v1/assets/{id}`, `DELETE v1/repositories/{repo}/browse?path=`, `v1/search?repository=&q=` | resizable tree, in-repo search, folder/component/asset panel, Usage block with client snippets for every format (`pages/search/details/dependencySnippets/usage/*`, ours), copy URL, download, delete |
| `repository-profile/{name}` | repository profile | none | as settings profile | same page with Back to Browse |
| `search?q&format&repository&sort&direction` | unified search | `search:read` | `v1/search?…&continuationToken`, `v1/repositories` | filters, sort, load more |
| `search/custom` | criteria search | `search:read` | `v1/search?<criteria>` | criteria rows, paging |
| `search/component/{format:group:name}?version=` (+ `/overview /versions /repos /files /security`) | component detail | `search:read` | `v1/search`, `v1/search/versions`, `v1/search/repositories` | version picker, dependency snippets, repositories, files |
| `upload`, `upload/{repo}` | upload | `component:create` | `v1/formats/upload-specs`, `v1/repositories`, `iui/repositories/details`, `POST iui/upload/{repo}` multipart | pick hosted repo, per-format fields, several assets |
| `api` | API explorer | signed in + `settings:read` | `/service/rest/swagger.json`, `iui/api/permissions`, `POST iui/security/access-check` | endpoint list, try it, who has access, grant access wizard (creates a role, assigns it) |
| `audit?repositoryName=` | audit log | `audit:read`, state `previewAuditEnabled` | `iui/audit-log?page&limit&categories&types&initiators&startDate&endDate&q&repositoryName` | filters, search, paging; reached from the repository profile only |

## Settings (`#settings` hub + `#admin/...`)
Hub: 4 sections (Repository, Security, Support, System) of cards (`settingsConfig.ts`), client search,
tiles on desktop, list on phone; visible if the user has any of 17 read permissions. Section redirects:
`#admin` → `#settings`, `#admin/repository` → repositories, `#admin/security` → privileges,
`#admin/support` → logs, `#admin/system` → tasks.

### Repository
| URL | Gate | Backend | Actions |
|---|---|---|---|
| `repository/repositories`, `/create`, `/create/{format}/{type}`, `/{name}?tab=` | any `repository-admin*` | list `iui/repositories/details`; types `iui/repositories/recipes`; `POST v1/repositories/{fmt}/{type}`; `GET/PUT v1/repositories/{fmt}/{type}/{name}`; `DELETE v1/repositories/{n}`; `POST v1/repositories/{n}/invalidate-cache`, `/rebuild-index`; pickers `v1/blobstores`, `int/cleanup-policies`, `v1/routing-rules`, `iui/docker/suggest-port`; proxy SSL `v1/security/ssl?host&port`, `v1/security/ssl/truststore` | format/type picker, form built from facets (storage, hosted, proxy, group, http client, negative cache, cleanup, routing rule + format facets: Alpine, Apt, Cargo, Conan, Docker, Maven, NuGet, PyPI, Raw, Terraform, Yum, ...), delete, online toggle, invalidate cache, rebuild index |
| `repository/repositories/{name}/profile?tab=` | same | `GET/PUT v1/repositories/...`, `iui/repositories/details?name=`, `v1/tasks`, `v1/capabilities`, `v1/routing-rules/{id}`, `v1/http`, `iui/blobstores`, `v1/security/*` | tabs: repository, structure (groups), membership, audit, security (who has access), system |
| `repository/blobstores`, `/create`, `/{type}/{name}` | `blobstores:read` | `iui/blobstores`, `/types`, `/quotaTypes`, `/usage/{n}`, `POST v1/blobstores/{type}`, `GET/PUT v1/blobstores/{type}/{n}`, `DELETE v1/blobstores/{n}`, `POST v1/blobstores/group/convert/{n}/{new}` | File, S3, Azure, Google, Group; quotas; usage; convert to group |
| `repository/selectors` | `selectors:read` | `v1/security/content-selectors` CRUD, `POST iui/content-selectors/preview` | CSEL editor with validation and preview |
| `repository/cleanup-policies` | ADMIN | `int/cleanup-policies` CRUD, `/criteria/formats`, `POST .../preview/components`, `.../preview/summary`, `.../preview/components/csv` | CRUD, preview with totals, CSV |
| `repository/routing-rules`, `/preview` | ADMIN | `iui/routing-rules` CRUD, `POST iui/routing-rules/test`, `iui/routing-rules/preview` | CRUD, test a path, global preview |
| `repository/proprietary` | `settings:read` | `int/proprietary-content` | mark hosted repos proprietary (upstream feature; decide whether to keep) |

### Security
| URL | Gate | Backend | Actions |
|---|---|---|---|
| `security/privileges` (+create by type, profile) | `privileges:read` | `v1/security/privileges`, `iui/privileges/types`, `POST/PUT v1/security/privileges/{type}[/{name}]`, `DELETE` | CRUD by type |
| `security/roles` (+profile) | `roles:read` + `privileges:read` | `v1/security/roles?source=`, `iui/roles?source&search`, `iui/roles/sources` | CRUD, map external role |
| `security/users`, `/create`, `/invite`, `/{userId}/{source}` (+profile) | `users:read` + `roles:read` | `v1/security/users`, `v1/security/user-sources`, `POST v1/security/users[/invite]`, `PUT .../{id}`, `.../change-password`, `.../roles`, `DELETE`, user token reset | CRUD, invite, password, roles, effective privileges |
| `security/anonymous` | `settings:read` | `iui/anonymous-settings`, `iui/realms/types` | enable, user, realm |
| `security/ldap` (ours) | `ldap:read` | `v1/security/ldap` CRUD, `/change-order`, `/cache`, `/templates`, `/verify-connection`, `/verify-user-mapping`, `/verify-login` | CRUD, order, cache, verify |
| `security/realms` | `settings:read` | `v1/security/realms/available`, `v1/security/realms/active` | activate and order realms |
| `security/sslcertificates` | `ssl-truststore:read` | `ED ssl_TrustStore.*`, `ED ssl_Certificate.*` | add by host:port or PEM, details, remove |
| `security/oauth2` (OpenID Connect) | ADMIN, state `oauth2Available` | `iui/oauth2` | configure OIDC |

### Support
| URL | Gate | Backend |
|---|---|---|
| `support/logs`, `/logs/{file}` | `logging:read` | `int/logging/logs[/{file}]`, `POST int/logging/log/mark` |
| `support/logging` | `logging:read` | `iui/loggingConfiguration[/{n}][/reset]` |
| `support/systeminformation` | `atlas:read` | `/service/rest/atlas/system-information` |
| `support/metrichealth` | `metrics:read` | `iui/status-check` |
| `support/recoverymode` | ADMIN | `iui/recovery-mode`, `POST/DELETE v1/recovery-mode` |
| `support/supportzip` | `atlas:read` | `POST v1/support/supportzippath`, download `/service/rest/wonderland/download/{f}` |

### System
| URL | Gate | Backend |
|---|---|---|
| `system/tasks`, `/create[/{typeId}]`, `/{id}` | `tasks:read` | `v1/tasks` CRUD, `v1/tasks/templates[/{type}]`, `POST v1/tasks/{id}/run`, `/stop` |
| `system/capabilities` | `capabilities:read` | `v1/capabilities` CRUD, `v1/capabilities/types` |
| `system/notifications` (ours) | `settings:read` | `iui/notifications` CRUD, `POST iui/notifications/test`, `/{id}/test` (telegram, slack, webhook; events published/updated/deleted; ru/en; proxy) |
| `system/emailserver` | `settings:read` | `GET/PUT v1/email`, `POST v1/email/verify` |
| `system/http` | `settings:read` | `GET/PUT v1/http` |
| `system/nodes` | ADMIN | `ED node_NodeAccess.nodes` |

## User, sign-in, 404
| URL | Backend | Actions |
|---|---|---|
| `#user/account` | `GET/PUT iui/user`, `POST /service/rest/wonderland/authenticate`, `PUT iui/user/{id}/password` | name, email, password |
| `#user/accesstokens` (ours) | `GET/POST iui/access-tokens`, `DELETE iui/access-tokens/{id}` | create (name, expiry, scope, shown once), revoke |
| `#user/nugetapitoken` | auth dialog then `int/nuget-api-key?authToken=` | reveal, reset |
| `#/login?returnTo=<b64>` | `POST /service/rapture/session` (base64 form), SSO `{ctx}/oidc/login?hash=` | local login, SSO button, continue anonymously, initial admin password hint, language menu; always light |
| `#404` | — | keeps the bad URL |

## Shell
- Header: menu toggle (rail on desktop, drawer on compact), logo by edition/theme, global search panel
  (5 recent searches in localStorage `astrum-recent-searches`, search everywhere / in current repo,
  5 repositories from `v1/repositories`, 6 components from `v1/search?q=`, arrows/Enter, Ctrl+K/⌘K,
  full-screen layer on compact), refresh (desktop), theme toggle, language menu (cookie
  `astrum-language`, reload), status bell (state `health_checks_failed`, opens Metric Health), help
  menu (links.globalart.dev + version), user menu (initials avatar, Account, Access tokens, NuGet key,
  Log out = `DELETE /service/rapture/session`; signed out → Sign in with returnTo).
- Left menu: Dashboard, Search, Browse, Upload, API, divider, Settings; items hide when their route is
  not visible; collapsed state in localStorage `astrum.sidebar.open`; version in the footer.
- Mobile: phone ≤639px, compact ≤1023px; drawer with scrim and focus trap; mobile filter drawers.
- Themes: light/dark/system, default dark, localStorage `astrum-theme-preference`, `data-theme` on
  `<html>`; Radix Themes with `accentColor="blue"` remapped to the Astrum violet in
  `[core]/styles/astrum-brand.css` (our design tokens; 189 properties) plus
  `[core]/styles/theme-variables.css`. Login always light.
- i18n: server picks language (cookie, Accept-Language, English); plurals ONE/FEW/MANY/OTHER;
  dictionaries react-ui 6,279 keys, react-coreui 190, extjs-* 103; tooling `tools/i18n/` + Ownlate.
- Onboarding: admin-only wizard (change admin password, anonymous access, EULA) from `iui/onboarding`.
- Session expiry: REST 401 → event `astrum:session-expired` → modal (Sign in / Refresh); idle timeout
  (`uiSettings.sessionTimeout`, 30 s countdown, cross-tab activity); redirect to login when the user
  disappears; router waits for auth/permissions.
- Other: branding header/footer HTML (DOMPurify), recovery-mode banner, unsaved-changes dialog, toasts,
  document title `<route title> - <uiSettings.title>`, path URLs rewritten to hash URLs.

## What the UI still takes from the ExtJS runtime (becomes plain API in the rewrite)
- State (`window.AS.State`, polled from `/service/extdirect/poll/rapture_State_get`): user, status
  {version, edition}, uiSettings {title, sessionTimeout}, anonymousUsername, context path, branding,
  uiLanguage, browseableformats, previewAuditEnabled, oauth2Available/Enabled, recoveryMode*,
  nugetApiKeyRealmEnabled, health_checks_failed, onboarding.required, admin.password.file, datastore
  flags, cleanup flags, ...
- Permissions: `ED rapture_Security.getPermissions` with Shiro-style wildcard matching.
- Auth: session cookie `ASTRUMSESSIONID`, `POST/DELETE /service/rapture/session`, re-auth token dialog
  (`ED rapture_Security.authenticationToken`), CSRF cookie/header `AS-ANTI-CSRF-TOKEN`, header
  `X-Astrum-UI`.
- ExtDirect calls in React pages: `coreui_Browse.read`, `coreui_Component.*`, `ssl_TrustStore.*`,
  `ssl_Certificate.*`, `node_NodeAccess.nodes`, `rapture_Security.getPermissions`.

## Size
- `[ui]` ≈ 212.6k non-test lines in ~1,322 files (+161k test lines); `[core]` ≈ 13.3k (+9.1k tests).
- Largest areas: settings 101.8k (repositories 15.2k, tasks 9.8k, blob stores 6.7k), search 38k (~22k
  are unreachable drafts), shared 16.6k, browse 13.7k.

## Not worth rebuilding (drafts, upstream-only or no backend here)
- Per-format search pages (npm, NuGet, Docker, PyPI, Helm, Go, Raw, Yum, APT, RubyGems) and
  `GASearchPage`: not routed; `search.api.ts` targets `v1/search/ga*` that has no backend.
- Dashboard leftovers (WelcomeDashboard, monthly metrics, CE limits, malware/health-check/firewall
  cards), unused coreui login/shared/SearchableSelect/OCI attestation code, Test Hub.
- Upstream-only features: Firewall, IQ Server, malware risk, repository health check, tags, user tokens,
  service account tokens, SAML, Crowd, support request, data store page, cloud usage, outreach.
- UI calls with no backend: `v1/iq/*`, `iui/firewall/*`, `iui/healthcheck*`, `v1/malicious-risk/*`,
  `v1/monthly-metrics`, `v1/daily-metrics/*`, `v1/evaluation/*`, `/service/rest/dependency_snippets`,
  `v1/tags`, `v1/plan`, `v1/clustered/*`, `ED outreach_Outreach`.
