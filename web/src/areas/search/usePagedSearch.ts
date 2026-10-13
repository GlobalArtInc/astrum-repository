import { useInfiniteQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import { type ApiError, api, withQuery } from '../../api';
import type { SearchPage } from './model';

export const BATCH = 50;

export interface PagedSearch<T> {
  items: T[] | undefined;
  loading: boolean;
  error: ApiError | null;
  hasMore: boolean;
  loadingMore: boolean;
  loadMore(): void;
  retry(): void;
}

export async function fetchBatch<T>(
  path: string,
  token: string | null,
  size: number,
  signal?: AbortSignal,
): Promise<SearchPage<T>> {
  const items: T[] = [];
  let next = token;
  do {
    const page = await api.get<SearchPage<T>>(withQuery(path, { continuationToken: next }), { signal });
    items.push(...page.items);
    next = page.continuationToken ?? null;
  } while (next !== null && items.length < size);
  return { items, continuationToken: next };
}

export function usePagedSearch<T>(path: string | null, size = 1): PagedSearch<T> {
  const query = useInfiniteQuery<
    SearchPage<T>,
    ApiError,
    { pages: SearchPage<T>[] },
    readonly unknown[],
    string | null
  >({
    queryKey: ['api', path ?? '', 'pages', size],
    queryFn: ({ pageParam, signal }) => fetchBatch<T>(path ?? '', pageParam, size, signal),
    initialPageParam: null,
    getNextPageParam: (last) => last.continuationToken ?? undefined,
    enabled: path !== null,
    staleTime: 15_000,
  });
  const items = useMemo(() => query.data?.pages.flatMap((page) => page.items), [query.data]);
  return {
    items: path === null ? undefined : items,
    loading: path !== null && query.isPending,
    error: query.error,
    hasMore: query.hasNextPage,
    loadingMore: query.isFetchingNextPage,
    loadMore: () => {
      void query.fetchNextPage();
    },
    retry: () => {
      void query.refetch();
    },
  };
}
