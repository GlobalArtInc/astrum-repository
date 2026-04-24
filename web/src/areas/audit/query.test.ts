import { describe, expect, it } from 'vitest';
import type { Translate } from '../../i18n';
import { categoryOf, kindLabel, summaryOf } from './events';
import { auditLogPath, EMPTY_FILTERS, pageWindow, readFilters, readPaging, writeParams } from './query';

const t = Object.assign(
  (key: string, vars?: Record<string, unknown>) =>
    vars
      ? `${key}(${Object.entries(vars)
          .map(([name, value]) => `${name}=${value}`)
          .join(',')})`
      : key,
  { rich: () => [] },
) as unknown as Translate;

describe('audit query', () => {
  it('reads and writes the filters and paging of the address', () => {
    const params = new URLSearchParams(
      'period=7&categories=security,bogus&types=login&repositoryName=x&page=2&size=50',
    );
    const filters = readFilters(params);
    expect(filters).toEqual({
      ...EMPTY_FILTERS,
      period: 7,
      categories: ['security'],
      types: ['login'],
      repository: 'x',
    });
    expect(readPaging(params)).toEqual({ page: 2, size: 50 });
    expect(writeParams(filters, { page: 2, size: 50 }).toString()).toBe(
      'period=7&categories=security&types=login&repositoryName=x&page=2&size=50',
    );
    expect(writeParams(EMPTY_FILTERS, { page: 1, size: 20 }).toString()).toBe('');
    expect(readPaging(new URLSearchParams('page=-3&size=7'))).toEqual({ page: 1, size: 20 });
  });

  it('asks the server for one page of a period', () => {
    const now = new Date('2026-10-13T00:00:00.000Z');
    expect(
      auditLogPath(
        { ...EMPTY_FILTERS, period: 1, types: ['created', 'deleted'] },
        { page: 3, size: 40 },
        now,
      ),
    ).toBe(
      '/service/rest/internal/ui/audit-log?page=3&limit=40&types=created%2Cdeleted&startDate=2026-10-12T00%3A00%3A00.000Z&endDate=2026-10-13T00%3A00%3A00.000Z',
    );
  });

  it('numbers the pages like the old pager', () => {
    expect(pageWindow(1, 3)).toEqual([1, 2, 3]);
    expect(pageWindow(1, 229)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 'gap', 229]);
    expect(pageWindow(229, 229)).toEqual([1, 'gap', 222, 223, 224, 225, 226, 227, 228, 229]);
    expect(pageWindow(50, 229)).toEqual([1, 'gap', 48, 49, 50, 51, 52, 'gap', 229]);
  });
});

describe('audit events', () => {
  it('turn sessions into sign-ins and group domains into categories', () => {
    expect(categoryOf({ domain: 'security.session', type: 'created' })).toBe('security');
    expect(categoryOf({ domain: 'asset', type: 'deleted' })).toBe('repository');
    expect(categoryOf({ domain: 'security.ssrf', type: 'updated' })).toBe('configuration');
    expect(kindLabel(t, { domain: 'security.session', type: 'deleted' })).toBe('audit.kind.logout');
    expect(kindLabel(t, { domain: 'repository', type: 'revoked-token-attempt' })).toBe(
      'Revoked Token Attempt',
    );
  });

  it('write the old summaries', () => {
    const base = { id: 1, timestamp: '', initiator: 'admin/10.0.0.1', attributes: {} };
    expect(summaryOf(t, { ...base, domain: 'security.session', type: 'created', context: 'jane' })).toBe(
      'audit.summary.session(name=jane,action=audit.summary.login)',
    );
    expect(summaryOf(t, { ...base, domain: 'security.user', type: 'created', context: 'jane' })).toBe(
      'audit.summary.event(entity=audit.entity.user,name=jane,action=audit.action.created,initiator=admin/10.0.0.1)',
    );
    expect(summaryOf(t, { ...base, domain: 'other', type: 'weird-kind', context: '' })).toBe(
      'audit.summary.event(entity=audit.entity.event,name=audit.summary.unknown,action=weird kind,initiator=admin/10.0.0.1)',
    );
  });
});
