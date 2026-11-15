import { useQueries } from '@tanstack/react-query';
import { api, apiKey, withQuery } from '../../../api';
import { UI_PATH } from '../types';
import type { PromotionTarget } from './model';

export function promotionTargetsPath(source: string): string {
  return withQuery(`${UI_PATH}/promotion-targets`, { source });
}

export function usePromotionTargets(sources: readonly string[]): Map<string, PromotionTarget[]> {
  const results = useQueries({
    queries: sources.map((source) => {
      const key = promotionTargetsPath(source);
      return {
        queryKey: apiKey(key),
        queryFn: ({ signal }: { signal: AbortSignal }) => api.get<PromotionTarget[]>(key, { signal }),
      };
    }),
  });
  return new Map(
    sources.map((source, index): [string, PromotionTarget[]] => [source, results[index]?.data ?? []]),
  );
}
