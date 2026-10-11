

import { useMemo } from 'react';
import { ENDPOINTS, restClient } from '../../../../../../interface/api';
import { useForm, type FormContext } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createHttpFormMachine, httpConfigToRest } from './httpFormMachine';
import { HttpConfiguration } from './types';
import HttpStrings from '../../../../constants/pages/settings/system/HttpStrings';

export interface UseHttpFormOptions {
  onCancel?: () => void;
}

const HTTP_SAVE_ERROR_MESSAGES: Record<number, string> = {
  400: HttpStrings.FORM_HOOK.INVALID_HTTP_SETTINGS_PLEASE_CHECK,
  403: HttpStrings.FORM_HOOK.YOU_DO_NOT_HAVE_PERMISSION,
  405: HttpStrings.FORM_HOOK.UNABLE_SAVE_HTTP_SETTINGS_SERVER,
  500: HttpStrings.FORM_HOOK.SERVER_ERROR_WHILE_SAVING_HTTP,
};

export function humanizeHttpError(err: unknown): string {
  if (err && typeof err === 'object' && 'response' in err) {
    const status = (err as { response?: { status?: number } }).response?.status;
    if (status && HTTP_SAVE_ERROR_MESSAGES[status]) {
      return HTTP_SAVE_ERROR_MESSAGES[status];
    }
  }
  const message = err instanceof Error ? err.message : String(err);
  if (message.includes('status code')) {
    return HttpStrings.FORM_HOOK.UNABLE_SAVE_HTTP_SETTINGS_PLEASE;
  }
  return message;
}

/**
 * Custom hook for managing HTTP settings form state.
 *
 * Uses XState form machine for state management with automatic dirty tracking
 * and unsaved changes warnings. Loads current HTTP configuration on mount,
 * validates on field changes, and saves via REST PUT.
 */
export function useHttpForm(options: UseHttpFormOptions = {}) {
  const toast = useToast();
  const { onCancel } = options;

  const machine = useMemo(() => createHttpFormMachine(), []);

  const form = useForm(machine, {
    actions: {
      onCancel: () => onCancel?.(),
    },
    services: {
      save: async (ctx: FormContext<HttpConfiguration>) => {
        try {
          const payload = httpConfigToRest(ctx.data);
          await restClient.put(ENDPOINTS.HTTP, payload);
          toast.success(HttpStrings.FORM_HOOK.HTTP_SETTINGS_SAVED);
        } catch (err) {
          const message = humanizeHttpError(err);
          toast.error(message);
          throw new Error(message);
        }
      },
    },
  });

  return form;
}
