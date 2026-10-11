

import { assign } from 'xstate';
import { API_INTERNAL_UI, ENDPOINTS, restClient } from '../../../../../../interface/api';
import { createFormMachine, type FormContext, type ValidationErrors } from '../../../../../../interface/form';
import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';
import CommonStrings from '../../../../constants/CommonStrings';

/**
 * Guard factory: creates a guard that checks if a TYPE_CHANGE event targets a specific type,
 * or if the current data type matches (for initial load).
 */
const isTypeGuard = (targetType: string) =>
  (context: any, event: any) => {
    if (event.type === 'TYPE_CHANGE') {
      return event.value === targetType;
    }
    return context.data?.type === targetType;
  };
import {
  Privilege,
  PrivilegeType,
  PrivilegeFormData,
  PRIVILEGE_TYPES,
  ContentSelector,
} from './types';
import { CONTENT_SELECTOR_API } from '../../repository/selectors/types';

// Internal UI endpoints for privilege types
const PRIVILEGE_TYPES_URL = `${API_INTERNAL_UI}/privileges/types`;

/**
 * Validate privilege form data
 * Returns an object with field names as keys and error messages as values
 */
function validatePrivilege(data: PrivilegeFormData): ValidationErrors {
  const errors: ValidationErrors = {};

  if (!data.name?.trim()) {
    errors.name = PrivilegesStrings.FORM_MACHINE.PRIVILEGE_NAME_REQUIRED;
  } else if (!/^[a-zA-Z0-9_-]+$/.test(data.name)) {
    errors.name = PrivilegesStrings.FORM_MACHINE.NAME_CAN_ONLY_CONTAIN_LETTERS;
  }

  if (!data.type) {
    errors.type = PrivilegesStrings.FORM_MACHINE.PRIVILEGE_TYPE_REQUIRED;
  }

  // Type-specific validation using nested property paths
  if (data.type === PRIVILEGE_TYPES.WILDCARD) {
    if (!data.properties?.pattern?.trim()) {
      errors['properties.pattern'] = PrivilegesStrings.FORM_MACHINE.PATTERN_REQUIRED_WILDCARD_PRIVILEGES;
    }
  }

  if (data.type === PRIVILEGE_TYPES.APPLICATION) {
    if (!data.properties?.domain?.trim()) {
      errors['properties.domain'] = PrivilegesStrings.FORM_MACHINE.DOMAIN_REQUIRED;
    }
    if (!data.properties?.actions?.trim()) {
      errors['properties.actions'] = PrivilegesStrings.FORM_MACHINE.ACTIONS_REQUIRED;
    }
  }

  if (
    data.type === PRIVILEGE_TYPES.REPOSITORY_VIEW ||
    data.type === PRIVILEGE_TYPES.REPOSITORY_ADMIN
  ) {
    if (!data.properties?.format?.trim()) {
      errors['properties.format'] = PrivilegesStrings.FORM_MACHINE.REPOSITORY_FORMAT_REQUIRED;
    }
    if (!data.properties?.repository?.trim()) {
      errors['properties.repository'] = PrivilegesStrings.FORM_MACHINE.REPOSITORY_REQUIRED;
    }
    if (!data.properties?.actions?.trim()) {
      errors['properties.actions'] = PrivilegesStrings.FORM_MACHINE.ACTIONS_REQUIRED;
    }
  }

  if (data.type === PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR) {
    if (!data.properties?.repository?.trim()) {
      errors['properties.repository'] = PrivilegesStrings.FORM_MACHINE.REPOSITORY_REQUIRED;
    }
    if (!data.properties?.contentSelector?.trim()) {
      errors['properties.contentSelector'] = PrivilegesStrings.FORM_MACHINE.CONTENT_SELECTOR_REQUIRED;
    }
    if (!data.properties?.actions?.trim()) {
      errors['properties.actions'] = PrivilegesStrings.FORM_MACHINE.ACTIONS_REQUIRED;
    }
  }

  if (data.type === PRIVILEGE_TYPES.SCRIPT) {
    if (!data.properties?.name?.trim()) {
      errors['properties.name'] = PrivilegesStrings.FORM_MACHINE.SCRIPT_NAME_REQUIRED;
    }
    if (!data.properties?.actions?.trim()) {
      errors['properties.actions'] = PrivilegesStrings.FORM_MACHINE.ACTIONS_REQUIRED;
    }
  }

  return errors;
}

/**
 * REST API privilege shape
 */
