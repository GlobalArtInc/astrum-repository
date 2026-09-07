

/**
 * Role data model matching the backend API (RoleXO)
 */
export interface Role {
  id: string;
  version: string;
  source: string;
  name: string;
  description: string;
  readOnly: boolean;
  privileges: string[];
  roles: string[];
}

/**
 * Role reference (id/name pair)
 */
export interface RoleReference {
  id: string;
  name: string;
}

/**
 * Privilege reference (id/name pair)
 */
export interface PrivilegeReference {
  id: string;
  name: string;
  description?: string;
}

/**
 * Role source reference
 */
export interface RoleSource {
  id: string;
  name: string;
}

/**
 * Create/update role form data
 */
export interface RoleFormData {
  id: string;
  name: string;
  description: string;
  privileges: string[];
  roles: string[];
  source?: string;
  version?: string;
  // External role mapping fields
  roleType?: 'astrum' | 'external';
  externalSource?: string;  // LDAP, SAML, Crowd
  mappedRole?: string;  // External role identifier
}

/**
 * Form validation errors
 */
export interface RoleFormErrors {
  id?: string;
  name?: string;
  description?: string;
  privileges?: string;
  roles?: string;
}

/**
 * Sort direction type
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Sortable fields for roles list
 */
export type RoleSortField = 'id' | 'name' | 'description' | 'source';

/**
 * Props for RolesPage component
 */
export interface RolesPageProps {
  className?: string;
}

/**
 * Props for RolesList component
 */
export interface RolesListProps {
  onSelect: (roleId: string, mode?: 'profile' | 'edit') => void;
  onDelete?: (roleId: string, roleName?: string) => void;
  onCreate: () => void;
  /**
   * When false, hides the row Edit (pencil) action so a user without
   * astrum:roles:update sees only the read-only View action (ASTRUM-54212).
   * Defaults to true.
   */
  canUpdate?: boolean;
  canDelete?: boolean;
}

/**
 * Props for RoleDetail component
 */
export interface RoleDetailProps {
  role: Role | null;
  loading: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onDelete: () => void;
  onCancel: () => void;
  onComplete?: () => void;
  error?: string;
}

/**
 * Props for RoleForm component
 */
export interface RoleFormProps {
  role?: Role | null;
  isCreate: boolean;
  onCancel: () => void;
  onDelete?: () => void;
  onComplete?: () => void;
  loading?: boolean;
  error?: string;
}

/**
 * Props for PrivilegesSelectionModal
 */
export interface PrivilegesSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  availablePrivileges: PrivilegeReference[];
  selectedPrivileges: string[];
  onSave: (privileges: string[]) => void;
  loading?: boolean;
}

/**
 * Props for RolesSelectionModal
 */
export interface RolesSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  availableRoles: RoleReference[];
  selectedRoles: string[];
  currentRoleId?: string; // To exclude current role from selection
  onSave: (roles: string[]) => void;
  loading?: boolean;
}

/**
 * Default source for Astrum roles
 */
export const ASTRUM_SOURCE = 'Default';
export const DEFAULT_SOURCE = 'default';

/**
 * Default page size for roles list
 */
export const ROLES_PAGE_SIZE = 25;

/**
 * Labels for privilege groups in the combobox
 */
export const PRIVILEGE_GROUP_LABELS: Record<string, string> = {
  'as-repository-view': 'Repository View',
  'as-repository-admin': 'Repository Admin',
  'as-repository-content-selector': 'Content Selector',
  'as-application': 'Application',
  'as-script': 'Script',
  'as-healthcheck': 'Health Check',
  'as-blobstore': 'Blob Store',
};

/**
 * Message shown when LDAP search returns no results
 */
export const LDAP_EMPTY_SEARCH_MESSAGE = 'If no results are found, you can still save this mapping and create an LDAP role with an identical name later. Mapping will not apply until an identically named LDAP role exists.';

/**
 * Extended form context for roles form machine
 */
export interface RolesFormContext {
  data: RoleFormData;
  role: Role | null;
  allPrivileges: PrivilegeReference[];
  allRoles: RoleReference[];
  allSources: RoleSource[];
}

/**
 * Check if role is from external source.
 * Default/Astrum source (case-insensitive) is NOT external.
 */
export const isExternalRole = (source: string): boolean => {
  const s = (source || '').toLowerCase();
  return s !== '' && s !== ASTRUM_SOURCE.toLowerCase() && s !== DEFAULT_SOURCE && s !== 'astrum';
};

/**
 * Format role source for display.
 * The API returns "default" (lowercase) for built-in roles; the UI should show "Default".
 */
export function formatRoleSourceDisplay(source: string): string {
  if (!source || source === DEFAULT_SOURCE || source === 'Astrum') {
    return 'Default';
  }
  return source;
}

/**
 * Check if role is read-only.
 * A role is locked if:
 * 1. readOnly: true is returned by the API
 * 2. It comes from an external source (LDAP, Crowd, etc.)
 */
export const isReadOnlyRole = (role: Role): boolean => 
  role.readOnly === true || isExternalRole(role.source);


