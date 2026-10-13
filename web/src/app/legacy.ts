const ADMIN_SECTIONS: Record<string, Record<string, string>> = {
  repository: {
    repositories: '/admin/repository/repositories',
    blobstores: '/admin/repository/blobstores',
    selectors: '/admin/repository/selectors',
    cleanuppolicies: '/admin/repository/cleanup-policies',
    'cleanup-policies': '/admin/repository/cleanup-policies',
    routingrules: '/admin/repository/routing-rules',
    'routing-rules': '/admin/repository/routing-rules',
    proprietary: '/admin/repository/proprietary',
  },
  security: {
    privileges: '/admin/security/privileges',
    roles: '/admin/security/roles',
    users: '/admin/security/users',
    anonymous: '/admin/security/anonymous',
    ldap: '/admin/security/ldap',
    realms: '/admin/security/realms',
    sslcertificates: '/admin/security/sslcertificates',
    oauth2: '/admin/security/oauth2',
  },
  support: {
    logs: '/admin/support/logs',
    logging: '/admin/support/logging',
    systeminformation: '/admin/support/system-information',
    metrichealth: '/admin/support/metric-health',
    recoverymode: '/admin/support/recoverymode',
    supportzip: '/admin/support/support-zip',
  },
  system: {
    tasks: '/admin/system/tasks',
    capabilities: '/admin/system/capabilities',
    notifications: '/admin/system/notifications',
    emailserver: '/admin/system/emailserver',
    http: '/admin/system/http',
    nodes: '/admin/system/nodes',
    api: '/api',
  },
};

const SECTION_HOMES: Record<string, string> = {
  repository: '/admin/repository/repositories',
  security: '/admin/security/privileges',
  support: '/admin/support/logs',
  system: '/admin/system/tasks',
};

const USER_PAGES: Record<string, string> = {
  account: '/user/account',
  accesstokens: '/user/access-tokens',
  'access-tokens': '/user/access-tokens',
  nugetapitoken: '/user/nuget-api-key',
};

const CLASSIC_SEARCH_FORMATS: Record<string, string> = {
  maven: 'maven2',
  maven2: 'maven2',
  npm: 'npm',
  nuget: 'nuget',
  docker: 'docker',
  pypi: 'pypi',
  raw: 'raw',
  rubygems: 'rubygems',
  gitlfs: 'gitlfs',
  bower: 'bower',
  yum: 'yum',
  apt: 'apt',
  helm: 'helm',
  go: 'go',
  golang: 'go',
  conda: 'conda',
  conan: 'conan',
  cocoapods: 'cocoapods',
  r: 'r',
  p2: 'p2',
  cargo: 'cargo',
  composer: 'composer',
  alpine: 'alpine',
  swift: 'swift',
  terraform: 'terraform',
  huggingface: 'huggingface',
};

const CLASSIC_CRITERIA: Record<string, string> = {
  keyword: 'q',
  format: 'format',
  repository_name: 'repository',
  'group.raw': 'group',
  'name.raw': 'name',
  version: 'version',
};

const ROOTS = new Set(['browse', 'admin', 'settings', 'user', 'preview', 'login', '404']);

function decode(text: string): string {
  try {
    return decodeURIComponent(text);
  } catch {
    return text;
  }
}

function encodePath(path: string): string {
  return path
    .split('/')
    .filter((segment) => segment !== '')
    .map((segment) => encodeURIComponent(segment))
    .join('/');
}

function withQuery(path: string, query: string): string {
  return query ? `${path}?${query}` : path;
}

function classicCriteria(text: string): string {
  const params = new URLSearchParams();
  for (const pair of decode(text).split('&')) {
    const [key = '', ...rest] = pair.split('=');
    const name = CLASSIC_CRITERIA[key];
    const value = rest.join('=');
    if (name && value) {
      params.set(name, value);
    }
  }
  return params.toString();
}

