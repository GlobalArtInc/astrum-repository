import { useCallback, useEffect, useRef, useState } from 'react';
import { parseApiError } from '../../../../../interface/api';
import { fetchUsageDashboard } from './usageDashboard.api';
import type { UsageDashboardData } from './usageDashboard.types';

export interface UseUsageDashboardResult {
  /** The figures last read; kept while the next ones load, so that switching the period does not blank the page. */
  data: UsageDashboardData | null;
  loading: boolean;
  error: string | null;
  reload: () => void;
}

/**
 * Reads the usage of the repositories for a period and an optional repository, again whenever either changes. Answers
 * to earlier requests that arrive late are dropped.
 */
export function useUsageDashboard(days: number, repository: string | null): UseUsageDashboardResult {
  const [data, setData] = useState<UsageDashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);
  const latest = useRef(0);

  // biome-ignore lint/correctness/useExhaustiveDependencies: a new attempt reads the same figures again
  useEffect(() => {
    const request = ++latest.current;
    const controller = typeof AbortController === 'undefined' ? null : new AbortController();
    setLoading(true);
    setError(null);
    fetchUsageDashboard(days, repository, controller?.signal)
      .then((result) => {
        if (request === latest.current) {
          setData(result);
        }
      })
      .catch((err) => {
        if (request === latest.current && !controller?.signal.aborted) {
          setError(parseApiError(err).message);
        }
      })
      .finally(() => {
        if (request === latest.current) {
          setLoading(false);
        }
      });
    return () => controller?.abort();
  }, [days, repository, attempt]);

  const reload = useCallback(() => setAttempt((value) => value + 1), []);

  return { data, loading, error, reload };
}

export default useUsageDashboard;
