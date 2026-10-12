import { INTERNAL, path, useApi } from '../../api';

export const UI_BLOB_STORES = `${INTERNAL}/ui/blobstores`;
export const LIST_ADDRESS = '/admin/repository/blobstores';
export const CREATE_ADDRESS = '/admin/repository/blobstores/create';

export type StoreType = 'file' | 's3' | 'group';
export const STORE_TYPES: readonly StoreType[] = ['file', 's3', 'group'];

export interface SoftQuota {
  type: string;
  limit: number;
}

export interface UiBlobStore {
  name: string;
  type: string;
  path?: string;
  bucket?: string;
  region?: string;
  prefix?: string;
  members?: string[];
  fillPolicy?: string;
  unavailable: boolean;
  blobCount: number;
  totalSize: number;
  availableSpace: number | null;
  softQuota: SoftQuota | null;
  quotaStatus: { violation: boolean; message: string } | null;
  repositories: string[];
  repositoryCount: number;
  groups: string[];
  deletable: boolean;
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

export function deletePath(name: string): string {
  return path`/service/rest/v1/blobstores/${name}`;
}
