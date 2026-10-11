

/**
 * Settings Hub Configuration
 *
 * Defines all settings cards and their organization into sections.
 * This file is the single source of truth for the Settings Hub.
 *
 * Section order intentionally mirrors the classic ExtJS admin navigation:
 * Repository → Security → Support → System
 */

import UIStrings from '../../../../constants/UIStrings';
import SettingsStrings from '../../constants/pages/settings/SettingsStrings';

export interface VisibilityRequirements {
  permissions?: string[];
  requiresPermission?: string;
  requiresAnyPermission?: string[];
  permissionPrefix?: string;
  permissionPrefixes?: string[];
  editions?: string[];
  requiresUser?: boolean;
  /** Required active bundle/plugin (matches isVisible bundle check). */
  bundle?: string;
  /** All listed licenses must be present and valid. */
  licenseValid?: Array<{ key: string; defaultValue?: boolean }>;
  /** All listed AS.State keys must resolve truthy. */
  statesEnabled?: Array<{ key: string; defaultValue?: boolean }>;
  /** Required capability type must be created and active. */
  capability?: string;
}

export interface SettingCard {
  /** Unique identifier */
  id: string;
  /** URL path (appended to #preview/admin/) when fullHash is not set */
  path: string;
  /**
   * Full hash URL for routes outside #preview/admin/... (e.g. API lives under #preview/browse/api).
   */
  fullHash?: string;
  /** Display label */
  label: string;
  /** Description text */
  description: string;
  /** Optional additional search keywords for discoverability */
  searchTerms?: string[];
  /** If true, this card is hidden on Cloud deployments */
  cloudExcluded?: boolean;
  /** If true, this card is shown ONLY on Cloud deployments (hidden on self-hosted) */
  cloudOnly?: boolean;
  /** If true, this card is hidden on CE/OSS self-hosted (requires PRO or Cloud edition) */
  proOnly?: boolean;
  /** If true, this card is hidden from non-admin users */
  adminOnly?: boolean;
  /**
   * Feature flag key from previewFeatureFlags.ts.
   * When set, the card shows a "Coming Soon" badge if isFeatureEnabled(featureKey) is false.
   * Cards without a featureKey are always treated as available.
   */
  featureKey?: string;
  /**
   * Permission requirements for this card, matching the corresponding route's
   * visibilityRequirements. Cards without this field are always visible.
   * Uses the same structure as route data.visibilityRequirements so isVisible() can evaluate it.
   */
  visibilityRequirements?: VisibilityRequirements;
}

export interface SettingsSection {
  /** Section identifier */
  id: string;
  /** Section heading label */
  label: string;
  /** Cards in this section */
  cards: SettingCard[];
  /** If true, the entire section is hidden on Cloud deployments */
  cloudExcluded?: boolean;
}

/**
 * All settings organized by section.
 */
