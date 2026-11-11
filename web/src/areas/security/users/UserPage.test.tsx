import { screen, waitFor, within } from '@testing-library/react';
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
    await userEvent.type(await screen.findByRole('textbox', { name: /^ID/ }), 'jane');
    await userEvent.type(screen.getByRole('textbox', { name: /^First Name/ }), 'Jane');
    await userEvent.type(screen.getByRole('textbox', { name: /^Last Name/ }), 'Doe');
    await userEvent.type(screen.getByRole('textbox', { name: /^Email/ }), 'jane@example.org');
    await userEvent.type(screen.getByLabelText(/^Password/), 'correct horse');
    await userEvent.type(screen.getByLabelText(/^Confirm Password/), 'correct hose');
    expect(screen.getByText('Grant at least one role')).toBeInTheDocument();
    await userEvent.dblClick(await screen.findByRole('option', { name: 'dev' }));
    expect(screen.getByText('1 role granted')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(screen.getByText('The passwords do not match')).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText(/^Confirm Password/));
    await userEvent.type(screen.getByLabelText(/^Confirm Password/), 'correct horse');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/users'));
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

  it('shows the security tree and changes the password as plain text', async () => {
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
    expect(await screen.findByRole('heading', { name: 'Edit Jane Doe', level: 1 })).toBeInTheDocument();
    expect(await screen.findByText('User: jane (Local)')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete User' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Change Password' }));
    await userEvent.type(await screen.findByLabelText(/^New password/), 'new secret 1');
    await userEvent.type(screen.getByLabelText(/Repeat the new password/), 'new secret 1');
    await userEvent.click(screen.getAllByRole('button', { name: 'Change Password' }).at(-1) as HTMLElement);
    await waitFor(() => expect(sent).toEqual([{ body: 'new secret 1', type: 'text/plain; charset=utf-8' }]));
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
  });

  it('saves a switched status and does not offer to delete the signed-in user', async () => {
    const server = serve({
      'PUT /service/rest/v1/security/users/jane': () => new Response(null, { status: 204 }),
    });
    renderWithApp(null, { path: '/admin/security/users/admin', routes: ROUTES, state: ADMIN_STATE });
    expect(await screen.findByRole('button', { name: 'Change Password' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete User' })).toBeNull();
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
  });

  it('switches a user off with the status switch', async () => {
    const server = serve({
      'PUT /service/rest/v1/security/users/jane': () => new Response(null, { status: 204 }),
    });
    renderWithApp(null, { path: '/admin/security/users/jane', routes: ROUTES });
    await userEvent.click(await screen.findByRole('switch'));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() =>
      expect(server.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({ status: 'disabled' }),
    );
  });
});

const STATUS = {
  userId: 'jack',
  enrolled: true,
  totp: true,
  recoveryCodesLeft: 2,
  requirement: 'policy',
  requirementSource: 'policy',
  required: true,
  enrollBy: null,
  lockedUntil: null,
};
const JACK = {
  ...JANE,
  userId: 'jack',
  roles: [],
  twoFactor: { enrolled: true, required: true, requirement: 'policy' },
};
const ADMIN_FACTOR = { ...JACK, userId: 'admin' };

function serveFactor(extra = {}) {
  return serve({
    'GET /service/rest/v1/security/users?source=default&userId=jack': () => jsonResponse([JACK]),
    'GET /service/rest/v1/security/users/jack/two-factor': () => jsonResponse(STATUS),
    ...extra,
  });
}

describe('the second factor panel of the user page', () => {
  it('shows the factor status of a user who has one', async () => {
    serveFactor();
    renderWithApp(null, { path: '/admin/security/users/jack', routes: ROUTES, state: ADMIN_STATE });
    expect(await screen.findByText('2 recovery codes left')).toBeInTheDocument();
    expect(screen.getByText('Server policy')).toBeInTheDocument();
    expect(screen.getByText('Required')).toBeInTheDocument();
  });

  it('hides the panel for a user without a second factor', async () => {
    serve();
    renderWithApp(null, { path: '/admin/security/users/jane', routes: ROUTES, state: ADMIN_STATE });
    expect(await screen.findByRole('heading', { name: 'Edit Jane Doe', level: 1 })).toBeInTheDocument();
    expect(screen.queryByText('Second factor')).toBeNull();
  });

  it('sends the chosen requirement and shows the server refusal', async () => {
    const server = serveFactor({
      'PUT /service/rest/v1/security/users/jack/requirement': () =>
        jsonResponse(
          { id: '*', message: 'only an administrator may exempt a user from the second factor' },
          403,
        ),
    });
    renderWithApp(null, { path: '/admin/security/users/jack', routes: ROUTES, state: ADMIN_STATE });
    await userEvent.click(await screen.findByRole('combobox', { name: 'Requirement' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Exempt' }));
    expect(
      await screen.findByText(
        'The server refused: only an administrator may exempt a user from the second factor',
      ),
    ).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual({ requirement: 'exempt' });
  });

  it('resets the second factor only after the confirmation', async () => {
    const server = serveFactor({
      'DELETE /service/rest/v1/security/users/jack/two-factor': () => new Response(null, { status: 204 }),
    });
    renderWithApp(null, { path: '/admin/security/users/jack', routes: ROUTES, state: ADMIN_STATE });
    await userEvent.click(await screen.findByRole('button', { name: 'Reset Second Factor' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(/signed out everywhere/)).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'DELETE')).toBe(false);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Reset Second Factor' }));
    await waitFor(() => expect(server.calls.filter((call) => call.method === 'DELETE')).toHaveLength(1));
  });

  it('does not offer the reset for the signed-in user', async () => {
    serveFactor({
      'GET /service/rest/v1/security/users?source=default&userId=admin': () => jsonResponse([ADMIN_FACTOR]),
      'GET /service/rest/v1/security/users/admin/two-factor': () =>
        jsonResponse({ ...STATUS, userId: 'admin' }),
    });
    renderWithApp(null, { path: '/admin/security/users/admin', routes: ROUTES, state: ADMIN_STATE });
    expect(await screen.findByText('2 recovery codes left')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Reset Second Factor' })).toBeNull();
  });
});
