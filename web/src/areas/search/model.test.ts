import { describe, expect, it } from 'vitest';
import { currentRepository, matchRepositories } from './HeaderSearch';
import { browsePath, componentPath, folderOf, lastModified, type SearchComponent } from './model';
import { forgetSearch, RECENT_KEY, readRecent, rememberSearch } from './recent';

const asset = (lastModified: string | null) => ({
  id: lastModified ?? 'none',
  path: '/a',
  downloadUrl: 'http://x/a',
  repository: 'r',
  format: 'raw',
  contentType: null,
  lastModified,
  fileSize: 1,
  checksum: {},
});

describe('search addresses', () => {
  it('point at the component page with the version', () => {
    expect(componentPath({ format: 'maven2', group: 'org.example', name: 'lib', version: '1.0' })).toBe(
      '/search/component/maven2%3Aorg.example%3Alib?version=1.0',
    );
    expect(componentPath({ format: 'npm', group: null, name: 'left-pad', version: '1.3.0+b' })).toBe(
      '/search/component/npm%3A%3Aleft-pad?version=1.3.0%2Bb',
    );
    expect(componentPath({ format: 'raw', group: '/a', name: '/a/b c.txt', version: '' })).toBe(
      '/search/component/raw%3A%2Fa%3A%2Fa%2Fb%20c.txt',
    );
  });

  it('point at browse folders', () => {
    expect(browsePath('raw files')).toBe('/browse/raw%20files');
    expect(browsePath('npm', '/@team/tool/-/')).toBe('/browse/npm/%40team/tool/-');
    expect(folderOf('/a/b/c.txt')).toBe('/a/b');
    expect(folderOf('/c.txt')).toBe('');
  });

  it('take the newest asset time of a component', () => {
    const component: SearchComponent = {
      id: '1',
      repository: 'r',
      format: 'raw',
      group: null,
      name: 'x',
      version: null,
      assets: [asset('2026-10-01T10:00:00.000+00:00'), asset(null), asset('2026-10-02T09:00:00.000+02:00')],
    };
    expect(lastModified(component)).toBe('2026-10-02T09:00:00.000+02:00');
    expect(lastModified({ ...component, assets: [] })).toBeNull();
  });
});

describe('the header search', () => {
  it('knows the repository of a browse page', () => {
    expect(currentRepository('/browse/maven%20central/org/x')).toBe('maven central');
    expect(currentRepository('/browse/npm')).toBe('npm');
    expect(currentRepository('/browse')).toBeNull();
    expect(currentRepository('/search')).toBeNull();
  });

  it('puts repositories that start with the text first and keeps five', () => {
    const repositories = ['b-npm', 'npm-a', 'npm-b', 'x-npm-y', 'npm-c', 'npm-d', 'npm-e', 'raw'].map(
      (name) => ({
        name,
        format: 'npm',
        type: 'hosted' as const,
        url: '',
      }),
    );
    expect(matchRepositories(repositories, 'NPM').map((repository) => repository.name)).toEqual([
      'npm-a',
      'npm-b',
      'npm-c',
      'npm-d',
      'npm-e',
    ]);
    expect(matchRepositories(repositories, ' ')).toEqual([]);
  });

  it('remembers five recent searches, newest first, without repeats', () => {
    expect(readRecent()).toEqual([]);
    for (const text of ['one', 'two', 'three', 'two', 'four', 'five', 'six', '  ']) {
      rememberSearch(text);
    }
    expect(readRecent()).toEqual(['six', 'five', 'four', 'two', 'three']);
    expect(forgetSearch('four')).toEqual(['six', 'five', 'two', 'three']);
    window.localStorage.setItem(RECENT_KEY, '{broken');
    expect(readRecent()).toEqual([]);
    window.localStorage.setItem(RECENT_KEY, JSON.stringify(['a', 3, '', 'b']));
    expect(readRecent()).toEqual(['a', 'b']);
  });
});
