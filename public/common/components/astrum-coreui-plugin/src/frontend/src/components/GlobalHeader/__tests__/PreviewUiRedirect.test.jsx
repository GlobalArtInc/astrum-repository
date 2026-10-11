

import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

const mockStateValues = {
  'astrum-context-path': '/',
  'anonymousEnabled': true,
  'loggedInEnabled': true,
  'defaultToPreviewUi': true,
  'disableLegacyUi': true,
};

const mockUseUser = jest.fn(() => ({ authenticated: false }));
const mockUseStatus = jest.fn(() => ({ edition: 'PRO' }));
const mockCheckPermission = jest.fn(() => true);
const mockRestClientPost = jest.fn(() => Promise.resolve({ data: {} }));
const mockState = jest.fn(() => ({
  getValue: jest.fn((key, defaultVal) => mockStateValues[key] ?? defaultVal),
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    useStatus: (...args) => mockUseStatus(...args),
    useUser: (...args) => mockUseUser(...args),
    state: (...args) => mockState(...args),
    checkPermission: (...args) => mockCheckPermission(...args),
    isExtJsRendered: jest.fn(() => false),
    refresh: jest.fn(),
  },
  handleExtJsUnsavedChanges: jest.fn((_, cb) => cb()),
  restClient: {
    post: (...args) => mockRestClientPost(...args),
  },
  useSideNavbarOpenState: () => [true],
}));

jest.mock('@uirouter/react', () => ({
  useRouter: jest.fn(() => ({ stateService: { go: jest.fn() } })),
}));

jest.mock('../../../contexts/ThemeContext', () => ({
  useTheme: jest.fn(() => ({ effectiveTheme: 'light', setTheme: jest.fn() })),
}));

jest.mock('../SearchRadix', () => () => <div data-testid="search-radix" />);
jest.mock('../SystemStatusRadix', () => () => <div data-testid="system-status" />);
jest.mock('../HelpMenuRadix', () => () => <div data-testid="help-menu" />);
jest.mock('../LoginAndUserButtonRadix', () => () => <div data-testid="login-button" />);
jest.mock('../../ThemeSwitcher/ThemeSwitcher', () => () => null);

import GlobalHeaderRadix from '../GlobalHeaderRadix';

function renderHeader() {
  return render(
    <Theme>
      <GlobalHeaderRadix />
    </Theme>
  );
}

describe('GlobalHeaderRadix - Classic URLs open Astrum One', () => {
  const originalHash = window.location.hash;

  beforeEach(() => {
    jest.clearAllMocks();
    window.location.hash = '';
    sessionStorage.clear();
    localStorage.clear();
    mockUseUser.mockReturnValue({ authenticated: false });
    mockUseStatus.mockReturnValue({ edition: 'PRO' });
    mockCheckPermission.mockReturnValue(true);
    mockRestClientPost.mockClear();
    mockStateValues['anonymousEnabled'] = true;
    mockStateValues['loggedInEnabled'] = true;
    mockStateValues['defaultToPreviewUi'] = true;
    mockStateValues['disableLegacyUi'] = true;
    mockState.mockReturnValue({
      getValue: jest.fn((key, defaultVal) => mockStateValues[key] ?? defaultVal),
    });
  });

  afterAll(() => {
    window.location.hash = originalHash;
  });

  const CLASSIC_URLS = [
    ['', '#preview/browse/welcome'],
    ['/', '#preview/browse/welcome'],
    ['browse/welcome', '#preview/browse/welcome'],
    ['admin/security/users', '#preview/admin/security/users'],
    ['browse/browse:maven-releases', '#preview/browse/maven-releases/'],
    ['browse/search/maven', '#preview/browse/search?format=maven'],
    ['admin/repository/cleanuppolicies', '#preview/admin/repository/cleanup-policies'],
    ['user/account', '#preview/user/account'],
    ['admin/system/api', '#preview/browse/api'],
    ['admin/system/blobstores', '#preview/browse/welcome'],
  ];

  describe.each([
    ['anonymous visitor', { authenticated: false }],
    ['signed-in user', { authenticated: true, userId: 'admin' }],
  ])('for a %s', (_who, user) => {
    it.each(CLASSIC_URLS)('opens %p as %p', async (classic, preview) => {
      mockUseUser.mockReturnValue(user);
      window.location.hash = classic;
      renderHeader();
      await waitFor(() => {
        expect(window.location.hash).toBe(preview);
      });
    });
  });

  it('leaves Astrum One URLs alone', async () => {
    window.location.hash = 'preview/admin/security/roles';
    renderHeader();
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(window.location.hash).toBe('#preview/admin/security/roles');
  });

  it.each([['login'], ['login?returnTo=I2FkbWlu'], ['/login'], ['404']])(
    'leaves the shared %p page alone so a visitor who must sign in is not sent in circles', async (path) => {
      window.location.hash = path;
      renderHeader();
      await new Promise((resolve) => setTimeout(resolve, 50));
      expect(window.location.hash).toBe(`#${path}`);
    });

  it('ignores a Classic choice left in this browser by an earlier release', async () => {
    sessionStorage.setItem('user_requested_legacy', 'true');
    localStorage.setItem('astrum-last-ui', 'classic');
    window.location.hash = 'browse/welcome';
    renderHeader();
    await waitFor(() => {
      expect(window.location.hash).toBe('#preview/browse/welcome');
    });
  });

  it('redirects whatever defaultToPreviewUi and disableLegacyUi say', async () => {
    mockStateValues['defaultToPreviewUi'] = false;
    mockStateValues['disableLegacyUi'] = false;
    window.location.hash = 'admin/security/roles';
    renderHeader();
    await waitFor(() => {
      expect(window.location.hash).toBe('#preview/admin/security/roles');
    });
  });

  it('remembers Astrum One as the interface used last once it is open', async () => {
    window.location.hash = 'preview/browse/welcome';
    renderHeader();
    await waitFor(() => {
      expect(localStorage.getItem('astrum-last-ui')).toBe('preview');
    });
  });

  it('shows no interface switch on either side', () => {
    window.location.hash = 'preview/browse/welcome';
    renderHeader();
    expect(screen.queryByText('Switch to Astrum One UI')).not.toBeInTheDocument();
    expect(screen.queryByText('Switch to Classic UI')).not.toBeInTheDocument();
    expect(screen.queryByTestId('ui-toggle-button')).not.toBeInTheDocument();
  });
});
