

import { formatAuditEvent, formatTimestamp, formatTimestampFull } from '../auditEventFormatter';
import type { AuditEvent } from '../audit.types';

describe('auditEventFormatter', () => {
  describe('formatAuditEvent', () => {
    describe('Category Assignment', () => {
      it('should assign security category for security domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.category).toBe('security');
      });

      it('should assign repository category for repository domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'repository',
          type: 'updated',
          context: 'maven-central',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.category).toBe('repository');
      });

      it('should assign configuration category for configuration domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'tasks',
          type: 'started',
          context: 'Cleanup Task',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: '*TASK',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.category).toBe('configuration');
      });

      it('should assign protection category for protection domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'protection.config',
          type: 'updated',
          context: 'RHC Config',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.category).toBe('protection');
      });

      it('should default to configuration for unknown domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'unknown.domain',
          type: 'updated',
          context: 'Something',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.category).toBe('configuration');
      });
    });

    describe('Sign-in events', () => {
      it('files a sign-in checked by a realm under security, named after the user', () => {
        const event = {
          id: 530,
          domain: 'AstrumAuthenticatingRealm',
          type: 'login',
          context: null,
          timestamp: '2026-10-11T17:16:54.290Z',
          initiator: 'admin/127.0.0.1',
          nodeId: 'node-1',
          attributes: { principal: 'admin' },
        };

        const result = formatAuditEvent(event);

        expect(result.category).toBe('security');
        expect(result.entityName).toBe('admin');
        expect(result.summary).toBe("User 'admin' signed in");
        expect(result.summary).not.toContain('Unknown');
      });
    });

    describe('Downloads', () => {
      it('calls the update of the last download of an asset a download', () => {
        const event = {
          id: 7,
          domain: 'repository.asset',
          type: 'updated-downloaded',
          context: '/docs/README.txt',
          timestamp: '2026-10-11T16:55:00.000Z',
          initiator: 'admin/127.0.0.1',
          nodeId: 'node-1',
          attributes: { 'repository.name': 'raw-hosted' },
        };

        const result = formatAuditEvent(event);

        expect(result.eventLabel).toBe('Downloaded');
        expect(result.summary).toContain("'/docs/README.txt' downloaded by admin/127.0.0.1");
      });
    });

    describe('Asset and component updates', () => {
      it.each([
        ['updated-attribute', 'Attributes Updated', 'attributes updated'],
        ['updated-kind', 'Kind Changed', 'kind changed'],
        ['updated-uploaded', 'Uploaded', 'uploaded'],
      ])('names %s events in words', (type, label, action) => {
        const event = {
          id: 8,
          domain: 'repository.asset',
          type,
          context: '/uploaded/install.txt',
          timestamp: '2026-10-11T18:11:00.000Z',
          initiator: 'admin/127.0.0.1',
          nodeId: 'node-1',
          attributes: { 'repository.name': 'raw-hosted' },
        };

        const result = formatAuditEvent(event);

        expect(result.eventLabel).toBe(label);
        expect(result.summary).toContain(`'/uploaded/install.txt' ${action} by admin/127.0.0.1`);
      });
    });

    describe('Event Label Formatting', () => {
      it('should capitalize event type', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.eventLabel).toBe('Created');
      });

      it('should handle hyphenated event types', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'malware.removal',
          type: 'automatic-malware-removed',
          context: 'component',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'system',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.eventLabel).toBe('Automatic Malware Removed');
      });

      it('should handle underscore-separated event types', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'firewall.quarantine',
          type: 'quarantined_new_violation',
          context: 'component',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'system',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.eventLabel).toBe('Quarantined New Violation');
      });
    });

    describe('Entity Type Derivation', () => {
      it('should derive User for security.user domain', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.entityType).toBe('User');
      });

      it('should derive Repository for repository domain', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'repository',
          type: 'updated',
          context: 'maven-central',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.entityType).toBe('Repository');
      });

      it('should derive Task for tasks domain', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'tasks',
          type: 'started',
          context: 'Cleanup',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: '*TASK',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.entityType).toBe('Task');
      });

      it('should default to Event for unknown domains', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'unknown.domain',
          type: 'updated',
          context: 'Something',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.entityType).toBe('Event');
      });
    });

    describe('Entity Name Extraction', () => {
      it('should use context as entity name when available', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: { name: 'different-name' },
        };

        const result = formatAuditEvent(event);
        expect(result.entityName).toBe('testuser');
      });

      it('should fallback to attributes.name when context is empty', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: '',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: { name: 'attr-name' },
        };

        const result = formatAuditEvent(event);
        expect(result.entityName).toBe('attr-name');
      });

      it('should use Unknown when neither context nor name available', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: '',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.entityName).toBe('Unknown');
      });
    });

    describe('Summary Building', () => {
      it('should include initiator in summary when not system', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.summary).toContain('by admin');
      });

      it('should not include initiator in summary when null (system)', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: null,
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.summary).not.toContain('by');
      });

      it('should handle malware removal events specially', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'malware.removal',
          type: 'automatic-malware-removed',
          context: 'evil-package',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'system',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.summary).toContain('removed (malware)');
      });

      it('should handle quarantine events specially', () => {
        const event: AuditEvent = {
          id: 1,
          domain: 'firewall.quarantine',
          type: 'quarantined-new-violation',
          context: 'blocked-component',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: {},
        };

        const result = formatAuditEvent(event);
        expect(result.summary).toContain('Download blocked');
      });
    });

    describe('Preserves Original Event Data', () => {
      it('should include all original event fields', () => {
        const event: AuditEvent = {
          id: 123,
          domain: 'security.user',
          type: 'created',
          context: 'testuser',
          timestamp: '2026-03-12T10:00:00.000Z',
          initiator: 'admin',
          nodeId: 'node-1',
          attributes: { key: 'value' },
        };

        const result = formatAuditEvent(event);

        expect(result.id).toBe(123);
        expect(result.domain).toBe('security.user');
        expect(result.type).toBe('created');
        expect(result.context).toBe('testuser');
        expect(result.timestamp).toBe('2026-03-12T10:00:00.000Z');
        expect(result.initiator).toBe('admin');
        expect(result.nodeId).toBe('node-1');
        expect(result.attributes).toEqual({ key: 'value' });
      });
    });
  });

  describe('formatTimestamp', () => {
    it('should format ISO timestamp to readable format', () => {
      const result = formatTimestamp('2026-03-12T10:30:00.000Z');

      // Format depends on locale, but should contain the date components
      expect(result).toContain('Mar');
      expect(result).toContain('12');
      expect(result).toContain('2026');
    });

    it('should return Invalid Date for invalid timestamps', () => {
      const result = formatTimestamp('invalid-date');

      expect(result).toBe('Invalid Date');
    });
  });

  describe('formatTimestampFull', () => {
    it('should format ISO timestamp with seconds', () => {
      const result = formatTimestampFull('2026-03-12T10:30:45.000Z');

      expect(result).toContain('Mar');
      expect(result).toContain('12');
      expect(result).toContain('2026');
    });

    it('should return Invalid Date for invalid timestamps', () => {
      const result = formatTimestampFull('invalid-date');

      expect(result).toBe('Invalid Date');
    });
  });
});
