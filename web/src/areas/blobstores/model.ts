import { ApiError, type FieldError } from '../../api';
import type { Translate } from '../../i18n';
import type { StoreType } from './api';

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type JsonObject = { [key: string]: Json };
export type StoreDocument = JsonObject;

export const MB = 1024 * 1024;
export const QUOTA_TYPES = ['spaceUsedQuota', 'spaceRemainingQuota'] as const;
export const FILL_POLICIES = ['writeToFirst', 'roundRobin'] as const;
export const ENCRYPTION_TYPES = ['s3ManagedEncryption', 'kmsManagedEncryption'] as const;

export interface StoreValues {
  name: string;
  path: string;
  quotaEnabled: boolean;
  quotaType: string;
  quotaLimit: number | null;
  region: string;
  bucket: string;
  prefix: string;
  expiration: number | null;
  accessKeyId: string;
  secretAccessKey: string;
  role: string;
  sessionToken: string;
  encryptionType: string;
  encryptionKey: string;
  endpoint: string;
  forcePathStyle: boolean;
  members: string[];
  fillPolicy: string;
}

type Key = keyof StoreValues;
type Kind = 'name' | 'text' | 'secret' | 'bool' | 'int' | 'list' | 'enum';

interface FieldDef {
  key: Key;
  path: readonly string[];
  kind: Kind;
  types: readonly StoreType[];
}

const BUCKET = ['bucketConfiguration', 'bucket'] as const;
const SECURITY = ['bucketConfiguration', 'bucketSecurity'] as const;
const ENCRYPTION = ['bucketConfiguration', 'encryption'] as const;
const CONNECTION = ['bucketConfiguration', 'advancedBucketConnection'] as const;
const ALL: readonly StoreType[] = ['file', 's3', 'group'];

const FIELDS: readonly FieldDef[] = [
  { key: 'name', path: ['name'], kind: 'name', types: ALL },
  { key: 'path', path: ['path'], kind: 'text', types: ['file'] },
  { key: 'region', path: [...BUCKET, 'region'], kind: 'text', types: ['s3'] },
  { key: 'bucket', path: [...BUCKET, 'name'], kind: 'text', types: ['s3'] },
  { key: 'prefix', path: [...BUCKET, 'prefix'], kind: 'text', types: ['s3'] },
  { key: 'expiration', path: [...BUCKET, 'expiration'], kind: 'int', types: ['s3'] },
  { key: 'accessKeyId', path: [...SECURITY, 'accessKeyId'], kind: 'text', types: ['s3'] },
  { key: 'secretAccessKey', path: [...SECURITY, 'secretAccessKey'], kind: 'secret', types: ['s3'] },
  { key: 'role', path: [...SECURITY, 'role'], kind: 'text', types: ['s3'] },
  { key: 'sessionToken', path: [...SECURITY, 'sessionToken'], kind: 'secret', types: ['s3'] },
  { key: 'encryptionType', path: [...ENCRYPTION, 'encryptionType'], kind: 'enum', types: ['s3'] },
  { key: 'encryptionKey', path: [...ENCRYPTION, 'encryptionKey'], kind: 'text', types: ['s3'] },
  { key: 'endpoint', path: [...CONNECTION, 'endpoint'], kind: 'text', types: ['s3'] },
  { key: 'forcePathStyle', path: [...CONNECTION, 'forcePathStyle'], kind: 'bool', types: ['s3'] },
  { key: 'members', path: ['members'], kind: 'list', types: ['group'] },
  { key: 'fillPolicy', path: ['fillPolicy'], kind: 'enum', types: ['group'] },
];

function isObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function getPath(document: JsonObject, path: readonly string[]): Json | undefined {
  let current: Json | undefined = document;
  for (const key of path) {
    if (!isObject(current)) {
      return undefined;
    }
    current = current[key];
  }
  return current;
}

