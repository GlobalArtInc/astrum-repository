

import type { AuditEvent, AuditEventDisplay, } from './audit.types';
import { DOMAIN_CATEGORY_MAP } from './audit.constants';
import { AUDIT_EVENT_STRINGS } from './auditStrings';
import { pageLanguage } from '../../interface/plural';

const SUMMARIES = AUDIT_EVENT_STRINGS.SUMMARIES;
const ENTITY_TYPES = AUDIT_EVENT_STRINGS.ENTITY_TYPES;
const ACTIONS = AUDIT_EVENT_STRINGS.ACTIONS;

/**
 * Event types of signing in and out. Their domain is the realm that checked the credentials, such as
 * AstrumAuthenticatingRealm or LdapRealm, and the user is the principal attribute.
 */
const SESSION_TYPES = new Set(['login', 'logout']);

function isSessionEvent(event: AuditEvent): boolean {
  return SESSION_TYPES.has(event.type) || event.domain.includes('login') || event.domain.includes('logout');
}

/**
 * Enhance audit event with computed display fields.
 */
export function formatAuditEvent(event: AuditEvent): AuditEventDisplay {
  const session = isSessionEvent(event);
  const category = DOMAIN_CATEGORY_MAP[event.domain] || (session ? 'security' : 'configuration');
  const entityType = deriveEntityType(event.domain);
  const principal = session ? (event.attributes?.principal as string | undefined) : undefined;
  const entityName = event.context || event.attributes?.name || principal || SUMMARIES.UNKNOWN;

  return {
    ...event,
    category,
    eventLabel: formatEventLabel(event.type),
    summary: buildSummary(event, entityType, entityName),
    entityType,
    entityName,
  };
}

/**
 * Format event type for display ("created" → "Created").
 */
