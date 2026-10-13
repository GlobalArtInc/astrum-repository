import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import type { User } from '../api';
import {
  deleteQuery,
  EMPTY_USER,
  fromUser,
  fullName,
  passwordProblems,
  sourceLabel,
  toCreateRequest,
  toUpdateRequest,
  validateUser,
} from './model';
import { filterUsers } from './UsersPage';

const t = makeTranslate('en');
const user = (overrides: Partial<User> = {}): User => ({
  userId: 'jane',
  firstName: 'Jane',
  lastName: 'Doe',
  emailAddress: 'jane@example.org',
  source: 'default',
  status: 'active',
  readOnly: false,
  roles: ['dev'],
  externalRoles: [],
  ...overrides,
});

describe('the user form', () => {
  it('creates a user with trimmed fields and the password as typed', () => {
    expect(
      toCreateRequest({
        ...EMPTY_USER,
        userId: ' jane ',
        firstName: ' Jane',
        lastName: 'Doe ',
        emailAddress: ' jane@example.org ',
        password: ' secret word ',
        confirm: ' secret word ',
        roles: ['dev'],
      }),
    ).toEqual({
      userId: 'jane',
      firstName: 'Jane',
      lastName: 'Doe',
      emailAddress: 'jane@example.org',
      password: ' secret word ',
      status: 'active',
      roles: ['dev'],
    });
  });

  it('updates a local user and only the roles of an external one', () => {
    const local = user();
    const values = { ...fromUser(local), firstName: 'Janet', status: 'locked' as const, roles: ['ops'] };
    expect(toUpdateRequest(values, local)).toEqual({
      userId: 'jane',
      source: 'default',
      firstName: 'Janet',
      lastName: 'Doe',
      emailAddress: 'jane@example.org',
      status: 'locked',
      roles: ['ops'],
    });
    const external = user({
      source: 'LDAP',
      firstName: null,
      lastName: null,
      emailAddress: null,
      readOnly: true,
    });
    expect(toUpdateRequest({ ...fromUser(external), firstName: 'x', status: 'disabled' }, external)).toEqual({
      userId: 'jane',
      source: 'LDAP',
      firstName: null,
      lastName: null,
      emailAddress: null,
      status: 'active',
      roles: ['dev'],
    });
  });

  it('asks for every field of a new user and a matching password', () => {
    expect(validateUser(t, EMPTY_USER, { creating: true, local: true })).toEqual({
      userId: 'Fill this in',
      password: 'Fill this in',
      firstName: 'Fill this in',
      lastName: 'Fill this in',
      emailAddress: 'Fill this in',
    });
    expect(passwordProblems(t, 'short', 'other')).toEqual({
      password: 'Use at least 8 characters',
      confirm: 'The passwords do not match',
    });
    expect(
      validateUser(t, { ...fromUser(user()), emailAddress: 'nope' }, { creating: false, local: true }),
    ).toEqual({ emailAddress: 'Enter an address such as jane@example.org' });
    expect(validateUser(t, { ...EMPTY_USER }, { creating: false, local: false })).toEqual({});
  });

  it('deletes external users through their realm', () => {
    const sources = [
      { id: 'default', name: 'AstrumAuthenticatingRealm' },
      { id: 'LDAP', name: 'LdapRealm' },
    ];
    expect(deleteQuery(user({ userId: 'a b' }), sources)).toBe('/service/rest/v1/security/users/a%20b');
    expect(deleteQuery(user({ source: 'LDAP' }), sources)).toBe(
      '/service/rest/v1/security/users/jane?realm=LdapRealm',
    );
  });

  it('names users and sources', () => {
    expect(fullName(user({ lastName: null }))).toBe('Jane');
    expect(sourceLabel(t, 'default')).toBe('Local');
    expect(sourceLabel(t, 'Crowd')).toBe('Crowd');
  });
});

describe('the user list', () => {
  const user = (userId: string, status: 'active' | 'disabled') => ({
    userId,
    firstName: null,
    lastName: null,
    emailAddress: null,
    source: 'default',
    status,
    readOnly: false,
    roles: [],
    externalRoles: [],
  });

  it('filters by status and by the user ID', () => {
    const users = [user('jane', 'active'), user('joe', 'disabled')];
    expect(filterUsers(users, { status: ['inactive'] }, '').map((item) => item.userId)).toEqual(['joe']);
    expect(filterUsers(users, {}, 'JA').map((item) => item.userId)).toEqual(['jane']);
    expect(filterUsers(users, { source: ['LDAP'] }, '')).toEqual([]);
  });
});
