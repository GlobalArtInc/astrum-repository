

import { useEffect, useState } from 'react';

import { ENDPOINTS, restClient } from '../../../../../../../interface/api';

import type { ApiAccessCheckResponseDto } from '../types';
import type { EndpointAccessDot } from '../utils/endpointAccess';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import { endpointRowId } from '../EndpointList';

const BATCH = 14;

export interface UseViewAsUserAccessResult {
  accessById: Record<string, EndpointAccessDot> | null;
  loading: boolean;
  error: string | null;
}

/**
 * When {@code viewAsUserId} differs from the signed-in user, loads per-endpoint access via
 * POST /internal/ui/security/access-check (admin-only for other users).
 */
export function useViewAsUserAccess(
  endpoints: MergedApiEndpoint[],
  viewAsUserId: string | null,
  currentUserId: string | null,
  enabled: boolean
): UseViewAsUserAccessResult {
  const [accessById, setAccessById] = useState<Record<string, EndpointAccessDot> | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const needsServer =
    enabled &&
    Boolean(viewAsUserId) &&
    Boolean(currentUserId) &&
    viewAsUserId !== currentUserId;

  useEffect(() => {
    if (!needsServer) {
      setAccessById(null);
      setLoading(false);
      setError(null);
      return undefined;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);
    setAccessById(null);

    (async () => {
      const map: Record<string, EndpointAccessDot> = {};
      if (endpoints.length === 0) {
        if (!cancelled) {
          setAccessById({});
        }
        return;
      }
      for (let i = 0; i < endpoints.length; i += BATCH) {
        if (cancelled) {
          return;
        }
        const slice = endpoints.slice(i, i + BATCH);
        const batch = await Promise.all(
          slice.map(async (row) => {
            const id = endpointRowId(row);
            try {
              const res = await restClient.post<ApiAccessCheckResponseDto>(ENDPOINTS.SECURITY_ACCESS_CHECK, {
                userId: viewAsUserId,
                endpoint: row.fullPath,
                method: row.httpMethod,
              });
              return { id, dot: (res.hasAccess ? 'granted' : 'denied') as EndpointAccessDot };
            } catch {
              return { id, dot: 'unknown' as EndpointAccessDot };
            }
          })
        );
        for (const { id, dot } of batch) {
          map[id] = dot;
        }
      }
      if (!cancelled) {
        setAccessById(map);
      }
    })()
      .catch((e: unknown) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : 'Access check failed');
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [needsServer, viewAsUserId, endpoints]);

  return { accessById: needsServer ? accessById : null, loading: needsServer && loading, error };
}
