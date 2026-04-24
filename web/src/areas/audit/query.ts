import { withQuery } from '../../api';
import { CATEGORIES, type Category, EVENT_TYPES } from './events';

export const AUDIT_LOG = '/service/rest/internal/ui/audit-log';
export const PERIODS = [1, 7, 30, 90] as const;
export const PAGE_SIZES = [20, 40, 50, 100, 250] as const;
export const DEFAULT_PERIOD = 30;
export const DEFAULT_SIZE = 20;

export type Period = (typeof PERIODS)[number];
export type EventType = (typeof EVENT_TYPES)[number];

export interface AuditFilters {
  period: Period;
  categories: Category[];
  types: EventType[];
  format: string;
  repository: string;
  initiator: string;
  q: string;
}

export interface AuditPaging {
  page: number;
  size: number;
}

export interface AuditPage<T> {
  items: T[];
  pagination: { totalItems: number; totalPages: number; currentPage: number; itemsPerPage: number };
}

export const EMPTY_FILTERS: AuditFilters = {
  period: DEFAULT_PERIOD,
  categories: [],
  types: [],
  format: '',
  repository: '',
  initiator: '',
  q: '',
};

function listOf<T extends string>(value: string | null, allowed: readonly T[]): T[] {
  return (value ?? '')
    .split(',')
    .map((item) => item.trim())
    .filter((item): item is T => (allowed as readonly string[]).includes(item));
}

function numberOf<T extends number>(value: string | null, allowed: readonly T[], fallback: T): T {
  const parsed = Number(value);
  return (allowed as readonly number[]).includes(parsed) ? (parsed as T) : fallback;
}

export function readFilters(params: URLSearchParams): AuditFilters {
  return {
    period: numberOf(params.get('period'), PERIODS, DEFAULT_PERIOD),
    categories: listOf(params.get('categories'), CATEGORIES),
    types: listOf(params.get('types'), EVENT_TYPES),
    format: params.get('format') ?? '',
    repository: params.get('repositoryName') ?? '',
    initiator: params.get('initiator') ?? '',
    q: params.get('q') ?? '',
  };
}

export function readPaging(params: URLSearchParams): AuditPaging {
  const page = Math.floor(Number(params.get('page')));
  return {
    page: Number.isFinite(page) && page > 1 ? page : 1,
    size: numberOf(params.get('size'), PAGE_SIZES, DEFAULT_SIZE),
  };
}

export function writeParams(filters: AuditFilters, paging: AuditPaging): URLSearchParams {
  const params = new URLSearchParams();
  if (filters.period !== DEFAULT_PERIOD) {
    params.set('period', String(filters.period));
  }
  const entries: Array<[string, string]> = [
    ['categories', filters.categories.join(',')],
    ['types', filters.types.join(',')],
    ['format', filters.format],
    ['repositoryName', filters.repository],
    ['initiator', filters.initiator],
    ['q', filters.q],
  ];
  for (const [key, value] of entries) {
    if (value.trim() !== '') {
      params.set(key, value.trim());
    }
  }
  if (paging.page > 1) {
    params.set('page', String(paging.page));
  }
  if (paging.size !== DEFAULT_SIZE) {
    params.set('size', String(paging.size));
  }
  return params;
}

export function isFiltered(filters: AuditFilters): boolean {
  return (
    filters.period !== DEFAULT_PERIOD ||
    filters.categories.length > 0 ||
    filters.types.length > 0 ||
    filters.format !== '' ||
    filters.repository !== '' ||
    filters.initiator.trim() !== '' ||
    filters.q.trim() !== ''
  );
}

export function auditLogPath(filters: AuditFilters, paging: AuditPaging, now: Date): string {
  const start = new Date(now.getTime() - filters.period * 24 * 60 * 60 * 1000);
  return withQuery(AUDIT_LOG, {
    page: paging.page,
    limit: paging.size,
    categories: filters.categories.join(','),
    types: filters.types.join(','),
    repository: filters.repository,
    initiator: filters.initiator.trim(),
    q: filters.q.trim(),
    startDate: start.toISOString(),
    endDate: now.toISOString(),
  });
}

export function pageWindow(current: number, total: number): Array<number | 'gap'> {
  if (total <= 9) {
    return Array.from({ length: total }, (_, index) => index + 1);
  }
  if (current <= 5) {
    return [1, 2, 3, 4, 5, 6, 7, 8, 'gap', total];
  }
  if (current >= total - 4) {
    return [1, 'gap', ...Array.from({ length: 8 }, (_, index) => total - 7 + index)];
  }
  return [1, 'gap', current - 2, current - 1, current, current + 1, current + 2, 'gap', total];
}
