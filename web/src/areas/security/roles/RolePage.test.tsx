import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import RolePage from './RolePage';

const ROUTES = [
  { path: '/admin/security/roles/create', element: <RolePage /> },
  { path: '/admin/security/roles/:id', element: <RolePage /> },
  { path: '/admin/security/roles/:id/profile', element: <p>profile</p> },
  { path: '/admin/security/roles', element: <p>list</p> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve() {
  const fake = fakeServer({
    'GET /service/rest/v1/security/roles': () =>
      jsonResponse([
        {
          id: 'readers',
          source: 'default',
          name: 'readers',
          description: '',
          readOnly: false,
          privileges: [],
          roles: [],
        },
      ]),
    'GET /service/rest/v1/security/roles/assignable': () =>
      jsonResponse([
        {
          id: 'readers',
          source: 'default',
          name: 'readers',
          description: '',
          readOnly: false,
          privileges: [],
          roles: [],
        },
      ]),
    'GET /service/rest/v1/security/privileges': () =>
      jsonResponse([
        {
          type: 'application',
          name: 'as-users-read',
          description: 'Read users',
          readOnly: true,
          domain: 'users',
          actions: ['READ'],
        },
        {
          type: 'repository-view',
          name: 'as-repository-view-*-*-read',
          description: 'Read',
          readOnly: true,
          format: '*',
          repository: '*',
          actions: ['READ'],
        },
      ]),
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

describe('the role wizard', () => {
  it('creates a role step by step with privileges and contained roles', async () => {
    const fake = serve();
    renderWithApp(null, { path: '/admin/security/roles/create', routes: ROUTES });
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));
    const next = await screen.findByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /^ID/ }), 'team');
    await userEvent.type(screen.getByRole('textbox', { name: /^Name/ }), 'Team');
    await userEvent.click(next);
    await userEvent.click(await screen.findByRole('combobox'));
    await userEvent.click(await screen.findByRole('option', { name: /as-users-read/ }));
    expect(screen.getByText('1 privilege granted')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await userEvent.dblClick(await screen.findByRole('option', { name: 'readers' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create Role' }));
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === 'POST')?.body).toEqual({
        id: 'team',
        source: 'default',
        name: 'Team',
        description: '',
        privileges: ['as-users-read'],
        roles: ['readers'],
      }),
    );
  });

  it('maps a group found in LDAP to a role', async () => {
    const fake = serve();
    renderWithApp(null, { path: '/admin/security/roles/create', routes: ROUTES });
    await userEvent.click(await screen.findByRole('combobox', { name: 'Type' }));
    await userEvent.click(await screen.findByRole('option', { name: 'External Role Mapping' }));
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(await screen.findByRole('combobox', { name: /External Source/ })).toHaveTextContent('LDAP');
    await userEvent.type(screen.getByRole('combobox', { name: 'Mapped Role' }), 'dev');
    await userEvent.click(await screen.findByRole('option', { name: 'developers' }, { timeout: 3000 }));
    expect(screen.getByRole('textbox', { name: /^ID/ })).toHaveValue('developers');
    expect(screen.getByRole('textbox', { name: /^Name/ })).toHaveValue('developers');
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Continue' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Create Role' }));
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
    expect(await screen.findByText('Step 1: Setup (source: LDAP)')).toBeInTheDocument();
    const source = await screen.findByRole('combobox', { name: /External Source/ });
    await waitFor(() => expect(source).toHaveTextContent('LDAP'));
    expect(source).toBeDisabled();
    expect(screen.getByRole('textbox', { name: /^ID/ })).toBeDisabled();
    expect(screen.queryByRole('combobox', { name: 'Mapped Role' })).toBeNull();
  });
});
