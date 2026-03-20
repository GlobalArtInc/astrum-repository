import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { DashboardPage } from './DashboardPage';
import { dayLabel, labelStep } from './DownloadsChart';
import { axisMax, componentName, isWeekend, peakOf, share, USAGE_PATH, type Usage } from './types';

function days(count: number, values: Record<number, number> = {}) {
  const last = Date.UTC(2026, 9, 12);
  return Array.from({ length: count }, (_, index) => ({
    day: new Date(last - (count - 1 - index) * 86_400_000).toISOString().slice(0, 10),
    downloads: values[index] ?? 0,
  }));
}

const LIB = {
  id: 'c1',
  repository: 'maven-releases',
  format: 'maven2',
  group: 'org.acme',
  name: 'lib',
  version: '1.0',
  downloads: 30,
  created: new Date().toISOString(),
  createdBy: 'jane',
  browsePath: 'org/acme/lib/1.0/',
};

const DATA: Usage = {
  days: 30,
  from: '2026-09-13',
  to: '2026-10-12',
  retentionDays: 180,
  unusedDays: 90,
  repository: null,
  repositories: [
    { name: 'maven-releases', format: 'maven2', type: 'hosted' },
    { name: 'npm-proxy', format: 'npm', type: 'proxy' },
  ],
  totals: {
    repositories: 2,
    hosted: 1,
    proxy: 1,
    group: 0,
    components: 1234,
    assets: 5678,
    storageBytes: 3 * 1024 * 1024,
    proxyStorageBytes: 1024,
    downloads: 900,
    downloadsToday: 36,
    downloadsInPeriod: 119,
    componentsDownloadedInPeriod: 7,
  },
  downloadsPerDay: days(30, { 28: 83, 29: 36 }),
  topComponents: [LIB],
  topComponentsAllTime: [{ ...LIB, downloads: 300 }],
  topRepositoriesByDownloads: [
    {
      name: 'maven-releases',
      format: 'maven2',
      type: 'hosted',
      downloads: 119,
      storageBytes: 2048,
      components: 3,
    },
  ],
  topRepositoriesByStorage: [
    { name: 'npm-proxy', format: 'npm', type: 'proxy', downloads: 0, storageBytes: 4096, components: 1 },
  ],
  recentComponents: [
    {
      ...LIB,
      id: 'c3',
      repository: 'npm-proxy',
      format: 'npm',
      group: 'team',
      name: 'tool',
      version: '2.0.0',
      browsePath: '@team/tool/-/tool-2.0.0.tgz',
      downloads: 0,
    },
  ],
  unused: {
    components: 2,
    bytes: 2048,
    neverDownloadedComponents: 4,
    neverDownloadedBytes: 1134,
    repositories: [{ name: 'maven-releases', format: 'maven2', components: 2, bytes: 2048 }],
  },
};

