

import { useCallback, useEffect, useMemo, useState } from 'react';
import { useMachine } from '@xstate/react';

import { usePrivilegesApi } from '../../../security/privileges/usePrivilegesApi';
import type { Role } from '../../../security/roles/types';
import { useRolesApi } from '../../../security/roles/useRolesApi';
import type { User, UserFormData } from '../../../security/users/types';
import { useUsersApi } from '../../../security/users/useUsersApi';

import {
  ensureSecurityDirectoryLoaded,
  getSecurityDirectoryCache,
  invalidateEndpointAccessCache,
  type SecurityDirectorySession,
} from '../hooks/securityDirectoryCache';
import { computeEndpointAccessSummaries } from '../hooks/useEndpointAccess';
import type { MergedApiEndpoint } from '../utils/mergeSwaggerPermissions';
import { pickPrivilegeIdsForRequirements } from '../utils/grantPrivilegePicker';
import { rankRolesForGrantRecommendations } from '../utils/grantRoleRecommendations';

import { createGrantWizardMachine, type GrantWizardContext } from './grantWizardMachine';
import ApiStrings from '../../../../../constants/pages/settings/system/ApiStrings';

export function userDirectoryKey(u: User): string {
  return `${u.userId}\0${u.source}`;
}

export function suggestNewRoleId(row: MergedApiEndpoint): string {
  const tag = (row.tag || 'api')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 40);
  const safeTag = tag || 'api';
  return `api-${safeTag}-${row.httpMethod.toLowerCase()}`;
}

export function suggestNewRoleName(row: MergedApiEndpoint): string {
  return `API — ${row.tag || 'Endpoint'} (${row.httpMethod})`;
}

