import { INTERNAL, V1 } from '../../api';

export const SECURITY = `${V1}/security`;
export const PRIVILEGES = `${SECURITY}/privileges`;
export const PRIVILEGE_TYPES = `${INTERNAL}/ui/privileges/types`;
export const ROLES = `${SECURITY}/roles`;
export const ASSIGNABLE_ROLES = `${SECURITY}/roles/assignable`;
export const USERS = `${SECURITY}/users`;
export const USER_SOURCES = `${SECURITY}/user-sources`;
export const SELECTORS = `${SECURITY}/content-selectors`;
export const ANONYMOUS = `${SECURITY}/anonymous`;
export const REALMS_AVAILABLE = `${SECURITY}/realms/available`;
export const REALMS_ACTIVE = `${SECURITY}/realms/active`;
export const SSRF = `${SECURITY}/ssrf-protection`;
export const REPOSITORIES = `${V1}/repositories`;

export const PRIVILEGE_TYPE_IDS = [
  'application',
  'wildcard',
  'repository-view',
  'repository-admin',
  'repository-content-selector',
] as const;

export type PrivilegeTypeId = (typeof PRIVILEGE_TYPE_IDS)[number];

export type Action =
  | 'READ'
  | 'BROWSE'
  | 'EDIT'
  | 'ADD'
  | 'DELETE'
  | 'RUN'
  | 'ASSOCIATE'
  | 'DISASSOCIATE'
  | 'START'
  | 'STOP'
  | 'ALL';

export interface Privilege {
  type: string;
  name: string;
  description: string;
  readOnly: boolean;
  actions?: Action[];
  domain?: string;
  pattern?: string;
  format?: string;
  repository?: string;
  contentSelector?: string;
}

export type FieldKind = 'text' | 'permission' | 'format' | 'repository' | 'contentSelector' | 'actions';

export interface PrivilegeField {
  id: string;
  kind: FieldKind;
  required: boolean;
  values?: string[];
  any: boolean;
}

export interface PrivilegeType {
  id: string;
  fields: PrivilegeField[];
}

export interface Role {
  id: string;
  source: string;
  name: string;
  description: string;
  readOnly: boolean;
  privileges: string[];
  roles: string[];
}

export type UserStatus = 'active' | 'locked' | 'disabled' | 'changepassword';

export interface User {
  userId: string;
  firstName: string | null;
  lastName: string | null;
  emailAddress: string | null;
  source: string;
  status: UserStatus;
  readOnly: boolean;
  roles: string[];
  externalRoles: string[];
}

export interface UserSource {
  id: string;
  name: string;
}

export interface Anonymous {
  enabled: boolean;
  userId: string;
  realmName: string;
}

export interface Realm {
  id: string;
  name: string;
}

export interface SsrfProtection {
  enabled: boolean;
  allowedIPs: string[];
  allowedDomains: string[];
}

export interface Repository {
  name: string;
  format: string;
  type: string;
}

export interface ContentSelector {
  name: string;
  type: string;
  description: string | null;
  expression: string;
}

export const DEFAULT_SOURCE = 'default';
