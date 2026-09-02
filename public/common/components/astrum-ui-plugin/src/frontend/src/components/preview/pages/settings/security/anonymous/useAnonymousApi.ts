

import { useState, useCallback } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { AnonymousSettings, RealmType } from './types';

const { REST } = APIConstants;

/**
 * Custom hook for Anonymous settings API operations
 */
export function useAnonymousApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch available realm types for the anonymous user dropdown
   */
  const fetchRealmTypes = useCallback(async (): Promise<RealmType[]> => {
    try {
      // restClient.get() returns data directly, not Axios response
      const data = await restClient.get<RealmType[]>(REST.INTERNAL.REALMS_TYPES);
      return Array.isArray(data) ? data : [];
    } catch (err: unknown) {
      console.error('Failed to fetch realm types:', err);
      const apiError = parseApiError(err);
      throw new Error(apiError.message);
    }
  }, []);

  /**
   * Fetch anonymous settings
   */
  const fetchSettings = useCallback(async (): Promise<AnonymousSettings> => {
    try {
      // restClient.get() returns data directly, not Axios response
      const data = await restClient.get<AnonymousSettings>(REST.INTERNAL.ANONYMOUS_SETTINGS);
      return data;
    } catch (err: unknown) {
      console.error('Failed to fetch anonymous settings:', err);
      const apiError = parseApiError(err);
      throw new Error(apiError.message);
    }
  }, []);

  /**
   * Update anonymous settings
   */
  const saveSettings = useCallback(async (settings: AnonymousSettings): Promise<AnonymousSettings> => {
    setLoading(true);
    setError(null);
    try {
      // restClient.put() returns data directly, not Axios response
      const data = await restClient.put<AnonymousSettings>(REST.INTERNAL.ANONYMOUS_SETTINGS, {
        ...settings,
        userId: settings.userId.trim(),
      });
      return data;
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
    fetchRealmTypes,
    fetchSettings,
    saveSettings,
  };
}

export default useAnonymousApi;


