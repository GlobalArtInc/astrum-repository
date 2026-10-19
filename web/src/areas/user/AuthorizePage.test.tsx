import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { leaveFor } from '../../auth/browser';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import AuthorizePage from './AuthorizePage';

vi.mock('../../auth/browser', () => ({ leaveFor: vi.fn(), openFresh: vi.fn() }));

const REQUEST = {
  id: 'req1',
  clientName: 'Claude Code',
  clientId: 'astrum-mcp-x',
  redirectHost: '127.0.0.1',
  requestedScope: ['read', 'write'],
  writeRequested: true,
  user: 'admin',
  expiresAt: '2099-01-01T00:00:00Z',
};
const PATH = '/service/rest/internal/ui/oauth/requests/req1';

function page(path: string) {
  return renderWithApp(null, {
    state: ADMIN_STATE,
    path,
    routes: [{ path: '/user/authorize', element: <AuthorizePage /> }],
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(leaveFor).mockReset();
});

describe('the agent consent page', () => {
  it('allows an agent with changes and sends the browser back', async () => {
    const server = fakeServer({
      [`GET ${PATH}`]: () => jsonResponse(REQUEST),
      [`POST ${PATH}/approve`]: () => jsonResponse({ redirect: 'http://127.0.0.1:3000/cb?code=c&state=s' }),
    });
    vi.stubGlobal('fetch', server.fetch);
    page('/user/authorize?request=req1');
    expect(
      await screen.findByRole('heading', { name: 'Connect Claude Code to Astrum Repository?' }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/as admin\. When you answer, your browser goes back to 127\.0\.0\.1\./),
    ).toBeInTheDocument();
    const write = screen.getByRole('checkbox', { name: /Allow changes as well/ });
    expect(write).not.toBeChecked();
    await userEvent.click(write);
    await userEvent.click(screen.getByRole('button', { name: 'Allow' }));
    await waitFor(() => expect(leaveFor).toHaveBeenCalledWith('http://127.0.0.1:3000/cb?code=c&state=s'));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({ write: true });
  });

  it('denies an agent', async () => {
    const server = fakeServer({
      [`GET ${PATH}`]: () => jsonResponse(REQUEST),
      [`POST ${PATH}/deny`]: () => jsonResponse({ redirect: 'http://127.0.0.1:3000/cb?error=access_denied' }),
    });
    vi.stubGlobal('fetch', server.fetch);
    page('/user/authorize?request=req1');
    await userEvent.click(await screen.findByRole('button', { name: 'Deny' }));
    await waitFor(() =>
      expect(leaveFor).toHaveBeenCalledWith('http://127.0.0.1:3000/cb?error=access_denied'),
    );
  });

  it('explains expired requests and broken links', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({
        [`GET ${PATH}`]: () => jsonResponse({ id: '*', message: 'gone' }, 404),
      }).fetch,
    );
    const { unmount } = page('/user/authorize?request=req1');
    expect(await screen.findByText(/This request has expired or was used already/)).toBeInTheDocument();
    unmount();
    const second = page('/user/authorize?problem=redirect');
    expect(await screen.findByText(/asked to return to an address it did not register/)).toBeInTheDocument();
    second.unmount();
    page('/user/authorize');
    expect(await screen.findByText(/This page opens when an agent asks to connect/)).toBeInTheDocument();
  });
});