export function useGrantWizard(row: MergedApiEndpoint | null, active: boolean) {
  const { fetchRoles, createRole } = useRolesApi();
  const { fetchSources, fetchUsers, updateUser } = useUsersApi();
  const { fetchPrivileges } = usePrivilegesApi();

  const [session, setSession] = useState<SecurityDirectorySession | null>(getSecurityDirectoryCache);
  const [dirError, setDirError] = useState<string | null>(null);
  const [loadingDir, setLoadingDir] = useState(false);

  const fetchAllPrivileges = useCallback(async () => {
    const res = await fetchPrivileges(undefined, undefined, undefined, 0, undefined);
    return res.data;
  }, [fetchPrivileges]);

  useEffect(() => {
    if (!(active && row) || session) {
      return undefined;
    }
    const cached = getSecurityDirectoryCache();
    if (cached) {
      setSession(cached);
      return undefined;
    }
    let cancelled = false;
    setLoadingDir(true);
    setDirError(null);
    ensureSecurityDirectoryLoaded({
      fetchRoles,
      fetchAllPrivileges,
      fetchSources,
      fetchUsers,
    })
      .then((loaded) => {
        if (!cancelled) {
          setSession(loaded);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          setDirError(e instanceof Error ? e.message : ApiStrings.GRANT_WIZARD_HOOK.FAILED_LOAD_SECURITY_DATA);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingDir(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [active, row, session, fetchRoles, fetchAllPrivileges, fetchSources, fetchUsers]);

  const { qualifyingRoles, noMappedPermissions } = useMemo(() => {
    if (!(row && session)) {
      return { qualifyingRoles: [] as { role: Role; userCount: number }[], noMappedPermissions: false };
    }
    return computeEndpointAccessSummaries(row, session.roles, session.privileges, session.users);
  }, [row, session]);

  const recommendedRoles = useMemo(() => {
    const roles = qualifyingRoles.map((q) => q.role);
    if (!session || roles.length === 0) {
      return [] as Role[];
    }
    return rankRolesForGrantRecommendations(roles, session.roles);
  }, [qualifyingRoles, session]);

  const userCountByRoleId = useMemo(() => {
    const m = new Map<string, number>();
    for (const { role, userCount } of qualifyingRoles) {
      m.set(role.id, userCount);
    }
    return m;
  }, [qualifyingRoles]);

  const applyGrant = useCallback(
    async (ctx: GrantWizardContext) => {
      if (!row) {
        throw new Error(ApiStrings.GRANT_WIZARD_HOOK.NO_ENDPOINT_SELECTED);
      }
      const snap = getSecurityDirectoryCache();
      if (!snap) {
        throw new Error(ApiStrings.GRANT_WIZARD_HOOK.SECURITY_DATA_NOT_LOADED);
      }
      const reqs = row.permission?.permissions?.map((p) => p.permission).filter(Boolean) ?? [];
      if (reqs.length === 0) {
        throw new Error(ApiStrings.GRANT_WIZARD_HOOK.NO_MAPPED_PERMISSIONS_ENDPOINT);
      }
      const logical = row.permission?.permissions?.[0]?.logical === 'OR' ? 'OR' : 'AND';

      let targetRoleId: string;
      if (ctx.mode === 'create') {
        const { ids, missing } = pickPrivilegeIdsForRequirements(reqs, logical, snap.privileges);
        if (missing.length > 0) {
          throw new Error(
            ApiStrings.GRANT_WIZARD_HOOK.NO_PRIVILEGE_FOUND_ASTRUM_CREATE(missing.join(', '))
          );
        }
        const created = await createRole({
          id: ctx.newRoleId.trim(),
          name: ctx.newRoleName.trim(),
          description: ctx.newRoleDescription.trim(),
          privileges: ids,
          roles: [],
        });
        targetRoleId = created.id;
      } else {
        if (!ctx.existingRoleId) {
          throw new Error(ApiStrings.GRANT_WIZARD_HOOK.NO_ROLE_SELECTED);
        }
        targetRoleId = ctx.existingRoleId;
      }

      const results: Array<{ userKey: string; ok: boolean; message?: string }> = [];
      for (const key of ctx.selectedUserKeys) {
        const [userId, source] = key.split('\0');
        const user = snap.users.find((u) => u.userId === userId && u.source === source);
        if (!user) {
          results.push({ userKey: key, ok: false, message: ApiStrings.GRANT_WIZARD_HOOK.USER_NOT_FOUND_DIRECTORY });
          continue;
        }
        try {
          const roles = [...new Set([...(user.roles || []), targetRoleId])];
          const form: UserFormData = {
            userId: user.userId,
            firstName: user.firstName,
            lastName: user.lastName,
            emailAddress: user.emailAddress,
            status: user.status === 'active',
            roles,
          };
          await updateUser(user.userId, form, user.source);
          results.push({ userKey: key, ok: true });
        } catch (e: unknown) {
          results.push({
            userKey: key,
            ok: false,
            message: e instanceof Error ? e.message : String(e),
          });
        }
      }

      invalidateEndpointAccessCache();
      setSession(null);
      return { results };
    },
    [row, createRole, updateUser]
  );

  const machine = useMemo(() => createGrantWizardMachine(), []);

  const [state, send] = useMachine(machine, {
    services: {
      applyGrant: (ctx) => applyGrant(ctx),
    },
  });

  useEffect(() => {
    if (!state.matches('step2')) {
      return;
    }
    if (state.context.mode !== 'existing') {
      return;
    }
    if (state.context.existingRoleId) {
      return;
    }
    const top = recommendedRoles[0];
    if (!top) {
      return;
    }
    send({ type: 'SELECT_EXISTING', roleId: top.id });
  }, [state, recommendedRoles, send]);

  return {
    state,
    send,
    session,
    loadingDir,
    dirError,
    recommendedRoles,
    qualifyingRoles,
    userCountByRoleId,
    noMappedPermissions,
    suggestedRoleId: row ? suggestNewRoleId(row) : '',
    suggestedRoleName: row ? suggestNewRoleName(row) : '',
  };
}

export type { GrantWizardEvent } from './grantWizardMachine';
