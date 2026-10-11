

import { useMemo } from 'react';
import { useForm, type CheckboxProps, type FieldProps } from '../../../../../../interface/form';
import { useToast } from '../../../../shared';
import { createOAuth2FormMachine } from './oauth2FormMachine';
import { saveOAuth2Config } from './oauth2Api';
import { OAuth2Config } from './types';
import OAuth2Strings from '../../../../constants/pages/settings/security/OAuth2Strings';

/**
 * Presentation-facing API for the OAuth2 settings page.
 *
 * A successful save raises a toast, matching the legacy page (which calls
 * ExtJS.showSuccessMessage) and the sibling Preview settings pages (SAML,
 * Crowd, Realms, User Tokens). Returning to a pristine state alone is too weak
 * a signal: nothing moves on screen except the Save button disabling itself
 * (ASTRUM-54266).
 */
export interface UseOAuth2FormResult {
  data: OAuth2Config;
  isLoading: boolean;
  isSaving: boolean;
  isPristine: boolean;
  hasValidationErrors: boolean;
  loadError: string | null;
  saveError: string | null;
  /** Props for binding a text/password/textarea field to the machine. */
  field: (name: keyof OAuth2Config) => FieldProps;
  /** Props for binding a boolean field (e.g. useTrustStore) to the machine. */
  checkbox: (name: keyof OAuth2Config) => CheckboxProps;
  /** Submit the form (validates, then saves when valid). */
  submit: () => void;
  /** Discard changes, resetting to the last-loaded values. */
  reset: () => void;
}

/**
 * Integration hook wiring the OAuth2 form machine to React. The machine owns
 * load/save/validation; this hook only projects state and exposes commands.
 */
export function useOAuth2Form(): UseOAuth2FormResult {
  const toast = useToast();
  const machine = useMemo(() => createOAuth2FormMachine(), []);

  // Only `save` is overridden; `load` falls through to the machine's own service
  // (useMachine merges the two service maps). The toast has to be raised here
  // rather than in the machine because it needs the provider from React context.
  const form = useForm<OAuth2Config>(machine, {
    services: {
      save: async (ctx) => {
        await saveOAuth2Config(ctx.data);
        toast.success(OAuth2Strings.MESSAGES.OAUTH2_CONFIGURATION_SAVED_SUCCESSFULLY);
      },
    },
  });

  return {
    data: form.data,
    isLoading: form.isLoading,
    isSaving: form.isSaving,
    isPristine: form.isPristine,
    hasValidationErrors: form.hasValidationErrors,
    loadError: form.loadError,
    saveError: form.saveError,
    field: (name) => form.field(name as string),
    checkbox: (name) => form.checkbox(name as string),
    submit: form.submit,
    reset: form.reset,
  };
}

export default useOAuth2Form;
