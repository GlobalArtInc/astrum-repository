import type { Translate } from '../../../i18n';
import type { AuthScheme, GroupType, LdapServer, LdapTemplate, Protocol } from '../api';

export type ProtocolChoice = 'ldap' | 'ldaps' | 'starttls';

export interface LdapValues {
  name: string;
  protocol: ProtocolChoice;
  useTrustStore: boolean;
  host: string;
  port: string;
  searchBase: string;
  authScheme: AuthScheme;
  authRealm: string;
  authUsername: string;
  authPassword: string;
  connectionTimeoutSeconds: string;
  connectionRetryDelaySeconds: string;
  maxIncidentsCount: string;
  userBaseDn: string;
  userSubtree: boolean;
  userObjectClass: string;
  userLdapFilter: string;
  userIdAttribute: string;
  userRealNameAttribute: string;
  userEmailAddressAttribute: string;
  userPasswordAttribute: string;
  ldapGroupsAsRoles: boolean;
  groupType: GroupType;
  groupBaseDn: string;
  groupSubtree: boolean;
  groupObjectClass: string;
  groupIdAttribute: string;
  groupMemberAttribute: string;
  groupMemberFormat: string;
  userMemberOfAttribute: string;
}

export type LdapErrors = Partial<Record<keyof LdapValues, string>>;

export const DEFAULT_PORTS: Record<ProtocolChoice, string> = { ldap: '389', ldaps: '636', starttls: '389' };

export const EMPTY_LDAP: LdapValues = {
  name: '',
  protocol: 'ldap',
  useTrustStore: false,
  host: '',
  port: DEFAULT_PORTS.ldap,
  searchBase: '',
  authScheme: 'SIMPLE',
  authRealm: '',
  authUsername: '',
  authPassword: '',
  connectionTimeoutSeconds: '30',
  connectionRetryDelaySeconds: '300',
  maxIncidentsCount: '3',
  userBaseDn: '',
  userSubtree: false,
  userObjectClass: 'inetOrgPerson',
  userLdapFilter: '',
  userIdAttribute: 'uid',
  userRealNameAttribute: 'cn',
  userEmailAddressAttribute: 'mail',
  userPasswordAttribute: '',
  ldapGroupsAsRoles: true,
  groupType: 'DYNAMIC',
  groupBaseDn: '',
  groupSubtree: false,
  groupObjectClass: 'groupOfUniqueNames',
  groupIdAttribute: 'cn',
  groupMemberAttribute: 'uniqueMember',
  groupMemberFormat: `uid=\${username},ou=people,dc=example,dc=com`,
  userMemberOfAttribute: 'memberOf',
};

export const AUTH_SCHEMES: AuthScheme[] = ['NONE', 'SIMPLE', 'DIGEST_MD5', 'CRAM_MD5'];

export function protocolChoice(server: Pick<LdapServer, 'protocol' | 'useStartTls'>): ProtocolChoice {
  if (server.protocol === 'LDAPS') {
    return 'ldaps';
  }
  return server.useStartTls ? 'starttls' : 'ldap';
}

export function secure(protocol: ProtocolChoice): boolean {
  return protocol !== 'ldap';
}

export function needsCredentials(scheme: AuthScheme): boolean {
  return scheme !== 'NONE';
}

export function usesRealm(scheme: AuthScheme): boolean {
  return scheme === 'DIGEST_MD5' || scheme === 'CRAM_MD5';
}

export function fromServer(server: LdapServer): LdapValues {
  const text = (value: string | undefined, fallback = '') => value ?? fallback;
  return {
    name: server.name,
    protocol: protocolChoice(server),
    useTrustStore: server.useTrustStore,
    host: server.host,
    port: String(server.port),
    searchBase: server.searchBase,
    authScheme: server.authScheme,
    authRealm: text(server.authRealm),
    authUsername: text(server.authUsername),
    authPassword: '',
    connectionTimeoutSeconds: String(server.connectionTimeoutSeconds),
    connectionRetryDelaySeconds: String(server.connectionRetryDelaySeconds),
    maxIncidentsCount: String(server.maxIncidentsCount),
    userBaseDn: text(server.userBaseDn),
    userSubtree: server.userSubtree,
    userObjectClass: text(server.userObjectClass),
    userLdapFilter: text(server.userLdapFilter),
    userIdAttribute: text(server.userIdAttribute),
    userRealNameAttribute: text(server.userRealNameAttribute),
    userEmailAddressAttribute: text(server.userEmailAddressAttribute),
    userPasswordAttribute: text(server.userPasswordAttribute),
    ldapGroupsAsRoles: server.ldapGroupsAsRoles,
    groupType: server.groupType ?? 'DYNAMIC',
    groupBaseDn: text(server.groupBaseDn),
    groupSubtree: server.groupSubtree,
    groupObjectClass: text(server.groupObjectClass, EMPTY_LDAP.groupObjectClass),
    groupIdAttribute: text(server.groupIdAttribute, EMPTY_LDAP.groupIdAttribute),
    groupMemberAttribute: text(server.groupMemberAttribute, EMPTY_LDAP.groupMemberAttribute),
    groupMemberFormat: text(server.groupMemberFormat, EMPTY_LDAP.groupMemberFormat),
    userMemberOfAttribute: text(server.userMemberOfAttribute, EMPTY_LDAP.userMemberOfAttribute),
  };
}

