import { getPath, isObject, type JsonObject, type RepositoryDocument } from './document';

export const QUOTA_UNITS = ['KiB', 'MiB', 'GiB', 'TiB'] as const;
export type QuotaUnit = (typeof QUOTA_UNITS)[number];

export const DEFAULT_WARN_PERCENT = 90;

const BYTES: Record<QuotaUnit, number> = {
  KiB: 1024,
  MiB: 1024 ** 2,
  GiB: 1024 ** 3,
  TiB: 1024 ** 4,
};

export type UsageLevel = 'normal' | 'warning' | 'full';

export interface QuotaFields {
  limitBytes: number | null;
  warnPercent: number | null;
  enforce: boolean;
}

export function amountIn(bytes: number, unit: QuotaUnit): number {
  return bytes / BYTES[unit];
}

export function bytesIn(amount: number, unit: QuotaUnit): number {
  return Math.round(amount * BYTES[unit]);
}

export function bestUnit(bytes: number | null): QuotaUnit {
  if (bytes === null) {
    return 'GiB';
  }
  const exact = [...QUOTA_UNITS].reverse().find((unit) => bytes >= BYTES[unit] && bytes % BYTES[unit] === 0);
  return exact ?? 'KiB';
}

export function quotaFields(document: RepositoryDocument): QuotaFields {
  const quota = getPath(document, ['storage', 'quota']);
  if (!isObject(quota)) {
    return { limitBytes: null, warnPercent: DEFAULT_WARN_PERCENT, enforce: false };
  }
  return {
    limitBytes: typeof quota.limitBytes === 'number' ? quota.limitBytes : null,
    warnPercent: typeof quota.warnPercent === 'number' ? quota.warnPercent : DEFAULT_WARN_PERCENT,
    enforce: quota.enforce === true,
  };
}

export function quotaDocument(fields: QuotaFields): JsonObject | null {
  if (fields.limitBytes === null) {
    return null;
  }
  return { limitBytes: fields.limitBytes, warnPercent: fields.warnPercent, enforce: fields.enforce };
}

export function usageLevel(percent: number, warnPercent: number): UsageLevel {
  if (percent >= 100) {
    return 'full';
  }
  return percent >= warnPercent ? 'warning' : 'normal';
}
