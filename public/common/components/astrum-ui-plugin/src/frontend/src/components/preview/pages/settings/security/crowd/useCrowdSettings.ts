

import { useMemo } from 'react';
import { useMachine } from '@xstate/react';
import { clearDirtyState, useToast } from '../../../../shared';
import { createCrowdSettingsMachine, CrowdMachineContext, CrowdValidationErrors } from './crowdSettingsMachine';
import { CrowdConfig } from './types';
import { fetchCrowdConfig, saveCrowdConfig, verifyCrowdConnection, clearCrowdCache } from './crowdApi';
import CrowdStrings from '../../../../constants/pages/settings/security/CrowdStrings';

export interface UseCrowdSettingsResult {
  config: CrowdConfig;
  validationErrors: CrowdValidationErrors;
  isDirty: boolean;
  isFormValid: boolean;
  /** Initial data load in progress (drives the full-page spinner). */
  isInitialLoading: boolean;
  /** A save/verify/clear-cache operation is in progress (disables actions). */
  isBusy: boolean;
  isSaving: boolean;
  isVerifyingConnection: boolean;
  isClearingCache: boolean;
  error: string | null;
  handleChange: (field: keyof CrowdConfig, value: string | boolean | number | undefined) => void;
  handleSubmit: () => void;
  handleDiscard: () => void;
  handleVerifyConnection: () => void;
  handleClearCache: () => void;
  clearError: () => void;
}

/**
 * Integration hook wiring the Crowd settings machine to React. Injects the API
 * services and success toasts; the machine owns config/validation/busy state.
 *
 * Note: the legacy 'crowd-form' dirty-tracking key is retained (cleared on save)
 * to preserve existing navigation-warning behavior.
 */
export function useCrowdSettings(): UseCrowdSettingsResult {
  const toast = useToast();
  const machine = useMemo(() => createCrowdSettingsMachine(), []);

  const [state, send] = useMachine(machine, {
    services: {
      load: () => fetchCrowdConfig(),
      save: async (ctx: CrowdMachineContext) => {
        await saveCrowdConfig(ctx.data);
        clearDirtyState('crowd-form');
        toast.success(CrowdStrings.MESSAGES.ATLASSIAN_CROWD_SETTINGS_UPDATED);
      },
      verifyConnection: async (ctx: CrowdMachineContext) => {
        await verifyCrowdConnection(ctx.data);
        toast.success(CrowdStrings.MESSAGES.CONNECTION_CROWD_SERVER_VERIFIED);
      },
      clearCache: async () => {
        await clearCrowdCache();
        toast.success(CrowdStrings.MESSAGES.CROWD_CACHE_HAS_BEEN_CLEARED);
      },
    },
  });

  const c = state.context;
  const isSaving = state.matches('saving');
  const isVerifyingConnection = state.matches('verifyingConnection');
  const isClearingCache = state.matches('clearingCache');
  const isBusy = isSaving || isVerifyingConnection || isClearingCache;

  return {
    config: c.data,
    validationErrors: c.validationErrors,
    isDirty: !c.isPristine,
    isFormValid: Object.keys(c.validationErrors).length === 0,
    isInitialLoading: state.matches('loading'),
    isBusy,
    isSaving,
    isVerifyingConnection,
    isClearingCache,
    error: c.error,
    handleChange: (field, value) => send({ type: 'UPDATE', field, value }),
    handleSubmit: () => send({ type: 'SUBMIT' }),
    handleDiscard: () => send({ type: 'DISCARD' }),
    handleVerifyConnection: () => send({ type: 'VERIFY_CONNECTION' }),
    handleClearCache: () => send({ type: 'CLEAR_CACHE' }),
    clearError: () => send({ type: 'CLEAR_ERROR' }),
  };
}

export default useCrowdSettings;
