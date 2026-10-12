import { useInfiniteQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import { type ApiError, api, withQuery } from '../../api';
import type { SearchPage } from './model';

export interface PagedSearch<T> {
  items: T[] | undefined;
  loading: boolean;
  error: ApiError | null;
  hasMore: boolean;
  loadingMore: boolean;
  loadMore(): void;
  retry(): void;
}

export function usePagedSearch<T>(path: string | null): PagedSearch<T> {
  const query = useInfiniteQuery<
    SearchPage<T>,
    ApiError,
    { pages: SearchPage<T>[] },
    readonly unknown[],
    string | null
  >({
    queryKey: ['api', path ?? '', 'pages'],
    queryFn: ({ pageParam, signal }) =>
      api.get<SearchPage<T>>(withQuery(path ?? '', { continuationToken: pageParam }), { signal }),
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
