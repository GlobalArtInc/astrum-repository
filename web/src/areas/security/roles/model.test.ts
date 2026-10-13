import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import type { Role } from '../api';
import { EMPTY_ROLE, fromRole, roleOptions, toRoleRequest } from './model';
import { setupErrors } from './RolePage';
import { filterRoles } from './RolesPage';

const t = makeTranslate('en');
const role = (id: string, overrides: Partial<Role> = {}): Role => ({
  id,
  source: 'default',
  name: id,
  description: '',
  readOnly: false,
  privileges: [],
  roles: [],
  ...overrides,
});

describe('the role form', () => {
  it('maps a role to the form and back, trimmed', () => {
    const values = fromRole(role('dev', { name: 'Developers', privileges: ['a'], roles: ['readers'] }));
    expect(values).toEqual({
      type: 'astrum',
      source: 'default',
      id: 'dev',
      name: 'Developers',
      description: '',
      privileges: ['a'],
      roles: ['readers'],
    });
    expect(toRoleRequest({ ...values, name: ' Developers ', description: ' Team ' })).toEqual({
      source: 'default',
      id: 'dev',
      name: 'Developers',
      description: 'Team',
      privileges: ['a'],
      roles: ['readers'],
    });
  });

  it('checks the id and the name', () => {
    expect(setupErrors(t, EMPTY_ROLE)).toEqual({ id: 'Fill this in', name: 'Fill this in' });
    expect(setupErrors(t, { ...EMPTY_ROLE, id: 'a:b', name: 'x' })).toEqual({
      id: 'Must not contain a colon',
    });
    expect(setupErrors(t, { ...EMPTY_ROLE, id: ' a', name: 'x' })).toEqual({
      id: 'Must not start or end with a space',
    });
  });

  it('offers assignable roles plus the chosen ones, without the role itself', () => {
    const all = [
      role('a', { description: 'A role' }),
      role('b'),
      role('c', { name: 'Charlie' }),
      role('self'),
    ];
    const options = roleOptions([role('a'), role('self')], all, ['c'], 'self');
    expect(options.map((option) => option.value)).toEqual(['a', 'c']);
    expect(options[1]?.detail).toBe('Charlie');
  });

  it('filters the list by id, name and description', () => {
    const roles = [role('dev', { name: 'Developers' }), role('ops', { description: 'Operations team' })];
    expect(filterRoles(roles, {}, 'team').map((item) => item.id)).toEqual(['ops']);
    expect(filterRoles(roles, {}, 'DEVELOPERS').map((item) => item.id)).toEqual(['dev']);
    expect(filterRoles(roles, { management: ['system'] }, '')).toEqual([]);
    expect(filterRoles(roles, { management: ['custom'] }, '')).toHaveLength(2);
  });
});
