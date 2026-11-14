import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import {
  READ,
  REMOTES,
  REPLICATIONS,
  type Replication,
  type ReplicationItem,
  type ReplicationRemote,
  type ReplicationStatus,
} from './api';
import ReplicationDetailPage from './ReplicationDetailPage';

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
  remoteVersion: null,
  checkedAt: null,
  failures: 0,
  suspendedUntil: null,
  lastError: null,
  createdAt: '2026-11-01T10:00:00Z',
  updatedAt: '2026-11-01T10:00:00Z',
};

const REPLICATION: Replication = {
  id: 'p1',
  name: 'Nightly',
  sourceRepository: 'releases',
  remoteId: 'r1',
  remoteRepository: 'releases-eu',
  enabled: true,
  nameFilter: null,
  versionFilter: null,
  propagateDeletes: false,
  settleSeconds: 30,
  reconcileMinutes: 360,
  alertChannels: [],
  alertState: 'dead_items',
  lastSuccessAt: null,
  lastFailureAt: null,
  lastError: null,
  lastReconcileAt: null,
  createdAt: '2026-11-01T10:00:00Z',
  updatedAt: '2026-11-01T10:00:00Z',
};

const STATUS: ReplicationStatus = {
  enabled: true,
  pending: 2,
  leased: 0,
  dead: 3,
  lagSeconds: 120,
  lastSuccessAt: null,
  lastFailureAt: null,
  lastError: 'connection refused',
  lastReconcileAt: null,
  alertState: 'dead_items',
  remote: { suspendedUntil: null, failures: 0, lastError: null },
};

function item(id: number, key: string): ReplicationItem {
  return {
    id,
    key,
    state: 'dead',
    reason: 'event',
    attempts: 12,
    notBefore: '2026-11-13T10:00:00Z',
    firstQueuedAt: '2026-11-13T09:00:00Z',
    lastError: 'the remote answered 503 Service Unavailable and the body was not kept',
  };
}

const DETAIL_PATH = '/admin/replication/replications/p1';
const REPLICATION_PATH = `${REPLICATIONS}/p1`;
const STATUS_PATH = `${REPLICATION_PATH}/status`;
const ITEMS_PATH = `${REPLICATION_PATH}/items`;
const RETRY_PATH = `${ITEMS_PATH}/retry`;
const DISCARD_PATH = `${ITEMS_PATH}/discard`;
const RECONCILE_PATH = `${REPLICATION_PATH}/reconcile`;

function handlers(status: ReplicationStatus = STATUS) {
  return {
    [`GET ${REPLICATIONS}`]: () => jsonResponse([REPLICATION]),
    [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
    [`GET ${STATUS_PATH}`]: () => jsonResponse(status),
    [`GET ${ITEMS_PATH}?state=dead&limit=50`]: () =>
      jsonResponse({ items: [item(1, 'raw/a@1'), item(2, 'raw/b@1')], next: null }),
  };
}

function renderDetail(state = ADMIN_STATE) {
  return renderWithApp(null, {
    state,
    path: DETAIL_PATH,
    routes: [{ path: '/admin/replication/replications/:id', element: <ReplicationDetailPage /> }],
  });
}

describe('the replication detail page', () => {
  it('shows the status card and a warning while the remote is suspended', async () => {
    const fake = fakeServer({
      ...handlers({
        ...STATUS,
        remote: { suspendedUntil: '2099-01-01T00:00:00Z', failures: 4, lastError: 'connection refused' },
      }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    expect(
      await screen.findByText(/^Suspended until .* after 4 failures: connection refused$/),
    ).toBeInTheDocument();
  });

  it('reports a reconcile that started', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`POST ${RECONCILE_PATH}`]: () => new Response(null, { status: 202 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click(await screen.findByRole('button', { name: 'Reconcile now' }));
    expect(await screen.findByText('Reconcile started')).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'POST' && call.path === RECONCILE_PATH)).toBe(true);
  });

  it('shows the message of a reconcile that is already running', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`POST ${RECONCILE_PATH}`]: () => jsonResponse('a reconcile run is already active', 409),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click(await screen.findByRole('button', { name: 'Reconcile now' }));
    expect(await screen.findByText('a reconcile run is already active')).toBeInTheDocument();
  });

  it('loads the next page of dead items from the cursor', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`GET ${ITEMS_PATH}?state=dead&limit=50`]: () =>
        jsonResponse({ items: [item(1, 'raw/a@1'), item(2, 'raw/b@1')], next: 2 }),
      [`GET ${ITEMS_PATH}?state=dead&after=2&limit=50`]: () =>
        jsonResponse({ items: [item(3, 'raw/c@1')], next: null }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    expect(await screen.findByText('raw/a@1')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(await screen.findByText('raw/c@1')).toBeInTheDocument();
    expect(screen.getByText('raw/a@1')).toBeInTheDocument();
  });

  it('retries the selected items by id after a confirmation with their count', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`POST ${RETRY_PATH}`]: () => jsonResponse({ retried: 1 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click(await screen.findByRole('checkbox', { name: 'Select raw/a@1' }));
    await userEvent.click(screen.getByRole('button', { name: 'Retry selected' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText('Retry 1 selected item?')).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Retry' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'POST')?.body).toEqual({ ids: [1] });
  });

  it('discards all dead items by the all selector and says the component waits for a change', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`POST ${DISCARD_PATH}`]: () => jsonResponse({ discarded: 3 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await userEvent.click(await screen.findByRole('button', { name: 'Discard all dead' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText('Discard 3 dead items?')).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        /The next reconcile queues a component again if it still differs on the remote\./,
      ),
    ).toBeInTheDocument();
    await userEvent.click(within(dialog).getByRole('button', { name: 'Discard' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'POST')?.body).toEqual({ all: true });
  });

  it('disables reconcile and the item actions without the update privilege', async () => {
    const fake = fakeServer(handlers());
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail({ ...ADMIN_STATE, permissions: [READ] });
    expect(await screen.findByRole('button', { name: 'Retry all dead' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Reconcile now' })).toBeDisabled();
  });

  it('hides the dead item actions when there are no dead items', async () => {
    const fake = fakeServer(handlers({ ...STATUS, dead: 0 }));
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await waitFor(() => expect(fake.calls.some((call) => call.path === STATUS_PATH)).toBe(true));
    expect(await screen.findByRole('combobox', { name: 'Show' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Retry all dead' })).not.toBeInTheDocument();
  });

  it('lists the items of the chosen state', async () => {
    const fake = fakeServer({
      ...handlers(),
      [`GET ${ITEMS_PATH}?state=pending&limit=50`]: () =>
        jsonResponse({ items: [{ ...item(9, 'raw/p@1'), state: 'pending' }], next: null }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderDetail();
    await screen.findByText('raw/a@1');
    await userEvent.click(screen.getByRole('combobox', { name: 'Show' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Pending' }));
    expect(await screen.findByText('raw/p@1')).toBeInTheDocument();
  });
});
