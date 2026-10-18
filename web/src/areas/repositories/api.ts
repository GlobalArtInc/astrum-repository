import { INTERNAL, path, type RepositoryDomain, type RepositoryType, useApi, V1, withQuery } from '../../api';

export const UI_REPOSITORIES = `${INTERNAL}/ui/repositories`;
export const RECIPES = `${INTERNAL}/ui/repositories/recipes`;
export const BLOB_STORES = `${V1}/blobstores`;
export const REPOSITORIES = `${V1}/repositories`;

export interface RepositoryFlags {
  browse: boolean;
  read: boolean;
  add: boolean;
  edit: boolean;
  delete: boolean;
  admin: boolean;
  adminEdit: boolean;
  adminDelete: boolean;
}

export interface UiRepository {
  name: string;
  format: string;
  type: RepositoryType;
  url: string;
  online: boolean;
  blobStoreName: string | null;
  remoteUrl: string | null;
  members?: string[];
  memberOf: string[];
  componentCount: number | null;
  assetCount: number | null;
  totalSize: number | null;
  lastUploaded: string | null;
  limited: boolean;
  permissions: RepositoryFlags;
  domain?: RepositoryDomain;
}

export interface Recipe {
  id: string;
  format: string;
  apiFormat: string;
  type: RepositoryType;
}

export interface BlobStoreSummary {
  name: string;
  type: string;
  unavailable: boolean;
}

export interface AuditEvent {
  id: number;
  at: string;
  domain: string;
  type: string;
  context: string;
  initiator: string;
  attributes: unknown;
}

export function useUiRepositories() {
  return useApi<UiRepository[]>(UI_REPOSITORIES);
}

export function useRecipes() {
  return useApi<Recipe[]>(RECIPES);
}

export function documentPath(apiFormat: string, type: RepositoryType, name: string): string {
  return path`/service/rest/v1/repositories/${apiFormat}/${type}/${name}`;
}

export function createPath(apiFormat: string, type: RepositoryType): string {
  return path`/service/rest/v1/repositories/${apiFormat}/${type}`;
}

export function repositoryPath(name: string): string {
  return path`/service/rest/v1/repositories/${name}`;
}

export function auditPath(name: string, limit: number): string {
  return withQuery(`${INTERNAL}/audit`, { repository: name, limit });
}

export function editAddress(name: string, tab?: string): string {
  const base = path`/admin/repository/repositories/${name}`;
  return tab ? `${base}?tab=${encodeURIComponent(tab)}` : base;
}

export function createAddress(format: string, type: RepositoryType): string {
  return path`/admin/repository/repositories/create/${format}/${type}`;
}

export function browseAddress(name: string): string {
  return path`/browse/${name}`;
}

export const LIST_ADDRESS = '/admin/repository/repositories';
export const CREATE_ADDRESS = '/admin/repository/repositories/create';
