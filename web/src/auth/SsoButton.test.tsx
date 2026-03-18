import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../api';
import { buildRoutes } from '../app/router';
import { dashboardArea } from '../areas/dashboard';
import { ANONYMOUS_STATE, renderWithApp } from '../test/render';
import { ssoHref } from './SsoButton';

const STRANGER: UiState = { ...ANONYMOUS_STATE, anonymousAccess: false, permissions: [] };

function app(path: string, state: UiState) {
  return renderWithApp(null, { path, state, areas: [dashboardArea], routes: buildRoutes([dashboardArea]) });
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('signing in with SSO', () => {
  it('hides the button while OpenID Connect is off', async () => {
    app('/login', STRANGER);
    expect(await screen.findByRole('heading', { name: 'Welcome' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Continue with SSO' })).toBeNull();
  });

  it('sends the browser to the provider with the page to return to', async () => {
    const assign = vi.fn();
    vi.spyOn(window, 'location', 'get').mockReturnValue({ ...window.location, assign });
    app('/login?returnTo=%2Fbrowse%2Fmaven', { ...STRANGER, oidcEnabled: true });
    expect(await screen.findByText('or')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Continue with SSO' }));
    expect(assign).toHaveBeenCalledWith(ssoHref('/browse/maven'));
    expect(ssoHref('/browse/maven')).toBe(`/oidc/login?hash=${encodeURIComponent(btoa('/browse/maven'))}`);
  });

  it('explains why the last SSO attempt failed', async () => {
    app('/login?sso=expired', { ...STRANGER, oidcEnabled: true });
    expect(await screen.findByRole('alert')).toHaveTextContent('The sign-in took too long. Try again.');
  });

  it('speaks Russian', async () => {
    renderWithApp(null, {
      path: '/login',
      state: { ...STRANGER, oidcEnabled: true },
      areas: [dashboardArea],
      routes: buildRoutes([dashboardArea]),
      language: 'ru',
    });
    expect(await screen.findByRole('button', { name: 'Продолжить через SSO' })).toBeInTheDocument();
    expect(screen.getByText('или')).toBeInTheDocument();
  });
});
