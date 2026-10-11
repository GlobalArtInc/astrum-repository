/**
 * The strings modules of Astrum One. Components import the module of their page directly; this list is what
 * tools/i18n reads the English texts from and what the tests of the translations load.
 */
export { default as CommonStrings } from './CommonStrings';
export { default as SharedStrings } from './SharedStrings';
export { default as SecurityReportStrings } from './SecurityReportStrings';
export { default as SettingsStrings } from './pages/settings/SettingsStrings';
export { default as LoggingStrings } from './pages/settings/support/LoggingStrings';
export { default as LogsStrings } from './pages/settings/support/LogsStrings';
export { default as MetricHealthStrings } from './pages/settings/support/MetricHealthStrings';
export { default as RecoveryModeStrings } from './pages/settings/support/RecoveryModeStrings';
export { default as SupportRequestStrings } from './pages/settings/support/SupportRequestStrings';
export { default as SupportZipStrings } from './pages/settings/support/SupportZipStrings';
export { default as SystemInfoStrings } from './pages/settings/support/SystemInfoStrings';
export { default as BlobStoresStrings } from './pages/settings/repository/BlobStoresStrings';
export { default as CleanupPoliciesStrings } from './pages/settings/repository/CleanupPoliciesStrings';
export { default as ContentSelectorsStrings } from './pages/settings/repository/ContentSelectorsStrings';
export { default as DataStoreStrings } from './pages/settings/repository/DataStoreStrings';
export { default as ProprietaryStrings } from './pages/settings/repository/ProprietaryStrings';
export { default as RepositoriesPageStrings } from './pages/settings/repository/RepositoriesPageStrings';
export { default as RepositoryFacetsStrings } from '../../../constants/pages/admin/repository/RepositoriesStrings';
export { default as RepositoryProfileStrings } from './pages/settings/repository/RepositoryProfileStrings';
export { default as RoutingRulesStrings } from './pages/settings/repository/RoutingRulesStrings';
export { default as AnonymousStrings } from './pages/settings/security/AnonymousStrings';
export { default as CrowdStrings } from './pages/settings/security/CrowdStrings';
export { default as LdapStrings } from './pages/settings/security/LdapStrings';
export { default as OAuth2Strings } from './pages/settings/security/OAuth2Strings';
export { default as PrivilegesStrings } from './pages/settings/security/PrivilegesStrings';
export { default as RealmsStrings } from './pages/settings/security/RealmsStrings';
export { default as RolesStrings } from './pages/settings/security/RolesStrings';
export { default as SamlStrings } from './pages/settings/security/SamlStrings';
export { default as SecurityStrings } from './pages/settings/security/SecurityStrings';
export { default as SslCertificatesStrings } from './pages/settings/security/SslCertificatesStrings';
export { default as UserTokensStrings } from './pages/settings/security/UserTokensStrings';
export { default as UsersStrings } from './pages/settings/security/UsersStrings';
export { SERVICE_ACCOUNT_TOKENS_STRINGS } from '../pages/settings/security/service-account-tokens/strings';
export { default as TasksStrings } from './pages/settings/system/TasksStrings';
export { TASK_TYPE_SELECTOR, DYNAMIC_FORM_FIELDS } from '../pages/settings/system/tasks/TaskStrings';