function setPath(document: JsonObject, path: readonly string[], value: Json): void {
  let current = document;
  for (const key of path.slice(0, -1)) {
    const next = current[key];
    if (isObject(next)) {
      current = next;
    } else {
      const created: JsonObject = {};
      current[key] = created;
      current = created;
    }
  }
  const last = path[path.length - 1];
  if (last !== undefined) {
    current[last] = value;
  }
}

function same(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function read(def: FieldDef, value: Json | undefined): StoreValues[Key] {
  switch (def.kind) {
    case 'name':
    case 'text':
    case 'enum':
      return typeof value === 'string' ? value : '';
    case 'secret':
      return '';
    case 'bool':
      return value === true;
    case 'int':
      return typeof value === 'number' ? value : null;
    case 'list':
      return Array.isArray(value) ? value.map(String) : [];
  }
}

function write(def: FieldDef, value: StoreValues[Key]): Json {
  switch (def.kind) {
    case 'name':
    case 'text':
      return String(value).trim();
    case 'enum':
      return value === '' ? null : String(value);
    case 'secret':
      return String(value);
    case 'bool':
      return Boolean(value);
    case 'int':
      return typeof value === 'number' ? value : null;
    case 'list':
      return Array.isArray(value) ? [...value] : [];
  }
}

export function quotaMegabytes(bytes: number): number {
  return Math.round((bytes / MB) * 1_000_000) / 1_000_000;
}

export function fromDocument(document: StoreDocument): StoreValues {
  const values: Record<string, unknown> = {};
  for (const def of FIELDS) {
    values[def.key] = read(def, getPath(document, def.path));
  }
  const quota = document.softQuota;
  values.quotaEnabled = isObject(quota);
  values.quotaType = isObject(quota) && typeof quota.type === 'string' ? quota.type : QUOTA_TYPES[0];
  values.quotaLimit = isObject(quota) && typeof quota.limit === 'number' ? quotaMegabytes(quota.limit) : null;
  return values as unknown as StoreValues;
}

export function toDocument(values: StoreValues, original: StoreDocument, type: StoreType): StoreDocument {
  const next = JSON.parse(JSON.stringify(original)) as StoreDocument;
  const before = fromDocument(original);
  for (const def of FIELDS) {
    if (!def.types.includes(type) || same(values[def.key], before[def.key])) {
      continue;
    }
    if (def.key === 'encryptionType' && values.encryptionType === '') {
      setPath(next, ENCRYPTION, null);
      continue;
    }
    if (def.key === 'encryptionKey' && values.encryptionType === '') {
      continue;
    }
    setPath(next, def.path, write(def, values[def.key]));
  }
  const quotaChanged =
    values.quotaEnabled !== before.quotaEnabled ||
    (values.quotaEnabled &&
      (values.quotaType !== before.quotaType || values.quotaLimit !== before.quotaLimit));
  if (quotaChanged) {
    const originalQuota = original.softQuota;
    const keepBytes =
      isObject(originalQuota) &&
      values.quotaLimit === before.quotaLimit &&
      typeof originalQuota.limit === 'number';
    next.softQuota = values.quotaEnabled
      ? {
          type: values.quotaType,
          limit:
            keepBytes && isObject(originalQuota)
              ? (originalQuota.limit as number)
              : Math.round((values.quotaLimit ?? 0) * MB),
        }
      : null;
  }
  return next;
}

export function template(type: StoreType): StoreDocument {
  switch (type) {
    case 'file':
      return { name: '', path: '', softQuota: null };
    case 's3':
      return {
        name: '',
        type: 'S3',
        softQuota: null,
        bucketConfiguration: {
          bucket: { region: 'DEFAULT', name: '', prefix: '', expiration: 3 },
          bucketSecurity: null,
          encryption: null,
          advancedBucketConnection: null,
        },
      };
    case 'group':
      return { name: '', softQuota: null, members: [], fillPolicy: 'writeToFirst' };
  }
}

export function storedSecret(document: StoreDocument): string | null {
  const key = getPath(document, [...SECURITY, 'accessKeyId']);
  return typeof key === 'string' && key !== '' ? key : null;
}

export const NAME_PATTERN = /^[A-Za-z0-9-][A-Za-z0-9._-]*$/;
export const BUCKET_PATTERN = /^[a-z0-9][a-z0-9.-]{1,61}[a-z0-9]$/;

export interface StoreContext {
  type: StoreType;
  creating: boolean;
  storedKeyId: string | null;
}

function isWebUrl(text: string): boolean {
  try {
    const url = new URL(text.trim());
    return (url.protocol === 'http:' || url.protocol === 'https:') && url.hostname !== '';
  } catch {
    return false;
  }
}

export function validate(
  values: StoreValues,
  context: StoreContext,
  t: Translate,
): Partial<Record<Key, string>> {
  const errors: Partial<Record<Key, string>> = {};
  if (context.creating) {
    const name = values.name.trim();
    if (name === '') {
      errors.name = t('blobstores.form.errors.nameRequired');
    } else if (!NAME_PATTERN.test(name)) {
      errors.name = t('blobstores.form.errors.namePattern');
    }
  }
  if (values.quotaEnabled) {
    if (values.quotaLimit === null || !(values.quotaLimit > 0)) {
      errors.quotaLimit = t('blobstores.form.errors.quotaLimit');
    }
  }
  if (context.type === 'file' && values.path.trim() === '') {
    errors.path = t('blobstores.form.errors.pathRequired');
  }
  if (context.type === 's3') {
    if (values.region.trim() === '') {
      errors.region = t('blobstores.form.errors.regionRequired');
    }
    if (values.bucket.trim() === '') {
      errors.bucket = t('blobstores.form.errors.bucketRequired');
    } else if (!BUCKET_PATTERN.test(values.bucket.trim())) {
      errors.bucket = t('blobstores.form.errors.bucketPattern');
    }
    if (values.expiration !== null && (!Number.isInteger(values.expiration) || values.expiration < -1)) {
      errors.expiration = t('blobstores.form.errors.expiration');
    }
    const keyId = values.accessKeyId.trim();
    const keeps = keyId !== '' && keyId === context.storedKeyId;
    if (keyId !== '' && values.secretAccessKey === '' && !keeps) {
      errors.secretAccessKey = t('blobstores.form.errors.secretRequired');
    }
    if (keyId === '' && (values.secretAccessKey !== '' || values.sessionToken !== '')) {
      errors.accessKeyId = t('blobstores.form.errors.keyIdRequired');
    }
    if (values.endpoint.trim() !== '' && !isWebUrl(values.endpoint)) {
      errors.endpoint = t('blobstores.form.errors.endpoint');
    }
  }
  if (context.type === 'group' && values.members.length === 0) {
    errors.members = t('blobstores.form.errors.membersRequired');
  }
  return errors;
}

const FIELD_IDS: Record<string, Key> = {
  name: 'name',
  path: 'path',
  region: 'region',
  bucket: 'bucket',
  prefix: 'prefix',
  expiration: 'expiration',
  accessKeyId: 'accessKeyId',
  secretAccessKey: 'secretAccessKey',
  sessionToken: 'sessionToken',
  role: 'role',
  encryptionType: 'encryptionType',
  encryptionKey: 'encryptionKey',
  endpoint: 'endpoint',
  members: 'members',
  fillPolicy: 'fillPolicy',
};

export function formField(id: string): Key | null {
  const name = id.replace(/^PARAMETER\s+/, '').trim();
  if (name === 'bucketConfiguration.bucket.name') {
    return 'bucket';
  }
  const last = name.split('.').pop() ?? name;
  return FIELD_IDS[last] ?? null;
}

export function onFormFields(error: unknown): unknown {
  if (!(error instanceof ApiError) || error.status !== 400 || error.fieldErrors.length === 0) {
    return error;
  }
  const fieldErrors: FieldError[] = error.fieldErrors.map((problem) => ({
    id: formField(problem.id) ?? '*',
    message: problem.message,
  }));
  return new ApiError(error.status, error.message, { fieldErrors });
}
