
/**
 * Decode a URL segment; return original string if decoding fails.
 * @param {string} segment
 * @returns {string}
 */
function safeDecodeURIComponent(segment) {
  try {
    return decodeURIComponent(segment);
  } catch {
    return segment;
  }
}

/**
 * Maps Preview browse URLs under `preview/browse/...` to Classic `browse/browse:...` (repo or repo:path).
 * Matches Classic MasterDetail + BrowseTree bookmarks: `#browse/browse:<encRepo>` or
 * `#browse/browse:<encRepo>:<encNodePath>`.
 *
 * @param {string} previewCleanPath full path e.g. `preview/browse/my-repo/` or `preview/browse/my-repo/com/foo`
 * @returns {string}
 */
export function previewBrowsePathToHeritageBrowseParam(previewCleanPath) {
  const prefix = 'preview/browse/';
  if (!previewCleanPath.startsWith(prefix)) {
    return 'browse/browse';
  }
  let afterPrefix = previewCleanPath.slice(prefix.length).replace(/^\/+/g, '');
  afterPrefix = afterPrefix.replace(/\/+$/g, '');
  if (!afterPrefix) {
    return 'browse/browse';
  }
  const slashIdx = afterPrefix.indexOf('/');
  if (slashIdx === -1) {
    const repo = safeDecodeURIComponent(afterPrefix);
    if (!repo) {
      return 'browse/browse';
    }
    return `browse/browse:${encodeURIComponent(repo)}`;
  }
  const repo = safeDecodeURIComponent(afterPrefix.slice(0, slashIdx));
  let nodePath = afterPrefix.slice(slashIdx + 1).replace(/\/+$/g, '');
  nodePath = safeDecodeURIComponent(nodePath);
  if (!repo) {
    return 'browse/browse';
  }
  if (!nodePath) {
    return `browse/browse:${encodeURIComponent(repo)}`;
  }
  return `browse/browse:${encodeURIComponent(repo)}:${encodeURIComponent(nodePath)}`;
}

/**
 * Bidirectional route mapping between Preview UI and Classic UI.
 * Ordered from most specific to least specific so pattern matching works correctly.
 */
export const PREVIEW_TO_HERITAGE_ROUTES = [
  { preview: 'preview/browse/welcome', heritage: 'browse/welcome' },
  { preview: 'preview/browse/upload', heritage: 'browse/upload' },
  { preview: 'preview/browse/tags', heritage: 'browse/tags' },
  { preview: 'preview/browse/audit', heritage: 'browse/browse' },
  { preview: 'preview/browse/api', heritage: 'browse/browse' },
  { preview: 'preview/browse/repository-profile', heritage: 'browse/browse' },
  { preview: 'preview/browse/health-report', heritage: 'browse/browse' },
  { preview: 'preview/browse/search', heritage: 'browse/search/generic' },
  { preview: 'preview/browse/', heritage: (p) => previewBrowsePathToHeritageBrowseParam(p) },
  { preview: 'preview/browse', heritage: 'browse/browse' },
  { preview: 'preview/settings', heritage: 'browse/welcome' },
  { preview: 'preview/admin/repository/repositories', heritage: 'admin/repository/repositories' },
  { preview: 'preview/admin/repository/blobstores', heritage: 'admin/repository/blobstores' },
  { preview: 'preview/admin/repository/selectors', heritage: 'admin/repository/selectors' },
  { preview: 'preview/admin/repository/cleanup-policies', heritage: 'admin/repository/cleanuppolicies' },
  { preview: 'preview/admin/repository/routing-rules', heritage: 'admin/repository/routingrules' },
  { preview: 'preview/admin/repository/datastore', heritage: 'admin/repository/datastore' },
  { preview: 'preview/admin/repository/proprietary', heritage: 'admin/repository/proprietary' },
  { preview: 'preview/admin/security/privileges', heritage: 'admin/security/privileges' },
  { preview: 'preview/admin/security/roles', heritage: 'admin/security/roles' },
  { preview: 'preview/admin/security/users', heritage: 'admin/security/users' },
  { preview: 'preview/admin/security/anonymous', heritage: 'admin/security/anonymous' },
  { preview: 'preview/admin/security/ldap', heritage: 'admin/security/ldap' },
  { preview: 'preview/admin/security/realms', heritage: 'admin/security/realms' },
  { preview: 'preview/admin/security/sslcertificates', heritage: 'admin/security/sslcertificates' },
  { preview: 'preview/admin/security/user-tokens', heritage: 'admin/security/usertoken' },
  { preview: 'preview/admin/security/saml', heritage: 'admin/security/saml' },
  { preview: 'preview/admin/security/oauth2', heritage: 'admin/security/oauth2' },
  { preview: 'preview/admin/security/crowd', heritage: 'admin/security/atlassiancrowd' },
  { preview: 'preview/admin/support/logs', heritage: 'admin/support/logs' },
  { preview: 'preview/admin/support/logging', heritage: 'admin/support/logging' },
  { preview: 'preview/admin/support/systeminformation', heritage: 'admin/support/systeminformation' },
  { preview: 'preview/admin/support/metrichealth', heritage: 'admin/support/status' },
  { preview: 'preview/admin/support/recoverymode', heritage: 'admin/support/recovery' },
  { preview: 'preview/admin/support/supportrequest', heritage: 'admin/support/supportrequest' },
  { preview: 'preview/admin/support/supportzip', heritage: 'admin/support/supportzip' },
  { preview: 'preview/admin/system/tasks', heritage: 'admin/system/tasks' },
  { preview: 'preview/admin/system/capabilities', heritage: 'admin/system/capabilities' },
  { preview: 'preview/admin/system/emailserver', heritage: 'admin/system/emailserver' },
  { preview: 'preview/admin/system/http', heritage: 'admin/system/http' },
  { preview: 'preview/admin/system/nodes', heritage: 'admin/system/nodes' },
  { preview: 'preview/admin/system/notifications', heritage: 'admin/system/notifications' },
  { preview: 'preview/admin/system/api', heritage: 'admin/system/api' },
  { preview: 'preview/user/account', heritage: 'user/account' },
  { preview: 'preview/user/nugetapitoken', heritage: 'user/NuGetApiToken' },
  { preview: 'preview/user/usertoken', heritage: 'user/usertoken' },
];

