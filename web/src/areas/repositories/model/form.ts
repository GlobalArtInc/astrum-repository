import { ApiError, type FieldError } from '../../../api';
import type { Translate } from '../../../i18n';
import {
  clone,
  getPath,
  isObject,
  type Json,
  type JsonObject,
  type RepositoryDocument,
  sameJson,
  setPath,
} from './document';
import { type Spec, sections } from './spec';

export interface RepositoryValues {
  name: string;
  online: boolean;
  blobStoreName: string;
  strictContentTypeValidation: boolean;
  writePolicy: string;
  latestPolicy: boolean;
  remoteUrl: string;
  contentMaxAge: number | null;
  metadataMaxAge: number | null;
  negativeCacheEnabled: boolean;
  timeToLive: number | null;
  blocked: boolean;
  autoBlock: boolean;
  retries: number | null;
  timeout: number | null;
  userAgentSuffix: string;
  enableCircularRedirects: boolean;
  enableCookies: boolean;
  authType: string;
  username: string;
  password: string;
  ntlmHost: string;
  ntlmDomain: string;
  bearerToken: string;
  preemptive: boolean;
  memberNames: string[];
  writableMember: string;
  versionPolicy: string;
  layoutPolicy: string;
  mavenContentDisposition: string;
  removeNonCataloged: boolean;
  removeQuarantined: boolean;
  pypiRemoveQuarantined: boolean;
  v1Enabled: boolean;
  forceBasicAuth: boolean;
  httpPort: number | null;
  httpsPort: number | null;
  pathEnabled: boolean;
  subdomain: string;
  indexType: string;
  indexUrl: string;
  cacheForeignLayers: boolean;
  foreignLayerUrlWhitelist: string;
  rawContentDisposition: string;
  forwardQueryParameters: boolean;
  excludedQueryParameters: string;
}

export type FieldKey = keyof RepositoryValues;
type Kind = 'name' | 'text' | 'secret' | 'bool' | 'int' | 'enum' | 'lines' | 'list';

interface FieldDef {
  key: FieldKey;
  path: readonly string[];
  kind: Kind;
  fallback?: boolean;
  auth?: boolean;
}

const AUTHENTICATION = ['httpClient', 'authentication'] as const;

