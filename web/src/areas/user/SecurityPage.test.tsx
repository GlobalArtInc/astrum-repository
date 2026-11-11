import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../test/http';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import SecurityPage from './SecurityPage';
import { qrDataUri, type TwoFactorStatus } from './twoFactor';

const BASE = '/service/rest/internal/ui/user/two-factor';
const PASSWORD = 'pw-1';
const SVG = '<svg xmlns="http://www.w3.org/2000/svg"><rect width="4" height="4"/></svg>';
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
  methods: ['totp'],
};
const ENROLLED: TwoFactorStatus = { ...STATUS, enrolled: true, totp: true, recoveryCodesLeft: 8 };
const ENROLLMENT = {
  ceremony: 'c1',
  secret: 'JBSWY3DPEHPK3PXP',
  otpauthUri: 'otpauth://totp/Astrum:admin?secret=JBSWY3DPEHPK3PXP',
  qrSvg: SVG,
  expiresAt: '2026-11-11T10:05:00Z',
};

afterEach(() => {
  vi.unstubAllGlobals();
});

function serverFor(status: TwoFactorStatus, extra: Record<string, Handler> = {}) {
  const server = fakeServer({
    [`GET ${BASE}`]: () => jsonResponse(status),
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

async function enrollToCodes(): Promise<HTMLElement> {
  await userEvent.click(await screen.findByRole('button', { name: 'Set Up' }));
  const dialog = await screen.findByRole('dialog', { name: 'Set Up Authenticator App' });
  await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
  await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
  const qr = await within(dialog).findByRole('img', { name: 'QR code for the authenticator app' });
  expect(qr).toHaveAttribute('src', qrDataUri(SVG));
  expect(within(dialog).getByText('JBSW Y3DP EHPK 3PXP')).toBeInTheDocument();
  await userEvent.type(within(dialog).getByLabelText('Code'), '123456');
  await userEvent.click(within(dialog).getByRole('button', { name: 'Confirm' }));
  await within(dialog).findByText(CODES[0] ?? '');
  return dialog;
}

describe('the security page status', () => {
  it.each([
    ['not enrolled', {}, /Not set up/],
    ['enrolled', { enrolled: true, totp: true, recoveryCodesLeft: 8 }, /8 recovery codes left/],
    [
      'required with a deadline',
      { required: true, enrollBy: '2026-12-01T00:00:00Z' },
      /requires a second factor/,
    ],
    ['restricted', { restricted: true }, /session is limited until you set up/],
  ])('renders the %s state', async (_name, changes, expected) => {
    serverFor({ ...STATUS, ...changes });
    renderWithApp(<SecurityPage />);
    expect(await screen.findByText(expected)).toBeInTheDocument();
  });
});

describe('the security page', () => {
  it('shows the managed-by-provider panel instead of the controls when the server answers 400', async () => {
    serverFor(STATUS);
    vi.stubGlobal(
      'fetch',
      fakeServer({
        [`GET ${BASE}`]: () =>
          jsonResponse(
            [{ id: '*', message: 'the account comes from ldap; its second factor is managed there' }],
            400,
          ),
      }).fetch,
    );
    renderWithApp(<SecurityPage />);
    expect(
      await screen.findByText('Two-factor authentication is managed by your identity provider.'),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set Up' })).toBeNull();
  });

  it('sets up the app: password, QR code, code, then the recovery codes once with the confirm call', async () => {
    const server = serverFor(STATUS, {
      [`POST ${BASE}/totp`]: () => jsonResponse(ENROLLMENT),
      [`POST ${BASE}/totp/confirm`]: () => jsonResponse({ recoveryCodes: CODES }),
    });
    renderWithApp(<SecurityPage />);
    const dialog = await enrollToCodes();
    expect(within(dialog).getByText('These codes are shown only once')).toBeInTheDocument();
    expect(within(dialog).getByText(CODES[1] ?? '')).toBeInTheDocument();
    expect(server.calls.find((call) => call.path.endsWith('/totp'))?.body).toEqual({
      currentPassword: PASSWORD,
    });
    expect(server.calls.find((call) => call.path.endsWith('/totp/confirm'))?.body).toEqual({
      ceremony: 'c1',
      code: '123456',
      currentPassword: PASSWORD,
    });
  });

  it('shows a wrong password under the password field', async () => {
    serverFor(STATUS, {
      [`POST ${BASE}/totp`]: () =>
        jsonResponse([{ id: 'currentPassword', message: 'the current password is wrong' }], 400),
    });
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Set Up' }));
    const dialog = await screen.findByRole('dialog', { name: 'Set Up Authenticator App' });
    await userEvent.type(within(dialog).getByLabelText('Current Password'), 'wrong');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    expect(await within(dialog).findByText('the current password is wrong')).toBeInTheDocument();
    expect(within(dialog).getByLabelText('Current Password')).toHaveAttribute('aria-invalid', 'true');
  });

  it('shows a wrong code from the server and keeps the step', async () => {
    serverFor(STATUS, {
      [`POST ${BASE}/totp`]: () => jsonResponse(ENROLLMENT),
      [`POST ${BASE}/totp/confirm`]: () => jsonResponse([{ id: 'code', message: 'the code is wrong' }], 400),
    });
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Set Up' }));
    const dialog = await screen.findByRole('dialog', { name: 'Set Up Authenticator App' });
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Continue' }));
    await userEvent.type(await within(dialog).findByLabelText('Code'), '000000');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Confirm' }));
    expect(await within(dialog).findByText('the code is wrong')).toBeInTheDocument();
    expect(within(dialog).getByLabelText('Code')).toBeInTheDocument();
    expect(within(dialog).queryByText(CODES[0] ?? '')).toBeNull();
  });

  it('drops the recovery codes when the dialog closes', async () => {
    serverFor(STATUS, {
      [`POST ${BASE}/totp`]: () => jsonResponse(ENROLLMENT),
      [`POST ${BASE}/totp/confirm`]: () => jsonResponse({ recoveryCodes: CODES }),
    });
    renderWithApp(<SecurityPage />);
    const dialog = await enrollToCodes();
    await userEvent.click(within(dialog).getByRole('checkbox', { name: 'I saved these codes' }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Done' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(screen.queryByText(CODES[0] ?? '')).toBeNull();
  });

  it('keeps Done disabled until the saved checkbox is ticked', async () => {
    serverFor(STATUS, {
      [`POST ${BASE}/totp`]: () => jsonResponse(ENROLLMENT),
      [`POST ${BASE}/totp/confirm`]: () => jsonResponse({ recoveryCodes: CODES }),
    });
    renderWithApp(<SecurityPage />);
    const dialog = await enrollToCodes();
    const done = within(dialog).getByRole('button', { name: 'Done' });
    expect(done).toBeDisabled();
    await userEvent.click(within(dialog).getByRole('checkbox', { name: 'I saved these codes' }));
    expect(done).toBeEnabled();
  });

  it('calls the regenerate route once on a double click', async () => {
    const server = serverFor(ENROLLED, {
      [`POST ${BASE}/recovery-codes`]: () => jsonResponse({ recoveryCodes: CODES }),
    });
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'New Recovery Codes' }));
    const dialog = await screen.findByRole('dialog', { name: 'New Recovery Codes' });
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.dblClick(within(dialog).getByRole('button', { name: 'New Recovery Codes' }));
    expect(await within(dialog).findByText(CODES[0] ?? '')).toBeInTheDocument();
    expect(
      server.calls.filter((call) => call.method === 'POST' && call.path.endsWith('/recovery-codes')),
    ).toHaveLength(1);
  });

  it('shows the server refusal when the second factor cannot be removed', async () => {
    const refusal = 'a second factor is required for this account; add another before removing this one';
    const server = serverFor(ENROLLED, {
      [`DELETE ${BASE}/totp`]: () => jsonResponse([{ id: '*', message: refusal }], 400),
    });
    renderWithApp(<SecurityPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Remove' }));
    const dialog = await screen.findByRole('dialog', { name: 'Remove Authenticator App' });
    await userEvent.type(within(dialog).getByLabelText('Current Password'), PASSWORD);
    await userEvent.click(within(dialog).getByRole('button', { name: 'Remove' }));
    expect(await within(dialog).findByText(refusal)).toBeInTheDocument();
    expect(server.calls.find((call) => call.method === 'DELETE')?.body).toEqual({
      currentPassword: PASSWORD,
    });
  });
});
