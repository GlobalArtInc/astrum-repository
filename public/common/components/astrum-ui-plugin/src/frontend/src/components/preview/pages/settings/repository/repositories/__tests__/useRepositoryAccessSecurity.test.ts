

import { renderHook, act, waitFor } from '@testing-library/react';
import { useRepositoryAccessSecurity } from '../useRepositoryAccessSecurity';

const mockRestClient = {
  get: jest.fn(),
};

jest.mock('../../../../../../../interface/api', () => ({
  ...jest.requireActual('../../../../../../../interface/api'),
  restClient: {
    get: (...args: unknown[]) => mockRestClient.get(...args),
  },
  parseApiError: jest.fn((err) => ({
    message: err?.message || 'An error occurred',
  })),
}));

const PRIVILEGES_URL = '/service/rest/v1/security/privileges';
const ROLES_URL = '/service/rest/v1/security/roles';
const USERS_URL = '/service/rest/v1/security/users';
const ANONYMOUS_URL = '/service/rest/v1/security/anonymous';

const REPO_NAME = 'my-repo';
const REPO_FORMAT = 'maven2';

const mockPrivilegeExplicit = {
  name: 'as-repository-view-maven2-my-repo-read',
  description: 'Read my-repo',
  type: 'repository-view',
  repository: REPO_NAME,
  format: REPO_FORMAT,
  actions: ['read'],
};

const mockPrivilegeNameSubstring = {
  name: 'custom-priv-my-repo-browse',
  type: 'repository-view',
  actions: ['browse'],
};

const mockPrivilegeNameSuffix = {
  name: 'legacy-priv-my-repo',
  type: 'repository-view',
  actions: ['read'],
};

const mockPrivilegeNameWildcardSubstring = {
  name: 'custom-priv-my-repo*',
  type: 'repository-view',
  actions: ['edit'],
};

const mockPrivilegeWildcardMatchingFormat = {
  name: 'as-repository-view-maven2-*-read',
  type: 'repository-view',
  repository: '*',
  format: REPO_FORMAT,
  actions: ['read'],
};

const mockPrivilegeWildcardOtherFormat = {
  name: 'as-repository-view-npm-*-read',
  type: 'repository-view',
  repository: '*',
  format: 'npm',
  actions: ['read'],
};

const mockPrivilegeUnrelated = {
  name: 'as-repository-view-maven2-other-read',
  type: 'repository-view',
  repository: 'other-repo',
  format: REPO_FORMAT,
  actions: ['read'],
};

const mockRoleWithMatchingPriv = {
  id: 'as-anonymous',
  name: 'as-anonymous',
  privileges: ['as-repository-view-maven2-my-repo-read'],
  roles: [],
};

const mockRoleWithUnmatchedPriv = {
  id: 'as-admin',
  name: 'as-admin',
  privileges: ['as-repository-view-maven2-other-read'],
  roles: [],
};

const mockUserWithMatchingRole = {
  userId: 'anonymous',
  roles: ['as-anonymous'],
};

const mockUserWithUnmatchedRole = {
  userId: 'admin',
  roles: ['as-admin'],
};

const mockAnonymousEnabled = {
  enabled: true,
  userId: 'anonymous',
  realmName: 'AstrumAuthorizingRealm',
};

interface MockResponses {
  privileges?: unknown[];
  roles?: unknown[];
  users?: unknown[];
  anonymous?: unknown;
}

function mockAllUrls({ privileges = [], roles = [], users = [], anonymous = null }: MockResponses = {}): void {
  mockRestClient.get.mockImplementation((url: string) => {
    if (url === PRIVILEGES_URL) return Promise.resolve(privileges);
    if (url === ROLES_URL) return Promise.resolve(roles);
    if (url === USERS_URL) return Promise.resolve(users);
    if (url === ANONYMOUS_URL) return Promise.resolve(anonymous);
    return Promise.reject(new Error(`Unexpected URL: ${url}`));
  });
}