function optional(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === '' ? null : trimmed;
}

function integer(value: string): number | null {
  const trimmed = value.trim();
  if (!/^\d+$/.test(trimmed)) {
    return null;
  }
  return Number(trimmed);
}

export function toRequest(values: LdapValues, existing?: Pick<LdapServer, 'id' | 'order'>) {
  const protocol: Protocol = values.protocol === 'ldaps' ? 'LDAPS' : 'LDAP';
  const credentials = needsCredentials(values.authScheme);
  const groups = values.ldapGroupsAsRoles;
  const isStatic = groups && values.groupType === 'STATIC';
  const isDynamic = groups && values.groupType === 'DYNAMIC';
  return {
    ...(existing ? { id: existing.id, order: existing.order } : {}),
    name: values.name.trim(),
    protocol,
    useStartTls: values.protocol === 'starttls',
    useTrustStore: secure(values.protocol) && values.useTrustStore,
    host: values.host.trim(),
    port: integer(values.port),
    searchBase: values.searchBase.trim(),
    authScheme: values.authScheme,
    authRealm: credentials && usesRealm(values.authScheme) ? optional(values.authRealm) : null,
    authUsername: credentials ? optional(values.authUsername) : null,
    authPassword: credentials && values.authPassword !== '' ? values.authPassword : null,
    connectionTimeoutSeconds: integer(values.connectionTimeoutSeconds),
    connectionRetryDelaySeconds: integer(values.connectionRetryDelaySeconds),
    maxIncidentsCount: integer(values.maxIncidentsCount),
    userBaseDn: optional(values.userBaseDn),
    userSubtree: values.userSubtree,
    userObjectClass: optional(values.userObjectClass),
    userLdapFilter: optional(values.userLdapFilter),
    userIdAttribute: optional(values.userIdAttribute),
    userRealNameAttribute: optional(values.userRealNameAttribute),
    userEmailAddressAttribute: optional(values.userEmailAddressAttribute),
    userPasswordAttribute: optional(values.userPasswordAttribute),
    ldapGroupsAsRoles: groups,
    groupType: groups ? values.groupType : null,
    groupBaseDn: isStatic ? optional(values.groupBaseDn) : null,
    groupSubtree: isStatic && values.groupSubtree,
    groupObjectClass: isStatic ? optional(values.groupObjectClass) : null,
    groupIdAttribute: isStatic ? optional(values.groupIdAttribute) : null,
    groupMemberAttribute: isStatic ? optional(values.groupMemberAttribute) : null,
    groupMemberFormat: isStatic ? optional(values.groupMemberFormat) : null,
    userMemberOfAttribute: isDynamic ? optional(values.userMemberOfAttribute) : null,
  };
}

