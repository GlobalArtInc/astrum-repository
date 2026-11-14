import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import TwoFactorPolicyPage from './TwoFactorPolicyPage';

const ROLES = [
  {
    id: 'dev',
    source: 'default',
    name: 'Developers',
    description: '',
    readOnly: false,
    privileges: [],
    roles: [],
  },
  {
    id: 'readers',
    source: 'default',
    name: 'Readers',
    description: '',
    readOnly: false,
    privileges: [],
    roles: [],
  },
];

const POLICY = {
  totp: true,
  passkeys: true,
  required: { everyone: false, administrators: false, roles: ['readers'] },
  graceDays: 7,
  requiredSince: '2026-10-01T00:00:00Z',
  relyingParty: { id: null, origins: [] },
};

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(extra = {}) {
  const server = fakeServer({
    'GET /service/rest/v1/security/two-factor': () => jsonResponse(POLICY),
    'GET /service/rest/v1/security/roles': () => jsonResponse(ROLES),
    'PUT /service/rest/v1/security/two-factor': ({ body }) => jsonResponse(body),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

describe('the two-factor policy page', () => {
  it('loads the policy and saves the whole body with the chosen role ids', async () => {
    const server = serve();
    renderWithApp(<TwoFactorPolicyPage />, { state: ADMIN_STATE });
    expect(await screen.findByRole('checkbox', { name: 'Everyone' })).not.toBeChecked();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Everyone' }));
    await userEvent.click(screen.getByRole('button', { name: /Roles/ }));
    await userEvent.click(await screen.findByRole('option', { name: 'dev' }));
    await userEvent.keyboard('{Escape}');
    const grace = screen.getByRole('spinbutton', { name: /Grace period/ });
    await userEvent.clear(grace);
    await userEvent.type(grace, '30');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual({
      totp: true,
      passkeys: true,
      required: { everyone: true, administrators: false, roles: ['readers', 'dev'] },
      graceDays: 30,
      requiredSince: '2026-10-01T00:00:00Z',
      relyingParty: { id: null, origins: [] },
    });
  });

  it('shows the server field errors on the controls they name', async () => {
    serve({
      'PUT /service/rest/v1/security/two-factor': () =>
        jsonResponse(
          [
            { id: 'graceDays', message: 'must be between 0 and 90' },
            { id: 'required.roles', message: 'the role dev does not exist' },
            { id: 'relyingParty.origins', message: 'https://evil.test: may use http only for localhost' },
          ],
          400,
        ),
    });
    renderWithApp(<TwoFactorPolicyPage />, { state: ADMIN_STATE });
    const grace = await screen.findByRole('spinbutton', { name: /Grace period/ });
    await userEvent.clear(grace);
    await userEvent.type(grace, '30');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('must be between 0 and 90')).toBeInTheDocument();
    expect(grace).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('button', { name: /Roles/ })).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('textbox', { name: 'Origins' })).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText('https://evil.test: may use http only for localhost')).toBeInTheDocument();
  });

  it('shows the passkeys switch disabled with the note that passkeys are not available yet', async () => {
    serve();
    renderWithApp(<TwoFactorPolicyPage />, { state: ADMIN_STATE });
    expect(await screen.findByRole('switch', { name: /Passkeys/ })).toBeDisabled();
    expect(screen.getByText(/People can use a passkey on their device/)).toBeInTheDocument();
  });

  it('warns when both methods are off and someone is required', async () => {
    serve({
      'GET /service/rest/v1/security/two-factor': () =>
        jsonResponse({ ...POLICY, passkeys: false, required: { ...POLICY.required, everyone: true } }),
    });
    renderWithApp(<TwoFactorPolicyPage />, { state: ADMIN_STATE });
    await userEvent.click(await screen.findByRole('switch', { name: /Authenticator apps/ }));
    expect(
      screen.getByText('Turn on at least one method before anyone is required to use a second factor.'),
    ).toBeInTheDocument();
  });
});
