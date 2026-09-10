

import type { Privilege } from '../../../security/privileges/types';
import type { Role } from '../../../security/roles/types';

import type { EndpointAccessDot } from './endpointAccess';
import type { MergedApiEndpoint } from './mergeSwaggerPermissions';
import {
  buildPrivilegeByIdMap,
  buildRoleByIdMap,
  roleSatisfiesRequiredPermissions,
} from './rolePrivilegeCoverage';

function endpointRowKey(row: MergedApiEndpoint): string {
  return `${row.httpMethod}|${row.fullPath}`;
}

export function computeRoleLensAccessById(
  endpoints: MergedApiEndpoint[],
  roleId: string,
  allRoles: Role[],
  privileges: Privilege[]
): Record<string, EndpointAccessDot> {
  const roleById = buildRoleByIdMap(allRoles);
  const privilegeById = buildPrivilegeByIdMap(privileges);
  const out: Record<string, EndpointAccessDot> = {};
  for (const row of endpoints) {
    const id = endpointRowKey(row);
    const reqs = row.permission?.permissions?.map((p) => p.permission).filter(Boolean) ?? [];
    if (reqs.length === 0) {
      out[id] = row.permission?.authenticated === false ? 'granted' : 'unknown';
      continue;
    }
    const mode = row.permission?.permissions?.[0]?.logical === 'OR' ? 'OR' : 'AND';
    const ok = roleSatisfiesRequiredPermissions(roleId, roleById, privilegeById, reqs, mode);
    if (ok) {
      out[id] = 'granted';
    } else {
      const partial =
        mode === 'AND' &&
        reqs.some((req) =>
          roleSatisfiesRequiredPermissions(roleId, roleById, privilegeById, [req], 'AND')
        );
      out[id] = partial ? 'partial' : 'denied';
    }
  }
  return out;
}
