
import {UIStrings} from '@globalart/astrum-ui-plugin';
import localize from '@globalart/astrum-ui-plugin/src/frontend/src/interface/localize';
import AnonymousAccessStrings from './pages/admin/security/AnonymousAccessStrings';
import RealmsStrings from './pages/admin/security/RealmsStrings';
import PrivilegesStrings from './pages/admin/security/PrivilegesStrings';
import RolesStrings from './pages/admin/security/RolesStrings';
import UsersStrings from './pages/admin/security/UsersStrings';
import SslCertificatesStrings from './pages/admin/security/SslCertificatesStrings';
import LdapStrings from './pages/admin/security/LdapStrings';
import EmailServerStrings from './pages/admin/system/EmailServerStrings';
import BlobStoresStrings from './pages/admin/repository/BlobStoresStrings';
import ContentSelectorsStrings from './pages/admin/repository/ContentSelectorsStrings';
import LoggingStrings from './pages/admin/support/LoggingStrings';
import LogsStrings from './pages/admin/support/LogsStrings';
import RecoveryModeStrings from './pages/admin/support/RecoveryModeStrings';
import RepositoriesStrings from './pages/admin/repository/RepositoriesStrings';
import RoutingRulesStrings from './pages/admin/repository/RoutingRulesStrings';
import Log4jVisualizerStrings from './pages/browse/Log4jVisualizerStrings';
import SystemInformationStrings from './pages/admin/system/SystemInformationStrings';
import SupportRequestStrings from './pages/admin/support/SupportRequestStrings';
import AccountStrings from './pages/user/AccountStrings';
import NuGetApiKeyStrings from './pages/user/NuGetApiKeyStrings';
import StatusStrings from './pages/admin/support/StatusStrings';
import SupportZipStrings from './pages/admin/support/SupportZipStrings';
import ApiStrings from './pages/admin/system/ApiStrings';
import ProprietaryRepositoriesStrings from './pages/admin/repository/ProprietaryRepositoriesStrings';
import HttpStrings from './pages/admin/system/HttpStrings';
import AtlassianCrowdStrings from './pages/admin/security/AtlassianCrowdStrings';
import SamlStrings from './pages/admin/security/SamlStrings';
import DataStoreStrings from './pages/admin/repository/DataStoreStrings';
import ReplicationStrings from './pages/admin/repository/ReplicationStrings';
import UserTokenConfigurationStrings from './pages/admin/security/UserTokenStrings';
import ServiceAccountTokensStrings from './pages/admin/security/ServiceAccountTokensStrings';
import UserTokenStrings from './pages/user/UserTokenStrings';
import WelcomeStrings from './pages/user/WelcomeStrings';
import TagsStrings from './pages/browse/tags/TagsStrings';
import UploadStrings from './pages/browse/upload/UploadStrings';
import NodesStrings from './pages/admin/system/NodesStrings';
import BrowseStrings from './pages/browse/browse/BrowseStrings';
import UpgradeAlertStrings from './UpgradeAlertStrings';
import UpgradeTriggerModalStrings from './UpgradeTriggerModalStrings';
import RecoveryModeAlertStrings from './RecoveryModeAlertStrings';
import SearchStrings from './pages/browse/search/SearchStrings';
import PreviewUiStrings from "./pages/admin/system/PreviewUiStrings";
import RepositoryDirectoryStrings from './pages/admin/repository/RepositoryDirectoryStrings';
import AdminSecurityDirectoryStrings from './pages/admin/repository/AdminSecurityDirectoryStrings';
import AdminSystemDirectoryStrings from './pages/admin/system/AdminSystemDirectoryStrings';
import AdminSupportDirectoryStrings from './pages/admin/system/AdminSupportDirectoryStrings';
import AdminDirectoryStrings from './pages/admin/system/AdminDirectoryStrings';
import OAuth2Strings from "./pages/admin/security/OAuth2Strings";
import TelemetryStrings from './pages/telemetry/TelemetryStrings';
import SettingsHubStrings from './pages/admin/SettingsHubStrings';
import ShellStrings from './ShellStrings';
import PageStrings from './PageStrings';

export default {
  ...UIStrings,
  ...localize('react-coreui', {

    // welcome page
    ...WelcomeStrings,

    // browse
    ...Log4jVisualizerStrings,
    ...TagsStrings,
    ...BrowseStrings,
    ...UploadStrings,
    ...SearchStrings,

    // admin

    // admin/repository
    ...BlobStoresStrings,
    ...ContentSelectorsStrings,
    ...DataStoreStrings,
    ...ProprietaryRepositoriesStrings,
    ...RepositoriesStrings,
    ...ReplicationStrings,
    ...RoutingRulesStrings,

    // admin/security
    ...AnonymousAccessStrings,
    ...AtlassianCrowdStrings,
    ...LdapStrings,
    ...OAuth2Strings,
    ...PrivilegesStrings,
    ...RealmsStrings,
    ...RolesStrings,
    ...SamlStrings,
    ...ServiceAccountTokensStrings,
    ...SslCertificatesStrings,
    ...UsersStrings,
    ...UserTokenConfigurationStrings,
    ...ServiceAccountTokensStrings,

    // admin/support
    ...LoggingStrings,
    ...LogsStrings,
    ...RecoveryModeStrings,
    ...StatusStrings,
    ...SupportRequestStrings,
    ...SupportZipStrings,

    // admin/system
    ...ApiStrings,
    ...EmailServerStrings,
    ...HttpStrings,
    ...NodesStrings,
    ...PreviewUiStrings,
    ...SystemInformationStrings,

    // admin/riskvisualizer

    // telemetry
    ...TelemetryStrings,

    // user
    ...AccountStrings,
    ...NuGetApiKeyStrings,
    ...UserTokenStrings,
    ...UpgradeAlertStrings,
    ...UpgradeTriggerModalStrings,
    ...RecoveryModeAlertStrings,

    ...RepositoryDirectoryStrings,
    ...AdminSecurityDirectoryStrings,
    ...AdminSystemDirectoryStrings,
    ...AdminSupportDirectoryStrings,
    ...AdminDirectoryStrings,
    ...SettingsHubStrings,
    ...ShellStrings,
    ...PageStrings,

    // other
    FORMAT_PLACEHOLDER: 'Format',

    HEALTHCHECK_EULA: {
      HEADER: 'Astrum IQ Server Terms of Use',
      BUTTONS: {
        ACCEPT: 'I accept',
        DECLINE: 'I do not accept'
      }
    },

    ANALYZE_APPLICATION: {
      HEADER: 'Analyze Application',
      MAIN: 'Application analysis performs a deep inspection of this application, identifying potential risks.<br/> More information is available here <a href=\"http://links.globalart.dev/products/insight/ac/home\">here</a>',
      EMAIL: {
        LABEL: 'Email address',
        DESCRIPTION: 'The address where the summary report will be sent'
      },
      PASSWORD: {
        LABEL: 'Report password',
        DESCRIPTION: 'A password to gain access to the detailed report'
      },
      PACKAGES: {
        LABEL: 'Proprietary packages',
        DESCRIPTION: 'A comma separated list of proprietary packages'
      },
      REPORT: {
        LABEL: 'Report label',
        DESCRIPTION: 'The name the report will be given'
      },
      SELECT_ASSET: {
        LABEL: 'Select Asset',
        DESCRIPTION: 'Select an asset to base the analysis on'
      },
      BUTTONS: {
        ANALYZE: 'Analyze',
        CANCEL: 'Cancel'
      }
    }
  })
};
