import { type ApiError, fieldName } from '../../api';
import type { Translate } from '../../i18n';
import type { Errors, Option } from '../../ui';
import type { DayDownloads } from '../dashboard/types';
import { DEFAULT_SOURCE, type User } from '../security/api';
import type { Repository, UsageRow } from './api';

export const READ = 'astrum:customers:read';
export const CREATE = 'astrum:customers:create';
export const UPDATE = 'astrum:customers:update';
export const DELETE = 'astrum:customers:delete';
export const USERS_READ = 'astrum:users:read';

export const EXPIRY_WARNING_DAYS = 30;
export const NAME_MAX = 200;
export const REFERENCE_MAX = 200;
export const NOTE_MAX = 2000;

const ID_PATTERN = /^[a-z0-9-]{1,64}$/;
const DAY_MS = 24 * 60 * 60 * 1000;
const GROUP = 'group';
const DEFAULT_URL = 'http://localhost';

export type ExpiryTone = 'warning' | 'danger';

export interface CustomerValues {
  name: string;
  reference: string;
  note: string;
  disabled: boolean;
}

export interface NewCustomerValues extends CustomerValues {
  id: string;
}

export interface CustomerRequest {
  name: string;
  reference: string;
  note: string;
  disabled: boolean;
}

export interface NewCustomerRequest {
  id: string;
  name: string;
  reference: string;
  note: string;
}

export const EMPTY_CUSTOMER: NewCustomerValues = {
  id: '',
  name: '',
  reference: '',
  note: '',
  disabled: false,
};

export function fromCustomer(customer: CustomerValues): CustomerValues {
  return {
    name: customer.name,
    reference: customer.reference,
    note: customer.note,
    disabled: customer.disabled,
  };
}

export function idProblem(t: Translate, id: string): string | undefined {
  return ID_PATTERN.test(id) ? undefined : t('customers.errors.id');
}

export function nameProblem(t: Translate, name: string): string | undefined {
  return name.trim() === '' ? t('customers.errors.name') : undefined;
}

export function validateCustomer(t: Translate, values: CustomerValues): Errors<CustomerValues> {
  const name = nameProblem(t, values.name);
  return name ? { name } : {};
}

export function validateNewCustomer(t: Translate, values: NewCustomerValues): Errors<NewCustomerValues> {
  const errors: Errors<NewCustomerValues> = validateCustomer(t, values);
  const id = idProblem(t, values.id);
  if (id) {
    errors.id = id;
  }
  return errors;
}

export function toCustomerRequest(values: CustomerValues): CustomerRequest {
  return {
    name: values.name.trim(),
    reference: values.reference.trim(),
    note: values.note.trim(),
    disabled: values.disabled,
  };
}

export function toNewCustomerRequest(values: NewCustomerValues): NewCustomerRequest {
  return {
    id: values.id,
    name: values.name.trim(),
    reference: values.reference.trim(),
    note: values.note.trim(),
  };
}

export function expiryTone(expiry: string | null, now: number): ExpiryTone | undefined {
  if (expiry === null) {
    return undefined;
  }
  const at = Date.parse(expiry);
  if (at <= now) {
    return 'danger';
  }
  return at - now <= EXPIRY_WARNING_DAYS * DAY_MS ? 'warning' : undefined;
}

export const USAGE_PERIODS = [30, 90, 365] as const;
export type UsagePeriod = (typeof USAGE_PERIODS)[number];
export const EXPIRY_PRESETS = [30, 90, 365, 730] as const;
export const CUSTOM_EXPIRY = 'custom';
export const MAX_TOKEN_DAYS = 3650;

export interface RepositoryTotals {
  repository: string;
  downloads: number;
  bytes: number;
}

export interface IssueTokenValues {
  name: string;
  owner: string;
  repositories: string[];
  expiry: string;
  customDays: number | null;
}

export interface IssueTokenRequest {
  name: string;
  owner: string;
  repositories: string[];
  expiresInDays: number | null;
}

export interface ReadyLine {
  repository: string;
  text: string;
}

export const EMPTY_ISSUE_TOKEN: IssueTokenValues = {
  name: '',
  owner: '',
  repositories: [],
  expiry: '',
  customDays: null,
};

const SERVER_FIELDS: Record<string, keyof IssueTokenValues> = {
  name: 'name',
  owner: 'owner',
  repositories: 'repositories',
  expiresInDays: 'expiry',
};

function isDays(value: number | null): value is number {
  return value !== null && Number.isInteger(value) && value >= 1 && value <= MAX_TOKEN_DAYS;
}

