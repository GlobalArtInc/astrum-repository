

import type { ApiEndpointPermissionDto } from '../../types';
import { computeEndpointAccess } from '../endpointAccess';

function perm(p: Partial<ApiEndpointPermissionDto>): ApiEndpointPermissionDto {
  return {
    httpMethod: 'GET',
    pathPattern: '/x',
    permissions: [],
    description: null,
    tag: null,
    authenticated: false,
    ...p,
  };
}

describe('computeEndpointAccess', () => {
  it('public unauthenticated without permissions is granted', () => {
    expect(computeEndpointAccess(perm({ authenticated: false, permissions: [] }), {})).toBe('granted');
  });

  it('AND all required', () => {
    const meta = perm({
      authenticated: true,
      permissions: [
        { permission: 'astrum:a:read', logical: 'AND' },
        { permission: 'astrum:b:read', logical: 'AND' },
      ],
    });
    expect(
      computeEndpointAccess(meta, { 'astrum:a:read': true, 'astrum:b:read': true })
    ).toBe('granted');
    expect(computeEndpointAccess(meta, { 'astrum:a:read': true })).toBe('partial');
    expect(computeEndpointAccess(meta, {})).toBe('denied');
  });

  it('OR any', () => {
    const meta = perm({
      authenticated: true,
      permissions: [
        { permission: 'astrum:a:read', logical: 'OR' },
        { permission: 'astrum:b:read', logical: 'OR' },
      ],
    });
    expect(computeEndpointAccess(meta, { 'astrum:b:read': true })).toBe('granted');
    expect(computeEndpointAccess(meta, {})).toBe('denied');
  });
});