export const SETTINGS_SECTIONS: SettingsSection[] = [
  {
    id: 'repository',
    label: SettingsStrings.CARDS.REPOSITORY,
    cards: [
      {
        id: 'repositories',
        path: 'repository/repositories',
        label: SettingsStrings.CARDS.REPOSITORIES,
        description: SettingsStrings.CARDS.MANAGE_MAVEN_NPM_DOCKER_OTHER,
        searchTerms: ['maven', 'npm', 'docker', 'pypi', 'nuget'],
        featureKey: 'repository.repositories',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js REPOSITORIES.ROOT), which uses a
        // repository-admin prefix so a user with admin on any single repo can see the menu.
        visibilityRequirements: { permissionPrefix: 'astrum:repository-admin' },
      },
      {
        id: 'blob-stores',
        path: 'repository/blobstores',
        label: SettingsStrings.CARDS.BLOB_STORES,
        description: SettingsStrings.CARDS.CONFIGURE_LOCAL_CLOUD_BLOB_STORAGE,
        searchTerms: ['storage', 's3', 'azure', 'google cloud'],
        cloudExcluded: true,
        featureKey: 'repository.blobstores',
        visibilityRequirements: { requiresPermission: 'astrum:blobstores:read' },
      },
      {
        id: 'cleanup-policies',
        path: 'repository/cleanup-policies',
        label: SettingsStrings.CARDS.CLEANUP_POLICIES,
        description: SettingsStrings.CARDS.CONFIGURE_AUTOMATED_COMPONENT_CLEANUP,
        searchTerms: ['retention', 'delete', 'purge'],
        featureKey: 'repository.cleanuppolicies',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js CLEANUPPOLICIES.ROOT), which gates
        // on astrum:* — the RoutingRules/CleanupPolicy write endpoints require admin, and the menu
        // must not surface to a settings:read-only user who would then hit a 403.
        visibilityRequirements: { requiresPermission: 'astrum:*' },
      },
      {
        id: 'routing-rules',
        path: 'repository/routing-rules',
        label: SettingsStrings.CARDS.ROUTING_RULES,
        description: SettingsStrings.CARDS.CONTROL_COMPONENT_ACCESS_PATH_PATTERNS,
        searchTerms: ['block', 'allow', 'filter'],
        featureKey: 'repository.routingrules',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js ROUTINGRULES.ROOT). RoutingRulesResource
        // requires astrum:* on every endpoint, including the GET/list — so settings:read here would show
        // the menu but 403 on load.
        visibilityRequirements: { requiresPermission: 'astrum:*' },
      },
      {
        id: 'content-selectors',
        path: 'repository/selectors',
        label: SettingsStrings.CARDS.CONTENT_SELECTORS,
        description: SettingsStrings.CARDS.DEFINE_CONTENT_FILTERING_EXPRESSIONS,
        searchTerms: ['csel', 'filter', 'query'],
        featureKey: 'repository.selectors',
        visibilityRequirements: { requiresPermission: 'astrum:selectors:read' },
      },
      {
        id: 'data-store',
        path: 'repository/datastore',
        label: SettingsStrings.CARDS.DATA_STORE,
        description: SettingsStrings.CARDS.CONFIGURE_REPOSITORY_METADATA_DATABASE,
        searchTerms: ['database', 'postgresql', 'h2', 'jdbc'],
        cloudExcluded: true,
        featureKey: 'repository.datastore',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js DATASTORE.ROOT) — gates on astrum:*
        // (admin-only screen) AND requires the pro-datastore bundle, restricted to PRO/COMMUNITY.
        // Without the bundle+editions gates the tile shows on editions where the screen is absent.
        visibilityRequirements: {
          requiresPermission: 'astrum:*',
          bundle: 'astrum-pro-datastore-plugin',
          editions: ['PRO', 'COMMUNITY'],
        },
      },
      {
        id: 'proprietary',
        path: 'repository/proprietary',
        label: SettingsStrings.CARDS.PROPRIETARY,
        description: SettingsStrings.CARDS.MANAGE_PROPRIETARY_COMPONENT_SETTINGS,
        featureKey: 'repository.proprietary',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
    ],
  },
  {
    id: 'security',
    label: SettingsStrings.CARDS.SECURITY,
    cards: [
      {
        id: 'users',
        path: 'security/users',
        label: SettingsStrings.CARDS.USERS,
        description: SettingsStrings.CARDS.MANAGE_USER_ACCOUNTS_ACCESS,
        featureKey: 'security.users',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js USERS.ROOT), which requires BOTH
        // users:read and roles:read (permissions[] is AND).
        visibilityRequirements: { permissions: ['astrum:users:read', 'astrum:roles:read'] },
      },
      {
        id: 'roles',
        path: 'security/roles',
        label: SettingsStrings.CARDS.ROLES,
        description: SettingsStrings.CARDS.DEFINE_USER_PERMISSIONS_ACCESS_CONTROL,
        featureKey: 'security.roles',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js ROLES.ROOT), which requires BOTH
        // roles:read and privileges:read (permissions[] is AND).
        visibilityRequirements: { permissions: ['astrum:roles:read', 'astrum:privileges:read'] },
      },
      {
        id: 'privileges',
        path: 'security/privileges',
        label: SettingsStrings.CARDS.PRIVILEGES,
        description: SettingsStrings.CARDS.CONFIGURE_GRANULAR_PERMISSION_SETTINGS,
        featureKey: 'security.privileges',
        visibilityRequirements: { requiresPermission: 'astrum:privileges:read' },
      },
      {
        id: 'realms',
        path: 'security/realms',
        label: SettingsStrings.CARDS.REALMS,
        description: SettingsStrings.CARDS.CONFIGURE_AUTHENTICATION_SOURCES_ORDER,
        cloudExcluded: true,
        featureKey: 'security.realms',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'ldap',
        path: 'security/ldap',
        label: 'LDAP',
        description: SettingsStrings.CARDS.CONFIGURE_LDAP_INTEGRATION,
        searchTerms: ['active directory', 'ad', 'directory service'],
        cloudExcluded: true,
        featureKey: 'security.ldap',
        visibilityRequirements: { requiresPermission: 'astrum:ldap:read', bundle: 'astrum-ldap-plugin' },
      },
      {
        id: 'saml',
        path: 'security/saml',
        label: 'SAML',
        description: SettingsStrings.CARDS.CONFIGURE_SAML_SINGLE_SIGN,
        searchTerms: ['sso', 'single sign-on'],
        cloudExcluded: true,
        featureKey: 'security.saml',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js SAML.ROOT) — gates on astrum:*
        // (admin-only screen) AND requires the saml bundle, restricted to PRO. Without the
        // bundle+editions gates the tile shows on editions/installs where SAML is unavailable.
        visibilityRequirements: {
          requiresPermission: 'astrum:*',
          bundle: 'astrum-saml-plugin',
          editions: ['PRO'],
        },
      },
      {
        id: 'oauth2',
        path: 'security/oauth2',
        label: SettingsStrings.CARDS.OPENID_CONNECT,
        description: SettingsStrings.CARDS.SIGN_ANY_OPENID_CONNECT_PROVIDER,
        cloudExcluded: true,
        featureKey: 'security.oauth2',
        visibilityRequirements: {
          requiresPermission: 'astrum:*',
          statesEnabled: [{ key: 'oauth2Available', defaultValue: false }],
        },
      },
      {
        id: 'crowd',
        path: 'security/crowd',
        label: SettingsStrings.CARDS.CROWD,
        description: SettingsStrings.CARDS.CONFIGURE_ATLASSIAN_CROWD_INTEGRATION,
        cloudExcluded: true,
        featureKey: 'security.crowd',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js CROWD.ROOT) — crowd:read AND the
        // crowd bundle + a valid crowd license. Without the bundle+license gates the tile shows on
        // installs where Crowd is unavailable and then dead-ends.
        visibilityRequirements: {
          requiresPermission: 'astrum:crowd:read',
          bundle: 'astrum-crowd-plugin',
          licenseValid: [{ key: 'crowd', defaultValue: false }],
        },
      },
      {
        id: 'anonymous',
        path: 'security/anonymous',
        label: SettingsStrings.CARDS.ANONYMOUS,
        description: SettingsStrings.CARDS.CONFIGURE_ANONYMOUS_ACCESS_SETTINGS,
        cloudExcluded: true,
        featureKey: 'security.anonymous',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'ssl-certificates',
        path: 'security/sslcertificates',
        label: SettingsStrings.CARDS.SSL_CERTIFICATES,
        description: SettingsStrings.CARDS.MANAGE_TRUSTED_SSL_CERTIFICATES,
        searchTerms: ['tls', 'https', 'certificate'],
        cloudExcluded: true,
        featureKey: 'security.sslcertificates',
        visibilityRequirements: { requiresPermission: 'astrum:ssl-truststore:read' },
      },
      {
        id: 'user-tokens',
        path: 'security/user-tokens',
        label: SettingsStrings.CARDS.USER_TOKENS,
        description: SettingsStrings.CARDS.MANAGE_API_ACCESS_TOKENS,
        searchTerms: ['api', 'token', 'authentication'],
        featureKey: 'security.usertokens',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js USERTOKEN.ROOT) — usertoken-settings:read
        // gated to PRO edition AND a valid usertoken license. Without the licenseValid gate the tile
        // shows on PRO installs whose license does not include user tokens and then dead-ends.
        visibilityRequirements: {
          requiresPermission: 'astrum:usertoken-settings:read',
          editions: ['PRO'],
          licenseValid: [{ key: 'usertoken', defaultValue: false }],
        },
      },
      {
        id: 'service-account-tokens',
        path: 'security/service-account-tokens',
        label: SettingsStrings.CARDS.SERVICE_ACCOUNT_TOKENS,
        description: SettingsStrings.CARDS.MANAGE_TOKENS_AUTOMATED_SERVICE_ACCESS,
        searchTerms: ['service account', 'sat', 'ci', 'cd', 'automation', 'token', 'bearer'],
        proOnly: true,
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js SERVICEACCOUNTTOKENS.ROOT) —
        // service-accounts:read + PRO edition + the serviceAccountEnabled state.
        visibilityRequirements: {
          requiresPermission: 'astrum:service-accounts:read',
          editions: ['PRO'],
          statesEnabled: [{ key: 'serviceAccountEnabled', defaultValue: false }],
        },
      },
    ],
  },
  {
    id: 'support',
    label: SettingsStrings.CARDS.SUPPORT,
    cloudExcluded: true,
    cards: [
      {
        id: 'system-info',
        path: 'support/systeminformation',
        label: SettingsStrings.CARDS.SYSTEM_INFO,
        description: SettingsStrings.CARDS.VIEW_SYSTEM_INFORMATION_DIAGNOSTICS,
        searchTerms: ['information', 'version', 'diagnostics'],
        featureKey: 'support.systeminfo',
        visibilityRequirements: { requiresPermission: 'astrum:atlas:read' },
      },
      {
        id: 'logs',
        path: 'support/logs',
        label: SettingsStrings.CARDS.LOGS,
        description: SettingsStrings.CARDS.VIEW_APPLICATION_LOGS,
        searchTerms: ['log', 'error', 'debug'],
        featureKey: 'support.logs',
        visibilityRequirements: { requiresPermission: 'astrum:logging:read' },
      },
      {
        id: 'logging-config',
        path: 'support/logging',
        label: SettingsStrings.CARDS.LOGGING_CONFIG,
        description: SettingsStrings.CARDS.CONFIGURE_LOG_LEVELS_OUTPUT,
        searchTerms: ['logger', 'log level', 'debug'],
        featureKey: 'support.logging',
        visibilityRequirements: { requiresPermission: 'astrum:logging:read' },
      },
      {
        id: 'metric-health',
        path: 'support/metrichealth',
        label: SettingsStrings.CARDS.METRIC_HEALTH,
        description: SettingsStrings.CARDS.VIEW_SYSTEM_METRICS_HEALTH_CHECKS,
        searchTerms: ['status', 'health', 'metrics', 'monitoring'],
        featureKey: 'support.metrics',
        visibilityRequirements: { requiresPermission: 'astrum:metrics:read' },
      },
      {
        id: 'recovery-mode',
        path: 'support/recoverymode',
        label: SettingsStrings.CARDS.RECOVERY_MODE,
        description: SettingsStrings.CARDS.VIEW_STATUS_TOGGLE_RECOVERY_MODE,
        searchTerms: ['recovery', 'repair', 'reconcile', 'maintenance'],
        adminOnly: true,
        // 'astrum:*' === Permissions.ADMIN — array form mirrors the route's visibilityRequirements shape.
        // Recovery mode comes with an optional plugin, which the server reports in its state.
        visibilityRequirements: {
          permissions: ['astrum:*'],
          statesEnabled: [{ key: 'recoveryModeAvailable', defaultValue: false }],
        },
      },
      {
        id: 'support-zip',
        path: 'support/supportzip',
        label: SettingsStrings.CARDS.SUPPORT_ZIP,
        description: SettingsStrings.CARDS.GENERATE_SUPPORT_DIAGNOSTICS_BUNDLE,
        searchTerms: ['diagnostic', 'troubleshoot', 'bundle'],
        featureKey: 'support.supportzip',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js SUPPORTZIP.ROOT) — atlas:read.
        // Preview previously gated on the non-existent astrum:supportzip:read privilege.
        visibilityRequirements: { requiresPermission: 'astrum:atlas:read' },
      },
      {
        id: 'support-request',
        path: 'support/supportrequest',
        label: SettingsStrings.CARDS.SUPPORT_REQUEST,
        description: SettingsStrings.CARDS.SUBMIT_SUPPORT_REQUEST,
        searchTerms: ['help', 'ticket', 'contact'],
        featureKey: 'support.supportrequest',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js SUPPORTREQUEST.ROOT) — atlas:create
        // gated to PRO. Preview previously gated on atlas:read, which showed the tile to users who
        // lack the create permission the page actually requires.
        visibilityRequirements: { requiresPermission: 'astrum:atlas:create', editions: ['PRO'] },
      },
    ],
  },
  {
    id: 'system',
    label: SettingsStrings.CARDS.SYSTEM,
    cards: [
      {
        id: 'tasks',
        path: 'system/tasks',
        label: SettingsStrings.CARDS.TASKS,
        description: SettingsStrings.CARDS.SCHEDULE_MONITOR_SYSTEM_TASKS,
        searchTerms: ['cron', 'schedule', 'job'],
        featureKey: 'system.tasks',
        visibilityRequirements: { requiresPermission: 'astrum:tasks:read' },
      },
      {
        id: 'email',
        path: 'system/emailserver',
        label: SettingsStrings.CARDS.EMAIL,
        description: SettingsStrings.CARDS.CONFIGURE_EMAIL_SERVER_NOTIFICATIONS,
        searchTerms: ['smtp', 'mail', 'notification'],
        featureKey: 'system.emailserver',
        cloudExcluded: true,
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'http',
        path: 'system/http',
        label: 'HTTP',
        description: SettingsStrings.CARDS.CONFIGURE_HTTP_HTTPS_SETTINGS,
        searchTerms: ['proxy', 'timeout', 'connection'],
        featureKey: 'system.http',
        cloudExcluded: true,
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'api',
        path: 'system/api',
        fullHash: '#preview/browse/api',
        label: 'API',
        description: SettingsStrings.CARDS.CONFIGURE_REST_API_SETTINGS,
        searchTerms: ['rest', 'endpoint'],
        featureKey: 'system.api',
        // ASTRUM-54048: API has no dedicated privilege; the Default UI and Classic UI
        // gate it on astrum:settings:read (matches the preview.browse.api route).
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'capabilities',
        path: 'system/capabilities',
        label: SettingsStrings.CARDS.CAPABILITIES,
        description: SettingsStrings.CARDS.CONFIGURE_SYSTEM_CAPABILITIES_INTEGRATIONS,
        featureKey: 'system.capabilities',
        visibilityRequirements: { requiresPermission: 'astrum:capabilities:read' },
      },
      {
        id: 'notifications',
        path: 'system/notifications',
        label: UIStrings.NOTIFICATIONS.MENU.text,
        description: UIStrings.NOTIFICATIONS.MENU.description,
        searchTerms: ['telegram', 'slack', 'mattermost', 'rocket.chat', 'chat', 'webhook', 'alert'],
        featureKey: 'system.notifications',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'nodes',
        path: 'system/nodes',
        label: SettingsStrings.CARDS.NODES,
        description: SettingsStrings.CARDS.VIEW_CLUSTER_NODES_STATUS,
        searchTerms: ['cluster', 'ha', 'high availability'],
        cloudExcluded: true,
        featureKey: 'system.nodes',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js NODES.ROOT), which gates on astrum:* —
        // the Nodes screen is admin-only. A settings:read gate surfaced the tile to read-only users
        // who would then hit a 403 opening the page.
        visibilityRequirements: { requiresPermission: 'astrum:*' },
      },
      {
        id: 'usage',
        path: 'system/usage',
        label: SettingsStrings.CARDS.USAGE,
        description: SettingsStrings.CARDS.MONITOR_HISTORICAL_USAGE_METRICS_TRENDS,
        searchTerms: ['metrics', 'egress', 'storage', 'usage insights'],
        cloudOnly: true,
        // ASTRUM-54048: match cloud Classic (cloud adminRoutes.js HISTORICAL_USAGE), gated on settings:read.
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
    ],
  },
];
