

import React from 'react';
import {render, screen, act} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import '@testing-library/jest-dom';
import {TooltipProvider} from '@radix-ui/react-tooltip';
import {Theme} from '@radix-ui/themes';

let mockUser = {authenticated: true};
let mockStatus = {edition: 'PRO'};
let mockStateValues = {
  'astrum-context-path': '',
  anonymousEnabled: true,
  loggedInEnabled: true,
};

let mockIsExtJsRendered = false;
const mockUseSideNavbarOpenState = jest.fn(() => [true]);

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    useUser: () => mockUser,
    useStatus: () => mockStatus,
    state: () => ({
      getValue: (key, defaultVal) => mockStateValues[key] ?? defaultVal,
    }),
    checkPermission: () => true,
    isExtJsRendered: () => mockIsExtJsRendered,
    showSuccessMessage: jest.fn(),
    refresh: jest.fn(),
  },
  handleExtJsUnsavedChanges: (_ctrl, fn) => fn(),
  hasUnsavedChanges: jest.fn(() => false),
  useSideNavbarOpenState: (...args) => mockUseSideNavbarOpenState(...args),
}));

jest.mock('@uirouter/react', () => ({
  useRouter: () => ({
    stateService: {go: jest.fn()},
    stateRegistry: {get: () => ({data: {visibilityRequirements: []}})},
  }),
  useCurrentStateAndParams: () => ({state: {name: 'browse.welcome'}, params: {}}),
}));

jest.mock('../../contexts/ThemeContext', () => ({
  useTheme: () => ({effectiveTheme: 'light', setTheme: jest.fn()}),
}));

jest.mock('./SearchRadix', () => () => <div data-testid="search-radix" />);
jest.mock('./SystemStatusRadix', () => () => <div data-testid="system-status" />);
jest.mock('../ThemeSwitcher/ThemeSwitcher', () => () => <div data-testid="theme-switcher" />);
jest.mock('./HelpMenuRadix', () => () => <div data-testid="help-menu" />);
jest.mock('./LanguageMenuRadix', () => () => <div data-testid="language-menu" />);
jest.mock('./LoginAndUserButtonRadix', () => () => <div data-testid="login-btn" />);
jest.mock('../../routerConfig/routerUtils', () => ({refreshReactPage: jest.fn()}));

import GlobalHeaderRadix from './GlobalHeaderRadix';
import {refreshReactPage} from '../../routerConfig/routerUtils';
import {hasUnsavedChanges} from '@globalart/astrum-ui-plugin';

function renderHeader(props) {
  return render(
    <Theme>
      <TooltipProvider>
        <GlobalHeaderRadix {...props} />
      </TooltipProvider>
    </Theme>
  );
}

