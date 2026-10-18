import { INTERNAL, V1 } from '../../api';

export const LDAP = `${V1}/security/ldap`;
export const LDAP_TEMPLATES = `${LDAP}/templates`;
export const LDAP_ORDER = `${LDAP}/change-order`;
export const LDAP_CACHE = `${LDAP}/cache`;
export const LDAP_VERIFY_CONNECTION = `${LDAP}/verify-connection`;
export const LDAP_VERIFY_MAPPING = `${LDAP}/verify-user-mapping`;
export const LDAP_VERIFY_LOGIN = `${LDAP}/verify-login`;
export const OIDC = `${INTERNAL}/ui/oauth2`;
export const DEFAULT_ROLE = `${INTERNAL}/ui/default-role`;
export const ROLES = `${V1}/security/roles`;
export const EXTERNAL_ROLES = `${INTERNAL}/ui/roles`;
export const ROLE_SOURCES = `${INTERNAL}/ui/roles/sources`;

export const LDAP_DOCS = 'https://links.globalart.dev/products/astrum/docs/ldap';
export const LDAP_HELP = 'https://help.globalart.dev/en/ldap.html';

export type Protocol = 'LDAP' | 'LDAPS';
export type AuthScheme = 'NONE' | 'SIMPLE' | 'DIGEST_MD5' | 'CRAM_MD5';
export type GroupType = 'STATIC' | 'DYNAMIC';

export interface LdapServer {
  id: string;
  order: number;
  name: string;
  protocol: Protocol;
  useTrustStore: boolean;
  useStartTls: boolean;
  host: string;
  port: number;
  searchBase: string;
  authScheme: AuthScheme;
  authRealm?: string;
  authUsername?: string;
  connectionTimeoutSeconds: number;
  connectionRetryDelaySeconds: number;
  maxIncidentsCount: number;
  userBaseDn?: string;
  userSubtree: boolean;
  userObjectClass?: string;
  userLdapFilter?: string;
  userIdAttribute?: string;
  userRealNameAttribute?: string;
  userEmailAddressAttribute?: string;
  userPasswordAttribute?: string;
  ldapGroupsAsRoles: boolean;
  groupType?: GroupType;
  groupBaseDn?: string;
  groupSubtree: boolean;
  groupObjectClass?: string;
  groupIdAttribute?: string;
  groupMemberAttribute?: string;
  groupMemberFormat?: string;
  userMemberOfAttribute?: string;
}

export interface LdapTemplate {
  name: string;
  userBaseDn: string | null;
  userSubtree: boolean;
  userObjectClass: string | null;
  userLdapFilter: string | null;
  userIdAttribute: string | null;
  userRealNameAttribute: string | null;
  userEmailAddressAttribute: string | null;
  userPasswordAttribute: string | null;
  ldapGroupsAsRoles: boolean;
  groupType: string | null;
  groupBaseDn: string | null;
  groupSubtree: boolean;
  groupObjectClass: string | null;
  groupIdAttribute: string | null;
  groupMemberAttribute: string | null;
  groupMemberFormat: string | null;
  userMemberOfAttribute: string | null;
}

export interface LdapUser {
  username: string;
  realName: string | null;
  email: string | null;
  membership: string[];
}

export interface OidcSettings {
  issuerUrl: string | null;
  clientId: string | null;
  clientSecret: string | null;
  clientSecretSet: boolean;
  idpAuthorizationUrl: string | null;
  idpTokenUrl: string | null;
  idpJwksUrl: string | null;
  idpLogoutUrl: string | null;
  idpJwsAlgorithm: string | null;
  idpJwks: string | null;
  usernameClaim: string | null;
  firstNameClaim: string | null;
  lastNameClaim: string | null;
  emailClaim: string | null;
  groupsClaim: string | null;
  exactMatchClaims: Record<string, string>;
  authorizationCustomParams: Record<string, string>;
  tokenRequestCustomParams: Record<string, string>;
  useTrustStore: boolean;
  redirectUri: string;
  configured: boolean;
  enabled: boolean;
}

export interface DefaultRole {
  role: string | null;
  enabled: boolean;
  missing: boolean;
}

export interface Role {
  id: string;
  source: string;
  name: string;
  description: string;
  readOnly: boolean;
}

export interface RoleSource {
  id: string;
  name: string;
}
