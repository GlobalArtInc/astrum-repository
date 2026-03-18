import { describe, expect, it } from 'vitest';
import { loginPath, safeReturnTo } from './returnTo';

describe('return addresses', () => {
  it('keeps paths of this site', () => {
    expect(safeReturnTo('/browse/maven-releases?tab=usage')).toBe('/browse/maven-releases?tab=usage');
  });

  it('refuses other sites and the login page itself', () => {
    for (const value of [
      null,
      '',
      'https://evil.example',
      '//evil.example',
      '/\\evil',
      '/login',
      '/login?x=1',
    ]) {
      expect(safeReturnTo(value)).toBe('/');
    }
  });

  it('builds login links', () => {
    expect(loginPath('/')).toBe('/login');
    expect(loginPath('/settings?tab=a')).toBe('/login?returnTo=%2Fsettings%3Ftab%3Da');
  });
});
