const mockGetValue = jest.fn();
// Controls what the mocked ExtJS.state() returns; each test sets it (an object, null, or undefined).
let mockState;

jest.mock('@globalart/astrum-ui-plugin', () => {
  const actual = jest.requireActual(
    '../../../../../astrum-ui-plugin/src/frontend/src/utils/loginUtils'
  );
  return {
    ExtJS: { state: () => mockState },
    PREVIEW_WELCOME_ROUTE: actual.PREVIEW_WELCOME_ROUTE,
  };
});

jest.mock('./routeNames/routeNames', () => ({
  ROUTE_NAMES: {
    LOGIN: 'login',
    BROWSE: { WELCOME: { ROOT: 'browse.welcome' } },
  },
}));

import { getInitialRoute } from './getInitialRoute';

describe('getInitialRoute', () => {
  beforeEach(() => {
    mockGetValue.mockReset();
    mockState = { getValue: mockGetValue };
    sessionStorage.clear();
    localStorage.clear();
  });

  const stateWith = (values) => (key, def) => (key in values ? values[key] : def);

  describe('when ExtJS.state is not available', () => {
    it('returns the Astrum One dashboard when ExtJS.state() returns null', () => {
      mockState = null;
      expect(getInitialRoute()).toBe('preview.browse.welcome');
    });

    it('returns the Astrum One dashboard when ExtJS.state() returns undefined', () => {
      mockState = undefined;
      expect(getInitialRoute()).toBe('preview.browse.welcome');
    });

    it('returns the Astrum One dashboard when ExtJS.state() throws', () => {
      mockGetValue.mockImplementation(() => {
        throw new Error('not ready');
      });
      expect(getInitialRoute()).toBe('preview.browse.welcome');
    });
  });

  it('returns login when anonymous access is disabled (real but falsy anonUser)', () => {
    mockGetValue.mockImplementation(stateWith({ anonymousUsername: '' }));
    expect(getInitialRoute()).toBe('login');
  });

  it('returns the Astrum One dashboard when state is not seeded (anonUser null)', () => {
    mockGetValue.mockImplementation(stateWith({ anonymousUsername: null }));
    expect(getInitialRoute()).toBe('preview.browse.welcome');
  });

  it.each([
    ['all Preview flags on', { defaultToPreviewUi: true, anonymousEnabled: true }],
    ['defaultToPreviewUi off', { defaultToPreviewUi: false, anonymousEnabled: true }],
    ['anonymousEnabled off', { defaultToPreviewUi: true, anonymousEnabled: false }],
  ])('sends an anonymous visitor to the Astrum One dashboard with %s', (_flags, values) => {
    mockGetValue.mockImplementation(stateWith({ anonymousUsername: 'anonymous', ...values }));
    expect(getInitialRoute()).toBe('preview.browse.welcome');
  });

  it('ignores a Classic choice saved in this browser by an earlier release', () => {
    sessionStorage.setItem('user_requested_legacy', 'true');
    localStorage.setItem('astrum-last-ui', 'classic');
    mockGetValue.mockImplementation(stateWith({ anonymousUsername: 'anonymous' }));
    expect(getInitialRoute()).toBe('preview.browse.welcome');
  });
});
