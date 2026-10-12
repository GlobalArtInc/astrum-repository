
/**
 * Preview Admin Routes - Phase 4.0 Fork Infrastructure
 * 
 * Admin (Settings) routes for Preview UI.
 * Uses ComingSoonPage for features being migrated.
 * Uses SettingsPageLayoutRadix for secondary sidebar navigation.
 * 
 * IMPORTANT: Route names must match the default admin route names (with preview. prefix)
 * so that useDefaultAdminRouteName() works correctly.
 * 
 * URL Structure (hash-based routing at the root):
 * - #settings                           → preview.settings
 * - #admin/repository/repositories      → preview.admin.repository.repositories
 * - #admin/repository/blobstores        → preview.admin.repository.blobstores
 * - etc.
 */

import React from 'react';
import {UIView} from '@uirouter/react';
import {
  Permissions,
  withFeatureGate,
  withCloudExcluded,
  SettingsPageLayoutRadix,
  SettingsHubPage,
  GlobalArtTestHub,
  GlobalArtInternalTestDashboard,
  GlobalArtInternalTestSearchHc,
  MalwareDashboardScenarioPage,
  RepositoriesPage as RepositoriesPageRaw,
  RepositoryProfilePage as RepositoryProfilePageRaw,
  BlobStoresPage as BlobStoresPageRaw,
  ContentSelectorsPage as ContentSelectorsPageRaw,
  CleanupPoliciesPage as CleanupPoliciesPageRaw,
  RoutingRulesPage as RoutingRulesPageRaw,
  DataStorePage as DataStorePageRaw,
  ProprietaryPage as ProprietaryPageRaw,
  PrivilegesPage as PrivilegesPageRaw,
  RolesPage as RolesPageRaw,
  UsersPage as UsersPageRaw,
  AnonymousPage as AnonymousPageRaw,
  CrowdPage as CrowdPageRaw,
  LdapPage as LdapPageRaw,
  OAuth2Page as OAuth2PageRaw,
  RealmsPage as RealmsPageRaw,
  SamlPage as SamlPageRaw,
  SslCertificatesPage as SslCertificatesPageRaw,
  UserTokensPage as UserTokensPageRaw,
  ServiceAccountTokensPage,
  LogsPage as LogsPageRaw,
  LoggingConfigPage as LoggingConfigPageRaw,
  SystemInfoPage as SystemInfoPageRaw,
  MetricHealthPage as MetricHealthPageRaw,
  RecoveryModePage as RecoveryModePageRaw,
  SupportRequestPage as SupportRequestPageRaw,
  SupportZipPage as SupportZipPageRaw,
  TasksPage as TasksPageRaw,
  CapabilitiesPage as CapabilitiesPageRaw,
  EmailPage as EmailPageRaw,
  HttpPage as HttpPageRaw,
  NodesPage as NodesPageRaw,
  NotificationsPage as NotificationsPageRaw,
  UsagePage,
  UserAccountPage as UserAccountPageRaw,
} from '@globalart/astrum-ui-plugin';
import {lazyLoad} from './lazyLoad';
import PreviewRouteTitles from './previewRouteTitles';

// ui-plugin already exports these as React.lazy(); wrapping with lazyLoad() would double-wrap them.
// Two distinct wrapper functions force React to unmount→remount when transitioning
// between iqConnected and iqConnection (same underlying component, different element type).

// GlobalArt-internal scenario route wraps MalwareDashboardScenarioPage with
// a transition-param unwrapper. The named export from ui-plugin is the
// underlying (non-default) export from MalwareDashboardTestPage.
const GlobalArtInternalTestScenario = ({ transition }) => {
  const id = transition?.params?.().scenarioId ?? 'A';
  return React.createElement(MalwareDashboardScenarioPage, { scenarioId: id });
};

const globalartInternalRoutes = [
  // Hub — index of all test pages
  {
    name: 'preview.test',
    url: 'test',
    component: GlobalArtTestHub,
    data: { title: '[Internal] Test Hub' },
  },
  // Dashboard / MalwareStatusCard
  {
    name: 'preview.test-dashboard',
    url: 'test-dashboard',
    component: GlobalArtInternalTestDashboard,
    data: { title: '[Internal] Test — Dashboard' },
  },
  {
    name: 'preview.test-dashboard.scenario',
    url: '/test-dashboard/scenario/:scenarioId',
    component: GlobalArtInternalTestScenario,
    data: { title: '[Internal] Dashboard Scenario' },
  },
  // Health Check cell permutations
  {
    name: 'preview.test-search-hc',
    url: 'test-search-hc',
    component: GlobalArtInternalTestSearchHc,
    data: { title: '[Internal] Test — Health Check Cell' },
  },
];

// =============================================================================
// FEATURE-GATED PAGES
// Each gated page wraps the ui-plugin export with ComingSoonPage in
// production. Raw (ungated) components are imported above as <Name>Raw.
// =============================================================================

