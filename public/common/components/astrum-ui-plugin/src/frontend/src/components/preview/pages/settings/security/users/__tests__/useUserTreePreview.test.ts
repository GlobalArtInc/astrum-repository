

import { act, renderHook } from '@testing-library/react';
import { useUserTreePreview } from '../useUserTreePreview';
import { User } from '../types';

const mockCombinedRoleTree = jest.fn();

jest.mock('../../roles/useRoleTree', () => ({
  useCombinedRoleTree: (...args: unknown[]) => mockCombinedRoleTree(...args),
}));

const roleTreeReturn = (tree: unknown[] = []) => ({
  tree,
  loading: false,
  error: null,
  toggleExpand: jest.fn(),
  expandAll: jest.fn(),
  collapseAll: jest.fn(),
});

const localUser: User = {
  userId: 'jsmith',
  realm: 'default',
  source: 'default',
  firstName: 'John',
  lastName: 'Smith',
  emailAddress: 'js@example.com',
  status: 'active',
  roles: ['as-admin'],
};

const externalUser: User = {
  userId: 'ldap-user',
  realm: 'LDAP',
  source: 'LDAP',
  firstName: 'Ldap',
  lastName: 'User',
  emailAddress: 'ldap@example.com',
  status: 'active',
  roles: [],
};

describe('useUserTreePreview', () => {
  beforeEach(() => {
    mockCombinedRoleTree.mockReset();
  });

  it('returns an empty tree when the combined role tree is empty', () => {
    mockCombinedRoleTree.mockReturnValue(roleTreeReturn([]));
    const { result } = renderHook(() =>
      useUserTreePreview(['as-admin'], localUser),
    );
    expect(result.current.tree).toEqual([]);
  });

  it('wraps the combined role tree under a synthetic user root when user is provided', () => {
    const roleNode = {
      id: 'role:as-admin',
      entityId: 'as-admin',
      name: 'Admin',
      type: 'role',
      inherited: false,
      expanded: false,
      isVisible: true,
      children: [],
    };
    mockCombinedRoleTree.mockReturnValue(roleTreeReturn([roleNode]));
    const { result } = renderHook(() =>
      useUserTreePreview(['as-admin'], localUser),
    );
    expect(result.current.tree).toHaveLength(1);
    const root = result.current.tree[0];
    expect(root.id).toBe('user:jsmith:default');
    expect(root.name).toBe('User: jsmith (Local)');
    expect(root.children).toEqual([roleNode]);
  });

  it('labels the user root with the external source name for external users', () => {
    const roleNode = {
      id: 'role:x',
      entityId: 'x',
      name: 'X',
      type: 'role',
      inherited: false,
      expanded: false,
      isVisible: true,
      children: [],
    };
    mockCombinedRoleTree.mockReturnValue(roleTreeReturn([roleNode]));
    const { result } = renderHook(() =>
      useUserTreePreview(['x'], externalUser),
    );
    expect(result.current.tree[0].name).toBe('User: ldap-user (LDAP)');
    expect(result.current.tree[0].id).toBe('user:ldap-user:LDAP');
  });

  it('returns the bare combined role tree when no user is provided (create mode)', () => {
    const roleNode = {
      id: 'role:a',
      entityId: 'a',
      name: 'A',
      type: 'role',
      inherited: false,
      expanded: false,
      isVisible: true,
      children: [],
    };
    mockCombinedRoleTree.mockReturnValue(roleTreeReturn([roleNode]));
    const { result } = renderHook(() => useUserTreePreview(['a']));
    expect(result.current.tree).toEqual([roleNode]);
  });

  it('passes the current search term through to useCombinedRoleTree', () => {
    mockCombinedRoleTree.mockReturnValue(roleTreeReturn([]));
    const { result } = renderHook(() =>
      useUserTreePreview(['as-admin'], localUser),
    );
    act(() => {
      result.current.setSearchTerm('admin');
    });
    const lastCall =
      mockCombinedRoleTree.mock.calls[mockCombinedRoleTree.mock.calls.length - 1];
    expect(lastCall[0]).toEqual(['as-admin']);
    expect(lastCall[1]).toEqual({ searchTerm: 'admin' });
  });

  it('proxies loading and error from useCombinedRoleTree', () => {
    mockCombinedRoleTree.mockReturnValue({
      tree: [],
      loading: true,
      error: 'boom',
      toggleExpand: jest.fn(),
      expandAll: jest.fn(),
      collapseAll: jest.fn(),
    });
    const { result } = renderHook(() =>
      useUserTreePreview(['as-admin'], localUser),
    );
    expect(result.current.loading).toBe(true);
    expect(result.current.error).toBe('boom');
  });
});
