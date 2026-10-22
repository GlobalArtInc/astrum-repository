import { describe, expect, it } from 'vitest';
import {
  BASE_CRITERIA,
  buildValue,
  criteriaPath,
  moreCriteria,
  operatorsOf,
  parseValue,
  readCriteria,
  withKey,
  writeCriteria,
} from './criteria';

const row = (key: string, value: string, operator: 'is' | 'startsWith' | 'contains' = 'is', id = 1) => ({
  id,
  key,
  operator,
  value,
});

describe('criteria', () => {
  it('offer the six fields of the old page first', () => {
    expect(BASE_CRITERIA.map((criterion) => criterion.key)).toEqual([
      'q',
      'format',
      'repository',
      'group',
      'name',
      'version',
    ]);
  });

  it('add checksums and the fields of the formats the server knows', () => {
    const keys = moreCriteria(['helm', 'raw'], '').map((criterion) => criterion.key);
    expect(keys).toContain('sha1');
    expect(keys).toContain('helm.description');
    expect(keys).not.toContain('nuget.id');
    expect(moreCriteria(['puppet'], '').map((criterion) => criterion.key)).toEqual(
      expect.arrayContaining(['puppet.tag', 'puppet.license']),
    );
    expect(moreCriteria(['jetbrains'], '').map((criterion) => criterion.key)).toEqual(
      expect.arrayContaining(['jetbrains.vendor', 'jetbrains.category', 'jetbrains.channel']),
    );
    expect(moreCriteria(['raw'], 'nuget.id').map((criterion) => criterion.key)).toContain('nuget.id');
    expect(operatorsOf('name')).toEqual(['is', 'startsWith']);
    expect(operatorsOf('helm.keywords')).toEqual(['is', 'startsWith', 'contains']);
    expect(operatorsOf('sha1')).toEqual(['is']);
    expect(withKey(row('helm.keywords', 'web', 'contains'), 'name')).toEqual({
      id: 1,
      key: 'name',
      operator: 'is',
      value: 'web',
    });
    expect(withKey(row('name', 'web'), 'docker.artifactKind').value).toBe('');
    const vagrant = moreCriteria(['vagrant'], '').map((criterion) => criterion.key);
    expect(vagrant).toContain('vagrant.provider');
    expect(vagrant).toContain('vagrant.architecture');
    const nix = moreCriteria(['nix'], '').map((criterion) => criterion.key);
    expect(nix).toContain('nix.storePath');
    expect(nix).toContain('nix.narHash');
  });

  it('offer the pacman fields for pacman repositories', () => {
    const keys = moreCriteria(['pacman'], '').map((criterion) => criterion.key);
    expect(keys).toEqual(expect.arrayContaining(['pacman.architecture', 'pacman.base', 'pacman.license']));
    expect(moreCriteria(['raw'], '').map((criterion) => criterion.key)).not.toContain('pacman.base');
  });

  it('turn "starts with" into a trailing wildcard', () => {
    expect(buildValue(row('name', ' junit '))).toBe('junit');
    expect(buildValue(row('name', 'junit', 'startsWith'))).toBe('junit*');
    expect(buildValue(row('q', 'spring', 'startsWith'))).toBe('spring*');
    expect(buildValue(row('name', 'junit*', 'startsWith'))).toBe('junit*');
    expect(buildValue(row('name', ' ', 'startsWith'))).toBe('');
    expect(buildValue(row('helm.keywords', '*web', 'contains'))).toBe('*web*');
    expect(buildValue(row('sha1', 'abc', 'startsWith'))).toBe('abc');
  });

  it('read operators back from the address', () => {
    expect(parseValue('name', 'junit*')).toEqual({ operator: 'startsWith', value: 'junit' });
    expect(parseValue('name', 'ju*it')).toEqual({ operator: 'is', value: 'ju*it' });
    expect(parseValue('name', '*')).toEqual({ operator: 'is', value: '*' });
    expect(parseValue('name', '*unit*')).toEqual({ operator: 'is', value: '*unit*' });
    expect(parseValue('helm.keywords', '*web*')).toEqual({ operator: 'contains', value: 'web' });
    expect(parseValue('sha1', 'abc*')).toEqual({ operator: 'is', value: 'abc*' });
  });

  it('round-trip through the address and skip empty or repeated fields', () => {
    const params = writeCriteria([
      row('group', 'org.apache', 'startsWith', 1),
      row('version', '2.0', 'is', 2),
      row('name', '', 'is', 3),
      row('version', '3.0', 'is', 4),
    ]);
    expect(params.toString()).toBe('group=org.apache*&version=2.0');
    expect(readCriteria(params).map(({ key, operator, value }) => [key, operator, value])).toEqual([
      ['group', 'startsWith', 'org.apache'],
      ['version', 'is', '2.0'],
    ]);
  });

  it('ask the components search', () => {
    expect(criteriaPath(new URLSearchParams('name=junit*&format=maven2'))).toBe(
      '/service/rest/v1/search?format=maven2&name=junit*',
    );
    expect(criteriaPath(new URLSearchParams('helm.keywords=*web*'))).toBe(
      '/service/rest/v1/search?helm.keywords=*web*',
    );
    expect(criteriaPath(new URLSearchParams('mode=assets&unknown=1'))).toBeNull();
  });
});