export const FIELDS: readonly FieldDef[] = [
  { key: 'name', path: ['name'], kind: 'name' },
  { key: 'online', path: ['online'], kind: 'bool', fallback: true },
  { key: 'blobStoreName', path: ['storage', 'blobStoreName'], kind: 'text' },
  { key: 'strictContentTypeValidation', path: ['storage', 'strictContentTypeValidation'], kind: 'bool' },
  { key: 'writePolicy', path: ['storage', 'writePolicy'], kind: 'enum' },
  { key: 'latestPolicy', path: ['storage', 'latestPolicy'], kind: 'bool' },
  { key: 'remoteUrl', path: ['proxy', 'remoteUrl'], kind: 'text' },
  { key: 'contentMaxAge', path: ['proxy', 'contentMaxAge'], kind: 'int' },
  { key: 'metadataMaxAge', path: ['proxy', 'metadataMaxAge'], kind: 'int' },
  { key: 'negativeCacheEnabled', path: ['negativeCache', 'enabled'], kind: 'bool' },
  { key: 'timeToLive', path: ['negativeCache', 'timeToLive'], kind: 'int' },
  { key: 'blocked', path: ['httpClient', 'blocked'], kind: 'bool' },
  { key: 'autoBlock', path: ['httpClient', 'autoBlock'], kind: 'bool' },
  { key: 'retries', path: ['httpClient', 'connection', 'retries'], kind: 'int' },
  { key: 'timeout', path: ['httpClient', 'connection', 'timeout'], kind: 'int' },
  { key: 'userAgentSuffix', path: ['httpClient', 'connection', 'userAgentSuffix'], kind: 'text' },
  {
    key: 'enableCircularRedirects',
    path: ['httpClient', 'connection', 'enableCircularRedirects'],
    kind: 'bool',
  },
  { key: 'enableCookies', path: ['httpClient', 'connection', 'enableCookies'], kind: 'bool' },
  { key: 'authType', path: [...AUTHENTICATION, 'type'], kind: 'enum' },
  { key: 'username', path: [...AUTHENTICATION, 'username'], kind: 'text', auth: true },
  { key: 'password', path: [...AUTHENTICATION, 'password'], kind: 'secret', auth: true },
  { key: 'ntlmHost', path: [...AUTHENTICATION, 'ntlmHost'], kind: 'text', auth: true },
  { key: 'ntlmDomain', path: [...AUTHENTICATION, 'ntlmDomain'], kind: 'text', auth: true },
  { key: 'bearerToken', path: [...AUTHENTICATION, 'bearerToken'], kind: 'secret', auth: true },
  { key: 'preemptive', path: [...AUTHENTICATION, 'preemptive'], kind: 'bool', auth: true },
  { key: 'memberNames', path: ['group', 'memberNames'], kind: 'list' },
  { key: 'writableMember', path: ['group', 'writableMember'], kind: 'enum' },
  { key: 'versionPolicy', path: ['maven', 'versionPolicy'], kind: 'enum' },
  { key: 'layoutPolicy', path: ['maven', 'layoutPolicy'], kind: 'enum' },
  { key: 'mavenContentDisposition', path: ['maven', 'contentDisposition'], kind: 'enum' },
  { key: 'removeNonCataloged', path: ['npm', 'removeNonCataloged'], kind: 'bool' },
  { key: 'removeQuarantined', path: ['npm', 'removeQuarantined'], kind: 'bool' },
  { key: 'pypiRemoveQuarantined', path: ['pypi', 'removeQuarantined'], kind: 'bool' },
  { key: 'v1Enabled', path: ['docker', 'v1Enabled'], kind: 'bool' },
  { key: 'forceBasicAuth', path: ['docker', 'forceBasicAuth'], kind: 'bool', fallback: true },
  { key: 'httpPort', path: ['docker', 'httpPort'], kind: 'int' },
  { key: 'httpsPort', path: ['docker', 'httpsPort'], kind: 'int' },
  { key: 'pathEnabled', path: ['docker', 'pathEnabled'], kind: 'bool' },
  { key: 'subdomain', path: ['docker', 'subdomain'], kind: 'text' },
  { key: 'indexType', path: ['dockerProxy', 'indexType'], kind: 'enum' },
  { key: 'indexUrl', path: ['dockerProxy', 'indexUrl'], kind: 'text' },
  { key: 'cacheForeignLayers', path: ['dockerProxy', 'cacheForeignLayers'], kind: 'bool' },
  { key: 'foreignLayerUrlWhitelist', path: ['dockerProxy', 'foreignLayerUrlWhitelist'], kind: 'lines' },
  { key: 'rawContentDisposition', path: ['raw', 'contentDisposition'], kind: 'enum' },
  { key: 'forwardQueryParameters', path: ['raw', 'forwardQueryParameters'], kind: 'bool' },
  { key: 'excludedQueryParameters', path: ['raw', 'excludedQueryParameters'], kind: 'lines' },
];

function read(def: FieldDef, value: Json | undefined): RepositoryValues[FieldKey] {
  switch (def.kind) {
    case 'name':
    case 'text':
    case 'enum':
      return typeof value === 'string' ? value : '';
    case 'secret':
      return '';
    case 'bool':
      return typeof value === 'boolean' ? value : (def.fallback ?? false);
    case 'int':
      return typeof value === 'number' ? value : null;
    case 'lines':
      return Array.isArray(value) ? value.map(String).join('\n') : '';
    case 'list':
      return Array.isArray(value) ? value.map(String) : [];
  }
}

export function splitLines(text: string): string[] {
  return text
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '');
}

function write(def: FieldDef, value: RepositoryValues[FieldKey]): Json {
  switch (def.kind) {
    case 'name':
      return String(value).trim();
    case 'text': {
      const text = String(value).trim();
      return text === '' ? null : text;
    }
    case 'enum':
      return value === '' ? null : String(value);
    case 'secret':
      return String(value);
    case 'bool':
      return Boolean(value);
    case 'int':
      return typeof value === 'number' ? value : null;
    case 'lines':
      return splitLines(String(value));
    case 'list':
      return Array.isArray(value) ? [...value] : [];
  }
}

export function fromDocument(document: RepositoryDocument): RepositoryValues {
  const values: Record<string, unknown> = {};
  for (const def of FIELDS) {
    values[def.key] = read(def, getPath(document, def.path));
  }
  return values as unknown as RepositoryValues;
}

