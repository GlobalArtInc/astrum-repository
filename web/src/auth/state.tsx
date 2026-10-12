import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useMemo } from 'react';
import { type ApiError, api, forgetApi, type UiState, type UiUser } from '../api';
import { type Need, PermissionSet } from './permissions';
import { STATE_PATH } from './session';

export const STATE_KEY = ['ui', 'state'] as const;

export function useUiStateQuery() {
  return useQuery<UiState, ApiError>({
    queryKey: STATE_KEY,
    queryFn: ({ signal }) => api.get<UiState>(STATE_PATH, { signal, quietUnauthorized: true }),
    staleTime: 60_000,
    refetchOnWindowFocus: 'always',
    refetchInterval: 5 * 60_000,
  });
}

export interface Session {
  state: UiState;
  user: UiUser | null;
  anonymousAccess: boolean;
  permissions: PermissionSet;
  can(need: Need | undefined): boolean;
}

const EMPTY: UiState = {
  version: '',
  revision: null,
  user: null,
  anonymousAccess: false,
  permissions: [],
  formats: [],
  baseUrl: '',
  sessionTimeoutMinutes: 0,
  adminPasswordFile: null,
};

export function useSession(): Session {
  const { data } = useUiStateQuery();
  const state = data ?? EMPTY;
  return useMemo(() => {
    const permissions = new PermissionSet(state.permissions);
    return {
      state,
      user: state.user,
      anonymousAccess: state.anonymousAccess,
      permissions,
      can: (need: Need | undefined) => permissions.satisfies(need),
    };
  }, [state]);
}

export function useCan(need: Need | undefined): boolean {
  return useSession().can(need);
}

export function useRefreshSession(): () => Promise<UiState> {
  const client = useQueryClient();
  return useCallback(async () => {
    forgetApi(client);
    return client.fetchQuery<UiState>({
      queryKey: STATE_KEY,
      queryFn: () => api.get<UiState>(STATE_PATH, { quietUnauthorized: true }),
      staleTime: 0,
    });
  }, [client]);
}
