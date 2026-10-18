import { describe, expect, it } from 'vitest';
import {
  componentKey,
  componentPageUrl,
  componentTitle,
  guideComponentName,
  parseComponentKey,
} from './links';

describe('component addresses', () => {
  it('encode format, group and name and read them back', () => {
    expect(componentPageUrl('maven2', 'org.acme', 'lib', '1.0', 'files')).toBe(
      '/search/component/maven2%3Aorg.acme%3Alib?version=1.0&tab=files',
    );
    expect(componentPageUrl('npm', '', 'left-pad')).toBe('/search/component/npm%3A%3Aleft-pad');
    expect(parseComponentKey(componentKey('raw', '/docs', '/docs/a:b.txt'))).toEqual({
      format: 'raw',
      group: '/docs',
      name: '/docs/a:b.txt',
    });
    expect(parseComponentKey('docker::team/app')).toEqual({ format: 'docker', group: '', name: 'team/app' });
    expect(parseComponentKey('maven2:x')).toBeNull();
    expect(parseComponentKey(':a:b')).toBeNull();
    expect(parseComponentKey('npm:a:')).toBeNull();
  });

  it('name components the way their format writes them', () => {
    expect(componentTitle('maven2', 'org.acme', 'lib')).toBe('org.acme:lib');
    expect(componentTitle('npm', 'team', 'tool')).toBe('@team/tool');
    expect(componentTitle('npm', '', 'tool')).toBe('tool');
    expect(componentTitle('raw', '/docs', '/docs/a.txt')).toBe('/docs/a.txt');
  });
});

describe('guide names', () => {
  it('name a package the way its public registry does', () => {
    expect(guideComponentName('maven2', 'org.acme', 'lib')).toBe('org.acme:lib');
    expect(guideComponentName('npm', 'babel', 'core')).toBe('@babel/core');
    expect(guideComponentName('npm', '@babel', 'core')).toBe('@babel/core');
    expect(guideComponentName('npm', '', 'react')).toBe('react');
    expect(guideComponentName('composer', 'symfony', 'console')).toBe('symfony/console');
    expect(guideComponentName('conan', 'stable', 'zlib')).toBe('zlib');
    expect(guideComponentName('go', '', 'github.com/gin-gonic/gin')).toBe('github.com/gin-gonic/gin');
  });
});
