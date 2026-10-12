import type { RepositoryType } from '../../api';

export const USAGE_PATH = '/service/rest/internal/ui/usage-dashboard';

export const PERIODS = [7, 30, 90] as const;
export type Period = (typeof PERIODS)[number];

export interface RepositoryRef {
  name: string;
  format: string;
  type: RepositoryType;
}

export interface Totals {
  repositories: number;
  hosted: number;
  proxy: number;
  group: number;
  components: number;
  assets: number;
  storageBytes: number;
  proxyStorageBytes: number;
  downloads: number;
  downloadsToday: number;
  downloadsInPeriod: number;
  componentsDownloadedInPeriod: number;
}

export interface DayDownloads {
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
  type: RepositoryType;
  downloads: number;
  storageBytes: number;
  components: number;
}

export interface UnusedRepository {
  name: string;
  format: string;
  components: number;
  bytes: number;
}

export interface Usage {
  days: number;
  from: string;
  to: string;
  retentionDays: number;
  unusedDays: number;
  repository: string | null;
  repositories: RepositoryRef[];
  totals: Totals;
  downloadsPerDay: DayDownloads[];
  topComponents: UsageComponent[];
  topComponentsAllTime: UsageComponent[];
  topRepositoriesByDownloads: RepositoryUsage[];
  topRepositoriesByStorage: RepositoryUsage[];
  recentComponents: UsageComponent[];
  unused: {
    components: number;
    bytes: number;
    neverDownloadedComponents: number;
    neverDownloadedBytes: number;
    repositories: UnusedRepository[];
  };
}

export function share(value: number, max: number): number {
  if (max <= 0 || value <= 0) {
    return 0;
  }
  return Math.min(100, (value / max) * 100);
}

const NICE = [1, 1.5, 2, 2.5, 3, 4, 5, 6, 8, 10];
const NICE_SMALL = [2, 4, 6, 8, 10];

export function axisMax(peak: number): number {
  if (peak <= 0) {
    return 4;
  }
  if (peak <= 10) {
    return NICE_SMALL.find((step) => step >= peak) ?? 10;
  }
  const scale = 10 ** Math.floor(Math.log10(peak));
  const factor = NICE.find((step) => step * scale >= peak) ?? 10;
  return factor * scale;
}

export function parseDay(day: string): Date {
  const [year, month, date] = day.split('-').map(Number);
  return new Date(Date.UTC(year ?? 1970, (month ?? 1) - 1, date ?? 1));
}

export function isWeekend(day: string): boolean {
  const weekday = parseDay(day).getUTCDay();
  return weekday === 0 || weekday === 6;
}

export interface Peak {
  day: string;
  downloads: number;
}

export function peakOf(days: DayDownloads[]): Peak | null {
  let peak: Peak | null = null;
  for (const entry of days) {
    if (entry.downloads > 0 && (peak === null || entry.downloads > peak.downloads)) {
      peak = entry;
    }
  }
  return peak;
}

export function componentName(component: Pick<UsageComponent, 'format' | 'group' | 'name'>): string {
  if (component.format === 'raw') {
    return component.name.replace(/^\/+/, '');
  }
  const group = component.group ?? '';
  if (group === '') {
    return component.name;
  }
  if (component.format === 'npm') {
    return `@${group.replace(/^@/, '')}/${component.name}`;
  }
  return `${group}:${component.name}`;
}
