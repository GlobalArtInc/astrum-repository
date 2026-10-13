import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import AnonymousPage, { anonymousRealms } from './AnonymousPage';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the anonymous access page', () => {
  it('turns anonymous access on as the named user', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/anonymous': () =>
        jsonResponse({ enabled: false, userId: 'anonymous', realmName: 'AstrumAuthorizingRealm' }),
      'PUT /service/rest/v1/security/anonymous': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<AnonymousPage />);
    const allow = await screen.findByRole('checkbox', { name: 'Let anonymous users access the server' });
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(screen.getByRole('combobox', { name: 'Realm' })).toHaveTextContent('Local Authorizing Realm');
    await userEvent.click(allow);
    const user = screen.getByRole('textbox', { name: 'Username' });
    await userEvent.clear(user);
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Fill this in')).toBeInTheDocument();
    await userEvent.type(user, ' guest ');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() =>
      expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual({
        enabled: true,
        userId: 'guest',
        realmName: 'AstrumAuthorizingRealm',
      }),
    );
    expect(await screen.findByText('Anonymous access is on')).toBeInTheDocument();
  });

  it('keeps an unknown realm selectable', () => {
    expect(anonymousRealms('Custom').map((realm) => realm.label)).toEqual([
      'LDAP Realm',
      'Local Authorizing Realm',
      'OpenID Connect Realm',
      'Custom',
    ]);
  });
});
