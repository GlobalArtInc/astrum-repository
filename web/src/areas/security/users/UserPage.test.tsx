import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import UserPage from './UserPage';

const ROLES = [
  {
    id: 'dev',
    source: 'default',
    name: 'Developers',
    description: '',
    readOnly: false,
    privileges: ['deploy'],
    roles: ['readers'],
  },
  {
    id: 'readers',
    source: 'default',
    name: 'Readers',
    description: '',
    readOnly: false,
    privileges: ['read'],
    roles: [],
  },
];
const PRIVILEGES = [
  { type: 'wildcard', name: 'deploy', description: '', readOnly: false, pattern: 'astrum:component:create' },
  { type: 'wildcard', name: 'read', description: '', readOnly: false, pattern: 'astrum:search:read' },
];
const JANE = {
  userId: 'jane',
  firstName: 'Jane',
  lastName: 'Doe',
  emailAddress: 'jane@example.org',
  source: 'default',
  status: 'active',
  readOnly: false,
  roles: ['dev'],
  externalRoles: [],
};

const ROUTES = [
  { path: '/admin/security/users/create', element: <UserPage /> },
  { path: '/admin/security/users/:userId', element: <UserPage /> },
  { path: '/admin/security/users', element: <p>list</p> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(extra = {}) {
  const server = fakeServer({
    'GET /service/rest/v1/security/roles': () => jsonResponse(ROLES),
    'GET /service/rest/v1/security/roles/assignable': () => jsonResponse(ROLES),
    'GET /service/rest/v1/security/privileges': () => jsonResponse(PRIVILEGES),
    'GET /service/rest/v1/security/user-sources': () =>
      jsonResponse([{ id: 'default', name: 'AstrumAuthenticatingRealm' }]),
    'GET /service/rest/v1/security/users?source=default&userId=jane': () => jsonResponse([JANE]),
    'GET /service/rest/v1/security/users?source=default&userId=admin': () =>
      jsonResponse([{ ...JANE, userId: 'admin', roles: [] }]),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

describe('the user page', () => {
  it('creates a user with roles after checking the password', async () => {
    const server = serve({ 'POST /service/rest/v1/security/users': () => jsonResponse(JANE) });
    const { router } = renderWithApp(null, { path: '/admin/security/users/create', routes: ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: /User ID/ }), 'jane');
    await userEvent.type(screen.getByRole('textbox', { name: /First name/ }), 'Jane');
    await userEvent.type(screen.getByRole('textbox', { name: /Last name/ }), 'Doe');
    await userEvent.type(screen.getByRole('textbox', { name: /Email/ }), 'jane@example.org');
    await userEvent.type(screen.getByLabelText(/^Password/), 'correct horse');
    await userEvent.type(screen.getByLabelText(/Repeat the password/), 'correct hose');
    await userEvent.click(await screen.findByRole('option', { name: /dev/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Create user' }));
    expect(screen.getByText('The passwords do not match')).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText(/Repeat the password/));
    await userEvent.type(screen.getByLabelText(/Repeat the password/), 'correct horse');
    await userEvent.click(screen.getByRole('button', { name: 'Create user' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/users/jane'));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      userId: 'jane',
      firstName: 'Jane',
      lastName: 'Doe',
      emailAddress: 'jane@example.org',
      password: 'correct horse',
      status: 'active',
      roles: ['dev'],
    });
  }, 20_000);

  it('shows effective privileges and changes the password as plain text', async () => {
    const server = serve();
    const sent: Array<{ body: unknown; type: string | null }> = [];
    vi.stubGlobal('fetch', async (input: RequestInfo | URL, init: RequestInit = {}) => {
      if (String(input) === '/service/rest/v1/security/users/jane/change-password') {
        sent.push({ body: init.body, type: new Headers(init.headers).get('Content-Type') });
        return new Response(null, { status: 204 });
      }
      return server.fetch(input, init);
    });
    renderWithApp(null, { path: '/admin/security/users/jane', routes: ROUTES });
    expect(await screen.findByRole('heading', { name: 'jane', level: 1 })).toBeInTheDocument();
    expect(await screen.findByText('dev → readers')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Change password' }));
    await userEvent.type(await screen.findByLabelText(/^New password/), 'new secret 1');
    await userEvent.type(screen.getByLabelText(/Repeat the new password/), 'new secret 1');
    await userEvent.click(screen.getAllByRole('button', { name: 'Change password' }).at(-1) as HTMLElement);
    await waitFor(() => expect(sent).toEqual([{ body: 'new secret 1', type: 'text/plain; charset=utf-8' }]));
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
  });

  it('does not offer to delete the signed-in user', async () => {
    serve();
    renderWithApp(null, { path: '/admin/security/users/admin', routes: ROUTES, state: ADMIN_STATE });
    expect(await screen.findByRole('heading', { name: 'admin', level: 1 })).toBeInTheDocument();
    expect(await screen.findByRole('button', { name: 'Change password' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();
  });
});