function serve(handlers: Record<string, Usage>) {
  const server = fakeServer(
    Object.fromEntries(
      Object.entries(handlers).map(([query, data]) => [
        `GET ${USAGE_PATH}?${query}`,
        () => jsonResponse(data),
      ]),
    ),
  );
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the overview', { timeout: 20_000 }, () => {
  it('shows the figures of the period', async () => {
    serve({ 'days=30': DATA });
    renderWithApp(<DashboardPage />);
    expect(await screen.findByText('today 36 · avg 4 a day')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1, name: 'Overview' })).toBeInTheDocument();
    expect(screen.getByText('How the repositories were used over the last 30 days')).toBeInTheDocument();
    expect(screen.getByText('119', { selector: 'div' })).toBeInTheDocument();
    expect(screen.getByText('1,234')).toBeInTheDocument();
    expect(screen.getByText('downloaded in the period: 7')).toBeInTheDocument();
    expect(screen.getByText('in 2 repositories')).toBeInTheDocument();
    expect(screen.getByText('incl. proxy cache 1 KB')).toBeInTheDocument();
    expect(screen.getByText('hosted 1 · proxy 1 · group 0')).toBeInTheDocument();
    expect(screen.getByText('Not used for 90 days')).toBeInTheDocument();
    expect(screen.getByText('2 KB can be freed')).toBeInTheDocument();
    expect(screen.getByText('Last 30 days · 119 in total · peak 83 (Oct 11)')).toBeInTheDocument();
    expect(screen.getByRole('img', { name: /Bars of the downloads/ })).toBeInTheDocument();

    const top = screen.getByRole('region', { name: 'Most downloaded' });
    expect(within(top).getByRole('link', { name: 'org.acme:lib' })).toHaveAttribute(
      'href',
      '/browse/maven-releases/org/acme/lib/1.0/',
    );
    expect(within(top).getByText('30')).toBeInTheDocument();
    await userEvent.click(within(top).getByRole('radio', { name: 'All time' }));
    expect(within(top).getByText('300')).toBeInTheDocument();
    expect(within(top).getByRole('link', { name: /All components in search/ })).toHaveAttribute(
      'href',
      '/search',
    );

    const repositories = screen.getByRole('region', { name: 'Repositories' });
    expect(within(repositories).getByRole('link', { name: 'maven-releases' })).toHaveAttribute(
      'href',
      '/browse/maven-releases',
    );
    await userEvent.click(within(repositories).getByRole('radio', { name: 'By size' }));
    expect(within(repositories).getByText('4 KB')).toBeInTheDocument();

    const recent = screen.getByRole('region', { name: 'Recently published' });
    expect(within(recent).getByRole('link', { name: '@team/tool' })).toHaveAttribute(
      'href',
      '/browse/npm-proxy/%40team/tool/-/tool-2.0.0.tgz',
    );
    expect(within(recent).getByText('npm-proxy · jane')).toBeInTheDocument();

    const unused = screen.getByRole('region', { name: 'Not downloaded for 90 days' });
    expect(within(unused).getByText('never downloaded: 4 (1.1 KB)')).toBeInTheDocument();
    expect(within(unused).getByText('2 components')).toBeInTheDocument();
    expect(within(unused).getByRole('link', { name: 'Configure cleanup policies' })).toHaveAttribute(
      'href',
      '/admin/repository/cleanup-policies',
    );
  });

  it('switches the period', async () => {
    const week: Usage = {
      ...DATA,
      days: 7,
      downloadsPerDay: days(7),
      topComponents: [],
      topRepositoriesByDownloads: [],
      unused: { ...DATA.unused, components: 0, bytes: 0, repositories: [] },
      totals: { ...DATA.totals, downloadsInPeriod: 0, downloadsToday: 0, proxy: 0 },
    };
    const server = serve({ 'days=30': DATA, 'days=7': week });
    renderWithApp(<DashboardPage />);
    await screen.findByText('today 36 · avg 4 a day');
    await userEvent.click(screen.getByRole('radio', { name: '7 days' }));
    expect(await screen.findByText('Last 7 days · no downloads yet')).toBeInTheDocument();
    expect(server.calls.map((call) => call.path)).toContain(`${USAGE_PATH}?days=7`);
    expect(screen.getByText('How the repositories were used over the last 7 days')).toBeInTheDocument();
    expect(screen.getByText('Nothing was downloaded in this period yet')).toBeInTheDocument();
    expect(screen.getByText('Nothing was downloaded in this period')).toBeInTheDocument();
    expect(screen.getByText('No repository was downloaded from in this period')).toBeInTheDocument();
    expect(
      screen.getByText('Every component was downloaded or added in the last 90 days.'),
    ).toBeInTheDocument();
    expect(screen.queryByText(/incl. proxy cache/)).toBeNull();
    expect(screen.getByRole('radio', { name: 'Last 7 days' })).toBeInTheDocument();
  });

  it('says when there is nothing to browse', async () => {
    serve({ 'days=30': { ...DATA, repositories: [] } });
    renderWithApp(<DashboardPage />);
    expect(await screen.findByText('There are no repositories you may browse yet.')).toBeInTheDocument();
  });

  it('computes the chart helpers', () => {
    expect(axisMax(257)).toBe(300);
    expect(axisMax(83)).toBe(100);
    expect(axisMax(149)).toBe(150);
    expect(axisMax(3)).toBe(4);
    expect(axisMax(0)).toBe(4);
    expect(peakOf(days(3, { 0: 5, 1: 9, 2: 9 }))?.day).toBe('2026-10-11');
    expect(peakOf(days(3))).toBeNull();
    expect(isWeekend('2026-10-11')).toBe(true);
    expect(isWeekend('2026-10-12')).toBe(false);
    expect(labelStep(30, 1000)).toBe(5);
    expect(labelStep(90, 1000)).toBe(15);
    expect(labelStep(7, 1000)).toBe(1);
    expect(labelStep(30, 300)).toBe(10);
    expect(dayLabel('en', '2026-10-06', 7)).toBe('Tue 6');
    expect(dayLabel('ru', '2026-10-06', 7)).toBe('вт 6');
    expect(dayLabel('en', '2026-09-17', 30)).toBe('Sep 17');
    expect(share(5, 10)).toBe(50);
    expect(share(3, 0)).toBe(0);
    expect(componentName({ format: 'raw', group: '/docs', name: '/docs/a.pdf' })).toBe('docs/a.pdf');
    expect(componentName({ format: 'npm', group: 'team', name: 'tool' })).toBe('@team/tool');
    expect(componentName({ format: 'docker', group: null, name: 'qa/hello' })).toBe('qa/hello');
  });
});
