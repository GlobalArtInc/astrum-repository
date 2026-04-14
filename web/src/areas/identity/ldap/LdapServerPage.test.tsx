import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import LdapServerPage from './LdapServerPage';

const LIST = '/service/rest/v1/security/ldap';
const ROUTES = [
  { path: '/admin/security/ldap/create', element: <LdapServerPage /> },
  { path: '/admin/security/ldap/:name', element: <LdapServerPage /> },
  { path: '/admin/security/ldap', element: <p>the list</p> },
];
const CORP = {
  id: 'id-1',
  order: 1,
  name: 'corp',
  protocol: 'LDAP',
  useTrustStore: false,
  useStartTls: false,
  host: 'ldap.example.com',
  port: 389,
  searchBase: 'dc=example,dc=com',
  authScheme: 'SIMPLE',
  authUsername: 'cn=svc,dc=example,dc=com',
  connectionTimeoutSeconds: 30,
  connectionRetryDelaySeconds: 300,
  maxIncidentsCount: 3,
  userBaseDn: 'ou=people',
  userSubtree: false,
  userObjectClass: 'inetOrgPerson',
  userIdAttribute: 'uid',
  userRealNameAttribute: 'cn',
  userEmailAddressAttribute: 'mail',
  ldapGroupsAsRoles: true,
  groupType: 'DYNAMIC',
  groupSubtree: false,
  userMemberOfAttribute: 'memberOf',
};
const ALICE = {
  username: 'alice',
  realName: 'Alice Smith',
  email: 'alice@example.com',
  membership: ['developers'],
};

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(extra = {}) {
  const fake = fakeServer({
    [`GET ${LIST}`]: () => jsonResponse([CORP]),
    [`GET ${LIST}/corp`]: () => jsonResponse(CORP),
    [`GET ${LIST}/templates`]: () => jsonResponse([]),
    [`POST ${LIST}/verify-connection`]: ({ body }) =>
      (body as { authPassword: string }).authPassword === 'svc-secret'
        ? new Response(null, { status: 204 })
        : jsonResponse(
            [{ id: '*', message: 'The LDAP server rejected the username or password of the system account' }],
            400,
          ),
    [`POST ${LIST}`]: () => new Response(null, { status: 201 }),
    ...extra,
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

const field = (name: string) => screen.getByRole('textbox', { name });

describe('the LDAP server wizard', () => {
  it('verifies the connection, maps users and creates the server', async () => {
    const fake = serve({
      [`POST ${LIST}/verify-user-mapping`]: () => jsonResponse([ALICE]),
      [`POST ${LIST}/verify-login`]: () =>
        jsonResponse([{ id: '*', message: "The password for user 'alice' is incorrect" }], 400),
    });
    const { router } = renderWithApp(null, { path: '/admin/security/ldap/create', routes: ROUTES });
    expect(await screen.findByRole('heading', { level: 1, name: 'Create LDAP Server' })).toBeInTheDocument();
    const next = await screen.findByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.type(field('Name'), 'Corp');
    await userEvent.type(field('Hostname'), 'ldap.example.com');
    await userEvent.type(field('Search Base DN'), 'dc=example,dc=com');
    await userEvent.type(field('Username'), 'cn=svc,dc=example,dc=com');
    await userEvent.type(screen.getByLabelText(/^Password/), 'wrong');
    expect(next).toBeDisabled();
    await userEvent.clear(field('Name'));
    await userEvent.type(field('Name'), 'office');
    await userEvent.click(screen.getByRole('button', { name: 'Verify Connection' }));
    expect(await screen.findByText('Connection verification failed')).toBeInTheDocument();
    expect(
      screen.getByText('The LDAP server rejected the username or password of the system account'),
    ).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText(/^Password/));
    await userEvent.type(screen.getByLabelText(/^Password/), 'svc-secret');
    expect(screen.queryByText('Connection verification failed')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Verify Connection' }));
    expect(await screen.findByText('Connection successful')).toBeInTheDocument();

    await userEvent.click(next);
    expect(await screen.findByRole('heading', { name: 'User & Group' })).toBeInTheDocument();
    await userEvent.type(field('User relative DN'), 'ou=people');
    await userEvent.click(screen.getByRole('button', { name: 'Verify User Mapping' }));
    const table = await screen.findByRole('table', { name: 'Verified LDAP users' });
    expect(
      within(table).getByRole('row', { name: /alice Alice Smith alice@example.com developers/ }),
    ).toBeVisible();
    expect(screen.getByText('1 user')).toBeInTheDocument();
    expect(screen.getByText('Found 1 user(s):')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Verify Login' }));
    const dialog = await screen.findByRole('dialog', { name: 'Test LDAP Login' });
    await userEvent.type(within(dialog).getByPlaceholderText('Enter LDAP username'), 'alice');
    await userEvent.type(within(dialog).getByPlaceholderText('Enter password'), 'nope');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Test Login' }));
    expect(await within(dialog).findByText("The password for user 'alice' is incorrect")).toBeInTheDocument();
    const login = fake.calls.find((call) => call.path === `${LIST}/verify-login`)?.body as Record<
      string,
      unknown
    >;
    expect(login).toMatchObject({ base64Username: btoa('alice'), base64Password: btoa('nope') });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Close' }));

    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/ldap'));
    const created = fake.calls.find((call) => call.method === 'POST' && call.path === LIST)?.body;
    expect(created).toMatchObject({
      name: 'office',
      protocol: 'LDAP',
      useStartTls: false,
      host: 'ldap.example.com',
      port: 389,
      authScheme: 'SIMPLE',
      authPassword: 'svc-secret',
      userBaseDn: 'ou=people',
      groupType: 'DYNAMIC',
      userMemberOfAttribute: 'memberOf',
    });
    expect(await screen.findByText('LDAP server "office" created successfully')).toBeInTheDocument();
  }, 30_000);

  it('asks for the bind password before verifying a stored server', async () => {
    const fake = serve({
      [`PUT ${LIST}/corp`]: () => new Response(null, { status: 204 }),
      [`POST ${LIST}/verify-connection?existingServerName=corp`]: () => new Response(null, { status: 204 }),
    });
    renderWithApp(null, { path: '/admin/security/ldap/corp', routes: ROUTES });
    expect(await screen.findByRole('heading', { level: 1, name: 'Edit corp' })).toBeInTheDocument();
    expect(await screen.findByText('Leave blank to keep the existing password')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Verify Connection' }));
    const dialog = await screen.findByRole('dialog', { name: 'Enter LDAP Password' });
    await userEvent.type(within(dialog).getByLabelText('Password'), 'svc-secret');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Verify' }));
    expect(await screen.findByText('Connection successful')).toBeInTheDocument();
    const verify = fake.calls.find((call) => call.path.startsWith(`${LIST}/verify-connection`));
    expect(verify?.path).toBe(`${LIST}/verify-connection?existingServerName=corp`);
    expect(verify?.body).toMatchObject({ id: 'id-1', authPassword: 'svc-secret' });

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await userEvent.click(await screen.findByRole('button', { name: 'Save' }));
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
        name: 'corp',
        authPassword: 'svc-secret',
      }),
    );
  });

  it('switches the port with the protocol and offers the truststore for TLS', async () => {
    serve();
    renderWithApp(null, { path: '/admin/security/ldap/create', routes: ROUTES });
    expect(await screen.findByRole('textbox', { name: 'Port' })).toHaveValue('389');
    expect(screen.queryByRole('checkbox', { name: 'Use Astrum SSL Trust Store' })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Protocol' }));
    await userEvent.click(await screen.findByRole('option', { name: 'ldaps (SSL)' }));
    expect(field('Port')).toHaveValue('636');
    expect(screen.getByRole('checkbox', { name: 'Use Astrum SSL Trust Store' })).toBeInTheDocument();
  });
});
