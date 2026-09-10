

import type { Role } from '../../../security/roles/types';

import { buildRoleByIdMap, collectPrivilegeIdsForRole } from './rolePrivilegeCoverage';

/**
 * Qualifying roles sorted for grant recommendations: fewer effective privileges first (more specific).
 * {@code allRoles} must include nested roles referenced by {@code qualifying} for accurate counts.
 */
export function rankRolesForGrantRecommendations(qualifying: Role[], allRoles: Role[]): Role[] {
  const roleById = buildRoleByIdMap(allRoles);
  return [...qualifying].sort((a, b) => {
    const ca = collectPrivilegeIdsForRole(a.id, roleById).size;
    const cb = collectPrivilegeIdsForRole(b.id, roleById).size;
    if (ca !== cb) {
      return ca - cb;
    }
    return a.name.localeCompare(b.name);
  });
}
