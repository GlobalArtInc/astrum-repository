

import { useMemo } from 'react';
import { ENDPOINTS, restClient } from '../../../../../../interface/api';
import { useForm, type FormContext } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createEmailFormMachine, emailConfigToRest } from './emailFormMachine';
import { EmailConfiguration } from './types';
import EmailStrings from '../../../../constants/pages/settings/system/EmailStrings';

export interface UseEmailFormOptions {
  onCancel?: () => void;
}

/**
 * Custom hook for managing Email settings form state.
 *
 * Uses XState form machine for state management with automatic dirty tracking
 * and unsaved changes warnings. Loads current email configuration on mount,
 * validates on field changes, and saves via REST PUT.
 */
export function useEmailForm(options: UseEmailFormOptions = {}) {
  const toast = useToast();
  const { onCancel } = options;

  const machine = useMemo(() => createEmailFormMachine(), []);

  const form = useForm(machine, {
    actions: {
      onCancel: () => onCancel?.(),
    },
    services: {
      save: async (ctx: FormContext<EmailConfiguration>) => {
        try {
          const payload = emailConfigToRest(ctx.data);
          await restClient.put(ENDPOINTS.EMAIL, payload);
          toast.success(EmailStrings.FORM_HOOK.EMAIL_SERVER_SETTINGS_SAVED);
        } catch (err) {
          toast.error(err instanceof Error ? err.message : EmailStrings.FORM_HOOK.FAILED_SAVE_EMAIL_SETTINGS);
          throw err;
        }
      },
    },
  });

  return form;
}
