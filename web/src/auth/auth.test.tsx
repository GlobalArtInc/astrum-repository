import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { request, type UiState } from '../api';
import { buildRoutes } from '../app/router';
import { dashboardArea } from '../areas/dashboard';
import { settingsArea } from '../areas/settings';
import { GITHUB_STARS } from '../shell/GithubLink';
import { ADMIN_STATE, ANONYMOUS_STATE, fakeServer, jsonResponse, renderWithApp } from '../test/render';
import { openFresh } from './browser';
import { STATE_KEY } from './state';

vi.mock('./browser', () => ({ openFresh: vi.fn() }));

const AREAS = [dashboardArea, settingsArea];
const STRANGER: UiState = { ...ANONYMOUS_STATE, anonymousAccess: false, permissions: [] };

function app(path: string, state: UiState) {
  return renderWithApp(null, { path, state, areas: AREAS, routes: buildRoutes(AREAS) });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the login page', () => {
  it('says where the first admin password is and needs both fields', async () => {
    app('/login', { ...STRANGER, adminPasswordFile: '/astrum-data/admin.password' });
    expect(await screen.findByRole('heading', { name: 'Welcome' })).toBeInTheDocument();
    expect(screen.getByText('/astrum-data/admin.password')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Continue without signing in' })).toBeNull();
    expect(document.title).toBe('Sign in - Astrum Repository');
    const submit = screen.getByRole('button', { name: 'Sign in' });
    expect(screen.getByLabelText('User name')).toHaveAttribute('placeholder', 'User name');
    expect(screen.getByLabelText('Password')).toHaveAttribute('placeholder', 'Password');
    expect(screen.getByLabelText('Password')).toHaveAttribute('type', 'password');
    expect(screen.queryByRole('button', { name: 'Show' })).toBeNull();
    expect(submit).toBeDisabled();
    await userEvent.type(screen.getByLabelText('User name'), 'admin');
    expect(submit).toBeDisabled();
    await userEvent.type(screen.getByLabelText('Password'), 'x');
    expect(submit).toBeEnabled();
  });

  it('shows a wrong password', async () => {
    const server = fakeServer({
      'POST /service/rest/internal/session': () =>
        jsonResponse({ id: '*', message: 'the user name or password is wrong' }, 401),
    });
    vi.stubGlobal('fetch', server.fetch);
    app('/login', STRANGER);
    await userEvent.type(await screen.findByLabelText('User name'), 'admin');
    await userEvent.type(screen.getByLabelText('Password'), 'nope');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('The user name or password is wrong');
    expect(server.calls[0]?.body).toEqual({ username: 'admin', password: 'nope' });
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.getByLabelText('Password')).toHaveValue('');
    expect(screen.getByLabelText('Password')).toHaveAccessibleDescription(
      'The user name or password is wrong',
    );
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeDisabled();
    expect(screen.getByLabelText('User name')).toHaveValue('admin');
  });

  it('signs in and returns to the page that asked', async () => {
    const server = fakeServer({
      'POST /service/rest/internal/session': () => jsonResponse({ authenticated: true }),
      'GET /service/rest/internal/ui/state': () => jsonResponse(ADMIN_STATE),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = app('/login?returnTo=%2Fsettings', STRANGER);
    await userEvent.type(await screen.findByLabelText('User name'), ' admin ');
    await userEvent.type(screen.getByLabelText('Password'), 'secret');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
    expect(server.calls[0]?.body).toEqual({ username: 'admin', password: 'secret' });
  });

  it('lets callers go on without signing in when anonymous access is on', async () => {
    app('/login?returnTo=%2Fbrowse', ANONYMOUS_STATE);
    expect(await screen.findByRole('link', { name: 'Continue without signing in' })).toHaveAttribute(
      'href',
      '/browse',
    );
  });

  it('sends signed-in users on', async () => {
    const { router } = app('/login?returnTo=%2Fsettings', ADMIN_STATE);
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
  });
});

describe('signing out', () => {
  it('ends the session and starts afresh on the dashboard', async () => {
    const server = fakeServer({
      'DELETE /service/rest/internal/session': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    app('/settings', ADMIN_STATE);
    await userEvent.click(await screen.findByRole('button', { name: 'Account menu of Administrator User' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Sign Out' }));
    await waitFor(() => expect(openFresh).toHaveBeenCalledWith('/'));
    expect(
      server.calls.filter((call) => call.path !== GITHUB_STARS).map((call) => `${call.method} ${call.path}`),
    ).toEqual(['DELETE /service/rest/internal/session']);
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('says when signing out failed', async () => {
    const server = fakeServer({
      'DELETE /service/rest/internal/session': () => jsonResponse({ id: '*', message: 'down' }, 503),
    });
    vi.stubGlobal('fetch', server.fetch);
    app('/', ADMIN_STATE);
    await userEvent.click(await screen.findByRole('button', { name: 'Account menu of Administrator User' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Sign Out' }));
    expect(await screen.findByText('Could not sign out')).toBeInTheDocument();
  });
});

describe('an ended session', () => {
  it('raises a dialog on a 401 and leads to the login page', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/thing': () =>
        jsonResponse({ id: '*', message: 'authentication is required' }, 401),
      'GET /service/rest/internal/ui/state': () => jsonResponse(ANONYMOUS_STATE),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = app('/?tab=x', ADMIN_STATE);
    await screen.findByRole('heading', { level: 1, name: 'Overview' });
    await act(() => request('/service/rest/v1/thing').catch(() => undefined));
    const dialog = await screen.findByRole('alertdialog', { name: 'Your session has ended' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/login'));
    expect(router.state.location.search).toBe('?returnTo=%2F%3Ftab%3Dx');
  });

  it('raises the dialog when the server no longer knows the user', async () => {
    const { client } = app('/', ADMIN_STATE);
    await screen.findByRole('navigation', { name: 'Main menu' });
    act(() => client.setQueryData(STATE_KEY, ANONYMOUS_STATE));
    expect(await screen.findByRole('alertdialog', { name: 'Your session has ended' })).toBeInTheDocument();
  });

  it('asks anonymous callers to sign in', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/thing': () =>
        jsonResponse({ id: '*', message: 'authentication is required' }, 401),
      'GET /service/rest/internal/ui/state': () => jsonResponse(ANONYMOUS_STATE),
    });
    vi.stubGlobal('fetch', server.fetch);
    app('/', ANONYMOUS_STATE);
    await screen.findByRole('navigation', { name: 'Main menu' });
    await act(() => request('/service/rest/v1/thing').catch(() => undefined));
    expect(await screen.findByRole('alertdialog', { name: 'Sign in to continue' })).toBeInTheDocument();
  });
});
