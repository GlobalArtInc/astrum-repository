

import type { ApiEndpointPermissionDto } from '../types';

import { userImpliesPermission } from './shiroWildcardImplies';

export type EndpointAccessDot = 'granted' | 'denied' | 'partial' | 'unknown';

/**
 * Client-side access indicator for the current user (Astrum permission map + Shiro-style wildcards).
 */
export function computeEndpointAccess(
  meta: ApiEndpointPermissionDto | null | undefined,
  userPermissionMap: Record<string, boolean>
): EndpointAccessDot {
  if (!meta) {
    return 'unknown';
  }
  const reqs = meta.permissions ?? [];
  if (reqs.length === 0) {
    if (!meta.authenticated) {
      return 'granted';
    }
    return 'unknown';
  }

  const logical = reqs[0]?.logical === 'OR' ? 'OR' : 'AND';
  const checks = reqs.map((r) =>
    userImpliesPermission((r.permission || '').toLowerCase(), userPermissionMap)
  );

  if (logical === 'OR') {
    return checks.some(Boolean) ? 'granted' : 'denied';
  }

  const ok = checks.filter(Boolean).length;
  if (ok === reqs.length) {
    return 'granted';
  }
  if (ok === 0) {
    return 'denied';
  }
  return 'partial';
}
