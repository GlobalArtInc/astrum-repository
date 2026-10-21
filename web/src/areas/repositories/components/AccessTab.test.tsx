import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../../test/http';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { UiRepository } from '../api';
import { AccessTab } from './AccessTab';

const REPOSITORY = { name: 'npm-hosted', format: 'npm' } as UiRepository;
const ACTIONS = ['BROWSE', 'READ', 'EDIT', 'ADD', 'DELETE'];
const n = (action: string) => `as-repository-view-npm-npm-hosted-${action.toLowerCase()}`;

const PRIVILEGES = ACTIONS.map((action) => ({
  name: n(action),
  type: 'repository-view',
  format: 'npm',
  repository: 'npm-hosted',
  actions: [action],
}));

const ROLES = [
  { id: 'npm-reader', name: 'npm-reader', description: '', privileges: [n('BROWSE'), n('READ')], roles: [] },
  {
    id: 'public-reader',
    name: 'public-reader',
    description: '',
    privileges: [n('BROWSE'), n('READ')],
    roles: [],
  },
  { id: 'ops', name: 'ops', description: '', privileges: ['other'], roles: [] },
];

const USERS = [
  {
    userId: 'anonymous',
    firstName: 'Anonymous',
    lastName: 'User',
    emailAddress: '',
    source: 'default',
    status: 'active',
    roles: ['public-reader'],
  },
  {
    userId: 'jane',
    firstName: 'Jane',
    lastName: 'Doe',
    emailAddress: 'jane@example.org',
    source: 'default',
    status: 'active',
    roles: ['npm-reader'],
  },
];

function serve(extra: Record<string, Handler> = {}) {
  const server = fakeServer({
    'GET /service/rest/v1/security/privileges': () => jsonResponse(PRIVILEGES),
    'GET /service/rest/v1/security/roles': () => jsonResponse(ROLES),
    'GET /service/rest/v1/security/roles/assignable': () => jsonResponse(ROLES),
    'GET /service/rest/v1/security/users': () => jsonResponse(USERS),
    'GET /service/rest/v1/security/anonymous': () => jsonResponse({ enabled: true, userId: 'anonymous' }),
    'PUT /service/rest/v1/security/roles/npm-reader': () => new Response(null, { status: 204 }),
    'PUT /service/rest/v1/security/roles/public-reader': () => new Response(null, { status: 204 }),
    'PUT /service/rest/v1/security/roles/ops': () => new Response(null, { status: 204 }),
    'PUT /service/rest/v1/security/users/jane': () => new Response(null, { status: 204 }),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  renderWithApp(<AccessTab repository={REPOSITORY} />);
  return server;
}

const writes = (server: ReturnType<typeof fakeServer>) =>
  server.calls.filter((call) => call.method !== 'GET');

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the access of a repository', () => {
  it('changes the level of a role', async () => {
    const server = serve();
    const roles = await screen.findByRole('region', { name: 'Roles with Access' });
    await userEvent.click(
      await within(roles).findByRole('combobox', { name: 'Access of the role npm-reader' }),
    );
    await userEvent.click(await screen.findByRole('option', { name: 'Full access' }));
    await waitFor(() => expect(writes(server)).toHaveLength(1));
    expect(writes(server)[0]).toEqual({
      method: 'PUT',
      path: '/service/rest/v1/security/roles/npm-reader',
      body: {
        id: 'npm-reader',
        name: 'npm-reader',
        description: '',
        privileges: [n('BROWSE'), n('READ'), n('ADD'), n('EDIT'), n('DELETE')],
        roles: [],
      },
    });
  });

  it('grants access to another role', async () => {
    const server = serve();
    const roles = await screen.findByRole('region', { name: 'Roles with Access' });
    await userEvent.click(await within(roles).findByRole('combobox', { name: 'Role' }));
    await userEvent.click(await screen.findByRole('option', { name: 'ops' }));
    await userEvent.click(within(roles).getByRole('button', { name: 'Grant access' }));
    await waitFor(() => expect(writes(server)).toHaveLength(1));
    expect(writes(server)[0]?.body).toMatchObject({
      id: 'ops',
      privileges: ['other', n('BROWSE'), n('READ')],
    });
  });

  it('takes a role from a user after asking', async () => {
    const server = serve();
    const users = await screen.findByRole('region', { name: 'Users with Access' });
    await userEvent.click(
      await within(users).findByRole('button', { name: 'Take the role npm-reader from jane' }),
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('jane loses the role npm-reader');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Take away' }));
    await waitFor(() => expect(writes(server)).toHaveLength(1));
    expect(writes(server)[0]).toMatchObject({
      path: '/service/rest/v1/security/users/jane',
      body: { userId: 'jane', source: 'default', status: 'active', roles: [] },
    });
  });

  it('closes the repository to anonymous users', async () => {
    const server = serve();
    const card = await screen.findByRole('region', { name: 'Anonymous Access' });
    const toggle = await within(card).findByRole('switch', { name: 'Anonymous access to this repository' });
    await waitFor(() => expect(toggle).toBeChecked());
    await userEvent.click(toggle);
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('removed from these roles: public-reader');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Close' }));
    await waitFor(() => expect(writes(server)).toHaveLength(1));
    expect(writes(server)[0]).toMatchObject({
      path: '/service/rest/v1/security/roles/public-reader',
      body: { privileges: [] },
    });
  });

  it('opens the repository to anonymous users through a role of their own', async () => {
    const closed = USERS.map((user) => (user.userId === 'anonymous' ? { ...user, roles: [] } : user));
    const server = serve({
      'GET /service/rest/v1/security/users': () => jsonResponse(closed),
      'POST /service/rest/v1/security/roles': () => new Response(null, { status: 200 }),
      'PUT /service/rest/v1/security/users/anonymous': () => new Response(null, { status: 204 }),
    });
    const card = await screen.findByRole('region', { name: 'Anonymous Access' });
    expect(
      await within(card).findByText('Anonymous users cannot reach this repository.'),
    ).toBeInTheDocument();
    await userEvent.click(within(card).getByRole('switch'));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Open' }),
    );
    await waitFor(() => expect(writes(server)).toHaveLength(2));
    expect(writes(server)[0]).toMatchObject({
      method: 'POST',
      body: { id: 'anonymous-reader', privileges: [n('BROWSE'), n('READ')] },
    });
    expect(writes(server)[1]).toMatchObject({
      path: '/service/rest/v1/security/users/anonymous',
      body: { roles: ['anonymous-reader'] },
    });
  });

  it('points to the server settings while anonymous access is off', async () => {
    serve({
      'GET /service/rest/v1/security/anonymous': () => jsonResponse({ enabled: false, userId: 'anonymous' }),
    });
    const card = await screen.findByRole('region', { name: 'Anonymous Access' });
    expect(
      await within(card).findByText('Anonymous access is turned off for the whole server.'),
    ).toBeInTheDocument();
    expect(within(card).getByRole('link', { name: 'Anonymous access settings' })).toHaveAttribute(
      'href',
      '/admin/security/anonymous',
    );
    expect(within(card).queryByRole('switch')).not.toBeInTheDocument();
  });
});
