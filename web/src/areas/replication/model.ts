import type { Translate } from '../../i18n';
import type { Errors } from '../../ui';
import type { ComponentItem, RemoteBody, Replication, ReplicationBody, ReplicationRemote } from './api';

export const MIB = 1024 * 1024;
export const DEFAULT_CONCURRENCY = 2;
export const MAX_CONCURRENCY = 16;
export const DEFAULT_SETTLE_SECONDS = 30;
export const MAX_SETTLE_SECONDS = 3600;
export const DEFAULT_RECONCILE_MINUTES = 360;
export const MIN_RECONCILE_MINUTES = 5;
export const PREVIEW_LIMIT = 20;

const NAME_MAX = 100;
const URL_MAX = 2048;
const TOKEN_MAX = 1024;
const REPOSITORY_MAX = 200;
const URL_FORBIDDEN = /[\s@\\?#]/;
const REPOSITORY_SHAPE = /^[A-Za-z0-9-][A-Za-z0-9_.-]*$/;

export interface RemoteValues {
  name: string;
  url: string;
  token: string;
  useTrustStore: boolean;
  allowPrivateNetwork: boolean;
  maxConcurrency: number | null;
  bandwidthMiB: number | null;
}

export interface ReplicationValues {
  name: string;
  sourceRepository: string;
  remoteId: string;
  remoteRepository: string;
  enabled: boolean;
  nameFilter: string;
  versionFilter: string;
  propagateDeletes: boolean;
  settleSeconds: number | null;
  reconcileMinutes: number | null;
  alertChannels: string[];
}

export interface PreviewRow {
  key: string;
  name: string;
  version: string;
  sent: boolean;
}

export type Preview =
  | { kind: 'rows'; rows: PreviewRow[] }
  | { kind: 'invalid'; field: 'nameFilter' | 'versionFilter' };

export function bytesFromMiB(mib: number): number {
  return Math.round(mib * MIB);
}

export function miBFromBytes(bytes: number): number {
  return bytes / MIB;
}

function hasControl(text: string): boolean {
  return Array.from(text).some((character) => {
    const code = character.charCodeAt(0);
    return code < 32 || code === 127;
  });
}

function printableAscii(text: string): boolean {
  return Array.from(text).every((character) => {
    const code = character.charCodeAt(0);
    return code > 32 && code < 127;
  });
}

function nameProblem(t: Translate, value: string): string | undefined {
  const text = value.trim();
  if (text === '') {
    return t('replication.validation.required');
  }
  if (Array.from(text).length > NAME_MAX) {
    return t('replication.validation.tooLong', { max: NAME_MAX });
  }
  if (hasControl(text)) {
    return t('replication.validation.controlChars');
  }
  return undefined;
}

function rangeProblem(t: Translate, value: number | null, low: number, high: number): string | undefined {
  if (value === null || !Number.isInteger(value) || value < low || value > high) {
    return t('replication.validation.range', { min: low, max: high });
  }
  return undefined;
}

function minimumProblem(t: Translate, value: number | null, low: number): string | undefined {
  if (value === null || !Number.isInteger(value) || value < low) {
    return t('replication.validation.atLeast', { min: low });
  }
  return undefined;
}

export function normalizedRemoteUrl(text: string): string {
  const value = text.trim();
  try {
    const url = new URL(value);
    return `${url.protocol}//${url.host}`;
  } catch {
    return value;
  }
}

export function remoteUrlChanged(text: string, remote: ReplicationRemote | null): boolean {
  return remote !== null && normalizedRemoteUrl(text) !== remote.url;
}

export function remoteUrlProblem(
  t: Translate,
  text: string,
  allowPrivateNetwork: boolean,
): string | undefined {
  const value = text.trim();
  if (value === '') {
    return t('replication.validation.required');
  }
  if (value.length > URL_MAX) {
    return t('replication.validation.tooLong', { max: URL_MAX });
  }
  if (URL_FORBIDDEN.test(value) || hasControl(value)) {
    return t('replication.validation.url');
  }
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return t('replication.validation.url');
  }
  const schemeAllowed = url.protocol === 'https:' || (url.protocol === 'http:' && allowPrivateNetwork);
  if (!schemeAllowed) {
    return t('replication.validation.urlHttps');
  }
  if (url.hostname === '' || url.username !== '' || url.password !== '' || url.pathname !== '/') {
    return t('replication.validation.url');
  }
  return undefined;
}

function tokenProblem(t: Translate, value: string, required: boolean): string | undefined {
  if (value === '') {
    return required ? t('replication.validation.tokenRequired') : undefined;
  }
  if (value.length > TOKEN_MAX) {
    return t('replication.validation.tooLong', { max: TOKEN_MAX });
  }
  if (!printableAscii(value)) {
    return t('replication.validation.token');
  }
  return undefined;
}

function bandwidthProblem(t: Translate, value: number | null): string | undefined {
  if (value === null) {
    return undefined;
  }
  if (!Number.isFinite(value) || bytesFromMiB(value) <= 0) {
    return t('replication.validation.positive');
  }
  return undefined;
}

export function remoteValues(remote: ReplicationRemote | null): RemoteValues {
  return {
    name: remote?.name ?? '',
    url: remote?.url ?? '',
    token: '',
    useTrustStore: remote?.useTrustStore ?? false,
    allowPrivateNetwork: remote?.allowPrivateNetwork ?? false,
    maxConcurrency: remote?.maxConcurrency ?? DEFAULT_CONCURRENCY,
    bandwidthMiB: remote?.maxBytesPerSecond ? miBFromBytes(remote.maxBytesPerSecond) : null,
  };
}

