import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../test/http';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import SecurityPage from './SecurityPage';
import type { Passkey, TwoFactorStatus } from './twoFactor';

const BASE = '/service/rest/internal/ui/user/two-factor';
const PASSWORD = 'pw-1';
const CODES = ['AAAA-1111', 'BBBB-2222'];
const STATUS: TwoFactorStatus = {
  enrolled: false,
  totp: false,
  required: false,
  requirementSource: 'none',
  enrollBy: null,
  restricted: false,
  recoveryCodesLeft: 0,
  lockedUntil: null,
  accessTokens: 0,
  methods: ['passkey'],
  passkeys: [],
  passkeysAvailable: true,
};
const LAPTOP: Passkey = {
  id: 'pk1',
  name: 'Laptop',
  createdAt: '2026-11-01T10:00:00Z',
  lastUsedAt: null,
  backedUp: true,
  usable: true,
};
const OTHER: Passkey = { ...LAPTOP, id: 'pk2', name: 'Old key', backedUp: false, usable: false };
const OPTIONS = {
  ceremony: 'p1',
  publicKey: { challenge: 'AQI', user: { id: 'AQI', name: 'admin', displayName: 'Administrator' } },
  expiresAt: '2026-11-11T10:05:00Z',
};

function bytes(...values: number[]): ArrayBuffer {
  return new Uint8Array(values).buffer;
}

function registration() {
  return {
    id: 'AQI',
    rawId: bytes(1, 2),
    type: 'public-key',
    response: { attestationObject: bytes(3, 4), clientDataJSON: bytes(5, 6) },
    getClientExtensionResults: () => ({}),
  };
}

