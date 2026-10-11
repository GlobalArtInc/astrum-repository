

import { useMemo } from 'react';
import { useForm } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createRolesFormMachine } from './rolesFormMachine';
import { Role, RoleFormData } from './types';
import RolesStrings from '../../../../constants/pages/settings/security/RolesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface UseRolesFormOptions {
  roleId?: string;
  role?: Role;
  onCancel: () => void;
  onComplete?: () => void;
  createRole: (data: RoleFormData) => Promise<void>;
  updateRole: (data: RoleFormData) => Promise<void>;
}

export interface UseRolesFormReturn {
  form: ReturnType<typeof useForm>;
  role: Role | null;
  isCreate: boolean;
}

/**
 * Custom hook for managing RoleForm state and logic.
 *
 * Uses XState form machine for state management with automatic dirty tracking
 * and unsaved changes warnings. The machine loads both the role being edited
 * (if roleId provided) and reference data (privileges, roles).
 *
 * This hook also handles save operations and toast notifications.
 */
export function useRolesForm({
  roleId,
  role,
  onCancel,
  onComplete,
  createRole,
  updateRole,
}: UseRolesFormOptions): UseRolesFormReturn {
  const toast = useToast();
  const isCreate = !(roleId || role);

  const machine = useMemo(
    () => createRolesFormMachine(roleId, role),
    [roleId, role]
  );

  const form = useForm(machine, {
    actions: {
      onCancel: onCancel,
    },
    services: {
      save: async (ctx: { data: RoleFormData; role: Role | null }) => {
        try {
          const roleToUpdate = role || ctx.role;

          const hasPrivilegeOrRole =
            (ctx.data.privileges?.length ?? 0) > 0 || (ctx.data.roles?.length ?? 0) > 0;
          if (!hasPrivilegeOrRole) {
            throw new Error(RolesStrings.MESSAGES.SELECT_LEAST_ONE_PRIVILEGE_CONTAINED);
          }

          if (isCreate) {
            await createRole(ctx.data);
            toast.success(RolesStrings.MESSAGES.ROLE_CREATED_SUCCESSFULLY(ctx.data.name));
          } else if (roleToUpdate) {
            await updateRole(ctx.data);
            toast.success(RolesStrings.MESSAGES.ROLE_UPDATED_SUCCESSFULLY(ctx.data.name));
          }
          if (onComplete) {
            onComplete();
          }
          onCancel();
        } catch (err) {
          toast.error(err instanceof Error ? err.message : CommonStrings.MESSAGES.OPERATION_FAILED);
          throw err;
        }
      },
    },
  });

  // Access the raw state to get the extended context with reference data and role
  const context = (form.state as { context: { role: Role | null } }).context;
  const loadedRole = context.role;

  return {
    form,
    role: loadedRole,
    isCreate,
  };
}