const HOST =
  /^(?:[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)*$/;
const IPV6 = /^\[?[0-9A-Fa-f:.]+\]?$/;

export function validHost(value: string): boolean {
  const host = value.trim();
  return (
    host.length > 0 && host.length <= 253 && (HOST.test(host) || (host.includes(':') && IPV6.test(host)))
  );
}

function inRange(value: string, min: number, max: number): boolean {
  const number = integer(value);
  return number !== null && number >= min && number <= max;
}

export function validateConnection(
  t: Translate,
  values: LdapValues,
  options: { names: string[]; passwordStored: boolean },
): LdapErrors {
  const errors: LdapErrors = {};
  const name = values.name.trim();
  if (name === '') {
    errors.name = t('identity.ldap.required');
  } else if (options.names.some((other) => other.toLowerCase() === name.toLowerCase())) {
    errors.name = t('identity.ldap.nameTaken');
  }
  if (values.host.trim() === '') {
    errors.host = t('identity.ldap.hostRequired');
  } else if (!validHost(values.host)) {
    errors.host = t('identity.ldap.hostInvalid');
  }
  if (!inRange(values.port, 1, 65535)) {
    errors.port = t('identity.ldap.portRange');
  }
  if (values.searchBase.trim() === '') {
    errors.searchBase = t('identity.ldap.searchBaseRequired');
  }
  if (needsCredentials(values.authScheme)) {
    if (values.authUsername.trim() === '') {
      errors.authUsername = t('identity.ldap.usernameRequired');
    }
    if (values.authPassword === '' && !options.passwordStored) {
      errors.authPassword = t('identity.ldap.passwordRequired');
    }
  }
  if (!inRange(values.connectionTimeoutSeconds, 1, 3600)) {
    errors.connectionTimeoutSeconds = t('identity.ldap.timeoutRange');
  }
  if (!inRange(values.connectionRetryDelaySeconds, 0, Number.MAX_SAFE_INTEGER)) {
    errors.connectionRetryDelaySeconds = t('identity.ldap.notNegative');
  }
  if (!inRange(values.maxIncidentsCount, 0, Number.MAX_SAFE_INTEGER)) {
    errors.maxIncidentsCount = t('identity.ldap.notNegative');
  }
  return errors;
}

export function validateMapping(t: Translate, values: LdapValues): LdapErrors {
  const errors: LdapErrors = {};
  const require = (field: keyof LdapValues, key: string) => {
    if (String(values[field]).trim() === '') {
      errors[field] = t(key);
    }
  };
  require('userObjectClass', 'identity.ldap.objectClassRequired');
  require('userIdAttribute', 'identity.ldap.userIdRequired');
  require('userRealNameAttribute', 'identity.ldap.realNameRequired');
  require('userEmailAddressAttribute', 'identity.ldap.emailRequired');
  if (values.ldapGroupsAsRoles && values.groupType === 'STATIC') {
    require('groupObjectClass', 'identity.ldap.groupObjectClassRequired');
    require('groupIdAttribute', 'identity.ldap.groupIdRequired');
    require('groupMemberAttribute', 'identity.ldap.groupMemberRequired');
    require('groupMemberFormat', 'identity.ldap.groupMemberFormatRequired');
  }
  if (values.ldapGroupsAsRoles && values.groupType === 'DYNAMIC') {
    require('userMemberOfAttribute', 'identity.ldap.memberOfRequired');
  }
  return errors;
}

export function applyTemplate(values: LdapValues, template: LdapTemplate): LdapValues {
  const pick = (value: string | null, fallback: string) => value ?? fallback;
  const groupType: GroupType = template.groupType?.toUpperCase() === 'STATIC' ? 'STATIC' : 'DYNAMIC';
  return {
    ...values,
    userBaseDn: template.userBaseDn ?? '',
    userSubtree: template.userSubtree,
    userObjectClass: pick(template.userObjectClass, ''),
    userLdapFilter: template.userLdapFilter ?? '',
    userIdAttribute: pick(template.userIdAttribute, ''),
    userRealNameAttribute: pick(template.userRealNameAttribute, ''),
    userEmailAddressAttribute: pick(template.userEmailAddressAttribute, ''),
    userPasswordAttribute: template.userPasswordAttribute ?? '',
    ldapGroupsAsRoles: template.ldapGroupsAsRoles,
    groupType,
    groupBaseDn: template.groupBaseDn ?? '',
    groupSubtree: template.groupSubtree,
    groupObjectClass: pick(template.groupObjectClass, values.groupObjectClass),
    groupIdAttribute: pick(template.groupIdAttribute, values.groupIdAttribute),
    groupMemberAttribute: pick(template.groupMemberAttribute, values.groupMemberAttribute),
    groupMemberFormat: pick(template.groupMemberFormat, values.groupMemberFormat),
    userMemberOfAttribute: pick(template.userMemberOfAttribute, values.userMemberOfAttribute),
  };
}

export function serverUrl(server: Pick<LdapServer, 'protocol' | 'host' | 'port'>): string {
  return `${server.protocol.toLowerCase()}://${server.host}:${server.port}`;
}

export function matches(server: LdapServer, filter: string): boolean {
  const wanted = filter.trim().toLowerCase();
  return (
    wanted === '' ||
    server.name.toLowerCase().includes(wanted) ||
    serverUrl(server).toLowerCase().includes(wanted)
  );
}

export function move<T>(items: readonly T[], from: number, to: number): T[] {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) {
    return [...items];
  }
  const next = [...items];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item as T);
  return next;
}

export function sameBindTarget(values: LdapValues, server: LdapServer): boolean {
  const realm = usesRealm(values.authScheme) ? values.authRealm.trim() : '';
  return (
    values.protocol === protocolChoice(server) &&
    values.host.trim().toLowerCase() === server.host.toLowerCase() &&
    integer(values.port) === server.port &&
    values.authScheme === server.authScheme &&
    values.authUsername.trim() === (server.authUsername ?? '') &&
    realm === (server.authRealm ?? '')
  );
}

export function base64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}
