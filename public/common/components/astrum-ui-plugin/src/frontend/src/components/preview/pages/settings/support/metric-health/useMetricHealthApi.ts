

import { useState, useCallback } from 'react';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient, parseApiError } from '../../../../../../interface/api';

import { MetricHealthData, HealthCheck, NodeInfo } from './types';
import MetricHealthStrings from '../../../../constants/pages/settings/support/MetricHealthStrings';

// API endpoints - matches Default UI implementation
const STATUS_CHECK_BASE_URL = 'service/rest/beta/status/check';
const STATUS_CHECK_CLUSTER_URL = `${STATUS_CHECK_BASE_URL}/cluster`;
const STATUS_CHECK_INTERNAL_URL = APIConstants.REST.INTERNAL.GET_STATUS;

/**
 * Get status check URL for a specific node (clustered mode)
 */
function getNodeStatusUrl(nodeId: string): string {
  return `${STATUS_CHECK_BASE_URL}/${nodeId}`;
}

/**
 * Custom hook for Metric Health API operations
 */
export function useMetricHealthApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch metric health data (non-clustered mode)
   * Uses the internal status-check endpoint for single node
   */
  const fetchMetricHealth = useCallback(async (): Promise<HealthCheck[]> => {
    setLoading(true);
    setError(null);
    try {
      const data = await restClient.get<MetricHealthData>(STATUS_CHECK_INTERNAL_URL);

      // Convert object to array of health checks
      const checks: HealthCheck[] = Object.entries(data || {}).map(([name, result]) => ({
        name,
        result: result || { healthy: false },
      }));

      return checks;
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || MetricHealthStrings.API.FAILED_LOAD_METRIC_HEALTH_DATA;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Fetch cluster nodes list (clustered mode)
   */
  const fetchClusterNodes = useCallback(async (): Promise<NodeInfo[]> => {
    try {
      const data = await restClient.get<NodeInfo[]>(STATUS_CHECK_CLUSTER_URL);
      if (Array.isArray(data)) {
        return data;
      }
      return [];
    } catch (_err: any) {
      return [];
    }
  }, []);

  /**
   * Fetch metric health data for a specific node (clustered mode)
   */
  const fetchNodeMetricHealth = useCallback(async (nodeId: string): Promise<HealthCheck[]> => {
    setLoading(true);
    setError(null);
    try {
      const data = await restClient.get<MetricHealthData & { results?: MetricHealthData }>(getNodeStatusUrl(nodeId));

      // Handle clustered response format which may have a results property
      const healthData: MetricHealthData = data?.results || data || {};

      // Convert object to array of health checks
      const checks: HealthCheck[] = Object.entries(healthData).map(([name, result]) => ({
        name,
        result: result || { healthy: false },
      }));

      return checks;
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || MetricHealthStrings.API.FAILED_LOAD_NODE_METRIC_HEALTH;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Download metric health data as JSON file
   */
  const downloadMetricHealth = useCallback((checks: HealthCheck[], filename: string = 'metric-health.json') => {
    const data = checks.reduce((acc, check) => {
      acc[check.name] = check.result;
      return acc;
    }, {} as MetricHealthData);

    const jsonStr = JSON.stringify(data, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }, []);

  return {
    loading,
    error,
    setError,
    fetchMetricHealth,
    fetchClusterNodes,
    fetchNodeMetricHealth,
    downloadMetricHealth,
  };
}

export default useMetricHealthApi;


