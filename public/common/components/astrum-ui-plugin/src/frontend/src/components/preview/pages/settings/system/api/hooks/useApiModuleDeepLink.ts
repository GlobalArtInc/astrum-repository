

import { useEffect, useMemo, useState } from 'react';
import { ExtJS } from '../../../../../../../interface/ExtJS';

import {
  DEEP_LINK_ID_PATTERN,
  parseApiModuleHashParams,
} from '../utils/apiModuleDeepLinkParams';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';

export interface ApiModuleDeepLinkState {
  /** Raw parsed params from the hash query string */
  raw: ReturnType<typeof parseApiModuleHashParams>;
  /** Human-readable issues (invalid param, missing admin for role lens, etc.) */
  warnings: string[];
  /** User id for view-as mode (server access-check), or null */
  viewAsUserId: string | null;
  /** Role id for role lens, or null if gated / invalid */
  roleLensId: string | null;
  permissionFilter: string | null;
  endpointParam: string | null;
}

function isAdmin(): boolean {
  try {
    return ExtJS.checkPermission('astrum:*');
  } catch {
    return false;
  }
}

function buildDeepLinkState(hash: string): ApiModuleDeepLinkState {
  const raw = parseApiModuleHashParams(hash);
  const warnings: string[] = [];
  let viewAsUserId: string | null = null;
  let roleLensId: string | null = null;

  if (raw.user) {
    if (!DEEP_LINK_ID_PATTERN.test(raw.user)) {
      warnings.push(ApiStrings.DEEP_LINK.IGNORING_INVALID_USER_QUERY_PARAMETER);
    } else {
      viewAsUserId = raw.user;
    }
  }

  if (raw.role) {
    if (!DEEP_LINK_ID_PATTERN.test(raw.role)) {
      warnings.push(ApiStrings.DEEP_LINK.IGNORING_INVALID_ROLE_QUERY_PARAMETER);
    } else if (!isAdmin()) {
      warnings.push(ApiStrings.DEEP_LINK.ROLE_LENS_ROLE_REQUIRES_ADMINISTRATOR);
    } else {
      roleLensId = raw.role;
    }
  }

  return {
    raw,
    warnings,
    viewAsUserId,
    roleLensId,
    permissionFilter: raw.permission,
    endpointParam: raw.endpoint,
  };
}

export function useApiModuleDeepLink(): ApiModuleDeepLinkState {
  const [_tick, setTick] = useState(0);

  useEffect(() => {
    const onHash = () => setTick((t) => t + 1);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);

  return useMemo(() => buildDeepLinkState(window.location.hash || ''), []);
}
