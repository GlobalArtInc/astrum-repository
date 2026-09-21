

import type { Privilege } from '../../../../security/privileges/types';
import { pickPrivilegeIdsForRequirements } from '../grantPrivilegePicker';

function priv(id: string, permission: string): Privilege {
  return {
    id,
    version: '1',
    name: id,
    description: '',
    type: 'wildcard',
    readOnly: false,
    properties: {},
    permission,
  };
}

describe('grantPrivilegePicker', () => {
  it('AND picks one privilege per requirement', () => {
    const privileges = [priv('p1', 'astrum:a:read'), priv('p2', 'astrum:b:read')];
    const { ids, missing } = pickPrivilegeIdsForRequirements(['astrum:a:read', 'astrum:b:read'], 'AND', privileges);
    expect(missing).toEqual([]);
    expect(ids.sort()).toEqual(['p1', 'p2'].sort());
  });

  it('OR picks a single covering privilege', () => {
    const privileges = [priv('wide', 'astrum:*')];
    const { ids, missing } = pickPrivilegeIdsForRequirements(['astrum:a:read', 'astrum:z:read'], 'OR', privileges);
    expect(missing).toEqual([]);
    expect(ids).toEqual(['wide']);
  });

  it('reports missing when no privilege matches', () => {
    const { ids, missing } = pickPrivilegeIdsForRequirements(['astrum:missing:read'], 'AND', []);
    expect(ids).toEqual([]);
    expect(missing).toContain('astrum:missing:read');
  });
});