// Repository - Gated (Coming Soon in production, WIP accessible in dev only)
const RepositoriesPage = withFeatureGate(RepositoriesPageRaw, 'repository.repositories', PreviewRouteTitles.REPOSITORIES);
const RepositoryProfilePage = withFeatureGate(RepositoryProfilePageRaw, 'repository.repositories', PreviewRouteTitles.REPOSITORY_PROFILE);
const BlobStoresPage = withFeatureGate(BlobStoresPageRaw, 'repository.blobstores', PreviewRouteTitles.BLOB_STORES);
const ContentSelectorsPage = withFeatureGate(ContentSelectorsPageRaw, 'repository.selectors', PreviewRouteTitles.CONTENT_SELECTORS);
const CleanupPoliciesPage = withFeatureGate(CleanupPoliciesPageRaw, 'repository.cleanuppolicies', PreviewRouteTitles.CLEANUP_POLICIES);
const RoutingRulesPage = withFeatureGate(RoutingRulesPageRaw, 'repository.routingrules', PreviewRouteTitles.ROUTING_RULES);
const DataStorePage = withFeatureGate(DataStorePageRaw, 'repository.datastore', PreviewRouteTitles.DATA_STORE);
const ProprietaryPage = withFeatureGate(ProprietaryPageRaw, 'repository.proprietary', PreviewRouteTitles.PROPRIETARY_REPOSITORIES);

// Security - Gated
const PrivilegesPage = withFeatureGate(PrivilegesPageRaw, 'security.privileges', PreviewRouteTitles.PRIVILEGES);
const RolesPage = withFeatureGate(RolesPageRaw, 'security.roles', PreviewRouteTitles.ROLES);
const UsersPage = withFeatureGate(UsersPageRaw, 'security.users', PreviewRouteTitles.USERS);
const AnonymousPage = withFeatureGate(AnonymousPageRaw, 'security.anonymous', PreviewRouteTitles.ANONYMOUS_ACCESS);
const CrowdPage = withFeatureGate(CrowdPageRaw, 'security.crowd', PreviewRouteTitles.ATLASSIAN_CROWD);
const LdapPage = withFeatureGate(LdapPageRaw, 'security.ldap', PreviewRouteTitles.LDAP);
const OAuth2Page = withFeatureGate(OAuth2PageRaw, 'security.oauth2', PreviewRouteTitles.OAUTH2);
const RealmsPage = withFeatureGate(RealmsPageRaw, 'security.realms', PreviewRouteTitles.REALMS);
const SamlPage = withFeatureGate(SamlPageRaw, 'security.saml', PreviewRouteTitles.SAML);
const SslCertificatesPage = withFeatureGate(SslCertificatesPageRaw, 'security.sslcertificates', PreviewRouteTitles.SSL_CERTIFICATES);
const UserTokensPage = withFeatureGate(UserTokensPageRaw, 'security.usertokens', PreviewRouteTitles.USER_TOKENS);

// Support - Gated (Logs and Logging Configuration are self-hosted only)
const LogsPage = withCloudExcluded(
  withFeatureGate(LogsPageRaw, 'support.logs', PreviewRouteTitles.LOGS),
  PreviewRouteTitles.LOGS,
);
const LoggingConfigPage = withCloudExcluded(
  withFeatureGate(LoggingConfigPageRaw, 'support.logging', PreviewRouteTitles.LOGGING_CONFIGURATION),
  PreviewRouteTitles.LOGGING_CONFIGURATION,
);
const SystemInfoPage = withFeatureGate(SystemInfoPageRaw, 'support.systeminfo', PreviewRouteTitles.SYSTEM_INFORMATION);
const MetricHealthPage = withFeatureGate(MetricHealthPageRaw, 'support.metrics', PreviewRouteTitles.METRIC_HEALTH);
const RecoveryModePage = RecoveryModePageRaw;
const SupportRequestPage = withFeatureGate(SupportRequestPageRaw, 'support.supportrequest', PreviewRouteTitles.SUPPORT_REQUEST);
const SupportZipPage = withFeatureGate(SupportZipPageRaw, 'support.supportzip', PreviewRouteTitles.SUPPORT_ZIP);

// System - Gated
const TasksPage = withFeatureGate(TasksPageRaw, 'system.tasks', PreviewRouteTitles.TASKS);
const CapabilitiesPage = withFeatureGate(CapabilitiesPageRaw, 'system.capabilities', PreviewRouteTitles.CAPABILITIES);
const EmailPage = withFeatureGate(EmailPageRaw, 'system.emailserver', PreviewRouteTitles.EMAIL_SERVER);
const HttpPage = withCloudExcluded(
  withFeatureGate(HttpPageRaw, 'system.http', PreviewRouteTitles.HTTP),
  PreviewRouteTitles.HTTP,
);
const NodesPage = withFeatureGate(NodesPageRaw, 'system.nodes', PreviewRouteTitles.NODES);
const NotificationsPage = withFeatureGate(NotificationsPageRaw, 'system.notifications', PreviewRouteTitles.NOTIFICATIONS);

