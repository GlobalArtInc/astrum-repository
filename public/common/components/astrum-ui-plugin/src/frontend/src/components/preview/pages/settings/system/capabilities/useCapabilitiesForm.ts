

import { useMemo, useEffect } from 'react';
import { useForm } from '../../../../../../interface/form';
import { useToast, clearDirtyState } from '../../../../shared';
import { createCapabilityFormMachine } from './capabilitiesFormMachine';
import { Capability, CapabilityType, CapabilityFormData } from './types';
import CapabilitiesStrings from '../../../../constants/pages/settings/system/CapabilitiesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface UseCapabilitiesFormOptions {
  capabilityId?: string;
  capability?: Capability;
  initialTypeId?: string;
  onCancel: () => void;
  /** Called after successful save - navigates to list. Defaults to onCancel if not provided. */
  onSaveComplete?: () => void;
  createCapability: (data: CapabilityFormData) => Promise<Capability>;
  updateCapability: (data: CapabilityFormData) => Promise<Capability>;
  deleteCapability?: (id: string) => Promise<void>;
}

export interface UseCapabilitiesFormReturn {
  form: ReturnType<typeof useForm>;
  capability: Capability | null;
  capabilityTypes: CapabilityType[];
  selectedCapabilityType: CapabilityType | null;
  isCreate: boolean;
}

/**
 * Custom hook for managing CapabilityForm state and logic.
 *
 * Uses XState form machine for state management with automatic dirty tracking
 * and unsaved changes warnings. The machine loads both the capability being edited
 * (if capabilityId provided) and reference data (capability types with form fields).
 *
 * Capability type changes trigger CAPABILITY_TYPE_CHANGE events (resets dynamic properties).
 */
export function useCapabilitiesForm({
  capabilityId,
  capability,
  initialTypeId,
  onCancel,
  onSaveComplete,
  createCapability,
  updateCapability,
  deleteCapability,
}: UseCapabilitiesFormOptions): UseCapabilitiesFormReturn {
  const toast = useToast();
  const isCreate = !(capabilityId || capability);
  // Use onSaveComplete for post-save navigation, fallback to onCancel
  const handleSaveComplete = onSaveComplete || onCancel;

  // Create the form machine - memoized based on capabilityId and capability
  const machine = useMemo(
    () => createCapabilityFormMachine(capabilityId, capability, initialTypeId),
    [capabilityId, capability, initialTypeId]
  );

  // Use the form machine with action/service overrides
  const form = useForm(machine, {
    actions: {
      onCancel: onCancel,
    },
    services: {
      save: async (ctx: { data: CapabilityFormData; capability: Capability | null }) => {
        try {
          const capabilityToUpdate = capability || ctx.capability;

          if (isCreate) {
            const created = await createCapability(ctx.data);
            toast.success(CapabilitiesStrings.FORM_HOOK.CAPABILITY_CREATED_SUCCESSFULLY(created.typeName || ctx.data.typeId));
          } else if (capabilityToUpdate) {
            const updateData = {
              ...ctx.data,
              id: capabilityToUpdate.id,
            };
            const updated = await updateCapability(updateData);
            toast.success(CapabilitiesStrings.FORM_HOOK.CAPABILITY_UPDATED_SUCCESSFULLY(updated.typeName || capabilityToUpdate.typeName || ctx.data.typeId));
          }
          // Don't call onCancel() here - let the machine transition to 'saved' state first
          // which will set isPristine=true and clear dirty tracking. Navigation happens
          // after the save completes via the onSaveComplete callback below.
        } catch (err) {
          toast.error(err instanceof Error ? err.message : CommonStrings.MESSAGES.OPERATION_FAILED);
          throw err;
        }
      },
      ...(deleteCapability && {
        delete: async (ctx: { data: CapabilityFormData; capability: Capability | null }) => {
          const capToDelete = capability || ctx.capability;
          if (capToDelete) {
            await deleteCapability(capToDelete.id);
            toast.success(CapabilitiesStrings.FORM_HOOK.CAPABILITY_DELETED_SUCCESSFULLY(capToDelete.typeName || capToDelete.typeId));
            onCancel();
          }
        },
      }),
    },
  });

  // Access the raw state to get the extended context
  const context = (form.state as {
    context: {
      capability: Capability | null;
      capabilityTypes: CapabilityType[];
      selectedCapabilityType: CapabilityType | null;
    };
  }).context;

  // Navigate after save completes - clear dirty state BEFORE navigation
  // to prevent beforeunload warning from firing
  useEffect(() => {
    if (form.isComplete) {
      // Clear dirty state synchronously before navigation
      const formId = `capability-form-${capabilityId ?? 'new'}`;
      clearDirtyState(formId);
      handleSaveComplete();
    }
  }, [form.isComplete, handleSaveComplete, capabilityId]);

  return {
    form,
    capability: context.capability,
    capabilityTypes: context.capabilityTypes,
    selectedCapabilityType: context.selectedCapabilityType,
    isCreate,
  };
}
