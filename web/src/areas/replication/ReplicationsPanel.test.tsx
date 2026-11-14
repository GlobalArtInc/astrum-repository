import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { REMOTES, REPLICATIONS, type Replication, type ReplicationRemote } from './api';
import { ReplicationsPanel } from './ReplicationsPanel';

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
  alertState: 'ok',
  lastSuccessAt: null,
  lastFailureAt: null,
  lastError: null,
  lastReconcileAt: null,
  createdAt: '2026-11-01T10:00:00Z',
  updatedAt: '2026-11-01T10:00:00Z',
};

const REPLICATION_PATH = `${REPLICATIONS}/p1`;
const COMPONENTS_PATH = '/service/rest/v1/components?repository=releases';
const REPOSITORIES_PATH = '/service/rest/v1/repositories';
const CHANNELS_PATH = '/service/rest/internal/ui/notifications';

function handlers(replications: Replication[]) {
  return {
    [`GET ${REMOTES}`]: () => jsonResponse([REMOTE]),
    [`GET ${REPLICATIONS}`]: () => jsonResponse(replications),
    [`GET ${REPOSITORIES_PATH}`]: () =>
      jsonResponse([
        { name: 'releases', format: 'raw', type: 'hosted' },
        { name: 'proxy-raw', format: 'raw', type: 'proxy' },
      ]),
    [`GET ${CHANNELS_PATH}`]: () => jsonResponse([]),
    [`GET ${COMPONENTS_PATH}`]: () =>
      jsonResponse({
        items: [
          { repository: 'releases', format: 'raw', group: null, name: 'lib-core', version: '1.0' },
          { repository: 'releases', format: 'raw', group: null, name: 'app', version: '1.0' },
        ],
        continuationToken: null,
      }),
  };
}

