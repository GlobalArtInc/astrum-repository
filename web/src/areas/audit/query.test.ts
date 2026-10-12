import { describe, expect, it } from 'vitest';
import {
  attributeEntries,
  auditPath,
  EMPTY_FILTERS,
  isFiltered,
  knownDomains,
  nextBefore,
  PAGE_SIZE,
  readFilters,
  writeFilters,
} from './query';

describe('audit filters', () => {
  it('read the address, including the old repositoryName parameter', () => {
    expect(
      readFilters(new URLSearchParams('repositoryName=files&type=deleted&since=2026-10-01&until=soon')),
    ).toEqual({
      ...EMPTY_FILTERS,
      repository: 'files',
      type: 'deleted',
      since: '2026-10-01',
    });
    expect(isFiltered(readFilters(new URLSearchParams('')))).toBe(false);
  });

  it('write only the filters that are set', () => {
    const filters = { ...EMPTY_FILTERS, q: ' jane ', domain: 'security.user', repository: 'files' };
    expect(writeFilters(filters).toString()).toBe('q=jane&domain=security.user&repositoryName=files');
    expect(writeFilters(EMPTY_FILTERS).toString()).toBe('');
  });

  it('ask the server page by page before the last id', () => {
    const filters = { ...EMPTY_FILTERS, repository: 'files', initiator: 'admin', until: '2026-10-12' };
    expect(auditPath(filters)).toBe(
      `/service/rest/internal/audit?limit=${PAGE_SIZE}&initiator=admin&repository=files&until=2026-10-12`,
    );
    expect(auditPath(EMPTY_FILTERS, 41)).toBe(`/service/rest/internal/audit?limit=${PAGE_SIZE}&before=41`);
    const event = (id: number) => ({
      id,
      at: '',
      domain: '',
      type: '',
      context: '',
      initiator: '',
      attributes: {},
    });
    expect(nextBefore(Array.from({ length: PAGE_SIZE }, (_, index) => event(100 - index)))).toBe(
      100 - PAGE_SIZE + 1,
    );
    expect(nextBefore([event(3)])).toBeUndefined();
  });

  it('show attributes as text and know the domains in use', () => {
    expect(attributeEntries({ repository: 'files', active: ['a', 'b'], size: 3 })).toEqual([
      ['repository', 'files'],
      ['active', '["a","b"]'],
      ['size', '3'],
    ]);
    expect(attributeEntries(null)).toEqual([]);
    expect(knownDomains(['repository', 'custom.domain'])).toContain('custom.domain');
    expect(knownDomains().filter((domain) => domain === 'repository')).toHaveLength(1);
  });
});
