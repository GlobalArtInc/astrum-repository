

import { useMemo } from 'react';
import { ENDPOINTS, restClient } from '../../../../../../interface/api';
import { useForm, type FormContext } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createEmailFormMachine, emailConfigToRest } from './emailFormMachine';
import { EmailConfiguration } from './types';

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
          toast.success('Email server settings saved');
        } catch (err) {
          toast.error(err instanceof Error ? err.message : 'Failed to save email settings');
          throw err;
        }
      },
    },
  });

  return form;
}