export function validateRemote(
  t: Translate,
  values: RemoteValues,
  remote: ReplicationRemote | null,
): Errors<RemoteValues> {
  const errors: Errors<RemoteValues> = {};
  const name = nameProblem(t, values.name);
  if (name) {
    errors.name = name;
  }
  const url = remoteUrlProblem(t, values.url, values.allowPrivateNetwork);
  if (url) {
    errors.url = url;
  }
  const token = tokenProblem(t, values.token, remote === null || remoteUrlChanged(values.url, remote));
  if (token) {
    errors.token = token;
  }
  const concurrency = rangeProblem(t, values.maxConcurrency, 1, MAX_CONCURRENCY);
  if (concurrency) {
    errors.maxConcurrency = concurrency;
  }
  const bandwidth = bandwidthProblem(t, values.bandwidthMiB);
  if (bandwidth) {
    errors.bandwidthMiB = bandwidth;
  }
  return errors;
}

export function remoteBody(values: RemoteValues): RemoteBody {
  const body: RemoteBody = {
    name: values.name.trim(),
    url: values.url.trim(),
    useTrustStore: values.useTrustStore,
    allowPrivateNetwork: values.allowPrivateNetwork,
    maxConcurrency: values.maxConcurrency ?? DEFAULT_CONCURRENCY,
    maxBytesPerSecond: values.bandwidthMiB === null ? null : bytesFromMiB(values.bandwidthMiB),
  };
  return values.token === '' ? body : { ...body, token: values.token };
}

export function replicationValues(replication: Replication | null): ReplicationValues {
  return {
    name: replication?.name ?? '',
    sourceRepository: replication?.sourceRepository ?? '',
    remoteId: replication?.remoteId ?? '',
    remoteRepository: replication?.remoteRepository ?? '',
    enabled: replication?.enabled ?? true,
    nameFilter: replication?.nameFilter ?? '',
    versionFilter: replication?.versionFilter ?? '',
    propagateDeletes: replication?.propagateDeletes ?? false,
    settleSeconds: replication?.settleSeconds ?? DEFAULT_SETTLE_SECONDS,
    reconcileMinutes: replication?.reconcileMinutes ?? DEFAULT_RECONCILE_MINUTES,
    alertChannels: replication?.alertChannels ?? [],
  };
}

export function validateReplication(t: Translate, values: ReplicationValues): Errors<ReplicationValues> {
  const errors: Errors<ReplicationValues> = {};
  const name = nameProblem(t, values.name);
  if (name) {
    errors.name = name;
  }
  if (values.sourceRepository === '') {
    errors.sourceRepository = t('replication.validation.required');
  }
  if (values.remoteId === '') {
    errors.remoteId = t('replication.validation.required');
  }
  const repository = values.remoteRepository.trim();
  if (repository === '') {
    errors.remoteRepository = t('replication.validation.required');
  } else if (repository.length > REPOSITORY_MAX) {
    errors.remoteRepository = t('replication.validation.tooLong', { max: REPOSITORY_MAX });
  } else if (!REPOSITORY_SHAPE.test(repository)) {
    errors.remoteRepository = t('replication.validation.repository');
  }
  const settle = rangeProblem(t, values.settleSeconds, 0, MAX_SETTLE_SECONDS);
  if (settle) {
    errors.settleSeconds = settle;
  }
  const reconcile = minimumProblem(t, values.reconcileMinutes, MIN_RECONCILE_MINUTES);
  if (reconcile) {
    errors.reconcileMinutes = reconcile;
  }
  return errors;
}

function blankToNull(text: string): string | null {
  return text.trim() === '' ? null : text;
}

export function replicationBody(values: ReplicationValues): ReplicationBody {
  return {
    name: values.name.trim(),
    sourceRepository: values.sourceRepository,
    remoteId: values.remoteId,
    remoteRepository: values.remoteRepository.trim(),
    enabled: values.enabled,
    nameFilter: blankToNull(values.nameFilter),
    versionFilter: blankToNull(values.versionFilter),
    propagateDeletes: values.propagateDeletes,
    settleSeconds: values.settleSeconds ?? DEFAULT_SETTLE_SECONDS,
    reconcileMinutes: values.reconcileMinutes ?? DEFAULT_RECONCILE_MINUTES,
    alertChannels: values.alertChannels,
  };
}

export function componentName(component: Pick<ComponentItem, 'group' | 'name'>): string {
  return component.group ? `${component.group}/${component.name}` : component.name;
}

function compiles(pattern: string): boolean {
  try {
    return new RegExp(pattern) instanceof RegExp;
  } catch {
    return false;
  }
}

export function filterPattern(pattern: string): RegExp | null | undefined {
  if (pattern.trim() === '') {
    return null;
  }
  if (!compiles(pattern)) {
    return undefined;
  }
  return new RegExp(`^(?:${pattern})$`);
}

export function previewFilters(
  components: ComponentItem[],
  nameFilter: string,
  versionFilter: string,
): Preview {
  const names = filterPattern(nameFilter);
  if (names === undefined) {
    return { kind: 'invalid', field: 'nameFilter' };
  }
  const versions = filterPattern(versionFilter);
  if (versions === undefined) {
    return { kind: 'invalid', field: 'versionFilter' };
  }
  const rows = components.slice(0, PREVIEW_LIMIT).map((component) => {
    const name = componentName(component);
    const sent = (names?.test(name) ?? true) && (versions?.test(component.version) ?? true);
    return { key: `${name}@${component.version}`, name, version: component.version, sent };
  });
  return { kind: 'rows', rows };
}

export function remoteName(remotes: ReplicationRemote[] | undefined, id: string): string {
  return remotes?.find((remote) => remote.id === id)?.name ?? id;
}
