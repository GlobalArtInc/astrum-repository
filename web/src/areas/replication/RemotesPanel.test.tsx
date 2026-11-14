import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { REMOTES, type ReplicationRemote } from './api';
import { RemotesPanel } from './RemotesPanel';

afterEach(() => {
  vi.unstubAllGlobals();
});

const REMOTE: ReplicationRemote = {
  id: 'r1',
  name: 'Mirror',
  url: 'https://mirror.example.com',
  hasToken: true,
  useTrustStore: false,
  allowPrivateNetwork: false,
  maxConcurrency: 2,
  maxBytesPerSecond: null,
  remoteInstanceId: null,
  remoteVersion: '3.0.4',
  checkedAt: null,
  createdAt: '2026-11-01T10:00:00Z',
  updatedAt: '2026-11-01T10:00:00Z',
};

const REMOTE_PATH = `${REMOTES}/r1`;

describe('the remotes panel', () => {
  it('checks the connection and shows the answer', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
      [`POST ${REMOTE_PATH}/check`]: () =>
        jsonResponse({
          ok: true,
          protocol: 1,
          instanceId: 'i1',
          version: '3.0.4',
          formats: ['raw', 'maven2'],
        }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    expect(await screen.findByText('Mirror')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Check connection' }));
    expect(await screen.findByText('Mirror answered: protocol 1, Astrum 3.0.4')).toBeInTheDocument();
    expect(screen.getByText('Formats: raw, maven2')).toBeInTheDocument();
  });

  it('shows the error kind and message when the check fails', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
      [`POST ${REMOTE_PATH}/check`]: () =>
        jsonResponse({ ok: false, error: { kind: 'refused', message: 'the address is refused' } }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Check connection' }));
    expect(
      await screen.findByText('Mirror did not answer (refused): the address is refused'),
    ).toBeInTheDocument();
  });

  it('creates a remote and maps server errors onto their fields', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([]),
      [`POST ${REMOTES}`]: () =>
        jsonResponse(
          [
            { id: 'url', message: 'the address resolves to a refused network' },
            { id: 'maxBytesPerSecond', message: 'must be above 0, or null for no limit' },
          ],
          400,
        ),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add remote' }));
    const dialog = await screen.findByRole('dialog', { name: 'New remote' });
    await userEvent.type(within(dialog).getByLabelText('Name'), 'Mirror');
    await userEvent.type(within(dialog).getByLabelText('URL'), 'https://mirror.example.com');
    await userEvent.type(within(dialog).getByLabelText(/^Token/), 'token-1');
    await userEvent.type(within(dialog).getByLabelText('Bandwidth limit, MiB/s'), '2');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(await within(dialog).findByText('the address resolves to a refused network')).toBeInTheDocument();
    expect(within(dialog).getByText('must be above 0, or null for no limit')).toBeInTheDocument();
    const post = fake.calls.find((call) => call.method === 'POST');
    expect(post?.body).toMatchObject({
      name: 'Mirror',
      maxConcurrency: 2,
      maxBytesPerSecond: 2097152,
      token: 'token-1',
    });
  });

  it('keeps the stored token when only the name changes', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
      [`PUT ${REMOTE_PATH}`]: () => jsonResponse({ ...REMOTE, name: 'Mirror 2' }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Edit remote Mirror' }));
    const dialog = await screen.findByRole('dialog', { name: 'Edit remote' });
    expect(within(dialog).getByText('Token is set, leave empty to keep it')).toBeInTheDocument();
    const name = within(dialog).getByLabelText('Name');
    await userEvent.clear(name);
    await userEvent.type(name, 'Mirror 2');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    const put = fake.calls.find((call) => call.method === 'PUT');
    expect(put?.body).toMatchObject({ name: 'Mirror 2', url: 'https://mirror.example.com' });
    expect(put?.body).not.toHaveProperty('token');
  });

  it('asks for the token again when the URL changes', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
      [`PUT ${REMOTE_PATH}`]: () => jsonResponse({ ...REMOTE, url: 'https://other.example.com' }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Edit remote Mirror' }));
    const dialog = await screen.findByRole('dialog', { name: 'Edit remote' });
    const url = within(dialog).getByLabelText('URL');
    await userEvent.clear(url);
    await userEvent.type(url, 'https://other.example.com');
    expect(
      within(dialog).getByText('The URL changed, so the token must be entered again'),
    ).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(
      await within(dialog).findByText('Enter the token: the URL is new or the token is not stored'),
    ).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'PUT')).toBe(false);
    await userEvent.type(within(dialog).getByLabelText(/^Token/), 'token-2');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(fake.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({ token: 'token-2' });
  });

  it('shows the replications that still use a remote it cannot delete', async () => {
    const fake = fakeServer({
      [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
      [`DELETE ${REMOTE_PATH}`]: () =>
        jsonResponse({ id: '*', message: 'The remote is used by these replications: Nightly, Weekly' }, 409),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<RemotesPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Delete remote Mirror' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(
      await within(dialog).findByText('The remote is used by these replications: Nightly, Weekly'),
    ).toBeInTheDocument();
  });
});
