import PrivilegesStrings from '../../../../constants/pages/settings/security/PrivilegesStrings';
import CommonStrings from '../../../../constants/CommonStrings';


/**
 * Privilege data model matching the backend API (PrivilegeXO)
 */
export interface Privilege {
  id: string;
  version: string;
  name: string;
  description: string;
  type: string;
  readOnly: boolean;
  properties: Record<string, string>;
  permission: string;
}

/**
 * Privilege type with form fields
 */
export interface PrivilegeType {
  id: string;
  name: string;
  formFields: FormField[] | null;
}

/**
 * Form field definition from backend
 */
export interface FormField {
  id: string;
  type: string;
  label: string;
  helpText?: string;
  required?: boolean;
  regexValidation?: string;
  initialValue?: string;
  storeApi?: string;
  storeFilters?: Record<string, string>;
  idMapping?: string;
  nameMapping?: string;
  allowAutocomplete?: boolean;
}

/**
 * Privilege reference (id/name pair)
 */
export interface PrivilegeReference {
  id: string;
  name: string;
}

/**
 * Create/update privilege form data
 */
export interface PrivilegeFormData {
  id?: string;
  name: string;
  description: string;
  type: string;
  properties: Record<string, string>;
  version?: string;
}

/**
 * Form validation errors
 */
export interface PrivilegeFormErrors {
  name?: string;
  description?: string;
  type?: string;
  [key: string]: string | undefined; // Dynamic property errors
}

/**
 * Sort direction type
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Sortable fields for privileges list
 */
export type PrivilegeSortField = 'name' | 'description' | 'type' | 'permission';

/**
 * Props for PrivilegesPage component
 */
export interface PrivilegesPageProps {
  className?: string;
}

/**
 * Props for PrivilegesList component
 */
export interface PrivilegesListProps {
  /** Navigate to profile (read-only) - used by row click and View icon */
  onSelect: (privilegeId: string) => void;
  /** Navigate to edit form - used by Edit icon */
  onEdit?: (privilegeId: string) => void;
  onDelete?: (privilegeId: string, privilegeName?: string) => void;
  onCreate: () => void;
  canEdit?: boolean;
  canDelete?: boolean;
}

/**
 * Props for PrivilegeDetail component
 */
export interface PrivilegeDetailProps {
  privilege: Privilege | null;
  loading: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onSave: (data: PrivilegeFormData) => Promise<void>;
  onDelete: () => void;
  onCancel: () => void;
  error?: string;
}

/**
 * Props for PrivilegeForm component
 */
export interface PrivilegeFormProps {
  privilege?: Privilege | null;
  isCreate: boolean;
  onSave: (data: PrivilegeFormData) => Promise<void>;
  onCancel: () => void;
  onDelete?: () => void;
  loading?: boolean;
  error?: string;
}

/**
 * Known privilege types
 */
export const PRIVILEGE_TYPES = {
  APPLICATION: 'application',
  REPOSITORY_ADMIN: 'repository-admin',
  REPOSITORY_CONTENT_SELECTOR: 'repository-content-selector',
  REPOSITORY_VIEW: 'repository-view',
  SCRIPT: 'script',
  WILDCARD: 'wildcard',
} as const;

/**
 * Display names for privilege types
 */
export const PRIVILEGE_TYPE_LABELS: Record<string, string> = {
  [PRIVILEGE_TYPES.APPLICATION]: PrivilegesStrings.TYPES.APPLICATION,
  [PRIVILEGE_TYPES.REPOSITORY_ADMIN]: PrivilegesStrings.TYPES.REPOSITORY_ADMIN,
  [PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR]: PrivilegesStrings.TYPES.REPOSITORY_CONTENT_SELECTOR,
  [PRIVILEGE_TYPES.REPOSITORY_VIEW]: PrivilegesStrings.TYPES.REPOSITORY_VIEW,
  [PRIVILEGE_TYPES.SCRIPT]: PrivilegesStrings.TYPES.SCRIPT,
  [PRIVILEGE_TYPES.WILDCARD]: PrivilegesStrings.TYPES.WILDCARD,
};

/**
 * Get display label for privilege type
 */
export const getPrivilegeTypeLabel = (type: string): string => 
  PRIVILEGE_TYPE_LABELS[type] || type;

/**
 * Check if privilege is read-only
 */
export const isReadOnlyPrivilege = (privilege: Privilege): boolean => privilege.readOnly === true;

/**
 * Option type for form fields
 */
export interface FieldOption {
  value: string;
  label: string;
}

/**
 * Available actions for Application privileges
 * These are application-specific operations beyond standard CRUD
 */
