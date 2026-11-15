import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { customerUrl, tokensUrl } from './api';
import CustomerPage from './CustomerPage';

afterEach(() => {
  vi.unstubAllGlobals();
});

const ACME = {
  id: 'acme',
  name: 'Acme',
  reference: '',
  note: '',
  disabled: false,
  createdAt: '2026-10-01T10:00:00Z',
  createdBy: 'admin',
  tokens: 0,
  nearestExpiry: null,
  downloads30d: 0,
  bytes30d: 0,
  lastAccessAt: null,
};

const REPOSITORIES = [
  { name: 'releases', format: 'docker', type: 'hosted' },
  { name: 'charts', format: 'helm', type: 'hosted' },
  { name: 'mirror', format: 'npm', type: 'proxy' },
  { name: 'files', format: 'raw', type: 'hosted' },
  { name: 'bundle', format: 'raw', type: 'group' },
];

const USERS = [
  { userId: 'deploy', source: 'default', status: 'active' },
  { userId: 'ghost', source: 'default', status: 'disabled' },
  { userId: 'ldap-user', source: 'ldap', status: 'active' },
];

const USAGE = { customer: 'acme', days: 30, totals: { downloads: 0, bytes: 0 }, daily: [] };

const ISSUED = {
  id: 'tok-3',
  name: 'Build',
  hint: 'astrum_tok3',
  token: 'astrum_secret_value',
  expiresAt: '2027-01-01T00:00:00Z',
  owner: 'deploy',
  repositories: ['releases', 'charts', 'mirror', 'files'],
};

const TOKENS_URL = tokensUrl('acme');
const REPOSITORIES_URL = '/service/rest/v1/repositories';
const USERS_URL = '/service/rest/v1/security/users?source=default';

function renderDetail(permissions?: string[]) {
  const state = permissions ? { ...ADMIN_STATE, permissions } : ADMIN_STATE;
  return renderWithApp(<CustomerPage />, {
    state,
    path: '/admin/customers/acme',
    routes: [{ path: '/admin/customers/:id', element: <CustomerPage /> }],
  });
}

