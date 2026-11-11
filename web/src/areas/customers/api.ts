import { path, V1 } from '../../api';

export const CUSTOMERS = `${V1}/customers`;
export const CUSTOMERS_PAGE = '/admin/customers';

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

export function customerUrl(id: string): string {
  return `${CUSTOMERS}${path`/${id}`}`;
}

export function tokensUrl(id: string): string {
  return `${customerUrl(id)}/tokens`;
}

export function customerPageHref(id: string): string {
  return `${CUSTOMERS_PAGE}/${encodeURIComponent(id)}`;
}
