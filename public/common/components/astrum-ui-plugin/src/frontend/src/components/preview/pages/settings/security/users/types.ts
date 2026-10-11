import UsersStrings from '../../../../constants/pages/settings/security/UsersStrings';


/**
 * User data model matching the backend API
 */
export interface User {
  userId: string;
  realm: string;
  source: string;
  firstName: string;
  lastName: string;
  emailAddress: string;
  email?: string; // Alias used in some API responses
  status: 'active' | 'disabled';
  roles: string[];
  externalRoles?: string[];
  readOnly?: boolean;
}

/**
 * Wire shape returned by the REST API. Kept separate from `User` so the
 * UI can normalize (e.g. alias `emailAddress` -> `email`, default `roles`).
 * Shared here so `useUsersApi` and `usersFormMachine` cannot drift.
 */
export interface RestUser {
  userId: string;
  firstName: string;
  lastName: string;
  emailAddress: string;
  source: string;
  status: 'active' | 'disabled';
  roles: string[];
  externalRoles?: string[];
  readOnly?: boolean;
}

/**
 * Normalize a REST user payload into the UI `User` model.
 */
export function restToUser(rest: RestUser): User {
  return {
    userId: rest.userId,
    realm: rest.source,
    source: rest.source,
    firstName: rest.firstName,
    lastName: rest.lastName,
    emailAddress: rest.emailAddress,
    email: rest.emailAddress,
    status: rest.status,
    roles: rest.roles || [],
    externalRoles: rest.externalRoles,
    readOnly: rest.readOnly,
  };
}

/**
 * User source/realm information
 */
export interface UserSource {
  id: string;
  name: string;
}

/**
 * Role information for assignment
 */
export interface Role {
  id: string;
  name: string;
  description?: string;
  source: string;
  roles?: string[];
  privileges?: string[];
  readOnly?: boolean;
}

/**
 * Create/update user form data
 */
export interface UserFormData {
  userId: string;
  firstName: string;
  lastName: string;
  emailAddress: string;
  password?: string;
  passwordConfirm?: string;
  status: boolean; // true = active, false = disabled
  roles: string[];
  source?: string;
}

/**
 * Form validation errors
 */
export interface UserFormErrors {
  userId?: string;
  firstName?: string;
  lastName?: string;
  emailAddress?: string;
  password?: string;
  passwordConfirm?: string;
  roles?: string;
}

/**
 * Sort direction type
 */
export type SortDirection = 'asc' | 'desc' | null;

/**
 * Sortable fields for users list
 */
export type UserSortField = 'userId' | 'realm' | 'firstName' | 'lastName' | 'email' | 'status';

/**
 * Props for UsersPage component
 */
export interface UsersPageProps {
  className?: string;
}

/**
 * Props for UsersList component
 */
export interface UsersListProps {
  /** Row click: navigate to Edit (for update-permitted users) or read-only User Profile. Parent owns the permission branch. */
  onSelect: (userId: string, realm: string) => void;
  /** Optional per-row aria-label producer. Lets the parent set an accessible name that matches the row's actual destination. */
  getRowAriaLabel?: (user: User) => string;
  onCreate: () => void;
  /** When true, fetches OAuth2 users and hides the source filter (cloud distribution only) */
  isCloud?: boolean;
}

/**
 * Default source for local users
 */
export const DEFAULT_SOURCE = 'default';

/**
 * User status options
 */
export const USER_STATUSES = {
  active: { id: 'active', label: UsersStrings.TYPES.ACTIVE },
  disabled: { id: 'disabled', label: UsersStrings.TYPES.DISABLED },
} as const;

/**
 * Check if user is from external source
 */
export const isExternalUser = (source: string): boolean => source !== DEFAULT_SOURCE;

/**
 * Get full name from user
 */
export const getFullName = (user: Pick<User, 'firstName' | 'lastName'>): string => 
  `${user.firstName || ''} ${user.lastName || ''}`.trim();

/**
 * Get source label (Local vs external source name)
 */
export const getSourceLabel = (source: string): string =>
  isExternalUser(source) ? source : UsersStrings.TYPES.LOCAL;

/**
 * Context for evaluating whether a user is protected from deletion.
 * Callers pass primitives read from ExtJS state, keeping this helper React-free.
 */
export interface ProtectionContext {
  anonymousUsername?: string | null;
  currentUserId?: string | null;
}

/**
 * Returns a human-readable reason if the user is protected from deletion, else null.
 * A protected user has the Delete button rendered but disabled with this reason as tooltip.
 */
export const getUserProtectionReason = (
  user: Pick<User, 'userId' | 'source'>,
  ctx: ProtectionContext,
): string | null => {
  if (isExternalUser(user.source)) {
    return UsersStrings.TYPES.EXTERNAL_USERS_CANNOT_DELETED_ASTRUM;
  }
  if (ctx.anonymousUsername && user.userId === ctx.anonymousUsername) {
    return UsersStrings.TYPES.ANONYMOUS_USER_SYSTEM_ACCOUNT_CANNOT;
  }
  if (ctx.currentUserId && user.userId === ctx.currentUserId) {
    return UsersStrings.TYPES.YOU_CANNOT_DELETE_OWN_ACCOUNT;
  }
  return null;
};

/**
 * True when the user is protected from deletion (see getUserProtectionReason).
 */
export const isProtectedUser = (
  user: Pick<User, 'userId' | 'source'>,
  ctx: ProtectionContext,
): boolean => getUserProtectionReason(user, ctx) !== null;
