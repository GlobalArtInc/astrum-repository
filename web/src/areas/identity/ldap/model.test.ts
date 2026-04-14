import { describe, expect, it } from 'vitest';
import type { Translate } from '../../../i18n';
import type { LdapServer, LdapTemplate } from '../api';
import {
  applyTemplate,
  base64,
  EMPTY_LDAP,
  fromServer,
  matches,
  move,
  sameBindTarget,
  serverUrl,
  toRequest,
  validateConnection,
  validateMapping,
  validHost,
} from './model';

const t = ((key: string) => key) as unknown as Translate;
const MEMBER_FORMAT = `uid=\${username},ou=people,dc=example,dc=com`;

const SERVER: LdapServer = {
  id: 'id-1',
  order: 2,
  name: 'corp',
  protocol: 'LDAP',
  useTrustStore: false,
  useStartTls: true,
  host: 'ldap.example.com',
  port: 389,
  searchBase: 'dc=example,dc=com',
  authScheme: 'SIMPLE',
  authUsername: 'cn=svc,dc=example,dc=com',
  connectionTimeoutSeconds: 30,
  connectionRetryDelaySeconds: 300,
  maxIncidentsCount: 3,
  userBaseDn: 'ou=people',
  userSubtree: true,
  userObjectClass: 'inetOrgPerson',
  userIdAttribute: 'uid',
  userRealNameAttribute: 'cn',
  userEmailAddressAttribute: 'mail',
  ldapGroupsAsRoles: true,
  groupType: 'STATIC',
  groupBaseDn: 'ou=groups',
  groupSubtree: false,
  groupObjectClass: 'groupOfNames',
  groupIdAttribute: 'cn',
  groupMemberAttribute: 'member',
  groupMemberFormat: MEMBER_FORMAT,
};

describe('the LDAP form model', () => {
  it('turns a stored server into form values and back without losing settings', () => {
    const values = fromServer(SERVER);
    expect(values.protocol).toBe('starttls');
    expect(values.port).toBe('389');
    expect(values.authPassword).toBe('');
    const request = toRequest(values, SERVER);
    expect(request).toMatchObject({
      id: 'id-1',
      order: 2,
      protocol: 'LDAP',
      useStartTls: true,
      port: 389,
      authPassword: null,
      authRealm: null,
      groupType: 'STATIC',
      groupObjectClass: 'groupOfNames',
      groupMemberFormat: MEMBER_FORMAT,
      userMemberOfAttribute: null,
    });
  });

  it('drops settings that the chosen options do not use', () => {
    const request = toRequest({
      ...EMPTY_LDAP,
      name: ' corp ',
      protocol: 'ldap',
      useTrustStore: true,
      authScheme: 'NONE',
      authUsername: 'cn=svc',
      authPassword: 'secret',
      ldapGroupsAsRoles: false,
    });
    expect(request).toMatchObject({
      name: 'corp',
      useTrustStore: false,
      authUsername: null,
      authPassword: null,
      groupType: null,
      userMemberOfAttribute: null,
    });
    expect(request).not.toHaveProperty('id');
  });

  it('checks the connection step', () => {
    const errors = validateConnection(
      t,
      { ...EMPTY_LDAP, name: 'Corp', host: 'bad host', port: '70000', connectionTimeoutSeconds: '0' },
      { names: ['corp'], passwordStored: false },
    );
    expect(errors).toEqual({
      name: 'identity.ldap.nameTaken',
      host: 'identity.ldap.hostInvalid',
      port: 'identity.ldap.portRange',
      searchBase: 'identity.ldap.searchBaseRequired',
      authUsername: 'identity.ldap.usernameRequired',
      authPassword: 'identity.ldap.passwordRequired',
      connectionTimeoutSeconds: 'identity.ldap.timeoutRange',
    });
    const fine = validateConnection(
      t,
      { ...fromServer(SERVER), name: 'corp' },
      { names: ['backup'], passwordStored: true },
    );
    expect(fine).toEqual({});
  });

  it('checks the mapping step for the chosen group type', () => {
    expect(validateMapping(t, { ...EMPTY_LDAP, userObjectClass: '', userMemberOfAttribute: '' })).toEqual({
      userObjectClass: 'identity.ldap.objectClassRequired',
      userMemberOfAttribute: 'identity.ldap.memberOfRequired',
    });
    expect(validateMapping(t, { ...EMPTY_LDAP, groupType: 'STATIC', groupMemberFormat: ' ' })).toEqual({
      groupMemberFormat: 'identity.ldap.groupMemberFormatRequired',
    });
    expect(
      validateMapping(t, { ...EMPTY_LDAP, ldapGroupsAsRoles: false, userMemberOfAttribute: '' }),
    ).toEqual({});
  });

  it('accepts host names and addresses only', () => {
    expect(validHost('ldap.example.com')).toBe(true);
    expect(validHost('10.0.0.1')).toBe(true);
    expect(validHost('::1')).toBe(true);
    expect(validHost('-bad.example.com')).toBe(false);
    expect(validHost('two words')).toBe(false);
  });

  it('fills the mapping from a template', () => {
    const template: LdapTemplate = {
      name: 'Active Directory',
      userBaseDn: 'cn=users',
      userSubtree: false,
      userObjectClass: 'user',
      userLdapFilter: null,
      userIdAttribute: 'sAMAccountName',
      userRealNameAttribute: 'cn',
      userEmailAddressAttribute: 'mail',
      userPasswordAttribute: null,
      ldapGroupsAsRoles: true,
      groupType: 'dynamic',
      groupBaseDn: null,
      groupSubtree: false,
      groupObjectClass: null,
      groupIdAttribute: null,
      groupMemberAttribute: null,
      groupMemberFormat: null,
      userMemberOfAttribute: 'memberOf',
    };
    const values = applyTemplate({ ...EMPTY_LDAP, name: 'corp', userLdapFilter: '(x=y)' }, template);
    expect(values).toMatchObject({
      name: 'corp',
      userBaseDn: 'cn=users',
      userObjectClass: 'user',
      userIdAttribute: 'sAMAccountName',
      userLdapFilter: '',
      groupType: 'DYNAMIC',
      groupObjectClass: EMPTY_LDAP.groupObjectClass,
    });
  });

  it('knows when the stored bind password still applies', () => {
    const values = fromServer(SERVER);
    expect(sameBindTarget(values, SERVER)).toBe(true);
    expect(sameBindTarget({ ...values, host: 'LDAP.example.com' }, SERVER)).toBe(true);
    expect(sameBindTarget({ ...values, protocol: 'ldap' }, SERVER)).toBe(false);
    expect(sameBindTarget({ ...values, authUsername: 'cn=other' }, SERVER)).toBe(false);
  });

  it('filters, moves and describes servers', () => {
    expect(serverUrl(SERVER)).toBe('ldap://ldap.example.com:389');
    expect(matches(SERVER, 'EXAMPLE.com')).toBe(true);
    expect(matches(SERVER, 'nope')).toBe(false);
    expect(move(['a', 'b', 'c'], 2, 0)).toEqual(['c', 'a', 'b']);
    expect(move(['a', 'b'], 0, 5)).toEqual(['a', 'b']);
    expect(base64('пароль')).toBe(btoa(unescape(encodeURIComponent('пароль'))));
  });
});