interface RestPrivilege {
  type: string;
  name: string;
  description?: string;
  readOnly: boolean;
  domain?: string;
  actions?: string[];
  pattern?: string;
  format?: string;
  repository?: string;
  contentSelector?: string;
  scriptName?: string;
}

/**
 * Compute permission string from REST privilege
 */
function computePermission(rest: RestPrivilege): string {
  switch (rest.type) {
    case 'application':
      return `astrum:${rest.domain || '*'}:${(rest.actions || []).join(',').toLowerCase() || '*'}`;
    case 'wildcard':
      return rest.pattern || '*';
    case 'repository-view':
      return `astrum:repository-view:${rest.format || '*'}:${rest.repository || '*'}:${(rest.actions || []).join(',').toLowerCase() || '*'}`;
    case 'repository-admin':
      return `astrum:repository-admin:${rest.format || '*'}:${rest.repository || '*'}:${(rest.actions || []).join(',').toLowerCase() || '*'}`;
    case 'repository-content-selector':
      return `astrum:repository-content-selector:${rest.format || '*'}:${rest.repository || '*'}:${rest.contentSelector || '*'}:${(rest.actions || []).join(',').toLowerCase() || '*'}`;
    case 'script':
      return `astrum:script:${rest.scriptName || '*'}:${(rest.actions || []).join(',').toLowerCase() || '*'}`;
    default:
      return rest.type;
  }
}

/**
 * Transform REST privilege to UI model with computed permission field
 */
function transformPrivilege(rest: RestPrivilege): Privilege {
  const properties: Record<string, string> = {};

  switch (rest.type) {
    case 'application':
      if (rest.domain) properties.domain = rest.domain;
      if (rest.actions) properties.actions = rest.actions.join(',');
      break;
    case 'wildcard':
      if (rest.pattern) properties.pattern = rest.pattern;
      break;
    case 'repository-view':
    case 'repository-admin':
      if (rest.format) properties.format = rest.format;
      if (rest.repository) properties.repository = rest.repository;
      if (rest.actions) properties.actions = rest.actions.join(',');
      break;
    case 'repository-content-selector':
      if (rest.format) properties.format = rest.format;
      if (rest.repository) properties.repository = rest.repository;
      if (rest.contentSelector) properties.contentSelector = rest.contentSelector;
      if (rest.actions) properties.actions = rest.actions.join(',');
      break;
    case 'script':
      if (rest.scriptName) properties.name = rest.scriptName;
      if (rest.actions) properties.actions = rest.actions.join(',');
      break;
  }

  return {
    id: rest.name,
    version: '1',
    name: rest.name,
    description: rest.description || '',
    type: rest.type,
    readOnly: rest.readOnly,
    properties,
    permission: computePermission(rest),
  };
}

/**
 * Find privilege by ID and transform to UI model
 */
async function findPrivilege(id: string): Promise<Privilege | null> {
  try {
    const data = await restClient.get<RestPrivilege>(`${ENDPOINTS.PRIVILEGES}/${id}`);
    return transformPrivilege(data);
  } catch (err) {
    console.error('Failed to load privilege:', err);
    throw err;
  }
}

/**
 * Fetch privilege types from internal UI endpoint
 */
async function fetchPrivilegeTypes(): Promise<PrivilegeType[]> {
  try {
    const data = await restClient.get(PRIVILEGE_TYPES_URL);
    return data as PrivilegeType[];
  } catch (err) {
    console.error('Failed to load privilege types:', err);
    return [];
  }
}

/**
 * Create a privilege form machine with XState
 * This factory allows creating a machine configured for create or edit mode
 */
