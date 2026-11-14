import { screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { apiKey } from '../../api';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { REPLICATIONS, type Replication } from './api';
import { ReplicatedBadge } from './ReplicatedBadge';

afterEach(() => {
  vi.unstubAllGlobals();
});

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

const OTHER: Replication = { ...REPLICATION, id: 'p2', name: 'Weekly', sourceRepository: 'other' };

describe('the replicated badge', () => {
  it('shows the number of enabled replications of the repository and links to the page', async () => {
    const fake = fakeServer({
      [`GET ${REPLICATIONS}`]: () =>
        jsonResponse([REPLICATION, { ...REPLICATION, id: 'p3', enabled: false }, OTHER]),
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicatedBadge repository="releases" />);
    expect(await screen.findByText('Replicated to 1')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Replicated to 1' })).toHaveAttribute(
      'href',
      '/admin/replication?tab=replications',
    );
  });

  it('is hidden when the repository has no enabled replication', async () => {
    const fake = fakeServer({ [`GET ${REPLICATIONS}`]: () => jsonResponse([OTHER]) });
    vi.stubGlobal('fetch', fake.fetch);
    const { client } = renderWithApp(<ReplicatedBadge repository="releases" />);
    await waitFor(() => expect(client.getQueryState(apiKey(REPLICATIONS))?.status).toBe('success'));
    expect(screen.queryByText(/Replicated to/)).not.toBeInTheDocument();
  });

  it('asks for nothing without the read privilege', async () => {
    const fake = fakeServer({ [`GET ${REPLICATIONS}`]: () => jsonResponse([REPLICATION]) });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<ReplicatedBadge repository="releases" />, {
      state: { ...ADMIN_STATE, permissions: ['astrum:repository-view:*:*:browse'] },
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(fake.calls).toHaveLength(0);
    expect(screen.queryByText(/Replicated to/)).not.toBeInTheDocument();
  });
});
