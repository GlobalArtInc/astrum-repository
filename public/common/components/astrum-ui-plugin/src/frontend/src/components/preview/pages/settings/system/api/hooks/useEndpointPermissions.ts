

import { useCallback, useEffect, useRef, useState } from 'react';

import { ENDPOINTS, restClient } from '../../../../../../../interface/api';

import type { ApiPermissionsResponseDto } from '../types';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';

export interface UseEndpointPermissionsResult {
  data: ApiPermissionsResponseDto | null;
  loading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

let sessionCache: ApiPermissionsResponseDto | null = null;

/**
 * Loads {@code GET /service/rest/internal/ui/api/permissions} once per browser session (shared cache).
 */
export function useEndpointPermissions(): UseEndpointPermissionsResult {
  const [data, setData] = useState<ApiPermissionsResponseDto | null>(sessionCache);
  const [loading, setLoading] = useState(!sessionCache);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await restClient.get<ApiPermissionsResponseDto>(ENDPOINTS.API_PERMISSIONS);
      sessionCache = response;
      if (mounted.current) {
        setData(response);
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : ApiStrings.ENDPOINT_PERMISSIONS.FAILED_LOAD_API_PERMISSIONS;
      if (mounted.current) {
        setError(message);
        setData(null);
      }
    } finally {
      if (mounted.current) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    mounted.current = true;
    if (sessionCache) {
      setData(sessionCache);
      setLoading(false);
      return () => {
        mounted.current = false;
      };
    }
    void load();
    return () => {
      mounted.current = false;
    };
  }, [load]);

  const refetch = useCallback(async () => {
    sessionCache = null;
    await load();
  }, [load]);

  return { data, loading, error, refetch };
}