function decodeReturnTo(value: string | null): string | null {
  if (!value) {
    return null;
  }
  try {
    return atob(decode(value));
  } catch {
    return null;
  }
}

function mapBrowse(rest: string, query: string): string {
  if (rest === '' || rest === 'welcome') {
    return '/';
  }
  if (rest === 'browse') {
    return withQuery('/browse', query);
  }
  if (rest.startsWith('browse:')) {
    const [repository = '', ...path] = rest.slice('browse:'.length).split(':');
    const target = [encodeURIComponent(decode(repository)), encodePath(decode(path.join(':')))]
      .filter(Boolean)
      .join('/');
    return `/browse/${target}`;
  }
  if (rest === 'search' || rest.startsWith('search?')) {
    return withQuery('/search', query);
  }
  if (rest.startsWith('search=')) {
    return withQuery('/search', classicCriteria(rest.slice('search='.length)));
  }
  if (rest.startsWith('search/')) {
    const sub = rest.slice('search/'.length);
    if (sub === 'custom' || sub.startsWith('component/')) {
      return withQuery(`/search/${sub}`, query);
    }
    const [name = '', criteria = ''] = sub.split('=');
    const params = new URLSearchParams(criteria ? classicCriteria(criteria) : query);
    const format = CLASSIC_SEARCH_FORMATS[name.toLowerCase()];
    if (format) {
      params.set('format', format);
    }
    return withQuery('/search', params.toString());
  }
  if (rest === 'upload') {
    return '/upload';
  }
  if (rest.startsWith('upload:') || rest.startsWith('upload/')) {
    return `/upload/${encodeURIComponent(decode(rest.slice('upload:'.length)))}`;
  }
  if (rest === 'api') {
    return '/api';
  }
  if (rest === 'audit') {
    return withQuery('/audit', query);
  }
  if (rest.startsWith('repository-profile/')) {
    const name = rest.slice('repository-profile/'.length);
    return `/admin/repository/repositories/${encodeURIComponent(decode(name))}/profile`;
  }
  return withQuery(`/browse/${rest.replace(/^\/+/, '')}`, query);
}

function mapAdmin(rest: string, query: string): string {
  if (rest === '') {
    return '/settings';
  }
  const [section = '', page = '', ...more] = rest.split('/');
  if (page === '') {
    return SECTION_HOMES[section] ?? '/';
  }
  const [pageName = '', ...ids] = page.split(':');
  const base = ADMIN_SECTIONS[section]?.[pageName.toLowerCase()];
  if (!base) {
    return '/';
  }
  const tail = [...ids.map((id) => encodeURIComponent(decode(id))), ...more].filter(Boolean).join('/');
  return withQuery(tail ? `${base}/${tail}` : base, query);
}

export function legacyPath(hash: string): string | null {
  let route = hash.replace(/^#/, '').replace(/^\/+/, '');
  if (route === '') {
    return null;
  }
  while (route.startsWith('preview/')) {
    route = route.slice('preview/'.length);
  }
  if (route === 'preview') {
    return '/';
  }
  const questionMark = route.indexOf('?');
  const path = questionMark >= 0 ? route.slice(0, questionMark) : route;
  const query = questionMark >= 0 ? route.slice(questionMark + 1) : '';
  const [root = '', ...restParts] = path.split('/');
  if (!ROOTS.has(root)) {
    return null;
  }
  const rest = restParts.join('/');
  if (root === 'browse') {
    return mapBrowse(rest, query);
  }
  if (root === 'admin') {
    return mapAdmin(rest, query);
  }
  if (root === 'settings') {
    return '/settings';
  }
  if (root === 'user') {
    return USER_PAGES[rest.toLowerCase()] ?? '/';
  }
  if (root === 'login') {
    const returnTo = decodeReturnTo(new URLSearchParams(query).get('returnTo'));
    const target = returnTo ? (legacyPath(returnTo) ?? '/') : '/';
    return target === '/' ? '/login' : `/login?returnTo=${encodeURIComponent(target)}`;
  }
  return '/404';
}
