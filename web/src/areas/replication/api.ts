import { useInfiniteQuery } from '@tanstack/react-query';
import { type ApiError, api, apiKey, type RepositoryType, useApi, V1, withQuery } from '../../api';
import { CHANNELS, type Channel } from '../notify/notifications/model';
import { REPOSITORIES } from '../repositories/api';

export const REMOTES = `${V1}/replication/remotes`;
export const REPLICATIONS = `${V1}/replications`;
export const COMPONENTS = `${V1}/components`;

export const READ = 'astrum:replication:read';
export const CREATE = 'astrum:replication:create';
export const UPDATE = 'astrum:replication:update';
export const DELETE = 'astrum:replication:delete';

export const STATUS_POLL = 30_000;
export const ITEMS_BATCH = 50;
export const REPLICATIONS_ADDRESS = '/admin/replication?tab=replications';

export const ITEM_STATES = ['dead', 'pending', 'leased'] as const;

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
  failures: number;
  suspendedUntil: string | null;
  lastError: string | null;
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

export interface ReplicationStatusRemote {
  suspendedUntil: string | null;
  failures: number;
  lastError: string | null;
}

export interface ReplicationStatus {
  enabled: boolean;
  pending: number;
  leased: number;
  dead: number;
  lagSeconds: number | null;
  lastSuccessAt: string | null;
  lastFailureAt: string | null;
  lastError: string | null;
  lastReconcileAt: string | null;
  alertState: string;
  remote: ReplicationStatusRemote;
}

export type ItemState = (typeof ITEM_STATES)[number];

export interface ReplicationItem {
  id: number;
  key: string;
  state: ItemState;
  reason: string;
  attempts: number;
  notBefore: string;
  firstQueuedAt: string;
  lastError: string | null;
}

export interface ItemPage {
  items: ReplicationItem[];
  next: number | null;
}

export interface ItemIds {
  ids: number[];
}

export interface ItemsAll {
  all: true;
}

export type ItemSelection = ItemIds | ItemsAll;

export interface RetryAnswer {
  retried: number;
}

export interface DiscardAnswer {
  discarded: number;
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

export interface RemoteCheckError {
  kind: string;
  message: string;
}

export interface RemoteCheckFailure {
  ok: false;
  error: RemoteCheckError;
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

export function replicationAddress(id: string): string {
  return `/admin/replication/replications/${encodeURIComponent(id)}`;
}

export function statusPath(id: string): string {
  return `${replicationPath(id)}/status`;
}

export function itemsPath(id: string, state: ItemState): string {
  return withQuery(`${replicationPath(id)}/items`, { state });
}

export function itemsActionPath(id: string, action: 'retry' | 'discard'): string {
  return `${replicationPath(id)}/items/${action}`;
}

export function reconcilePath(id: string): string {
  return `${replicationPath(id)}/reconcile`;
}

export function useReplicationRemotes() {
  return useApi<ReplicationRemote[]>(REMOTES);
}

export function useReplicationList() {
  return useApi<Replication[]>(REPLICATIONS, { refetchInterval: STATUS_POLL });
}

export function useReplicationStatus(id: string) {
  return useApi<ReplicationStatus>(statusPath(id), { refetchInterval: STATUS_POLL });
}

export function statusQuery(id: string) {
  const path = statusPath(id);
  return {
    queryKey: apiKey(path),
    queryFn: ({ signal }: { signal: AbortSignal }) => api.get<ReplicationStatus>(path, { signal }),
    refetchInterval: STATUS_POLL,
  };
}

export function useReplicationItems(id: string, state: ItemState) {
  const path = itemsPath(id, state);
  return useInfiniteQuery<ItemPage, ApiError, { pages: ItemPage[] }, readonly unknown[], number | null>({
    queryKey: apiKey(path),
    queryFn: ({ pageParam, signal }) =>
      api.get<ItemPage>(withQuery(path, { after: pageParam, limit: ITEMS_BATCH }), { signal }),
    initialPageParam: null,
    getNextPageParam: (last) => last.next ?? undefined,
  });
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
