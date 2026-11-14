import { withQuery } from '../../../api';
import { LICENCE_EXPORT, LICENCES, type LicenceFilters, type LicenceRow } from './api';

export const NO_FILTERS: LicenceFilters = { repository: '', license: '', state: '', format: '' };

export const STATE_COLORS = { spdx: 'green', mapped: 'blue', unknown: 'amber', none: 'gray' } as const;

export const VERDICT_COLORS = { pass: 'green', blocked: 'red', reported: 'amber', unjudged: 'gray' } as const;

function filterQuery(filters: LicenceFilters): Record<string, string> {
  return {
    repository: filters.repository,
    license: filters.license.trim(),
    state: filters.state,
    format: filters.format,
  };
}

export function licencePath(filters: LicenceFilters, token: string | null): string {
  return withQuery(LICENCES, { ...filterQuery(filters), continuationToken: token });
}

export function exportPath(filters: LicenceFilters): string {
  return withQuery(LICENCE_EXPORT, filterQuery(filters));
}

export function rowKey(row: LicenceRow): string {
  return [row.repository, row.format, row.namespace, row.name, row.version].join('/');
}
