import { type RepositoryType, useApi, V1, withQuery } from '../../api';
import { CHANNELS, type Channel } from '../notify/notifications/model';
import { REPOSITORIES } from '../repositories/api';

export const REMOTES = `${V1}/replication/remotes`;
export const REPLICATIONS = `${V1}/replications`;
export const COMPONENTS = `${V1}/components`;

export const READ = 'astrum:replication:read';
export const CREATE = 'astrum:replication:create';
export const UPDATE = 'astrum:replication:update';
export const DELETE = 'astrum:replication:delete';

export interface ReplicationRemote {
  id: string;
  name: string;
  url: string;
  hasToken: boolean;
  useTrustStore: boolean;
  allowPrivateNetwork: boolean;
  maxConcurrency: number;
  maxBytesPerSecond: number | null;
  remoteInstanceId: string | null;
  remoteVersion: string | null;
  checkedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface Replication {
  id: string;
  name: string;
  sourceRepository: string;
  remoteId: string;
  remoteRepository: string;
  enabled: boolean;
  nameFilter: string | null;
  versionFilter: string | null;
  propagateDeletes: boolean;
  settleSeconds: number;
  reconcileMinutes: number;
  alertChannels: string[];
  alertState: string;
  lastSuccessAt: string | null;
  lastFailureAt: string | null;
  lastError: string | null;
  lastReconcileAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface RemoteBody {
  name: string;
  url: string;
  token?: string;
  useTrustStore: boolean;
  allowPrivateNetwork: boolean;
  maxConcurrency: number;
  maxBytesPerSecond: number | null;
}

export interface ReplicationBody {
  name: string;
  sourceRepository: string;
  remoteId: string;
  remoteRepository: string;
  enabled: boolean;
  nameFilter: string | null;
  versionFilter: string | null;
  propagateDeletes: boolean;
  settleSeconds: number;
  reconcileMinutes: number;
  alertChannels: string[];
}

export interface RemoteCheckOk {
  ok: true;
  protocol: number;
  instanceId: string;
  version: string;
  formats: string[];
}

export interface RemoteCheckFailure {
  ok: false;
  error: { kind: string; message: string };
}

export type RemoteCheck = RemoteCheckOk | RemoteCheckFailure;

export interface ComponentItem {
  repository: string;
  format: string;
  group: string | null;
  name: string;
  version: string;
}

export interface ComponentPage {
  items: ComponentItem[];
  continuationToken: string | null;
}

export interface SourceRepository {
  name: string;
  format: string;
  type: RepositoryType;
}

export function remotePath(id: string): string {
  return `${REMOTES}/${encodeURIComponent(id)}`;
}

export function remoteCheckPath(id: string): string {
  return `${remotePath(id)}/check`;
}

export function replicationPath(id: string): string {
  return `${REPLICATIONS}/${encodeURIComponent(id)}`;
}

export function useReplicationRemotes() {
  return useApi<ReplicationRemote[]>(REMOTES);
}

export function useReplicationList() {
  return useApi<Replication[]>(REPLICATIONS);
}

export function useSourceRepositories() {
  return useApi<SourceRepository[]>(REPOSITORIES);
}

export function useSourceComponents(repository: string) {
  return useApi<ComponentPage>(repository === '' ? null : withQuery(COMPONENTS, { repository }));
}

export function useAlertChannels() {
  return useApi<Channel[]>(CHANNELS);
}
