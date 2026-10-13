import { INTERNAL, REST, V1 } from '../../api';

export const LOGS = `${INTERNAL}/logging/logs`;
export const MARK = `${INTERNAL}/logging/log/mark`;
export const LOGGERS = `${INTERNAL}/ui/loggingConfiguration`;
export const CHECKS = `${INTERNAL}/ui/status-check`;
export const SYSTEM_INFORMATION = `${REST}/atlas/system-information`;
export const SUPPORT_ZIP = `${V1}/support/supportzippath`;
export const DOWNLOAD = `${REST}/wonderland/download`;
export const DOCS = 'http://links.globalart.dev/products/astrum/docs';

export const LOGS_PAGE = '/admin/support/logs';
export const LOGGING_PAGE = '/admin/support/logging';
export const HEALTH_PAGE = '/admin/support/metric-health';

export const LOGGING_READ = 'astrum:logging:read';
export const LOGGING_UPDATE = 'astrum:logging:update';
export const LOGGING_MARK = 'astrum:logging:create';
export const ATLAS_READ = 'astrum:atlas:read';
export const METRICS_READ = 'astrum:metrics:read';

export interface LogFile {
  fileName: string;
  size: number;
  lastModified: number | null;
  node: string;
  current: boolean;
}

export const LEVELS = ['OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'] as const;
export type Level = (typeof LEVELS)[number];

export interface Logger {
  name: string;
  level: Level;
  override: boolean;
}

export interface HealthCheck {
  healthy: boolean;
  message: string | null;
  error: string | null;
  details: unknown;
  time: number;
  duration: number;
  timestamp: string;
}

export type HealthChecks = Record<string, HealthCheck>;

export interface CreatedZip {
  file: string;
  name: string;
  size: number;
  truncated: boolean;
}

export function failing(checks: HealthChecks | undefined): number {
  return Object.values(checks ?? {}).filter((check) => !check.healthy).length;
}

export function saveJson(name: string, value: unknown) {
  const blob = new Blob([`${JSON.stringify(value, null, 2)}\n`], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  document.body.append(link);
  link.click();
  link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function sectionTitle(key: string): string {
  return key
    .split('-')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
    .replace(/\bOs\b/, 'OS');
}

export function flatten(value: unknown, prefix = ''): Array<[string, string]> {
  if (value === null || typeof value !== 'object') {
    return [[prefix, value === null || value === undefined ? '' : String(value)]];
  }
  const entries = Array.isArray(value)
    ? value.map((item, index) => [String(index), item] as const)
    : Object.entries(value as Record<string, unknown>);
  if (entries.length === 0) {
    return prefix ? [[prefix, '']] : [];
  }
  return entries.flatMap(([key, item]) => flatten(item, prefix ? `${prefix}.${key}` : key));
}
