import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { KeyRound, Search, Users } from 'lucide-react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../api';
import { dashboardArea } from '../areas/dashboard';
import { settingsArea } from '../areas/settings';
import { SearchGroup, SearchItem } from '../shell/search';
import { ADMIN_STATE, ANONYMOUS_STATE, renderWithApp } from '../test/render';
import { setMedia } from '../test/setup';
import { COMPACT } from '../ui/hooks';
import type { Area, SearchResultsProps } from './area';
import { buildRoutes } from './router';

function Results({ query }: SearchResultsProps) {
  if (!query) {
    return null;
  }
  return (
    <SearchGroup title="Things">
      <SearchItem to={`/search?q=${query}`}>Search for {query}</SearchItem>
      <SearchItem to="/user/account">Second</SearchItem>
    </SearchGroup>
  );
}

const testArea: Area = {
  id: 'test',
  routes: [
    { path: 'search', page: () => <p>search page</p>, need: 'astrum:search:read' },
    { path: 'user/account', page: () => <p>account page</p>, requiresUser: true },
    { path: 'admin/security/users', page: () => <p>users page</p>, need: 'astrum:users:read' },
  ],
  menu: [
    {
      id: 'search',
      to: '/search',
      label: 'shell.search.label',
      icon: Search,
      group: 'main',
      order: 20,
      need: 'astrum:search:read',
    },
  ],
  settings: [
    {
      id: 'users',
      section: 'security',
      to: '/admin/security/users',
      title: 'shell.user.account',
      description: 'shell.user.accessTokens',
      icon: Users,
      order: 10,
      need: 'astrum:users:read',
    },
    {
      id: 'stores',
      section: 'repository',
      to: '/admin/repository/blobstores',
      title: 'shell.theme.label',
      description: 'shell.theme.dark',
      icon: KeyRound,
      order: 10,
      need: 'astrum:blobstores:read',
    },
  ],
  search: [{ id: 'things', order: 10, Results, submit: (query) => `/search?q=${query}` }],
};

const AREAS = [dashboardArea, settingsArea, testArea];

function app(path: string, state: UiState = ADMIN_STATE) {
  return renderWithApp(null, { path, state, areas: AREAS, routes: buildRoutes(AREAS) });
}