function serverFor(status: TwoFactorStatus, extra: Record<string, Handler> = {}) {
  const server = fakeServer({
    [`GET ${BASE}`]: () => jsonResponse(status),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function supportPasskeys(create: () => Promise<unknown>) {
  vi.stubGlobal('PublicKeyCredential', class {});
  const credentials = { create: vi.fn(create), get: vi.fn() };
  Object.defineProperty(navigator, 'credentials', { configurable: true, value: credentials });
  return credentials;
}

beforeEach(() => {
  supportPasskeys(async () => registration());
});

afterEach(() => {
  vi.unstubAllGlobals();
  Reflect.deleteProperty(navigator, 'credentials');
});

async function openAdd(): Promise<HTMLElement> {
  await userEvent.click(await screen.findByRole('button', { name: 'Add a Passkey' }));
  return screen.findByRole('dialog', { name: 'Add a Passkey' });
}

describe('the passkeys section', () => {
  it('lists a passkey of another address with the not-usable badge and its hint', async () => {
    serverFor({ ...STATUS, passkeys: [LAPTOP, OTHER] });
    renderWithApp(<SecurityPage />);
    expect(await screen.findByText('Old key')).toBeInTheDocument();
    expect(screen.getByText('Not usable here')).toBeInTheDocument();
    expect(screen.getByText('Backed up')).toBeInTheDocument();
    expect(
      screen.getByText(
        'It was made for another address of this server, so it cannot be used here. Open the address it was made for to use it.',
      ),
    ).toBeInTheDocument();
  });

  it('explains instead of offering passkeys when no relying party is usable', async () => {
    serverFor({ ...STATUS, passkeysAvailable: false });
    renderWithApp(<SecurityPage />);
    expect(
      await screen.findByText(
        'Passkeys are not available here: this server has no base URL to make them for. Ask an administrator to set one.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Add a Passkey' })).toBeNull();
  });

  it('is absent when the browser has no WebAuthn', async () => {
    vi.unstubAllGlobals();
    serverFor(STATUS);
    renderWithApp(<SecurityPage />);
    expect(await screen.findByRole('button', { name: 'Set Up' })).toBeInTheDocument();
    expect(screen.queryByText('Passkeys')).toBeNull();
  });

  it('asks the name and password, creates the passkey and shows the recovery codes it returns', async () => {
    const server = serverFor(STATUS, {
      [`POST ${BASE}/passkeys/options`]: () => jsonResponse(OPTIONS),
      [`POST ${BASE}/passkeys`]: () => jsonResponse({ passkey: LAPTOP, recoveryCodes: CODES }),
    });
    renderWithApp(<SecurityPage />);
    const dialog = await openAdd();
    await userEvent.type(within(dialog).getByLabelText('Passkey Name'), 'Laptop');
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    expect(await within(dialog).findByText(CODES[0] ?? '')).toBeInTheDocument();
    expect(server.calls.find((call) => call.path.endsWith('/passkeys/options'))?.body).toEqual({
      currentPassword: PASSWORD,
      name: 'Laptop',
    });
    expect(
      server.calls.find((call) => call.method === 'POST' && call.path.endsWith('/passkeys'))?.body,
    ).toEqual({
      ceremony: 'p1',
      credential: {
        id: 'AQI',
        rawId: 'AQI',
        type: 'public-key',
        response: { attestationObject: 'AwQ', clientDataJSON: 'BQY' },
        clientExtensionResults: {},
      },
      currentPassword: PASSWORD,
    });
  });

  it('closes quietly when the browser prompt is cancelled', async () => {
    supportPasskeys(() => Promise.reject(new DOMException('refused', 'NotAllowedError')));
    serverFor(STATUS, { [`POST ${BASE}/passkeys/options`]: () => jsonResponse(OPTIONS) });
    renderWithApp(<SecurityPage />);
    const dialog = await openAdd();
    await userEvent.type(within(dialog).getByLabelText('Passkey Name'), 'Laptop');
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    expect(
      await within(dialog).findByText('The browser prompt was closed. No passkey was added.'),
    ).toBeInTheDocument();
    expect(within(dialog).queryByRole('alert')).toBeNull();
  });

  it('shows a wrong password under the password field', async () => {
    serverFor(STATUS, {
      [`POST ${BASE}/passkeys/options`]: () =>
        jsonResponse([{ id: 'currentPassword', message: 'the current password is wrong' }], 400),
    });
    renderWithApp(<SecurityPage />);
    const dialog = await openAdd();
    await userEvent.type(within(dialog).getByLabelText('Passkey Name'), 'Laptop');
    await userEvent.type(within(dialog).getByLabelText('Current Password'), 'wrong');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    expect(await within(dialog).findByText('the current password is wrong')).toBeInTheDocument();
    expect(within(dialog).getByLabelText('Current Password')).toHaveAttribute('aria-invalid', 'true');
  });

  it('renames a passkey with a PATCH carrying the new name', async () => {
    const server = serverFor(
      { ...STATUS, passkeys: [LAPTOP] },
      {
        [`PATCH ${BASE}/passkeys/pk1`]: () => new Response(null, { status: 204 }),
      },
    );
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Rename Laptop' }));
    const dialog = await screen.findByRole('dialog', { name: 'Rename Passkey' });
    const name = within(dialog).getByLabelText('Passkey Name');
    await userEvent.clear(name);
    await userEvent.type(name, 'Work laptop');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Rename' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(server.calls.find((call) => call.method === 'PATCH')?.body).toEqual({ name: 'Work laptop' });
  });

  it('shows the server refusal when the last factor cannot be removed', async () => {
    const refusal = 'a second factor is required for this account; add another before removing this one';
    const server = serverFor(
      { ...STATUS, passkeys: [LAPTOP] },
      {
        [`DELETE ${BASE}/passkeys/pk1`]: () => jsonResponse({ id: '*', message: refusal }, 400),
      },
    );
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Remove Laptop' }));
    const dialog = await screen.findByRole('dialog', { name: 'Remove Passkey' });
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Remove' }));
    expect(await within(dialog).findByText(refusal)).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'DELETE')?.body).toEqual({
      currentPassword: PASSWORD,
    });
  });
});

describe('the passkey limits', () => {
  it('keeps stored passkeys listed, renameable and removable when no relying party is usable', async () => {
    serverFor({ ...STATUS, passkeysAvailable: false, passkeys: [OTHER] });
    renderWithApp(<SecurityPage />);
    expect(await screen.findByText('Old key')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Rename Old key' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Remove Old key' })).toBeInTheDocument();
    expect(
      screen.getByText(
        'Passkeys are not available here: this server has no base URL to make them for. Ask an administrator to set one.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Add a Passkey' })).toBeNull();
  });

  it('offers no Add when the policy turns passkeys off, and keeps the list manageable', async () => {
    serverFor({ ...STATUS, methods: ['totp'], passkeys: [LAPTOP] });
    renderWithApp(<SecurityPage />);
    expect(
      await screen.findByText('An administrator turned passkeys off on this server.'),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Add a Passkey' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Rename Laptop' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Remove Laptop' })).toBeInTheDocument();
  });
});
