

import { useState, useCallback } from 'react';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { LogFile, LOGS_API } from './types';

/**
 * Custom hook for Logs API operations
 */
export function useLogsApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch list of log files
   */
  const fetchLogs = useCallback(async (): Promise<LogFile[]> => {
    setLoading(true);
    setError(null);
    try {
      const data = await restClient.get<LogFile[]>(LOGS_API.LIST);
      return Array.isArray(data) ? data : [];
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || 'Failed to load log files';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Fetch log file content
   * @param filename - The log file name
   * @param bytesCount - Number of bytes to fetch (negative for last N bytes)
   */
  const fetchLogContent = useCallback(async (
    filename: string,
    bytesCount?: number
  ): Promise<string> => {
    try {
      const params: Record<string, number> = {};
      if (bytesCount !== undefined) {
        params.bytesCount = bytesCount;
      }

      const data = await restClient.get<string>(LOGS_API.VIEW(filename), {
        params,
        headers: { Accept: 'text/plain' },
      });
      return data || '';
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || 'Failed to load log content';
      throw new Error(message);
    }
  }, []);

  /**
   * Insert a mark into the astrum.log file
   * @param mark - The mark text to insert
   */
  const insertMark = useCallback(async (mark: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await restClient.post(LOGS_API.MARK, mark, {
        headers: { 'Content-Type': 'text/plain' },
      });
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || 'Failed to insert mark';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Get download URL for a log file
   */
  const getDownloadUrl = useCallback((filename: string): string => {
    return LOGS_API.VIEW(filename);
  }, []);

  return {
    loading,
    error,
    setError,
    fetchLogs,
    fetchLogContent,
    insertMark,
    getDownloadUrl,
  };
}

export default useLogsApi;
