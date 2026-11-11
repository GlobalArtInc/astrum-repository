import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { type Customer, type CustomerToken, customerUrl, tokensUrl } from './api';
import CustomerPage from './CustomerPage';

afterEach(() => {
  vi.unstubAllGlobals();
});

const ACME: Customer = {
  id: 'acme',
  name: 'Acme',
  reference: 'CT-1',
  note: 'Renewal in March',
  disabled: false,
  createdAt: '2026-10-01T10:00:00Z',
  createdBy: 'admin',
  tokens: 2,
  nearestExpiry: null,
  downloads30d: 0,
  bytes30d: 0,
  lastAccessAt: null,
};

function token(id: string, name: string): CustomerToken {
  return {
    id,
    name,
    hint: `astrum_${id}`,
    createdAt: '2026-10-02T10:00:00Z',
    expiresAt: null,
    lastUsedAt: null,
    lastUsedIp: null,
    owner: 'deploy',
    repositories: ['releases', 'nightly'],
    createdBy: 'admin',
    expiryNotice: null,
  };
}

const TOKENS = [token('tok-1', 'Nightly'), token('tok-2', 'Weekly')];

function renderDetail(permissions?: string[]) {
  const state = permissions ? { ...ADMIN_STATE, permissions } : ADMIN_STATE;
  return renderWithApp(<CustomerPage />, {
    state,
    path: '/admin/customers/acme',
    routes: [
      { path: '/admin/customers', element: <p>customers list</p> },
      { path: '/admin/customers/:id', element: <CustomerPage /> },
    ],
  });
}

describe('the customer page', () => {
  it('sends all four fields when the form is saved', async () => {
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse(TOKENS),
      [`PUT ${customerUrl('acme')}`]: () => jsonResponse({ ...ACME, name: 'Acme Corp' }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    const name = await screen.findByLabelText(/^Name/);
    await userEvent.clear(name);
    await userEvent.type(name, 'Acme Corp');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'PUT')).toBe(true));
    const put = fake.calls.find((call) => call.method === 'PUT');
    expect(put?.body).toEqual({
      name: 'Acme Corp',
      reference: 'CT-1',
      note: 'Renewal in March',
      disabled: false,
    });
  });

  it('explains the disabled switch and saves it', async () => {
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse(TOKENS),
      [`PUT ${customerUrl('acme')}`]: () => jsonResponse({ ...ACME, disabled: true }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    expect(await screen.findByText("A disabled customer's tokens stop working at once.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole('switch'));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({ disabled: true });
  });

  it('revokes one token after the confirmation and refreshes the table', async () => {
    let tokens = TOKENS;
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse(tokens),
      [`DELETE ${tokensUrl('acme')}/tok-1`]: () => {
        tokens = [TOKENS[1] as CustomerToken];
        return new Response(null, { status: 204 });
      },
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click((await screen.findAllByRole('button', { name: 'Actions' }))[0] as HTMLElement);
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Revoke' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Revoke' }));
    await waitFor(() => expect(screen.queryByText('Nightly')).not.toBeInTheDocument());
    expect(fake.calls.filter((call) => call.method === 'DELETE')).toEqual([
      { method: 'DELETE', path: `${tokensUrl('acme')}/tok-1`, body: undefined },
    ]);
  });

  it('names the number of tokens in the revoke-all confirmation', async () => {
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse(TOKENS),
      [`POST ${customerUrl('acme')}/revoke`]: () => jsonResponse({ revoked: 2 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click(await screen.findByRole('button', { name: 'Revoke All Tokens' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(/^2 tokens of this customer stop working/)).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Revoke All Tokens' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(fake.calls.filter((call) => call.method === 'POST')).toHaveLength(1);
  });

  it('deletes the customer only after its id is typed', async () => {
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse([]),
      [`DELETE ${customerUrl('acme')}`]: () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    const { router } = renderDetail();
    await userEvent.click(await screen.findByRole('button', { name: 'Delete Customer' }));
    const dialog = await screen.findByRole('alertdialog');
    const confirm = within(dialog).getByRole('button', { name: 'Delete' });
    expect(confirm).toBeDisabled();
    await userEvent.type(within(dialog).getByLabelText('Customer ID'), 'acme');
    expect(confirm).toBeEnabled();
    await userEvent.click(confirm);
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/customers'));
    expect(fake.calls.filter((call) => call.method === 'DELETE')).toHaveLength(1);
  });

  it('hides the save, revoke and delete actions without their privileges', async () => {
    const fake = fakeServer({
      [`GET ${customerUrl('acme')}`]: () => jsonResponse(ACME),
      [`GET ${tokensUrl('acme')}`]: () => jsonResponse(TOKENS),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail(['astrum:customers:read']);
    expect(await screen.findByLabelText(/^Name/)).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Revoke All Tokens' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete Customer' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Actions' })).not.toBeInTheDocument();
  });
});
