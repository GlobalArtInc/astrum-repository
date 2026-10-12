import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, ANONYMOUS_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { DashboardPage } from './DashboardPage';
import { DASHBOARD_PATH, type Dashboard, type StoreUsage, share, storeUse } from './types';

const STORE: StoreUsage = {
  name: 'default',
  type: 'File',
  unavailable: false,
  blobCount: 10,
  totalSize: 1024,
  availableSpace: 3072,
};

const DATA: Dashboard = {
  repositories: {
    total: 5,
    byType: { hosted: 3, proxy: 1, group: 1 },
    byFormat: [
      { format: 'maven2', total: 3, hosted: 1, proxy: 1, group: 1 },
      { format: 'npm', total: 2, hosted: 2, proxy: 0, group: 0 },
    ],
  },
  components: 1234,
  assets: 5678,
  totalSize: 3 * 1024 * 1024,
  downloads: {
    total: 42,
    topComponents: [
      {
        repository: 'maven-releases',
        format: 'maven2',
        group: 'org.acme',
        name: 'lib',
        version: '1.0',
        componentId: 'c1',
        downloads: 30,
      },
    ],
    topAssets: [
      {
        repository: 'raw-files',
        format: 'raw',
        path: '/docs/readme.txt',
        assetId: 'a1',
        componentId: 'c2',
        downloads: 12,
        lastDownloaded: null,
      },
    ],
  },
  recentUploads: [
    {
      kind: 'component',
      repository: 'npm-internal',
      format: 'npm',
      componentId: 'c3',
      group: 'team',
      name: 'tool',
      version: '2.0.0',
      path: '/@team/tool/-/tool-2.0.0.tgz',
      assetId: 'a3',
      assetCount: 1,
      uploadedAt: new Date().toISOString(),
      uploader: 'jane',
    },
  ],
  blobStores: [STORE],
};

function serve(data: Dashboard) {
  vi.stubGlobal('fetch', fakeServer({ [`GET ${DASHBOARD_PATH}`]: () => jsonResponse(data) }).fetch);
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the dashboard', { timeout: 20_000 }, () => {
  it('shows the totals, the most downloaded and the latest uploads', async () => {
    serve(DATA);
    renderWithApp(<DashboardPage />);
    expect(await screen.findByText('1,234')).toBeInTheDocument();
    expect(screen.getByText('5,678')).toBeInTheDocument();
    expect(screen.getByText('3 MB')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('Hosted 3 · Proxy 1 · Group 1')).toBeInTheDocument();
    const top = screen.getByRole('list', { name: 'Components' });
    expect(within(top).getByRole('link', { name: 'org.acme:lib' })).toHaveAttribute(
      'href',
      '/search/component/maven2%3Aorg.acme%3Alib?version=1.0',
    );
    await userEvent.click(screen.getByRole('radio', { name: 'Files' }));
    expect(screen.getByRole('link', { name: '/docs/readme.txt' })).toHaveAttribute(
      'href',
      '/browse/raw-files/docs/readme.txt',
    );
    expect(screen.getByRole('link', { name: '@team/tool' })).toBeInTheDocument();
    expect(screen.getByText('npm-internal · jane · 1 file')).toBeInTheDocument();
    expect(screen.getByText('Maven')).toBeInTheDocument();
    expect(screen.getByText('Hosted 2')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
    expect(screen.getByText('10 blobs · 3 KB free')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Upload' })).toHaveAttribute('href', '/upload');
  });

  it('hides blob stores and links the caller may not use', async () => {
    serve({ ...DATA, blobStores: undefined });
    renderWithApp(<DashboardPage />, { state: ANONYMOUS_STATE });
    expect(await screen.findByText('1,234')).toBeInTheDocument();
    expect(screen.queryByText('Blob stores')).toBeNull();
    expect(screen.queryByRole('link', { name: 'Upload' })).toBeNull();
    expect(screen.getByRole('link', { name: 'Browse' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Search' })).toBeInTheDocument();
  });

  it('says there is nothing yet without repositories', async () => {
    serve({
      ...DATA,
      repositories: { total: 0, byType: { hosted: 0, proxy: 0, group: 0 }, byFormat: [] },
    });
    renderWithApp(<DashboardPage />, { state: ADMIN_STATE });
    expect(await screen.findByText('Nothing to show yet')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Create a repository' })).toBeInTheDocument();
  });

  it('scales bars and store use', () => {
    expect(share(0, 10)).toBe(0);
    expect(share(5, 10)).toBe(50);
    expect(share(1, 1000)).toBe(2);
    expect(share(3, 0)).toBe(0);
    expect(storeUse(STORE)).toBe(25);
    expect(storeUse({ ...STORE, availableSpace: null })).toBeNull();
  });
});
