import { describe, expect, it } from 'vitest';
import {
  fromSortChoice,
  isSearchable,
  readSearchState,
  SORT_CHOICES,
  searchAddress,
  searchPath,
  sortChoice,
  writeSearchState,
} from './query';

describe('the search address', () => {
  it('reads the query, filters and sort with newest first by default', () => {
    expect(readSearchState(new URLSearchParams(''))).toEqual({
      q: '',
      format: '',
      repository: '',
      sort: '',
      direction: 'desc',
    });
    expect(
      readSearchState(
        new URLSearchParams('q=junit&format=maven2&repository=central&sort=name&direction=desc'),
      ),
    ).toEqual({ q: 'junit', format: 'maven2', repository: 'central', sort: 'name', direction: 'desc' });
    expect(readSearchState(new URLSearchParams('sort=version')).direction).toBe('asc');
    expect(readSearchState(new URLSearchParams('sort=size&direction=up'))).toMatchObject({
      sort: '',
      direction: 'desc',
    });
  });

  it('writes only what differs from the defaults', () => {
    const state = readSearchState(new URLSearchParams(''));
    expect(writeSearchState(state).toString()).toBe('');
    expect(writeSearchState({ ...state, q: 'a b', direction: 'asc' }).toString()).toBe('q=a+b&direction=asc');
    expect(
      writeSearchState({
        ...state,
        format: 'npm',
        repository: 'npm-internal',
        sort: 'name',
        direction: 'asc',
      }).toString(),
    ).toBe('format=npm&repository=npm-internal&sort=name');
    for (const text of ['q=x&sort=repository&direction=desc', 'format=pypi&direction=asc']) {
      expect(writeSearchState(readSearchState(new URLSearchParams(text))).toString()).toBe(text);
    }
  });

  it('maps sort choices to sort and direction and back', () => {
    for (const choice of SORT_CHOICES) {
      expect(sortChoice(fromSortChoice(choice.value))).toBe(choice.value);
    }
    expect(fromSortChoice('newest')).toEqual({ sort: '', direction: 'desc' });
    expect(fromSortChoice('version-desc')).toEqual({ sort: 'version', direction: 'desc' });
    expect(fromSortChoice('nonsense')).toEqual({ sort: '', direction: 'desc' });
  });

  it('searches once there are two characters or a filter', () => {
    const empty = readSearchState(new URLSearchParams(''));
    expect(isSearchable(empty)).toBe(false);
    expect(isSearchable({ ...empty, q: ' a ' })).toBe(false);
    expect(isSearchable({ ...empty, q: 'ab' })).toBe(true);
    expect(isSearchable({ ...empty, format: 'raw' })).toBe(true);
    expect(isSearchable({ ...empty, repository: 'x' })).toBe(true);
  });

  it('asks the server with an explicit direction', () => {
    expect(searchPath(readSearchState(new URLSearchParams('q=+junit+&sort=name')))).toBe(
      '/service/rest/v1/search?q=junit&sort=name&direction=asc',
    );
    expect(searchPath(readSearchState(new URLSearchParams('repository=r1')))).toBe(
      '/service/rest/v1/search?repository=r1&direction=desc',
    );
  });

  it('builds page addresses for everywhere and one repository', () => {
    expect(searchAddress(' log4j ')).toBe('/search?q=log4j');
    expect(searchAddress('a&b', 'maven central')).toBe('/search?q=a%26b&repository=maven+central');
    expect(searchAddress('')).toBe('/search');
  });
});
