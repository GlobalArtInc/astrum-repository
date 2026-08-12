

import { useState, useEffect, useCallback } from 'react';
import { restClient } from '../../../../../interface/api';

const BLOB_STORES_URL = '/service/rest/internal/ui/blobstores';

interface BlobStoreRecord {
  unavailable?: boolean;
  totalSizeInBytes?: number;
}

/**
 * Returns current instance storage by summing totalSizeInBytes from all blob stores.
 * Use when monthly-metrics peakStorage is null (e.g. new instance, Cloud).
 */
export function useInstanceStorage(): {
  currentStorageBytes: number | null;
  loading: boolean;
  error: string | null;
} {
  const [state, setState] = useState<{
    currentStorageBytes: number | null;
    loading: boolean;
    error: string | null;
  }>({
    currentStorageBytes: null,
    loading: true,
    error: null,
  });

  const fetchStorage = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const data = await restClient.get<BlobStoreRecord[]>(BLOB_STORES_URL);
      const stores = Array.isArray(data) ? data : [];
      const total = stores.reduce((sum, s) => {
        if (s.unavailable) return sum;
        const size = s.totalSizeInBytes;
        return sum + (typeof size === 'number' && size > 0 ? size : 0);
      }, 0);
      setState({
        currentStorageBytes: total > 0 ? total : null,
        loading: false,
        error: null,
      });
    } catch {
      setState({
        currentStorageBytes: null,
        loading: false,
        error: null,
      });
    }
  }, []);

  useEffect(() => {
    fetchStorage();
  }, [fetchStorage]);

  return state;
}
