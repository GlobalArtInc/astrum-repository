

import {
  parseRetryAfter,
  resolveDefaultLandingRoute,
  isPreviewHash,
  isClassicLandingHash,
  returnToTargetsPreview,
  returnToTargetsClassicLanding,
  PREVIEW_WELCOME_ROUTE,
  CLASSIC_WELCOME_ROUTE,
  USER_REQUESTED_LEGACY_KEY,
  LAST_USED_UI_KEY,
  getLastUsedUi,
  setLastUsedUi,
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

describe('last used interface', () => {
  beforeEach(() => localStorage.clear());

  it('is null until an interface is remembered', () => {
    expect(getLastUsedUi()).toBeNull();
  });

  it('remembers preview and classic in localStorage', () => {
    setLastUsedUi('preview');
    expect(localStorage.getItem(LAST_USED_UI_KEY)).toBe('preview');
    expect(getLastUsedUi()).toBe('preview');
    setLastUsedUi('classic');
    expect(getLastUsedUi()).toBe('classic');
  });

  it('ignores values it does not know', () => {
    localStorage.setItem(LAST_USED_UI_KEY, 'other');
    expect(getLastUsedUi()).toBeNull();
  });
});

describe('resolveDefaultLandingRoute', () => {
  beforeEach(() => localStorage.clear());

  it('opens the Astrum One dashboard for a user allowed into it', () => {
    expect(resolveDefaultLandingRoute({ canAccessPreviewUi: true })).toBe(PREVIEW_WELCOME_ROUTE);
  });

  it.each([
    ['Classic used last in this browser', { lastUsedUi: 'classic' }],
    ['the Classic UI requested in this session', { userRequestedLegacy: true }],
    ['defaultToPreviewUi off', { defaultToPreviewUi: false }],
  ])('ignores %s, saved by an earlier release', (_what, options) => {
    setLastUsedUi('classic');
    expect(resolveDefaultLandingRoute({ canAccessPreviewUi: true, ...options })).toBe(PREVIEW_WELCOME_ROUTE);
  });

  it('opens the Classic dashboard only when the server does not report Astrum One access', () => {
    expect(resolveDefaultLandingRoute({ canAccessPreviewUi: false })).toBe(CLASSIC_WELCOME_ROUTE);
  });

  it('exposes the expected constant values', () => {
    expect(PREVIEW_WELCOME_ROUTE).toBe('preview.browse.welcome');
    expect(CLASSIC_WELCOME_ROUTE).toBe('browse.welcome');
    expect(USER_REQUESTED_LEGACY_KEY).toBe('user_requested_legacy');
  });
});

describe('toPreviewReturnTo', () => {
  it.each([
    ['#admin/security/users', '#preview/admin/security/users'],
    ['#/admin/security/users', '#/preview/admin/security/users'],
    ['/admin/repository/cleanuppolicies', '/preview/admin/repository/cleanup-policies'],
    ['#browse/browse:maven-releases', '#preview/browse/maven-releases/'],
    ['#browse/search/maven', '#preview/browse/search?format=maven'],
    ['#', '#preview/browse/welcome'],
    ['#preview/admin/security/roles', '#preview/admin/security/roles'],
    ['#/preview/browse/welcome?tab=overview', '#/preview/browse/welcome?tab=overview'],
  ])('maps %p to %p', (classic, preview) => {
    expect(toPreviewReturnTo(classic)).toBe(preview);
  });
});

describe('toPreviewReturnToParam', () => {
  it('maps a Base64 Classic returnTo and encodes it again', () => {
    expect(atob(toPreviewReturnToParam(btoa('#admin/security/users')))).toBe('#preview/admin/security/users');
  });

  it('maps a raw Classic returnTo and Base64-encodes it', () => {
    expect(atob(toPreviewReturnToParam('#admin/system/tasks'))).toBe('#preview/admin/system/tasks');
  });

  it('keeps an Astrum One returnTo', () => {
    expect(atob(toPreviewReturnToParam(btoa('#preview/browse/welcome')))).toBe('#preview/browse/welcome');
  });

  it.each([[undefined], [null], ['']])('leaves a missing returnTo %p missing', (value) => {
    expect(toPreviewReturnToParam(value)).toBe(value);
  });

  it('returns a value that is neither a hash nor a path unchanged', () => {
    expect(toPreviewReturnToParam('%23browse%2Fsearch')).toBe('%23browse%2Fsearch');
  });
});


describe('isPreviewHash', () => {
  it('returns true for preview hashes', () => {
    expect(isPreviewHash('#preview/browse/welcome')).toBe(true);
    expect(isPreviewHash('#/preview/browse/welcome')).toBe(true);
    expect(isPreviewHash('#preview')).toBe(true);
    expect(isPreviewHash('preview/browse/welcome')).toBe(true);
  });

  it('returns false for classic hashes and empty values', () => {
    expect(isPreviewHash('#browse/welcome')).toBe(false);
    expect(isPreviewHash('#/browse/welcome')).toBe(false);
    expect(isPreviewHash('#admin/repository/repositories')).toBe(false);
    expect(isPreviewHash('')).toBe(false);
    expect(isPreviewHash(null)).toBe(false);
    expect(isPreviewHash(undefined)).toBe(false);
  });
});

describe('returnToTargetsPreview', () => {
  it('returns true for a Base64-encoded preview returnTo', () => {
    expect(returnToTargetsPreview(btoa('#preview/browse/welcome'))).toBe(true);
    expect(returnToTargetsPreview(btoa('#/preview/browse/welcome'))).toBe(true);
  });

  it('returns false for a Base64-encoded classic returnTo', () => {
    expect(returnToTargetsPreview(btoa('#browse/welcome'))).toBe(false);
    expect(returnToTargetsPreview(btoa('#admin/repository/repositories'))).toBe(false);
  });

  it('handles raw (non-Base64) hash values', () => {
    expect(returnToTargetsPreview('#preview/browse/welcome')).toBe(true);
    expect(returnToTargetsPreview('#admin/repository/repositories')).toBe(false);
  });

  it('returns false for empty values', () => {
    expect(returnToTargetsPreview('')).toBe(false);
    expect(returnToTargetsPreview(null)).toBe(false);
    expect(returnToTargetsPreview(undefined)).toBe(false);
  });
});

describe('isClassicLandingHash', () => {
  it('returns true for Classic landing pages and empty values', () => {
    expect(isClassicLandingHash('#browse/welcome')).toBe(true);
    expect(isClassicLandingHash('#/browse/welcome')).toBe(true);
    expect(isClassicLandingHash('#')).toBe(true);
    expect(isClassicLandingHash('#/')).toBe(true);
    expect(isClassicLandingHash('')).toBe(true);
    expect(isClassicLandingHash(null)).toBe(true);
    expect(isClassicLandingHash(undefined)).toBe(true);
  });

  it('returns false for Classic deep links and Preview hashes', () => {
    expect(isClassicLandingHash('#admin/security/users')).toBe(false);
    expect(isClassicLandingHash('#browse/browse')).toBe(false);
    expect(isClassicLandingHash('#preview/browse/welcome')).toBe(false);
  });
});

describe('returnToTargetsClassicLanding', () => {
  it('returns true for a missing returnTo (default applies)', () => {
    expect(returnToTargetsClassicLanding('')).toBe(true);
    expect(returnToTargetsClassicLanding(null)).toBe(true);
    expect(returnToTargetsClassicLanding(undefined)).toBe(true);
  });

  it('returns true for a Base64-encoded Classic landing returnTo', () => {
    expect(returnToTargetsClassicLanding(btoa('#browse/welcome'))).toBe(true);
    expect(returnToTargetsClassicLanding(btoa('#/'))).toBe(true);
  });

  it('returns false for a Classic deep link (preserved)', () => {
    expect(returnToTargetsClassicLanding(btoa('#admin/security/users'))).toBe(false);
    expect(returnToTargetsClassicLanding('#admin/security/users')).toBe(false);
  });

  it('returns false for a Preview returnTo (preserved)', () => {
    expect(returnToTargetsClassicLanding(btoa('#preview/browse/welcome'))).toBe(false);
  });
});