describe('GlobalHeaderRadix', () => {
  beforeEach(() => {
    mockUser = {authenticated: true};
    mockStatus = {edition: 'PRO'};
    mockStateValues = {
      'astrum-context-path': '',
      anonymousEnabled: true,
      loggedInEnabled: true,
    };
    mockIsExtJsRendered = false;
    window.location.hash = '';
    jest.clearAllMocks();
    hasUnsavedChanges.mockReturnValue(false);
    mockUseSideNavbarOpenState.mockReturnValue([true]);
  });

  it('renders the header', () => {
    renderHeader();
    expect(screen.getByTestId('search-radix')).toBeInTheDocument();
  });

  it('shows UI toggle when user has access', () => {
    renderHeader();
    expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();
  });

  it('hides toggle for unauthenticated user with anonymous disabled', () => {
    mockUser = {authenticated: false};
    mockStateValues.anonymousEnabled = false;

    renderHeader();

    expect(screen.queryByText('Switch to Astrum One UI')).not.toBeInTheDocument();
  });

  describe('race condition fix (bug gyca)', () => {
    it('keeps toggle visible when loggedInEnabled flashes false during login', () => {
      const {rerender} = renderHeader();
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();

      mockStateValues.loggedInEnabled = false;
      mockUser = {authenticated: true, id: 'admin'};

      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);

      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();
    });

    it('keeps toggle visible during route transitions that reset ExtJS state', () => {
      const {rerender} = renderHeader();
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();

      mockStateValues.loggedInEnabled = false;
      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();

      mockStateValues.loggedInEnabled = true;
      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();
    });

    it('hides toggle on logout when anonymous access is disabled', () => {
      const {rerender} = renderHeader();
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();

      mockUser = {authenticated: false};
      mockStateValues.anonymousEnabled = false;

      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);

      expect(screen.queryByText('Switch to Astrum One UI')).not.toBeInTheDocument();
    });

    it('keeps toggle visible on logout when anonymous access is enabled', () => {
      const {rerender} = renderHeader();
      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();

      mockUser = {authenticated: false};
      mockStateValues.anonymousEnabled = true;

      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);

      expect(screen.getByText('Switch to Astrum One UI')).toBeInTheDocument();
    });

    it('does not redirect out of preview during flag flaps', () => {
      window.location.hash = 'preview/browse/welcome';

      const {rerender} = renderHeader();

      mockStateValues.loggedInEnabled = false;
      rerender(<Theme><TooltipProvider><GlobalHeaderRadix /></TooltipProvider></Theme>);

      expect(window.location.hash).toBe('#preview/browse/welcome');
    });
  });

  describe('onRefreshClick — Preview UI (ASTRUM-53775)', () => {
    beforeEach(() => {
      jest.spyOn(window, 'confirm');
    });

    it('refreshes without confirm when no unsaved changes', async () => {
      hasUnsavedChanges.mockReturnValue(false);
      renderHeader();

      await userEvent.click(screen.getByRole('button', {name: 'Refresh'}));

      expect(window.confirm).not.toHaveBeenCalled();
      expect(refreshReactPage).toHaveBeenCalled();
    });

    it('prompts and refreshes when user confirms with unsaved changes', async () => {
      hasUnsavedChanges.mockReturnValue(true);
      window.confirm.mockReturnValue(true);
      renderHeader();

      await userEvent.click(screen.getByRole('button', {name: 'Refresh'}));

      expect(window.confirm).toHaveBeenCalled();
      expect(refreshReactPage).toHaveBeenCalled();
    });

    it('does not refresh when user cancels with unsaved changes', async () => {
      hasUnsavedChanges.mockReturnValue(true);
      window.confirm.mockReturnValue(false);
      renderHeader();

      await userEvent.click(screen.getByRole('button', {name: 'Refresh'}));

      expect(window.confirm).toHaveBeenCalled();
      expect(refreshReactPage).not.toHaveBeenCalled();
    });
  });

  describe('shell layout', () => {
    it('opens the side menu expanded until the user chose otherwise', () => {
      renderHeader();

      expect(mockUseSideNavbarOpenState).toHaveBeenCalledWith(true);
      expect(screen.getByRole('button', {name: 'Collapse Sidebar'})).toHaveAttribute('aria-expanded', 'true');
    });

    it('offers to expand the menu when it is collapsed', () => {
      mockUseSideNavbarOpenState.mockReturnValue([false]);

      renderHeader();

      expect(screen.getByRole('button', {name: 'Expand Sidebar'})).toHaveAttribute('aria-expanded', 'false');
    });

    it('puts the theme toggle into Astrum One only', () => {
      const {unmount} = renderHeader();
      expect(screen.queryByTestId('theme-switcher')).not.toBeInTheDocument();
      unmount();

      window.location.hash = 'preview/browse/welcome';
      renderHeader();
      expect(screen.getByTestId('theme-switcher')).toBeInTheDocument();
    });

    it('promotes Astrum One with the brand accent and leaves it with a neutral button', () => {
      const {unmount} = renderHeader();
      expect(screen.getByRole('button', {name: 'Switch to Astrum One UI'})).toHaveAttribute('data-accent-color', 'blue');
      unmount();

      window.location.hash = 'preview/browse/welcome';
      renderHeader();
      expect(screen.getByRole('button', {name: 'Switch to Classic UI'})).toHaveAttribute('data-accent-color', 'gray');
    });

    it('gives every header control the same square style', () => {
      renderHeader();

      expect(screen.getByRole('button', {name: 'Refresh'})).toHaveClass('as-global-header-radix__icon-button');
      expect(screen.getByRole('button', {name: 'Collapse Sidebar'})).toHaveClass('as-global-header-radix__icon-button');
    });
  });
});
