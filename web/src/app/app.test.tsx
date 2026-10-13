import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Activity, Search, Sparkles } from 'lucide-react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../api';
import { dashboardArea } from '../areas/dashboard';
import { settingsArea } from '../areas/settings';
import { SearchGroup, SearchItem } from '../shell/search';
import { ADMIN_STATE, ANONYMOUS_STATE, fakeServer, jsonResponse, renderWithApp } from '../test/render';
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
      label: 'shell.search.open',
      icon: Search,
      group: 'main',
      order: 20,
      need: 'astrum:search:read',
    },
  ],
  settings: [
    { id: 'privileges', to: '/admin/security/privileges', need: 'astrum:privileges:read' },
    {
      id: 'users',
      section: 'repository',
      to: '/admin/security/users',
      title: 'shell.user.account',
      description: 'shell.user.accessTokens',
      icon: Sparkles,
      order: 999,
      need: 'astrum:users:read',
    },
    { id: 'blobstores', to: '/admin/repository/blobstores', need: 'astrum:blobstores:read' },
    {
      id: 'extra',
      section: 'system',
      to: '/admin/system/extra',
      title: 'shell.user.account',
      description: 'shell.user.accessTokens',
      icon: Activity,
      order: 5,
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

  it('lists the help links and names the version and revision', async () => {
    app('/', { ...ADMIN_STATE, revision: 'abc1234' });
    expect(await screen.findByText('Astrum Repository 3.0.0-test')).toBeInTheDocument();
    await userEvent.click(await screen.findByRole('button', { name: 'Help and documentation' }));
    expect(
      await screen.findByText('Version: 3.0.0-test (abc1234)', { selector: '[role="menu"] *' }),
    ).toBeInTheDocument();
    const items = screen.getAllByRole('menuitem').map((item) => item.textContent);
    expect(items).toEqual([
      'Documentation',
      'Release Notes',
      'Knowledge Base',
      'GlobalArt Guides',
      'Community',
      'Issue Tracker',
    ]);
    const documentation = screen.getByRole('menuitem', { name: 'Documentation' });
    expect(documentation).toHaveAttribute('target', '_blank');
    expect(documentation.getAttribute('href')).toMatch(
      /^https:\/\/links\.globalart\.dev\/products\/astrum\/docs\/3\.0\?utm_medium=product&/,
    );
  });

  it('keeps the old header controls in their order', async () => {
    app('/');
    const [header] = await screen.findAllByRole('banner');
    if (!header) {
      throw new Error('no header');
    }
    const names = within(header)
      .getAllByRole('button')
      .map((button) => button.getAttribute('aria-label'));
    expect(names).toEqual([
      'Collapse the menu',
      'Refresh',
      'Switch to the light theme',
      'Language',
      'System status',
      'Help and documentation',
      'Account menu of Administrator User',
    ]);
    expect(within(header).getByRole('button', { name: /Account menu/ })).toHaveTextContent('AD');
    expect(within(header).getByRole('combobox', { name: 'Search components' })).toHaveAttribute(
      'placeholder',
      'Search for components or CVEs...',
    );
  });

  it('opens the health page from the bell once it exists', async () => {
    const health: Area = {
      id: 'health',
      routes: [{ path: 'admin/support/metric-health', page: () => <p>health page</p> }],
      settings: [{ id: 'metric-health', to: '/admin/support/metric-health' }],
    };
    const areas = [...AREAS, health];
    const { router } = renderWithApp(null, {
      path: '/',
      state: ADMIN_STATE,
      areas,
      routes: buildRoutes(areas),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'System status' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/support/metric-health'));
  });

  it('marks the bell when a health check fails', async () => {
    const check = { message: null, error: null, details: null, time: 1, duration: 0, timestamp: '' };
    vi.stubGlobal(
      'fetch',
      fakeServer({
        'GET /service/rest/internal/ui/status-check': () =>
          jsonResponse({ Database: { ...check, healthy: true }, Scheduler: { ...check, healthy: false } }),
      }).fetch,
    );
    app('/');
    expect(
      await screen.findByRole('button', { name: 'System status: 1 health check fails' }),
    ).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toBeInTheDocument();
  });

  it('shows no bell to callers without the metrics permission', async () => {
    app('/', { ...ADMIN_STATE, permissions: ['astrum:search:read'] });
    await screen.findByRole('navigation', { name: 'Main menu' });
    expect(screen.queryByRole('button', { name: 'System status' })).toBeNull();
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
    expect(await screen.findByRole('heading', { name: 'Page Not Found' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Back to the Dashboard' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('link', { name: 'Read the Documentation' })).toHaveAttribute('target', '_blank');
    expect(router.state.location.pathname).toBe('/no/such/page');
    expect(document.title).toBe('Page Not Found - GlobalArt Astrum Repository');
  });

  it('sends section addresses to their first page', async () => {
    const { router } = app('/admin');
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
  });

  it('switches the theme with one click and remembers it', async () => {
    app('/');
    expect(document.documentElement.dataset.theme).toBe('dark');
    await userEvent.click(await screen.findByRole('button', { name: 'Switch to the light theme' }));
    expect(document.documentElement.dataset.theme).toBe('light');
    expect(window.localStorage.getItem('astrum-theme-preference')).toBe('light');
    await userEvent.click(screen.getByRole('button', { name: 'Switch to the dark theme' }));
    expect(document.documentElement.dataset.theme).toBe('dark');
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

  it('focuses the header search with Ctrl+K and moves through the dropdown', async () => {
    const { router } = app('/');
    await screen.findByRole('navigation', { name: 'Main menu' });
    await userEvent.keyboard('{Control>}k{/Control}');
    const input = screen.getByRole('combobox', { name: 'Search components' });
    expect(input).toHaveFocus();
    expect(input).toHaveAttribute('aria-expanded', 'true');
    await userEvent.type(input, 'react');
    const list = screen.getByRole('listbox', { name: 'Search suggestions' });
    expect(within(list).getByRole('group', { name: 'Things' })).toBeInTheDocument();
    await userEvent.keyboard('{ArrowDown}{ArrowDown}');
    const second = within(list).getByRole('option', { name: 'Second' });
    expect(second).toHaveAttribute('aria-selected', 'true');
    expect(input).toHaveAttribute('aria-activedescendant', second.id);
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(router.state.location.pathname).toBe('/user/account'));
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(input).toHaveValue('');
  });

  it('closes the dropdown with Escape and keeps the text', async () => {
    app('/');
    const input = await screen.findByRole('combobox', { name: 'Search components' });
    await userEvent.type(input, 'react');
    expect(screen.getByRole('listbox')).toBeInTheDocument();
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(input).toHaveValue('react');
    await userEvent.click(screen.getByRole('button', { name: 'Clear the search' }));
    expect(input).toHaveValue('');
  });

  it('submits a search when no result is chosen', async () => {
    const { router } = app('/');
    await userEvent.type(await screen.findByRole('combobox', { name: 'Search components' }), 'lodash{Enter}');
    await waitFor(() => expect(router.state.location.search).toBe('?q=lodash'));
  });

  it('turns the phone header into a search bar', async () => {
    setMedia(COMPACT, true);
    const { router } = app('/');
    expect(screen.queryByRole('combobox', { name: 'Search components' })).toBeNull();
    await userEvent.click(await screen.findByRole('button', { name: 'Search' }));
    const layer = await screen.findByRole('dialog', { name: 'Search' });
    const input = within(layer).getByRole('combobox', { name: 'Search components' });
    expect(input).toHaveFocus();
    await userEvent.type(input, 'vue{Enter}');
    await waitFor(() => expect(router.state.location.search).toBe('?q=vue'));
    expect(screen.queryByRole('dialog', { name: 'Search' })).toBeNull();
  });
});

describe('the settings hub', () => {
  it('shows the tiles the caller may open in the old sections and order', async () => {
    app('/settings');
    expect(await screen.findByRole('heading', { level: 1, name: 'Settings' })).toBeInTheDocument();
    expect(screen.getAllByRole('heading', { level: 2 }).map((heading) => heading.textContent)).toEqual([
      'Repository',
      'Security',
      'System',
    ]);
    const security = screen.getByRole('region', { name: 'Security' });
    expect(
      within(security)
        .getAllByRole('link')
        .map((link) => link.getAttribute('href')),
    ).toEqual(['/admin/security/users', '/admin/security/privileges']);
    expect(within(security).getByRole('link', { name: /^Users/ })).toHaveTextContent(
      'Accounts that can sign in, and their roles',
    );
    const repository = screen.getByRole('region', { name: 'Repository' });
    expect(within(repository).getByRole('link', { name: /Blob Stores/ })).toHaveAttribute(
      'href',
      '/admin/repository/blobstores',
    );
    const system = screen.getByRole('region', { name: 'System' });
    expect(within(system).getByRole('link', { name: /My Account/ })).toHaveAttribute(
      'href',
      '/admin/system/extra',
    );
    await userEvent.type(screen.getByRole('searchbox', { name: 'Search settings...' }), 'blob');
    expect(screen.queryByRole('region', { name: 'Security' })).toBeNull();
    expect(screen.getByRole('region', { name: 'Repository' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Search settings...' }), 'zzz');
    expect(screen.getByText('No settings match “blobzzz”')).toBeInTheDocument();
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
