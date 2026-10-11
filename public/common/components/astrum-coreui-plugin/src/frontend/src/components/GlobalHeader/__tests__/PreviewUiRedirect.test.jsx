import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

const mockStateValues = {
  'astrum-context-path': '/',
};

const mockUseUser = jest.fn(() => ({ authenticated: false }));
const mockUseStatus = jest.fn(() => ({ edition: 'PRO' }));
const mockState = jest.fn(() => ({
  getValue: jest.fn((key, defaultVal) => mockStateValues[key] ?? defaultVal),
}));

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    useStatus: (...args) => mockUseStatus(...args),
    useUser: (...args) => mockUseUser(...args),
    state: (...args) => mockState(...args),
    checkPermission: jest.fn(() => true),
    isExtJsRendered: jest.fn(() => false),
    refresh: jest.fn(),
  },
  handleExtJsUnsavedChanges: jest.fn((_, cb) => cb()),
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

/*
 * Astrum One owns the root of the hash, and the URL rules of the router open the Astrum One page of an old
 * URL (see routerConfig.test.js and createRouter.test.js). The header no longer redirects on its own.
 */
describe('GlobalHeaderRadix - old URLs are left to the router', () => {
  const originalHash = window.location.hash;

  beforeEach(() => {
    jest.clearAllMocks();
    window.location.hash = '';
    sessionStorage.clear();
    localStorage.clear();
  });

  afterAll(() => {
    window.location.hash = originalHash;
  });

  describe.each([
    ['anonymous visitor', { authenticated: false }],
    ['signed-in user', { authenticated: true, userId: 'admin' }],
  ])('for a %s', (_who, user) => {
    it.each([
      ['browse/welcome'],
      ['admin/security/users'],
      ['admin/repository/cleanuppolicies'],
      ['browse/browse:maven-releases'],
      ['preview/admin/security/roles'],
      ['login?returnTo=I2FkbWlu'],
      ['404'],
    ])('leaves %p alone', async (path) => {
      mockUseUser.mockReturnValue(user);
      window.location.hash = path;
      renderHeader();
      await new Promise((resolve) => setTimeout(resolve, 50));
      expect(window.location.hash).toBe(`#${path}`);
    });
  });

  it('remembers no interface: there is only one', async () => {
    window.location.hash = 'browse/welcome';
    renderHeader();
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(localStorage.getItem('astrum-last-ui')).toBeNull();
  });

  it('links the logo to the Astrum One dashboard at the root of the hash', () => {
    window.location.hash = 'admin/security/users';
    renderHeader();
    expect(screen.getByTitle('Home')).toHaveAttribute('href', '/#browse/welcome');
  });

  it('shows no interface switch', () => {
    window.location.hash = 'browse/welcome';
    renderHeader();
    expect(screen.queryByText('Switch to Astrum One UI')).not.toBeInTheDocument();
    expect(screen.queryByText('Switch to Classic UI')).not.toBeInTheDocument();
    expect(screen.queryByTestId('ui-toggle-button')).not.toBeInTheDocument();
  });
});
