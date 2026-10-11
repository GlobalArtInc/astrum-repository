import CapabilitiesStrings from '../../../../constants/pages/settings/system/CapabilitiesStrings';

/**
 * Capability state enum
 */
export type CapabilityState = 'active' | 'disabled' | 'error' | 'passive';

/**
 * Form field type for dynamic capability forms
 */
export type FormFieldType =
  | 'string'
  | 'text'
  | 'text-area'
  | 'password'
  | 'boolean'
  | 'checkbox'
  | 'number'
  | 'url'
  | 'itemselect'
  | 'combobox'
  | 'repo-or-group-target'
  | 'repo-target';

/**
 * Form field definition from capability type descriptor
 */
export interface FormField {
  id: string;
  type: FormFieldType;
  label: string;
  helpText?: string;
  required: boolean;
  disabled: boolean;
  readOnly: boolean;
  regexValidation?: string;
  initialValue?: string | boolean | number;
  attributes?: Record<string, unknown>;
  minValue?: number;
  maxValue?: number;
  storeApi?: string;
  storeFilters?: Record<string, string>;
  idMapping?: string;
  nameMapping?: string;
  allowAutocomplete?: boolean;
}

/**
 * Capability type - defines what types of capabilities can be created
 */
export interface CapabilityType {
  id: string;
  name: string;
  about?: string;
  formFields?: FormField[];
}

/**
 * Capability instance
 */
export interface Capability {
  id: string;
  typeId: string;
  typeName: string;
  enabled: boolean;
  active: boolean;
  error: boolean;
  state: CapabilityState;
  stateDescription?: string;
  description?: string;
  notes?: string;
  status?: string;
  properties: Record<string, string>;
  tags?: Record<string, string>;
  disableWarningMessage?: string;
  deleteWarningMessage?: string;
  isSystem?: boolean;
}

/**
 * Form data for creating/updating a capability
 */
export interface CapabilityFormData {
  id?: string;
  typeId: string;
  enabled: boolean;
  notes?: string;
  properties: Record<string, string>;
}

/**
 * View mode for the capabilities page
 */
export type CapabilitiesViewMode = 'list' | 'selectType' | 'create' | 'detail';

/**
 * Page props
 */
export interface CapabilitiesPageProps {
  className?: string;
}

/**
 * Helper function to get state icon color
 */
export function getStateColor(state: CapabilityState): string {
  switch (state) {
    case 'active':
      return 'var(--color-success)';
    case 'disabled':
      return 'var(--color-text-secondary)';
    case 'error':
      return 'var(--color-danger)';
    case 'passive':
      return 'var(--color-warning)';
    default:
      return 'var(--color-text-secondary)';
  }
}

const STATE_NAMES: Record<CapabilityState, string> = {
  active: CapabilitiesStrings.LIST.ACTIVE,
  disabled: CapabilitiesStrings.LIST.DISABLED,
  error: CapabilitiesStrings.LIST.ERROR,
  passive: CapabilitiesStrings.LIST.PASSIVE,
};

const STATE_BADGE_COLORS: Record<CapabilityState, 'green' | 'gray' | 'red' | 'orange'> = {
  active: 'green',
  disabled: 'gray',
  error: 'red',
  passive: 'orange',
};

/**
 * Helper function to get a displayable state name
 */
export function getStateName(state: CapabilityState): string {
  return STATE_NAMES[state] ?? state.charAt(0).toUpperCase() + state.slice(1);
}

/**
 * Helper function to get the colour of the state badge
 */
export function getStateBadgeColor(state: CapabilityState): 'green' | 'gray' | 'red' | 'orange' {
  return STATE_BADGE_COLORS[state] ?? 'gray';
}

/**
 * Helper function to get capability description
 */
export function getCapabilityDescription(capability: Capability): string {
  let description = capability.typeName;
  if (capability.description) {
    description += ' - ' + capability.description;
  }
  return description;
}

const CAPABILITY_CATEGORIES: Record<string, string> = {
  audit: CapabilitiesStrings.LIST.CATEGORY_AUDIT,
  baseurl: CapabilitiesStrings.LIST.CORE,
  crowd: CapabilitiesStrings.LIST.SECURITY,
  customs3regions: CapabilitiesStrings.LIST.CORE,
  defaultrole: CapabilitiesStrings.LIST.SECURITY,
  // 'firewall.audit' deliberately omitted: filtered out at fetchCapabilities() and not
  // advertised by /v1/capabilities/types post-migration. See useCapabilitiesApi.ts.
  healthcheck: CapabilitiesStrings.LIST.HEALTH_CHECK,
  'license-expiration': CapabilitiesStrings.LIST.CORE,
  LegacyUrlCapability: CapabilitiesStrings.LIST.CORE,
  'node.identity': CapabilitiesStrings.LIST.CORE,
  outreach: CapabilitiesStrings.LIST.CORE,
  OutreachManagementCapability: CapabilitiesStrings.LIST.CORE,
  'browse.trim': CapabilitiesStrings.LIST.REPOSITORY,
  rutauth: CapabilitiesStrings.LIST.SECURITY,
  'scheduling.scheduler': CapabilitiesStrings.LIST.SCHEDULING,
  StorageSettings: CapabilitiesStrings.LIST.CORE,
  'rapture.branding': 'UI',
  'rapture.settings': 'UI',
  migration: CapabilitiesStrings.LIST.CORE,
  'webhook.global': CapabilitiesStrings.LIST.WEBHOOK,
  'webhook.repository': CapabilitiesStrings.LIST.WEBHOOK,
};

export function getCategoryForType(typeId: string): string {
  return CAPABILITY_CATEGORIES[typeId] || CapabilitiesStrings.LIST.CATEGORY_OTHER;
}

const TAG_LABELS: Record<string, string> = {
  Category: CapabilitiesStrings.LIST.CATEGORY,
  Repository: CapabilitiesStrings.LIST.REPOSITORY,
};

export function getTagLabel(key: string): string {
  return TAG_LABELS[key] || key;
}

export function getTagValue(capability: Pick<Capability, 'typeId'>, key: string, value: string): string {
  return key === 'Category' ? getCategoryForType(capability.typeId) : value;
}