export function expiryDays(values: IssueTokenValues): number | null {
  if (values.expiry === CUSTOM_EXPIRY) {
    return values.customDays;
  }
  const preset = EXPIRY_PRESETS.find((days) => String(days) === values.expiry);
  return preset ?? null;
}

export function validateIssueToken(t: Translate, values: IssueTokenValues): Errors<IssueTokenValues> {
  const errors: Errors<IssueTokenValues> = {};
  const name = nameProblem(t, values.name);
  if (name) {
    errors.name = name;
  }
  if (values.owner.trim() === '') {
    errors.owner = t('customers.errors.owner');
  }
  if (values.repositories.length === 0) {
    errors.repositories = t('customers.errors.repositories');
  }
  if (values.expiry === CUSTOM_EXPIRY) {
    if (!isDays(values.customDays)) {
      errors.customDays = t('customers.errors.days', { max: MAX_TOKEN_DAYS });
    }
  } else if (expiryDays(values) === null) {
    errors.expiry = t('customers.errors.expiry');
  }
  return errors;
}

export function toIssueTokenRequest(values: IssueTokenValues): IssueTokenRequest {
  return {
    name: values.name.trim(),
    owner: values.owner.trim(),
    repositories: values.repositories,
    expiresInDays: expiryDays(values),
  };
}

export interface ServerErrors {
  errors: Errors<IssueTokenValues>;
  unmatched: string[];
}

export function serverErrors(error: ApiError): ServerErrors {
  const errors: Errors<IssueTokenValues> = {};
  const unmatched: string[] = [];
  for (const problem of error.fieldErrors) {
    const field = SERVER_FIELDS[fieldName(problem.id)];
    if (field && !errors[field]) {
      errors[field] = problem.message;
    } else {
      unmatched.push(problem.message);
    }
  }
  return { errors, unmatched };
}

export function hasGroup(names: string[], repositories: Repository[]): boolean {
  return repositories.some((repository) => repository.type === GROUP && names.includes(repository.name));
}

export function repositoryOptions(repositories: Repository[]): Option[] {
  return [...repositories]
    .sort((left, right) => left.name.localeCompare(right.name))
    .map((repository) => ({
      value: repository.name,
      label: `${repository.name} (${repository.format}, ${repository.type})`,
    }));
}

export function usableOwners(users: User[]): string[] {
  return users
    .filter((user) => user.status === 'active' && user.source === DEFAULT_SOURCE)
    .map((user) => user.userId)
    .sort((left, right) => left.localeCompare(right));
}

function readyLine(
  target: Pick<Repository, 'name' | 'format'>,
  customer: string,
  token: string,
  url: URL,
): string {
  const prefix = url.pathname.replace(/\/+$/, '');
  const root = `${url.origin}${prefix}`;
  const name = target.name;
  switch (target.format) {
    case 'docker':
      return `docker login -u ${customer} ${url.host}`;
    case 'helm':
      return `helm repo add ${name} ${root}/repository/${name}/ --username ${customer} --password ${token}`;
    case 'npm':
      return `//${url.host}${prefix}/repository/${name}/:_authToken=${token}`;
    default:
      return `curl -u ${customer}:${token} -fLO ${root}/repository/${name}/<path>`;
  }
}

export function readyLines(
  baseUrl: string,
  customer: string,
  token: string,
  targets: Array<Pick<Repository, 'name' | 'format'>>,
): ReadyLine[] {
  const url = new URL(baseUrl.trim() === '' ? DEFAULT_URL : baseUrl);
  return targets.map((target) => ({
    repository: target.name,
    text: readyLine(target, customer, token, url),
  }));
}

export function dailyDownloads(rows: UsageRow[], days: number, now: Date): DayDownloads[] {
  const sums = new Map<string, number>();
  for (const row of rows) {
    sums.set(row.day, (sums.get(row.day) ?? 0) + row.downloads);
  }
  const today = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  return Array.from({ length: days }, (_, index) => {
    const day = new Date(today - (days - 1 - index) * DAY_MS).toISOString().slice(0, 10);
    return { day, downloads: sums.get(day) ?? 0 };
  });
}

export function repositoryTotals(rows: UsageRow[]): RepositoryTotals[] {
  const totals = new Map<string, RepositoryTotals>();
  for (const row of rows) {
    const current = totals.get(row.repository) ?? { repository: row.repository, downloads: 0, bytes: 0 };
    current.downloads += row.downloads;
    current.bytes += row.bytes;
    totals.set(row.repository, current);
  }
  return [...totals.values()].sort(
    (left, right) => right.downloads - left.downloads || left.repository.localeCompare(right.repository),
  );
}
