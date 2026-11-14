import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import type { CooldownWait } from './api';
import CooldownPage from './CooldownPage';

const BLOCKED: CooldownWait = {
  repository: 'npm-proxy',
  format: 'npm',
  namespace: '@acme',
  name: 'widgets',
  version: '2.0.0',
  publishedAt: '2026-11-10T08:00:00Z',
  availableAt: '2026-11-17T08:00:00Z',
  blocked: true,
  refusals: 3,
  firstRefusedAt: '2026-11-11T09:00:00Z',
};

const REPORTED: CooldownWait = {
  ...BLOCKED,
  name: 'gadgets',
  version: '1.4.0',
  blocked: false,
  refusals: 0,
  firstRefusedAt: null,
};

const LIST = '/service/rest/v1/security/cooldown';
const REPOSITORIES = '/service/rest/v1/repositories';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the cooldown waits page', () => {
  it('lists the waiting versions with their state and refusals', async () => {
    const server = fakeServer({
      [`GET ${LIST}`]: () => jsonResponse({ items: [BLOCKED, REPORTED], continuationToken: null }),
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<CooldownPage />);
    const row = (await screen.findByText('@acme/widgets')).closest('tr') as HTMLElement;
    expect(within(row).getByText('npm-proxy')).toBeInTheDocument();
    expect(within(row).getByText('2.0.0')).toBeInTheDocument();
    expect(within(row).getByText('Blocked')).toBeInTheDocument();
    expect(within(row).getByText('3')).toBeInTheDocument();
    const reported = screen.getByText('@acme/gadgets').closest('tr') as HTMLElement;
    expect(within(reported).getByText('Reported')).toBeInTheDocument();
  });

  it('says that nothing is waiting when the list is empty', async () => {
    const server = fakeServer({
      [`GET ${LIST}`]: () => jsonResponse({ items: [], continuationToken: null }),
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<CooldownPage />);
    expect(await screen.findByText('Nothing is waiting')).toBeInTheDocument();
  });

  it('asks the list for one repository when the filter names it', async () => {
    const server = fakeServer({
      [`GET ${LIST}`]: () => jsonResponse({ items: [], continuationToken: null }),
      [`GET ${LIST}?repository=npm-proxy`]: () => jsonResponse({ items: [BLOCKED], continuationToken: null }),
      [`GET ${REPOSITORIES}`]: () => jsonResponse([{ name: 'npm-proxy', format: 'npm', type: 'proxy' }]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<CooldownPage />);
    await screen.findByText('Nothing is waiting');
    await userEvent.click(screen.getByRole('combobox', { name: 'Repository' }));
    await userEvent.click(await screen.findByRole('option', { name: 'npm-proxy' }));
    expect(await screen.findByText('@acme/widgets')).toBeInTheDocument();
    expect(server.calls.some((call) => call.path === `${LIST}?repository=npm-proxy`)).toBe(true);
  });

  it('loads the next page with the continuation token on Load more', async () => {
    const server = fakeServer({
      [`GET ${LIST}`]: () => jsonResponse({ items: [BLOCKED], continuationToken: 'page-2' }),
      [`GET ${LIST}?continuationToken=page-2`]: () =>
        jsonResponse({ items: [REPORTED], continuationToken: null }),
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<CooldownPage />);
    await userEvent.click(await screen.findByRole('button', { name: 'Load more' }));
    await waitFor(() => expect(screen.getByText('@acme/gadgets')).toBeInTheDocument());
    expect(screen.getByText('@acme/widgets')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Load more' })).not.toBeInTheDocument();
  });
});
