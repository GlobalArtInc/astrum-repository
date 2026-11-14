import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import ExceptionsPage from './ExceptionsPage';
import type { PolicyException } from './exceptions';
import { DAY } from './model';

const LIVE: PolicyException = {
  id: '0c0d6d7e-1f2a-4b3c-8d4e-5f6a7b8c9d01',
  policy: 'strict',
  rule: 'vulnerability',
  advisory: 'GHSA-aaaa-bbbb-cccc',
  license: null,
  format: null,
  namespace: null,
  name: null,
  version: null,
  digest: null,
  reason: 'vendor patch in review',
  expiresAt: new Date(Date.now() + 30 * DAY).toISOString(),
  createdBy: 'admin',
  createdAt: '2026-11-01T00:00:00Z',
};

const OLD: PolicyException = {
  ...LIVE,
  id: '0c0d6d7e-1f2a-4b3c-8d4e-5f6a7b8c9d02',
  advisory: 'GHSA-dddd-eeee-ffff',
  reason: 'accepted last quarter',
  expiresAt: '2026-10-01T00:00:00Z',
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the policy exceptions page', () => {
  it('marks an exception whose end has passed as expired', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/exceptions': () => jsonResponse([LIVE, OLD]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ExceptionsPage />);
    const expiredRow = (await screen.findByText('accepted last quarter')).closest('tr') as HTMLElement;
    expect(within(expiredRow).getByText('Expired')).toBeInTheDocument();
    const liveRow = screen.getByText('vendor patch in review').closest('tr') as HTMLElement;
    expect(within(liveRow).queryByText('Expired')).not.toBeInTheDocument();
  });

  it('asks for the subject and the reason before it sends anything', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/exceptions': () => jsonResponse([]),
      'GET /service/rest/v1/security/policies': () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ExceptionsPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add Exception' }));
    const dialog = await screen.findByRole('dialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Exception' }));
    expect(await within(dialog).findByText('Fill this in')).toBeInTheDocument();
    expect(within(dialog).getByText('Give the subject of the exception')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'POST')).toBe(false);
  });

  it('creates an exception that ends in seven days from now', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/exceptions': () => jsonResponse([]),
      'GET /service/rest/v1/security/policies': () => jsonResponse([]),
      'POST /service/rest/v1/security/exceptions': () => jsonResponse(LIVE, 201),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ExceptionsPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add Exception' }));
    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Advisory' }), 'GHSA-aaaa-bbbb-cccc');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Reason' }), 'vendor patch in review');
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Expiry' }));
    await userEvent.click(screen.getByRole('option', { name: '7 days' }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Exception' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    const body = server.calls.find((call) => call.method === 'POST')?.body as Record<string, unknown>;
    expect(body).toMatchObject({
      rule: 'vulnerability',
      advisory: 'GHSA-aaaa-bbbb-cccc',
      reason: 'vendor patch in review',
    });
    expect(body).not.toHaveProperty('policy');
    expect(Math.abs(Date.parse(body.expiresAt as string) - (Date.now() + 7 * DAY))).toBeLessThan(60_000);
  });

  it('sends no component of a typed component that the advisory subject hides', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/exceptions': () => jsonResponse([]),
      'GET /service/rest/v1/security/policies': () => jsonResponse([]),
      'POST /service/rest/v1/security/exceptions': () => jsonResponse(LIVE, 201),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ExceptionsPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add Exception' }));
    const dialog = await screen.findByRole('dialog');
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Subject' }));
    await userEvent.click(screen.getByRole('option', { name: 'Package or component' }));
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Name' }), 'left-pad');
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Subject' }));
    await userEvent.click(screen.getByRole('option', { name: 'Advisory' }));
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Advisory' }), 'GHSA-aaaa-bbbb-cccc');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Reason' }), 'vendor patch in review');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Exception' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    const body = server.calls.find((call) => call.method === 'POST')?.body as Record<string, unknown>;
    expect(body).not.toHaveProperty('name');
    expect(body.advisory).toBe('GHSA-aaaa-bbbb-cccc');
  });

  it('revokes an exception after the admin confirms it', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/exceptions': () => jsonResponse([LIVE]),
      [`DELETE /service/rest/v1/security/exceptions/${LIVE.id}`]: () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ExceptionsPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Revoke' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Revoke' }),
    );
    await waitFor(() => expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true));
    expect(server.calls.find((call) => call.method === 'DELETE')?.path).toBe(
      `/service/rest/v1/security/exceptions/${LIVE.id}`,
    );
  });
});
