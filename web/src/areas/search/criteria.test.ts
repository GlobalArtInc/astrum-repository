import { describe, expect, it } from 'vitest';
import { buildValue, CRITERIA, criteriaPath, parseValue, readCriteria, writeCriteria } from './criteria';

const row = (key: string, value: string, operator: 'is' | 'startsWith' = 'is', id = 1) => ({
  id,
  key,
  operator,
  value,
});

describe('criteria', () => {
  it('offer the six fields of the old page', () => {
    expect(CRITERIA.map((criterion) => criterion.key)).toEqual([
      'q',
      'format',
      'repository',
      'group',
      'name',
      'version',
    ]);
  });

  it('turn "starts with" into a trailing wildcard', () => {
    expect(buildValue(row('name', ' junit '))).toBe('junit');
    expect(buildValue(row('name', 'junit', 'startsWith'))).toBe('junit*');
    expect(buildValue(row('q', 'spring', 'startsWith'))).toBe('spring*');
    expect(buildValue(row('name', 'junit*', 'startsWith'))).toBe('junit*');
    expect(buildValue(row('name', ' ', 'startsWith'))).toBe('');
  });

  it('read operators back from the address', () => {
    expect(parseValue('junit*')).toEqual({ operator: 'startsWith', value: 'junit' });
    expect(parseValue('ju*it')).toEqual({ operator: 'is', value: 'ju*it' });
    expect(parseValue('*')).toEqual({ operator: 'is', value: '*' });
    expect(parseValue('*unit*')).toEqual({ operator: 'is', value: '*unit*' });
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
    expect(criteriaPath(new URLSearchParams('mode=assets&unknown=1'))).toBeNull();
  });
});
