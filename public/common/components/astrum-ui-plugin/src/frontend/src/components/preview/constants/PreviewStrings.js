/**
 * The strings modules of Astrum One. Components import the module of their page directly; this list is what
 * tools/i18n reads the English texts from and what the tests of the translations load.
 */
export { default as CommonStrings } from './CommonStrings';
export { default as SharedStrings } from './SharedStrings';
export { default as SecurityReportStrings } from './SecurityReportStrings';
export { default as SettingsStrings } from './pages/settings/SettingsStrings';
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
