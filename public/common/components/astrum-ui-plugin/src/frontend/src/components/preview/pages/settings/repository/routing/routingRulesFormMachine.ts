

import { assign } from 'xstate';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import { restClient } from '../../../../../../interface/api';
import {
  RoutingRule,
  RoutingRuleFormData,
  INITIAL_ROUTING_RULE_FORM,
  NAME_PATTERN,
  NAME_PATTERN_MESSAGE,
} from './types';
import CommonStrings from '../../../../constants/CommonStrings';
import RoutingRulesStrings from '../../../../constants/pages/settings/repository/RoutingRulesStrings';

// API endpoint for routing rules
const ROUTING_RULES_URL = '/service/rest/internal/ui/routing-rules';

/**
 * Extended form context with the loaded routing rule
 */
interface RoutingRulesFormContext extends FormContext<RoutingRuleFormData> {
  routingRule: RoutingRule | null;
}

/**
 * Validate routing rule form data.
 *
 * - name: required, must match NAME_PATTERN
 * - mode: required (ALLOW or BLOCK)
 * - matchers: at least one non-empty matcher, all must be valid regex
 */
function validateRoutingRule(data: RoutingRuleFormData): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.name?.trim()) {
    errors.name = CommonStrings.MESSAGES.NAME_REQUIRED;
  } else if (!NAME_PATTERN.test(data.name)) {
    errors.name = NAME_PATTERN_MESSAGE;
  }

  if (!data.mode) {
    errors.mode = RoutingRulesStrings.FORM_MACHINE.MODE_REQUIRED;
  }

  const validMatchers = data.matchers.filter((m) => m.trim());
  if (validMatchers.length === 0) {
    errors.matchers = RoutingRulesStrings.FORM_MACHINE.LEAST_ONE_MATCHER_REQUIRED;
  } else {
    for (const matcher of validMatchers) {
      try {
        new RegExp(matcher);
      } catch {
        errors.matchers = RoutingRulesStrings.FORM_MACHINE.INVALID_REGEX_PATTERN(matcher);
        break;
      }
    }
  }

  return errors;
}

/**
 * Fetch a routing rule by name from the internal UI API.
 */
async function findRoutingRule(name: string): Promise<RoutingRule | null> {
  try {
    const url = `${ROUTING_RULES_URL}/${encodeURIComponent(name)}`;
    const data = await restClient.get<RoutingRule>(url);
    return data || null;
  } catch (err) {
    console.error('Failed to load routing rule:', err);
    throw err;
  }
}

/**
 * Create a routing rules form machine.
 *
 * This form supports both create and edit modes:
 * - Create mode (no ruleName): starts with INITIAL_ROUTING_RULE_FORM defaults
 * - Edit mode (ruleName provided): loads existing rule from API
 *
 * The form validates:
 * - name: required, alphanumeric + hyphens/underscores, starts with letter
 * - mode: required (ALLOW or BLOCK)
 * - matchers: at least one non-empty pattern, all must be valid regex
 *
 * The mode field (ALLOW/BLOCK) affects help text but not which form fields
 * are visible, so no editingConfig sub-states are used.
 *
 * Delete is supported in edit mode when the rule has no assigned repositories.
 */
export function createRoutingRulesFormMachine(
  ruleName: string | undefined,
  preloadedRule?: RoutingRule
) {
  const hasExistingRule = Boolean(ruleName) || Boolean(preloadedRule);

  return createFormMachine({
    id: `routing-rule-form-${ruleName ?? 'new'}`,
    context: {
      data: { ...INITIAL_ROUTING_RULE_FORM } as RoutingRuleFormData,
      routingRule: preloadedRule ?? (null as RoutingRule | null),
    } as RoutingRulesFormContext,
    actions: {
      validate: assign((ctx: RoutingRulesFormContext) => ({
        validationErrors: validateRoutingRule(ctx.data),
      })),
    },
    services: {
      load: async () => {
        const rule = preloadedRule
          ? preloadedRule
          : ruleName
            ? await findRoutingRule(ruleName).catch((err: unknown) => {
                console.error('Failed to load routing rule:', err);
                throw err;
              })
            : null;

        const initialData: RoutingRuleFormData = rule
          ? {
              name: rule.name,
              description: rule.description || '',
              mode: rule.mode,
              matchers: rule.matchers.length > 0 ? [...rule.matchers] : [''],
            }
          : { ...INITIAL_ROUTING_RULE_FORM };

        return {
          data: initialData,
          routingRule: rule,
        };
      },
      // Delete service: only available in edit mode
      ...(hasExistingRule && {
        delete: async (ctx: RoutingRulesFormContext) => {
          const name = ctx.routingRule?.name || ctx.data.name;
          if (!name) {
            throw new Error(RoutingRulesStrings.FORM_MACHINE.CANNOT_DELETE_ROUTING_RULE_NAME);
          }
          const url = `${ROUTING_RULES_URL}/${encodeURIComponent(name)}`;
          await restClient.delete(url);
        },
      }),
    },
  });
}