function formatEventLabel(type: string): string {
  const label = (AUDIT_EVENT_STRINGS.EVENT_LABELS as Record<string, string>)[type];
  if (label) {
    return label;
  }
  return type
    .split(/[-_]/)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

/**
 * Build human-readable summary string.
 */
function buildSummary(event: AuditEvent, entityType: string, entityName: string): string {
  const action = formatAction(event.type);
  const initiator = event.initiator || 'system';

  // Handle special cases
  if (isSessionEvent(event)) {
    const sessionEvents: Record<string, string> = {
      login: AUDIT_EVENT_STRINGS.SESSION_EVENTS.LOGIN_EVENT,
      logout: AUDIT_EVENT_STRINGS.SESSION_EVENTS.LOGOUT_EVENT,
    };
    return SUMMARIES.USER_SESSION(entityName, sessionEvents[event.type] ?? event.type);
  }

  if (event.type === 'automatic-malware-removed') {
    const parts = entityName.split(':');
    if (parts.length >= 2) {
      const repo = parts[0];
      const asset = parts.slice(1).join(':');
      return SUMMARIES.MALWARE_REMOVED_IN_REPOSITORY(asset, repo);
    }
    return SUMMARIES.MALWARE_REMOVED(entityName);
  }

  if (event.type === 'quarantined-new-violation' || event.type === 'blocked-download-already-quarantined') {
    return initiator !== 'system'
      ? SUMMARIES.DOWNLOAD_BLOCKED_FOR_USER(entityName, initiator)
      : SUMMARIES.DOWNLOAD_BLOCKED(entityName);
  }

  // Default format
  return initiator !== 'system'
    ? SUMMARIES.EVENT_BY(entityType, entityName, action, initiator)
    : SUMMARIES.EVENT(entityType, entityName, action);
}

/**
 * Format event type as action verb.
 */
function formatAction(type: string): string {
  const actionMap: Record<string, string> = {
    created: ACTIONS.WAS_CREATED,
    updated: ACTIONS.WAS_UPDATED,
    deleted: ACTIONS.WAS_DELETED,
    started: ACTIONS.WAS_STARTED,
    stopped: ACTIONS.WAS_STOPPED,
    finished: ACTIONS.WAS_FINISHED,
    failed: ACTIONS.HAS_FAILED,
    login: ACTIONS.LOGGED_IN,
    logout: ACTIONS.LOGGED_OUT,
    scheduled: ACTIONS.WAS_SCHEDULED,
    canceled: ACTIONS.WAS_CANCELED,
    activated: ACTIONS.WAS_ACTIVATED,
    passivated: ACTIONS.WAS_DEACTIVATED,
    purged: ACTIONS.WAS_PURGED,
    downloaded: ACTIONS.WAS_DOWNLOADED,
    'updated-downloaded': ACTIONS.WAS_DOWNLOADED,
    uploaded: ACTIONS.WAS_UPLOADED,
    'updated-uploaded': ACTIONS.WAS_UPLOADED,
    'updated-attribute': ACTIONS.ATTRIBUTES_UPDATED,
    'updated-kind': ACTIONS.KIND_CHANGED,
    associated: ACTIONS.WAS_TAGGED,
    disassociated: ACTIONS.WAS_UNTAGGED,
    changed: ACTIONS.WAS_CHANGED,
    'user-session-invalidation': ACTIONS.USER_SESSION_INVALIDATION,
  };

  return actionMap[type] || type.replace(/-/g, ' ');
}

/**
 * Derive entity type from domain.
 */
function deriveEntityType(domain: string): string {
  const typeMap: Record<string, string> = {
    'security.user': ENTITY_TYPES.USER,
    'security.role': ENTITY_TYPES.ROLE,
    'security.privilege': ENTITY_TYPES.PRIVILEGE,
    'security.user-role-mapping': ENTITY_TYPES.USER_ROLE_MAPPING,
    'security.anonymous': ENTITY_TYPES.ANONYMOUS_ACCESS,
    'security.realm': ENTITY_TYPES.REALM_CONFIGURATION,
    'security.ldap': ENTITY_TYPES.LDAP_CONFIGURATION,
    'security.crowd': ENTITY_TYPES.CROWD_CONFIGURATION,
    'security.sslcertificate': ENTITY_TYPES.SSL_CERTIFICATE,
    'security.secrets': ENTITY_TYPES.ENCRYPTION_KEY,
    'security.jwt': ENTITY_TYPES.JWT_TOKEN,
    'SamlRealm': ENTITY_TYPES.SAML_CONFIGURATION,
    'repository': ENTITY_TYPES.REPOSITORY,
    'repository.component': ENTITY_TYPES.COMPONENT,
    'repository.asset': ENTITY_TYPES.ASSET,
    'repository.component.tag': ENTITY_TYPES.COMPONENT_TAG,
    'blobstore': ENTITY_TYPES.BLOB_STORE,
    'tasks': ENTITY_TYPES.TASK,
    'capability': ENTITY_TYPES.CAPABILITY,
    'cleanupPolicy': ENTITY_TYPES.CLEANUP_POLICY,
    'ContentSelector': ENTITY_TYPES.CONTENT_SELECTOR,
    'RoutingRule': ENTITY_TYPES.ROUTING_RULE,
    'email': ENTITY_TYPES.EMAIL_CONFIGURATION,
    'httpclient': ENTITY_TYPES.HTTP_CLIENT_CONFIGURATION,
    'logging': ENTITY_TYPES.LOGGING_CONFIGURATION,
    'script': ENTITY_TYPES.SCRIPT,
    'license': ENTITY_TYPES.LICENSE,
    'freeze': ENTITY_TYPES.DATABASE_FREEZE,
    'DataStore': ENTITY_TYPES.DATA_STORE,
    'database-migration': ENTITY_TYPES.DATABASE_MIGRATION,
    'userToken': ENTITY_TYPES.USER_TOKEN,
    'userToken.admin': ENTITY_TYPES.USER_TOKEN_ADMIN,
    'protection.config': ENTITY_TYPES.PROTECTION_CONFIGURATION,
    'malware.removal': ENTITY_TYPES.MALWARE_REMOVAL,
    'firewall.quarantine': ENTITY_TYPES.FIREWALL_QUARANTINE,
  };

  return typeMap[domain] || ENTITY_TYPES.EVENT;
}

/**
 * Format timestamp for display.
 */
export function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleString(pageLanguage(), {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

/**
 * Format timestamp for full display (with seconds).
 */
export function formatTimestampFull(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleString(pageLanguage(), {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return timestamp;
  }
}
