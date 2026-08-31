

import { createMachine, assign } from 'xstate';

import { restClient, parseApiError } from '../../../../../../interface/api';

export type RepositoryUsageKind = 'hosted' | 'proxy' | 'group';

export interface RepositoryMetrics {
  componentCount: number | undefined;
  assetCount: number | undefined;
  totalSize: number | undefined;
}

export interface RepositoryUsageContext {
  repositoryName: string;
  repositoryType: RepositoryUsageKind;
  metrics: RepositoryMetrics | null;
  groupMembers: string[];
  whereUsed: string[];
  error: string | null;
  membershipError: string | null;
}

export type RepositoryUsageEvent =
  | { type: 'REFRESH' }
  | { type: 'RETRY' };

const REPOSITORIES_DETAILS_URL = '/service/rest/internal/ui/repositories/details';
const REPOSITORIES_REST_URL = '/service/rest/v1/repositories';
const REPOSITORIES_LIST_URL = '/service/rest/internal/ui/repositories';

async function fetchMetrics(repositoryName: string): Promise<RepositoryMetrics> {
  const details = await restClient.get<Array<{
    name: string;
    componentCount?: number;
    assetCount?: number;
    size?: number;
  }>>(
    `${REPOSITORIES_DETAILS_URL}?name=${encodeURIComponent(repositoryName)}`
  );
  const detail = Array.isArray(details) ? details.find((r) => r.name === repositoryName) : null;

  return {
    componentCount: detail?.componentCount,
    assetCount: detail?.assetCount,
    totalSize: detail?.size,
  };
}

async function fetchGroupMembers(repositoryName: string): Promise<string[]> {
  const repo = await restClient.get<{
    name: string;
    type: string;
    group?: { memberNames?: string[] };
  }>(`${REPOSITORIES_REST_URL}/${encodeURIComponent(repositoryName)}`);

  return repo?.group?.memberNames || [];
}

async function fetchWhereUsed(repositoryName: string): Promise<string[]> {
  const allRepos = await restClient.get<Array<{
    name: string;
    type: string;
  }>>(REPOSITORIES_LIST_URL);

  if (!Array.isArray(allRepos)) {
    return [];
  }

  const groups = allRepos.filter(r => r.type === 'group');

  const groupDetails = await Promise.all(
    groups.map(g => restClient.get<{
      name: string;
      group?: { memberNames?: string[] };
    }>(`${REPOSITORIES_REST_URL}/${encodeURIComponent(g.name)}`))
  );

  return groupDetails
    .filter(detail => detail?.group?.memberNames?.includes(repositoryName) ?? false)
    .map(detail => detail.name);
}

export interface RepositoryUsageMachineOptions {
  repositoryName: string;
  repositoryType: RepositoryUsageKind;
}

interface FetchDataResult {
  metrics: RepositoryMetrics | null;
  metricsError: string | null;
  groupMembers: string[];
  whereUsed: string[];
  membershipError: string | null;
}

export function createRepositoryUsageMachine(options: RepositoryUsageMachineOptions) {
  const { repositoryName, repositoryType } = options;

  return createMachine<RepositoryUsageContext, RepositoryUsageEvent>(
    {
      id: 'repositoryUsage',
      initial: 'loading',
      context: {
        repositoryName,
        repositoryType,
        metrics: null,
        groupMembers: [],
        whereUsed: [],
        error: null,
        membershipError: null,
      },
      states: {
        loading: {
          invoke: {
            src: 'fetchData',
            onDone: [
              {
                target: 'error',
                cond: 'allFailed',
                actions: 'assignData',
              },
              {
                target: 'loaded',
                actions: 'assignData',
              },
            ],
          },
        },
        loaded: {
          on: {
            REFRESH: 'loading',
          },
        },
        error: {
          on: {
            RETRY: 'loading',
          },
        },
      },
    },
    {
      services: {
        fetchData: async (): Promise<FetchDataResult> => {
          const membershipPromise: Promise<string[]> = repositoryType === 'group'
            ? fetchGroupMembers(repositoryName)
            : fetchWhereUsed(repositoryName);

          const [metricsResult, membershipResult] = await Promise.allSettled([
            fetchMetrics(repositoryName),
            membershipPromise,
          ]);

          const metrics = metricsResult.status === 'fulfilled' ? metricsResult.value : null;
          const metricsError = metricsResult.status === 'rejected'
            ? parseApiError(metricsResult.reason).message
            : null;

          const membershipValue = membershipResult.status === 'fulfilled' ? membershipResult.value : [];
          const membershipError = membershipResult.status === 'rejected'
            ? parseApiError(membershipResult.reason).message
            : null;

          return {
            metrics,
            metricsError,
            groupMembers: repositoryType === 'group' ? membershipValue : [],
            whereUsed: repositoryType === 'group' ? [] : membershipValue,
            membershipError,
          };
        },
      },
      guards: {
        allFailed: (_context, event) => {
          const data = event.data as FetchDataResult;
          return data.metricsError !== null && data.membershipError !== null;
        },
      },
      actions: {
        assignData: assign({
          metrics: (_context, event) => (event.data as FetchDataResult).metrics,
          groupMembers: (_context, event) => (event.data as FetchDataResult).groupMembers,
          whereUsed: (_context, event) => (event.data as FetchDataResult).whereUsed,
          error: (_context, event) => (event.data as FetchDataResult).metricsError,
          membershipError: (_context, event) => (event.data as FetchDataResult).membershipError,
        }),
      },
    }
  );
}