export const APPLICATION_ACTIONS: FieldOption[] = [
  { value: 'create', label: CommonStrings.ACTIONS.CREATE },
  { value: 'read', label: PrivilegesStrings.TYPES.READ },
  { value: 'update', label: PrivilegesStrings.TYPES.UPDATE },
  { value: 'delete', label: CommonStrings.ACTIONS.DELETE },
  { value: 'start', label: PrivilegesStrings.TYPES.START },
  { value: 'stop', label: PrivilegesStrings.TYPES.STOP },
  { value: 'associate', label: PrivilegesStrings.TYPES.ASSOCIATE },
  { value: 'disassociate', label: PrivilegesStrings.TYPES.DISASSOCIATE },
];

/**
 * Available actions for Repository privileges (View & Admin)
 * Standard repository operations
 */
export const REPOSITORY_ACTIONS: FieldOption[] = [
  { value: 'browse', label: PrivilegesStrings.TYPES.BROWSE },
  { value: 'read', label: PrivilegesStrings.TYPES.READ },
  { value: 'update', label: CommonStrings.ACTIONS.EDIT },
  { value: 'create', label: PrivilegesStrings.TYPES.ADD },
  { value: 'delete', label: CommonStrings.ACTIONS.DELETE },
];

/**
 * Available actions for Content Selector privileges
 * Same as repository actions
 */
export const CONTENT_SELECTOR_ACTIONS: FieldOption[] = [
  { value: 'browse', label: PrivilegesStrings.TYPES.BROWSE },
  { value: 'read', label: PrivilegesStrings.TYPES.READ },
  { value: 'update', label: CommonStrings.ACTIONS.EDIT },
  { value: 'create', label: PrivilegesStrings.TYPES.ADD },
  { value: 'delete', label: CommonStrings.ACTIONS.DELETE },
];

/**
 * Available actions for Script privileges
 * Operations related to script execution
 */
export const SCRIPT_ACTIONS: FieldOption[] = [
  { value: 'browse', label: PrivilegesStrings.TYPES.BROWSE },
  { value: 'read', label: PrivilegesStrings.TYPES.READ },
  { value: 'update', label: CommonStrings.ACTIONS.EDIT },
  { value: 'create', label: PrivilegesStrings.TYPES.ADD },
  { value: 'delete', label: CommonStrings.ACTIONS.DELETE },
  { value: 'run', label: PrivilegesStrings.TYPES.RUN },
];

/**
 * Get available actions for a privilege type
 */
export const getActionsForPrivilegeType = (type: string): FieldOption[] => {
  switch (type) {
    case PRIVILEGE_TYPES.APPLICATION:
      return APPLICATION_ACTIONS;
    case PRIVILEGE_TYPES.REPOSITORY_ADMIN:
    case PRIVILEGE_TYPES.REPOSITORY_VIEW:
      return REPOSITORY_ACTIONS;
    case PRIVILEGE_TYPES.REPOSITORY_CONTENT_SELECTOR:
      return CONTENT_SELECTOR_ACTIONS;
    case PRIVILEGE_TYPES.SCRIPT:
      return SCRIPT_ACTIONS;
    case PRIVILEGE_TYPES.WILDCARD:
      return []; // Wildcard privileges don't have action checkboxes
    default:
      return [];
  }
};

/**
 * Common application domains
 * These are the known domains used throughout the application
 */
export const APPLICATION_DOMAINS: FieldOption[] = [
  { value: '*', label: '* (All)' },
  { value: 'analytics', label: 'analytics' },
  { value: 'apikey', label: 'apikey' },
  { value: 'blobstores', label: 'blobstores' },
  { value: 'bundles', label: 'bundles' },
  { value: 'capabilities', label: 'capabilities' },
  { value: 'component', label: 'component' },
  { value: 'healthcheck', label: 'healthcheck' },
  { value: 'healthchecksummary', label: 'healthchecksummary' },
  { value: 'ldap', label: 'ldap' },
  { value: 'licensing', label: 'licensing' },
  { value: 'logging', label: 'logging' },
  { value: 'metrics', label: 'metrics' },
  { value: 'privileges', label: 'privileges' },
  { value: 'realms', label: 'realms' },
  { value: 'replication', label: 'replication' },
  { value: 'repository-admin', label: 'repository-admin' },
  { value: 'roles', label: 'roles' },
  { value: 'routing-rules', label: 'routing-rules' },
  { value: 'script', label: 'script' },
  { value: 'search', label: 'search' },
  { value: 'security', label: 'security' },
  { value: 'selectors', label: 'selectors' },
  { value: 'settings', label: 'settings' },
  { value: 'ssl-truststore', label: 'ssl-truststore' },
  { value: 'status', label: 'status' },
  { value: 'support', label: 'support' },
  { value: 'tasks', label: 'tasks' },
  { value: 'users', label: 'users' },
  { value: 'userschangepw', label: 'userschangepw' },
  { value: 'wonderland', label: 'wonderland' },
];
