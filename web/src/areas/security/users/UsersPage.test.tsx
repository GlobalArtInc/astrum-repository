import { screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import UsersPage from './UsersPage';

function user(userId: string, extra: object = {}) {
  return {
    userId,
    firstName: null,
    lastName: null,
    emailAddress: null,
    source: 'default',
    status: 'active',
    readOnly: false,
    roles: [],
    externalRoles: [],
    ...extra,
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the users list', () => {
  it('shows the second factor badge for the four states', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/user-sources': () => jsonResponse([]),
      'GET /service/rest/v1/security/users?source=default': () =>
        jsonResponse([
          user('alice', { twoFactor: { enrolled: true, required: true, requirement: 'policy' } }),
          user('bob', { twoFactor: { enrolled: false, required: true, requirement: 'policy' } }),
          user('carol', { twoFactor: { enrolled: false, required: false, requirement: 'exempt' } }),
          user('dave', { twoFactor: { enrolled: false, required: false, requirement: 'policy' } }),
          user('eve'),
        ]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<UsersPage />, { state: ADMIN_STATE });
    const rowOf = async (userId: string) => (await screen.findByText(userId)).closest('tr') as HTMLElement;
    expect(within(await rowOf('alice')).getByText('Set up')).toBeVisible();
    expect(within(await rowOf('bob')).getByText('Required')).toBeVisible();
    expect(within(await rowOf('carol')).getByText('Exempt')).toBeVisible();
    for (const userId of ['dave', 'eve']) {
      expect(within(await rowOf(userId)).queryByText(/^(Set up|Required|Exempt)$/)).toBeNull();
    }
  });
});