/**
 * Classic UI equivalent path for the given Preview hash path, or null if none exists.
 */
export function getHeritageEquivalent(previewPath) {
  const raw = (previewPath || '').replace(/^\//, '');
  const [clean, queryString] = raw.split('?');
  if (!clean.startsWith('preview/')) return null;

  for (const { preview, heritage } of PREVIEW_TO_HERITAGE_ROUTES) {
    if (typeof heritage === 'function') {
      if (clean.startsWith(preview)) {
        const result = heritage(clean);
        return queryString ? `${result}?${queryString}` : result;
      }
    } else if (clean === preview) {
      return queryString ? `${heritage}?${queryString}` : heritage;
    } else if (clean.startsWith(preview + '/')) {
      const suffix = clean.slice(preview.length + 1);
      const result = suffix ? `${heritage}:${suffix}` : heritage;
      return queryString ? `${result}?${queryString}` : result;
    }
  }
  return null;
}

/** Hash path (no leading #) of the Astrum One dashboard; also the target of every unknown Classic URL. */
export const PREVIEW_WELCOME_PATH = 'browse/welcome';

/**
 * Classic search ids (ExtJS search features and Classic React routes) to Astrum One search formats.
 * Ids that are not here, such as `generic` and `keyword`, search every format.
 */
const SEARCH_FORMATS = {
  alpine: 'alpine',
  ansiblegalaxy: 'ansiblegalaxy',
  apt: 'apt',
  cargo: 'cargo',
  cocoapods: 'cocoapods',
  composer: 'composer',
  conan: 'conan',
  conda: 'conda',
  docker: 'docker',
  gitlfs: 'gitlfs',
  go: 'go',
  golang: 'go',
  helm: 'helm',
  huggingface: 'huggingface',
  hugging_face: 'huggingface',
  maven: 'maven',
  maven2: 'maven',
  npm: 'npm',
  nuget: 'nuget',
  oci: 'oci',
  p2: 'p2',
  pub: 'pub',
  pypi: 'pypi',
  r: 'r',
  raw: 'raw',
  rubygems: 'rubygems',
  swift: 'swift',
  terraform: 'terraform',
  yum: 'yum',
};

/** Re-encodes one URL segment so encoded and plain spellings of a Classic URL give the same result. */
function encodeSegment(segment) {
  return encodeURIComponent(safeDecodeURIComponent(segment));
}

/** Encodes every segment of a slash-separated path and keeps the slashes. */
function encodePath(path) {
  return path.split('/').filter(Boolean).map(encodeURIComponent).join('/');
}

/**
 * Reads the criteria of a Classic search bookmark: `keyword=foo AND repository_name=bar`, URI-encoded as
 * a whole by the Classic UI. `keyword` becomes the query, `repository_name` the repository filter and
 * `format` the format; the values of the other criteria (group, name, version...) become the query when
 * there is no keyword.
 */
function parseSearchCriteria(rawCriteria) {
  const result = {};
  const otherValues = [];
  const criteria = safeDecodeURIComponent(rawCriteria || '');
  for (const part of criteria.split(' AND ')) {
    const separator = part.indexOf('=');
    const key = separator === -1 ? '' : part.slice(0, separator).trim();
    const value = (separator === -1 ? part : part.slice(separator + 1)).trim();
    if (!value) {
      continue;
    }
    if (key === 'keyword') {
      result.q = value;
    } else if (key === 'repository_name') {
      result.repository = value;
    } else if (key === 'format') {
      result.format = SEARCH_FORMATS[value.toLowerCase()];
    } else {
      otherValues.push(value);
    }
  }
  if (!result.q && otherValues.length) {
    result.q = otherValues.join(' ');
  }
  return result;
}

function searchPath({ q, format, repository }) {
  const params = [];
  if (q) params.push(`q=${encodeURIComponent(q)}`);
  if (format) params.push(`format=${encodeURIComponent(format)}`);
  if (repository) params.push(`repository=${encodeURIComponent(repository)}`);
  return params.length ? `browse/search?${params.join('&')}` : 'browse/search';
}

/**
 * `browse/search/<id>=<criteria>` (ExtJS bookmark) or `browse/search/<id>/<keyword>` (Classic React route).
 */
function formatSearchPath(searchId, criteria, keyword) {
  let parsed = {};
  if (criteria !== undefined) {
    parsed = parseSearchCriteria(criteria);
  } else if (keyword) {
    parsed = keyword.includes('=') ? parseSearchCriteria(keyword) : { q: safeDecodeURIComponent(keyword) };
  }
  const format = SEARCH_FORMATS[searchId.toLowerCase()] || parsed.format;
  return searchPath({ ...parsed, format });
}

function browsePath(rawRepo, rawNodePath) {
  const repo = safeDecodeURIComponent(rawRepo);
  if (!repo) {
    return 'browse';
  }
  const nodePath = encodePath(safeDecodeURIComponent(rawNodePath || ''));
  return `browse/${encodeURIComponent(repo)}/${nodePath}`;
}

/**
 * `admin/security/users:<itemId>` where itemId is `<source>/<userId>` or `<userId>` (default source),
 * URI-encoded once more by the Classic master-detail page.
 */
function userPath(itemId) {
  const decoded = safeDecodeURIComponent(itemId);
  const separator = decoded.indexOf('/');
  const source = separator === -1 ? 'default' : decoded.slice(0, separator);
  const userId = separator === -1 ? decoded : decoded.slice(separator + 1);
  if (!userId) {
    return 'admin/security/users';
  }
  return `admin/security/users/${encodeSegment(userId)}/${encodeSegment(source || 'default')}`;
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** A Classic page that has no sub-pages in Astrum One: the page itself and anything below it. */
function page(classicPath, previewPath) {
  return [new RegExp(`^${escapeRegExp(classicPath)}(?:[/:].*)?$`), () => previewPath];
}

/** A Classic master-detail page: `<path>` lists, `<path>:<id>` opens one item. */
function masterDetail(classicPath, previewPath, toItem = encodeSegment) {
  return [
    [new RegExp(`^${escapeRegExp(classicPath)}$`), () => previewPath],
    [new RegExp(`^${escapeRegExp(classicPath)}:(.+)$`), (m) => `${previewPath}/${toItem(m[1])}`],
  ];
}

/** A Classic list page with `/create` and `/edit/<id>` sub-pages (older bookmarks use `:<id>`). */
function listCreateEdit(classicPath, previewPath) {
  const base = escapeRegExp(classicPath);
  return [
    [new RegExp(`^${base}$`), () => previewPath],
    [new RegExp(`^${base}/create$`), () => `${previewPath}/create`],
    [new RegExp(`^${base}/edit/(.+)$`), (m) => `${previewPath}/${encodeSegment(m[1])}`],
    [new RegExp(`^${base}:(.+)$`), (m) => `${previewPath}/${encodeSegment(m[1])}`],
    page(classicPath, previewPath),
  ];
}

/**
 * Classic URL patterns (hash path without `#`, query string and trailing slashes) and the Astrum One
 * hash path each one opens. The first match wins.
 */
const CLASSIC_TO_PREVIEW_ROUTES = [
  [/^(?:browse(?:\/welcome)?)?$/, () => PREVIEW_WELCOME_PATH],
  [/^browse\/browse$/, () => 'browse'],
  [/^browse\/browse:([^:]*)(?::(.*))?$/, (m) => browsePath(m[1], m[2])],
  [/^browse\/search(?:=(.*))?$/, (m) => searchPath(parseSearchCriteria(m[1]))],
  [/^browse\/search\/custom(?:[=/:].*)?$/, () => 'browse/search/custom'],
  [/^browse\/search\/saved(?:\/.*)?$/, () => 'browse/search'],
  [/^browse\/search\/([A-Za-z0-9_-]+)(?:[=:](.*)|\/(.*))?$/, (m) => formatSearchPath(m[1], m[2], m[3])],
  [/^browse\/upload$/, () => 'browse/upload'],
  [/^browse\/upload[/:](.+)$/, (m) => `browse/upload/${encodeSegment(m[1])}`],
  ...masterDetail('browse/tags', 'browse/tags'),

  [/^admin(?:\/hub)?$/, () => 'settings'],
  [/^admin\/repository$/, () => 'admin/repository/repositories'],
  [/^admin\/security$/, () => 'admin/security/users'],
  [/^admin\/support$/, () => 'admin/support/systeminformation'],
  [/^admin\/system$/, () => 'admin/system/tasks'],

  ...masterDetail('admin/repository/repositories', 'admin/repository/repositories'),
  [/^admin\/repository\/blobstores$/, () => 'admin/repository/blobstores'],
  [/^admin\/repository\/blobstores\/create$/, () => 'admin/repository/blobstores/create'],
  [/^admin\/repository\/blobstores\/edit\/([^/]+)\/(.+)$/,
    (m) => `admin/repository/blobstores/${encodeSegment(m[1])}/${encodeSegment(m[2])}`],
  page('admin/repository/blobstores', 'admin/repository/blobstores'),
  ...listCreateEdit('admin/repository/selectors', 'admin/repository/selectors'),
  ...listCreateEdit('admin/repository/cleanuppolicies', 'admin/repository/cleanup-policies'),
  [/^admin\/repository\/routingrules\/preview$/, () => 'admin/repository/routing-rules/preview'],
  ...listCreateEdit('admin/repository/routingrules', 'admin/repository/routing-rules'),
  page('admin/repository/datastore', 'admin/repository/datastore'),
  page('admin/repository/proprietary', 'admin/repository/proprietary'),

  ...listCreateEdit('admin/security/privileges', 'admin/security/privileges'),
  ...listCreateEdit('admin/security/roles', 'admin/security/roles'),
  ...masterDetail('admin/security/users', 'admin/security/users'),
  ...masterDetail('admin/security/ldap', 'admin/security/ldap'),
  page('admin/security/anonymous', 'admin/security/anonymous'),
  page('admin/security/realms', 'admin/security/realms'),
  page('admin/security/sslcertificates', 'admin/security/sslcertificates'),
  page('admin/security/oauth2', 'admin/security/oauth2'),
  page('admin/security/usertoken', 'admin/security/user-tokens'),
  page('admin/security/satokens', 'admin/security/service-account-tokens'),
  page('admin/security/atlassiancrowd', 'admin/security/crowd'),
  page('admin/security/saml', 'admin/security/saml'),

  page('admin/support/systeminformation', 'admin/support/systeminformation'),
  page('admin/support/status', 'admin/support/metrichealth'),
  page('admin/support/supportzip', 'admin/support/supportzip'),
  page('admin/support/supportrequest', 'admin/support/supportrequest'),
  ...masterDetail('admin/support/logs', 'admin/support/logs',
    (file) => encodePath(safeDecodeURIComponent(file))),
  page('admin/support/recovery', 'admin/support/recoverymode'),
  ...listCreateEdit('admin/support/logging', 'admin/support/logging'),

  ...masterDetail('admin/system/tasks', 'admin/system/tasks'),
  ...masterDetail('admin/system/capabilities-extjs', 'admin/system/capabilities'),
  ...listCreateEdit('admin/system/capabilities', 'admin/system/capabilities'),
  page('admin/system/api', 'browse/api'),
  page('admin/system/emailserver', 'admin/system/emailserver'),
  page('admin/system/http', 'admin/system/http'),
  page('admin/system/nodes', 'admin/system/nodes'),
  page('admin/system/notifications', 'admin/system/notifications'),
  page('admin/system/previewui', 'settings'),

  [/^user$/, () => 'user/account'],
  page('user/account', 'user/account'),
  page('user/accesstokens', 'user/accesstokens'),
  page('user/NuGetApiToken', 'user/nugetapitoken'),
  page('user/usertoken', 'user/usertoken'),
];

const USERS_DETAIL = /^admin\/security\/users:(.+)$/;

/** Astrum One used to live under `#preview/`; old bookmarks and links in the code still point there. */
const PREVIEW_PREFIX = /^\/*preview(?:\/+([\s\S]*))?$/;

/**
 * Hash paths of the Classic UI: the application root and everything under `browse`, `admin` and `user`, which
 * Astrum One now shares. Only paths no Astrum One page claims are mapped; anything else is a missing page.
 */
const CLASSIC_PATH = /^\/*(?:(?:browse|admin|user)(?:[/:=?][\s\S]*)?)?$/;

/**
 * Classic search URLs: `browse/search=<criteria>` and `browse/search/<id>...`, but not the Astrum One search pages
 * `browse/search/custom` and `browse/search/component/...`.
 */
const CLASSIC_SEARCH_PATH = /^\/*browse\/search(?:=[\s\S]*|\/(?!custom$|component\/)[^/][\s\S]*)$/;

/**
 * True for a Classic search URL, with or without a leading '#' or '/'. Astrum One opens a repository at
 * `browse/<name>/<path>`, so these URLs would otherwise open a repository named `search`: the router must map
 * them before it tries its states.
 *
 * @param {string} currentPath e.g. `browse/search/maven`
 * @returns {boolean}
 */
export function isClassicSearchPath(currentPath) {
  return CLASSIC_SEARCH_PATH.test(String(currentPath ?? '').replace(/^#/, '').split('?')[0]);
}

/**
 * True for a hash path in a part of the URL space the Classic UI used, with or without a leading '#' or '/'.
 *
 * @param {string} currentPath e.g. `admin/repository/cleanuppolicies`
 * @returns {boolean}
 */
export function isClassicPath(currentPath) {
  return CLASSIC_PATH.test(String(currentPath ?? '').replace(/^#/, ''));
}

/**
 * The hash path without the `preview/` prefix Astrum One pages used to have, or null when it has none.
 *
 * @param {string} currentPath e.g. `preview/admin/security/users?tab=roles`
 * @returns {?string} e.g. `admin/security/users?tab=roles`; `preview` alone gives an empty path
 */
export function withoutPreviewPrefix(currentPath) {
  const raw = String(currentPath ?? '').replace(/^#/, '');
  const queryIndex = raw.indexOf('?');
  const path = queryIndex === -1 ? raw : raw.slice(0, queryIndex);
  const queryString = queryIndex === -1 ? '' : raw.slice(queryIndex);
  const match = PREVIEW_PREFIX.exec(path);
  return match ? `${match[1] || ''}${queryString}` : null;
}

/**
 * Astrum One hash path (no leading #) that replaces a Classic UI hash path, so old bookmarks and links
 * in emails keep working now that the Classic UI is retired. A path under the old `preview/` prefix
 * loses the prefix; a Classic path that Astrum One has no page for opens the dashboard.
 *
 * Astrum One shares the root of the URL space with the Classic UI, so this must only see paths that no
 * Astrum One page matches: the router applies it after its states.
 *
 * @param {string} currentPath a hash path with or without a leading '#' or '/', e.g. `admin/repository/cleanuppolicies`
 * @returns {string} e.g. `admin/repository/cleanup-policies`
 */
export function heritageToPreviewPath(currentPath) {
  const raw = String(currentPath ?? '').replace(/^#/, '').replace(/^\/+/, '');
  const queryIndex = raw.indexOf('?');
  const path = (queryIndex === -1 ? raw : raw.slice(0, queryIndex)).replace(/\/+$/, '');
  const queryString = queryIndex === -1 ? '' : raw.slice(queryIndex + 1);

  const astrumOnePath = withoutPreviewPrefix(raw);
  if (astrumOnePath !== null) {
    return astrumOnePath.split('?')[0] ? astrumOnePath : PREVIEW_WELCOME_PATH;
  }

  const usersDetail = USERS_DETAIL.exec(path);
  if (usersDetail) {
    return userPath(usersDetail[1]);
  }

  for (const [pattern, toPreview] of CLASSIC_TO_PREVIEW_ROUTES) {
    const match = pattern.exec(path);
    if (match) {
      const target = toPreview(match);
      return queryString && !target.includes('?') ? `${target}?${queryString}` : target;
    }
  }
  return PREVIEW_WELCOME_PATH;
}