export function createPrivilegeFormMachine(
  privilegeId: string | undefined,
  preloadedPrivilege?: Privilege,
  initialTypeId?: string
) {
  return createFormMachine({
    id: `privilege-form-${privilegeId ?? 'new'}`,
    context: {
      data: {
        name: '',
        description: '',
        type: initialTypeId || preloadedPrivilege?.type || PRIVILEGE_TYPES.APPLICATION,
        properties: {},
      } as PrivilegeFormData,
      // Initialize reference data as empty - will be populated by load service
      privilege: preloadedPrivilege ?? (null as Privilege | null),
      privilegeTypes: [] as PrivilegeType[],
      repositories: [] as { name: string; format: string; type: string; status: { online: boolean } }[],
      formats: [] as string[],
      contentSelectors: [] as ContentSelector[],
      scripts: [] as { name: string }[],
    },
    actions: {
      validate: assign((ctx: FormContext<PrivilegeFormData>) => ({
        validationErrors: validatePrivilege(ctx.data),
      })),
      changeType: assign((context: any, event: any) => ({
        data: { ...context.data, type: event.value, properties: {} },
        touched: { ...context.touched, type: true },
      })),
    },
    // Guards for TYPE_CHANGE transitions between sub-states
    guards: {
      isTypeApplication: isTypeGuard(PRIVILEGE_TYPES.APPLICATION) as any,
      isTypeWildcard: isTypeGuard(PRIVILEGE_TYPES.WILDCARD) as any,
      isTypeRepositoryView: isTypeGuard(PRIVILEGE_TYPES.REPOSITORY_VIEW) as any,
      isTypeRepositoryAdmin: isTypeGuard(PRIVILEGE_TYPES.REPOSITORY_ADMIN) as any,
      isTypeRepositoryContentSelector: isTypeGuard(PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR) as any,
      isTypeScript: isTypeGuard(PRIVILEGE_TYPES.SCRIPT) as any,
    },
    services: {
      load: async () => {
        // Load privilege and reference data in parallel
        const [privilege, privilegeTypes, repos, selectors, scriptsData] = await Promise.all([
          // If privilege is preloaded, use it; otherwise fetch if privilegeId is provided
          preloadedPrivilege
            ? Promise.resolve(preloadedPrivilege)
            : privilegeId
            ? findPrivilege(privilegeId).catch((err: unknown) => {
                console.error('Failed to load privilege:', err);
                throw err; // Re-throw so form machine catches it
              })
            : Promise.resolve(null),
          fetchPrivilegeTypes().catch((err: unknown) => {
            console.error('Failed to load privilege types:', err);
            return [] as PrivilegeType[];
          }),
          restClient
            .get('/service/rest/v1/repositories')
            .then((data: unknown) => data as { name: string; format: string; type: string; online: boolean }[])
            .then((repos) => repos.map(r => ({
              name: r.name,
              format: r.format,
              type: r.type,
              status: { online: r.online }
            })))
            .catch((err: unknown) => {
              console.warn('Could not load repositories for privilege form:', err);
              return [] as { name: string; format: string; type: string; status: { online: boolean } }[];
            }),
          restClient
            .get(CONTENT_SELECTOR_API.BASE_URL)
            .then((data: unknown) => data as ContentSelector[])
            .catch((err: unknown) => {
              console.warn('Could not load content selectors:', err);
              return [] as ContentSelector[];
            }),
          restClient
            .get(ENDPOINTS.SCRIPTS)
            .then((data: unknown) => data as { name: string }[])
            .catch((err: unknown) => {
              console.warn('Could not load scripts:', err);
              return [] as { name: string }[];
            }),
        ]);

        // Return raw data arrays (no UI formatting)
        const repositories = Array.isArray(repos) ? repos : [];
        const formats = Array.isArray(repos) ? [...new Set(repos.map((r) => r.format))].sort() : [];
        const contentSelectors = Array.isArray(selectors) ? selectors : [];
        const scripts = Array.isArray(scriptsData) ? scriptsData : [];

        // Build initial form data from loaded privilege or use defaults
        const initialData: PrivilegeFormData = privilege
          ? {
              name: privilege.name,
              description: privilege.description,
              type: privilege.type,
              properties: privilege.properties,
              version: privilege.version,
            }
          : {
              name: '',
              description: '',
              type: initialTypeId || PRIVILEGE_TYPES.APPLICATION,
              properties: {},
            };

        return {
          data: initialData,
          privilege,
          privilegeTypes,
          repositories,
          formats,
          contentSelectors,
          scripts,
        };
      },
      // save service is provided via useForm options
    },
    // Custom event for privilege type changes (transitions to correct sub-state)
    on: {
      TYPE_CHANGE: [
        { target: '.application', cond: 'isTypeApplication', actions: ['changeType', 'validate', 'computePristine'] },
        { target: '.wildcard', cond: 'isTypeWildcard', actions: ['changeType', 'validate', 'computePristine'] },
        { target: '.repository-view', cond: 'isTypeRepositoryView', actions: ['changeType', 'validate', 'computePristine'] },
        { target: '.repository-admin', cond: 'isTypeRepositoryAdmin', actions: ['changeType', 'validate', 'computePristine'] },
        { target: '.repository-content-selector', cond: 'isTypeRepositoryContentSelector', actions: ['changeType', 'validate', 'computePristine'] },
        { target: '.script', cond: 'isTypeScript', actions: ['changeType', 'validate', 'computePristine'] },
      ],
    },
    // Privilege type variant sub-states within the editing state.
    // Each sub-state declares metadata about which fields are visible and required
    // for that type variant. This enables:
    // 1. The component to read field config from machine state (no switch/case)
    // 2. Model-based testing that auto-generates paths through every variant
    // 3. Single source of truth for form structure
    editingConfig: {
      defaultState: PRIVILEGE_TYPES.APPLICATION,
      typeField: 'type',
      states: {
        [PRIVILEGE_TYPES.APPLICATION]: {
          always: [
            { target: PRIVILEGE_TYPES.WILDCARD, cond: 'isTypeWildcard' },
            { target: PRIVILEGE_TYPES.REPOSITORY_VIEW, cond: 'isTypeRepositoryView' },
            { target: PRIVILEGE_TYPES.REPOSITORY_ADMIN, cond: 'isTypeRepositoryAdmin' },
            { target: PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR, cond: 'isTypeRepositoryContentSelector' },
            { target: PRIVILEGE_TYPES.SCRIPT, cond: 'isTypeScript' },
          ],
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.APPLICATION,
            fields: ['properties.domain', 'properties.actions'],
            requiredFields: ['properties.domain', 'properties.actions'],
            fieldConfig: {
              'properties.domain': { label: PrivilegesStrings.FORM_MACHINE.DOMAIN, type: 'combobox', allowCustom: true },
              'properties.actions': { label: CommonStrings.LABELS.ACTIONS, type: 'checkboxGroup' },
            },
          },
        },
        [PRIVILEGE_TYPES.WILDCARD]: {
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.WILDCARD,
            fields: ['properties.pattern'],
            requiredFields: ['properties.pattern'],
            fieldConfig: {
              'properties.pattern': { label: PrivilegesStrings.FORM_MACHINE.PATTERN, type: 'text', helpText: PrivilegesStrings.FORM_MACHINE.WILDCARD_PATTERN_E_G_ASTRUM },
            },
          },
        },
        [PRIVILEGE_TYPES.REPOSITORY_VIEW]: {
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.REPOSITORY_VIEW,
            fields: ['properties.format', 'properties.repository', 'properties.actions'],
            requiredFields: ['properties.format', 'properties.repository', 'properties.actions'],
            fieldConfig: {
              'properties.format': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY_FORMAT, type: 'combobox', allowCustom: true },
              'properties.repository': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY, type: 'combobox', allowCustom: true },
              'properties.actions': { label: CommonStrings.LABELS.ACTIONS, type: 'checkboxGroup' },
            },
          },
        },
        [PRIVILEGE_TYPES.REPOSITORY_ADMIN]: {
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.REPOSITORY_ADMIN,
            fields: ['properties.format', 'properties.repository', 'properties.actions'],
            requiredFields: ['properties.format', 'properties.repository', 'properties.actions'],
            fieldConfig: {
              'properties.format': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY_FORMAT, type: 'combobox', allowCustom: true },
              'properties.repository': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY, type: 'combobox', allowCustom: true },
              'properties.actions': { label: CommonStrings.LABELS.ACTIONS, type: 'checkboxGroup' },
            },
          },
        },
        [PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR]: {
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.REPOSITORY_CONTENT_SELECTOR,
            fields: ['properties.format', 'properties.repository', 'properties.contentSelector', 'properties.actions'],
            requiredFields: ['properties.format', 'properties.repository', 'properties.contentSelector', 'properties.actions'],
            fieldConfig: {
              'properties.format': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY_FORMAT, type: 'combobox', allowCustom: true },
              'properties.repository': { label: PrivilegesStrings.FORM_MACHINE.REPOSITORY, type: 'combobox', allowCustom: true },
              'properties.contentSelector': { label: PrivilegesStrings.FORM_MACHINE.CONTENT_SELECTOR, type: 'combobox' },
              'properties.actions': { label: CommonStrings.LABELS.ACTIONS, type: 'checkboxGroup' },
            },
          },
        },
        [PRIVILEGE_TYPES.SCRIPT]: {
          meta: {
            typeLabel: PrivilegesStrings.FORM_MACHINE.SCRIPT,
            fields: ['properties.name', 'properties.actions'],
            requiredFields: ['properties.name', 'properties.actions'],
            fieldConfig: {
              'properties.name': { label: PrivilegesStrings.FORM_MACHINE.SCRIPT_NAME, type: 'combobox', allowCustom: true },
              'properties.actions': { label: CommonStrings.LABELS.ACTIONS, type: 'checkboxGroup' },
            },
          },
        },
      },
    },
  });
}
