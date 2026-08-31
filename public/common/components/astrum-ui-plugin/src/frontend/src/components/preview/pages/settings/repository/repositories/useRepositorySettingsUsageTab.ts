

import { useMemo, useCallback } from 'react';
import { useMachine } from '@xstate/react';

import { createRepositoryUsageMachine } from './repositoryUsageMachine';
import type { RepositoryUsageContext, RepositoryUsageKind } from './repositoryUsageMachine';

export interface UseRepositorySettingsUsageTabReturn {
  metrics: RepositoryUsageContext['metrics'];
  groupMembers: RepositoryUsageContext['groupMembers'];
  whereUsed: RepositoryUsageContext['whereUsed'];

  loading: boolean;
  loaded: boolean;
  error: string | null;
  membershipError: string | null;

  componentCountPending: boolean;
  assetCountPending: boolean;
  totalSizePending: boolean;

  refresh: () => void;
  retry: () => void;
}

export function useRepositorySettingsUsageTab(
  repositoryName: string,
  repositoryType: RepositoryUsageKind,
): UseRepositorySettingsUsageTabReturn {
  const machine = useMemo(
    () => createRepositoryUsageMachine({ repositoryName, repositoryType }),
    [repositoryName, repositoryType]
  );

  const [state, send] = useMachine(machine);

  const { context, matches } = state;

  const loading = matches('loading');
  const loaded = matches('loaded');
  const error = context.error;
  const membershipError = context.membershipError;

  const componentCountPending = context.metrics?.componentCount === undefined
    || context.metrics?.componentCount === null;
  const assetCountPending = context.metrics?.assetCount === undefined
    || context.metrics?.assetCount === null;
  const totalSizePending = context.metrics?.totalSize === undefined
    || context.metrics?.totalSize === null;

  const refresh = useCallback(() => {
    send({ type: 'REFRESH' });
  }, [send]);

  const retry = useCallback(() => {
    send({ type: 'RETRY' });
  }, [send]);

  return {
    metrics: context.metrics,
    groupMembers: context.groupMembers,
    whereUsed: context.whereUsed,

    loading,
    loaded,
    error,
    membershipError,

    componentCountPending,
    assetCountPending,
    totalSizePending,

    refresh,
    retry,
  };
}

export default useRepositorySettingsUsageTab;