describe('the replications panel', () => {
  it('lists replications and toggles one off with the full body', async () => {
    const fake = fakeServer({
      ...handlers([REPLICATION]),
      [`PUT ${REPLICATION_PATH}`]: () => jsonResponse({ ...REPLICATION, enabled: false }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    expect(await screen.findByText('Nightly')).toBeInTheDocument();
    expect(screen.getByText('Mirror · releases-eu')).toBeInTheDocument();
    expect(screen.getByText('All components')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('switch', { name: 'Disable replication Nightly' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'PUT')?.body).toEqual({
      name: 'Nightly',
      sourceRepository: 'releases',
      remoteId: 'r1',
      remoteRepository: 'releases-eu',
      enabled: false,
      nameFilter: null,
      versionFilter: null,
      propagateDeletes: false,
      settleSeconds: 30,
      reconcileMinutes: 360,
      alertChannels: [],
    });
  });

  it('shows which components the name filter would send and rejects a broken pattern inline', async () => {
    const fake = fakeServer(handlers([REPLICATION]));
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Edit replication Nightly' }));
    const dialog = await screen.findByRole('dialog', { name: 'Edit replication' });
    const filter = within(dialog).getByLabelText('Name filter');
    await userEvent.type(filter, 'lib-.*');
    expect(await within(dialog).findByText('lib-core:1.0')).toBeInTheDocument();
    expect(within(dialog).getByText('app:1.0')).toBeInTheDocument();
    expect(within(dialog).getByText('Would be sent')).toBeInTheDocument();
    expect(within(dialog).getByText('Would be skipped')).toBeInTheDocument();
    await userEvent.clear(filter);
    await userEvent.type(filter, '(lib');
    expect(
      await within(dialog).findByText(
        'This is not a valid pattern in the browser. The server decides when you save.',
      ),
    ).toBeInTheDocument();
  });

  it('creates a replication from the source and remote choices', async () => {
    const fake = fakeServer({
      ...handlers([]),
      [`POST ${REPLICATIONS}`]: () => jsonResponse(REPLICATION, 201),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add replication' }));
    const dialog = await screen.findByRole('dialog', { name: 'New replication' });
    await userEvent.type(within(dialog).getByLabelText('Name'), 'Nightly');
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Source repository' }));
    await userEvent.click(await screen.findByRole('option', { name: 'releases' }));
    await userEvent.click(within(dialog).getByRole('combobox', { name: 'Remote' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Mirror' }));
    await userEvent.type(within(dialog).getByLabelText('Remote repository'), 'releases-eu');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(fake.calls.find((call) => call.method === 'POST')?.body).toMatchObject({
      name: 'Nightly',
      sourceRepository: 'releases',
      remoteId: 'r1',
      remoteRepository: 'releases-eu',
      enabled: true,
      settleSeconds: 30,
      reconcileMinutes: 360,
    });
  });

  it('deletes a replication after confirmation', async () => {
    const fake = fakeServer({
      ...handlers([REPLICATION]),
      [`DELETE ${REPLICATION_PATH}`]: () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Delete replication Nightly' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'DELETE')).toBe(true));
  });
});

describe('the reconcile period', () => {
  it('offers the 13 server periods and selects 6 hours by default', async () => {
    const fake = fakeServer(handlers([]));
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Add replication' }));
    const dialog = await screen.findByRole('dialog', { name: 'New replication' });
    const period = within(dialog).getByRole('combobox', { name: 'Reconcile every, minutes' });
    expect(period).toHaveTextContent('Every 6 hours');
    await userEvent.click(period);
    expect(await screen.findAllByRole('option')).toHaveLength(13);
  });

  it('shows a legacy value as an extra option and sends the period that is chosen', async () => {
    const fake = fakeServer({
      ...handlers([{ ...REPLICATION, reconcileMinutes: 45 }]),
      [`PUT ${REPLICATION_PATH}`]: () => jsonResponse({ ...REPLICATION, reconcileMinutes: 60 }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Edit replication Nightly' }));
    const dialog = await screen.findByRole('dialog', { name: 'Edit replication' });
    const period = within(dialog).getByRole('combobox', { name: 'Reconcile every, minutes' });
    expect(period).toHaveTextContent('45 minutes (not offered any more: pick one of the periods to save)');
    await userEvent.click(period);
    await userEvent.click(await screen.findByRole('option', { name: 'Every hour' }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(fake.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({ reconcileMinutes: 60 });
  });

  it('shows a server field error on reconcileMinutes on the select', async () => {
    const fake = fakeServer({
      ...handlers([REPLICATION]),
      [`PUT ${REPLICATION_PATH}`]: () =>
        jsonResponse([{ id: 'reconcileMinutes', message: 'Use one of the listed periods' }], 400),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    await userEvent.click(await screen.findByRole('button', { name: 'Edit replication Nightly' }));
    const dialog = await screen.findByRole('dialog', { name: 'Edit replication' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(await within(dialog).findByText('Use one of the listed periods')).toBeInTheDocument();
    expect(within(dialog).getByRole('combobox', { name: 'Reconcile every, minutes' })).toHaveAttribute(
      'aria-invalid',
      'true',
    );
  });
});

describe('the status columns', () => {
  const DISABLED: Replication = { ...REPLICATION, id: 'p2', name: 'Weekly', enabled: false };

  it('shows the state, pending, dead, lag and last success of each replication', async () => {
    const fake = fakeServer({
      ...handlers([REPLICATION, DISABLED]),
      [`GET ${REPLICATIONS}/p1/status`]: () =>
        jsonResponse({
          enabled: true,
          pending: 2,
          leased: 0,
          dead: 3,
          lagSeconds: 120,
          lastSuccessAt: null,
          lastFailureAt: null,
          lastError: null,
          lastReconcileAt: null,
          alertState: 'dead_items',
          remote: { suspendedUntil: null, failures: 0, lastError: null },
        }),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    const row = (await screen.findByText('Nightly')).closest('tr') as HTMLElement;
    expect(within(row).getByText('Dead items')).toBeInTheDocument();
    expect(within(row).getByText('2')).toBeInTheDocument();
    expect(within(row).getByText('3')).toBeInTheDocument();
    expect(within(row).getByText('2 minutes')).toBeInTheDocument();
    expect(within(row).getByText('Never')).toBeInTheDocument();
  });

  it('shows a dash for a row whose status request fails and Disabled for a disabled replication', async () => {
    const fake = fakeServer(handlers([REPLICATION, DISABLED]));
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicationsPanel />);
    const row = (await screen.findByText('Nightly')).closest('tr') as HTMLElement;
    expect(await within(row).findAllByText('—')).toHaveLength(5);
    const disabledRow = screen.getByText('Weekly').closest('tr') as HTMLElement;
    expect(within(disabledRow).getByText('Disabled')).toBeInTheDocument();
  });
});
