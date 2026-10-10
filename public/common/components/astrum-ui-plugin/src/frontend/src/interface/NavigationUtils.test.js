
import { isVisible } from './NavigationUtils';

describe('NavigationUtils', () => {
  let mockAS;

  beforeEach(() => {
    // Mock the global AS object
    mockAS = {
      app: {
        Application: {
          bundleActive: jest.fn(() => true),
        },
      },
      State: {
        getValue: jest.fn((key, defaultValue) => {
          if (key === 'browseableformats') {
            return ['maven2', 'npm', 'docker'];
          }
          return defaultValue;
        }),
        getEdition: jest.fn(() => 'PRO'),
      },
      Permissions: {
        permissions: {},
        check: jest.fn((permission) => {
          return mockAS.Permissions.permissions[permission] === true;
        }),
      },
      Security: {
        hasUser: jest.fn(() => true),
      },
      // hasValidDependencies() in NavigationUtils requires AS.getApplication
      // to be a function returning a truthy value, otherwise the dependency
      // check short-circuits and isVisible() falls through to the cookie path.
      getApplication: jest.fn(() => ({})),
    };
    global.AS = mockAS;
    global.window = global.window || {};
    global.window.AS = mockAS;
  });

  afterEach(() => {
    delete global.AS;
    if (global.window) {
      delete global.window.AS;
    }
    jest.clearAllMocks();
  });

  describe('permissionPrefix', () => {
    it('should return true when user has permission matching the prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
        'astrum:repository-view:npm:npm-proxy:read': true,
      };

      const result = isVisible({
        permissionPrefix: 'astrum:repository-view',
      });

      expect(result).toBe(true);
    });

    it('should return false when user has no permission matching the prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:selectors:read': true,
        'astrum:admin:read': true,
      };

      const result = isVisible({
        permissionPrefix: 'astrum:repository-view',
      });

      expect(result).toBe(false);
    });

    it('should return false when user has permission with similar but non-matching prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-admin:maven2:my-repo:browse': true,
      };

      const result = isVisible({
        permissionPrefix: 'astrum:repository-view',
      });

      expect(result).toBe(false);
    });
  });

  describe('permissionPrefixes (ASTRUM-50552 fix)', () => {
    it('should return true when user has permission matching the first prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(true);
    });

    it('should return true when user has permission matching the second prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-content-selector:my-selector:maven2:my-repo:browse': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(true);
    });

    it('should return true when user has permissions matching both prefixes', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
        'astrum:repository-content-selector:my-selector:maven2:another-repo:browse': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(true);
    });

    it('should return false when user has no permissions matching any prefix', () => {
      mockAS.Permissions.permissions = {
        'astrum:selectors:read': true,
        'astrum:admin:read': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(false);
    });

    it('should return false when permissionPrefixes is an empty array', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
      };

      const result = isVisible({
        permissionPrefixes: [],
      });

      expect(result).toBe(false);
    });

    it('should return true when user has content selector permission with browse action', () => {
      // Test case specifically for ASTRUM-50552: Content Selector privileges with browse action
      mockAS.Permissions.permissions = {
        'astrum:repository-content-selector:my-content-selector:maven2:maven-central:browse': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(true);
    });

    it('should return true when user has content selector permission with read action', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-content-selector:my-content-selector:npm:npm-proxy:read': true,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(true);
    });
  });

  describe('Browse route visibility requirements (ASTRUM-50552)', () => {
    it('should show Browse menu when user has repository-view permission', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
      };

      // Simulate the actual visibility requirements from browseRoutes.js
      const result = isVisible({
        anonymousAccessOrHasUser: true,
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
        statesEnabled: [
          {
            key: 'browseableformats',
            defaultValue: [],
          },
        ],
      });

      expect(result).toBe(true);
    });

    it('should show Browse menu when user has content-selector permission', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-content-selector:my-selector:maven2:my-repo:browse': true,
      };

      // Simulate the actual visibility requirements from browseRoutes.js
      const result = isVisible({
        anonymousAccessOrHasUser: true,
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
        statesEnabled: [
          {
            key: 'browseableformats',
            defaultValue: [],
          },
        ],
      });

      expect(result).toBe(true);
    });

    it('should NOT show Browse menu when user has no browse-related permissions', () => {
      mockAS.Permissions.permissions = {
        'astrum:selectors:read': true,
        'astrum:admin:read': true,
      };

      // Simulate the actual visibility requirements from browseRoutes.js
      const result = isVisible({
        anonymousAccessOrHasUser: true,
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
        statesEnabled: [
          {
            key: 'browseableformats',
            defaultValue: [],
          },
        ],
      });

      expect(result).toBe(false);
    });
  });

  describe('backward compatibility', () => {
    it('should work when both permissionPrefix and permissionPrefixes are absent', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': true,
      };

      const result = isVisible({
        // No permission checks
      });

      expect(result).toBe(true);
    });

    it('should still support single permissionPrefix for existing routes', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-admin:maven2:my-repo:read': true,
      };

      const result = isVisible({
        permissionPrefix: 'astrum:repository-admin',
      });

      expect(result).toBe(true);
    });
  });

  describe('edge cases', () => {
    it('should handle missing AS.Permissions gracefully', () => {
      delete global.AS.Permissions;

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view'],
      });

      expect(result).toBe(false);
    });

    it('should handle missing AS.Permissions.permissions gracefully', () => {
      delete global.AS.Permissions.permissions;

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view'],
      });

      expect(result).toBe(false);
    });

    it('should handle empty permissions object', () => {
      mockAS.Permissions.permissions = {};

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(false);
    });

    it('should handle permissions with false values', () => {
      mockAS.Permissions.permissions = {
        'astrum:repository-view:maven2:my-repo:browse': false,
        'astrum:repository-content-selector:selector:maven2:repo:browse': false,
      };

      const result = isVisible({
        permissionPrefixes: ['astrum:repository-view', 'astrum:repository-content-selector'],
      });

      expect(result).toBe(false);
    });
  });

  describe('requiresUser with anonymous permissions (ASTRUM-47114)', () => {
    // AS.Permissions.permissions is the flat map populated by AS.controller.Permissions from
    // rapture_Security.getPermissions. Anonymous subjects with an elevated role should be allowed
    // through requiresUser gates once that map is non-empty; empty map means no elevation and the
    // gate should still hide the route.
    it('should hide requiresUser route for anonymous subject with no permissions', () => {
      mockAS.Security.hasUser = jest.fn(() => false);
      mockAS.Permissions.permissions = {};

      const result = isVisible({
        requiresUser: true,
      });

      expect(result).toBe(false);
    });

    it('should show requiresUser route for anonymous subject with non-empty permissions', () => {
      mockAS.Security.hasUser = jest.fn(() => false);
      mockAS.Permissions.permissions = {
        'astrum:*': true,
        'astrum:settings:read': true,
      };

      const result = isVisible({
        requiresUser: true,
      });

      expect(result).toBe(true);
    });

    it('should show requiresUser route for a logged-in user (regression check)', () => {
      mockAS.Security.hasUser = jest.fn(() => true);
      mockAS.Permissions.permissions = {};

      const result = isVisible({
        requiresUser: true,
      });

      expect(result).toBe(true);
    });

    it('should hide route when anonymous has permissions but per-route permission check fails', () => {
      mockAS.Security.hasUser = jest.fn(() => false);
      mockAS.Permissions.permissions = {
        'astrum:browse:read': true,
        'astrum:settings:read': false,
      };

      const result = isVisible({
        requiresUser: true,
        permissions: ['astrum:settings:read'],
      });

      expect(result).toBe(false);
    });
  });
});
