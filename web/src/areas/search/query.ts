import { withQuery } from '../../api';
import { SEARCH } from './model';

export type SortField = '' | 'name' | 'repository';
export type Direction = 'asc' | 'desc';

export interface SearchState {
  q: string;
  format: string;
  repository: string;
  sort: SortField;
  direction: Direction;
}

export const SORT_FIELDS: readonly SortField[] = ['', 'name', 'repository'];
export const MIN_QUERY = 2;

export const SORT_CHOICES = [
  { value: 'newest', sort: '', direction: 'desc', label: 'search.sort.newest' },
  { value: 'oldest', sort: '', direction: 'asc', label: 'search.sort.oldest' },
  { value: 'name-asc', sort: 'name', direction: 'asc', label: 'search.sort.nameAsc' },
  { value: 'name-desc', sort: 'name', direction: 'desc', label: 'search.sort.nameDesc' },
  { value: 'repository-asc', sort: 'repository', direction: 'asc', label: 'search.sort.repositoryAsc' },
  { value: 'repository-desc', sort: 'repository', direction: 'desc', label: 'search.sort.repositoryDesc' },
] as const satisfies ReadonlyArray<{ value: string; sort: SortField; direction: Direction; label: string }>;

export function defaultDirection(sort: SortField): Direction {
  return sort === '' ? 'desc' : 'asc';
}

function sortField(value: string | null): SortField {
  return SORT_FIELDS.find((field) => field !== '' && field === value) ?? '';
}

export function readSearchState(params: URLSearchParams): SearchState {
  const sort = sortField(params.get('sort'));
  const direction = params.get('direction');
  return {
    q: params.get('q') ?? '',
    format: params.get('format') ?? '',
    repository: params.get('repository') ?? '',
    sort,
    direction: direction === 'asc' || direction === 'desc' ? direction : defaultDirection(sort),
  };
}

export function writeSearchState(state: SearchState): URLSearchParams {
  const params = new URLSearchParams();
  if (state.q !== '') {
    params.set('q', state.q);
  }
  if (state.format !== '') {
    params.set('format', state.format);
  }
  if (state.repository !== '') {
    params.set('repository', state.repository);
  }
  if (state.sort !== '') {
    params.set('sort', state.sort);
  }
  if (state.direction !== defaultDirection(state.sort)) {
    params.set('direction', state.direction);
  }
  return params;
}

export function sortChoice(state: Pick<SearchState, 'sort' | 'direction'>): string {
  const found = SORT_CHOICES.find(
    (choice) => choice.sort === state.sort && choice.direction === state.direction,
  );
  return found?.value ?? `${state.sort || 'newest'}-${state.direction}`;
}

export function fromSortChoice(value: string): Pick<SearchState, 'sort' | 'direction'> {
  const found = SORT_CHOICES.find((choice) => choice.value === value);
  if (found) {
    return { sort: found.sort, direction: found.direction };
  }
  const [field = '', direction = ''] = value.split('-');
  const sort = sortField(field);
  return {
    sort,
    direction: direction === 'asc' || direction === 'desc' ? direction : defaultDirection(sort),
  };
}

export function isSearchable(state: SearchState): boolean {
  return state.q.trim().length >= MIN_QUERY || state.format !== '' || state.repository !== '';
}

export function searchPath(state: SearchState): string {
  return withQuery(SEARCH, {
    q: state.q.trim(),
    format: state.format,
    repository: state.repository,
    sort: state.sort,
    direction: state.direction,
  });
}

export function searchAddress(query: string, repository?: string): string {
  const params = writeSearchState({
    q: query.trim(),
    format: '',
    repository: repository ?? '',
    sort: '',
    direction: 'desc',
  });
  const text = params.toString();
  return text ? `/search?${text}` : '/search';
}
