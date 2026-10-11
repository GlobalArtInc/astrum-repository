

import { assign } from 'xstate';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import { restClient } from '../../../../../../interface/api';
import { AnonymousSettings, RealmType, DEFAULT_ANONYMOUS_SETTINGS } from './types';
import AnonymousStrings from '../../../../constants/pages/settings/security/AnonymousStrings';

const { REST } = APIConstants;

/**
 * Form data shape for anonymous access settings
 */
export type AnonymousFormData = AnonymousSettings;

/**
 * Extended form context with realm types reference data
 */
interface AnonymousFormContext extends FormContext<AnonymousFormData> {
  realmTypes: RealmType[];
}

/**
 * Validate anonymous form data.
 * UserId is always required regardless of enabled state.
 */
function validateAnonymous(data: AnonymousFormData): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.userId?.trim()) {
    errors.userId = AnonymousStrings.VALIDATION.USERNAME_REQUIRED;
  }

  if (!data.realmName?.trim()) {
    errors.realmName = AnonymousStrings.VALIDATION.REALM_REQUIRED;
  }

  return errors;
}

/**
 * Create an anonymous access form machine.
 *
 * This is a simple settings form with no type variants:
 * - Loads current anonymous settings + available realm types on mount
 * - Validates userId is non-empty
 * - Save service is provided via useForm options
 */
export function createAnonymousFormMachine() {
  return createFormMachine({
    id: 'anonymous-form',
    context: {
      data: { ...DEFAULT_ANONYMOUS_SETTINGS } as AnonymousFormData,
      realmTypes: [] as RealmType[],
    } as AnonymousFormContext,
    stayEditableAfterSave: true, // Allow continued editing after save
    actions: {
      validate: assign((ctx: AnonymousFormContext) => ({
        validationErrors: validateAnonymous(ctx.data),
      })),
    },
    services: {
      load: async () => {
        const [settings, realmTypes] = await Promise.all([
          restClient.get<AnonymousSettings>(REST.INTERNAL.ANONYMOUS_SETTINGS).catch((err: unknown) => {
            console.error('Failed to load anonymous settings:', err);
            throw err;
          }),
          restClient.get<RealmType[]>(REST.INTERNAL.REALMS_TYPES).then((data: unknown) => {
            return Array.isArray(data) ? (data as RealmType[]) : [];
          }).catch((err: unknown) => {
            console.warn('Could not load realm types:', err);
            return [] as RealmType[];
          }),
        ]);

        const initialData: AnonymousFormData = {
          enabled: settings.enabled,
          userId: settings.userId,
          realmName: settings.realmName,
        };

        return {
          data: initialData,
          realmTypes,
        };
      },
    },
  });
}
