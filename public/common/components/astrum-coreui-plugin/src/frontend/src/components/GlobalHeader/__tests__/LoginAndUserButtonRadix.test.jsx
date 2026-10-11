import React from 'react';
import {render, screen, fireEvent} from '@testing-library/react';
import {Theme} from '@radix-ui/themes';
import '@testing-library/jest-dom';

const mockGo = jest.fn();
const mockSignOut = jest.fn();
const mockSetTheme = jest.fn();
let mockHasUser = true;
let mockUser = {id: 'admin', authenticated: true};

jest.mock('@globalart/astrum-ui-plugin', () => ({
  Tooltip: ({children}) => children,
  usePortalContainer: () => undefined,
  useIsVisible: () => true,
  ExtJS: {
    useUser: () => mockUser,
    state: () => ({getValue: (key) => (key === 'user' ? mockUser : undefined)}),
    signOut: (...args) => mockSignOut(...args),
  },
}));

jest.mock('@uirouter/react', () => ({
  useRouter: () => ({
    stateService: {go: mockGo},
    stateRegistry: {get: () => ({data: {visibilityRequirements: {}}})},
    urlService: {url: () => 'browse/welcome'},
  }),
  useSref: (name) => ({href: `#${name}`}),
}));

jest.mock('../../../hooks/useHasUser', () => ({
  __esModule: true,
  default: () => mockHasUser,
}));

jest.mock('../../../contexts/ThemeContext', () => ({
  useTheme: () => ({effectiveTheme: 'dark', setTheme: mockSetTheme}),
}));

jest.mock('../../../utils/api', () => ({restClient: {}}));

import LoginAndUserButtonRadix, {getInitials} from '../LoginAndUserButtonRadix';

// Radix opens its menus on pointer events jsdom lacks, so the menu is opened from the keyboard
function openMenu() {
  fireEvent.keyDown(screen.getByRole('button', {name: 'My Account'}), {key: 'Enter'});
}

function renderButton() {
  return render(
    <Theme>
      <LoginAndUserButtonRadix />
    </Theme>
  );
}

describe('getInitials', () => {
  it('takes the first two letters of a single-word name', () => {
    expect(getInitials('admin')).toBe('AD');
  });

  it('takes the first letters of the first two words', () => {
    expect(getInitials('jane.doe')).toBe('JD');
    expect(getInitials('John Smith')).toBe('JS');
    expect(getInitials('ci_deploy-bot')).toBe('CD');
  });

  it('keeps letters of any script', () => {
    expect(getInitials('иван.петров')).toBe('ИП');
  });

  it('copes with a one-letter name and with no name', () => {
    expect(getInitials('x')).toBe('X');
    expect(getInitials('')).toBe('');
    expect(getInitials(null)).toBe('');
    expect(getInitials('---')).toBe('');
  });
});

describe('LoginAndUserButtonRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockHasUser = true;
    mockUser = {id: 'admin', authenticated: true};
    window.location.hash = '#preview/browse/welcome';
  });

  it('shows the signed-in user as a round avatar with initials', () => {
    renderButton();

    const avatar = screen.getByRole('button', {name: 'My Account'});
    expect(avatar).toHaveClass('as-login-and-user-button__avatar');
    expect(avatar).toHaveTextContent('AD');
  });

  it('derives the initials from the user name', () => {
    mockUser = {id: 'jane.doe', authenticated: true};

    renderButton();

    expect(screen.getByRole('button', {name: 'My Account'})).toHaveTextContent('JD');
  });

  it('opens the account menu with the user, the account pages and sign out', async () => {
    renderButton();

    openMenu();

    expect(await screen.findByRole('menu')).toBeInTheDocument();
    expect(screen.getByText('admin')).toBeInTheDocument();
    expect(screen.getByRole('menuitem', {name: /My Account/})).toHaveAttribute('href', '#preview.user.account');
    expect(screen.getByTestId('user-menu-access-tokens')).toHaveAttribute('href', '#preview.user.accesstokens');
    expect(screen.getByRole('menuitem', {name: /Log Out/})).toBeInTheDocument();
  });

  it('signs out and returns to the light theme', async () => {
    renderButton();

    openMenu();
    fireEvent.click(await screen.findByRole('menuitem', {name: /Log Out/}));

    expect(mockSetTheme).toHaveBeenCalledWith('light');
    expect(mockSignOut).toHaveBeenCalled();
  });

  it('offers to sign in when nobody is signed in', () => {
    mockHasUser = false;
    mockUser = undefined;

    renderButton();

    const signIn = screen.getByRole('button', {name: 'Log In'});
    expect(screen.queryByTestId('user-menu')).not.toBeInTheDocument();

    fireEvent.click(signIn);

    expect(mockGo).toHaveBeenCalledWith('login', {returnTo: btoa('#browse/welcome')});
  });
});
