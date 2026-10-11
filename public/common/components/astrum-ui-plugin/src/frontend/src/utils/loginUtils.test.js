

import {
  parseRetryAfter,
  resolveDefaultLandingRoute,
  PREVIEW_WELCOME_HASH,
  PREVIEW_WELCOME_ROUTE,
  USER_REQUESTED_LEGACY_KEY,
  toPreviewReturnTo,
  toPreviewReturnToParam,
} from './loginUtils';

describe('parseRetryAfter', () => {
  it('returns null for absent header', () => {
    expect(parseRetryAfter(null)).toBeNull();
    expect(parseRetryAfter(undefined)).toBeNull();
    expect(parseRetryAfter('')).toBeNull();
  });

  it('parses a delay-seconds integer string', () => {
    expect(parseRetryAfter('30')).toBe(30);
    expect(parseRetryAfter('120')).toBe(120);
  });

  it('returns at least 1 for zero or negative delay-seconds', () => {
    expect(parseRetryAfter('0')).toBe(1);
    expect(parseRetryAfter('-5')).toBe(1);
  });

  it('parses a future HTTP-date string', () => {
    const future = new Date(Date.now() + 60000).toUTCString();
    const result = parseRetryAfter(future);
    expect(result).toBeGreaterThan(0);
    expect(result).toBeLessThanOrEqual(60);
  });

  it('returns null for an unparseable value', () => {
    expect(parseRetryAfter('not-a-date-or-number')).toBeNull();
  });

  it('returns null for a past HTTP-date string', () => {
    const past = new Date(Date.now() - 60000).toUTCString();
    expect(parseRetryAfter(past)).toBeNull();
  });
});

describe('resolveDefaultLandingRoute', () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  it('opens the Astrum One dashboard', () => {
    expect(resolveDefaultLandingRoute()).toBe(PREVIEW_WELCOME_ROUTE);
  });

  it('ignores a Classic choice saved in this browser by an earlier release', () => {
    localStorage.setItem('astrum-last-ui', 'classic');
    sessionStorage.setItem(USER_REQUESTED_LEGACY_KEY, 'true');
    expect(resolveDefaultLandingRoute()).toBe(PREVIEW_WELCOME_ROUTE);
  });

  it('exposes the expected constant values', () => {
    expect(PREVIEW_WELCOME_ROUTE).toBe('preview.browse.welcome');
    expect(PREVIEW_WELCOME_HASH).toBe('#browse/welcome');
    expect(USER_REQUESTED_LEGACY_KEY).toBe('user_requested_legacy');
  });
});

describe('toPreviewReturnTo', () => {
  it.each([
    ['#preview/admin/security/roles', '#admin/security/roles'],
    ['#/preview/browse/welcome?tab=overview', '#/browse/welcome?tab=overview'],
    ['/preview/admin/system/tasks', '/admin/system/tasks'],
    ['#preview', '#'],
    ['#admin/security/users/jdoe/default', '#admin/security/users/jdoe/default'],
    ['#browse/maven-releases/com/foo', '#browse/maven-releases/com/foo'],
    ['#admin/repository/cleanuppolicies', '#admin/repository/cleanuppolicies'],
    ['#', '#'],
  ])('maps %p to %p', (returnTo, expected) => {
    expect(toPreviewReturnTo(returnTo)).toBe(expected);
  });
});

describe('toPreviewReturnToParam', () => {
  it('drops the preview/ prefix of a Base64 returnTo and encodes it again', () => {
    expect(atob(toPreviewReturnToParam(btoa('#preview/admin/security/users')))).toBe('#admin/security/users');
  });

  it('drops the preview/ prefix of a raw returnTo and Base64-encodes it', () => {
    expect(atob(toPreviewReturnToParam('#preview/admin/system/tasks'))).toBe('#admin/system/tasks');
  });

  it('keeps an Astrum One returnTo, which the router also opens for a Classic one', () => {
    expect(atob(toPreviewReturnToParam(btoa('#browse/welcome')))).toBe('#browse/welcome');
    expect(atob(toPreviewReturnToParam(btoa('#admin/repository/cleanuppolicies'))))
      .toBe('#admin/repository/cleanuppolicies');
  });

  it.each([[undefined], [null], ['']])('leaves a missing returnTo %p missing', (value) => {
    expect(toPreviewReturnToParam(value)).toBe(value);
  });

  it('returns a value that is neither a hash nor a path unchanged', () => {
    expect(toPreviewReturnToParam('%23browse%2Fsearch')).toBe('%23browse%2Fsearch');
  });
});
