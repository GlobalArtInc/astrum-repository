import type { UsageDashboardData, UsageComponent } from '../usageDashboard.types';

export function component(overrides: Partial<UsageComponent> = {}): UsageComponent {
  return {
    id: 'c1',
    repository: 'maven-releases',
    format: 'maven2',
    group: 'org.example',
    name: 'demo',
    version: '1.0.0',
    downloads: 12,
    created: '2026-10-11T10:00:00Z',
    createdBy: 'ci-bot',
    browsePath: 'org/example/demo/1.0.0',
    ...overrides,
  };
}

function days(count: number, downloads: (index: number) => number) {
  const last = Date.UTC(2026, 9, 11);
  return Array.from({ length: count }, (_, index) => ({
    day: new Date(last - (count - 1 - index) * 86_400_000).toISOString().slice(0, 10),
    downloads: downloads(index),
  }));
}

export function dashboard(overrides: Partial<UsageDashboardData> = {}): UsageDashboardData {
  return {
    days: 30,
    from: '2026-09-12',
    to: '2026-10-11',
    retentionDays: 180,
    unusedDays: 90,
    repository: null,
    repositories: [
      { name: 'maven-releases', format: 'maven2', type: 'hosted' },
      { name: 'raw-files', format: 'raw', type: 'hosted' },
      { name: 'maven-central', format: 'maven2', type: 'proxy' },
    ],
    totals: {
      repositories: 3,
      hosted: 2,
      proxy: 1,
      group: 0,
      byFormat: [],
      components: 2418,
      assets: 19774,
      storageBytes: 5 * 1024 ** 3,
      proxyStorageBytes: 1024 ** 3,
      downloads: 5000,
      downloadsToday: 40,
      downloadsLast7Days: 300,
      downloadsLast30Days: 1200,
      downloadsInPeriod: 1200,
      componentsDownloadedInPeriod: 380,
    },
    downloadsPerDay: days(30, (index) => (index === 20 ? 200 : 30 + index)),
    topComponents: [
      component(),
      component({ id: 'c2', repository: 'raw-files', format: 'raw', group: null, name: 'tools/keel.tar.gz',
        version: null, downloads: 6, browsePath: null }),
    ],
    topComponentsAllTime: [component({ id: 'c3', name: 'legacy', downloads: 4000 })],
    topRepositoriesByDownloads: [
      { name: 'maven-releases', format: 'maven2', type: 'hosted', downloads: 900, storageBytes: 1024 ** 3,
        components: 10 },
      { name: 'raw-files', format: 'raw', type: 'hosted', downloads: 300, storageBytes: 3 * 1024 ** 3,
        components: 5 },
    ],
    topRepositoriesByStorage: [
      { name: 'raw-files', format: 'raw', type: 'hosted', downloads: 300, storageBytes: 3 * 1024 ** 3,
        components: 5 },
      { name: 'maven-releases', format: 'maven2', type: 'hosted', downloads: 900, storageBytes: 1024 ** 3,
        components: 10 },
    ],
    recentComponents: [component({ id: 'r1', name: 'fresh', version: '2.0.0', created: '2026-10-11T11:48:00Z' })],
    unused: {
      components: 612,
      bytes: 41 * 1024 ** 3,
      neverDownloadedComponents: 31,
      neverDownloadedBytes: 1024 ** 2,
      repositories: [
        { name: 'maven-releases', format: 'maven2', components: 600, bytes: 40 * 1024 ** 3 },
        { name: 'raw-files', format: 'raw', components: 12, bytes: 1024 ** 3 },
      ],
    },
    ...overrides,
  };
}

export function emptyDashboard(): UsageDashboardData {
  const base = dashboard();
  return dashboard({
    totals: { ...base.totals, components: 0, assets: 0, storageBytes: 0, proxyStorageBytes: 0, downloads: 0,
      downloadsToday: 0, downloadsLast7Days: 0, downloadsLast30Days: 0, downloadsInPeriod: 0,
      componentsDownloadedInPeriod: 0 },
    downloadsPerDay: days(30, () => 0),
    topComponents: [],
    topComponentsAllTime: [],
    topRepositoriesByDownloads: [],
    topRepositoriesByStorage: [],
    recentComponents: [],
    unused: { components: 0, bytes: 0, neverDownloadedComponents: 0, neverDownloadedBytes: 0, repositories: [] },
  });
}
