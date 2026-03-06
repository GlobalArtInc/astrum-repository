import {
  QueryClient,
  type UseQueryOptions,
  type UseQueryResult,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import { useCallback } from 'react';
import { api } from './client';
import { ApiError } from './errors';

export const API_KEY = 'api';

export function apiKey(path: string): readonly [typeof API_KEY, string] {
  return [API_KEY, path];
}

export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        refetchOnWindowFocus: false,
        retry: (failures, error) => {
          if (error instanceof ApiError && error.status >= 400 && error.status < 500) {
            return false;
          }
          return failures < 2;
        },
      },
      mutations: { retry: false },
    },
  });
}

export function useApi<T>(
  path: string | null,
  options: Omit<
    UseQueryOptions<T, ApiError, T, readonly [typeof API_KEY, string]>,
    'queryKey' | 'queryFn'
  > = {},
): UseQueryResult<T, ApiError> {
  return useQuery({
    ...options,
    queryKey: apiKey(path ?? ''),
    queryFn: ({ signal }) => api.get<T>(path ?? '', { signal }),
    enabled: path !== null && (options.enabled ?? true),
  });
}

export function matchesApiPrefix(queryKey: readonly unknown[], prefix: string): boolean {
  return queryKey[0] === API_KEY && typeof queryKey[1] === 'string' && queryKey[1].startsWith(prefix);
}

export function invalidateApi(client: QueryClient, prefix: string): Promise<void> {
  return client.invalidateQueries({ predicate: (query) => matchesApiPrefix(query.queryKey, prefix) });
}

export function forgetApi(client: QueryClient): void {
  client.removeQueries({ predicate: (query) => query.queryKey[0] === API_KEY });
}

export function useInvalidateApi(): (prefix: string) => Promise<void> {
  const client = useQueryClient();
  return useCallback((prefix: string) => invalidateApi(client, prefix), [client]);
}
