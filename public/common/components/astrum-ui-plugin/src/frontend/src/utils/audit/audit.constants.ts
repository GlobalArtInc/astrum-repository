

import type { AuditCategory } from './audit.types';
import { AUDIT_EVENT_STRINGS } from './auditStrings';

/**
 * Maps audit event domains to UI categories for filtering and display.
 */
export const DOMAIN_CATEGORY_MAP: Record<string, AuditCategory> = {
  // Security domains
  'security.user': 'security',
  'security.role': 'security',
  'security.privilege': 'security',
  'security.user-role-mapping': 'security',
  'security.anonymous': 'security',
  'security.realm': 'security',
  'security.ldap': 'security',
  'security.crowd': 'security',
  'security.sslcertificate': 'security',
  'security.secrets': 'security',
  'security.jwt': 'security',
  'SamlRealm': 'security',

  // Repository domains
  'repository': 'repository',
  'repository.component': 'repository',
  'repository.asset': 'repository',
  'repository.component.tag': 'repository',
  'blobstore': 'repository',

  // Configuration domains
  'tasks': 'configuration',
  'capability': 'configuration',
  'cleanupPolicy': 'configuration',
  'ContentSelector': 'configuration',
  'RoutingRule': 'configuration',
  'email': 'configuration',
  'httpclient': 'configuration',
  'logging': 'configuration',
  'script': 'configuration',
  'license': 'configuration',
  'freeze': 'configuration',
  'DataStore': 'configuration',
  'database-migration': 'configuration',
  'userToken': 'configuration',
  'userToken.admin': 'configuration',

  // Protection domains
  'protection.config': 'protection',
  'malware.removal': 'protection',
  'firewall.quarantine': 'protection',
};

/**
 * Category colors for badges and icons.
 */
export const CATEGORY_COLORS: Record<AuditCategory, string> = {
  security: 'blue',
  repository: 'purple',
  configuration: 'gray',
  protection: 'amber',
};

/**
 * User-friendly labels for categories.
 */
export const CATEGORY_LABELS: Record<AuditCategory, string> = {
  security: AUDIT_EVENT_STRINGS.CATEGORIES.security,
  repository: AUDIT_EVENT_STRINGS.CATEGORIES.repository,
  configuration: AUDIT_EVENT_STRINGS.CATEGORIES.configuration,
  protection: AUDIT_EVENT_STRINGS.CATEGORIES.protection,
};

/**
 * Common event types for filtering.
 */
export const COMMON_EVENT_TYPES = [
  'created',
  'updated',
  'deleted',
  'started',
  'finished',
  'failed',
  'login',
  'logout',
] as const;

/**
 * Common domain filters grouped by category.
 */
export const COMMON_DOMAINS = [
  // Security
  { value: 'security.user', label: AUDIT_EVENT_STRINGS.DOMAINS.USERS, category: 'security' as AuditCategory },
  { value: 'security.role', label: AUDIT_EVENT_STRINGS.DOMAINS.ROLES, category: 'security' as AuditCategory },
  { value: 'security.privilege', label: AUDIT_EVENT_STRINGS.DOMAINS.PRIVILEGES, category: 'security' as AuditCategory },
  { value: 'security.ldap', label: 'LDAP', category: 'security' as AuditCategory },
  { value: 'SamlRealm', label: 'SAML', category: 'security' as AuditCategory },
  // Repository
  { value: 'repository', label: AUDIT_EVENT_STRINGS.DOMAINS.REPOSITORIES, category: 'repository' as AuditCategory },
  { value: 'blobstore', label: AUDIT_EVENT_STRINGS.DOMAINS.BLOB_STORES, category: 'repository' as AuditCategory },
  { value: 'repository.component', label: AUDIT_EVENT_STRINGS.DOMAINS.COMPONENTS, category: 'repository' as AuditCategory },
  // Configuration
  { value: 'tasks', label: AUDIT_EVENT_STRINGS.DOMAINS.TASKS, category: 'configuration' as AuditCategory },
  { value: 'capability', label: AUDIT_EVENT_STRINGS.DOMAINS.CAPABILITIES, category: 'configuration' as AuditCategory },
  { value: 'cleanupPolicy', label: AUDIT_EVENT_STRINGS.DOMAINS.CLEANUP_POLICIES, category: 'configuration' as AuditCategory },
  // Protection
  { value: 'protection.config', label: AUDIT_EVENT_STRINGS.DOMAINS.PROTECTION_CONFIG, category: 'protection' as AuditCategory },
  { value: 'malware.removal', label: AUDIT_EVENT_STRINGS.DOMAINS.MALWARE_REMOVAL, category: 'protection' as AuditCategory },
  { value: 'firewall.quarantine', label: AUDIT_EVENT_STRINGS.DOMAINS.FIREWALL_QUARANTINE, category: 'protection' as AuditCategory },
] as const;