function server(extra: Record<string, () => Response> = {}) {
  return fakeServer({
    [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
    [`GET ${TOKENS_URL}`]: () => jsonResponse([]),
    [`GET ${customerUrl('acme')}/usage?days=30`]: () => jsonResponse(USAGE),
    [`GET ${REPOSITORIES_URL}`]: () => jsonResponse(REPOSITORIES),
    [`GET ${USERS_URL}`]: () => jsonResponse(USERS),
    ...extra,
  });
}

async function openDialog() {
  await userEvent.click(await screen.findByRole('button', { name: 'Issue Token' }));
  return screen.findByRole('dialog', { name: 'Issue Token' });
}

async function chooseRepositories(names: string[]) {
  await userEvent.click(screen.getByRole('button', { name: 'Repositories' }));
  for (const name of names) {
    await userEvent.click(await screen.findByRole('option', { name: new RegExp(`^${name} `) }));
  }
  await userEvent.keyboard('{Escape}');
}

async function fillRequired(dialog: HTMLElement, repositories: string[]) {
  await userEvent.type(within(dialog).getByRole('textbox', { name: 'Name' }), 'Build');
  await userEvent.type(within(dialog).getByRole('combobox', { name: 'Owner' }), 'deploy');
  if (repositories.length > 0) {
    await chooseRepositories(repositories);
  }
  await userEvent.click(within(dialog).getByRole('combobox', { name: 'Expires in' }));
  await userEvent.click(await screen.findByRole('option', { name: '90 days' }));
}

describe('the issue token dialog', () => {
  it('needs the create privilege to show the button', async () => {
    vi.stubGlobal('fetch', server().fetch);
    renderDetail(['astrum:customers:read']);
    expect(await screen.findByText('This customer has no tokens.')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Issue Token' })).not.toBeInTheDocument();
  });

  it('asks for at least one repository and sends nothing', async () => {
    const fake = server();
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    const dialog = await openDialog();
    await fillRequired(dialog, []);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Issue' }));
    expect(await within(dialog).findByText('Choose at least one repository')).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'POST')).toBe(false);
  });

  it('warns when a group is among the chosen repositories', async () => {
    vi.stubGlobal('fetch', server().fetch);
    renderDetail();
    const dialog = await openDialog();
    expect(within(dialog).queryByText(/A group gives access/)).not.toBeInTheDocument();
    await chooseRepositories(['bundle']);
    expect(
      await within(dialog).findByText(
        'A group gives access to everything in its members, now and after later changes.',
      ),
    ).toBeInTheDocument();
  });

  it('shows the token once with a line per format, then clears it and refreshes the table', async () => {
    let tokens: unknown[] = [];
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${TOKENS_URL}`]: () => jsonResponse(tokens),
      [`GET ${customerUrl('acme')}/usage?days=30`]: () => jsonResponse(USAGE),
      [`GET ${REPOSITORIES_URL}`]: () => jsonResponse(REPOSITORIES),
      [`GET ${USERS_URL}`]: () => jsonResponse(USERS),
      [`POST ${TOKENS_URL}`]: () => {
        tokens = [
          {
            ...ISSUED,
            createdAt: '2026-11-11T10:00:00Z',
            lastUsedAt: null,
            lastUsedIp: null,
            createdBy: 'admin',
            expiryNotice: null,
          },
        ];
        return jsonResponse(ISSUED, 201);
      },
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    const dialog = await openDialog();
    await fillRequired(dialog, ['releases', 'charts', 'mirror', 'files']);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Issue' }));
    expect(await within(dialog).findByText('This token is shown only once')).toBeInTheDocument();
    expect(within(dialog).getByText('astrum_secret_value')).toBeInTheDocument();
    expect(within(dialog).getByText('docker login -u acme localhost')).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        'helm repo add charts http://localhost/repository/charts/ --username acme --password astrum_secret_value',
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText('//localhost/repository/mirror/:_authToken=astrum_secret_value'),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        'curl -u acme:astrum_secret_value -fLO http://localhost/repository/files/<path>',
      ),
    ).toBeInTheDocument();
    expect(fake.calls.find((call) => call.method === 'POST')).toMatchObject({
      body: {
        name: 'Build',
        owner: 'deploy',
        repositories: ['releases', 'charts', 'mirror', 'files'],
        expiresInDays: 90,
      },
    });
    const listed = fake.calls.filter((call) => call.method === 'GET' && call.path === TOKENS_URL).length;
    await userEvent.click(within(dialog).getByRole('button', { name: 'Done' }));
    await waitFor(() => expect(screen.queryByText('astrum_secret_value')).not.toBeInTheDocument());
    expect(await screen.findByText('Build')).toBeInTheDocument();
    expect(
      fake.calls.filter((call) => call.method === 'GET' && call.path === TOKENS_URL).length,
    ).toBeGreaterThan(listed);
  });

  it('puts server field errors on their controls and a refusal at the top', async () => {
    let refuse = false;
    const fake = server({
      [`POST ${TOKENS_URL}`]: () =>
        refuse
          ? jsonResponse('the repository releases is not readable', 403)
          : jsonResponse(
              [
                { id: 'owner', message: 'enable an existing local user' },
                { id: 'expiresInDays', message: 'must be given, from 1 to 3650 days' },
              ],
              400,
            ),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    const dialog = await openDialog();
    await fillRequired(dialog, ['releases']);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Issue' }));
    expect(await within(dialog).findByText('enable an existing local user')).toBeInTheDocument();
    expect(within(dialog).getByRole('combobox', { name: 'Owner' })).toHaveAttribute('aria-invalid', 'true');
    expect(within(dialog).getByText('must be given, from 1 to 3650 days')).toBeInTheDocument();
    refuse = true;
    await userEvent.click(within(dialog).getByRole('button', { name: 'Issue' }));
    expect(await within(dialog).findByText('the repository releases is not readable')).toBeInTheDocument();
  });

  it('takes the owner suggestions from the enabled local users', async () => {
    vi.stubGlobal('fetch', server().fetch);
    renderDetail();
    const dialog = await openDialog();
    await waitFor(() => expect(dialog.querySelectorAll('datalist option')).toHaveLength(1));
    expect(dialog.querySelector('datalist option')?.getAttribute('value')).toBe('deploy');
  });

  it('keeps the owner a plain text input without users:read', async () => {
    const fake = server();
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail(['astrum:customers:read', 'astrum:customers:create']);
    const dialog = await openDialog();
    await userEvent.type(within(dialog).getByRole('combobox', { name: 'Owner' }), 'anyone');
    expect(within(dialog).getByRole('combobox', { name: 'Owner' })).toHaveValue('anyone');
    expect(dialog.querySelectorAll('datalist option')).toHaveLength(0);
    expect(fake.calls.some((call) => call.path === USERS_URL)).toBe(false);
  });
});
