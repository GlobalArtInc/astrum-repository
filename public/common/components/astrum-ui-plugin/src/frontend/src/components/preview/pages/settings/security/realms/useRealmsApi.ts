

import { useState, useCallback } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { Realm } from './types';

const { REST } = APIConstants;

/**
 * Custom hook for Realms API operations
 */
export function useRealmsApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch all available realms
   */
  const fetchAvailableRealms = useCallback(async (): Promise<Realm[]> => {
    try {
      // restClient.get() returns data directly, not Axios response
      const data = await restClient.get<Realm[]>(REST.PUBLIC.AVAILABLE_REALMS);
      return Array.isArray(data) ? data : [];
    } catch (err: unknown) {
      console.error('Failed to fetch available realms:', err);
      const apiError = parseApiError(err);
      throw new Error(apiError.message);
    }
  }, []);

  /**
   * Fetch active (configured) realm IDs
   */
  const fetchActiveRealmIds = useCallback(async (): Promise<string[]> => {
    try {
      // restClient.get() returns data directly, not Axios response
      const data = await restClient.get<string[]>(REST.PUBLIC.ACTIVE_REALMS);
      return Array.isArray(data) ? data : [];
    } catch (err: unknown) {
      console.error('Failed to fetch active realms:', err);
      const apiError = parseApiError(err);
      throw new Error(apiError.message);
    }
  }, []);

  /**
   * Update active realm IDs (order matters)
   */
  const updateActiveRealms = useCallback(async (realmIds: string[]): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.put(REST.PUBLIC.ACTIVE_REALMS, realmIds);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    loading,
    error,
    setError,
    fetchAvailableRealms,
    fetchActiveRealmIds,
    updateActiveRealms,
  };
}

export default useRealmsApi;



