import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import LdapListPage from './LdapListPage';

const LIST = '/service/rest/v1/security/ldap';

function server(name: string, order: number) {
  return {
    id: name,
    order,
    name,
    protocol: 'LDAP',
    useTrustStore: false,
    useStartTls: false,
    host: `${name}.example.com`,
    port: 389,
    searchBase: 'dc=example,dc=com',
    authScheme: 'NONE',
    connectionTimeoutSeconds: 30,
    connectionRetryDelaySeconds: 300,
    maxIncidentsCount: 3,
    userSubtree: false,
    ldapGroupsAsRoles: false,
    groupSubtree: false,
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(servers: unknown[]) {
  const fake = fakeServer({
    [`GET ${LIST}`]: () => jsonResponse(servers),
    [`POST ${LIST}/change-order`]: () => new Response(null, { status: 204 }),
    [`DELETE ${LIST}/cache`]: () => new Response(null, { status: 204 }),
    [`DELETE ${LIST}/backup`]: () => new Response(null, { status: 204 }),
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

describe('the LDAP list page', () => {
  it('says how to start when no server is configured', async () => {
    serve([]);
    renderWithApp(<LdapListPage />);
    expect(await screen.findByText('No LDAP servers configured')).toBeInTheDocument();
    expect(screen.getByText('Click "Create LDAP Server" to add one')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Change Order' })).toBeDisabled();
    expect(screen.getByRole('link', { name: 'Create LDAP Server' })).toHaveAttribute(
      'href',
      '/admin/security/ldap/create',
    );
    expect(screen.getByRole('link', { name: 'View Documentation' })).toHaveAttribute('target', '_blank');
  });

  it('lists servers in order, filters them and clears the cache', async () => {
    const fake = serve([server('backup', 2), server('corp', 1)]);
    renderWithApp(<LdapListPage />);
    const rows = await screen.findAllByRole('row');
    expect(rows.slice(1).map((row) => row.textContent)).toEqual([
      expect.stringContaining('corp'),
      expect.stringContaining('backup'),
    ]);
    expect(screen.getByText('ldap://corp.example.com:389')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter servers' }), 'BACKUP.example');
    expect(screen.getAllByRole('row')).toHaveLength(2);
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter servers' }), 'zzz');
    expect(screen.getByText('No servers match the filter')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Clear Cache' }));
    expect(await screen.findByText('LDAP cache cleared successfully')).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'DELETE' && call.path === `${LIST}/cache`)).toBe(true);
  });

  it('changes the order in a dialog', async () => {
    const fake = serve([server('corp', 1), server('backup', 2)]);
    renderWithApp(<LdapListPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Change Order' }));
    const dialog = await screen.findByRole('dialog', { name: 'Change Server Order' });
    expect(within(dialog).getByRole('button', { name: 'Move corp up' })).toBeDisabled();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Move corp down' }));
    expect(
      within(dialog)
        .getAllByRole('listitem')
        .map((item) => item.textContent),
    ).toEqual(['1.backup', '2.corp']);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save Order' }));
    await waitFor(() =>
      expect(fake.calls.find((call) => call.path === `${LIST}/change-order`)?.body).toEqual([
        'backup',
        'corp',
      ]),
    );
    expect(await screen.findByText('Server order updated successfully')).toBeInTheDocument();
  });

  it('deletes a server after confirmation', async () => {
    const fake = serve([server('corp', 1), server('backup', 2)]);
    renderWithApp(<LdapListPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Delete backup' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Delete LDAP Server' });
    expect(dialog).toHaveTextContent('Are you sure you want to delete the LDAP server "backup"?');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete Server' }));
    expect(await screen.findByText('LDAP server "backup" deleted successfully')).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'DELETE' && call.path === `${LIST}/backup`)).toBe(true);
  });

  it('shows Russian texts', async () => {
    serve([]);
    renderWithApp(<LdapListPage />, { language: 'ru' });
    expect(await screen.findByText('LDAP-серверы не настроены')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Создать LDAP-сервер' })).toBeInTheDocument();
  });
});