export function toDocument(values: RepositoryValues, original: RepositoryDocument): RepositoryDocument {
  const next = clone(original);
  const before = fromDocument(original);
  for (const def of FIELDS) {
    const value = values[def.key];
    if (sameJson(value, before[def.key])) {
      continue;
    }
    if (def.auth && values.authType === '') {
      continue;
    }
    if (def.key === 'authType' && value === '') {
      setPath(next, AUTHENTICATION, null);
      continue;
    }
    setPath(next, def.path, write(def, value));
  }
  return next;
}

export function changedPaths(values: RepositoryValues, original: RepositoryDocument): string[] {
  const before = fromDocument(original);
  return FIELDS.filter((def) => !sameJson(values[def.key], before[def.key])).map((def) => def.path.join('.'));
}

export function storedSecrets(document: RepositoryDocument): { password: boolean; bearerToken: boolean } {
  const authentication = getPath(document, AUTHENTICATION);
  const type = isObject(authentication) ? authentication.type : null;
  return {
    password: type === 'username' || type === 'ntlm',
    bearerToken: type === 'bearerToken',
  };
}

export function documentMembers(document: RepositoryDocument): string[] {
  const members = getPath(document, ['group', 'memberNames']);
  return Array.isArray(members) ? members.map(String) : [];
}

export function hasSection(document: RepositoryDocument, key: string): boolean {
  return isObject(document[key]);
}

