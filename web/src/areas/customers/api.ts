import { path, type RepositoryType, V1, withQuery } from '../../api';

export const CUSTOMERS = `${V1}/customers`;
export const CUSTOMERS_PAGE = '/admin/customers';
export const REPOSITORIES = `${V1}/repositories`;
export const USAGE_CSV = `${CUSTOMERS}/usage.csv`;

export interface Customer {
  id: string;
  name: string;
  reference: string;
  note: string;
  disabled: boolean;
  createdAt: string;
  createdBy: string;
  tokens: number;
  nearestExpiry: string | null;
  downloads30d: number;
  bytes30d: number;
  lastAccessAt: string | null;
}

export interface CustomerToken {
  id: string;
  name: string;
  hint: string;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
  lastUsedIp: string | null;
  owner: string;
  repositories: string[];
  createdBy: string | null;
  expiryNotice: number | null;
}

export interface RevokeResult {
  revoked: number;
}

export interface IssuedToken {
  id: string;
  name: string;
  hint: string;
  token: string;
  expiresAt: string | null;
  owner: string;
  repositories: string[];
}

export interface Repository {
  name: string;
  format: string;
  type: RepositoryType;
}

export interface UsageTotals {
  downloads: number;
  bytes: number;
}

export interface UsageRow {
  day: string;
  repository: string;
  downloads: number;
  bytes: number;
}

export interface UsageReport {
  customer: string;
  days: number;
  totals: UsageTotals;
  daily: UsageRow[];
}

export function customerUrl(id: string): string {
  return `${CUSTOMERS}${path`/${id}`}`;
}

export function tokensUrl(id: string): string {
  return `${customerUrl(id)}/tokens`;
}

export function usageUrl(id: string): string {
  return `${customerUrl(id)}/usage`;
}

export function usageCsvHref(id: string, days: number): string {
  return withQuery(USAGE_CSV, { customer: id, days });
}

export function customerPageHref(id: string): string {
  return `${CUSTOMERS_PAGE}/${encodeURIComponent(id)}`;
}
