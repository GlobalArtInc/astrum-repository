

/**
 * Settings Hub Configuration
 *
 * Defines all settings cards and their organization into sections.
 * This file is the single source of truth for the Settings Hub.
 *
 * Section order intentionally mirrors the classic ExtJS admin navigation:
 * Repository → Security → Support → System
 */

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
  /** All listed NX.State keys must resolve truthy. */
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
    label: 'Repository',
    cards: [
      {
        id: 'repositories',
        path: 'repository/repositories',
        label: 'Repositories',
        description: 'Manage Maven, npm, Docker, and other repositories',
        searchTerms: ['maven', 'npm', 'docker', 'pypi', 'nuget'],
        featureKey: 'repository.repositories',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js REPOSITORIES.ROOT), which uses a
        // repository-admin prefix so a user with admin on any single repo can see the menu.
        visibilityRequirements: { permissionPrefix: 'astrum:repository-admin' },
      },
      {
        id: 'blob-stores',
        path: 'repository/blobstores',
        label: 'Blob Stores',
        description: 'Configure local and cloud blob storage',
        searchTerms: ['storage', 's3', 'azure', 'google cloud'],
        cloudExcluded: true,
        featureKey: 'repository.blobstores',
        visibilityRequirements: { requiresPermission: 'astrum:blobstores:read' },
      },
      {
        id: 'cleanup-policies',
        path: 'repository/cleanup-policies',
        label: 'Cleanup Policies',
        description: 'Configure automated component cleanup',
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
        label: 'Routing Rules',
        description: 'Control component access by path patterns',
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
        label: 'Content Selectors',
        description: 'Define content filtering expressions',
        searchTerms: ['csel', 'filter', 'query'],
        featureKey: 'repository.selectors',
        visibilityRequirements: { requiresPermission: 'astrum:selectors:read' },
      },
      {
        id: 'data-store',
        path: 'repository/datastore',
        label: 'Data Store',
        description: 'Configure repository metadata database',
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
        label: 'Proprietary',
        description: 'Manage proprietary component settings',
        featureKey: 'repository.proprietary',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
    ],
  },
  {
    id: 'security',
    label: 'Security',
    cards: [
      {
        id: 'users',
        path: 'security/users',
        label: 'Users',
        description: 'Manage user accounts and access',
        featureKey: 'security.users',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js USERS.ROOT), which requires BOTH
        // users:read and roles:read (permissions[] is AND).
        visibilityRequirements: { permissions: ['astrum:users:read', 'astrum:roles:read'] },
      },
      {
        id: 'roles',
        path: 'security/roles',
        label: 'Roles',
        description: 'Define user permissions and access control',
        featureKey: 'security.roles',
        // ASTRUM-54048: match Classic/Default UI (adminRoutes.js ROLES.ROOT), which requires BOTH
        // roles:read and privileges:read (permissions[] is AND).
        visibilityRequirements: { permissions: ['astrum:roles:read', 'astrum:privileges:read'] },
      },
      {
        id: 'privileges',
        path: 'security/privileges',
        label: 'Privileges',
        description: 'Configure granular permission settings',
        featureKey: 'security.privileges',
        visibilityRequirements: { requiresPermission: 'astrum:privileges:read' },
      },
      {
        id: 'realms',
        path: 'security/realms',
        label: 'Realms',
        description: 'Configure authentication sources and order',
        cloudExcluded: true,
        featureKey: 'security.realms',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'ldap',
        path: 'security/ldap',
        label: 'LDAP',
        description: 'Configure LDAP integration',
        searchTerms: ['active directory', 'ad', 'directory service'],
        cloudExcluded: true,
        featureKey: 'security.ldap',
        visibilityRequirements: { requiresPermission: 'astrum:ldap:read', bundle: 'astrum-ldap-plugin' },
      },
      {
        id: 'saml',
        path: 'security/saml',
        label: 'SAML',
        description: 'Configure SAML single sign-on',
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
        label: 'OpenID Connect',
        description: 'Sign in with any OpenID Connect provider',
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
        label: 'Crowd',
        description: 'Configure Atlassian Crowd integration',
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
        label: 'Anonymous',
        description: 'Configure anonymous access settings',
        cloudExcluded: true,
        featureKey: 'security.anonymous',
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'ssl-certificates',
        path: 'security/sslcertificates',
        label: 'SSL Certificates',
        description: 'Manage trusted SSL certificates',
        searchTerms: ['tls', 'https', 'certificate'],
        cloudExcluded: true,
        featureKey: 'security.sslcertificates',
        visibilityRequirements: { requiresPermission: 'astrum:ssl-truststore:read' },
      },
      {
        id: 'user-tokens',
        path: 'security/user-tokens',
        label: 'User Tokens',
        description: 'Manage API access tokens',
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
        label: 'Service Account Tokens',
        description: 'Manage tokens for automated service access',
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
    label: 'Support',
    cloudExcluded: true,
    cards: [
      {
        id: 'system-info',
        path: 'support/systeminformation',
        label: 'System Info',
        description: 'View system information and diagnostics',
        searchTerms: ['information', 'version', 'diagnostics'],
        featureKey: 'support.systeminfo',
        visibilityRequirements: { requiresPermission: 'astrum:atlas:read' },
      },
      {
        id: 'logs',
        path: 'support/logs',
        label: 'Logs',
        description: 'View application logs',
        searchTerms: ['log', 'error', 'debug'],
        featureKey: 'support.logs',
        visibilityRequirements: { requiresPermission: 'astrum:logging:read' },
      },
      {
        id: 'logging-config',
        path: 'support/logging',
        label: 'Logging Config',
        description: 'Configure log levels and output',
        searchTerms: ['logger', 'log level', 'debug'],
        featureKey: 'support.logging',
        visibilityRequirements: { requiresPermission: 'astrum:logging:read' },
      },
      {
        id: 'metric-health',
        path: 'support/metrichealth',
        label: 'Metric Health',
        description: 'View system metrics and health checks',
        searchTerms: ['status', 'health', 'metrics', 'monitoring'],
        featureKey: 'support.metrics',
        visibilityRequirements: { requiresPermission: 'astrum:metrics:read' },
      },
      {
        id: 'recovery-mode',
        path: 'support/recoverymode',
        label: 'Recovery Mode',
        description: 'View status and toggle recovery mode for system maintenance and data repairs',
        searchTerms: ['recovery', 'repair', 'reconcile', 'maintenance'],
        adminOnly: true,
        // 'astrum:*' === Permissions.ADMIN — array form mirrors the route's visibilityRequirements shape.
        visibilityRequirements: { permissions: ['astrum:*'] },
      },
      {
        id: 'support-zip',
        path: 'support/supportzip',
        label: 'Support Zip',
        description: 'Generate support diagnostics bundle',
        searchTerms: ['diagnostic', 'troubleshoot', 'bundle'],
        featureKey: 'support.supportzip',
        // ASTRUM-54019: match Classic/Default UI (adminRoutes.js SUPPORTZIP.ROOT) — atlas:read.
        // Preview previously gated on the non-existent astrum:supportzip:read privilege.
        visibilityRequirements: { requiresPermission: 'astrum:atlas:read' },
      },
      {
        id: 'support-request',
        path: 'support/supportrequest',
        label: 'Support Request',
        description: 'Submit a support request',
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
    label: 'System',
    cards: [
      {
        id: 'tasks',
        path: 'system/tasks',
        label: 'Tasks',
        description: 'Schedule and monitor system tasks',
        searchTerms: ['cron', 'schedule', 'job'],
        featureKey: 'system.tasks',
        visibilityRequirements: { requiresPermission: 'astrum:tasks:read' },
      },
      {
        id: 'email',
        path: 'system/emailserver',
        label: 'Email',
        description: 'Configure email server and notifications',
        searchTerms: ['smtp', 'mail', 'notification'],
        featureKey: 'system.emailserver',
        cloudExcluded: true,
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'http',
        path: 'system/http',
        label: 'HTTP',
        description: 'Configure HTTP and HTTPS settings',
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
        description: 'Configure REST API settings',
        searchTerms: ['rest', 'endpoint'],
        featureKey: 'system.api',
        // ASTRUM-54048: API has no dedicated privilege; the Default UI and Classic UI
        // gate it on astrum:settings:read (matches the preview.browse.api route).
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'capabilities',
        path: 'system/capabilities',
        label: 'Capabilities',
        description: 'Configure system capabilities and integrations',
        featureKey: 'system.capabilities',
        visibilityRequirements: { requiresPermission: 'astrum:capabilities:read' },
      },
      {
        id: 'nodes',
        path: 'system/nodes',
        label: 'Nodes',
        description: 'View cluster nodes and status',
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
        label: 'Usage',
        description: 'Monitor historical usage metrics and trends',
        searchTerms: ['metrics', 'egress', 'storage', 'usage insights'],
        cloudOnly: true,
        // ASTRUM-54048: match cloud Classic (cloud adminRoutes.js HISTORICAL_USAGE), gated on settings:read.
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
      {
        id: 'preview-ui',
        path: 'system/previewui',
        label: 'Astrum One UI',
        description: 'Configure Astrum One UI settings and features',
        searchTerms: ['interface', 'ui', 'theme'],
        // No featureKey — always available
        visibilityRequirements: { requiresPermission: 'astrum:settings:read' },
      },
    ],
  },
];
