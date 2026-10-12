

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';

const mockStateValues = {
  'astrum-context-path': '/',
  'anonymousEnabled': true,
  'loggedInEnabled': true,
  'disableSwitchFeedback': false,
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
  },
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
jest.mock('../LanguageMenuRadix', () => () => <div data-testid="language-menu" />);
jest.mock('../LoginAndUserButtonRadix', () => () => <div data-testid="login-button" />);
jest.mock('../../ThemeSwitcher/ThemeSwitcher', () => () => null);
jest.mock('../SearchLayer', () => ({ onClose }) => (
  <div data-testid="search-layer">
    <button type="button" onClick={onClose}>close layer</button>
  </div>
));

import GlobalHeaderRadix from '../GlobalHeaderRadix';

function renderHeader() {
  return render(
    <Theme>
      <GlobalHeaderRadix />
    </Theme>
  );
}

describe('GlobalHeaderRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.location.hash = '';
    mockUseUser.mockReturnValue({ authenticated: false });
    mockUseStatus.mockReturnValue({ edition: 'PRO', version: '3.78.0' });
    mockCheckPermission.mockReturnValue(true);
    mockStateValues['anonymousEnabled'] = true;
    mockStateValues['loggedInEnabled'] = true;
    mockStateValues['disableSwitchFeedback'] = false;
    mockState.mockReturnValue({
      getValue: jest.fn((key, defaultVal) => mockStateValues[key] ?? defaultVal),
    });
  });

  afterEach(() => {
    sessionStorage.clear();
  });

  it('renders logo and core UI elements', () => {
    renderHeader();
    expect(screen.getByTitle('Home')).toBeInTheDocument();
    expect(screen.getByTestId('search-radix')).toBeInTheDocument();
    expect(screen.getByTestId('help-menu')).toBeInTheDocument();
    expect(screen.getByTestId('login-button')).toBeInTheDocument();
    expect(screen.getByLabelText('Refresh')).toBeInTheDocument();
  });

  it.each([
    ['an anonymous visitor in Astrum One', { authenticated: false }, 'browse/welcome'],
    ['a signed-in user in Astrum One', { authenticated: true, userId: 'admin' }, 'admin/repository/repositories'],
    ['a signed-in user on an unmapped Astrum One page', { authenticated: true, userId: 'admin' }, 'browse/audit'],
  ])('offers %s no way back to the Classic UI', (_who, user, hash) => {
    mockUseUser.mockReturnValue(user);
    window.location.hash = hash;

    renderHeader();

    expect(screen.queryByTestId('ui-toggle-button')).not.toBeInTheDocument();
    expect(screen.queryByText('Switch to Classic UI')).not.toBeInTheDocument();
    expect(screen.queryByText('Switch to Astrum One UI')).not.toBeInTheDocument();
    expect(screen.queryByText('Not Available in Classic UI')).not.toBeInTheDocument();
    expect(mockRestClientPost).not.toHaveBeenCalled();
  });

  describe('on phones and tablets', () => {
    const originalMatchMedia = window.matchMedia;

    function viewport(width) {
      window.matchMedia = jest.fn((query) => ({
        matches: width <= Number(/max-width:\s*(\d+)px/.exec(query)?.[1]),
        media: query,
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
      }));
    }

    beforeEach(() => {
      window.location.hash = 'browse/welcome';
      mockUseUser.mockReturnValue({ authenticated: true, userId: 'admin' });
    });

    afterEach(() => {
      window.matchMedia = originalMatchMedia;
    });

    it('uses the phone header on every page, now that Astrum One owns the root of the hash', () => {
      viewport(390);
      window.location.hash = 'admin/security/users';

      const { container } = renderHeader();

      expect(container.querySelector('.as-global-header-radix--phone')).not.toBeNull();
      expect(screen.queryByTestId('search-radix')).not.toBeInTheDocument();
    });

    it('shows the icons of the phone header and moves the rest into the drawer', () => {
      viewport(390);

      const { container } = renderHeader();

      expect(container.querySelector('.as-global-header-radix--phone')).not.toBeNull();
      expect(screen.getByRole('button', { name: 'Open menu' })).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getByRole('button', { name: 'Search' })).toBeInTheDocument();
      expect(screen.getByTestId('system-status')).toBeInTheDocument();
      expect(screen.getByTestId('login-button')).toBeInTheDocument();
      expect(screen.queryByTestId('search-radix')).not.toBeInTheDocument();
      expect(screen.queryByText('Switch to Classic UI')).not.toBeInTheDocument();
      expect(screen.queryByLabelText('Refresh')).not.toBeInTheDocument();
      expect(screen.queryByTestId('language-menu')).not.toBeInTheDocument();
      expect(screen.queryByTestId('help-menu')).not.toBeInTheDocument();
    });

    it('keeps language and help on tablets', () => {
      viewport(768);

      const { container } = renderHeader();

      expect(container.querySelector('.as-global-header-radix--compact')).not.toBeNull();
      expect(container.querySelector('.as-global-header-radix--phone')).toBeNull();
      expect(screen.getByTestId('language-menu')).toBeInTheDocument();
      expect(screen.getByTestId('help-menu')).toBeInTheDocument();
      expect(screen.queryByText('Switch to Classic UI')).not.toBeInTheDocument();
    });

    it('opens the menu drawer instead of collapsing the side menu', () => {
      viewport(390);
      const onDrawer = jest.fn();
      const onSidebar = jest.fn();
      window.addEventListener('as-navigation-drawer', onDrawer);
      window.addEventListener('as-sidebar-toggle', onSidebar);

      renderHeader();
      fireEvent.click(screen.getByRole('button', { name: 'Open menu' }));

      expect(onDrawer).toHaveBeenCalledTimes(1);
      expect(onDrawer.mock.calls[0][0].detail).toEqual({ open: true });
      expect(onSidebar).not.toHaveBeenCalled();
      expect(screen.getByRole('button', { name: 'Close menu' })).toHaveAttribute('aria-expanded', 'true');

      window.removeEventListener('as-navigation-drawer', onDrawer);
      window.removeEventListener('as-sidebar-toggle', onSidebar);
    });

    it('opens the search full screen and closes it again', () => {
      viewport(390);

      renderHeader();
      fireEvent.click(screen.getByRole('button', { name: 'Search' }));
      expect(screen.getByTestId('search-layer')).toBeInTheDocument();

      fireEvent.click(screen.getByText('close layer'));
      expect(screen.queryByTestId('search-layer')).not.toBeInTheDocument();
    });

    it('offers no switch to the Classic UI', () => {
      viewport(390);

      renderHeader();

      expect(screen.queryByTestId('ui-toggle-button')).not.toBeInTheDocument();
    });
  });
});
