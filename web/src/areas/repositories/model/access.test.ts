import { describe, expect, it } from 'vitest';
import type { UiRepository } from '../api';
import {
  ANONYMOUS_ROLE,
  accessPaths,
  canSetLevel,
  closeForAnonymous,
  levelOf,
  openForAnonymous,
  ownPrivileges,
  type PrivilegeRow,
  type RoleRow,
  repositoryPrivileges,
  rolesWith,
  type UserRow,
  userBody,
  withLevel,
} from './access';

const REPOSITORY = { name: 'npm-hosted', format: 'npm' } as UiRepository;

function view(repository: string, action: string, format = 'npm'): PrivilegeRow {
  return {
    name: `as-repository-view-${format}-${repository}-${action.toLowerCase()}`,
    type: 'repository-view',
    format,
    repository,
    actions: [action],
  };
}

const PRIVILEGES: PrivilegeRow[] = [
  ...['BROWSE', 'READ', 'EDIT', 'ADD', 'DELETE', 'ALL'].map((action) => view('npm-hosted', action)),
  view('*', 'READ'),
  view('*', 'READ', '*'),
  view('raw-files', 'READ', 'raw'),
];

const OWN = ownPrivileges(PRIVILEGES, REPOSITORY);
const ALL = new Set(repositoryPrivileges(PRIVILEGES, REPOSITORY, true).map((privilege) => privilege.name));
const n = (action: string) => `as-repository-view-npm-npm-hosted-${action}`;

describe('the privileges of a repository', () => {
  it('include the ones for its format and for every format', () => {
    expect([...ALL]).toContain('as-repository-view-npm-*-read');
    expect([...ALL]).toContain('as-repository-view-*-*-read');
    expect([...ALL]).not.toContain('as-repository-view-raw-raw-files-read');
  });

  it('are named by action when they belong to the repository alone', () => {
    expect([...OWN.keys()].sort()).toEqual(['add', 'browse', 'delete', 'edit', 'read']);
    expect(OWN.get('read')).toBe(n('read'));
  });

  it('give every level when all five exist', () => {
    expect(canSetLevel(OWN, 'full')).toBe(true);
    expect(canSetLevel(new Map([['read', n('read')]]), 'read')).toBe(false);
  });
});

describe('the access level of a role', () => {
  const role = (privileges: string[]): RoleRow => ({ id: 'r', privileges });

  it('is read, write or full when the privileges match exactly', () => {
    expect(levelOf(role([n('browse'), n('read')]), OWN)).toBe('read');
    expect(levelOf(role([n('browse'), n('read'), n('add'), n('edit')]), OWN)).toBe('write');
    expect(levelOf(role(['x', n('browse'), n('read'), n('add'), n('edit'), n('delete')]), OWN)).toBe('full');
  });

  it('is custom for a partial set and none without any', () => {
    expect(levelOf(role([n('read')]), OWN)).toBe('custom');
    expect(levelOf(role(['as-repository-view-npm-*-read']), OWN)).toBe('none');
  });

  it('changes only the privileges of the repository', () => {
    const before = role(['other', n('read'), n('delete')]);
    expect(withLevel(before, OWN, 'write')).toEqual(['other', n('browse'), n('read'), n('add'), n('edit')]);
    expect(withLevel(before, OWN, 'none')).toEqual(['other']);
  });
});

describe('roles with access', () => {
  it('include roles that hold a granting role', () => {
    const roles: RoleRow[] = [
      { id: 'readers', privileges: [n('read')] },
      { id: 'nested', privileges: [], roles: ['readers'] },
      { id: 'others', privileges: ['x'] },
    ];
    expect(rolesWith(roles, ALL)).toEqual(['readers', 'nested']);
  });

  it('tell direct, wider and nested access apart', () => {
    const roles: RoleRow[] = [
      { id: 'readers', privileges: [n('read'), 'as-repository-view-npm-*-read'] },
      { id: 'nested', privileges: [], roles: ['readers'] },
    ];
    expect(accessPaths(roles[0] as RoleRow, roles, OWN, ALL)).toEqual({
      direct: true,
      wide: true,
      nested: false,
    });
    expect(accessPaths(roles[1] as RoleRow, roles, OWN, ALL)).toEqual({
      direct: false,
      wide: false,
      nested: true,
    });
  });
});

describe('anonymous access', () => {
  const anonymous: UserRow = { userId: 'anonymous', roles: ['public-reader'] };

  it('is closed by editing the roles only the anonymous user holds', () => {
    const roles: RoleRow[] = [{ id: 'public-reader', privileges: [n('browse'), n('read')] }];
    const plan = closeForAnonymous(anonymous, [anonymous], roles, OWN, ALL);
    expect(plan).toEqual({ blocked: [], roles, sharedWith: [] });
  });

  it('names the users who share the role', () => {
    const roles: RoleRow[] = [{ id: 'public-reader', privileges: [n('read')] }];
    const users = [anonymous, { userId: 'jane', roles: ['public-reader'] }];
    expect(closeForAnonymous(anonymous, users, roles, OWN, ALL).sharedWith).toEqual(['jane']);
  });

  it('cannot be closed here through built-in or format-wide roles', () => {
    const roles: RoleRow[] = [
      { id: 'public-reader', privileges: ['as-repository-view-npm-*-read'] },
      { id: 'as-anonymous', readOnly: true, privileges: ['as-repository-view-*-*-read'] },
    ];
    const user = { userId: 'anonymous', roles: ['public-reader', 'as-anonymous'] };
    const plan = closeForAnonymous(user, [user], roles, OWN, ALL);
    expect(plan.blocked).toEqual(['public-reader', 'as-anonymous']);
    expect(plan.roles).toEqual([]);
  });

  it('is opened through a role the anonymous user holds alone', () => {
    const roles: RoleRow[] = [
      { id: 'shared', privileges: [] },
      { id: ANONYMOUS_ROLE, privileges: [] },
      { id: 'public-reader', privileges: [] },
    ];
    const user = { userId: 'anonymous', roles: ['shared', 'public-reader', ANONYMOUS_ROLE] };
    const users = [user, { userId: 'jane', roles: ['shared'] }];
    expect(openForAnonymous(user, users, roles)?.id).toBe(ANONYMOUS_ROLE);
    expect(openForAnonymous({ ...user, roles: ['shared'] }, users, roles)).toBeNull();
  });
});

describe('a user update', () => {
  it('keeps the details and changes the roles', () => {
    const user: UserRow = {
      userId: 'jane',
      firstName: 'Jane',
      lastName: null,
      emailAddress: 'jane@example.org',
      source: 'LDAP',
      status: 'active',
      roles: ['a'],
    };
    expect(userBody(user, ['a', 'b'])).toEqual({
      userId: 'jane',
      firstName: 'Jane',
      lastName: null,
      emailAddress: 'jane@example.org',
      source: 'LDAP',
      status: 'active',
      roles: ['a', 'b'],
    });
  });
});
