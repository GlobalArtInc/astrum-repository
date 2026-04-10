import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Role, User } from '../api';
import PrivilegeProfilePage, { rolesWith, usersWith } from './PrivilegeProfilePage';

const role = (id: string, privileges: string[], roles: string[] = []): Role => ({
  id,
  source: 'default',
  name: id,
  description: '',
  readOnly: false,
  privileges,
  roles,
});

const user = (userId: string, roles: string[]): User => ({
  userId,
  firstName: 'QA',
  lastName: 'User',
  emailAddress: null,
  source: 'default',
  status: 'active',
  readOnly: false,
  roles,
  externalRoles: [],
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the privilege profile', () => {
  it('finds the roles that grant a privilege and the users who reach it through nested roles', () => {
    const roles = [role('readers', ['docs']), role('team', [], ['readers']), role('other', ['x'])];
    expect(rolesWith('docs', roles).map((item) => item.id)).toEqual(['readers']);
    const users = [user('jane', ['team']), user('joe', ['other'])];
    expect(usersWith('docs', users, roles).map((item) => item.userId)).toEqual(['jane']);
  });

  it('shows the overview and the roles tab', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/privileges/docs': () =>
        jsonResponse({
          type: 'repository-view',
          name: 'docs',
          description: 'Docs',
          readOnly: false,
          format: 'raw',
          repository: 'files',
          actions: ['BROWSE', 'READ'],
        }),
      'GET /service/rest/v1/security/roles': () => jsonResponse([role('readers', ['docs'])]),
      'GET /service/rest/v1/security/users': () => jsonResponse([user('jane', ['readers'])]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, {
      path: '/admin/security/privileges/docs/profile',
      routes: [{ path: '/admin/security/privileges/:name/profile', element: <PrivilegeProfilePage /> }],
    });
    expect(await screen.findByRole('heading', { name: 'About the Privilege' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Edit Privilege' })).toHaveAttribute(
      'href',
      '/admin/security/privileges/docs',
    );
    await userEvent.click(screen.getByRole('tab', { name: /Roles With It/ }));
    expect(await screen.findByRole('link', { name: /readers/ })).toHaveAttribute(
      'href',
      '/admin/security/roles/readers/profile',
    );
  });
});
