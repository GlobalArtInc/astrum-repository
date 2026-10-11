

import { useState, useCallback } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { Logger, LogLevel, LOGGING_CONFIG_API } from './types';
import LoggingStrings from '../../../../constants/pages/settings/support/LoggingStrings';

/**
 * Custom hook for Logging Configuration API operations
 */
export function useLoggingConfigApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch all loggers
   */
  const fetchLoggers = useCallback(async (): Promise<Logger[]> => {
    setLoading(true);
    setError(null);
    try {
      const data = await restClient.get<Logger[]>(LOGGING_CONFIG_API.LIST);
      return Array.isArray(data) ? data : [];
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || LoggingStrings.API.FAILED_LOAD_LOGGERS;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Fetch a single logger by name
   */
  const fetchLogger = useCallback(async (name: string): Promise<Logger | null> => {
    try {
      const data = await restClient.get<Logger>(LOGGING_CONFIG_API.GET(name));
      return data || null;
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || LoggingStrings.API.FAILED_LOAD_LOGGER;
      throw new Error(message);
    }
  }, []);

  /**
   * Update or create a logger
   */
  const updateLogger = useCallback(async (name: string, level: LogLevel): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.put(LOGGING_CONFIG_API.UPDATE(name), { level });
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || LoggingStrings.API.FAILED_UPDATE_LOGGER;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Reset a single logger to default level
   */
  const resetLogger = useCallback(async (name: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.post(LOGGING_CONFIG_API.RESET(name));
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || LoggingStrings.API.FAILED_RESET_LOGGER;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Reset all loggers to default levels
   */
  const resetAllLoggers = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.post(LOGGING_CONFIG_API.RESET_ALL);
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || LoggingStrings.API.FAILED_RESET_ALL_LOGGERS;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    loading,
    error,
    setError,
    fetchLoggers,
    fetchLogger,
    updateLogger,
    resetLogger,
    resetAllLoggers,
  };
}

export default useLoggingConfigApi;


