import { act, renderHook, waitFor } from '@testing-library/react';

import type { Privilege } from '../../../../security/privileges/types';
import type { Role } from '../../../../security/roles/types';
import type { User } from '../../../../security/users/types';
import type { MergedApiEndpoint } from '../../utils/mergeSwaggerPermissions';

const mockFetchRoles = jest.fn();
const mockFetchPrivileges = jest.fn();
const mockFetchSources = jest.fn();
const mockFetchUsers = jest.fn();

jest.mock('../../../../security/roles/useRolesApi', () => ({
  useRolesApi: () => ({ fetchRoles: mockFetchRoles }),
}));

jest.mock('../../../../security/privileges/usePrivilegesApi', () => ({
  usePrivilegesApi: () => ({ fetchPrivileges: mockFetchPrivileges }),
}));

jest.mock('../../../../security/users/useUsersApi', () => ({
  useUsersApi: () => ({ fetchSources: mockFetchSources, fetchUsers: mockFetchUsers }),
}));

import { invalidateEndpointAccessCache, useEndpointAccess } from '../useEndpointAccess';

const ROLES: Role[] = [
  {
    id: 'as-admin',
    version: '',
    source: 'default',
    name: 'as-admin',
    description: '',
    readOnly: true,
    privileges: ['as-all'],
    roles: [],
  },
  {
    id: 'qa-dev',
    version: '',
    source: 'default',
    name: 'qa-dev',
    description: '',
    readOnly: false,
    privileges: ['as-search-read'],
    roles: [],
  },
];

const PRIVILEGES: Privilege[] = [
  {
    id: 'as-all',
    version: '',
    name: 'as-all',
    description: '',
    type: 'wildcard',
    readOnly: true,
    properties: {},
    permission: 'astrum:*',
  },
  {
    id: 'as-search-read',
    version: '',
    name: 'as-search-read',
    description: '',
    type: 'application',
    readOnly: true,
    properties: {},
    permission: 'astrum:search:read',
  },
];

const USERS: User[] = [
  {
    userId: 'admin',
    realm: 'default',
    source: 'default',
    firstName: 'Administrator',
    lastName: 'User',
    emailAddress: 'admin@example.org',
    status: 'active',
    roles: ['as-admin'],
  },
];

const ROW: MergedApiEndpoint = {
  httpMethod: 'GET',
  swaggerPathKey: '/v1/capabilities',
  fullPath: '/service/rest/v1/capabilities',
  tag: 'Capabilities',
  permission: {
    httpMethod: 'GET',
    pathPattern: '/service/rest/v1/capabilities',
    permissions: [{ permission: 'astrum:capabilities:read', logical: 'AND' }],
    description: null,
    tag: null,
    authenticated: true,
  },
};

describe('useEndpointAccess', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    invalidateEndpointAccessCache();
    mockFetchRoles.mockResolvedValue(ROLES);
    mockFetchPrivileges.mockResolvedValue({ data: PRIVILEGES, total: PRIVILEGES.length });
    mockFetchSources.mockResolvedValue([{ id: 'default' }]);
    mockFetchUsers.mockResolvedValue(USERS);
  });

  it('lists the roles and users with access once the directory has loaded', async () => {
    const { result } = renderHook(() => useEndpointAccess(ROW, true));

    await waitFor(() => expect(result.current.qualifyingRoles).toHaveLength(1));

    expect(result.current.loading).toBe(false);
    expect(result.current.qualifyingRoles[0].role.id).toBe('as-admin');
    expect(result.current.qualifyingRoles[0].userCount).toBe(1);
    expect(result.current.usersWithAccess.map(({ user }) => user.userId)).toEqual(['admin']);
  });

  it('does not load the directory while the tab is inactive', () => {
    renderHook(() => useEndpointAccess(ROW, false));

    expect(mockFetchRoles).not.toHaveBeenCalled();
  });

  it('loads the directory again on refetch', async () => {
    const { result } = renderHook(() => useEndpointAccess(ROW, true));
    await waitFor(() => expect(result.current.qualifyingRoles).toHaveLength(1));

    mockFetchRoles.mockResolvedValue([
      ROLES[0],
      { ...ROLES[1], privileges: ['as-all'] },
    ]);
    await act(async () => {
      await result.current.refetch();
    });

    await waitFor(() => expect(result.current.qualifyingRoles).toHaveLength(2));
    expect(mockFetchRoles).toHaveBeenCalledTimes(2);
  });
});
