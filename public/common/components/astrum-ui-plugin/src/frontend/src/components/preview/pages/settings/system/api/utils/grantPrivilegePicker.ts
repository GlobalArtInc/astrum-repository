

import type { Privilege } from '../../../security/privileges/types';

import { wildcardImplies } from './shiroWildcardImplies';

/** Higher score = more specific (prefer for new role attachment). */
function specificityScore(permission: string): number {
  const stars = (permission.match(/\*/g) || []).length;
  return permission.length - stars * 50;
}

function pickForRequirement(req: string, privileges: Privilege[]): Privilege | null {
  const r = req.toLowerCase();
  const candidates = privileges.filter(
    (p) => p.permission && wildcardImplies(p.permission.toLowerCase(), r)
  );
  if (candidates.length === 0) {
    return null;
  }
  const exact = candidates.find((p) => p.permission.toLowerCase() === r);
  if (exact) {
    return exact;
  }
  return candidates.reduce((best, cur) =>
    specificityScore(cur.permission) > specificityScore(best.permission) ? cur : best
  );
}

/**
 * Picks privilege IDs to attach to a new role so it satisfies the endpoint requirements.
 */
export function pickPrivilegeIdsForRequirements(
  requirements: string[],
  logical: 'AND' | 'OR',
  privileges: Privilege[]
): { ids: string[]; missing: string[] } {
  if (requirements.length === 0) {
    return { ids: [], missing: [] };
  }
  if (logical === 'OR') {
    const reqs = requirements.map((x) => x.toLowerCase());
    let best: Privilege | null = null;
    let bestCoverage = 0;
    let bestScore = -Infinity;
    for (const p of privileges) {
      if (!p.permission) {
        continue;
      }
      const perm = p.permission.toLowerCase();
      const coverage = reqs.filter((req) => wildcardImplies(perm, req)).length;
      if (coverage === 0) {
        continue;
      }
      const sc = specificityScore(p.permission);
      if (coverage > bestCoverage || (coverage === bestCoverage && sc > bestScore)) {
        best = p;
        bestCoverage = coverage;
        bestScore = sc;
      }
    }
    if (!best) {
      return { ids: [], missing: [...requirements] };
    }
    return { ids: [best.id], missing: [] };
  }

  const ids: string[] = [];
  const missing: string[] = [];
  const seen = new Set<string>();
  for (const req of requirements) {
    const p = pickForRequirement(req, privileges);
    if (!p) {
      missing.push(req);
    } else if (!seen.has(p.id)) {
      seen.add(p.id);
      ids.push(p.id);
    }
  }
  return { ids, missing };
}
