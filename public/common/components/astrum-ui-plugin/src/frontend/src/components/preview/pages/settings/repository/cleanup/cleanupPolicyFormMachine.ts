

import { assign } from 'xstate';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';

import {
  CleanupPolicy,
  CleanupPolicyFormData,
  FormatCriteria,
  EMPTY_CLEANUP_POLICY,
  NOTES_MAX_LENGTH,
  isValidCriteriaNumber,
  isReleaseType,
  getDefaultSortBy,
} from './types';
import CommonStrings from '../../../../constants/CommonStrings';
import CleanupPoliciesStrings from '../../../../constants/pages/settings/repository/CleanupPoliciesStrings';

/**
 * Extended context for the cleanup policy form machine.
 * Tracks which criteria checkboxes are enabled and
 * which format criteria are available.
 */
export interface CleanupPolicyMachineContext {
  data: CleanupPolicyFormData;
  /** Reference data: available format criteria from the server */
  formatCriteria: FormatCriteria[];
  /** The original policy being edited (null for create) */
  policy: CleanupPolicy | null;
  /** Criteria enable/disable flags tracked in machine context */
  criteriaEnabled: {
    lastBlobUpdated: boolean;
    lastDownloaded: boolean;
    assetRegex: boolean;
    retain: boolean;
  };
}

/**
 * Get available criteria for the currently selected format.
 */
function _getAvailableCriteria(
  format: string,
  formatCriteria: FormatCriteria[]
): string[] {
  const fc = formatCriteria.find((f) => f.id === format);
  return fc?.availableCriteria ?? [];
}

/**
 * Validate cleanup policy form data.
 *
 * Criteria validation depends on which checkboxes are enabled,
 * which is tracked via the criteriaEnabled flags in context.
 */
export function validateCleanupPolicy(
  data: CleanupPolicyFormData,
  criteriaEnabled: CleanupPolicyMachineContext['criteriaEnabled'],
  _formatCriteria: FormatCriteria[]
): ValidationErrors {
  const errors: ValidationErrors = {};

  // Name validation
  if (!data.name?.trim()) {
    errors.name = CommonStrings.MESSAGES.NAME_REQUIRED;
  } else if (!/^[a-zA-Z0-9\-_.]+$/.test(data.name)) {
    errors.name = CleanupPoliciesStrings.FORM_MACHINE.NAME_MUST_CONTAIN_ONLY_LETTERS;
  }

  // Format validation
  if (!data.format) {
    errors.format = CleanupPoliciesStrings.FORM_MACHINE.FORMAT_REQUIRED;
  }

  // Notes length
  if (data.notes && data.notes.length > NOTES_MAX_LENGTH) {
    errors.notes = CleanupPoliciesStrings.FORM_MACHINE.DESCRIPTION_MUST_CHARACTERS_LESS(NOTES_MAX_LENGTH);
  }

  // Criteria number fields - only validate when enabled
  if (criteriaEnabled.lastBlobUpdated) {
    if (!data.criteriaLastBlobUpdated) {
      errors.criteriaLastBlobUpdated = CleanupPoliciesStrings.FORM_MACHINE.COMPONENT_AGE_REQUIRED;
    } else if (!isValidCriteriaNumber(data.criteriaLastBlobUpdated)) {
      errors.criteriaLastBlobUpdated = CleanupPoliciesStrings.FORM_MACHINE.MUST_WHOLE_NUMBER_BETWEEN_1;
    }
  }

  if (criteriaEnabled.lastDownloaded) {
    if (!data.criteriaLastDownloaded) {
      errors.criteriaLastDownloaded = CleanupPoliciesStrings.FORM_MACHINE.COMPONENT_USAGE_REQUIRED;
    } else if (!isValidCriteriaNumber(data.criteriaLastDownloaded)) {
      errors.criteriaLastDownloaded = CleanupPoliciesStrings.FORM_MACHINE.MUST_WHOLE_NUMBER_BETWEEN_1;
    }
  }

  if (criteriaEnabled.assetRegex) {
    if (!data.criteriaAssetRegex?.trim()) {
      errors.criteriaAssetRegex = CleanupPoliciesStrings.FORM_MACHINE.ASSET_NAME_PATTERN_REQUIRED;
    } else {
      try {
        new RegExp(data.criteriaAssetRegex);
      } catch {
        errors.criteriaAssetRegex = CleanupPoliciesStrings.FORM_MACHINE.INVALID_REGULAR_EXPRESSION_PATTERN;
      }
    }
  }

  if (criteriaEnabled.retain) {
    if (!data.retain) {
      errors.retain = CleanupPoliciesStrings.FORM_MACHINE.NUMBER_VERSIONS_REQUIRED;
    } else if (!isValidCriteriaNumber(data.retain)) {
      errors.retain = CleanupPoliciesStrings.FORM_MACHINE.MUST_WHOLE_NUMBER_BETWEEN_1;
    }
  }

  // At least one criteria must be enabled when a format is selected
  if (
    data.format &&
    !criteriaEnabled.lastBlobUpdated &&
    !criteriaEnabled.lastDownloaded &&
    !criteriaEnabled.assetRegex &&
    !criteriaEnabled.retain
  ) {
    errors.criteriaSelected = CleanupPoliciesStrings.FORM_MACHINE.LEAST_ONE_CRITERION_MUST_SELECTED;
  }

  return errors;
}

