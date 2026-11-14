import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../test/render';
import { DISMISSED_KEY, SecurityBanner } from './SecurityBanner';

const STATUS = '/service/rest/internal/ui/user/two-factor';
const LINK = 'Open security settings';
const DUE = {
  enrolled: false,
  required: true,
  enrollBy: '2026-12-01T12:00:00Z',
  recoveryCodesLeft: 0,
  restricted: false,
};

function showBanner(body: unknown, status = 200) {
  const server = fakeServer({ [`GET ${STATUS}`]: () => jsonResponse(body, status) });
  vi.stubGlobal('fetch', server.fetch);
  return renderWithApp(<SecurityBanner />, { state: ADMIN_STATE });
}

afterEach(() => {
  window.sessionStorage.clear();
  vi.unstubAllGlobals();
});

describe('the security banner', () => {
  it('says when two-factor authentication must be set up and links to the settings', async () => {
    showBanner(DUE);
    expect(
      await screen.findByText('Set up two-factor authentication before Dec 1, 2026.'),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: LINK })).toHaveAttribute('href', '/user/security');
  });

  it('warns when two or fewer recovery codes are left', async () => {
    showBanner({ enrolled: true, required: false, enrollBy: null, recoveryCodesLeft: 2, restricted: false });
    expect(await screen.findByText('Only 2 recovery codes are left.')).toBeInTheDocument();
  });

  it('shows nothing when the status is not 200', async () => {
    const { client } = showBanner({ id: '*', message: 'two-factor status is not for this account' }, 400);
    await waitFor(() => expect(client.getQueryState(['api', STATUS])?.status).toBe('error'));
    expect(screen.queryByRole('link', { name: LINK })).toBeNull();
  });

  it('can be dismissed for the browser session', async () => {
    showBanner(DUE);
    await userEvent.click(await screen.findByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByRole('link', { name: LINK })).toBeNull();
    expect(window.sessionStorage.getItem(DISMISSED_KEY)).toBe('enroll');
  });
});
