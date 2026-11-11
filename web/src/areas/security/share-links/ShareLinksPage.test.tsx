import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { ShareLink } from './model';
import ShareLinksPage from './ShareLinksPage';

const MINE: ShareLink = {
  id: '0b6c3f3e-0000-4000-8000-000000000001',
  repository: 'files',
  path: '/releases/app.tgz',
  prefix: false,
  note: 'for CI',
  createdBy: 'admin',
  createdAt: '2026-11-11T10:00:00.000Z',
  expiresAt: '2099-01-01T00:00:00.000Z',
  maxDownloads: 5,
  downloads: 2,
  bytesServed: 10,
  lastUsedAt: null,
  expired: false,
};

const OLD: ShareLink = {
  ...MINE,
  id: '0b6c3f3e-0000-4000-8000-000000000002',
  path: '/releases/old.tgz',
  note: '',
  maxDownloads: null,
  expiresAt: '2020-01-01T00:00:00.000Z',
  expired: true,
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the share links page', () => {
  it('lists the links and revokes one after confirmation', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/share-links': () => jsonResponse([MINE, OLD]),
      [`DELETE /service/rest/v1/share-links/${MINE.id}`]: () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<ShareLinksPage />);
    expect(await screen.findByText('/releases/app.tgz')).toBeInTheDocument();
    expect(screen.getByText('Expired')).toBeInTheDocument();
    expect(screen.getByText('Up to 5 copies')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Revoke the link to /releases/app.tgz' }));
    const confirm = await screen.findByRole('alertdialog', { name: 'Revoke this share link?' });
    expect(within(confirm).getByText(/stops working at once/)).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'DELETE')).toBe(false);
    await userEvent.click(within(confirm).getByRole('button', { name: 'Revoke' }));
    await waitFor(() =>
      expect(
        server.calls.some(
          (call) => call.method === 'DELETE' && call.path === `/service/rest/v1/share-links/${MINE.id}`,
        ),
      ).toBe(true),
    );
  });
});
