/**
 * Response of GET /service/rest/internal/ui/usage-dashboard: how much of what is used in the repositories the user
 * may browse. Days are UTC days, `yyyy-MM-dd`.
 */
export interface UsageDashboardData {
  days: number;
  from: string;
  to: string;
  retentionDays: number;
  unusedDays: number;
  repository: string | null;
  repositories: UsageRepository[];
  totals: UsageTotals;
  downloadsPerDay: DownloadsOfDay[];
  topComponents: UsageComponent[];
  topComponentsAllTime: UsageComponent[];
  topRepositoriesByDownloads: RepositoryUsage[];
  topRepositoriesByStorage: RepositoryUsage[];
  recentComponents: UsageComponent[];
  unused: UnusedComponents;
}

export interface UsageRepository {
  name: string;
  format: string;
  type: string;
}

export interface UsageTotals {
  repositories: number;
  hosted: number;
  proxy: number;
  group: number;
  byFormat: Array<{ format: string; hosted: number; proxy: number; group: number }>;
  components: number;
  assets: number;
  storageBytes: number;
  proxyStorageBytes: number;
  downloads: number;
  downloadsToday: number;
  downloadsLast7Days: number;
  downloadsLast30Days: number;
  downloadsInPeriod: number;
  componentsDownloadedInPeriod: number;
}

export interface DownloadsOfDay {
  day: string;
  downloads: number;
}

export interface UsageComponent {
  id: string;
  repository: string;
  format: string;
  group: string | null;
  name: string;
  version: string | null;
  downloads: number;
  created: string;
  createdBy: string | null;
  browsePath: string | null;
}

export interface RepositoryUsage {
  name: string;
  format: string;
  type: string;
  downloads: number;
  storageBytes: number;
  components: number;
}

export interface UnusedComponents {
  components: number;
  bytes: number;
  neverDownloadedComponents: number;
  neverDownloadedBytes: number;
  repositories: Array<{ name: string; format: string; components: number; bytes: number }>;
}

/** Which user interface the links of the dashboard lead to. */
export type UsageDashboardVariant = 'preview' | 'classic';

export const USAGE_PERIODS = [7, 30, 90] as const;