function menu() {
  return screen.getByRole('navigation', { name: 'Main menu' });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the shell', () => {
  it('shows the menu entries the caller may use', async () => {
    app('/');
    expect(await within(menu()).findByRole('link', { name: 'Dashboard' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    expect(within(menu()).getByRole('link', { name: 'Search' })).toBeInTheDocument();
    expect(within(menu()).getByRole('link', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByText('Astrum Repository 3.0.0-test')).toBeInTheDocument();
  });

  it('hides what anonymous callers cannot use and offers to sign in', async () => {
    app('/', { ...ANONYMOUS_STATE, permissions: [] });
    expect(await within(menu()).findByRole('link', { name: 'Dashboard' })).toBeInTheDocument();
    expect(within(menu()).queryByRole('link', { name: 'Search' })).toBeNull();
    expect(within(menu()).queryByRole('link', { name: 'Settings' })).toBeNull();
    expect(screen.getByRole('link', { name: 'Sign in' })).toHaveAttribute('href', '/login');
  });

  it('sends strangers to the login page when anonymous access is off', async () => {
    const { router } = app('/settings?x=1', { ...ANONYMOUS_STATE, anonymousAccess: false });
    await waitFor(() => expect(router.state.location.pathname).toBe('/login'));
    expect(router.state.location.search).toBe('?returnTo=%2Fsettings%3Fx%3D1');
    expect(document.documentElement.dataset.theme).toBe('light');
  });

  it('asks anonymous callers to sign in for pages that need a user', async () => {
    const { router } = app('/user/account', ANONYMOUS_STATE);
    await waitFor(() => expect(router.state.location.pathname).toBe('/login'));
    expect(router.state.location.search).toBe('?returnTo=%2Fuser%2Faccount');
  });

  it('tells signed-in users when a page is not theirs', async () => {
    app('/admin/security/users', { ...ADMIN_STATE, permissions: ['astrum:search:read'] });
    expect(await screen.findByRole('heading', { name: 'You cannot open this page' })).toBeInTheDocument();
  });

  it('keeps the address of a missing page', async () => {
    const { router } = app('/no/such/page?x=1');
    expect(await screen.findByRole('heading', { name: 'Page not found' })).toBeInTheDocument();
    expect(screen.getByText('/no/such/page?x=1')).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/no/such/page');
    expect(document.title).toBe('Page not found - Astrum Repository');
  });

  it('sends section addresses to their first page', async () => {
    const { router } = app('/admin');
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
  });

  it('switches the theme and remembers it', async () => {
    app('/');
    expect(document.documentElement.dataset.theme).toBe('dark');
    await userEvent.click(await screen.findByRole('button', { name: 'Theme' }));
    await userEvent.click(await screen.findByRole('menuitemradio', { name: 'Light' }));
    expect(document.documentElement.dataset.theme).toBe('light');
    expect(window.localStorage.getItem('astrum-theme-preference')).toBe('light');
  });

  it('switches the language and keeps it in a cookie', async () => {
    app('/');
    await userEvent.click(await screen.findByRole('button', { name: 'Language' }));
    await userEvent.click(await screen.findByRole('menuitemradio', { name: 'Русский' }));
    const russian = await screen.findByRole('navigation', { name: 'Основное меню' });
    expect(within(russian).getByRole('link', { name: 'Обзор' })).toBeInTheDocument();
    expect(document.cookie).toContain('astrum-language=ru');
    expect(document.documentElement.lang).toBe('ru');
  });

  it('collapses the menu and remembers it', async () => {
    app('/');
    await userEvent.click(await screen.findByRole('button', { name: 'Collapse the menu' }));
    expect(window.localStorage.getItem('astrum.sidebar.open')).toBe('false');
    expect(within(menu()).getByRole('link', { name: 'Dashboard' })).not.toHaveTextContent('Dashboard');
    expect(screen.getByRole('button', { name: 'Expand the menu' })).toHaveAttribute('aria-expanded', 'false');
  });

  it('opens the menu as a drawer on compact screens', async () => {
    setMedia(COMPACT, true);
    const { router } = app('/');
    await userEvent.click(await screen.findByRole('button', { name: 'Open the menu' }));
    const drawer = await screen.findByRole('dialog', { name: 'Main menu' });
    await userEvent.click(within(drawer).getByRole('link', { name: 'Search' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/search'));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('opens the search panel with Ctrl+K and moves through results', async () => {
    const { router } = app('/');
    await screen.findByRole('navigation', { name: 'Main menu' });
    await userEvent.keyboard('{Control>}k{/Control}');
    const panel = await screen.findByRole('dialog', { name: 'Search' });
    const input = within(panel).getByRole('combobox', { name: 'Search' });
    expect(input).toHaveFocus();
    await userEvent.type(input, 'react');
    expect(within(panel).getByRole('group', { name: 'Things' })).toBeInTheDocument();
    await userEvent.keyboard('{ArrowDown}{ArrowDown}');
    expect(within(panel).getByRole('option', { name: 'Second' })).toHaveAttribute('aria-selected', 'true');
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(router.state.location.pathname).toBe('/user/account'));
    expect(screen.queryByRole('dialog', { name: 'Search' })).toBeNull();
  });

  it('submits a search when no result is chosen', async () => {
    const { router } = app('/');
    await userEvent.click(await screen.findByRole('button', { name: /Search components/ }));
    await userEvent.type(await screen.findByRole('combobox', { name: 'Search' }), 'lodash{Enter}');
    await waitFor(() => expect(router.state.location.search).toBe('?q=lodash'));
  });
});

describe('the settings hub', () => {
  it('shows the tiles the caller may open, by section', async () => {
    app('/settings');
    expect(await screen.findByRole('heading', { level: 1, name: 'Settings' })).toBeInTheDocument();
    const security = screen.getByRole('region', { name: 'Security' });
    expect(within(security).getByRole('link', { name: /My account/ })).toHaveAttribute(
      'href',
      '/admin/security/users',
    );
    expect(screen.getByRole('region', { name: 'Repository' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Find a setting' }), 'theme');
    expect(screen.queryByRole('region', { name: 'Security' })).toBeNull();
    expect(screen.getByRole('region', { name: 'Repository' })).toBeInTheDocument();
  });

  it('hides sections without visible tiles', async () => {
    app('/settings', { ...ADMIN_STATE, permissions: ['astrum:users:read'] });
    expect(await screen.findByRole('region', { name: 'Security' })).toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'Repository' })).toBeNull();
  });

  it('says so when nothing can be changed', async () => {
    app('/settings', { ...ADMIN_STATE, permissions: [] });
    expect(await screen.findByText('There is nothing here you can change')).toBeInTheDocument();
    expect(within(menu()).queryByRole('link', { name: 'Settings' })).toBeNull();
  });
});

describe('old addresses', () => {
  it('follow a hash change to the new path', async () => {
    const { router } = app('/');
    await screen.findByRole('navigation', { name: 'Main menu' });
    act(() => {
      window.location.hash = '#admin/security/users';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/users'));
    window.location.hash = '';
  });
});

describe('a server without settings areas', () => {
  it('says that there are no settings yet', async () => {
    const areas = [dashboardArea, settingsArea];
    renderWithApp(null, { path: '/settings', state: ADMIN_STATE, areas, routes: buildRoutes(areas) });
    expect(await screen.findByText('No settings are available on this server yet')).toBeInTheDocument();
  });
});
