

import { assign } from 'xstate';
import { ENDPOINTS, restClient } from '../../../../../../interface/api';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';

import {
  Role,
  RoleFormData,
  RoleReference,
  PrivilegeReference,
  RoleSource,
  DEFAULT_SOURCE,
  isExternalRole,
} from './types';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';

/**
 * REST API role shape (from REST API response)
 */
interface RestRole {
  id: string;
  source: string;
  name: string;
  description: string;
  readOnly: boolean;
  privileges: string[];
  roles: string[];
}

/**
 * Convert REST role to UI Role model
 */
function restToRole(rest: RestRole): Role {
  return {
    id: rest.id,
    version: '1',
    source: rest.source === DEFAULT_SOURCE ? 'Default' : rest.source,
    name: rest.name,
    description: rest.description || '',
    readOnly: rest.readOnly ?? false,
    privileges: rest.privileges || [],
    roles: rest.roles || [],
  };
}

/**
 * Find a role by ID and transform to UI model
 */
async function findRole(id: string): Promise<Role | null> {
  try {
    const data = await restClient.get<RestRole>(`${ENDPOINTS.ROLES}/${encodeURIComponent(id)}`);
    return restToRole(data);
  } catch (err) {
    console.error('Failed to load role:', err);
    return null;
  }
}

/**
 * Validate role form data.
 * Returns an object with field names as keys and error messages as values.
 */
function validateRole(
  data: RoleFormData,
  existingRoles: RoleReference[] = [],
  currentRoleId?: string
): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.id?.trim()) {
    errors.id = RolesStrings.VALIDATION.ROLE_ID_REQUIRED;
  } else if (!/^[a-zA-Z0-9_-]+$/.test(data.id)) {
    errors.id = RolesStrings.VALIDATION.ROLE_ID_CAN_ONLY_CONTAIN;
  }

  if (!data.name?.trim()) {
    errors.name = RolesStrings.VALIDATION.ROLE_NAME_REQUIRED;
  } else {
    const duplicate = existingRoles.find(
      (r) => r.name.toLowerCase() === data.name.trim().toLowerCase() && r.id !== currentRoleId
    );
    if (duplicate) {
      errors.name = RolesStrings.VALIDATION.ROLE_NAMED_ALREADY_EXISTS_ID(duplicate.name, duplicate.id);
    }
  }

  if (
    (!data.privileges || data.privileges.length === 0) &&
    (!data.roles || data.roles.length === 0)
  ) {
    errors.privileges = RolesStrings.VALIDATION.SELECT_LEAST_ONE_PRIVILEGE_CONTAINED;
  }

  return errors;
}

/**
 * Create a roles form machine with XState.
 * Uses createFormMachine with no editingConfig since roles don't have type variants.
 *
 * This factory creates a machine configured for create or edit mode.
 * The machine loads both the role being edited (if roleId provided) and
 * reference data (available privileges and roles).
 */
export function createRolesFormMachine(
  roleId: string | undefined,
  preloadedRole?: Role
) {
  return createFormMachine({
    id: `roles-form-${roleId ?? 'new'}`,
    context: {
      data: {
        id: '',
        name: '',
        description: '',
        privileges: [] as string[],
        roles: [] as string[],
      } as RoleFormData,
      // Initialize reference data as empty - populated by load service
      role: preloadedRole ?? (null as Role | null),
      allPrivileges: [] as PrivilegeReference[],
      allRoles: [] as RoleReference[],
      allSources: [] as RoleSource[],
    },
    actions: {
      validate: assign((ctx: FormContext<RoleFormData>) => {
        const { allRoles, role } = ctx as any;
        return {
          validationErrors: validateRole(ctx.data, allRoles || [], role?.id ?? roleId),
        };
      }),
    },
    services: {
      load: async () => {
        // Load role and reference data in parallel
        const [role, privileges, roles, sources] = await Promise.all([
          // If role is preloaded, use it; otherwise fetch if roleId is provided
          preloadedRole
            ? Promise.resolve(preloadedRole)
            : roleId
            ? findRole(roleId)
            : Promise.resolve(null),
          restClient
            .get(ENDPOINTS.PRIVILEGES)
            .then((data: unknown) => {
              const arr = data as Array<{ name: string; description?: string }>;
              return arr.map((p) => ({ id: p.name, name: p.name, description: p.description }));
            })
            .catch((err: unknown) => {
              console.warn('Could not load privileges for role form:', err);
              return [] as PrivilegeReference[];
            }),
          restClient
            .get(`${ENDPOINTS.ROLES}?source=${DEFAULT_SOURCE}`)
            .then((data: unknown) => {
              const arr = data as RestRole[];
              return arr.map((r) => ({ id: r.id, name: r.name }));
            })
            .catch((err: unknown) => {
              console.warn('Could not load roles for role form:', err);
              return [] as RoleReference[];
            }),
          restClient
            .get(ENDPOINTS.ROLE_SOURCES)
            .then((data: unknown) => {
              const arr = data as Array<{ id: string; name: string }>;
              return arr;
            })
            .catch((err: unknown) => {
              console.warn('Could not load role sources:', err);
              return [] as RoleSource[];
            }),
        ]);

        const allPrivileges = Array.isArray(privileges) ? privileges : [];
        // Filter out current role from available roles to prevent circular reference
        // and deduplicate by ID (same role ID can appear from multiple sources)
        const rolesById = new Map<string, RoleReference>();
        if (Array.isArray(roles)) {
          for (const r of roles) {
            if (r.id !== (role?.id ?? roleId) && !rolesById.has(r.id)) {
              rolesById.set(r.id, { id: r.id, name: r.name });
            }
          }
        }
        const allRoles = Array.from(rolesById.values());
        const allSources = Array.isArray(sources) ? sources : [];

        // Build initial form data from loaded role or use defaults.
        // In edit mode, derive roleType/externalSource from role.source so the
        // Type step (step 0) can render a read-only view of the persisted type.
        // Conditional-spread externalSource so the key is simply absent for
        // Astrum roles — avoids `undefined`-vs-absent ambiguity if the context
        // merge strategy ever changes.
        const external = role ? isExternalRole(role.source) : false;
        const initialData: RoleFormData = role
          ? {
              id: role.id,
              name: role.name,
              description: role.description,
              privileges: role.privileges,
              roles: role.roles,
              source: role.source,
              version: role.version,
              roleType: external ? 'external' : 'astrum',
              ...(external ? { externalSource: role.source } : {}),
            }
          : {
              id: '',
              name: '',
              description: '',
              privileges: [],
              roles: [],
            };

        return {
          data: initialData,
          role,
          allPrivileges,
          allRoles,
          allSources,
        };
      },
      // save service is provided via useForm options
    },
  });
}
