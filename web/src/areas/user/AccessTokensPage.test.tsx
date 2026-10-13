import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import AccessTokensPage from './AccessTokensPage';
import AccountPage from './AccountPage';

const TOKEN = {
  id: '0b6c3f3e-0000-4000-8000-000000000001',
  userId: 'admin',
  name: 'ci',
  hint: 'astrum_abcd',
  scope: 'READ',
  createdAt: '2026-10-01T10:00:00Z',
  expiresAt: '2099-01-01T00:00:00Z',
  lastUsedAt: null,
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the access tokens page', () => {
  it('creates a token, shows it once with examples and revokes it', async () => {
    let tokens: object[] = [TOKEN];
    const server = fakeServer({
      'GET /service/rest/internal/access-tokens': () => jsonResponse(tokens),
      'POST /service/rest/internal/access-tokens': ({ body }) => {
        const created = {
          ...TOKEN,
          id: 'new',
          name: (body as { name: string }).name,
          scope: 'WRITE',
          expiresAt: null,
          token: 'astrum_secret123',
        };
        tokens = [...tokens, created];
        return jsonResponse(created, 201);
      },
      [`DELETE /service/rest/internal/access-tokens/${TOKEN.id}`]: () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<AccessTokensPage />);
    expect(await screen.findByText('ci')).toBeInTheDocument();
    expect(screen.getByText('Not used yet')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), 'laptop');
    await userEvent.click(screen.getByRole('combobox', { name: 'Access' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Read and write' }));
    expect(screen.getByText('Everything you can do yourself, publishing included')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Lifetime' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Never' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create Token' }));
    const dialog = await screen.findByRole('dialog', { name: 'Token Created' });
    expect(within(dialog).getByTestId('created-token')).toHaveTextContent('astrum_secret123');
    expect(within(dialog).getByText(/docker login .* -u admin --password-stdin/)).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('tab', { name: /^curl/ }));
    expect(within(dialog).getByText(/curl -u admin:astrum_secret123 -O/)).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'laptop',
      scope: 'WRITE',
      expiresInDays: null,
    });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Done' }));
    expect(screen.queryByText('astrum_secret123')).toBeNull();
    expect(await screen.findByText('laptop')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Revoke ci' }));
    const confirm = await screen.findByRole('alertdialog', { name: 'Revoke Token' });
    expect(confirm).toHaveTextContent('Clients that use “ci” stop working at once.');
    await userEvent.click(within(confirm).getByRole('button', { name: 'Revoke' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true));
  }, 20_000);

  it('explains that tokens need a password sign-in', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({
        'GET /service/rest/internal/access-tokens': () =>
          jsonResponse(
            { id: '*', message: 'access tokens are managed after signing in with a password' },
            403,
          ),
      }).fetch,
    );
    renderWithApp(<AccessTokensPage />);
    expect(await screen.findByText('Sign in with your password first')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Create Token' })).toBeNull();
  });
});

describe('the account page', () => {
  it('saves the name and reports a wrong current password on its field', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/user': () =>
        jsonResponse({
          userId: 'admin',
          firstName: 'Administrator',
          lastName: 'User',
          email: 'admin@example.org',
          source: 'default',
          realm: 'AstrumAuthenticatingRealm',
          status: 'active',
          roles: ['as-admin'],
          readOnly: false,
        }),
      'PUT /service/rest/internal/ui/user': ({ body }) => jsonResponse(body),
      'PUT /service/rest/internal/ui/user/password': () =>
        jsonResponse([{ id: 'currentPassword', message: 'the current password is wrong' }], 400),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<AccountPage />);
    expect(await screen.findByText('Local')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Change Password' })).toBeDisabled();
    const first = screen.getByRole('textbox', { name: /First name/ });
    await userEvent.clear(first);
    await userEvent.type(first, 'Ada');
    await userEvent.click(screen.getByRole('button', { name: 'Save changes' }));
    await waitFor(() =>
      expect(
        server.calls.find((call) => call.path === '/service/rest/internal/ui/user' && call.method === 'PUT')
          ?.body,
      ).toEqual({
        firstName: 'Ada',
        lastName: 'User',
        email: 'admin@example.org',
      }),
    );
    await userEvent.type(screen.getByLabelText(/Current Password/), 'not it');
    await userEvent.type(screen.getByLabelText(/^New Password/), 'a better one');
    await userEvent.type(screen.getByLabelText(/Repeat New Password/), 'a better one');
    await userEvent.click(screen.getByRole('button', { name: 'Change Password' }));
    expect(await screen.findByText('The current password is wrong')).toBeInTheDocument();
    expect(screen.getByLabelText(/Current Password/)).toHaveAttribute('aria-invalid', 'true');
  }, 20_000);
});
