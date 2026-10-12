import { describe, expect, it } from 'vitest';
import type { Privilege, Role } from '../api';
import { effectivePrivileges } from './effective';

const role = (id: string, privileges: string[], roles: string[] = []): Role => ({
  id,
  source: 'default',
  name: id,
  description: '',
  readOnly: false,
  privileges,
  roles,
});

const privilege = (name: string): Privilege => ({ type: 'wildcard', name, description: '', readOnly: false });

const ROLES = new Map(
  [
    role('developers', ['deploy'], ['readers']),
    role('readers', ['read', 'browse']),
    role('ops', ['deploy', 'admin'], ['developers']),
    role('loop-a', ['a'], ['loop-b']),
    role('loop-b', ['b'], ['loop-a']),
  ].map((item) => [item.id, item]),
);
const PRIVILEGES = new Map(['deploy', 'read', 'browse', 'admin', 'a'].map((name) => [name, privilege(name)]));

describe('effective privileges', () => {
  it('collects privileges of contained roles with the chain they come through', () => {
    const result = effectivePrivileges(['developers'], ROLES, PRIVILEGES);
    expect(result.privileges.map((entry) => [entry.name, entry.paths])).toEqual([
      ['browse', [['developers', 'readers']]],
      ['deploy', [['developers']]],
      ['read', [['developers', 'readers']]],
    ]);
    expect(result.missingRoles).toEqual([]);
  });

  it('lists every way a privilege is reached, shortest first', () => {
    const result = effectivePrivileges(['ops', 'readers'], ROLES, PRIVILEGES);
    const deploy = result.privileges.find((entry) => entry.name === 'deploy');
    expect(deploy?.paths).toEqual([['ops'], ['ops', 'developers']]);
    const read = result.privileges.find((entry) => entry.name === 'read');
    expect(read?.paths).toEqual([['readers'], ['ops', 'developers', 'readers']]);
  });

  it('stops at cycles and reports missing roles and privileges', () => {
    const result = effectivePrivileges(['loop-a', 'gone'], ROLES, PRIVILEGES);
    expect(result.privileges.map((entry) => entry.name)).toEqual(['a', 'b']);
    expect(result.privileges.find((entry) => entry.name === 'b')?.privilege).toBeUndefined();
    expect(result.missingRoles).toEqual(['gone']);
  });
});