// User Account
const UserAccountPage = withFeatureGate(UserAccountPageRaw, 'useraccount', PreviewRouteTitles.USER_ACCOUNT);

// =============================================================================
// ROUTE DEFINITIONS
// Route names MUST match default admin routes (preview.admin.repository.repositories, etc.)
// URLs are flat at the admin level
// =============================================================================

// Exported separately so test pages can be registered at the preview.* level
// (not preview.admin.*) — they render standalone without the Settings sidebar.
export const globalartInternalTestRoutes = globalartInternalRoutes;

export const previewAdminRoutes = [
  // =============================================================================
  // SETTINGS HUB - Standalone at preview.settings (no sidebar)
  // =============================================================================
  {
    name: 'preview.settings',
    url: 'settings',
    component: SettingsHubPage,
    data: {
      title: PreviewRouteTitles.SETTINGS,
      visibilityRequirements: {
        requiresAnyPermission: [
          Permissions.REPOSITORY_ADMIN.READ,
          Permissions.BLOB_STORES.READ,
          Permissions.SELECTORS.READ,
          Permissions.PRIVILEGES.READ,
          Permissions.ROLES.READ,
          Permissions.USERS.READ,
          Permissions.SETTINGS.READ,
          Permissions.SSL_TRUSTSTORE.READ,
          Permissions.LDAP.READ,
          Permissions.USER_TOKENS_SETTINGS.READ,
          'astrum:crowd:read',
          Permissions.TASKS.READ,
          Permissions.CAPABILITIES.READ,
          Permissions.LOGGING.READ,
          Permissions.ATLAS.READ,
          Permissions.METRICS.READ,
          Permissions.LICENSING.READ,
        ],
      },
    },
  },
  // =============================================================================
  // ROOT ADMIN ROUTE - Abstract parent for /admin paths
  // =============================================================================
  {
    name: 'preview.admin',
    url: 'admin',
    abstract: true,
    component: UIView,
  },

  // =============================================================================
  // SETTINGS HUB REDIRECT - /admin/settings redirects to Settings Hub
  // =============================================================================
  {
    name: 'preview.admin.settings',
    url: '/settings',
    redirectTo: 'preview.settings',
  },

  // =============================================================================
  // ADMIN PAGES WRAPPER - Provides sidebar for all settings pages
  // =============================================================================
  {
    name: 'preview.admin.pages',
    url: '',
    // #admin alone, as the Classic settings link was, opens the settings hub
    redirectTo: 'preview.settings',
    component: SettingsPageLayoutRadix,
    data: {
      visibilityRequirements: {
        requiresAnyPermission: [
          Permissions.REPOSITORY_ADMIN.READ,
          Permissions.BLOB_STORES.READ,
          Permissions.SELECTORS.READ,
          Permissions.PRIVILEGES.READ,
          Permissions.ROLES.READ,
          Permissions.USERS.READ,
          Permissions.SETTINGS.READ,
          Permissions.SSL_TRUSTSTORE.READ,
          Permissions.LDAP.READ,
          Permissions.USER_TOKENS_SETTINGS.READ,
          'astrum:crowd:read',
          Permissions.TASKS.READ,
          Permissions.CAPABILITIES.READ,
          Permissions.LOGGING.READ,
          Permissions.ATLAS.READ,
          Permissions.METRICS.READ,
          Permissions.LICENSING.READ,
        ],
      },
    },
  },

  // =============================================================================
  // REPOSITORY SECTION
  // Names match: admin.repository.* → preview.admin.repository.*
  // =============================================================================
  {
    name: 'preview.admin.repository',
    url: '/repository',
    redirectTo: 'preview.admin.repository.repositories.list', /* DO NOT CHANGE - blobstores not on cloud (Agent0 fix) */
    component: UIView,
    data: { title: PreviewRouteTitles.REPOSITORY },
  },
  {
    name: 'preview.admin.repository.repositories',
    url: '/repositories',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.REPOSITORIES,
      // ASTRUM-54048: match the retired Classic page, which uses a
      // repository-admin prefix so a user with admin on any single repo can see the menu. A specific
      // requiresPermission: 'astrum:repository-admin:*:*:read' would NOT be satisfied by a narrower
      // per-repo grant, hiding the menu from users the Default UI shows it to.
      visibilityRequirements: {
        permissionPrefix: 'astrum:repository-admin',
      },
    },
  },
  {
    name: 'preview.admin.repository.repositories.list',
    url: '',
    component: RepositoriesPage,
    data: { title: PreviewRouteTitles.REPOSITORIES },
  },
  {
    name: 'preview.admin.repository.repositories.create',
    url: '/create',
    component: RepositoriesPage,
    data: { title: PreviewRouteTitles.CREATE_REPOSITORY },
  },
  {
    name: 'preview.admin.repository.repositories.createForm',
    url: '/create/:format/:type',
    component: RepositoriesPage,
    data: { title: PreviewRouteTitles.CREATE_REPOSITORY },
  },
  {
    name: 'preview.admin.repository.repositories.detail',
    url: '/:repositoryName?tab',
    component: RepositoriesPage,
    params: {
      tab: { value: null, type: 'string', dynamic: true },
    },
    data: { title: PreviewRouteTitles.REPOSITORY_DETAILS },
  },
  {
    name: 'preview.admin.repository.repositories.profile',
    url: '/:repositoryName/profile?tab',
    component: RepositoryProfilePage,
    params: {
      tab: { value: null, type: 'string', dynamic: true },
    },
    data: { title: PreviewRouteTitles.REPOSITORY_PROFILE },
    resolve: [
      {
        token: 'repositoryName',
        deps: ['$stateParams'],
        resolveFn: ($stateParams) => $stateParams.repositoryName,
      },
      {
        token: 'context',
        resolveFn: () => 'settings', // Settings context - shows Settings sidebar
      },
    ],
  },
  {
    name: 'preview.admin.repository.blobstores',
    url: '/blobstores',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.BLOB_STORES,
      visibilityRequirements: {
        requiresPermission: Permissions.BLOB_STORES.READ,
      },
    },
  },
  {
    name: 'preview.admin.repository.blobstores.list',
    url: '',
    component: BlobStoresPage,
    data: { title: PreviewRouteTitles.BLOB_STORES },
  },
  {
    name: 'preview.admin.repository.blobstores.create',
    url: '/create',
    component: BlobStoresPage,
    data: { title: PreviewRouteTitles.CREATE_BLOB_STORE },
  },
  {
    name: 'preview.admin.repository.blobstores.edit',
    url: '/:type/:name',
    component: BlobStoresPage,
    data: { title: PreviewRouteTitles.EDIT_BLOB_STORE },
  },
  {
    name: 'preview.admin.repository.selectors',
    url: '/selectors',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.CONTENT_SELECTORS,
      visibilityRequirements: {
        requiresPermission: Permissions.SELECTORS.READ,
      },
    },
  },
  {
    name: 'preview.admin.repository.selectors.list',
    url: '',
    component: ContentSelectorsPage,
    data: { title: PreviewRouteTitles.CONTENT_SELECTORS },
  },
  {
    name: 'preview.admin.repository.selectors.create',
    url: '/create',
    component: ContentSelectorsPage,
    data: { title: PreviewRouteTitles.CREATE_CONTENT_SELECTOR },
  },
  {
    name: 'preview.admin.repository.selectors.detail',
    url: '/:name',
    component: ContentSelectorsPage,
    data: { title: PreviewRouteTitles.CONTENT_SELECTOR_DETAILS },
  },
  {
    name: 'preview.admin.repository.cleanuppolicies',
    url: '/cleanup-policies',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.CLEANUP_POLICIES,
      // ASTRUM-54048: match the retired Classic page, which gates on
      // astrum:* — the write endpoints require admin, and the menu must not surface to a
      // settings:read-only user who would then hit a 403.
      visibilityRequirements: {
        requiresPermission: Permissions.ADMIN,
      },
    },
  },
  {
    name: 'preview.admin.repository.cleanuppolicies.list',
    url: '',
    component: CleanupPoliciesPage,
    data: { title: PreviewRouteTitles.CLEANUP_POLICIES },
  },
  {
    name: 'preview.admin.repository.cleanuppolicies.create',
    url: '/create',
    component: CleanupPoliciesPage,
    data: { title: PreviewRouteTitles.CREATE_CLEANUP_POLICY },
  },
  {
    name: 'preview.admin.repository.cleanuppolicies.detail',
    url: '/:name',
    component: CleanupPoliciesPage,
    data: { title: PreviewRouteTitles.CLEANUP_POLICY_DETAILS },
  },
  {
    name: 'preview.admin.repository.routingrules',
    url: '/routing-rules',
    abstract: true,
    redirectTo: 'preview.admin.repository.routingrules.list',
    component: UIView,
    data: {
      title: PreviewRouteTitles.ROUTING_RULES,
      // ASTRUM-54048: match the retired Classic page. RoutingRulesResource
      // requires astrum:* on every endpoint, including the GET/list — so settings:read here would show
      // the menu but 403 on load.
      visibilityRequirements: {
        requiresPermission: Permissions.ADMIN,
      },
    },
  },
  {
    name: 'preview.admin.repository.routingrules.list',
    url: '',
    component: RoutingRulesPage,
    data: { title: PreviewRouteTitles.ROUTING_RULES },
  },
  {
    name: 'preview.admin.repository.routingrules.create',
    url: '/create',
    component: RoutingRulesPage,
    data: { title: PreviewRouteTitles.CREATE_ROUTING_RULE },
  },
  {
    name: 'preview.admin.repository.routingrules.preview',
    url: '/preview',
    component: RoutingRulesPage,
    data: { title: PreviewRouteTitles.ROUTING_RULES_PREVIEW },
  },
  {
    name: 'preview.admin.repository.routingrules.detail',
    url: '/:name',
    component: RoutingRulesPage,
    data: { title: PreviewRouteTitles.ROUTING_RULE_DETAILS },
  },
  {
    name: 'preview.admin.repository.datastore',
    url: '/datastore',
    component: DataStorePage,
    data: {
      title: PreviewRouteTitles.DATA_STORE,
      visibilityRequirements: {
        // ASTRUM-54019: match the retired Classic page. The Data Store
        // screen is admin-only (astrum:*) AND only available with the pro-datastore bundle on
        // PRO/COMMUNITY editions — without the bundle+editions gates the tile shows where the
        // feature does not exist and then dead-ends.
        bundle: 'astrum-pro-datastore-plugin',
        requiresPermission: Permissions.ADMIN,
        editions: ['PRO', 'COMMUNITY'],
      },
    },
  },
  {
    name: 'preview.admin.repository.proprietary',
    url: '/proprietary',
    component: ProprietaryPage,
    data: {
      title: PreviewRouteTitles.PROPRIETARY_REPOSITORIES,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },

  // =============================================================================
  // SECURITY SECTION  
  // Names match: admin.security.* → preview.admin.security.*
  // =============================================================================
  {
    name: 'preview.admin.security',
    url: '/security',
    redirectTo: 'preview.admin.security.privileges.list',
    component: UIView,
    data: { title: PreviewRouteTitles.SECURITY },
  },
  {
    name: 'preview.admin.security.privileges',
    url: '/privileges',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.PRIVILEGES,
      visibilityRequirements: {
        requiresPermission: 'astrum:privileges:read',
      },
    },
  },
  {
    name: 'preview.admin.security.privileges.list',
    url: '',
    component: PrivilegesPage,
    data: {
      title: PreviewRouteTitles.PRIVILEGES,
    },
  },
  {
    name: 'preview.admin.security.privileges.create',
    url: '/create',
    component: PrivilegesPage,
    data: { title: PreviewRouteTitles.CREATE_PRIVILEGE },
  },
  {
    name: 'preview.admin.security.privileges.createForm',
    url: '/create/:typeId',
    component: PrivilegesPage,
    params: { typeId: { type: 'string', value: null, dynamic: true } },
    data: { title: PreviewRouteTitles.CREATE_PRIVILEGE },
  },
  {
    // ASTRUM-52167: `?tab` keeps the active profile tab in the URL so it survives
    // browser Back. `dynamic` avoids a remount on tab switch. Mirrors
    // repositories.profile above.
    name: 'preview.admin.security.privileges.profile',
    url: '/:privilegeId/profile?tab',
    component: PrivilegesPage,
    params: {
      tab: { value: null, type: 'string', dynamic: true },
    },
    data: { title: PreviewRouteTitles.PRIVILEGE_PROFILE },
  },
  {
    name: 'preview.admin.security.privileges.detail',
    url: '/:privilegeId',
    component: PrivilegesPage,
    data: { title: PreviewRouteTitles.PRIVILEGE_DETAILS },
  },
  {
    name: 'preview.admin.security.roles',
    url: '/roles',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.ROLES,
      // ASTRUM-54048: match the retired Classic page, which requires BOTH
      // roles:read and privileges:read (permissions[] is AND).
      visibilityRequirements: {
        permissions: [Permissions.ROLES.READ, Permissions.PRIVILEGES.READ],
      },
    },
  },
  {
    name: 'preview.admin.security.roles.list',
    url: '',
    component: RolesPage,
    data: {
      title: PreviewRouteTitles.ROLES,
    },
  },
  {
    name: 'preview.admin.security.roles.create',
    url: '/create',
    component: RolesPage,
    data: { title: PreviewRouteTitles.CREATE_ROLE },
  },
  {
    name: 'preview.admin.security.roles.detail',
    url: '/:roleId',
    component: RolesPage,
    data: { title: PreviewRouteTitles.ROLE_DETAILS },
  },
  {
    name: 'preview.admin.security.roles.profile',
    url: '/:roleId/profile',
    component: RolesPage,
    data: { title: PreviewRouteTitles.ROLE_PROFILE },
  },
  {
    name: 'preview.admin.security.users',
    url: '/users',
    abstract: true,
    redirectTo: 'preview.admin.security.users.list',
    component: UIView,
    data: {
      title: PreviewRouteTitles.USERS,
      // ASTRUM-54048: match the retired Classic page, which requires BOTH
      // users:read and roles:read (permissions[] is AND).
      visibilityRequirements: {
        permissions: [Permissions.USERS.READ, Permissions.ROLES.READ],
      },
    },
  },
  {
    name: 'preview.admin.security.users.list',
    url: '',
    component: UsersPage,
    data: { title: PreviewRouteTitles.USERS },
  },
  {
    name: 'preview.admin.security.users.create',
    url: '/create',
    component: UsersPage,
    data: { title: PreviewRouteTitles.CREATE_USER },
  },
  {
    name: 'preview.admin.security.users.invite',
    url: '/invite',
    component: UsersPage,
    data: { title: PreviewRouteTitles.INVITE_USER },
  },
  {
    name: 'preview.admin.security.users.detail',
    url: '/:userId/:source',
    component: UsersPage,
    params: {
      userId: { type: 'string', value: null, dynamic: true },
      source: { type: 'string', value: null, dynamic: true },
    },
    data: { title: PreviewRouteTitles.USER_DETAILS },
  },
  {
    name: 'preview.admin.security.users.profile',
    url: '/:userId/:source/profile',
    component: UsersPage,
    params: {
      userId: { type: 'string', value: null, dynamic: true },
      source: { type: 'string', value: null, dynamic: true },
    },
    data: { title: PreviewRouteTitles.USER_PROFILE },
  },
  {
    name: 'preview.admin.security.anonymous',
    url: '/anonymous',
    component: AnonymousPage,
    data: {
      title: PreviewRouteTitles.ANONYMOUS_ACCESS,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },
  {
    name: 'preview.admin.security.ldap',
    url: '/ldap',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.LDAP,
      visibilityRequirements: {
        bundle: 'astrum-ldap-plugin',
        requiresPermission: Permissions.LDAP.READ,
      },
    },
  },
  {
    name: 'preview.admin.security.ldap.list',
    url: '',
    component: LdapPage,
    data: { title: PreviewRouteTitles.LDAP },
  },
  {
    name: 'preview.admin.security.ldap.create',
    url: '/create',
    component: LdapPage,
    data: { title: PreviewRouteTitles.CREATE_LDAP_SERVER },
  },
  {
    name: 'preview.admin.security.ldap.detail',
    url: '/:serverId',
    component: LdapPage,
    data: { title: PreviewRouteTitles.LDAP_SERVER_DETAILS },
  },
  {
    name: 'preview.admin.security.realms',
    url: '/realms',
    component: RealmsPage,
    data: {
      title: PreviewRouteTitles.REALMS,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },
  {
    name: 'preview.admin.security.sslcertificates',
    url: '/sslcertificates',
    abstract: true,
    component: UIView,
    data: { 
      title: PreviewRouteTitles.SSL_CERTIFICATES,
      visibilityRequirements: {
        requiresPermission: 'astrum:ssl-truststore:read',
      },
    },
  },
  {
    name: 'preview.admin.security.sslcertificates.list',
    url: '',
    component: SslCertificatesPage,
    data: { title: PreviewRouteTitles.SSL_CERTIFICATES },
  },
  {
    name: 'preview.admin.security.usertoken',
    url: '/user-tokens',
    component: UserTokensPage,
    data: {
      title: PreviewRouteTitles.USER_TOKENS,
      // ASTRUM-54019: match the retired Classic page — usertoken-settings:read
      // gated to PRO edition AND a valid usertoken license. Without the licenseValid gate the tile
      // shows on PRO installs whose license does not include user tokens and then dead-ends.
      visibilityRequirements: {
        requiresPermission: Permissions.USER_TOKENS_SETTINGS.READ,
        editions: ['PRO'],
        licenseValid: [{ key: 'usertoken', defaultValue: false }],
      },
    },
  },
  {
    name: 'preview.admin.security.serviceaccounttokens',
    url: '/service-account-tokens',
    component: ServiceAccountTokensPage,
    data: {
      title: PreviewRouteTitles.SERVICE_ACCOUNT_TOKENS,
      visibilityRequirements: {
        requiresPermission: Permissions.SERVICE_ACCOUNTS.READ,
        editions: ['PRO'],
        statesEnabled: [{ key: 'serviceAccountEnabled', defaultValue: false }],
      },
    },
  },
  {
    name: 'preview.admin.security.saml',
    url: '/saml',
    component: SamlPage,
    data: {
      title: PreviewRouteTitles.SAML,
      visibilityRequirements: {
        // ASTRUM-54019: match the retired Classic page. SAML is admin-only
        // (astrum:*) AND only available with the saml bundle on PRO — without the bundle+editions
        // gates the tile shows where the feature does not exist and then dead-ends.
        bundle: 'astrum-saml-plugin',
        requiresPermission: Permissions.ADMIN,
        editions: ['PRO'],
      },
    },
  },
  {
    name: 'preview.admin.security.oauth2',
    url: '/oauth2',
    component: OAuth2Page,
    data: {
      title: PreviewRouteTitles.OPENID_CONNECT,
      visibilityRequirements: {
        requiresPermission: Permissions.ADMIN,
        statesEnabled: [{ key: 'oauth2Available', defaultValue: false }],
      },
    },
  },
  {
    name: 'preview.admin.security.crowd',
    url: '/crowd',
    component: CrowdPage,
    data: {
      title: PreviewRouteTitles.ATLASSIAN_CROWD,
      visibilityRequirements: {
        // ASTRUM-54019: match the retired Classic page. Crowd is
        // only available with the crowd bundle and a valid crowd license — without the
        // bundle+licenseValid gates the tile shows where the feature does not exist and dead-ends.
        bundle: 'astrum-crowd-plugin',
        licenseValid: [{ key: 'crowd', defaultValue: false }],
        permissions: ['astrum:crowd:read'],
      },
    },
  },

  // =============================================================================
  // SUPPORT SECTION
  // Names match: admin.support.* → preview.admin.support.*
  // =============================================================================
  {
    name: 'preview.admin.support',
    url: '/support',
    redirectTo: 'preview.admin.support.logs.list',
    component: UIView,
    data: { title: PreviewRouteTitles.SUPPORT },
  },
  {
    name: 'preview.admin.support.logs',
    url: '/logs',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.LOGS,
      visibilityRequirements: {
        requiresPermission: Permissions.LOGGING.READ,
      },
    },
  },
  {
    name: 'preview.admin.support.logs.list',
    url: '',
    component: LogsPage,
    data: {
      title: PreviewRouteTitles.LOGS,
    },
  },
  {
    name: 'preview.admin.support.logs.detail',
    url: '/*filename',
    component: LogsPage,
    params: {
      // raw: true preserves '/' in task log paths (e.g. tasks/foo.log)
      // without it UIRouter would treat slashes as segment separators
      filename: {value: null, raw: true, dynamic: true},
    },
    data: {
      title: PreviewRouteTitles.LOG_VIEWER,
    },
  },
  {
    name: 'preview.admin.support.logging',
    url: '/logging',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.LOGGING,
      visibilityRequirements: {
        requiresPermission: Permissions.LOGGING.READ,
      },
    },
  },
  {
    name: 'preview.admin.support.logging.list',
    url: '',
    component: LoggingConfigPage,
    data: {
      title: PreviewRouteTitles.LOGGING,
      visibilityRequirements: {
        requiresPermission: Permissions.LOGGING.READ,
      },
    },
  },
  {
    name: 'preview.admin.support.logging.create',
    url: '/create',
    component: LoggingConfigPage,
    data: {
      title: PreviewRouteTitles.CREATE_LOGGER,
      visibilityRequirements: {
        requiresPermission: Permissions.LOGGING.UPDATE,
      },
    },
  },
  {
    name: 'preview.admin.support.logging.detail',
    url: '/:itemId',
    component: LoggingConfigPage,
    data: {
      title: PreviewRouteTitles.LOGGER_DETAILS,
      visibilityRequirements: {
        requiresPermission: Permissions.LOGGING.READ,
      },
    },
  },
  {
    name: 'preview.admin.support.systeminformation',
    url: '/systeminformation',
    component: SystemInfoPage,
    data: {
      title: PreviewRouteTitles.SYSTEM_INFORMATION,
      visibilityRequirements: {
        requiresPermission: 'astrum:atlas:read',
      },
    },
  },
  {
    name: 'preview.admin.support.metrichealth',
    url: '/metrichealth',
    component: MetricHealthPage,
    data: {
      title: PreviewRouteTitles.METRIC_HEALTH,
      visibilityRequirements: {
        requiresPermission: 'astrum:metrics:read',
      },
    },
  },
  {
    name: 'preview.admin.support.recoverymode',
    url: '/recoverymode',
    component: RecoveryModePage,
    data: {
      title: PreviewRouteTitles.RECOVERY_MODE,
      visibilityRequirements: {
        permissions: [Permissions.ADMIN],
        // Recovery mode comes with an optional plugin; without it the page has no server to talk to
        statesEnabled: [{ key: 'recoveryModeAvailable', defaultValue: false }],
      },
    },
  },
  {
    name: 'preview.admin.support.supportrequest',
    url: '/supportrequest',
    component: SupportRequestPage,
    data: {
      title: PreviewRouteTitles.SUPPORT_REQUEST,
      // ASTRUM-54019: match the retired Classic page — atlas:create
      // gated to PRO. Preview previously gated on atlas:read, which showed the tile to users who
      // lack the create permission the page actually requires.
      visibilityRequirements: {
        requiresPermission: Permissions.ATLAS.CREATE,
        editions: ['PRO'],
      },
    },
  },
  {
    name: 'preview.admin.support.supportzip',
    url: '/supportzip',
    component: SupportZipPage,
    data: {
      title: PreviewRouteTitles.SUPPORT_ZIP,
      // ASTRUM-54019: match the retired Classic page — atlas:read.
      // Preview previously gated on the non-existent astrum:supportzip:read privilege.
      visibilityRequirements: {
        requiresPermission: Permissions.ATLAS.READ,
      },
    },
  },

  // =============================================================================
  // SYSTEM SECTION
  // Names match: admin.system.* → preview.admin.system.*
  // =============================================================================
  {
    name: 'preview.admin.system',
    url: '/system',
    redirectTo: 'preview.admin.system.tasks.list',
    component: UIView,
    data: { title: PreviewRouteTitles.SYSTEM },
  },
  {
    name: 'preview.admin.system.tasks',
    url: '/tasks',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.TASKS,
      visibilityRequirements: {
        requiresPermission: Permissions.TASKS.READ,
      },
    },
  },
  {
    name: 'preview.admin.system.tasks.list',
    url: '',
    component: TasksPage,
    data: { title: PreviewRouteTitles.TASKS },
  },
  {
    name: 'preview.admin.system.tasks.create',
    url: '/create',
    component: TasksPage,
    data: { title: PreviewRouteTitles.CREATE_TASK },
  },
  {
    name: 'preview.admin.system.tasks.createWithType',
    url: '/create/:typeId',
    component: TasksPage,
    data: { title: PreviewRouteTitles.CREATE_TASK },
  },
  {
    name: 'preview.admin.system.tasks.detail',
    url: '/:taskId',
    component: TasksPage,
    data: { title: PreviewRouteTitles.TASK_DETAILS },
  },
  {
    name: 'preview.admin.system.capabilities',
    url: '/capabilities',
    abstract: true,
    component: UIView,
    data: {
      title: PreviewRouteTitles.CAPABILITIES,
      visibilityRequirements: {
        requiresPermission: Permissions.CAPABILITIES.READ,
      },
    },
  },
  {
    name: 'preview.admin.system.capabilities.list',
    url: '',
    component: CapabilitiesPage,
    data: { title: PreviewRouteTitles.CAPABILITIES },
  },
  {
    name: 'preview.admin.system.capabilities.create',
    url: '/create',
    component: CapabilitiesPage,
    data: { title: PreviewRouteTitles.CREATE_CAPABILITY },
  },
  {
    name: 'preview.admin.system.capabilities.createWithType',
    url: '/create/:typeId',
    component: CapabilitiesPage,
    params: { typeId: { type: 'string', value: null, dynamic: true } },
    data: { title: PreviewRouteTitles.CREATE_CAPABILITY },
  },
  {
    name: 'preview.admin.system.capabilities.detail',
    url: '/:capabilityId',
    component: CapabilitiesPage,
    params: { capabilityId: { type: 'string', value: null, dynamic: true } },
    data: { title: PreviewRouteTitles.CAPABILITY_DETAILS },
  },
  {
    name: 'preview.admin.system.notifications',
    url: '/notifications',
    component: NotificationsPage,
    data: {
      title: PreviewRouteTitles.NOTIFICATIONS,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },
  {
    name: 'preview.admin.system.emailserver',
    url: '/emailserver',
    component: EmailPage,
    data: {
      title: PreviewRouteTitles.EMAIL_SERVER,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },
  {
    name: 'preview.admin.system.http',
    url: '/http',
    component: HttpPage,
    data: {
      title: PreviewRouteTitles.HTTP,
      visibilityRequirements: {
        requiresPermission: Permissions.SETTINGS.READ,
      },
    },
  },
  {
    name: 'preview.admin.system.nodes',
    url: '/nodes',
    component: NodesPage,
    data: {
      title: PreviewRouteTitles.NODES,
      visibilityRequirements: {
        // ASTRUM-54048: match the retired Classic page, which gates on
        // astrum:* — the Nodes screen is an admin-only screen.
        requiresPermission: Permissions.ADMIN,
      },
    },
  },
  {
    name: 'preview.admin.system.usage',
    url: '/usage',
    component: UsagePage,
    data: {
      title: PreviewRouteTitles.USAGE,
      // ASTRUM-54048: Usage is a cloud-oriented feature — its Settings Hub card is cloudOnly, so
      // this route is NOT surfaced in the self-hosted Settings menu (the card is filtered out).
      // The gate here is therefore dormant on self-hosted. The value differs from the cloud route
      // (settings:read) and the shared card (settings:read); note the underlying monthly-/daily-
      // metrics APIs actually require astrum:*, so none of these frontend gates is backend-exact.
      // Left as METRICS.READ (no behaviour change) since the route is not user-reachable here;
      // realigning the metrics gates to astrum:* is tracked separately, out of this ticket's scope.
      visibilityRequirements: {
        permissions: [Permissions.METRICS.READ],
      },
    },
  },

  // =============================================================================
  // GLOBALART INTERNAL TEST PAGES — now registered at preview.test* level
  // (standalone, outside SettingsPageLayoutRadix) via globalartInternalTestRoutes
  // export in routerConfig.js — removed from here (xtrz).
  //
  // Redirect stubs so bookmarked #preview/admin/test* URLs still work.
  // =============================================================================
  {
    name: 'preview.admin.test',
    url: '/test',
    redirectTo: 'preview.test',
  },
  {
    name: 'preview.admin.test-dashboard',
    url: '/test-dashboard',
    redirectTo: 'preview.test-dashboard',
  },
  {
    name: 'preview.admin.test-search-hc',
    url: '/test-search-hc',
    redirectTo: 'preview.test-search-hc',
  },

  // =============================================================================
  // USER ACCOUNT — redirect to standalone preview.user.account (outside Settings layout)
  // =============================================================================
  {
    name: 'preview.admin.useraccount',
    url: '/useraccount',
    // Redirect so old/bookmarked links land on the standalone page, not Settings
    redirectTo: 'preview.user.account',
    data: { title: PreviewRouteTitles.USER_ACCOUNT },
  },
];
