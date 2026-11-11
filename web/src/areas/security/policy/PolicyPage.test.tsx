import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../../test/http';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Policy } from './model';
import PolicyPage from './PolicyPage';

const STRICT: Policy = {
  name: 'strict',
  notes: '',
  vulnerabilities: { mode: 'report', severity: 'high', malicious: 'block', unscanned: 'allow', notify: null },
  licenses: { mode: 'off', list: null, ids: [], unknown: 'report' },
  signatures: { mode: 'off', keys: [] },
  revision: 3,
};

const EDIT_PATH = '/admin/security/policies/strict';
const EDIT_ROUTES = [
  { path: '/admin/security/policies/create', element: <PolicyPage /> },
  { path: '/admin/security/policies/:name', element: <PolicyPage /> },
];

function editing(handlers: Record<string, Handler> = {}) {
  return fakeServer({
    'GET /service/rest/v1/security/policies/strict': () => jsonResponse(STRICT),
    'GET /service/rest/v1/security/keys': () => jsonResponse([]),
    ...handlers,
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the policy page', () => {
  it('creates a policy with its three blocks and the name', async () => {
    const server = fakeServer({
      'POST /service/rest/v1/security/policies': () => jsonResponse(STRICT, 201),
      'GET /service/rest/v1/security/policies/strict': () => jsonResponse(STRICT),
      'GET /service/rest/v1/security/keys': () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, { path: '/admin/security/policies/create', routes: EDIT_ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: 'Name' }), 'strict');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Policy strict created')).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'strict',
      notes: '',
      vulnerabilities: { mode: 'off', severity: null, malicious: 'block', unscanned: 'allow', notify: null },
      licenses: { mode: 'off', list: null, ids: [], unknown: 'report' },
      signatures: { mode: 'off', keys: [] },
    });
  });

  it('asks for a name before it sends anything', async () => {
    const server = fakeServer({ 'GET /service/rest/v1/security/keys': () => jsonResponse([]) });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, { path: '/admin/security/policies/create', routes: EDIT_ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: 'Notes' }), 'draft');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Fill this in')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'POST')).toBe(false);
  });

  it('asks before a mode becomes block and keeps the form when the admin cancels', async () => {
    const server = editing({
      'PUT /service/rest/v1/security/policies/strict': () => jsonResponse(STRICT),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, { path: EDIT_PATH, routes: EDIT_ROUTES });
    const [vulnerabilityMode] = await screen.findAllByRole('combobox', { name: 'Mode' });
    await userEvent.click(vulnerabilityMode as HTMLElement);
    await userEvent.click(screen.getByRole('option', { name: 'Block' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(
      within(dialog).getByText(
        'Downloads of packages that this policy blocks will be refused as soon as you save.',
      ),
    ).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Save and Block' }),
    );
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
      vulnerabilities: { mode: 'block' },
    });
  });

  it('shows the refusal of a policy in use as the server wrote it', async () => {
    const server = editing({
      'DELETE /service/rest/v1/security/policies/strict': () =>
        jsonResponse(
          { id: '*', message: 'the content policy strict is used by repositories: docs-a, docs-b' },
          409,
        ),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, { path: EDIT_PATH, routes: EDIT_ROUTES });
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Delete' }),
    );
    expect(
      await screen.findByText('the content policy strict is used by repositories: docs-a, docs-b'),
    ).toBeInTheDocument();
  });

  it('shows a validation error from the server next to the buttons', async () => {
    const server = editing({
      'PUT /service/rest/v1/security/policies/strict': () =>
        jsonResponse(
          [
            {
              id: 'licenses.ids',
              message: 'must hold at least one id when the mode is block and the list is allow',
            },
          ],
          400,
        ),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, { path: EDIT_PATH, routes: EDIT_ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: 'Notes' }), 'weekly review');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(
      await screen.findByText('must hold at least one id when the mode is block and the list is allow'),
    ).toBeInTheDocument();
  });

  it('disables the fields and the save for a reader', async () => {
    const server = editing();
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PolicyPage />, {
      path: EDIT_PATH,
      routes: EDIT_ROUTES,
      state: { ...ADMIN_STATE, permissions: ['astrum:policies:read'] },
    });
    expect(await screen.findByRole('textbox', { name: 'Notes' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
  });
});
