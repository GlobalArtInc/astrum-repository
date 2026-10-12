import type { RepositoryType } from '../../../api';
import type { RepositoryItem } from '../types';

export type Status = 'online' | 'offline';
export type SortKey = 'name' | 'format' | 'type' | 'status' | 'components' | 'size' | 'uploaded';
export type Direction = 'asc' | 'desc';

export interface ListFilters {
  q: string;
  formats: string[];
  types: RepositoryType[];
  statuses: Status[];
  sort: SortKey;
  direction: Direction;
}

export const SORT_KEYS: SortKey[] = ['name', 'format', 'type', 'status', 'components', 'size', 'uploaded'];
const TYPES: RepositoryType[] = ['hosted', 'proxy', 'group'];
const STATUSES: Status[] = ['online', 'offline'];

export const DEFAULT_FILTERS: ListFilters = {
  q: '',
  formats: [],
  types: [],
  statuses: [],
  sort: 'name',
  direction: 'asc',
};

function list(params: URLSearchParams, name: string): string[] {
  return params
    .getAll(name)
    .flatMap((value) => value.split(','))
    .map((value) => value.trim())
    .filter((value) => value !== '');
}

export function readFilters(params: URLSearchParams): ListFilters {
  const sort = params.get('sort') as SortKey | null;
  return {
    q: params.get('q') ?? '',
    formats: list(params, 'format'),
    types: list(params, 'type').filter((value): value is RepositoryType =>
      TYPES.includes(value as RepositoryType),
    ),
    statuses: list(params, 'status').filter((value): value is Status => STATUSES.includes(value as Status)),
    sort: sort && SORT_KEYS.includes(sort) ? sort : 'name',
    direction: params.get('direction') === 'desc' ? 'desc' : 'asc',
  };
}

export function writeFilters(filters: ListFilters): URLSearchParams {
  const params = new URLSearchParams();
  if (filters.q.trim() !== '') {
    params.set('q', filters.q);
  }
  if (filters.formats.length > 0) {
    params.set('format', filters.formats.join(','));
  }
  if (filters.types.length > 0) {
    params.set('type', filters.types.join(','));
  }
  if (filters.statuses.length > 0) {
    params.set('status', filters.statuses.join(','));
  }
  if (filters.sort !== 'name') {
    params.set('sort', filters.sort);
  }
  if (filters.direction !== 'asc') {
    params.set('direction', filters.direction);
  }
  return params;
}

export function activeFilterCount(filters: ListFilters): number {
  return filters.formats.length + filters.types.length + filters.statuses.length;
}

export function statusOf(item: RepositoryItem): Status {
  return item.online ? 'online' : 'offline';
}

type Facet = 'format' | 'type' | 'status' | 'text';

function matches(item: RepositoryItem, filters: ListFilters, skip?: Facet): boolean {
  const text = filters.q.trim().toLowerCase();
  if (skip !== 'text' && text !== '' && !item.name.toLowerCase().includes(text)) {
    return false;
  }
  if (skip !== 'format' && filters.formats.length > 0 && !filters.formats.includes(item.format)) {
    return false;
  }
  if (skip !== 'type' && filters.types.length > 0 && !filters.types.includes(item.type)) {
    return false;
  }
  if (skip !== 'status' && filters.statuses.length > 0 && !filters.statuses.includes(statusOf(item))) {
    return false;
  }
  return true;
}

function sortValue(item: RepositoryItem, key: SortKey): string | number | null {
  switch (key) {
    case 'name':
      return item.name;
    case 'format':
      return item.format;
    case 'type':
      return item.type;
    case 'status':
      return item.online ? 0 : 1;
    case 'components':
      return item.componentCount;
    case 'size':
      return item.totalSize;
    case 'uploaded':
      return item.lastUploaded ? Date.parse(item.lastUploaded) : null;
  }
}

export function applyFilters(
  items: readonly RepositoryItem[],
  filters: ListFilters,
  collator: Intl.Collator,
): RepositoryItem[] {
  const sign = filters.direction === 'asc' ? 1 : -1;
  return items
    .filter((item) => matches(item, filters))
    .sort((left, right) => {
      const a = sortValue(left, filters.sort);
      const b = sortValue(right, filters.sort);
      if (a === null || b === null) {
        if (a === b) {
          return collator.compare(left.name, right.name);
        }
        return a === null ? 1 : -1;
      }
      const compared =
        typeof a === 'number' && typeof b === 'number' ? a - b : collator.compare(String(a), String(b));
      return compared === 0 ? collator.compare(left.name, right.name) : sign * compared;
    });
}

export interface FacetCounts {
  formats: Array<[string, number]>;
  types: Array<[RepositoryType, number]>;
  statuses: Array<[Status, number]>;
}

function count<T extends string>(values: T[]): Map<T, number> {
  const counts = new Map<T, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return counts;
}

export function facetCounts(items: readonly RepositoryItem[], filters: ListFilters): FacetCounts {
  const formats = count(items.filter((item) => matches(item, filters, 'format')).map((item) => item.format));
  for (const format of filters.formats) {
    if (!formats.has(format)) {
      formats.set(format, 0);
    }
  }
  const types = count(items.filter((item) => matches(item, filters, 'type')).map((item) => item.type));
  const statuses = count(items.filter((item) => matches(item, filters, 'status')).map(statusOf));
  return {
    formats: [...formats.entries()].sort(([a], [b]) => a.localeCompare(b)),
    types: TYPES.filter((type) => types.has(type) || filters.types.includes(type)).map((type) => [
      type,
      types.get(type) ?? 0,
    ]),
    statuses: STATUSES.filter((status) => statuses.has(status) || filters.statuses.includes(status)).map(
      (status) => [status, statuses.get(status) ?? 0],
    ),
  };
}

export function toggle<T>(values: readonly T[], value: T): T[] {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
