import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../api';
import { buildRoutes } from '../app/router';
import { settingsArea } from '../areas/settings';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../test/render';
import { Guard } from './Guard';
import type { SessionState } from './state';

const AREAS = [settingsArea];
const SESSION = '/service/rest/internal/session';
const SECOND = '/service/rest/internal/session/second-factor';
const STATE = '/service/rest/internal/ui/state';
const STRANGER: UiState = { ...ADMIN_STATE, user: null, anonymousAccess: false, permissions: [] };

function challenge(methods: string[]) {
  return jsonResponse({
    authenticated: false,
    anonymous: false,
    user: null,
    permissions: [],
    secondFactor: { methods, expiresAt: '2026-11-11T10:05:00Z' },
  });
}

function reply(body: unknown, status: number, headers: Record<string, string> = {}) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json', ...headers },
  });
}

async function atCodeStep(methods: string[], extra: Parameters<typeof fakeServer>[0] = {}) {
  const server = fakeServer({
    [`POST ${SESSION}`]: () => challenge(methods),
    [`GET ${STATE}`]: () => jsonResponse(ADMIN_STATE),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  const result = renderWithApp(null, {
    path: '/login?returnTo=%2Fsettings',
    state: STRANGER,
    areas: AREAS,
    routes: buildRoutes(AREAS),
  });
  await userEvent.type(await screen.findByLabelText('User name'), 'admin');
  await userEvent.type(screen.getByLabelText('Password'), 'secret');
  await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
  await screen.findByRole('heading', { name: 'Two-factor authentication' });
  return { ...result, server };
}

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('the second step of the login page', () => {
  it('replaces the password form by the code step when the answer names methods', async () => {
    await atCodeStep(['totp', 'recovery']);
    expect(screen.getByLabelText('Authentication code')).toHaveAttribute('inputmode', 'numeric');
    expect(screen.queryByLabelText('Password')).toBeNull();
    expect(screen.getByRole('button', { name: 'Use a recovery code instead' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Back to password' })).toBeInTheDocument();
  });

  it('posts a TOTP code without its spaces and continues like a password sign-in', async () => {
    const { router, server } = await atCodeStep(['totp', 'recovery'], {
      [`POST ${SECOND}`]: () => jsonResponse({ authenticated: true }),
    });
    await userEvent.type(screen.getByLabelText('Authentication code'), '123 456');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
    expect(server.calls.find((call) => call.path === SECOND)?.body).toEqual({
      method: 'totp',
      code: '123456',
    });
  });

  it('posts a recovery code in recovery mode', async () => {
    const { server } = await atCodeStep(['totp', 'recovery'], {
      [`POST ${SECOND}`]: () => jsonResponse({ authenticated: true }),
    });
    await userEvent.click(screen.getByRole('button', { name: 'Use a recovery code instead' }));
    await userEvent.type(screen.getByLabelText('Recovery code'), 'aaaaa-bbbbb');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() =>
      expect(server.calls.find((call) => call.path === SECOND)?.body).toEqual({
        method: 'recovery',
        code: 'aaaaa-bbbbb',
      }),
    );
  });

  it('shows a wrong code under the input and keeps the step', async () => {
    await atCodeStep(['totp'], {
      [`POST ${SECOND}`]: () =>
        reply(
          { id: '*', message: 'the code is wrong or was already used', code: 'second-factor-invalid' },
          401,
        ),
    });
    await userEvent.type(screen.getByLabelText('Authentication code'), '000000');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('The code is wrong or was already used.');
    expect(screen.getByRole('heading', { name: 'Two-factor authentication' })).toBeInTheDocument();
    expect(screen.getByLabelText('Authentication code')).toHaveValue('');
  });

  it('returns an expired sign-in to the password form with a notice', async () => {
    await atCodeStep(['totp'], {
      [`POST ${SECOND}`]: () =>
        reply(
          { id: '*', message: 'the sign-in expired; start again with the password', code: 'sign-in-expired' },
          401,
        ),
    });
    await userEvent.type(screen.getByLabelText('Authentication code'), '123456');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(await screen.findByLabelText('Password')).toHaveValue('');
    expect(screen.getByText('The sign-in expired. Start again with your password.')).toBeInTheDocument();
  });

  it('starts in recovery mode when only recovery codes are offered', async () => {
    await atCodeStep(['recovery']);
    expect(screen.getByLabelText('Recovery code')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Use the authenticator app' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Use a recovery code instead' })).toBeNull();
  });

  it('counts down a second-step lock from Retry-After and re-enables the button at zero', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    await atCodeStep(['totp'], {
      [`POST ${SECOND}`]: () =>
        reply({ id: '*', message: 'too many failed sign-ins; wait a few minutes' }, 429, {
          'Retry-After': '3',
        }),
    });
    await userEvent.type(screen.getByLabelText('Authentication code'), '123456');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(await screen.findByText('Too many attempts. Try again in 3 seconds.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeDisabled();
    await act(() => vi.advanceTimersByTimeAsync(1000));
    expect(screen.getByText('Too many attempts. Try again in 2 seconds.')).toBeInTheDocument();
    await act(() => vi.advanceTimersByTimeAsync(2000));
    await userEvent.type(screen.getByLabelText('Authentication code'), '123456');
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeEnabled();
  });

  it('keeps the password button off for fifteen seconds after a throttle without Retry-After', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const server = fakeServer({
      [`POST ${SESSION}`]: () =>
        reply({ id: '*', message: 'too many failed sign-ins; wait a few minutes' }, 429),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, { path: '/login', state: STRANGER, areas: AREAS, routes: buildRoutes(AREAS) });
    await userEvent.type(await screen.findByLabelText('User name'), 'admin');
    await userEvent.type(screen.getByLabelText('Password'), 'secret');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Too many failed attempts.');
    await userEvent.type(screen.getByLabelText('Password'), 'secret');
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeDisabled();
    await act(() => vi.advanceTimersByTimeAsync(10_000));
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeDisabled();
    await act(() => vi.advanceTimersByTimeAsync(5_000));
    expect(screen.getByRole('button', { name: 'Sign in' })).toBeEnabled();
  });

  it('takes a restricted session after the second step to the Security page', async () => {
    const restricted = { ...ADMIN_STATE, restricted: true, permissions: [] };
    const { router } = await atCodeStep(['totp'], {
      [`POST ${SECOND}`]: () => jsonResponse({ authenticated: true }),
      [`GET ${STATE}`]: () => jsonResponse(restricted),
    });
    await userEvent.type(screen.getByLabelText('Authentication code'), '123456');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/user/security'));
  });
});

describe('a restricted session', () => {
  const restricted: SessionState = { ...ADMIN_STATE, restricted: true, permissions: [] };
  const routes = [
    {
      path: 'settings',
      element: (
        <Guard>
          <p>Settings page</p>
        </Guard>
      ),
    },
    {
      path: 'user/security',
      element: (
        <Guard>
          <p>Security page</p>
        </Guard>
      ),
    },
  ];

  it('is sent from any page to the Security page', async () => {
    const { router } = renderWithApp(null, { state: restricted, path: '/settings', routes });
    await waitFor(() => expect(router.state.location.pathname).toBe('/user/security'));
    expect(screen.getByText('Security page')).toBeInTheDocument();
  });

  it('stays on the Security page', async () => {
    const { router } = renderWithApp(null, { state: restricted, path: '/user/security', routes });
    expect(await screen.findByText('Security page')).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/user/security');
  });
});

describe('the passkey option of the second step', () => {
  const OPTIONS = '/service/rest/internal/session/second-factor/passkey-options';
  const challenge = {
    ceremony: 's1',
    publicKey: { challenge: 'AQI', allowCredentials: [{ id: 'Bgc', type: 'public-key' }] },
  };
  const assertion = {
    id: 'AQI',
    rawId: new Uint8Array([1, 2]).buffer,
    type: 'public-key',
    response: {
      authenticatorData: new Uint8Array([7]).buffer,
      clientDataJSON: new Uint8Array([8]).buffer,
      signature: new Uint8Array([9, 10]).buffer,
      userHandle: null,
    },
    getClientExtensionResults: () => ({}),
  };

  function supportPasskeys(get: () => Promise<unknown>) {
    vi.stubGlobal('PublicKeyCredential', class {});
    Object.defineProperty(navigator, 'credentials', {
      configurable: true,
      value: { create: vi.fn(), get: vi.fn(get) },
    });
  }

  afterEach(() => {
    Reflect.deleteProperty(navigator, 'credentials');
  });

  it('offers the button only with a passkey among the methods and WebAuthn in the browser', async () => {
    supportPasskeys(async () => assertion);
    await atCodeStep(['totp', 'passkey']);
    expect(screen.getByRole('button', { name: 'Use a passkey' })).toBeEnabled();
  });

  it('leaves the button out when the challenge names no passkey', async () => {
    supportPasskeys(async () => assertion);
    await atCodeStep(['totp']);
    expect(screen.queryByRole('button', { name: 'Use a passkey' })).toBeNull();
  });

  it('leaves the button out when the browser has no WebAuthn', async () => {
    await atCodeStep(['totp', 'passkey']);
    expect(screen.queryByRole('button', { name: 'Use a passkey' })).toBeNull();
  });

  it('posts the assertion with its ceremony and continues like a password sign-in', async () => {
    supportPasskeys(async () => assertion);
    const { router, server } = await atCodeStep(['totp', 'passkey'], {
      [`POST ${OPTIONS}`]: () => jsonResponse(challenge),
      [`POST ${SECOND}`]: () => jsonResponse({ authenticated: true }),
    });
    await userEvent.click(screen.getByRole('button', { name: 'Use a passkey' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/settings'));
    expect(server.calls.find((call) => call.path === SECOND)?.body).toEqual({
      method: 'passkey',
      ceremony: 's1',
      credential: {
        id: 'AQI',
        rawId: 'AQI',
        type: 'public-key',
        response: { authenticatorData: 'Bw', clientDataJSON: 'CA', signature: 'CQo', userHandle: null },
        clientExtensionResults: {},
      },
    });
  });

  it('shows a cancelled prompt inline and keeps the step', async () => {
    supportPasskeys(() => Promise.reject(new DOMException('refused', 'NotAllowedError')));
    await atCodeStep(['totp', 'passkey'], {
      [`POST ${OPTIONS}`]: () => jsonResponse(challenge),
    });
    await userEvent.click(screen.getByRole('button', { name: 'Use a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The passkey prompt was closed. Try again or enter a code.',
    );
    expect(screen.getByRole('heading', { name: 'Two-factor authentication' })).toBeInTheDocument();
  });

  it('shows a refused passkey inline and keeps the step', async () => {
    supportPasskeys(async () => assertion);
    await atCodeStep(['totp', 'passkey'], {
      [`POST ${OPTIONS}`]: () => jsonResponse(challenge),
      [`POST ${SECOND}`]: () =>
        reply(
          {
            id: '*',
            message: 'the passkey was not accepted or was already used',
            code: 'second-factor-invalid',
          },
          401,
        ),
    });
    await userEvent.click(screen.getByRole('button', { name: 'Use a passkey' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The passkey was not accepted or was already used.',
    );
    expect(screen.getByRole('heading', { name: 'Two-factor authentication' })).toBeInTheDocument();
  });
});
