import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { TrustedKey } from './model';
import TrustedKeysPage from './TrustedKeysPage';

const PEM =
  '-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAGb9ECWmEzf6FQbrBZ9w7lshQhqowtrbLDFw4rXAxZuE=\n-----END PUBLIC KEY-----';

const RELEASE: TrustedKey = {
  name: 'release',
  algorithm: 'ed25519',
  publicKey: PEM,
  fingerprint: 'SHA256:9f1c2d3e4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f',
  notes: '',
  createdAt: '2026-11-01T00:00:00Z',
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the trusted keys page', () => {
  it('lists the keys with their algorithm and fingerprint', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/keys': () => jsonResponse([RELEASE]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<TrustedKeysPage />);
    const row = (await screen.findByText('release')).closest('tr') as HTMLElement;
    expect(within(row).getByText('ed25519')).toBeInTheDocument();
    expect(within(row).getByText(RELEASE.fingerprint)).toBeInTheDocument();
  });

  it('adds a key from its PEM and shows the fingerprint after saving', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/keys': () => jsonResponse([]),
      'POST /service/rest/v1/security/keys': () => jsonResponse(RELEASE, 201),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<TrustedKeysPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add Key' }));
    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Name' }), 'release');
    fireEvent.change(within(dialog).getByRole('textbox', { name: /Public key/ }), { target: { value: PEM } });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Key' }));
    expect(await screen.findByText(RELEASE.fingerprint)).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'release',
      publicKey: PEM,
      notes: '',
    });
  });

  it('shows the validation error of the PEM on its field', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/keys': () => jsonResponse([]),
      'POST /service/rest/v1/security/keys': () =>
        jsonResponse([{ id: 'publicKey', message: 'the public key must be PEM text' }], 400),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<TrustedKeysPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add Key' }));
    const dialog = await screen.findByRole('dialog');
    await userEvent.type(within(dialog).getByRole('textbox', { name: 'Name' }), 'release');
    fireEvent.change(within(dialog).getByRole('textbox', { name: /Public key/ }), {
      target: { value: 'nope' },
    });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Add Key' }));
    expect(await within(dialog).findByText('the public key must be PEM text')).toBeInTheDocument();
  });

  it('deletes a key after the admin confirms it', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/keys': () => jsonResponse([RELEASE]),
      'DELETE /service/rest/v1/security/keys/release': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<TrustedKeysPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Delete' }),
    );
    await waitFor(() => expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true));
  });
});
