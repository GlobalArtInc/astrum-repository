

import { useState, useEffect } from 'react';
import { restClient } from '../../../../interface/api';
import SharedStrings from '../../constants/SharedStrings';

export interface RepositoryMetrics {
  componentCount: number;
  assetCount: number;
  totalSize: number;
}

export interface UseRepositoryMetricsResult {
  data: RepositoryMetrics | null;
  isLoading: boolean;
  error: string | null;
}

interface RepositoryDetail {
  name: string;
  componentCount?: number;
  assetCount?: number;
  size?: number;
}

// TODO: /details returns every repo the user can read; the ?name= param is ignored
// server-side. Add a per-name backend endpoint or reuse a parent-fetched list for large installs.
async function fetchMetrics(repositoryName: string): Promise<RepositoryMetrics> {
  try {
    const details = await restClient.get<RepositoryDetail[]>('/service/rest/internal/ui/repositories/details');
    const detail = Array.isArray(details) ? details.find((r) => r.name === repositoryName) : null;

    return {
      componentCount: detail?.componentCount || 0,
      assetCount: detail?.assetCount || 0,
      totalSize: detail?.size || 0,
    };
  } catch (err) {
    console.warn('Could not fetch repository metrics:', err);
    throw err;
  }
}

export function useRepositoryMetrics(repositoryName: string): UseRepositoryMetricsResult {
  const [data, setData] = useState<RepositoryMetrics | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!repositoryName) {
      setIsLoading(false);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    fetchMetrics(repositoryName)
      .then((result) => {
        if (!cancelled) {
          setData(result);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          const errorMessage = err instanceof Error ? err.message : SharedStrings.REPOSITORY_METRICS.FAILED_FETCH_REPOSITORY_METRICS;
          setError(errorMessage);
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [repositoryName]);

  return { data, isLoading, error };
}
