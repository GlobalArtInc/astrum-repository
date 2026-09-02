

import { useState, useCallback } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { CrowdConfig, DEFAULT_CROWD_CONFIG } from './types';

const CROWD_API_URL = '/service/rest/v1/security/atlassian-crowd';

/**
 * Custom hook for Crowd API operations
 * Uses restClient for consistent API handling
 */
export function useCrowdApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch Crowd configuration
   */
  const fetchConfig = useCallback(async (): Promise<CrowdConfig> => {
    try {
      const data = await restClient.get<CrowdConfig>(CROWD_API_URL);
      return {
        ...DEFAULT_CROWD_CONFIG,
        ...data,
      };
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      console.error('Failed to fetch Crowd config:', err);
      throw new Error(apiError.message);
    }
  }, []);

  /**
   * Save Crowd configuration
   */
  const saveConfig = useCallback(async (config: CrowdConfig): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.put(CROWD_API_URL, config);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Verify connection to Crowd server
   */
  const verifyConnection = useCallback(async (config: CrowdConfig): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.post(`${CROWD_API_URL}/verify-connection`, config);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Clear Crowd cache
   */
  const clearCache = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.post(`${CROWD_API_URL}/clear-cache`);
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
    fetchConfig,
    saveConfig,
    verifyConnection,
    clearCache,
  };
}

export default useCrowdApi;


