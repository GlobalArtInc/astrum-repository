import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import RolePage from './RolePage';

const ROUTES = [
  { path: '/admin/security/roles/create', element: <RolePage /> },
  { path: '/admin/security/roles/:id', element: <RolePage /> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve() {
  const fake = fakeServer({
    'GET /service/rest/v1/security/roles': () => jsonResponse([]),
    'GET /service/rest/v1/security/roles/assignable': () => jsonResponse([]),
    'GET /service/rest/v1/security/privileges': () => jsonResponse([]),
    'GET /service/rest/internal/ui/roles/sources': () =>
      jsonResponse([
        { id: 'LDAP', name: 'LDAP' },
        { id: 'OAuth2', name: 'OpenID Connect' },
      ]),
    'GET /service/rest/internal/ui/roles?source=LDAP&search=dev': () =>
      jsonResponse([
        { id: 'developers', source: 'LDAP', name: 'developers', description: '', readOnly: true },
      ]),
    'POST /service/rest/v1/security/roles': () => new Response(null, { status: 200 }),
    'GET /service/rest/v1/security/roles/developers': () =>
      jsonResponse({
        id: 'developers',
        source: 'LDAP',
        name: 'LDAP developers',
        description: '',
        readOnly: false,
        privileges: [],
        roles: [],
      }),
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

describe('external role mapping', () => {
  it('maps a group found in LDAP to a role', async () => {
    const fake = serve();
    renderWithApp(null, { path: '/admin/security/roles/create', routes: ROUTES });
    await userEvent.click(await screen.findByRole('combobox', { name: 'Type' }));
    await userEvent.click(await screen.findByRole('option', { name: 'External Role Mapping' }));
    expect(await screen.findByRole('combobox', { name: 'External Source' })).toHaveTextContent('LDAP');
    expect(
      screen.getByText('Enter or search for a role from LDAP. Type 3+ characters to search.'),
    ).toBeInTheDocument();
    await userEvent.type(screen.getByRole('combobox', { name: 'Mapped Role' }), 'dev');
    await userEvent.click(await screen.findByRole('option', { name: 'developers' }, { timeout: 3000 }));
    expect(screen.getByRole('textbox', { name: /^Role ID/ })).toHaveValue('developers');
    expect(screen.getByRole('textbox', { name: /^Name/ })).toHaveValue('developers');
    await userEvent.click(screen.getByRole('button', { name: 'Create role' }));
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === 'POST')?.body).toMatchObject({
        id: 'developers',
        source: 'LDAP',
        name: 'developers',
      }),
    );
  });

  it('shows the source of a stored mapping without letting it change', async () => {
    serve();
    renderWithApp(null, { path: '/admin/security/roles/developers', routes: ROUTES });
    const source = await screen.findByRole('combobox', { name: 'External Source' });
    await waitFor(() => expect(source).toHaveTextContent('LDAP'));
    expect(source).toBeDisabled();
    expect(screen.getByRole('combobox', { name: 'Type' })).toBeDisabled();
    expect(screen.queryByRole('combobox', { name: 'Mapped Role' })).toBeNull();
  });
});
