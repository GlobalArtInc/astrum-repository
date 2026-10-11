import { renderHook, waitFor } from '@testing-library/react';

import type { Privilege } from '../../../../security/privileges/types';
import type { Role } from '../../../../security/roles/types';
import type { User } from '../../../../security/users/types';
import type { MergedApiEndpoint } from '../../utils/mergeSwaggerPermissions';

const mockFetchRoles = jest.fn();
const mockFetchPrivileges = jest.fn();
const mockFetchSources = jest.fn();
const mockFetchUsers = jest.fn();

jest.mock('../../../../security/roles/useRolesApi', () => ({
  useRolesApi: () => ({ fetchRoles: mockFetchRoles, createRole: jest.fn() }),
}));

jest.mock('../../../../security/privileges/usePrivilegesApi', () => ({
  usePrivilegesApi: () => ({ fetchPrivileges: mockFetchPrivileges }),
}));

jest.mock('../../../../security/users/useUsersApi', () => ({
  useUsersApi: () => ({ fetchSources: mockFetchSources, fetchUsers: mockFetchUsers, updateUser: jest.fn() }),
}));

import { invalidateEndpointAccessCache } from '../../hooks/securityDirectoryCache';
import { useGrantWizard } from '../useGrantWizard';

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

describe('useGrantWizard', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    invalidateEndpointAccessCache();
    mockFetchRoles.mockResolvedValue(ROLES);
    mockFetchPrivileges.mockResolvedValue({ data: PRIVILEGES, total: PRIVILEGES.length });
    mockFetchSources.mockResolvedValue([{ id: 'default' }]);
    mockFetchUsers.mockResolvedValue(USERS);
  });

  it('exposes the directory loaded after the wizard opened', async () => {
    const { result } = renderHook(() => useGrantWizard(ROW, true));

    expect(result.current.session).toBeNull();

    await waitFor(() => expect(result.current.session).not.toBeNull());

    expect(result.current.session?.users.map((u) => u.userId)).toEqual(['admin']);
    expect(result.current.recommendedRoles.map((r) => r.id)).toEqual(['as-admin']);
    expect(result.current.loadingDir).toBe(false);
  });

  it('does not load the directory while the tab is inactive', () => {
    const { result } = renderHook(() => useGrantWizard(ROW, false));

    expect(result.current.session).toBeNull();
    expect(mockFetchRoles).not.toHaveBeenCalled();
  });
});
