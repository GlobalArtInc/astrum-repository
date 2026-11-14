import { INTERNAL, path, useApi } from '../../api';

export const UI_BLOB_STORES = `${INTERNAL}/ui/blobstores`;
export const LIST_ADDRESS = '/admin/repository/blobstores';
export const CREATE_ADDRESS = '/admin/repository/blobstores/create';

export type StoreType = 'file' | 's3' | 'remote' | 'group';
export const STORE_TYPES: readonly StoreType[] = ['file', 's3', 'remote', 'group'];

export interface SoftQuota {
  type: string;
  limit: number;
}

export type ForecastBasis = 'capacity' | 'softQuota';

export interface StoreForecast {
  daysUntilFull: number | null;
  bytesPerDay: number | null;
  basis: ForecastBasis | null;
  sampleDays: number;
}

export interface UiBlobStore {
  name: string;
  type: string;
  path?: string;
  bucket?: string;
  region?: string;
  prefix?: string;
  url?: string;
  members?: string[];
  fillPolicy?: string;
  unavailable: boolean;
  blobCount: number;
  totalSize: number;
  availableSpace: number | null;
  softQuota: SoftQuota | null;
  quotaStatus: { violation: boolean; message: string } | null;
  forecast: StoreForecast | null;
  repositories: string[];
  repositoryCount: number;
  groups: string[];
  deletable: boolean;
}

export function daysUntilFull(store: UiBlobStore): number | null {
  const days = store.forecast?.daysUntilFull;
  return days === null || days === undefined ? null : Math.max(1, Math.ceil(days));
}

export function storeType(value: string): StoreType | null {
  const lower = value.toLowerCase();
  return (STORE_TYPES as readonly string[]).includes(lower) ? (lower as StoreType) : null;
}

export function useUiBlobStores() {
  return useApi<UiBlobStore[]>(UI_BLOB_STORES);
}

export function storeAddress(type: StoreType, name: string): string {
  return path`/admin/repository/blobstores/${type}/${name}`;
}

export function createAddress(type: StoreType): string {
  return path`/admin/repository/blobstores/create/${type}`;
}

export function configPath(type: StoreType, name?: string): string {
  return name === undefined
    ? path`/service/rest/v1/blobstores/${type}`
    : path`/service/rest/v1/blobstores/${type}/${name}`;
}

export function convertPath(name: string, member: string): string {
  return path`/service/rest/v1/blobstores/group/convert/${name}/${member}`;
}

export function deletePath(name: string): string {
  return path`/service/rest/v1/blobstores/${name}`;
}
