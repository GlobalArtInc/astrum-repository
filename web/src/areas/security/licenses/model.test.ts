import { describe, expect, it } from 'vitest';
import type { LicenceRow } from './api';
import { exportPath, licencePath, NO_FILTERS, rowKey } from './model';

const row: LicenceRow = {
  repository: 'files',
  format: 'npm',
  namespace: '@scope',
  name: 'pkg',
  version: '1.0.0',
  declared: 'MIT',
  expression: 'MIT',
  ids: ['MIT'],
  state: 'spdx',
  verdict: 'pass',
};

describe('the licence report query', () => {
  it('asks for the whole list when no filter is set', () => {
    expect(licencePath(NO_FILTERS, null)).toBe('/service/rest/v1/security/licenses');
  });

  it('carries the set filters, the trimmed licence and the continuation token', () => {
    expect(licencePath({ repository: 'files', license: ' MIT ', state: 'none', format: 'raw' }, 'abc')).toBe(
      '/service/rest/v1/security/licenses?repository=files&license=MIT&state=none&format=raw&continuationToken=abc',
    );
  });

  it('exports with the same filters and no paging', () => {
    expect(exportPath({ ...NO_FILTERS, license: 'Apache-2.0' })).toBe(
      '/service/rest/v1/security/licenses/export.csv?license=Apache-2.0',
    );
  });

  it('keys a row by its component', () => {
    expect(rowKey(row)).toBe('files/npm/@scope/pkg/1.0.0');
  });
});
