import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../../api';
import { buildRoutes } from '../../app/router';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { dashboardArea } from '../dashboard';
import { settingsArea } from '../settings';
import { onboardingArea } from '.';

const AREAS = [dashboardArea, settingsArea, onboardingArea];
const FRESH: UiState = { ...ADMIN_STATE, anonymousAccess: false, adminPasswordFile: '/data/admin.password' };

afterEach(() => {
  vi.unstubAllGlobals();
});

function app(path: string, state: UiState) {
  return renderWithApp(null, { path, state, areas: AREAS, routes: buildRoutes(AREAS) });
}

describe('onboarding', () => {
  it('opens once per session for administrators while the password file exists', async () => {
    document.cookie = 'astrum_csrf=session-1';
    const { router } = app('/settings', FRESH);
    await waitFor(() => expect(router.state.location.pathname).toBe('/onboarding'));
    expect(router.state.location.search).toBe('?returnTo=%2Fsettings');
    expect(
      await screen.findByRole('heading', { name: 'Set up Astrum Repository' }, { timeout: 5000 }),
    ).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Skip for now' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
    await router.navigate('/');
    expect(router.state.location.pathname).toBe('/');
  });

  it('stays away from users who are not administrators or when the file is gone', async () => {
    document.cookie = 'astrum_csrf=session-2';
    const { router } = app('/', { ...FRESH, permissions: ['astrum:settings:read'] });
    await screen.findByRole('heading', { level: 1 });
    expect(router.state.location.pathname).toBe('/');
    const other = app('/', { ...FRESH, adminPasswordFile: null });
    await new Promise((done) => setTimeout(done, 50));
    expect(other.router.state.location.pathname).toBe('/');
  });

  it('changes the password, sets anonymous access and finishes', async () => {
    document.cookie = 'astrum_csrf=session-3';
    const server = fakeServer({
      'PUT /service/rest/internal/ui/user/password': () => new Response(null, { status: 204 }),
      'GET /service/rest/v1/security/anonymous': () =>
        jsonResponse({ enabled: true, userId: 'anonymous', realmName: 'AstrumAuthorizingRealm' }),
      'PUT /service/rest/v1/security/anonymous': ({ body }) => jsonResponse(body),
      'GET /service/rest/internal/ui/state': () => jsonResponse({ ...FRESH, adminPasswordFile: null }),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = app('/onboarding', FRESH);
    await userEvent.type(
      await screen.findByLabelText(/Current password/, {}, { timeout: 5000 }),
      'first-password',
    );
    await userEvent.type(screen.getByLabelText(/^New password/), 'first-password');
    await userEvent.type(screen.getByLabelText(/Repeat the new password/), 'first-password');
    await userEvent.click(screen.getByRole('button', { name: 'Change password' }));
    expect(screen.getByText('Choose a password other than the current one')).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText(/^New password/));
    await userEvent.type(screen.getByLabelText(/^New password/), 'a new secret');
    await userEvent.clear(screen.getByLabelText(/Repeat the new password/));
    await userEvent.type(screen.getByLabelText(/Repeat the new password/), 'a new secret');
    await userEvent.click(screen.getByRole('button', { name: 'Change password' }));
    expect(await screen.findByRole('heading', { name: 'Anonymous access' })).toBeInTheDocument();
    expect(server.calls[0]?.body).toEqual({ currentPassword: 'first-password', newPassword: 'a new secret' });
    await userEvent.click(screen.getByRole('radio', { name: /Require sign-in/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(await screen.findByRole('heading', { name: 'All set' })).toBeInTheDocument();
    expect(
      server.calls.find((call) => call.method === 'PUT' && call.path.endsWith('/anonymous'))?.body,
    ).toEqual({
      enabled: false,
      userId: 'anonymous',
      realmName: 'AstrumAuthorizingRealm',
    });
    await userEvent.click(screen.getByRole('button', { name: 'Go to the dashboard' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/'));
  }, 20_000);
});
