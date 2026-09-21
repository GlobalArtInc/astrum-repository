

jest.mock('../../../../../../../../interface/ExtJS', () => {
  const mockExtJS = {
    checkPermission: jest.fn().mockReturnValue(false),
  };
  return {
    __esModule: true,
    default: mockExtJS,
    ExtJS: mockExtJS,
  };
});

import { ExtJS } from '../../../../../../../../interface/ExtJS';

import { canGrantAccess, canReadSecurityDirectory } from '../endpointPermissions';

const PERM_ROLES_READ = 'astrum:roles:read';
const PERM_USERS_READ = 'astrum:users:read';
const PERM_PRIVILEGES_READ = 'astrum:privileges:read';
const PERM_ROLES_UPDATE = 'astrum:roles:update';
const PERM_ROLES_CREATE = 'astrum:roles:create';
const PERM_USERS_UPDATE = 'astrum:users:update';

const mockCheckPermission = ExtJS.checkPermission as jest.MockedFunction<typeof ExtJS.checkPermission>;

function grantPermissions(granted: string[]): void {
  mockCheckPermission.mockImplementation((perm: string) => granted.includes(perm));
}

describe('endpointPermissions', () => {
  beforeEach(() => {
    mockCheckPermission.mockReset();
    mockCheckPermission.mockReturnValue(false);
  });

  // Both tabs load roles + privileges + users; any missing read causes a 403 and a red error banner
  // in the tab body. The read gate therefore requires ALL three reads (AND), not any one (OR).
  const ALL_READS = [PERM_ROLES_READ, PERM_USERS_READ, PERM_PRIVILEGES_READ];
  const ALL_GRANT_PERMS = [...ALL_READS, PERM_ROLES_UPDATE, PERM_USERS_UPDATE];

  describe('canReadSecurityDirectory', () => {
    it('returns true when all three security reads are granted', () => {
      grantPermissions(ALL_READS);
      expect(canReadSecurityDirectory()).toBe(true);
    });

    it('returns false when only astrum:roles:read is granted', () => {
      grantPermissions([PERM_ROLES_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when only astrum:users:read is granted', () => {
      grantPermissions([PERM_USERS_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when only astrum:privileges:read is granted', () => {
      grantPermissions([PERM_PRIVILEGES_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when roles:read and users:read are granted but privileges:read is missing', () => {
      grantPermissions([PERM_ROLES_READ, PERM_USERS_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when roles:read and privileges:read are granted but users:read is missing', () => {
      grantPermissions([PERM_ROLES_READ, PERM_PRIVILEGES_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when users:read and privileges:read are granted but roles:read is missing', () => {
      grantPermissions([PERM_USERS_READ, PERM_PRIVILEGES_READ]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when no security read permissions are granted', () => {
      grantPermissions([]);
      expect(canReadSecurityDirectory()).toBe(false);
    });

    it('returns false when only unrelated permissions are granted', () => {
      grantPermissions(['astrum:repository-admin:*:*:read', 'astrum:blobstores:read']);
      expect(canReadSecurityDirectory()).toBe(false);
    });
  });

  describe('canGrantAccess', () => {
    it('returns true when all three reads plus roles:update and users:update are granted', () => {
      grantPermissions(ALL_GRANT_PERMS);
      expect(canGrantAccess()).toBe(true);
    });

    it('returns true when roles:create substitutes for roles:update', () => {
      grantPermissions([...ALL_READS, PERM_ROLES_CREATE, PERM_USERS_UPDATE]);
      expect(canGrantAccess()).toBe(true);
    });

    it('returns false when astrum:roles:read is missing', () => {
      grantPermissions([PERM_USERS_READ, PERM_PRIVILEGES_READ, PERM_ROLES_UPDATE, PERM_USERS_UPDATE]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when astrum:users:read is missing', () => {
      grantPermissions([PERM_ROLES_READ, PERM_PRIVILEGES_READ, PERM_ROLES_UPDATE, PERM_USERS_UPDATE]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when astrum:privileges:read is missing', () => {
      grantPermissions([PERM_ROLES_READ, PERM_USERS_READ, PERM_ROLES_UPDATE, PERM_USERS_UPDATE]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when neither roles:update nor roles:create is granted', () => {
      grantPermissions([...ALL_READS, PERM_USERS_UPDATE]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when astrum:users:update is missing', () => {
      grantPermissions([...ALL_READS, PERM_ROLES_UPDATE]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when no permissions are granted', () => {
      grantPermissions([]);
      expect(canGrantAccess()).toBe(false);
    });

    it('returns false when only the read permissions are granted (satisfies canReadSecurityDirectory but not this)', () => {
      grantPermissions(ALL_READS);
      expect(canGrantAccess()).toBe(false);
    });
  });
});