export function template(spec: Spec, blobStoreName = ''): RepositoryDocument {
  const document: JsonObject = {
    name: '',
    online: true,
    storage: { blobStoreName, strictContentTypeValidation: true },
  };
  const storage = document.storage as JsonObject;
  if (sections.writePolicy(spec)) {
    storage.writePolicy = 'ALLOW_ONCE';
  }
  if (sections.latestPolicy(spec)) {
    storage.latestPolicy = false;
  }
  if (sections.proxy(spec)) {
    document.proxy = {
      remoteUrl: '',
      contentMaxAge: spec.format === 'maven2' ? -1 : 1440,
      metadataMaxAge: 1440,
    };
    document.negativeCache = { enabled: true, timeToLive: 1440 };
    document.httpClient = {
      blocked: false,
      autoBlock: true,
      connection: {
        retries: null,
        userAgentSuffix: null,
        timeout: null,
        enableCircularRedirects: false,
        enableCookies: false,
      },
      authentication: null,
    };
  }
  if (sections.group(spec)) {
    document.group = { memberNames: [] };
  }
  if (sections.maven(spec)) {
    document.maven = { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT', contentDisposition: 'INLINE' };
  }
  if (sections.docker(spec)) {
    document.docker = { v1Enabled: false, forceBasicAuth: true, pathEnabled: true };
  }
  if (sections.dockerProxy(spec)) {
    document.dockerProxy = { indexType: 'REGISTRY', cacheForeignLayers: false, foreignLayerUrlWhitelist: [] };
  }
  if (spec.format === 'npm' && spec.type === 'proxy') {
    document.npm = { removeNonCataloged: false, removeQuarantined: false };
  }
  if (sections.raw(spec)) {
    document.raw = sections.rawQuery(spec)
      ? { contentDisposition: 'ATTACHMENT', forwardQueryParameters: false, excludedQueryParameters: [] }
      : { contentDisposition: 'ATTACHMENT' };
  }
  return document;
}

export const NAME_PATTERN = /^[A-Za-z0-9-][A-Za-z0-9._-]*$/;
export const MAX_NAME_LENGTH = 200;

export function isWebUrl(text: string): boolean {
  try {
    const url = new URL(text.trim());
    return (url.protocol === 'http:' || url.protocol === 'https:') && url.hostname !== '';
  } catch {
    return false;
  }
}

export interface ValidationContext {
  spec: Spec;
  creating: boolean;
  stored: { password: boolean; bearerToken: boolean };
}

type Errors = Partial<Record<FieldKey, string>>;

function between(value: number | null, min: number, max: number): boolean {
  return value !== null && Number.isInteger(value) && value >= min && value <= max;
}

export function validate(values: RepositoryValues, context: ValidationContext, t: Translate): Errors {
  const { spec } = context;
  const errors: Errors = {};
  if (context.creating) {
    const name = values.name.trim();
    if (name === '') {
      errors.name = t('repositories.form.errors.nameRequired');
    } else if (!NAME_PATTERN.test(name)) {
      errors.name = t('repositories.form.errors.namePattern');
    } else if (name.length > MAX_NAME_LENGTH) {
      errors.name = t('repositories.form.errors.nameLength', { max: MAX_NAME_LENGTH });
    }
    if (values.blobStoreName.trim() === '') {
      errors.blobStoreName = t('repositories.form.errors.blobStoreRequired');
    }
  }
  if (sections.writePolicy(spec) && values.writePolicy === '') {
    errors.writePolicy = t('repositories.form.errors.writePolicyRequired');
  }
  if (sections.proxy(spec)) {
    if (values.remoteUrl.trim() === '') {
      errors.remoteUrl = t('repositories.form.errors.remoteUrlRequired');
    } else if (!isWebUrl(values.remoteUrl)) {
      errors.remoteUrl = t('repositories.form.errors.urlInvalid');
    }
    for (const key of ['contentMaxAge', 'metadataMaxAge'] as const) {
      if (!between(values[key], -1, 2_147_483_647)) {
        errors[key] = t('repositories.form.errors.maxAge');
      }
    }
    if (!between(values.timeToLive, 0, 2_147_483_647)) {
      errors.timeToLive = t('repositories.form.errors.timeToLive');
    }
    if (values.retries !== null && !between(values.retries, 0, 10)) {
      errors.retries = t('repositories.form.errors.retries');
    }
    if (values.timeout !== null && !between(values.timeout, 1, 3600)) {
      errors.timeout = t('repositories.form.errors.timeout');
    }
    if (values.authType === 'username' || values.authType === 'ntlm') {
      if (values.username.trim() === '') {
        errors.username = t('repositories.form.errors.usernameRequired');
      }
      if (values.password === '' && !context.stored.password) {
        errors.password = t('repositories.form.errors.passwordRequired');
      }
    }
    if (values.authType === 'bearerToken' && values.bearerToken === '' && !context.stored.bearerToken) {
      errors.bearerToken = t('repositories.form.errors.bearerTokenRequired');
    }
  }
  if (sections.group(spec) && values.memberNames.length === 0) {
    errors.memberNames = t('repositories.form.errors.membersRequired');
  }
  if (sections.docker(spec)) {
    for (const key of ['httpPort', 'httpsPort'] as const) {
      if (values[key] !== null && !between(values[key], 1, 65535)) {
        errors[key] = t('repositories.form.errors.port');
      }
    }
    if (!errors.httpsPort && values.httpPort !== null && values.httpPort === values.httpsPort) {
      errors.httpsPort = t('repositories.form.errors.portsDiffer');
    }
  }
  if (sections.dockerProxy(spec) && values.indexType !== 'REGISTRY') {
    if (values.indexType === 'CUSTOM' && values.indexUrl.trim() === '') {
      errors.indexUrl = t('repositories.form.errors.indexUrlRequired');
    } else if (values.indexUrl.trim() !== '' && !isWebUrl(values.indexUrl)) {
      errors.indexUrl = t('repositories.form.errors.urlInvalid');
    }
  }
  return errors;
}

const SAME_NAME: ReadonlySet<string> = new Set(FIELDS.map((def) => def.key));

export function formField(id: string, spec: Spec): FieldKey | null {
  const name = id.replace(/^PARAMETER\s+/, '').trim();
  const last = name.split('.').pop() ?? name;
  switch (last) {
    case 'enabled':
      return 'negativeCacheEnabled';
    case 'type':
      return 'authType';
    case 'contentDisposition':
      return spec.format === 'raw' ? 'rawContentDisposition' : 'mavenContentDisposition';
    case 'removeQuarantined':
      return spec.format === 'pypi' ? 'pypiRemoveQuarantined' : 'removeQuarantined';
    default:
      return SAME_NAME.has(last) ? (last as FieldKey) : null;
  }
}

export function onFormFields(error: unknown, spec: Spec): unknown {
  if (!(error instanceof ApiError) || error.status !== 400 || error.fieldErrors.length === 0) {
    return error;
  }
  const fieldErrors: FieldError[] = error.fieldErrors.map((problem) => {
    const key = formField(problem.id, spec);
    if (key) {
      return { id: key, message: problem.message };
    }
    const name = problem.id.replace(/^PARAMETER\s+/, '').trim();
    return { id: '*', message: name && name !== '*' ? `${name}: ${problem.message}` : problem.message };
  });
  return new ApiError(error.status, error.message, { fieldErrors });
}