/**
 * Create a cleanup policy form machine with XState.
 *
 * The cleanup policy form models format-aware criteria:
 * - Format selection determines which criteria checkboxes are available
 * - Each criterion has an enable/disable toggle tracked in context
 * - Changing format resets all criteria values and flags
 * - Retain/exclusion criteria is only available for specific formats + release type
 *
 * Custom events:
 * - FORMAT_CHANGE: changes format and resets criteria
 * - TOGGLE_CRITERIA: enables/disables a specific criterion checkbox
 * - RELEASE_TYPE_CHANGE: changes release type, may disable retain
 */
export function createCleanupPolicyFormMachine(
  policyName: string | undefined,
  preloadedPolicy?: CleanupPolicy | null,
  formatCriteria: FormatCriteria[] = []
) {
  const initialCriteriaEnabled = preloadedPolicy
    ? {
        lastBlobUpdated: Boolean(preloadedPolicy.criteriaLastBlobUpdated),
        lastDownloaded: Boolean(preloadedPolicy.criteriaLastDownloaded),
        assetRegex: Boolean(preloadedPolicy.criteriaAssetRegex),
        retain: Boolean(preloadedPolicy.retain) || Boolean(preloadedPolicy.sortBy),
      }
    : {
        lastBlobUpdated: false,
        lastDownloaded: false,
        assetRegex: false,
        retain: false,
      };

  return createFormMachine({
    id: `cleanup-policy-form-${policyName ?? 'new'}`,
    context: {
      data: { ...EMPTY_CLEANUP_POLICY } as CleanupPolicyFormData,
      formatCriteria,
      policy: preloadedPolicy ?? (null as CleanupPolicy | null),
      criteriaEnabled: initialCriteriaEnabled,
    },
    actions: {
      validate: assign((ctx: FormContext<CleanupPolicyFormData>) => {
        const extCtx = ctx as unknown as FormContext<CleanupPolicyFormData> & CleanupPolicyMachineContext;
        return {
          validationErrors: validateCleanupPolicy(
            ctx.data,
            extCtx.criteriaEnabled,
            extCtx.formatCriteria
          ),
        };
      }),
      // Change format and reset all criteria
      changeFormat: assign((context: any, event: any) => {
        const format = event.value as string;
        return {
          data: {
            ...context.data,
            format,
            criteriaLastBlobUpdated: null,
            criteriaLastDownloaded: null,
            criteriaReleaseType: null,
            criteriaAssetRegex: null,
            retain: null,
            sortBy: null,
          },
          criteriaEnabled: {
            lastBlobUpdated: false,
            lastDownloaded: false,
            assetRegex: false,
            retain: false,
          },
          touched: { ...context.touched, format: true },
        };
      }),
      // Toggle a criteria checkbox on/off
      toggleCriteria: assign((context: any, event: any) => {
        const { criteria, enabled } = event as { type: string; criteria: string; enabled: boolean };
        const newCriteriaEnabled = {
          ...context.criteriaEnabled,
          [criteria]: enabled,
        };

        // When disabling a criteria, clear its data value
        const newData = { ...context.data };
        if (!enabled) {
          switch (criteria) {
            case 'lastBlobUpdated':
              newData.criteriaLastBlobUpdated = null;
              break;
            case 'lastDownloaded':
              newData.criteriaLastDownloaded = null;
              break;
            case 'assetRegex':
              newData.criteriaAssetRegex = null;
              break;
            case 'retain':
              newData.retain = null;
              newData.sortBy = null;
              break;
          }
        }

        // When enabling retain, set default sortBy for the format
        if (enabled && criteria === 'retain') {
          newData.sortBy = getDefaultSortBy(context.data.format);
        }

        // For docker: disable retain when no other criteria remain selected
        if (!enabled && criteria !== 'retain' && newData.format === 'docker') {
          const hasOtherCriteria = Boolean((
            newData.criteriaLastBlobUpdated ||
            newData.criteriaLastDownloaded ||
            newData.criteriaAssetRegex
          ));
          if (!hasOtherCriteria && newCriteriaEnabled.retain) {
            newCriteriaEnabled.retain = false;
            newData.retain = null;
            newData.sortBy = null;
          }
        }

        return {
          data: newData,
          criteriaEnabled: newCriteriaEnabled,
        };
      }),
      // Change release type, may disable retain criteria
      changeReleaseType: assign((context: any, event: any) => {
        const releaseType = event.value as string;
        const shouldDisableRetain = !isReleaseType(releaseType);
        const newCriteriaEnabled = shouldDisableRetain
          ? { ...context.criteriaEnabled, retain: false }
          : context.criteriaEnabled;
        const newData = {
          ...context.data,
          criteriaReleaseType: releaseType || null,
        };
        if (shouldDisableRetain) {
          newData.retain = null;
          newData.sortBy = null;
        }
        return {
          data: newData,
          criteriaEnabled: newCriteriaEnabled,
          touched: { ...context.touched, criteriaReleaseType: true },
        };
      }),
    },
    services: {
      load: async () => {
        const initialData: CleanupPolicyFormData = preloadedPolicy
          ? {
              name: preloadedPolicy.name,
              format: preloadedPolicy.format,
              notes: preloadedPolicy.notes || '',
              criteriaLastBlobUpdated: preloadedPolicy.criteriaLastBlobUpdated,
              criteriaLastDownloaded: preloadedPolicy.criteriaLastDownloaded,
              criteriaReleaseType: preloadedPolicy.criteriaReleaseType,
              criteriaAssetRegex: preloadedPolicy.criteriaAssetRegex,
              retain: preloadedPolicy.retain,
              sortBy: preloadedPolicy.sortBy,
            }
          : { ...EMPTY_CLEANUP_POLICY };

        return {
          data: initialData,
          policy: preloadedPolicy ?? null,
          formatCriteria,
          criteriaEnabled: initialCriteriaEnabled,
        };
      },
      // save service is provided via useForm options
    },
    on: {
      FORMAT_CHANGE: {
        actions: ['changeFormat', 'validate', 'computePristine'],
      },
      TOGGLE_CRITERIA: {
        actions: ['toggleCriteria', 'validate', 'computePristine'],
      },
      RELEASE_TYPE_CHANGE: {
        actions: ['changeReleaseType', 'validate', 'computePristine'],
      },
    },
  });
}
