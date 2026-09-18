

import { interpret } from 'xstate';
import { waitFor } from 'xstate/lib/waitFor';
import { createRolesFormMachine } from '../rolesFormMachine';

// Mock the local API module that rolesFormMachine imports from
jest.mock('../../../../../../../interface/api', () => ({
  restClient: {
    get: jest.fn().mockResolvedValue([]),
  },
  ENDPOINTS: {
    PRIVILEGES: '/service/rest/v1/security/privileges',
    ROLES: '/service/rest/v1/security/roles',
    ROLE_SOURCES: '/service/rest/v1/security/roles/sources',
  },
}));

const { restClient } = jest.requireMock('../../../../../../../interface/api');

/**
 * Helper: start a machine and wait for it to reach the editing state
 */
async function startAndLoad(
  machine: ReturnType<typeof createRolesFormMachine>,
  loadData?: Record<string, unknown>
) {
  // Mock the API responses for the load service
  restClient.get.mockImplementation((url: string) => {
    if (url?.endsWith('/privileges')) {
      return Promise.resolve([
        { name: 'as-all' },
        { name: 'as-repository-view-*-*-browse' },
        { name: 'as-repository-view-*-*-read' },
      ]);
    }
    if (url?.endsWith('/sources')) {
      return Promise.resolve([
        { id: 'default', name: 'Default' },
        { id: 'LDAP', name: 'LDAP' },
      ]);
    }
    // Match /roles or /roles?source=default (excluding /roles/sources)
    if (url?.includes('/roles') && !url?.includes('/sources') && !url?.includes('/roles/')) {
      return Promise.resolve([
        { id: 'as-admin', name: 'as-admin', source: 'default', description: 'Admin role', readOnly: false, privileges: [], roles: [] },
        { id: 'as-anonymous', name: 'as-anonymous', source: 'default', description: 'Anonymous role', readOnly: false, privileges: [], roles: [] },
      ]);
    }
    return Promise.resolve(loadData || []);
  });

  const service = interpret(machine).start();

  // Wait for loading to complete
  await waitFor(service, (state) => state.matches('editing'));

  return service;
}

