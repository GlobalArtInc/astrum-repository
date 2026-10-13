import { describe, expect, it } from 'vitest';
import type { RepositoryItem } from '../types';
import {
  activeFilterCount,
  applyFilters,
  DEFAULT_FILTERS,
  facetCounts,
  type ListFilters,
  readFilters,
  toggle,
  writeFilters,
} from './filters';

const FLAGS = {
  browse: true,
  read: true,
  add: false,
  edit: false,
  delete: false,
  admin: false,
  adminEdit: false,
  adminDelete: false,
};

function repository(
  name: string,
  format: string,
  type: RepositoryItem['type'],
  extra: Partial<RepositoryItem> = {},
): RepositoryItem {
  return {
    name,
    format,
    type,
    url: `http://h/repository/${name}`,
    online: true,
    blobStoreName: 'default',
    remoteUrl: null,
    memberOf: [],
    componentCount: 0,
    assetCount: 0,
    totalSize: 0,
    lastUploaded: null,
    limited: false,
    permissions: FLAGS,
    ...extra,
  };
}

const ITEMS = [
  repository('maven-releases', 'maven2', 'hosted', { componentCount: 5, totalSize: 900 }),
  repository('maven-central', 'maven2', 'proxy', { componentCount: 2, totalSize: 100 }),
  repository('npm-internal', 'npm', 'hosted', {
    componentCount: 9,
    totalSize: 50,
    lastUploaded: '2026-10-01T00:00:00Z',
  }),
  repository('raw-offline', 'raw', 'hosted', { online: false, componentCount: null, totalSize: null }),
  repository('Maven-public', 'maven2', 'group', { componentCount: 0, totalSize: 0 }),
];

const collator = new Intl.Collator('en', { numeric: true, sensitivity: 'base' });
const names = (items: RepositoryItem[]) => items.map((item) => item.name);
const filters = (overrides: Partial<ListFilters>): ListFilters => ({ ...DEFAULT_FILTERS, ...overrides });

describe('repository list filters', () => {
  it('read and write the address without defaults', () => {
    const read = readFilters(
      new URLSearchParams(
        'q=mav&format=maven2,npm&type=hosted&type=bogus&status=offline&sort=type&direction=desc',
      ),
    );
    expect(read).toEqual({
      q: 'mav',
      formats: ['maven2', 'npm'],
      types: ['hosted'],
      statuses: ['offline'],
      sort: 'type',
      direction: 'desc',
    });
    expect(writeFilters(read).toString()).toBe(
      'q=mav&format=maven2%2Cnpm&type=hosted&status=offline&sort=type&direction=desc',
    );
    expect(readFilters(new URLSearchParams('sort=nope'))).toEqual(DEFAULT_FILTERS);
    expect(writeFilters(DEFAULT_FILTERS).toString()).toBe('');
    expect(activeFilterCount(read)).toBe(4);
  });

  it('filter by name, format, type and status', () => {
    expect(names(applyFilters(ITEMS, filters({ q: 'MAVEN' }), collator))).toEqual([
      'maven-central',
      'Maven-public',
      'maven-releases',
    ]);
    expect(names(applyFilters(ITEMS, filters({ formats: ['npm', 'raw'] }), collator))).toEqual([
      'npm-internal',
      'raw-offline',
    ]);
    expect(
      names(applyFilters(ITEMS, filters({ types: ['hosted'], statuses: ['online'] }), collator)),
    ).toEqual(['maven-releases', 'npm-internal']);
  });

  it('sort by status with offline repositories last', () => {
    expect(names(applyFilters(ITEMS, filters({ sort: 'status' }), collator))[4]).toBe('raw-offline');
  });

  it('count each facet against the other filters', () => {
    const counts = facetCounts(ITEMS, filters({ formats: ['maven2'], types: ['hosted'] }));
    expect(counts.formats).toEqual([
      ['maven2', 1],
      ['npm', 1],
      ['raw', 1],
    ]);
    expect(counts.types).toEqual([
      ['hosted', 1],
      ['proxy', 1],
      ['group', 1],
    ]);
    expect(counts.statuses).toEqual([['online', 1]]);
    expect(facetCounts(ITEMS, filters({ formats: ['helm'] })).formats).toContainEqual(['helm', 0]);
  });

  it('toggle values', () => {
    expect(toggle(['a'], 'b')).toEqual(['a', 'b']);
    expect(toggle(['a', 'b'], 'a')).toEqual(['b']);
  });
});
