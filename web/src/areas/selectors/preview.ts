import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { ApiError, api } from '../../api';
import { PREVIEW, type Preview } from './model';
import { useDebounced } from './useDebounced';

export interface PreviewState {
  data: Preview | undefined;
  problem: string | null;
  error: unknown;
  loading: boolean;
  checked: boolean;
}

export function expressionProblem(error: unknown): string | null {
  if (error instanceof ApiError && error.status === 400) {
    return error.fieldErrors.find((problem) => problem.id === 'expression')?.message ?? error.message;
  }
  return null;
}

export function usePreview(expression: string, repository: string): PreviewState {
  const settled = useDebounced(expression.trim(), 400);
  const query = useQuery<Preview, ApiError>({
    queryKey: ['selectors', 'preview', settled, repository],
    queryFn: ({ signal }) => api.post<Preview>(PREVIEW, { expression: settled, repository }, { signal }),
    enabled: settled !== '',
    retry: false,
    staleTime: 10_000,
    placeholderData: keepPreviousData,
  });
  const problem = settled === '' ? null : expressionProblem(query.error);
  return {
    data: settled === '' || query.isError ? undefined : query.data,
    problem,
    error: problem ? null : query.error,
    loading: settled !== '' && (query.isFetching || settled !== expression.trim()),
    checked: settled !== '' && query.isSuccess && !query.isPlaceholderData,
  };
}