describe('rolesFormMachine', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('create mode', () => {
    it('starts in loading state then transitions to editing', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = interpret(machine).start();

      expect(service.getSnapshot().matches('loading')).toBe(true);

      await waitFor(service, (state) => state.matches('editing'));

      expect(service.getSnapshot().matches('editing')).toBe(true);
      service.stop();
    });

    it('initializes with empty form data in create mode', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.data.id).toBe('');
      expect(state.context.data.name).toBe('');
      expect(state.context.data.description).toBe('');
      expect(state.context.data.privileges).toEqual([]);
      expect(state.context.data.roles).toEqual([]);

      service.stop();
    });

    it('loads privilege and role references on start', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.allPrivileges).toHaveLength(3);
      expect(state.context.allPrivileges[0]).toEqual({ id: 'as-all', name: 'as-all' });
      expect(state.context.allRoles).toHaveLength(2);

      service.stop();
    });

    it('has null role reference in create mode', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      expect(service.getSnapshot().context.role).toBeNull();

      service.stop();
    });
  });

  describe('edit mode', () => {
    it('loads role data from preloaded role', async () => {
      const preloadedRole = {
        id: 'as-admin',
        version: '1',
        source: 'Default',
        name: 'as-admin',
        description: 'Administrator role',
        readOnly: false,
        privileges: ['as-all'],
        roles: [],
      };

      const machine = createRolesFormMachine('as-admin', preloadedRole);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.data.id).toBe('as-admin');
      expect(state.context.data.name).toBe('as-admin');
      expect(state.context.data.description).toBe('Administrator role');
      expect(state.context.data.privileges).toEqual(['as-all']);
      expect(state.context.role).toEqual(preloadedRole);

      service.stop();
    });

    it('seeds roleType from role.source for a Astrum role', async () => {
      const preloadedRole = {
        id: 'as-admin',
        version: '1',
        source: 'Default',
        name: 'as-admin',
        description: 'Administrator role',
        readOnly: false,
        privileges: ['as-all'],
        roles: [],
      };

      const machine = createRolesFormMachine('as-admin', preloadedRole);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.data.roleType).toBe('astrum');
      expect(state.context.data.externalSource).toBeUndefined();

      service.stop();
    });

    it('seeds roleType and externalSource from role.source for an external role', async () => {
      const preloadedRole = {
        id: 'ldap-devs',
        version: '1',
        source: 'LDAP',
        name: 'ldap-devs',
        description: 'LDAP-mapped role',
        readOnly: true,
        privileges: [],
        roles: [],
      };

      const machine = createRolesFormMachine('ldap-devs', preloadedRole);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.data.roleType).toBe('external');
      expect(state.context.data.externalSource).toBe('LDAP');

      service.stop();
    });

    // Guards the non-preloaded path: role fetched by ID via restClient → restToRole → seeding.
    // The mocked response is a RestRole shape (no `version` field), matching what
    // the REST API actually returns; restToRole normalises `source` and stamps version.
    it('seeds roleType/externalSource when an external role is fetched by id (non-preloaded path)', async () => {
      restClient.get.mockImplementation((url: string) => {
        if (url?.endsWith('/roles/ldap-fetched')) {
          return Promise.resolve({
            id: 'ldap-fetched',
            source: 'LDAP',
            name: 'ldap-fetched',
            description: 'Fetched external role',
            readOnly: true,
            privileges: [],
            roles: [],
          });
        }
        if (url?.endsWith('/privileges')) return Promise.resolve([]);
        if (url?.endsWith('/sources')) return Promise.resolve([]);
        if (url?.includes('/roles') && !url?.includes('/sources') && !url?.includes('/roles/')) return Promise.resolve([]);
        return Promise.resolve([]);
      });

      const machine = createRolesFormMachine('ldap-fetched');
      const service = interpret(machine).start();
      await waitFor(service, (state) => state.matches('editing'));

      const state = service.getSnapshot();
      expect(state.context.role?.source).toBe('LDAP');
      expect(state.context.data.roleType).toBe('external');
      expect(state.context.data.externalSource).toBe('LDAP');

      service.stop();
    });

    // Confirms restToRole normalisation runs on the findRole path: REST returns
    // lowercase 'default' → restToRole maps to 'Default' → seeding produces
    // roleType='astrum' and no externalSource.
    it("normalises source 'default' to 'Default' and seeds roleType='astrum' on the findRole path", async () => {
      restClient.get.mockImplementation((url: string) => {
        if (url?.endsWith('/roles/plain-fetched')) {
          return Promise.resolve({
            id: 'plain-fetched',
            source: 'default',  // lowercase, as returned by the REST API
            name: 'plain-fetched',
            description: 'Fetched Astrum role',
            readOnly: false,
            privileges: [],
            roles: [],
          });
        }
        if (url?.endsWith('/privileges')) return Promise.resolve([]);
        if (url?.endsWith('/sources')) return Promise.resolve([]);
        if (url?.includes('/roles') && !url?.includes('/sources') && !url?.includes('/roles/')) return Promise.resolve([]);
        return Promise.resolve([]);
      });

      const machine = createRolesFormMachine('plain-fetched');
      const service = interpret(machine).start();
      await waitFor(service, (state) => state.matches('editing'));

      const state = service.getSnapshot();
      expect(state.context.role?.source).toBe('Default');  // normalised by restToRole
      expect(state.context.data.roleType).toBe('astrum');
      expect(state.context.data.externalSource).toBeUndefined();

      service.stop();
    });

    it('filters out current role from available roles', async () => {
      const preloadedRole = {
        id: 'as-admin',
        version: '1',
        source: 'Default',
        name: 'as-admin',
        description: 'Admin role',
        readOnly: false,
        privileges: ['as-all'],
        roles: [],
      };

      const machine = createRolesFormMachine('as-admin', preloadedRole);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      // as-admin should be filtered out from available roles
      const roleIds = state.context.allRoles.map((r: { id: string }) => r.id);
      expect(roleIds).not.toContain('as-admin');
      expect(roleIds).toContain('as-anonymous');

      service.stop();
    });
  });

  describe('validation', () => {
    it('validates role ID is required', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      // Try to submit with empty fields
      service.send({ type: 'SUBMIT' } as any);

      const state = service.getSnapshot();
      expect(state.matches('editing')).toBe(true);
      expect(state.context.validationErrors.id).toBeTruthy();

      service.stop();
    });

    it('validates role ID format (alphanumeric, hyphens, underscores)', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'invalid id!' } as any);
      service.send({ type: 'SUBMIT' } as any);

      const state = service.getSnapshot();
      expect(state.context.validationErrors.id).toContain('letters, numbers');

      service.stop();
    });

    it('validates role name is required', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'test-role' } as any);
      service.send({ type: 'SUBMIT' } as any);

      const state = service.getSnapshot();
      expect(state.matches('editing')).toBe(true);
      expect(state.context.validationErrors.name).toBeTruthy();

      service.stop();
    });

    it('validates at least one privilege or role must be assigned (P0 - blocks submit)', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'test-role' } as any);
      service.send({ type: 'UPDATE', name: 'name', value: 'Test Role' } as any);
      service.send({ type: 'SUBMIT' } as any);

      const state = service.getSnapshot();
      expect(state.matches('editing')).toBe(true);
      expect(state.context.validationErrors.privileges).toBe('Select at least one privilege or contained role');

      service.stop();
    });

    it('shows error when role name duplicates an existing role', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'new-role' } as any);
      service.send({ type: 'UPDATE', name: 'name', value: 'as-admin' } as any);
      service.send({ type: 'UPDATE', name: 'privileges', value: ['as-all'] } as any);
      service.send({ type: 'SUBMIT' } as any);

      const state = service.getSnapshot();
      expect(state.context.validationErrors.name).toContain('already exists');

      service.stop();
    });

    it('allows same name when editing the role itself', async () => {
      const preloadedRole = {
        id: 'as-admin',
        version: '1',
        source: 'Default',
        name: 'as-admin',
        description: 'Admin role',
        readOnly: false,
        privileges: ['as-all'],
        roles: [],
      };

      const machine = createRolesFormMachine('as-admin', preloadedRole);
      const service = await startAndLoad(machine);

      const state = service.getSnapshot();
      expect(state.context.validationErrors.name).toBeFalsy();

      service.stop();
    });

    it('passes validation with privileges assigned', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'test-role' } as any);
      service.send({ type: 'UPDATE', name: 'name', value: 'Test Role' } as any);
      service.send({ type: 'UPDATE', name: 'privileges', value: ['as-all'] } as any);

      const state = service.getSnapshot();
      expect(state.context.validationErrors.id).toBeFalsy();
      expect(state.context.validationErrors.name).toBeFalsy();
      expect(state.context.validationErrors.privileges).toBeFalsy();

      service.stop();
    });

    it('passes validation with contained roles assigned (no privileges)', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'test-role' } as any);
      service.send({ type: 'UPDATE', name: 'name', value: 'Test Role' } as any);
      service.send({ type: 'UPDATE', name: 'roles', value: ['as-admin'] } as any);

      const state = service.getSnapshot();
      expect(state.context.validationErrors.privileges).toBeFalsy();

      service.stop();
    });
  });

  describe('field updates', () => {
    it('updates fields via UPDATE event', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'new-role' } as any);
      service.send({ type: 'UPDATE', name: 'name', value: 'New Role' } as any);
      service.send({ type: 'UPDATE', name: 'description', value: 'A new role' } as any);

      const state = service.getSnapshot();
      expect(state.context.data.id).toBe('new-role');
      expect(state.context.data.name).toBe('New Role');
      expect(state.context.data.description).toBe('A new role');

      service.stop();
    });

    it('tracks dirty state after field update', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      expect(service.getSnapshot().context.isPristine).toBe(true);

      service.send({ type: 'UPDATE', name: 'id', value: 'new-role' } as any);

      expect(service.getSnapshot().context.isPristine).toBe(false);

      service.stop();
    });

    it('resets to pristine after RESET event', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'id', value: 'new-role' } as any);
      expect(service.getSnapshot().context.isPristine).toBe(false);

      service.send({ type: 'RESET' } as any);
      expect(service.getSnapshot().context.isPristine).toBe(true);
      expect(service.getSnapshot().context.data.id).toBe('');

      service.stop();
    });

    it('updates array fields (privileges and roles)', async () => {
      const machine = createRolesFormMachine(undefined);
      const service = await startAndLoad(machine);

      service.send({ type: 'UPDATE', name: 'privileges', value: ['as-all', 'as-repository-view-*-*-browse'] } as any);
      service.send({ type: 'UPDATE', name: 'roles', value: ['as-anonymous'] } as any);

      const state = service.getSnapshot();
      expect(state.context.data.privileges).toEqual(['as-all', 'as-repository-view-*-*-browse']);
      expect(state.context.data.roles).toEqual(['as-anonymous']);

      service.stop();
    });
  });

  describe('error handling', () => {
    it('handles load errors gracefully for reference data', async () => {
      restClient.get.mockImplementation((url: string) => {
        if (url?.includes('privileges')) {
          return Promise.reject(new Error('Network error'));
        }
        if (url?.includes('roles')) {
          return Promise.resolve([]);
        }
        return Promise.resolve([]);
      });

      const machine = createRolesFormMachine(undefined);
      const service = interpret(machine).start();

      // Should still reach editing state (reference data failures are non-fatal)
      await waitFor(service, (state) => state.matches('editing'));

      const state = service.getSnapshot();
      expect(state.context.allPrivileges).toEqual([]);

      service.stop();
    });

    it('handles gracefully when role fetch fails', async () => {
      restClient.get.mockImplementation((url: string) => {
        if (url?.includes('roles/missing-role')) {
          return Promise.reject(new Error('Not found'));
        }
        return Promise.resolve([]);
      });

      const machine = createRolesFormMachine('missing-role');
      const service = interpret(machine).start();

      // Machine transitions to editing with role: null when fetch fails
      await waitFor(service, (state) => state.matches('editing'));

      // Verify role is null due to fetch failure
      expect(service.getSnapshot().context.role).toBeNull();

      service.stop();
    });
  });
});
