
import { convertActionsToObject, convertActionsToArray, normalizeWildCardFormat, TYPES } from './PrivilegesHelper';

describe('PrivilegesHelper', () => {
  const mockTypes = {
    [TYPES.REPOSITORY_CONTENT_SELECTOR]: {
      formFields: [
        {
          id: 'actions',
          attributes: {
            options: ['browse', 'read', 'edit', 'add', 'delete']
          }
        }
      ]
    },
    [TYPES.SCRIPT]: {
      formFields: [
        {
          id: 'actions',
          attributes: {
            options: ['browse', 'read', 'edit', 'add', 'delete', 'run']
          }
        }
      ]
    },
    [TYPES.APPLICATION]: {
      formFields: [
        {
          id: 'actions',
          attributes: {
            options: ['read', 'delete', 'create', 'update']
          }
        }
      ]
    }
  };

  describe('convertActionsToObject', () => {
    it('should expand ALL action to individual actions for repository-content-selector', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['ALL']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        browse: true,
        read: true,
        edit: true,
        add: true,
        delete: true
      });
    });

    it('should expand * action to individual actions', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['*']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        browse: true,
        read: true,
        edit: true,
        add: true,
        delete: true
      });
    });

    it('should expand lowercase "all" action to individual actions', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['all']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        browse: true,
        read: true,
        edit: true,
        add: true,
        delete: true
      });
    });

    it('should expand ALL action for script privilege type', () => {
      const input = {
        type: TYPES.SCRIPT,
        name: 'test',
        actions: ['ALL']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        browse: true,
        read: true,
        edit: true,
        add: true,
        delete: true,
        run: true
      });
    });

    it('should handle individual actions normally', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['READ', 'BROWSE']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        read: true,
        browse: true
      });
    });

    it('should handle application privilege type with create/update mapping', () => {
      const input = {
        type: TYPES.APPLICATION,
        name: 'test',
        actions: ['ADD', 'EDIT', 'READ']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        create: true,
        update: true,
        read: true
      });
    });

    it('should expand ALL for application privilege type', () => {
      const input = {
        type: TYPES.APPLICATION,
        name: 'test',
        actions: ['ALL']
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({
        read: true,
        delete: true,
        create: true,
        update: true
      });
    });

    it('should handle empty actions array', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: []
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result.actions).toEqual({});
    });

    it('should handle missing types definition gracefully', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['ALL']
      };

      const result = convertActionsToObject(input, {});

      // Should fall through to original logic without expansion
      expect(result.actions).toEqual({
        all: true
      });
    });

    it('should not modify data without actions field', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        description: 'test description'
      };

      const result = convertActionsToObject(input, mockTypes);

      expect(result).toEqual(input);
    });
  });

  describe('convertActionsToArray', () => {
    it('should collapse all selected actions to ALL', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: {
          browse: true,
          read: true,
          edit: true,
          add: true,
          delete: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toEqual(['ALL']);
    });

    it('should collapse all selected actions to ALL for script type', () => {
      const input = {
        type: TYPES.SCRIPT,
        name: 'test',
        actions: {
          browse: true,
          read: true,
          edit: true,
          add: true,
          delete: true,
          run: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toEqual(['ALL']);
    });

    it('should keep individual actions if not all are selected', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: {
          browse: true,
          read: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toHaveLength(2);
      expect(result.actions).toContain('browse');
      expect(result.actions).toContain('read');
    });

    it('should handle application privilege type with create/update mapping', () => {
      const input = {
        type: TYPES.APPLICATION,
        name: 'test',
        actions: {
          create: true,
          read: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toHaveLength(2);
      expect(result.actions).toContain('ADD');
      expect(result.actions).toContain('read');
    });

    it('should collapse all application actions to ALL', () => {
      const input = {
        type: TYPES.APPLICATION,
        name: 'test',
        actions: {
          read: true,
          delete: true,
          create: true,
          update: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toEqual(['ALL']);
    });

    it('should handle empty actions object', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: {}
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toEqual([]);
    });

    it('should handle actions with false values', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: {
          browse: true,
          read: false,
          edit: true
        }
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result.actions).toHaveLength(2);
      expect(result.actions).toContain('browse');
      expect(result.actions).toContain('edit');
      expect(result.actions).not.toContain('read');
    });

    it('should handle missing types definition gracefully', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: {
          browse: true,
          read: true
        }
      };

      const result = convertActionsToArray(input, {});

      expect(result.actions).toHaveLength(2);
      expect(result.actions).toContain('browse');
      expect(result.actions).toContain('read');
    });

    it('should not modify data without actions field', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        description: 'test description'
      };

      const result = convertActionsToArray(input, mockTypes);

      expect(result).toEqual(input);
    });
  });

  describe('round-trip conversion', () => {
    it('should maintain consistency when converting to object and back with ALL', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['ALL']
      };

      const asObject = convertActionsToObject(input, mockTypes);
      const backToArray = convertActionsToArray(asObject, mockTypes);

      expect(backToArray.actions).toEqual(['ALL']);
    });

    it('should maintain consistency when converting individual actions', () => {
      const input = {
        type: TYPES.REPOSITORY_CONTENT_SELECTOR,
        name: 'test',
        actions: ['READ', 'BROWSE']
      };

      const asObject = convertActionsToObject(input, mockTypes);
      const backToArray = convertActionsToArray(asObject, mockTypes);

      expect(backToArray.actions).toHaveLength(2);
      expect(backToArray.actions).toContain('read');
      expect(backToArray.actions).toContain('browse');
    });
  });

  describe('normalizeWildCardFormat', () => {
    describe('when type is REPOSITORY_CONTENT_SELECTOR', () => {
      it('should split repository and format when repository is wildcard with format', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: '*-maven2',
          name: 'test-privilege',
        };

        const result = normalizeWildCardFormat(data);

        expect(result.format).toBe('maven2');
        expect(result.repository).toBe('*');
        expect(result.name).toBe('test-privilege');
      });

      it('should not modify when repository is wildcard without format', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: '*',
          name: 'test-privilege',
        };

        const result = normalizeWildCardFormat(data);

        expect(result.format).toBeUndefined();
        expect(result.repository).toBe('*');
        expect(result.name).toBe('test-privilege');
      });

      it('should not modify when repository is not wildcard', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: 'repo-maven2',
          name: 'test-privilege',
        };

        const result = normalizeWildCardFormat(data);

        expect(result.format).toBeUndefined();
        expect(result.repository).toBe('repo-maven2');
        expect(result.name).toBe('test-privilege');
      });

      it('should handle repository with multiple hyphens correctly', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: '*-maven2-snapshot',
          name: 'test-privilege',
        };

        const result = normalizeWildCardFormat(data);

        expect(result.format).toBe('maven2');
        expect(result.repository).toBe('*');
      });
    });

    describe('data immutability', () => {
      it('should not mutate the original data object', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: '*-maven2',
          name: 'test-privilege',
        };
        const originalData = { ...data };

        const result = normalizeWildCardFormat(data);

        expect(data).toEqual(originalData);
        expect(result).not.toBe(data);
        expect(result.format).toBe('maven2');
        expect(result.repository).toBe('*');
      });

      it('should preserve all other properties when modifying', () => {
        const data = {
          type: TYPES.REPOSITORY_CONTENT_SELECTOR,
          repository: '*-maven2',
          name: 'test-privilege',
          description: 'Test description',
          actions: ['READ', 'BROWSE'],
          contentSelector: 'test-selector',
        };

        const result = normalizeWildCardFormat(data);

        expect(result.name).toBe('test-privilege');
        expect(result.description).toBe('Test description');
        expect(result.actions).toEqual(['READ', 'BROWSE']);
        expect(result.contentSelector).toBe('test-selector');
        expect(result.format).toBe('maven2');
        expect(result.repository).toBe('*');
      });
    });
  });
});
