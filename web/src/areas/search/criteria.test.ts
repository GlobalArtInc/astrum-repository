import { describe, expect, it } from 'vitest';
import {
  availableCriteria,
  buildValue,
  checkCriterion,
  criteriaPath,
  nextKey,
  operatorsOf,
  parseValue,
  readCriteria,
  writeCriteria,
} from './criteria';

const row = (key: string, value: string, operator: 'is' | 'startsWith' | 'contains' = 'is', id = 1) => ({
  id,
  key,
  operator,
  value,
});

describe('criteria', () => {
  it('offer format keys only for formats the server has', () => {
    const keys = availableCriteria(['raw', 'maven2']).map((criterion) => criterion.key);
    expect(keys).toContain('maven.groupId');
    expect(keys).toContain('sha512');
    expect(keys).not.toContain('npm.scope');
    expect(keys).not.toContain('docker.imageName');
    expect(availableCriteria(['docker']).map((criterion) => criterion.key)).toContain('docker.artifactKind');
    expect(availableCriteria(['helm']).map((criterion) => criterion.key)).toEqual(
      expect.arrayContaining(['helm.appVersion', 'helm.description', 'helm.keywords']),
    );
    expect(buildValue(row('helm.keywords', 'web', 'contains'))).toBe('*web*');
    expect(availableCriteria(['nuget']).map((criterion) => criterion.key)).toEqual(
      expect.arrayContaining(['nuget.id', 'nuget.tags']),
    );
    expect(keys).not.toContain('nuget.id');
  });

  it('turn operators into wildcards only where the server matches patterns', () => {
    expect(operatorsOf('name')).toEqual(['is', 'startsWith', 'contains']);
    expect(operatorsOf('q')).toEqual(['contains']);
    expect(operatorsOf('sha1')).toEqual(['is']);
    expect(buildValue(row('name', ' junit '))).toBe('junit');
    expect(buildValue(row('name', 'junit', 'startsWith'))).toBe('junit*');
    expect(buildValue(row('maven.groupId', '*apache*', 'contains'))).toBe('*apache*');
    expect(buildValue(row('npm.scope', 'acme', 'contains'))).toBe('*acme*');
    expect(buildValue(row('q', 'junit', 'contains'))).toBe('junit');
    expect(buildValue(row('format', 'maven2', 'startsWith'))).toBe('maven2');
  });

  it('read operators back from the address', () => {
    expect(parseValue('name', 'junit*')).toEqual({ operator: 'startsWith', value: 'junit' });
    expect(parseValue('name', '*unit*')).toEqual({ operator: 'contains', value: 'unit' });
    expect(parseValue('name', 'ju*it')).toEqual({ operator: 'is', value: 'ju*it' });
    expect(parseValue('name', '*')).toEqual({ operator: 'is', value: '*' });
    expect(parseValue('q', 'junit*')).toEqual({ operator: 'contains', value: 'junit*' });
    expect(parseValue('sha1', 'ab')).toEqual({ operator: 'is', value: 'ab' });
  });

  it('round-trip through the address with the mode', () => {
    const params = writeCriteria(
      [
        row('maven.groupId', 'org.apache', 'startsWith', 1),
        row('version', '2', 'contains', 2),
        row('name', '', 'is', 3),
      ],
      'assets',
    );
    expect(params.toString()).toBe('maven.groupId=org.apache*&version=*2*&mode=assets');
    const read = readCriteria(params);
    expect(read.mode).toBe('assets');
    expect(read.criteria.map(({ key, operator, value }) => [key, operator, value])).toEqual([
      ['version', 'contains', '2'],
      ['maven.groupId', 'startsWith', 'org.apache'],
    ]);
    expect(
      writeCriteria([row('q', 'x', 'contains', 1), row('q', 'y', 'contains', 2)], 'components').toString(),
    ).toBe('q=x');
  });

  it('ask the components or the assets search', () => {
    expect(criteriaPath(new URLSearchParams('name=junit*&format=maven2'))).toBe(
      '/service/rest/v1/search?format=maven2&name=junit*',
    );
    expect(criteriaPath(new URLSearchParams('sha256=abc&mode=assets'))).toBe(
      '/service/rest/v1/search/assets?sha256=abc',
    );
    expect(criteriaPath(new URLSearchParams('mode=assets&unknown=1'))).toBeNull();
  });

  it('check checksums and pick the next unused key', () => {
    expect(checkCriterion(row('sha1', 'a'.repeat(40)))).toBeNull();
    expect(checkCriterion(row('sha1', 'a'.repeat(39)))).toBe('checksum');
    expect(checkCriterion(row('md5', 'z'.repeat(32)))).toBe('checksum');
    expect(checkCriterion(row('name', ' '))).toBe('empty');
    expect(nextKey([row('q', 'x')], ['raw'])).toBe('format');
    const all = availableCriteria(['raw']).map((criterion, index) => row(criterion.key, 'x', 'is', index));
    expect(nextKey(all, ['raw'])).toBeNull();
  });
});
