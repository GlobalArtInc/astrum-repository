import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { CreatedShareLink } from '../../security/share-links/model';
import { ShareDialog } from './ShareDialog';

const SHARE_URL =
  'http://localhost/service/share/astrum_share_0123456789abcdefghijklmnopqrstuvwxyzABCD/app.tgz';

const LINK: CreatedShareLink = {
  id: '0b6c3f3e-0000-4000-8000-000000000001',
  repository: 'files',
  path: '/releases/app.tgz',
  prefix: false,
  note: 'for CI',
  createdBy: 'admin',
  createdAt: '2026-11-11T10:00:00.000Z',
  expiresAt: '2026-11-18T10:00:00.000Z',
  maxDownloads: 3,
  downloads: 0,
  bytesServed: 0,
  lastUsedAt: null,
  expired: false,
  url: SHARE_URL,
};

function Harness({ prefix = false }: { prefix?: boolean }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Open share
      </button>
      <ShareDialog
        open={open}
        onOpenChange={setOpen}
        repository="files"
        path={prefix ? '/releases/' : '/releases/app.tgz'}
        prefix={prefix}
      />
    </>
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the share dialog', () => {
  it('submits the chosen lifetime and limit, shows the SHARE_URL once and forgets it on reopen', async () => {
    const server = fakeServer({
      'POST /service/rest/v1/share-links': () => jsonResponse(LINK, 201),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<Harness />);
    await userEvent.click(screen.getByRole('button', { name: 'Open share' }));
    const dialog = await screen.findByRole('dialog', { name: 'Create a share link' });
    await userEvent.type(within(dialog).getByRole('spinbutton', { name: 'Download limit' }), '3');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Note' }), 'for CI');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create link' }));
    expect(await within(dialog).findByTestId('share-url')).toHaveTextContent(SHARE_URL);
    expect(within(dialog).getByText(`curl -fLO '${SHARE_URL}'`)).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      repository: 'files',
      path: '/releases/app.tgz',
      prefix: false,
      expiresInHours: 168,
      maxDownloads: 3,
      note: 'for CI',
    });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Done' }));
    expect(screen.queryByText(SHARE_URL)).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Open share' }));
    expect(await screen.findByRole('dialog', { name: 'Create a share link' })).toBeInTheDocument();
    expect(screen.queryByText(SHARE_URL)).toBeNull();
    expect(screen.getByRole('button', { name: 'Create link' })).toBeInTheDocument();
  });

  it('hides the download limit for a folder and sends a prefix link', async () => {
    const server = fakeServer({
      'POST /service/rest/v1/share-links': () =>
        jsonResponse(
          { ...LINK, prefix: true, path: '/releases/', maxDownloads: null, url: `${SHARE_URL}/` },
          201,
        ),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<Harness prefix />);
    await userEvent.click(screen.getByRole('button', { name: 'Open share' }));
    const dialog = await screen.findByRole('dialog', { name: 'Create a folder share link' });
    expect(within(dialog).queryByRole('spinbutton', { name: 'Download limit' })).toBeNull();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create link' }));
    expect(await within(dialog).findByTestId('share-url')).toHaveTextContent(`${SHARE_URL}/`);
    expect(within(dialog).getByText(/opens an index of the files under the folder/)).toBeInTheDocument();
    expect(within(dialog).queryByText(/curl -fLO/)).toBeNull();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      repository: 'files',
      path: '/releases/',
      prefix: true,
      expiresInHours: 168,
    });
  });

  it('shows the message the REST answers with', async () => {
    const server = fakeServer({
      'POST /service/rest/v1/share-links': () =>
        jsonResponse({ id: '*', message: 'must be between 1 and 720 (the setting shareLinks.maxDays)' }, 400),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<Harness />);
    await userEvent.click(screen.getByRole('button', { name: 'Open share' }));
    const dialog = await screen.findByRole('dialog', { name: 'Create a share link' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Create link' }));
    expect(
      await within(dialog).findByText('must be between 1 and 720 (the setting shareLinks.maxDays)'),
    ).toBeInTheDocument();
  });
});