describe('useRepositoryAccessSecurity', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('happy path', () => {
    it('shouldFetchAllFourEndpointsAndReturnFilteredResults', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeExplicit, mockPrivilegeUnrelated],
        roles: [mockRoleWithMatchingPriv, mockRoleWithUnmatchedPriv],
        users: [mockUserWithMatchingRole, mockUserWithUnmatchedRole],
        anonymous: mockAnonymousEnabled,
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(USERS_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(ANONYMOUS_URL);

      expect(result.current.privileges).toHaveLength(1);
      expect(result.current.privileges[0].name).toBe(mockPrivilegeExplicit.name);
      expect(result.current.roles).toHaveLength(1);
      expect(result.current.roles[0].id).toBe('as-anonymous');
      expect(result.current.users).toHaveLength(1);
      expect(result.current.users[0].userId).toBe('anonymous');
      expect(result.current.anonymousAccess).toEqual(mockAnonymousEnabled);
      expect(result.current.error).toBeNull();
    });
  });

  describe('privilege filtering', () => {
    it('shouldMatchPrivilegesByExplicitRepositoryProperty', async () => {
      mockAllUrls({ privileges: [mockPrivilegeExplicit, mockPrivilegeUnrelated] });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));
      const names = result.current.privileges.map((p) => p.name).sort();
      expect(names).toEqual([mockPrivilegeExplicit.name]);
    });

    it('shouldMatchPrivilegesByNameSubstringPattern', async () => {
      mockAllUrls({
        privileges: [
          mockPrivilegeNameSubstring,
          mockPrivilegeNameSuffix,
          mockPrivilegeNameWildcardSubstring,
          mockPrivilegeUnrelated,
        ],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));
      const names = result.current.privileges.map((p) => p.name).sort();
      expect(names).toEqual([
        mockPrivilegeNameWildcardSubstring.name,
        mockPrivilegeNameSubstring.name,
        mockPrivilegeNameSuffix.name,
      ].sort());
    });

    it('shouldMatchWildcardRepositoryPrivilegeWhenFormatMatches', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeWildcardMatchingFormat, mockPrivilegeWildcardOtherFormat],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));
      const names = result.current.privileges.map((p) => p.name).sort();
      expect(names).toEqual([mockPrivilegeWildcardMatchingFormat.name]);
    });

    it('shouldNotMatchWildcardRepositoryPrivilegeWhenFormatOmitted', async () => {
      mockAllUrls({ privileges: [mockPrivilegeWildcardMatchingFormat] });

      const { result } = renderHook(() => useRepositoryAccessSecurity(REPO_NAME));

      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.privileges).toEqual([]);
    });
  });

  describe('dependent-chain short-circuit', () => {
    it('shouldSkipRolesFetchWhenNoPrivilegesMatched', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeUnrelated],
        roles: [mockRoleWithMatchingPriv],
        users: [mockUserWithMatchingRole],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(USERS_URL);
      expect(result.current.roles).toEqual([]);
      expect(result.current.users).toEqual([]);
    });

    it('shouldSkipUsersFetchWhenNoRolesMatched', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeExplicit],
        roles: [mockRoleWithUnmatchedPriv],
        users: [mockUserWithMatchingRole],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(USERS_URL);
      expect(result.current.privileges).toHaveLength(1);
      expect(result.current.roles).toEqual([]);
      expect(result.current.users).toEqual([]);
    });
  });

  describe('permission-aware fetching', () => {
    it('shouldSkipPrivilegesRolesAndUsersFetchesWhenCanReadPrivilegesIsFalse', async () => {
      mockAllUrls({ anonymous: mockAnonymousEnabled });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { canReadPrivileges: false })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).not.toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(USERS_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(ANONYMOUS_URL);
      expect(result.current.privileges).toEqual([]);
      expect(result.current.anonymousAccess).toEqual(mockAnonymousEnabled);
    });

    it('shouldSkipRolesAndUsersFetchesWhenCanReadRolesIsFalse', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeExplicit],
        roles: [mockRoleWithMatchingPriv],
        users: [mockUserWithMatchingRole],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, {
          canReadRoles: false,
          repositoryFormat: REPO_FORMAT,
        })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(USERS_URL);
      expect(result.current.privileges).toHaveLength(1);
      expect(result.current.roles).toEqual([]);
      expect(result.current.users).toEqual([]);
    });

    it('shouldSkipUsersFetchWhenCanReadUsersIsFalse', async () => {
      mockAllUrls({
        privileges: [mockPrivilegeExplicit],
        roles: [mockRoleWithMatchingPriv],
        users: [mockUserWithMatchingRole],
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, {
          canReadUsers: false,
          repositoryFormat: REPO_FORMAT,
        })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).toHaveBeenCalledWith(PRIVILEGES_URL);
      expect(mockRestClient.get).toHaveBeenCalledWith(ROLES_URL);
      expect(mockRestClient.get).not.toHaveBeenCalledWith(USERS_URL);
      expect(result.current.roles).toHaveLength(1);
      expect(result.current.users).toEqual([]);
    });

    it('shouldSkipAnonymousFetchWhenCanReadAnonymousIsFalse', async () => {
      mockAllUrls({ anonymous: mockAnonymousEnabled });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { canReadAnonymous: false })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).not.toHaveBeenCalledWith(ANONYMOUS_URL);
      expect(result.current.anonymousAccess).toBeNull();
    });

    it('shouldMakeNoFetchesWhenAllFlagsAreFalse', async () => {
      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, {
          canReadPrivileges: false,
          canReadRoles: false,
          canReadUsers: false,
          canReadAnonymous: false,
        })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get).not.toHaveBeenCalled();
      expect(result.current.privileges).toEqual([]);
      expect(result.current.roles).toEqual([]);
      expect(result.current.users).toEqual([]);
      expect(result.current.anonymousAccess).toBeNull();
      expect(result.current.error).toBeNull();
    });
  });

  describe('error handling', () => {
    it('shouldExposeErrorWhenPrivilegesCallFails', async () => {
      mockRestClient.get.mockImplementation((url: string) => {
        if (url === PRIVILEGES_URL) return Promise.reject(new Error('Privileges blew up'));
        return Promise.resolve(null);
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.error).toBe('Privileges blew up');
      expect(result.current.privileges).toEqual([]);
      expect(result.current.roles).toEqual([]);
      expect(result.current.users).toEqual([]);
      expect(result.current.anonymousAccess).toBeNull();
    });

    it('shouldExposeErrorWhenRolesCallFails', async () => {
      mockRestClient.get.mockImplementation((url: string) => {
        if (url === PRIVILEGES_URL) return Promise.resolve([mockPrivilegeExplicit]);
        if (url === ROLES_URL) return Promise.reject(new Error('Roles down'));
        if (url === ANONYMOUS_URL) return Promise.resolve(mockAnonymousEnabled);
        return Promise.resolve(null);
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );

      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.error).toBe('Roles down');
    });

    it('shouldSilentlyDegradeAnonymousToNullWhenEndpointFails', async () => {
      const warnSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
      try {
        mockRestClient.get.mockImplementation((url: string) => {
          if (url === PRIVILEGES_URL) return Promise.resolve([mockPrivilegeExplicit]);
          if (url === ROLES_URL) return Promise.resolve([mockRoleWithMatchingPriv]);
          if (url === USERS_URL) return Promise.resolve([mockUserWithMatchingRole]);
          if (url === ANONYMOUS_URL) return Promise.reject(new Error('Anonymous down'));
          return Promise.resolve(null);
        });

        const { result } = renderHook(() =>
          useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
        );

        await waitFor(() => expect(result.current.loading).toBe(false));
        expect(result.current.error).toBeNull();
        expect(result.current.anonymousAccess).toBeNull();
        expect(result.current.privileges).toHaveLength(1);
        expect(result.current.roles).toHaveLength(1);
        expect(result.current.users).toHaveLength(1);
        expect(warnSpy).toHaveBeenCalled();
      } finally {
        warnSpy.mockRestore();
      }
    });
  });

  describe('lifecycle', () => {
    it('shouldExposeLoadingStateUntilPromisesResolve', async () => {
      let resolvePrivileges!: (v: unknown) => void;
      let resolveAnonymous!: (v: unknown) => void;
      mockRestClient.get.mockImplementation((url: string) => {
        if (url === PRIVILEGES_URL) return new Promise((r) => { resolvePrivileges = r; });
        if (url === ANONYMOUS_URL) return new Promise((r) => { resolveAnonymous = r; });
        return Promise.resolve(null);
      });

      const { result } = renderHook(() =>
        useRepositoryAccessSecurity(REPO_NAME, { repositoryFormat: REPO_FORMAT })
      );
      expect(result.current.loading).toBe(true);

      await act(async () => {
        resolvePrivileges([]);
      });
      expect(result.current.loading).toBe(true);

      await act(async () => {
        resolveAnonymous(mockAnonymousEnabled);
      });
      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.anonymousAccess).toEqual(mockAnonymousEnabled);
    });

    it('shouldSupportManualRefetch', async () => {
      mockAllUrls();

      const { result } = renderHook(() => useRepositoryAccessSecurity(REPO_NAME));
      await waitFor(() => expect(result.current.loading).toBe(false));

      const callsBefore = mockRestClient.get.mock.calls.length;

      await act(async () => {
        result.current.refetch();
      });
      await waitFor(() => expect(result.current.loading).toBe(false));

      expect(mockRestClient.get.mock.calls.length).toBeGreaterThan(callsBefore);
    });

    it('shouldReturnNullAnonymousWhenEndpointYieldsNull', async () => {
      mockAllUrls({ anonymous: null });

      const { result } = renderHook(() => useRepositoryAccessSecurity(REPO_NAME));
      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.anonymousAccess).toBeNull();
      expect(result.current.error).toBeNull();
    });

    it('shouldHandleNullPrivilegesResponseGracefully', async () => {
      mockRestClient.get.mockImplementation((url: string) => {
        if (url === PRIVILEGES_URL) return Promise.resolve(null);
        if (url === ANONYMOUS_URL) return Promise.resolve(null);
        return Promise.resolve(null);
      });

      const { result } = renderHook(() => useRepositoryAccessSecurity(REPO_NAME));
      await waitFor(() => expect(result.current.loading).toBe(false));
      expect(result.current.privileges).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });
});
